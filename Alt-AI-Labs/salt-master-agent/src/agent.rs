//! Rig [`Agent`] construction, SurrealMCP-backed hooks, and startup history hydration.

use std::sync::Arc;

use rig::{
    agent::{Agent, AgentBuilder, HookAction, PromptHook, ToolCallHookAction},
    client::{CompletionClient, Nothing},
    completion::Prompt,
    message::ToolChoice,
    providers::ollama,
};
use rmcp::model::{CallToolRequestParams, CallToolResult, RawContent};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::config::AppConfig;
use crate::mcp::{McpRuntime, SurrealMemoryTools};

/// Hooks that persist audit rows through **discovered** SurrealMCP tools (`store_*`, etc.),
/// falling back to `insert` / `select` when those semantic tools are absent (e.g. stock surrealmcp).
///
/// There is **no** in-process conversation buffer: persistence is always an explicit MCP `call_tool`.
#[derive(Clone)]
pub struct SurrealAuditHook {
    sink: Option<rmcp::service::ServerSink>,
    /// Resolved at SurrealMCP connect from `tools/list` (see [`crate::mcp::resolve_surreal_memory_tools`]).
    memory_tools: Option<Arc<SurrealMemoryTools>>,
    trace_table: String,
    history_table: String,
}

impl SurrealAuditHook {
    pub fn new(
        sink: Option<rmcp::service::ServerSink>,
        memory_tools: Option<Arc<SurrealMemoryTools>>,
        trace_table: String,
        history_table: String,
    ) -> Self {
        Self {
            sink,
            memory_tools,
            trace_table,
            history_table,
        }
    }

    /// End-of-turn reflection: prefers `persist_reflection` / `store_reflection` when advertised.
    pub async fn persist_reflection(&self, user_prompt: &str, assistant_text: &str) {
        let Some(sink) = &self.sink else {
            debug!("persist_reflection: no Surreal sink");
            return;
        };

        let tool_name = self
            .memory_tools
            .as_ref()
            .and_then(|t| t.persist_reflection.as_deref());

        if let Some(name) = tool_name {
            let mut args = serde_json::Map::new();
            args.insert("ts".into(), json!(chrono::Utc::now().to_rfc3339()));
            args.insert("source".into(), json!("salt-master-agent"));
            args.insert("user_prompt".into(), json!(user_prompt));
            args.insert("assistant_reply".into(), json!(assistant_text));
            args.insert("table".into(), json!(&self.history_table));

            match call_mcp_tool(sink, name, args).await {
                Ok(r) if r.is_error != Some(true) => {
                    info!(tool = name, "persist_reflection via SurrealMCP tool");
                    return;
                }
                Ok(r) => warn!(tool = name, ?r, "persist_reflection tool returned error"),
                Err(e) => warn!(tool = name, %e, "persist_reflection tool call failed"),
            }
        }

        self.persist_row_fallback(
            "reflection",
            None,
            Some(user_prompt),
            Some(assistant_text),
            json!({}),
        )
        .await;
    }

    async fn persist_row_via_store_tool(
        &self,
        sink: &rmcp::service::ServerSink,
        phase: &str,
        tool: Option<&str>,
        args: Option<&str>,
        result: Option<&str>,
        extra: &Value,
    ) -> bool {
        let Some(name) = self
            .memory_tools
            .as_ref()
            .and_then(|m| m.store_action_log.as_deref())
        else {
            return false;
        };

        let mut payload = serde_json::Map::new();
        payload.insert("phase".into(), json!(phase));
        payload.insert("ts".into(), json!(chrono::Utc::now().to_rfc3339()));
        payload.insert("source".into(), json!("salt-master-agent"));
        if let Some(t) = tool {
            payload.insert("related_mcp_tool".into(), json!(t));
        }
        if let Some(a) = args {
            payload.insert("arguments_json".into(), json!(a));
        }
        if let Some(r) = result {
            payload.insert("result_json".into(), json!(r));
        }
        payload.insert("extra".into(), extra.clone());
        payload.insert("trace_table".into(), json!(&self.trace_table));

        match call_mcp_tool(sink, name, payload).await {
            Ok(r) if r.is_error != Some(true) => {
                debug!(tool = name, phase, "store_* audit tool succeeded");
                true
            }
            Ok(r) => {
                warn!(tool = name, phase, ?r, "store_* audit tool returned error");
                false
            }
            Err(e) => {
                warn!(tool = name, phase, %e, "store_* audit tool call failed");
                false
            }
        }
    }

    async fn persist_row_fallback(
        &self,
        phase: &str,
        tool: Option<&str>,
        args: Option<&str>,
        result: Option<&str>,
        extra: Value,
    ) {
        let Some(sink) = &self.sink else {
            debug!(phase, tool, "Surreal sink absent; skipping insert fallback");
            return;
        };

        let mut row = serde_json::Map::new();
        row.insert("ts".into(), json!(chrono::Utc::now().to_rfc3339()));
        row.insert("phase".into(), json!(phase));
        row.insert("source".into(), json!("salt-master-agent"));
        if let Some(t) = tool {
            row.insert("tool".into(), json!(t));
        }
        if let Some(a) = args {
            row.insert("args".into(), json!(a));
        }
        if let Some(r) = result {
            row.insert("result".into(), json!(r));
        }
        row.insert("extra".into(), extra);

        let mut params = serde_json::Map::new();
        params.insert("target".into(), json!(&self.trace_table));
        params.insert("values".into(), json!(vec![row]));

        let req = CallToolRequestParams::new("insert").with_arguments(params);
        match sink.call_tool(req).await {
            Ok(r) => {
                if r.is_error == Some(true) {
                    warn!(phase, ?r, "SurrealMCP insert fallback reported error");
                }
            }
            Err(e) => warn!(phase, %e, "SurrealMCP insert fallback failed"),
        }
    }

    async fn persist_row(
        &self,
        phase: &str,
        tool: Option<&str>,
        args: Option<&str>,
        result: Option<&str>,
        extra: Value,
    ) {
        let Some(sink) = &self.sink else {
            debug!(phase, tool, "Surreal sink absent");
            return;
        };

        if self
            .persist_row_via_store_tool(sink, phase, tool, args, result, &extra)
            .await
        {
            return;
        }

        self.persist_row_fallback(phase, tool, args, result, extra)
            .await;
    }
}

async fn call_mcp_tool(
    sink: &rmcp::service::ServerSink,
    name: &str,
    arguments: serde_json::Map<String, Value>,
) -> Result<CallToolResult, rmcp::ServiceError> {
    let req = CallToolRequestParams::new(name.to_string()).with_arguments(arguments);
    sink.call_tool(req).await
}

impl<M> PromptHook<M> for SurrealAuditHook
where
    M: rig::completion::CompletionModel,
{
    async fn on_tool_call(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
    ) -> ToolCallHookAction {
        self.persist_row("tool_call", Some(tool_name), Some(args), None, json!({}))
            .await;
        ToolCallHookAction::cont()
    }

    async fn on_tool_result(
        &self,
        tool_name: &str,
        _tool_call_id: Option<String>,
        _internal_call_id: &str,
        args: &str,
        result: &str,
    ) -> HookAction {
        self.persist_row(
            "tool_result",
            Some(tool_name),
            Some(args),
            Some(result),
            json!({}),
        )
        .await;
        HookAction::cont()
    }
}

fn flatten_tool_result(r: &CallToolResult) -> String {
    r.content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Startup hydration: calls discovered `retrieve_recent_history` (or `retrieve_*`) with `limit`, else `select`.
pub async fn retrieve_recent_history(
    sink: &rmcp::service::ServerSink,
    resolved: Option<&SurrealMemoryTools>,
    cfg: &AppConfig,
    limit: u32,
) -> Option<String> {
    if let Some(names) = resolved
        && let Some(tname) = names.retrieve_recent_history.as_deref()
    {
        let mut args = serde_json::Map::new();
        args.insert("limit".into(), json!(limit));
        args.insert("table".into(), json!(&cfg.surreal_history_table));

        match call_mcp_tool(sink, tname, args).await {
            Ok(r) if r.is_error != Some(true) => {
                let text = flatten_tool_result(&r);
                if !text.is_empty() {
                    info!(tool = tname, limit, "retrieve_recent_history via SurrealMCP tool");
                    return Some(text);
                }
            }
            Ok(r) => warn!(tool = tname, ?r, "retrieve_recent_history tool returned error"),
            Err(e) => warn!(tool = tname, %e, "retrieve_recent_history tool call failed"),
        }
    }

    let mut args = serde_json::Map::new();
    args.insert("targets".into(), json!(vec![cfg.surreal_history_table.as_str()]));
    args.insert("order_clause".into(), json!("ts DESC"));
    args.insert("limit_clause".into(), json!(limit.to_string()));

    let req = CallToolRequestParams::new("select").with_arguments(args);
    match sink.call_tool(req).await {
        Ok(r) => {
            if r.is_error == Some(true) {
                warn!(?r, "retrieve_recent_history: select fallback returned error");
                return None;
            }
            let text = flatten_tool_result(&r);
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        Err(e) => {
            warn!(%e, "retrieve_recent_history: select fallback failed");
            None
        }
    }
}

const SYSTEM_PREAMBLE: &str = r#"You are the Salt Master Agent: a production operations agent that controls SaltStack and related infrastructure through MCP tools only.

## Tooling
- You discover capabilities only from the tools exposed by connected MCP servers (SurrealMCP, saltapimcp, xapimcp, github MCP, etc.).
- Plan in a ReAct style: reason briefly, pick a tool, execute, observe, repeat.

## Memory (mandatory)
You MUST use SurrealMCP tools for ALL memory, history, reflections, and long-term storage. Never maintain local state or buffers.
Prefer semantic tools when your SurrealMCP exposes them: `store_action_log` (or other `store_*`), `retrieve_recent_history` (or `retrieve_*`), and `persist_reflection` / `store_reflection` for reflections.
The process also mirrors tool calls into SurrealDB via those same tool names when advertised; if they are absent, generic `insert` / `select` fallbacks apply on the agent side only — you should still use SurrealMCP for durable application memory.

## Safety
- Prefer narrow targeting (explicit minions, saltenv, pillarenv) over wildcards unless the user clearly requests cluster-wide changes.
- Explain destructive actions before executing them when the user did not explicitly ask for destructive behavior.
"#;

/// Build the Rig agent with Ollama, forced tool use when tools exist, and shared MCP tool server.
pub async fn build_agent(
    cfg: &AppConfig,
    mcp: &McpRuntime,
) -> anyhow::Result<Agent<ollama::CompletionModel, SurrealAuditHook>> {
    let ollama_client = ollama::Client::builder()
        .api_key(Nothing)
        .base_url(&cfg.ollama_base_url)
        .build()
        .map_err(|e| anyhow::anyhow!("ollama client build failed: {e}"))?;

    let model = ollama_client.completion_model(cfg.ollama_model.clone());
    let handle = mcp.tool_server_handle();

    let n_tools = handle
        .get_tool_defs(None)
        .await
        .map(|d| d.len())
        .unwrap_or(0);

    let tool_choice = if n_tools > 0 {
        ToolChoice::Required
    } else {
        warn!("no MCP tools registered; falling back to ToolChoice::Auto");
        ToolChoice::Auto
    };

    let mut preamble = SYSTEM_PREAMBLE.to_string();
    if let Some(sink) = mcp.surreal_sink()
        && !cfg.skip_startup_history
        && let Some(hist) = retrieve_recent_history(
            &sink,
            mcp.surreal_memory_tools().as_deref(),
            cfg,
            10,
        )
        .await
    {
        preamble.push_str(
            "\n\n## Recent history (startup `retrieve_recent_history(10)` or `select` fallback)\n",
        );
        preamble.push_str(&hist);
    }

    let hook = SurrealAuditHook::new(
        mcp.surreal_sink(),
        mcp.surreal_memory_tools(),
        cfg.surreal_trace_table.clone(),
        cfg.surreal_history_table.clone(),
    );

    let agent = AgentBuilder::new(model)
        .name("salt-master-agent")
        .preamble(&preamble)
        .temperature(0.2)
        .default_max_turns(48)
        .tool_choice(tool_choice)
        .hook(hook)
        .tool_server_handle(handle)
        .build();

    Ok(agent)
}

/// Run an interactive stdin/stdout loop.
pub async fn run_repl(agent: Agent<ollama::CompletionModel, SurrealAuditHook>) -> anyhow::Result<()> {
    use std::io::Write;
    use tokio::io::{AsyncBufReadExt, BufReader};

    let hook = agent.hook.clone();
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut line = String::new();
    loop {
        line.clear();
        print!("salt-master-agent> ");
        std::io::stdout().flush().ok();
        let n = stdin.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        match agent.prompt(input).await {
            Ok(reply) => {
                println!("{reply}");
                if let Some(h) = &hook {
                    h.persist_reflection(input, &reply).await;
                }
            }
            Err(e) => eprintln!("error: {e:#}"),
        }
    }
    Ok(())
}
