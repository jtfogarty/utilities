//! X bookmark archiver: **xapimcp** MCP tools `get_my_bookmarks` + `delete_bookmark` only (no direct X HTTP),
//! dedupe + `create` via SurrealMCP (`query` / `create` tools).

/*
# X API Rate Limit Handling

This binary is the **only** place that enforces pacing for X traffic. `xapimcp` stays a thin,
stateless MCP wrapper over the X API — **no** rate-limit logic lives there.

## Strategy (governor, 2026-03 refresh)

- Two independent **GCRA** limiters (`governor::RateLimiter` + `StateInformationMiddleware`):
  - **GET** `get_my_bookmarks` → **180** cells / **15 min** window (`Quota::new(180, 15 min)`).
  - **DELETE** `delete_bookmark` → **50** cells / **15 min** window (`Quota::new(50, 15 min)`),
    burst capped at **5** to prevent exhausting the X API window on the first run.
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
- **`--dry-run`**: still acquires governor slots and **still runs Surreal `query` / `create`** (writes
  to the DB); it only **skips** the xapimcp `delete_bookmark` MCP call.

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
use tracing_subscriber::EnvFilter;

type ClientPeer = rmcp::service::Peer<rmcp::service::RoleClient>;
type RunningClient = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

type InfoDirectRateLimiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
    StateInformationMiddleware,
>;

/// Surreal table for archived bookmarks. Used for both `create` targets (`{table}:{id}`) and
/// `SELECT … FROM {table}` in `query` — keep a single source of truth.
const SURREAL_X_BOOKMARKS_TABLE: &str = "x_bookmarks";

/// `SELECT … FROM x_bookmarks` fails with "table does not exist" until the first `CREATE`; log once.
static SURREAL_TABLE_MISSING_LOGGED: AtomicBool = AtomicBool::new(false);

fn env_one_of(keys: &[&str], label: &str) -> Result<String> {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            let t = v.trim();
            if !t.is_empty() {
                return Ok(t.to_string());
            }
        }
    }
    bail!(
        "{label} must be set (non-empty). Looked at env vars: {}",
        keys.join(", ")
    );
}

#[derive(Parser, Debug)]
#[command(name = "xbookmark-puller")]
struct Cli {
    /// Store via SurrealMCP but do not call xapimcp `delete_bookmark` (timing and governor still apply).
    #[arg(long)]
    dry_run: bool,
    /// Connect to SurrealMCP only: `use_namespace` / `use_database`, insert one test row into `x_bookmarks`, verify with `SELECT count()`, then exit (does not use xapimcp).
    #[arg(long)]
    test_surreal_write: bool,
}

#[derive(Clone)]
struct Config {
    surreal_mcp_url: String,
    surreal_mcp_auth_token: Option<String>,
    surreal_namespace: String,
    surreal_database: String,
    x_mcp_url: String,
    x_mcp_auth_token: Option<String>,
    pull_interval: Duration,
    rate_limit_enforced: bool,
    dry_run: bool,
}

/// Minimal config for `--test-surreal-write` (no `X_MCP_URL` required).
struct SurrealTestConfig {
    surreal_mcp_url: String,
    surreal_mcp_auth_token: Option<String>,
    surreal_namespace: String,
    surreal_database: String,
}

impl SurrealTestConfig {
    fn from_env() -> Result<Self> {
        let surreal_mcp_url = std::env::var("SURREAL_MCP_URL").context(
            "SURREAL_MCP_URL must be set (SurrealMCP streamable HTTP, e.g. http://127.0.0.1:8800/mcp)",
        )?;
        let surreal_namespace =
            env_one_of(&["NAMESPACE", "SURREALDB_NAMESPACE", "SURREALDB_NS"], "NAMESPACE")?;
        let surreal_database =
            env_one_of(&["DATABASE", "SURREALDB_DATABASE", "SURREALDB_DB"], "DATABASE")?;
        Ok(Self {
            surreal_mcp_url,
            surreal_mcp_auth_token: std::env::var("SURREAL_MCP_AUTH_TOKEN").ok(),
            surreal_namespace,
            surreal_database,
        })
    }
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

        // Applied via SurrealMCP tool calls (`use_namespace` + `use_database`).
        // "Prefixed if needed" support: accept bare + prefixed spellings.
        let surreal_namespace =
            env_one_of(&["NAMESPACE", "SURREALDB_NAMESPACE", "SURREALDB_NS"], "NAMESPACE")?;
        let surreal_database =
            env_one_of(&["DATABASE", "SURREALDB_DATABASE", "SURREALDB_DB"], "DATABASE")?;

        let rate_limit_enforced = match std::env::var("RATE_LIMIT_ENFORCED") {
            Ok(s) => !matches!(s.to_lowercase().as_str(), "0" | "false" | "no" | "off"),
            Err(_) => true,
        };

        Ok(Self {
            surreal_mcp_url,
            surreal_mcp_auth_token: std::env::var("SURREAL_MCP_AUTH_TOKEN").ok(),
            surreal_namespace,
            surreal_database,
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
            .allow_burst(NonZeroU32::new(5).expect("5"));

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
    let uri_for_log = uri.clone();
    let auth_token_set = auth_token.is_some();
    let mut tcfg = StreamableHttpClientTransportConfig::with_uri(uri);
    if let Some(tok) = auth_token {
        tcfg = tcfg.auth_header(tok);
    }
    let transport = StreamableHttpClientTransport::from_config(tcfg);
    info!(
        label,
        mcp_url = %uri_for_log,
        mcp_auth_token_set = auth_token_set,
        "connecting to streamable MCP (TCP + MCP initialize; can hang here if URL/port wrong)"
    );
    ()
        .serve(transport)
        .await
        .map_err(|e| anyhow!("{label} MCP connect to {uri_for_log}: {e}"))
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

    // SurrealMCP / driver `Debug`: zero rows (e.g. no matching `bookmark_id`).
    if trimmed.contains("Ok(Array(Array([])))") || trimmed.contains("Ok(Array(Array([ ])))") {
        return Some(0);
    }
    // Loose match: empty inner array after Array(Array(
    if let Some(pos) = trimmed.find("Array(Array([") {
        let after = &trimmed[pos + "Array(Array([".len()..];
        if after.starts_with("])") || after.starts_with(" ])") {
            return Some(0);
        }
    }

    if let Ok(v) = serde_json::from_str::<Value>(trimmed)
        && let Some(n) = parse_count_from_json_value(&v)
    {
        return Some(n);
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

async fn surreal_use_namespace(surreal: &ClientPeer, namespace: &str) -> Result<()> {
    let mut args = JsonObject::new();
    args.insert("namespace".to_string(), json!(namespace));

    let res = surreal
        .call_tool(CallToolRequestParams {
            meta: None,
            name: Cow::Borrowed("use_namespace"),
            arguments: Some(args),
            task: None,
        })
        .await
        .map_err(|e| anyhow!("SurrealMCP use_namespace tool: {e}"))?;

    mcp_ensure_ok_surreal(&res).context("SurrealMCP use_namespace result")?;
    Ok(())
}

async fn surreal_use_database(surreal: &ClientPeer, database: &str) -> Result<()> {
    let mut args = JsonObject::new();
    args.insert("database".to_string(), json!(database));

    let res = surreal
        .call_tool(CallToolRequestParams {
            meta: None,
            name: Cow::Borrowed("use_database"),
            arguments: Some(args),
            task: None,
        })
        .await
        .map_err(|e| anyhow!("SurrealMCP use_database tool: {e}"))?;

    mcp_ensure_ok_surreal(&res).context("SurrealMCP use_database result")?;
    Ok(())
}

async fn surreal_apply_namespace_database(
    surreal: &ClientPeer,
    namespace: &str,
    database: &str,
) -> Result<()> {
    surreal_use_namespace(surreal, namespace).await?;
    surreal_use_database(surreal, database).await?;
    Ok(())
}

async fn surreal_query_exists(surreal: &ClientPeer, bookmark_id: &str) -> Result<bool> {
    let mut args = JsonObject::new();
    let q = format!(
        "SELECT count() FROM {SURREAL_X_BOOKMARKS_TABLE} WHERE bookmark_id = $bid"
    );
    args.insert("query".to_string(), json!(q));
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
    // Empty NS/DB: table isn't defined until the first `CREATE`. Query returns an error, not
    // "count 0". Treat as "row absent" so bootstrap works; do not `bail!` (that would block creation).
    if text.contains("does not exist") && text.contains(SURREAL_X_BOOKMARKS_TABLE) {
        if !SURREAL_TABLE_MISSING_LOGGED.swap(true, Ordering::Relaxed) {
            info!(
                table = %SURREAL_X_BOOKMARKS_TABLE,
                "Surreal: table not defined yet (empty database). First successful CREATE defines it; until then existence checks see this error and are treated as not found."
            );
        }
        return Ok(false);
    }
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

/// Inserts one bookmark row via SurrealMCP `create`. Used by **`--dry-run` / normal pull** and
/// **`--test-surreal-write`** (same code path).
async fn surreal_create_bookmark(surreal: &ClientPeer, record: &JsonObject) -> Result<()> {
    // SurrealMCP `create` expects a full record id (`table:id`), not a bare table name.
    let bid = record
        .get("bookmark_id")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("surreal_create_bookmark: missing or empty bookmark_id"))?;
    let target = format!("{SURREAL_X_BOOKMARKS_TABLE}:{bid}");

    let mut args = JsonObject::new();
    args.insert("target".to_string(), json!(target));
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

/// One-shot check: calls [`surreal_create_bookmark`] + [`surreal_query_exists`] (same as pull / `--dry-run`).
async fn run_surreal_write_test(cfg: SurrealTestConfig) -> Result<()> {
    let test_id = format!(
        "_puller_write_test_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    info!(
        surreal_mcp = %cfg.surreal_mcp_url,
        namespace = %cfg.surreal_namespace,
        database = %cfg.surreal_database,
        test_bookmark_id = %test_id,
        "test-surreal-write: connecting to SurrealMCP"
    );

    let mut surreal_svc = connect_streamable_mcp(
        cfg.surreal_mcp_url.clone(),
        cfg.surreal_mcp_auth_token.clone(),
        "SurrealMCP",
    )
    .await?;

    let surreal = surreal_svc.peer();
    surreal_apply_namespace_database(surreal, &cfg.surreal_namespace, &cfg.surreal_database).await?;

    let mut data = JsonObject::new();
    data.insert("bookmark_id".to_string(), json!(&test_id));
    data.insert(
        "text".to_string(),
        json!("xbookmark-puller --test-surreal-write (safe to delete)"),
    );
    data.insert(
        "url".to_string(),
        json!("https://example.invalid/xbookmark-puller-test"),
    );
    data.insert("author".to_string(), json!(""));
    data.insert("created_at".to_string(), json!(Utc::now().to_rfc3339()));
    data.insert("raw_json".to_string(), json!({}));
    data.insert("pulled_at".to_string(), json!(Utc::now().to_rfc3339()));
    data.insert("processed".to_string(), json!(false));

    surreal_create_bookmark(surreal, &data).await?;
    info!(test_bookmark_id = %test_id, "test-surreal-write: SurrealMCP create succeeded");

    let visible = surreal_query_exists(surreal, &test_id).await?;
    if !visible {
        bail!(
            "test-surreal-write: row not found after create (SELECT count() FROM {SURREAL_X_BOOKMARKS_TABLE} WHERE bookmark_id = …); check table name and SurrealMCP"
        );
    }

    info!(
        test_bookmark_id = %test_id,
        "test-surreal-write: read-back OK; remove test row in Surreal if you want (DELETE WHERE bookmark_id = this id)"
    );
    let _ = surreal_svc.close().await;
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
        && (
            detail.contains("rate")
                || detail.contains("limit")
                || msg.contains("rate limit")
                // X sometimes returns a bland 429 where both title and detail are just
                // "Too Many Requests" with no "rate"/"limit" substring. Treat that as
                // a retryable rate-limit response rather than a fatal error.
                || detail.contains("too many requests")
        )
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
        info!(
            has_pagination_token = pagination_token.is_some(),
            "stage: waiting for GET rate-limit slot (governor) before xapimcp get_my_bookmarks"
        );
        rl.acquire_get("get_my_bookmarks").await;

        let mut args = JsonObject::new();
        if let Some(t) = pagination_token {
            args.insert("pagination_token".to_string(), json!(t));
        }

        info!(
            has_pagination_token = pagination_token.is_some(),
            "stage: calling xapimcp tool get_my_bookmarks (blocks until X API responds)"
        );
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

        if let Some(errs) = v.get("errors").and_then(|e| e.as_array())
            && !errs.is_empty()
        {
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

        if let Some(errs) = v.get("errors").and_then(|e| e.as_array())
            && !errs.is_empty()
        {
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
    info!("stage: pull cycle started");
    let mut token: Option<String> = None;
    let mut deletes_this_run: u32 = 0;
    // Each iteration is one xapimcp `get_my_bookmarks` call → one X bookmark-list request (paginated).
    let mut get_my_bookmarks_pages: u32 = 0;

    loop {
        let page = x_get_bookmarks_page(x, token.as_deref(), rl, is_deleting).await?;
        get_my_bookmarks_pages = get_my_bookmarks_pages.saturating_add(1);
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
                    table = SURREAL_X_BOOKMARKS_TABLE,
                    "duplicate bookmark in Surreal; skipping store and delete"
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
            let pulled_at = Utc::now().to_rfc3339();

            let mut data = JsonObject::new();
            data.insert("bookmark_id".to_string(), json!(bid));
            data.insert("text".to_string(), json!(text));
            data.insert("url".to_string(), json!(url));
            data.insert("author".to_string(), json!(author));
            data.insert("created_at".to_string(), json!(created_at));
            // Store the tweet as a nested object (not a JSON string) for Surreal/inspection.
            data.insert("raw_json".to_string(), tweet.clone());
            data.insert("pulled_at".to_string(), json!(pulled_at));
            data.insert("processed".to_string(), json!(false));

            surreal_create_bookmark(surreal, &data).await?;
            info!(
                bookmark_id = bid,
                table = SURREAL_X_BOOKMARKS_TABLE,
                "stored new bookmark row in Surreal"
            );

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

    info!(
        get_my_bookmarks_pages,
        dry_run = cfg.dry_run,
        "pull cycle finished bookmark list fetch (each page is one X bookmark-list API call; dry_run skips delete_bookmark only)"
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Default to `info` when RUST_LOG is unset; otherwise people see **no** lines and assume stdout
    // (tracing emits to **stderr** by default).
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .init();

    if cli.test_surreal_write {
        let scfg = SurrealTestConfig::from_env()?;
        run_surreal_write_test(scfg).await?;
        return Ok(());
    }

    let cfg = Arc::new(Config::from_env(cli.dry_run)?);
    info!(
        pull_interval_secs = cfg.pull_interval.as_secs(),
        pull_interval_minutes = cfg.pull_interval.as_secs() / 60,
        surreal_mcp = %cfg.surreal_mcp_url,
        surreal_namespace = %cfg.surreal_namespace,
        surreal_database = %cfg.surreal_database,
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
    info!(
        surreal_mcp = %cfg.surreal_mcp_url,
        "stage: connecting to SurrealMCP (after this, use_namespace/use_database)"
    );
    let mut surreal_svc = connect_streamable_mcp(
        cfg.surreal_mcp_url.clone(),
        cfg.surreal_mcp_auth_token.clone(),
        "SurrealMCP",
    )
    .await?;
    info!("stage: SurrealMCP transport connected");

    // Ensure SurrealMCP session is pinned to the requested namespace/database
    // before we run any `query` / `create` tools (including in --dry-run mode).
    info!(
        namespace = %cfg.surreal_namespace,
        database = %cfg.surreal_database,
        "stage: calling SurrealMCP use_namespace then use_database"
    );
    surreal_apply_namespace_database(
        surreal_svc.peer(),
        &cfg.surreal_namespace,
        &cfg.surreal_database,
    )
    .await?;
    info!(
        surreal_namespace = %cfg.surreal_namespace,
        surreal_database = %cfg.surreal_database,
        "SurrealMCP session configured via use_namespace/use_database"
    );

    info!(
        x_mcp = %cfg.x_mcp_url,
        "stage: connecting to xapimcp (if process hangs here with no new surrealmcp lines, xapimcp is not reachable or MCP handshake stuck)"
    );
    let mut x_svc = connect_streamable_mcp(
        cfg.x_mcp_url.clone(),
        cfg.x_mcp_auth_token.clone(),
        "xapimcp",
    )
    .await?;
    info!("stage: xapimcp transport connected; entering scheduler loop (logs go to stderr unless redirected)");

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
                info!(
                    pull_interval_secs = cfg.pull_interval.as_secs(),
                    "stage: scheduler tick (first tick is immediate; later ticks are on pull_interval)"
                );
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
                                if let Err(e) = surreal_apply_namespace_database(
                                    surreal_svc.peer(),
                                    &cfg.surreal_namespace,
                                    &cfg.surreal_database,
                                )
                                .await
                                {
                                    error!(error = %e, "SurrealMCP reconnect: use_namespace/use_database failed");
                                }
                                info!("reconnected SurrealMCP and xapimcp (namespace/database re-applied)");
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
