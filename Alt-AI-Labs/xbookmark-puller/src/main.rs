//! X bookmark archiver: **xapimcp** MCP tools `get_my_bookmarks` + `delete_bookmark` only (no direct X HTTP),
//! dedupe + `create` via SurrealMCP (`query` / `create` tools).

use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use rmcp::model::{CallToolRequestParams, CallToolResult, JsonObject, RawContent};
use rmcp::service::ServiceExt;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use serde_json::{Value, json};
use tokio::sync::watch;
use tracing::{error, info, warn};

type ClientPeer = rmcp::service::Peer<rmcp::service::RoleClient>;
type RunningClient = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

#[derive(Clone)]
struct Config {
    surreal_mcp_url: String,
    surreal_mcp_auth_token: Option<String>,
    x_mcp_url: String,
    x_mcp_auth_token: Option<String>,
    pull_interval: Duration,
}

impl Config {
    fn from_env() -> Result<Self> {
        let pull_minutes: u64 = std::env::var("PULL_INTERVAL_MINUTES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);
        let surreal_mcp_url = std::env::var("SURREAL_MCP_URL")
            .context("SURREAL_MCP_URL must be set (SurrealMCP streamable HTTP, e.g. http://127.0.0.1:8080/mcp)")?;
        let x_mcp_url = std::env::var("X_MCP_URL")
            .context("X_MCP_URL must be set (xapimcp streamable HTTP endpoint, e.g. http://127.0.0.1:8090/mcp)")?;

        Ok(Self {
            surreal_mcp_url,
            surreal_mcp_auth_token: std::env::var("SURREAL_MCP_AUTH_TOKEN").ok(),
            x_mcp_url,
            x_mcp_auth_token: std::env::var("X_MCP_AUTH_TOKEN").ok(),
            pull_interval: Duration::from_secs(pull_minutes.max(1).saturating_mul(60)),
        })
    }
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

/// Parse Surreal debug output (e.g. IndexedResults) for count() query responses.
/// Handles nested arrays/objects by first checking key-based forms (`count`, `c`)
/// and then scanning for `Number(Int(...))` / `Number(Float(...))`.
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

fn x_error_entry_rate_limited(e: &Value) -> bool {
    let status = e.get("status").and_then(|s| s.as_u64()).unwrap_or(0);
    if status == 429 {
        return true;
    }
    let title = e
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    let detail = e.get("detail").and_then(|t| t.as_str()).unwrap_or("");
    let msg = e
        .get("message")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    title.contains("too many")
        || detail.contains("429")
        || msg.contains("rate limit")
        || msg.contains("429")
}

fn x_value_rate_limited(v: &Value) -> bool {
    let Some(errs) = v.get("errors").and_then(|e| e.as_array()) else {
        return false;
    };
    if errs.is_empty() {
        return false;
    }
    errs.iter().all(|e| x_error_entry_rate_limited(e))
}

fn x_value_fatal_errors(v: &Value) -> Option<String> {
    let errs = v.get("errors")?.as_array()?;
    if errs.is_empty() {
        return None;
    }
    if x_value_rate_limited(v) {
        return None;
    }
    let first = &errs[0];
    let title = first.get("title").and_then(|t| t.as_str()).unwrap_or("error");
    let detail = first.get("detail").and_then(|t| t.as_str()).unwrap_or("");
    Some(format!("{title} — {detail}"))
}

async fn x_get_bookmarks_page(x: &ClientPeer, pagination_token: Option<&str>) -> Result<Value> {
    let mut attempt = 0u32;
    loop {
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

        if x_value_rate_limited(&v) {
            let wait_secs = backoff_secs(attempt).min(900).max(1);
            warn!(wait_secs, attempt, "xapimcp bookmarks response rate limited; backing off");
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            attempt = attempt.saturating_add(1).min(12);
            continue;
        }

        if let Some(fatal) = x_value_fatal_errors(&v) {
            bail!("X API (via xapimcp): {fatal}");
        }

        return Ok(v);
    }
}

async fn x_delete_bookmark_mcp(x: &ClientPeer, tweet_id: &str) -> Result<()> {
    let mut attempt = 0u32;
    loop {
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

        if x_value_rate_limited(&v) {
            let wait_secs = backoff_secs(attempt).min(900).max(1);
            warn!(
                tweet_id,
                wait_secs,
                attempt,
                "xapimcp delete response rate limited; backing off"
            );
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
            attempt = attempt.saturating_add(1).min(12);
            continue;
        }

        if let Some(fatal) = x_value_fatal_errors(&v) {
            bail!("X delete (via xapimcp): {fatal}");
        }

        return Ok(());
    }
}

fn backoff_secs(attempt: u32) -> u64 {
    (1u64 << attempt.min(8)).min(300)
}

async fn run_pull_cycle(surreal: &ClientPeer, x: &ClientPeer) -> Result<()> {
    let mut token: Option<String> = None;
    loop {
        let page = x_get_bookmarks_page(x, token.as_deref()).await?;
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
                    "duplicate bookmark detected in x_bookmarks; skipping store and delete"
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
            info!(bookmark_id = bid, "stored in x_bookmarks");

            x_delete_bookmark_mcp(x, bid).await?;
            info!(
                bookmark_id = bid,
                "deleted bookmark from X via xapimcp after successful store"
            );
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
        tokio::time::sleep(Duration::from_millis(350)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let cfg = Arc::new(Config::from_env()?);
    info!(
        pull_interval_secs = cfg.pull_interval.as_secs(),
        surreal_mcp = %cfg.surreal_mcp_url,
        x_mcp = %cfg.x_mcp_url,
        "xbookmark-puller starting"
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
                match run_pull_cycle(surreal, x).await {
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
