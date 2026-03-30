//! MCP client wiring: `rmcp` streamable HTTP transports + Rig [`McpClientHandler`].
//!
//! Each configured endpoint gets its own [`McpClientHandler`] and
//! [`rmcp::service::RunningService`]. All handlers share one [`ToolServerHandle`] so the
//! agent sees a unified tool namespace. **Tool names must be unique across servers** (prefix
//! tools in each MCP server if needed).
//!
//! SurrealMCP is connected first (retry/backoff), then we call `tools/list` on that session to
//! resolve semantic memory tool names (`store_*`, `retrieve_*`, `persist_reflection`) for
//! [`crate::agent::SurrealAuditHook`].

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use rig::tool::rmcp::McpClientHandler;
use rig::tool::server::ToolServerHandle;
use rmcp::{
    model::Tool,
    model::{ClientInfo, Implementation},
    service::RunningService,
    transport::StreamableHttpClientTransport,
    transport::streamable_http_client::StreamableHttpClientTransportConfig,
};
use tracing::{info, warn};

use crate::config::AppConfig;

type ClientRunningService = RunningService<rmcp::RoleClient, McpClientHandler>;

/// Resolved SurrealMCP tool names from `tools/list` (plus optional config overrides).
#[derive(Debug, Clone, Default)]
pub struct SurrealMemoryTools {
    /// e.g. `store_action_log` or another `store_*` suitable for hook audit rows.
    pub store_action_log: Option<String>,
    /// e.g. `retrieve_recent_history` or another `retrieve_*`.
    pub retrieve_recent_history: Option<String>,
    /// e.g. `persist_reflection` or `store_reflection`.
    pub persist_reflection: Option<String>,
}

/// Owns MCP sessions; dropping this aborts remote tool connectivity.
pub struct McpRuntime {
    tool_server_handle: ToolServerHandle,
    /// Surreal MCP peer used by audit hooks (same connection as registered tools).
    surreal_sink: Option<rmcp::service::ServerSink>,
    /// Names chosen from SurrealMCP's advertised tools (for direct `call_tool` in hooks).
    surreal_memory_tools: Option<Arc<SurrealMemoryTools>>,
    _services: Vec<ClientRunningService>,
}

impl McpRuntime {
    pub fn tool_server_handle(&self) -> ToolServerHandle {
        self.tool_server_handle.clone()
    }

    pub fn surreal_sink(&self) -> Option<rmcp::service::ServerSink> {
        self.surreal_sink.clone()
    }

    pub fn surreal_memory_tools(&self) -> Option<Arc<SurrealMemoryTools>> {
        self.surreal_memory_tools.clone()
    }
}

fn transport_for(url: &str, auth: Option<&str>) -> StreamableHttpClientTransport<reqwest::Client> {
    let mut cfg = StreamableHttpClientTransportConfig::with_uri(url);
    if let Some(token) = auth {
        cfg = cfg.auth_header(token);
    }
    StreamableHttpClientTransport::from_config(cfg)
}

fn client_info(name: &str) -> ClientInfo {
    ClientInfo {
        client_info: Implementation {
            name: format!("salt-master-agent-{name}"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

async fn connect_one(
    label: &'static str,
    url: &str,
    auth: Option<&str>,
    tool_server_handle: ToolServerHandle,
    max_attempts: u32,
    base_delay_ms: u64,
) -> anyhow::Result<ClientRunningService> {
    let mut last_err = None;
    for attempt in 1..=max_attempts {
        let transport = transport_for(url, auth);
        let handler = McpClientHandler::new(client_info(label), tool_server_handle.clone());
        match handler.connect(transport).await {
            Ok(service) => {
                info!(label, url, attempt, "MCP session established");
                return Ok(service);
            }
            Err(e) => {
                warn!(label, url, attempt, %e, "MCP connect failed; retrying");
                last_err = Some(e);
                let exp = attempt.saturating_sub(1).min(6);
                let sleep = base_delay_ms.saturating_mul(2u64.saturating_pow(exp));
                tokio::time::sleep(Duration::from_millis(sleep.min(32_000))).await;
            }
        }
    }
    Err(anyhow::anyhow!(
        "MCP `{label}` at {url} failed after {max_attempts} attempts: {last_err:?}"
    ))
}

fn validate_override(name: &str, available: &HashSet<String>) -> Option<String> {
    if available.contains(name) {
        Some(name.to_string())
    } else {
        warn!(
            name,
            "configured SurrealMCP tool override not found in tools/list; ignoring"
        );
        None
    }
}

/// Map SurrealMCP `tools/list` to the semantic slots hooks and startup use.
pub fn resolve_surreal_memory_tools(advertised: &[Tool], cfg: &AppConfig) -> SurrealMemoryTools {
    let names: HashSet<String> = advertised
        .iter()
        .map(|t| t.name.as_ref().to_string())
        .collect();

    let mut out = SurrealMemoryTools::default();

    if let Some(ref o) = cfg.surreal_tool_store_action_log {
        out.store_action_log = validate_override(o, &names);
    }
    if let Some(ref o) = cfg.surreal_tool_retrieve_recent_history {
        out.retrieve_recent_history = validate_override(o, &names);
    }
    if let Some(ref o) = cfg.surreal_tool_persist_reflection {
        out.persist_reflection = validate_override(o, &names);
    }

    if out.store_action_log.is_none() {
        for candidate in ["store_action_log", "store_audit_log", "store_agent_trace"] {
            if names.contains(candidate) {
                out.store_action_log = Some(candidate.to_string());
                break;
            }
        }
    }
    if out.store_action_log.is_none() {
        let exclude: HashSet<&str> = ["store_reflection", "persist_reflection"].into_iter().collect();
        let pick = names.iter().find(|n| {
            n.starts_with("store_") && !exclude.contains(n.as_str())
        });
        if let Some(p) = pick {
            out.store_action_log = Some(p.clone());
        }
    }

    if out.retrieve_recent_history.is_none() {
        if names.contains("retrieve_recent_history") {
            out.retrieve_recent_history = Some("retrieve_recent_history".to_string());
        } else {
            let pick = names.iter().find(|n| n.starts_with("retrieve_"));
            if let Some(p) = pick {
                out.retrieve_recent_history = Some(p.clone());
            }
        }
    }

    if out.persist_reflection.is_none() {
        for candidate in ["persist_reflection", "store_reflection"] {
            if names.contains(candidate) {
                out.persist_reflection = Some(candidate.to_string());
                break;
            }
        }
    }
    if out.persist_reflection.is_none() {
        let pick = names.iter().find(|n| n.starts_with("persist_"));
        if let Some(p) = pick {
            out.persist_reflection = Some(p.clone());
        }
    }

    info!(
        store_action_log = ?out.store_action_log,
        retrieve_recent_history = ?out.retrieve_recent_history,
        persist_reflection = ?out.persist_reflection,
        "resolved SurrealMCP memory tool names"
    );

    out
}

/// Connect SurrealMCP with the same retry/backoff as other endpoints, then resolve memory tool names.
async fn connect_surreal_mcp(
    cfg: &AppConfig,
    tool_server_handle: ToolServerHandle,
) -> anyhow::Result<(ClientRunningService, SurrealMemoryTools)> {
    let url = cfg
        .mcp
        .surreal_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("SurrealMCP URL not configured"))?;

    let svc = connect_one(
        "surreal",
        url,
        cfg.mcp.surreal_auth_header.as_deref(),
        tool_server_handle,
        cfg.mcp_connect_max_attempts,
        cfg.mcp_connect_base_delay_ms,
    )
    .await
    .context("SurrealMCP connection")?;

    let resolved = match svc.peer().list_all_tools().await {
        Ok(tools) => resolve_surreal_memory_tools(&tools, cfg),
        Err(e) => {
            warn!(
                error = %e,
                "SurrealMCP tools/list failed; hooks fall back to insert/select"
            );
            SurrealMemoryTools::default()
        }
    };

    Ok((svc, resolved))
}

/// Connect all configured MCP servers and register tools on `tool_server_handle`.
pub async fn initialize_mcp_clients(cfg: &AppConfig) -> anyhow::Result<McpRuntime> {
    let tool_server_handle = rig::tool::server::ToolServer::new().run();
    let mut _services = Vec::new();
    let mut surreal_sink = None;
    let mut surreal_memory_tools = None;

    if cfg.mcp.surreal_url.is_some() {
        match connect_surreal_mcp(cfg, tool_server_handle.clone()).await {
            Ok((svc, resolved)) => {
                surreal_sink = Some(svc.peer().clone());
                surreal_memory_tools = Some(Arc::new(resolved));
                _services.push(svc);
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        warn!("SMA_MCP__SURREAL_URL not set; running without SurrealMCP (hooks will log only)");
    }

    let m = &cfg.mcp;

    if let Some(url) = m.salt_url.as_deref() {
        let svc = connect_one(
            "salt",
            url,
            m.salt_auth_header.as_deref(),
            tool_server_handle.clone(),
            cfg.mcp_connect_max_attempts,
            cfg.mcp_connect_base_delay_ms,
        )
        .await
        .context("Salt MCP connection")?;
        _services.push(svc);
    }

    if let Some(url) = m.x_url.as_deref() {
        let svc = connect_one(
            "x",
            url,
            m.x_auth_header.as_deref(),
            tool_server_handle.clone(),
            cfg.mcp_connect_max_attempts,
            cfg.mcp_connect_base_delay_ms,
        )
        .await
        .context("X MCP connection")?;
        _services.push(svc);
    }

    if let Some(url) = m.github_url.as_deref() {
        let svc = connect_one(
            "github",
            url,
            m.github_auth_header.as_deref(),
            tool_server_handle.clone(),
            cfg.mcp_connect_max_attempts,
            cfg.mcp_connect_base_delay_ms,
        )
        .await
        .context("GitHub MCP connection")?;
        _services.push(svc);
    }

    Ok(McpRuntime {
        tool_server_handle,
        surreal_sink,
        surreal_memory_tools,
        _services,
    })
}
