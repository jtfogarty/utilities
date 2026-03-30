//! Configuration via `figment` (environment + optional TOML).
//!
//! All environment keys use the `SMA_` prefix. Nested keys use `__`, e.g.
//! `SMA_MCP__SURREAL_URL=http://localhost:3000/mcp`.

use std::path::Path;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::error::SaltMasterAgentError;

/// Application configuration loaded at startup.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AppConfig {
    /// Ollama HTTP API base (no trailing slash), e.g. `http://localhost:11434`.
    pub ollama_base_url: String,
    /// Model id passed to Ollama (e.g. `llama3.2`, `qwen2.5:14b`).
    pub ollama_model: String,
    /// Streamable HTTP endpoint for SurrealMCP (memory / audit).
    pub mcp: McpEndpoints,
    /// Max attempts per MCP endpoint before failing startup.
    pub mcp_connect_max_attempts: u32,
    /// Base delay in milliseconds; exponential backoff caps at ~32s per step.
    pub mcp_connect_base_delay_ms: u64,
    /// Surreal table used by hooks for trace rows (`insert` tool).
    pub surreal_trace_table: String,
    /// Surreal table queried on startup for recent context (`select` tool).
    pub surreal_history_table: String,
    /// Force MCP tool name for audit rows (must exist on SurrealMCP). If unset, resolved from `tools/list`.
    pub surreal_tool_store_action_log: Option<String>,
    /// Force MCP tool name for startup history. If unset, prefers `retrieve_recent_history` or first `retrieve_*`.
    pub surreal_tool_retrieve_recent_history: Option<String>,
    /// Force MCP tool name for end-of-turn reflections. If unset, prefers `persist_reflection` / `store_reflection`.
    pub surreal_tool_persist_reflection: Option<String>,
    /// If true, skip `select` on startup when history table may not exist yet.
    pub skip_startup_history: bool,
    /// HTTP bind address for `serve` mode.
    pub http_bind: String,
    /// RUST_LOG-style filter (also respected by `tracing_subscriber::EnvFilter`).
    pub log_filter: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct McpEndpoints {
    pub surreal_url: Option<String>,
    pub surreal_auth_header: Option<String>,
    pub salt_url: Option<String>,
    pub salt_auth_header: Option<String>,
    pub x_url: Option<String>,
    pub x_auth_header: Option<String>,
    pub github_url: Option<String>,
    pub github_auth_header: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            mcp: McpEndpoints::default(),
            mcp_connect_max_attempts: 8,
            mcp_connect_base_delay_ms: 500,
            surreal_trace_table: "agent_mcp_trace".to_string(),
            surreal_history_table: "agent_history".to_string(),
            surreal_tool_store_action_log: None,
            surreal_tool_retrieve_recent_history: None,
            surreal_tool_persist_reflection: None,
            skip_startup_history: false,
            http_bind: "127.0.0.1:7099".to_string(),
            log_filter: "info,salt_master_agent=debug,rig=info,rmcp=warn".to_string(),
        }
    }
}

impl AppConfig {
    /// Load configuration: defaults → optional `salt-master-agent.toml` → `SMA_*` env.
    pub fn load() -> Result<Self, SaltMasterAgentError> {
        let path = Path::new("salt-master-agent.toml");
        let mut figment = Figment::new().merge(Serialized::defaults(AppConfig::default()));
        if path.exists() {
            figment = figment.merge(Toml::file(path));
        }
        Ok(figment.merge(Env::prefixed("SMA_").split("__")).extract()?)
    }
}
