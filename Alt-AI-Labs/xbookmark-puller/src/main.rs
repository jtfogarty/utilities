//! X bookmark archiver: **xapimcp** MCP tools `get_my_bookmarks` + `delete_bookmark` only (no direct X HTTP),
//! dedupe + `create` via SurrealMCP (`query` / `create` tools).

/*
# X API Rate Limit Handling

This binary is the **only** place that enforces pacing for X traffic. `xapimcp` stays a thin,
stateless MCP wrapper over the X API — **no** rate-limit logic lives there.

## Strategy (governor, 2026-03 refresh)

- Two independent **GCRA** limiters (`governor::RateLimiter` + `StateInformationMiddleware`):
  - **GET** `get_my_bookmarks` → **180** cells / **15 min** window (`Quota::new(180, 15 min)`).
  - **DELETE** `delete_bookmark` → **50** cells / **15 min** window (`Quota::new(50, 15 min)`).
- Before each MCP call, the client **waits** until the matching limiter grants a cell. Waits use
  `NotUntil::wait_time_from(Instant::now())` plus **±10%** jitter (via `rand`) on the sleep duration
  to reduce thundering herds.
- After each granted cell, we log **remaining burst capacity** and quota timing at **debug**
  (`tracing::debug!`, JSON subscriber).
- **429 fallback**: if X still returns a **documented** rate-limit error in the JSON body after
  governor pacing, we retry a few times with a short sleep. **Non–rate-limit 429** responses are
  **fatal** (distinct from “too many requests / rate limit” payloads).
- **Large delete batches**: after **800** successful deletes in one pull cycle, we **sleep until the
  next 15-minute UTC wall-clock boundary** so the next cycle aligns with a fresh window.
- **`RATE_LIMIT_ENFORCED`**: when `false`, governor waits are skipped (still logs a warning); 429
  fallback remains.
- **`--dry-run`**: still acquires governor slots and runs Surreal inserts, but **skips** the delete
  MCP call.

## Changes since original (2026-03)

- Replaced ad-hoc exponential backoff + fixed inter-page sleep with **governor** + wall-clock batch
  alignment for predictable 800-bookmark workflows.
- Split GET vs DELETE quotas; added jittered waits, stricter 429 classification, dry-run, and
  optional enforcement bypass for emergencies.

*/

use std::borrow::Cow;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::Parser;
use governor::clock::{Clock, DefaultClock};
use governor::middleware::StateInformationMiddleware;
use governor::middleware::StateSnapshot;
use governor::{Quota, RateLimiter};
use rand::Rng;
use rmcp::model::{CallToolRequestParams, CallToolResult, JsonObject, RawContent};
use rmcp::service::ServiceExt;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use serde_json::{Value, json};
use tokio::sync::watch;
use tracing::{debug, error, info, warn};

type ClientPeer = rmcp::service::Peer<rmcp::service::RoleClient>;
type RunningClient = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

type InfoDirectRateLimiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
    StateInformationMiddleware,
>;

#[derive(Parser, Debug)]
#[command(name = "xbookmark-puller")]
struct Cli {
    /// Store via SurrealMCP but do not call xapimcp `delete_bookmark` (timing and governor still apply).
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone)]
struct Config {
    surreal_mcp_url: String,
    surreal_mcp_auth_token: Option<String>,
    x_mcp_url: String,
    x_mcp_auth_token: Option<String>,
    pull_interval: Duration,
    rate_limit_enforced: bool,
    dry_run: bool,
}

impl Config {
    fn from_env(dry_run: bool) -> Result<Self> {
        let pull_minutes: u64 = std::env::var("PULL_INTERVAL_MINUTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);
        let surreal_mcp_url = std::env::var("SURREAL_MCP_URL")
            .context("SURREAL_MCP_URL must be set (SurrealMCP streamable HTTP, e.g. http://127.0.0.1:8080/mcp)")?;
        let x_mcp_url = std::env::var("X_MCP_URL")
            .context("X_MCP_URL must be set (xapimcp streamable HTTP endpoint, e.g. http://127.0.0.1:8090/mcp)")?;

        let rate_limit_enforced = match std::env::var("RATE_LIMIT_ENFORCED") {
            Ok(s) => !matches!(s.to_lowercase().as_str(), "0" | "false" | "no" | "off"),
            Err(_) => true,
        };

        Ok(Self {
            surreal_mcp_url,
            surreal_mcp_auth_token: std::env::var("SURREAL_MCP_AUTH_TOKEN").ok(),
            x_mcp_url,
            x_mcp_auth_token: std::env::var("X_MCP_AUTH_TOKEN").ok(),
            pull_interval: Duration::from_secs(pull_minutes.max(1).saturating_mul(60)),
            rate_limit_enforced,
            dry_run,
        })
    }
}

/// Two X endpoint quotas + enforcement flag (all client-side).
struct RateLimitTracker {
    clock: Arc<DefaultClock>,
    get: Arc<InfoDirectRateLimiter>,
    delete: Arc<InfoDirectRateLimiter>,
    enforced: bool,
}

impl RateLimitTracker {
    fn new(enforced: bool) -> Self {
        // 180 GET / 15 min ⇒ 1 cell / 5 s, burst 180. 50 DELETE / 15 min ⇒ 1 cell / 18 s, burst 50.
        let get_quota = Quota::with_period(Duration::from_secs(5))
            .expect("GET quota period")
            .allow_burst(NonZeroU32::new(180).expect("180"));
        let del_quota = Quota::with_period(Duration::from_secs(18))
            .expect("DELETE quota period")
            .allow_burst(NonZeroU32::new(50).expect("50"));

        let clock = Arc::new(DefaultClock::default());
        let get = Arc::new(
            RateLimiter::direct_with_clock(get_quota, clock.as_ref())
                .with_middleware::<StateInformationMiddleware>(),
        );
        let delete = Arc::new(
            RateLimiter::direct_with_clock(del_quota, clock.as_ref())
                .with_middleware::<StateInformationMiddleware>(),
        );

        Self {
            clock,
            get,
            delete,
            enforced,
        }
    }

    async fn acquire_get(&self, label: &'static str) -> Option<StateSnapshot> {
        self.acquire(self.get.as_ref(), label).await
    }

    async fn acquire_delete(&self, label: &'static str) -> Option<StateSnapshot> {
        self.acquire(self.delete.as_ref(), label).await
    }

    async fn acquire(
        &self,
        limiter: &InfoDirectRateLimiter,
        label: &'static str,
    ) -> Option<StateSnapshot> {
        if !self.enforced {
            warn!(label, "RATE_LIMIT_ENFORCED=false; skipping governor wait for this call");
            return None;
        }
        loop {
            let now = self.clock.now();
            match limiter.check() {
                Ok(snapshot) => {
                    let q = snapshot.quota();
                    debug!(
                        label,
                        remaining = snapshot.remaining_burst_capacity(),
                        burst = q.burst_size().get(),
                        replenish_interval_secs = q.replenish_interval().as_secs(),
                        full_refill_in_secs = q.burst_size_replenished_in().as_secs(),
                        "governor granted cell"
                    );
                    return Some(snapshot);
                }
                Err(not_until) => {
                    let base = not_until.wait_time_from(now);
                    let sleep_d = jitter_duration(base, 0.10);
                    debug!(
                        label,
                        wait_base_ms = base.as_millis(),
                        sleep_ms = sleep_d.as_millis(),
                        "governor wait before next cell"
                    );
                    tokio::time::sleep(sleep_d).await;
                }
            }
        }
    }
}

fn jitter_duration(base: Duration, pct: f64) -> Duration {
    let mut rng = rand::thread_rng();
    let factor = 1.0 + rng.gen_range(-pct..=pct);
    let secs = (base.as_secs_f64() * factor).max(1e-6);
    Duration::from_secs_f64(secs)
}

fn duration_until_next_15min_boundary() -> Duration {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();
    let period = 15 * 60;
    let next = ((now / period) + 1) * period;
    Duration::from_secs(next.saturating_sub(now))
}

async fn connect_streamable_mcp(
    uri: String,
    auth_token: Option<String>,
    label: &'static str,
) -> Result<RunningClient> {
    let mut tcfg = StreamableHttpClientTransportConfig::with_uri(uri);
    if let Some(tok) = auth_token {
        tcfg = tcfg.auth_header(tok);
    }
    let transport = StreamableHttpClientTransport::from_config(tcfg);
    ()
        .serve(transport)
        .await
        .map_err(|e| anyhow!("{label} MCP connect: {e}"))
}

fn mcp_tool_text(res: &CallToolResult) -> String {
    res.content
        .iter()
        .filter_map(|block| {
            let raw: &RawContent = block;
            match raw {
                RawContent::Text(t) => Some(t.text.clone()),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn mcp_ensure_ok_surreal(res: &CallToolResult) -> Result<()> {
    if res.is_error == Some(true) {
        bail!("SurrealMCP tool error: {}", mcp_tool_text(res));
    }
    Ok(())
}

fn parse_count_from_json_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64(),
        Value::Array(items) => items.iter().find_map(parse_count_from_json_value),
        Value::Object(map) => {
            if let Some(n) = map.get("count").and_then(parse_count_from_json_value) {
                return Some(n);
            }
            if let Some(n) = map.get("c").and_then(parse_count_from_json_value) {
                return Some(n);
            }
            map.values().find_map(parse_count_from_json_value)
        }
        Value::String(s) => parse_count_from_surreal_debug(s),
        _ => None,
    }
}

fn parse_count_from_surreal_debug(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        if let Some(n) = parse_count_from_json_value(&v) {
            return Some(n);
        }
    }

    for key in ["\"count\": Number(Int(", "count: Number(Int(", "\"c\": Number(Int(", "c: Number(Int("] {
        if let Some(idx) = trimmed.find(key) {
            let start = idx + key.len();
            if let Some(end_rel) = trimmed[start..].find(')') {
                let digits = &trimmed[start..start + end_rel];
                if let Ok(n) = digits.parse::<u64>() {
                    return Some(n);
                }
            }
        }
    }

    if let Some(idx) = trimmed.find("Number(Int(") {
        let start = idx + "Number(Int(".len();
        if let Some(end_rel) = trimmed[start..].find(')') {
            let digits = &trimmed[start..start + end_rel];
            if let Ok(n) = digits.parse::<u64>() {
                return Some(n);
            }
        }
    }

    if let Some(idx) = trimmed.find("Number(Float(") {
        let start = idx + "Number(Float(".len();
        if let Some(end_rel) = trimmed[start..].find(')') {
            let digits = &trimmed[start..start + end_rel];
            if let Ok(n) = digits.parse::<f64>() {
                return Some(n as u64);
            }
        }
    }

    None
}

async fn surreal_query_exists(surreal: &ClientPeer, bookmark_id: &str) -> Result<bool> {
    let mut args = JsonObject::new();
    args.insert(
        "query".to_string(),
        json!("SELECT count() FROM x_bookmarks WHERE bookmark_id = $bid"),
    );
    let mut params = JsonObject::new();
    params.insert("bid".to_string(), json!(bookmark_id));
    args.insert("parameters".to_string(), json!(params));

    let res = surreal
        .call_tool(CallToolRequestParams {
            meta: None,
            name: Cow::Borrowed("query"),
            arguments: Some(args),
            task: None,
        })
        .await
        .map_err(|e| anyhow!("SurrealMCP query tool: {e}"))?;

    mcp_ensure_ok_surreal(&res).context("SurrealMCP query result")?;
    let text = mcp_tool_text(&res);
    let Some(count) = parse_count_from_surreal_debug(&text) else {
        warn!(
            bookmark_id,
            output_len = text.len(),
            output = text,
            "failed to parse Surreal count() response; treating as non-existent"
        );
        return Ok(false);
    };
    Ok(count > 0)
}

async fn surreal_create_bookmark(surreal: &ClientPeer, record: &JsonObject) -> Result<()> {
    let mut args = JsonObject::new();
    args.insert("target".to_string(), json!("x_bookmarks"));
    args.insert("data".to_string(), Value::Object(record.clone()));

    let res = surreal
        .call_tool(CallToolRequestParams {
            meta: None,
            name: Cow::Borrowed("create"),
            arguments: Some(args),
            task: None,
        })
        .await
        .map_err(|e| anyhow!("SurrealMCP create tool: {e}"))?;

    mcp_ensure_ok_surreal(&res).context("SurrealMCP create result")?;
    Ok(())
}

fn tweet_author_label(tweet: &Value, includes: Option<&Value>) -> String {
    let aid = tweet
        .get("author_id")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    if let Some(users) = includes.and_then(|i| i.get("users")).and_then(|u| u.as_array()) {
        for u in users {
            if u.get("id").and_then(|x| x.as_str()) != Some(aid) {
                continue;
            }
            let un = u.get("username").and_then(|x| x.as_str()).unwrap_or(aid);
            let nm = u.get("name").and_then(|x| x.as_str()).unwrap_or("");
            return format!("@{un} ({nm})");
        }
    }
    aid.to_string()
}

/// True when X marks this error entry as a **rate-limit** style 429 (retry / fallback OK).
fn x_error_is_documented_rate_limit(e: &Value) -> bool {
    let status = e.get("status").and_then(|s| s.as_u64()).unwrap_or(0);
    if status != 429 {
        return false;
    }
    if e.get("code").and_then(|c| c.as_u64()) == Some(88) {
        return true;
    }
    let title = e
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    let detail = e
        .get("detail")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    let msg = e
        .get("message")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    title.contains("too many requests")
        && (detail.contains("rate") || detail.contains("limit") || msg.contains("rate limit"))
}

fn x_errors_fatal_non_rate_limit_429(errs: &[Value]) -> Option<String> {
    for e in errs {
        if x_error_is_documented_rate_limit(e) {
            continue;
        }
        if e.get("status").and_then(|s| s.as_u64()) == Some(429) {
            return Some(format!("fatal 429 (not classified as X rate limit): {e}"));
        }
    }
    None
}

fn x_errors_all_documented_rate_limits(errs: &[Value]) -> bool {
    !errs.is_empty() && errs.iter().all(x_error_is_documented_rate_limit)
}

async fn x_get_bookmarks_page(
    x: &ClientPeer,
    pagination_token: Option<&str>,
    rl: &RateLimitTracker,
    is_deleting: &AtomicBool,
) -> Result<Value> {
    while is_deleting.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let mut attempt: u32 = 0;
    loop {
        rl.acquire_get("get_my_bookmarks").await;

        let mut args = JsonObject::new();
        if let Some(t) = pagination_token {
            args.insert("pagination_token".to_string(), json!(t));
        }

        let res = x
            .call_tool(CallToolRequestParams {
                meta: None,
                name: Cow::Borrowed("get_my_bookmarks"),
                arguments: Some(args),
                task: None,
            })
            .await
            .map_err(|e| anyhow!("xapimcp get_my_bookmarks: {e}"))?;

        if res.is_error == Some(true) {
            let msg = mcp_tool_text(&res);
            bail!("xapimcp get_my_bookmarks MCP error: {msg}");
        }

        let text = mcp_tool_text(&res).trim().to_string();
        let v: Value = serde_json::from_str(&text).context("parse xapimcp get_my_bookmarks JSON")?;

        if let Some(errs) = v.get("errors").and_then(|e| e.as_array()) {
            if !errs.is_empty() {
                if let Some(fatal) = x_errors_fatal_non_rate_limit_429(errs) {
                    bail!("X API (via xapimcp): {fatal}");
                }
                if x_errors_all_documented_rate_limits(errs) {
                    if attempt >= 5 {
                        bail!("X API rate limit (via xapimcp): exceeded fallback retries");
                    }
                    warn!(
                        attempt,
                        "429 rate-limit fallback after governor; short sleep and retry"
                    );
                    attempt = attempt.saturating_add(1);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                let first = &errs[0];
                let title = first.get("title").and_then(|t| t.as_str()).unwrap_or("error");
                let detail = first.get("detail").and_then(|t| t.as_str()).unwrap_or("");
                bail!("X API (via xapimcp): {title} — {detail}");
            }
        }

        return Ok(v);
    }
}

async fn x_delete_bookmark_mcp(
    x: &ClientPeer,
    tweet_id: &str,
    rl: &RateLimitTracker,
) -> Result<()> {
    let mut attempt: u32 = 0;
    loop {
        rl.acquire_delete("delete_bookmark").await;

        let mut args = JsonObject::new();
        args.insert("tweet_id".to_string(), json!(tweet_id));

        let res = x
            .call_tool(CallToolRequestParams {
                meta: None,
                name: Cow::Borrowed("delete_bookmark"),
                arguments: Some(args),
                task: None,
            })
            .await
            .map_err(|e| anyhow!("xapimcp delete_bookmark: {e}"))?;

        if res.is_error == Some(true) {
            let msg = mcp_tool_text(&res);
            bail!("xapimcp delete_bookmark MCP error: {msg}");
        }

        let text = mcp_tool_text(&res).trim().to_string();
        let v: Value = serde_json::from_str(&text).context("parse xapimcp delete_bookmark JSON")?;

        if let Some(errs) = v.get("errors").and_then(|e| e.as_array()) {
            if !errs.is_empty() {
                if let Some(fatal) = x_errors_fatal_non_rate_limit_429(errs) {
                    bail!("X delete (via xapimcp): {fatal}");
                }
                if x_errors_all_documented_rate_limits(errs) {
                    if attempt >= 5 {
                        bail!("X delete rate limit (via xapimcp): exceeded fallback retries");
                    }
                    warn!(
                        tweet_id,
                        attempt,
                        "429 rate-limit fallback on delete after governor; short sleep and retry"
                    );
                    attempt = attempt.saturating_add(1);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue;
                }
                let first = &errs[0];
                let title = first.get("title").and_then(|t| t.as_str()).unwrap_or("error");
                let detail = first.get("detail").and_then(|t| t.as_str()).unwrap_or("");
                bail!("X delete (via xapimcp): {title} — {detail}");
            }
        }

        return Ok(());
    }
}

fn backoff_secs(attempt: u32) -> u64 {
    (1u64 << attempt.min(8)).min(300)
}

async fn run_pull_cycle(
    surreal: &ClientPeer,
    x: &ClientPeer,
    rl: &RateLimitTracker,
    cfg: &Config,
    is_deleting: &AtomicBool,
) -> Result<()> {
    let mut token: Option<String> = None;
    let mut deletes_this_run: u32 = 0;

    loop {
        let page = x_get_bookmarks_page(x, token.as_deref(), rl, is_deleting).await?;
        let includes = page.get("includes");
        let tweets = page
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        for tweet in &tweets {
            let Some(bid) = tweet.get("id").and_then(|x| x.as_str()) else {
                continue;
            };
            let exists = surreal_query_exists(surreal, bid).await?;
            if exists {
                info!(
                    bookmark_id = bid,
                    "duplicate bookmark in Surreal (x_bookmarks); skipping store and delete"
                );
                continue;
            }

            let text = tweet
                .get("text")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let url = format!("https://x.com/i/web/status/{bid}");
            let author = tweet_author_label(tweet, includes);
            let created_at = tweet
                .get("created_at")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let raw_json = serde_json::to_string(tweet).unwrap_or_else(|_| "{}".to_string());
            let pulled_at = Utc::now().to_rfc3339();

            let mut data = JsonObject::new();
            data.insert("bookmark_id".to_string(), json!(bid));
            data.insert("text".to_string(), json!(text));
            data.insert("url".to_string(), json!(url));
            data.insert("author".to_string(), json!(author));
            data.insert("created_at".to_string(), json!(created_at));
            data.insert("raw_json".to_string(), json!(raw_json));
            data.insert("pulled_at".to_string(), json!(pulled_at));
            data.insert("processed".to_string(), json!(false));

            surreal_create_bookmark(surreal, &data).await?;
            info!(bookmark_id = bid, "stored new bookmark row in Surreal (x_bookmarks)");

            is_deleting.store(true, Ordering::SeqCst);
            let delete_outcome = async {
                if cfg.dry_run {
                    info!(
                        bookmark_id = bid,
                        "dry-run: would delete bookmark on X via xapimcp (delete_bookmark skipped)"
                    );
                    Ok(())
                } else {
                    x_delete_bookmark_mcp(x, bid, rl).await?;
                    info!(
                        bookmark_id = bid,
                        "removed bookmark from X account via xapimcp delete_bookmark"
                    );
                    Ok::<(), anyhow::Error>(())
                }
            }
            .await;
            is_deleting.store(false, Ordering::SeqCst);
            delete_outcome?;

            if !cfg.dry_run {
                deletes_this_run = deletes_this_run.saturating_add(1);
                if deletes_this_run >= 800 {
                    let wait = duration_until_next_15min_boundary();
                    info!(
                        wait_secs = wait.as_secs(),
                        deletes_this_run,
                        "800 deletes completed in this pull cycle; waiting for next 15-minute window boundary"
                    );
                    tokio::time::sleep(wait).await;
                    deletes_this_run = 0;
                }
            }
        }

        let next = page
            .get("meta")
            .and_then(|m| m.get("next_token"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        if next.is_none() {
            break;
        }
        token = next;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let cfg = Arc::new(Config::from_env(cli.dry_run)?);
    info!(
        pull_interval_secs = cfg.pull_interval.as_secs(),
        pull_interval_minutes = cfg.pull_interval.as_secs() / 60,
        surreal_mcp = %cfg.surreal_mcp_url,
        x_mcp = %cfg.x_mcp_url,
        rate_limit_enforced = cfg.rate_limit_enforced,
        dry_run = cfg.dry_run,
        "xbookmark-puller starting (MCP endpoints and scheduler)"
    );

    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut term = signal(SignalKind::terminate()).expect("SIGTERM");
            let mut intr = signal(SignalKind::interrupt()).expect("SIGINT");
            tokio::select! {
                _ = term.recv() => info!("SIGTERM"),
                _ = intr.recv() => info!("SIGINT"),
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
            info!("ctrl-c");
        }
        let _ = shutdown_tx.send(true);
    });

    let rate_limits = Arc::new(RateLimitTracker::new(cfg.rate_limit_enforced));

    let mut mcp_attempt: u32 = 0;
    let mut surreal_svc = connect_streamable_mcp(
        cfg.surreal_mcp_url.clone(),
        cfg.surreal_mcp_auth_token.clone(),
        "SurrealMCP",
    )
    .await?;
    let mut x_svc = connect_streamable_mcp(
        cfg.x_mcp_url.clone(),
        cfg.x_mcp_auth_token.clone(),
        "xapimcp",
    )
    .await?;

    let mut ticker = tokio::time::interval(cfg.pull_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let is_deleting = Arc::new(AtomicBool::new(false));

    loop {
        tokio::select! {
            biased;
            r = shutdown_rx.changed() => {
                if r.is_err() {
                    info!("shutdown channel closed");
                    let _ = surreal_svc.close().await;
                    let _ = x_svc.close().await;
                    break;
                }
                if *shutdown_rx.borrow() {
                    info!("graceful shutdown");
                    let _ = surreal_svc.close().await;
                    let _ = x_svc.close().await;
                    break;
                }
            }
            _ = ticker.tick() => {
                let surreal = surreal_svc.peer();
                let x = x_svc.peer();
                match run_pull_cycle(surreal, x, rate_limits.as_ref(), cfg.as_ref(), is_deleting.as_ref()).await {
                    Ok(()) => {
                        mcp_attempt = 0;
                        info!("pull cycle finished");
                    }
                    Err(e) => {
                        error!(error = %e, "pull cycle failed");
                        let wait = Duration::from_secs(backoff_secs(mcp_attempt));
                        tokio::time::sleep(wait).await;
                        mcp_attempt = mcp_attempt.saturating_add(1).min(12);
                        let surreal_res = connect_streamable_mcp(
                            cfg.surreal_mcp_url.clone(),
                            cfg.surreal_mcp_auth_token.clone(),
                            "SurrealMCP",
                        ).await;
                        let x_res = connect_streamable_mcp(
                            cfg.x_mcp_url.clone(),
                            cfg.x_mcp_auth_token.clone(),
                            "xapimcp",
                        ).await;
                        match (surreal_res, x_res) {
                            (Ok(new_s), Ok(new_x)) => {
                                let _ = surreal_svc.close().await;
                                let _ = x_svc.close().await;
                                surreal_svc = new_s;
                                x_svc = new_x;
                                info!("reconnected SurrealMCP and xapimcp");
                            }
                            (s_err, x_err) => {
                                if let Err(ref e) = s_err {
                                    error!(error = %e, "SurrealMCP reconnect failed");
                                }
                                if let Err(ref e) = x_err {
                                    error!(error = %e, "xapimcp reconnect failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
