You are an expert Rust systems engineer working on the salt-master-agent project.

You are building a thin, production-grade Rust binary called `salt-master-agent`. This is the central entry-point for the LLM-powered Master Agent that fully controls a SaltStack cluster using natural language prompts. It discovers and calls tools exclusively via the Model Context Protocol (MCP). The binary must be a single static executable (no Python, no LangChain, no in-memory history structs).

### Agent Purpose
- The agent receives high-level natural-language commands (e.g. “Deploy a new minion”, “Apply state X to all minions”, “Pull my latest X bookmarks and store them”, “Create 3 Polymarket trading bots”).
- It uses a local Ollama LLM (via Rig) to reason, plan, and execute ReAct-style loops.
- Every capability comes from dynamically discovered MCP tools (SurrealMCP for memory, saltapimcp for SaltStack, xapimcp, githubMCP, etc.).
- After every cycle it logs the full trace, stores outcomes, and remains completely stateless except for what lives in SurrealDB via MCP.

### Pattern We Must Follow (Latest & Greatest Rust 2026)
- Use the **Rig + rmcp** stack exclusively (Rig for agent orchestration + forced tool calling, rmcp for MCP client with full async support and type-safe schemas).
- Rust edition 2024, Tokio runtime, async/await everywhere.
- Cargo workspace layout with `salt-master-agent` as the binary crate.
- Strict error handling with `thiserror` + `anyhow`, tracing with `tracing` + `tracing-subscriber`, configuration via `figment` or `clap` + env vars.
- Zero hard-coded queries or in-memory state — everything is an MCP tool call.
- Forced tool-calling pattern in every ReAct step (no free-form LLM output until a tool is selected).
- Clean separation: `main.rs` only wires the agent; all logic lives in `agent.rs`, `mcp.rs`, `config.rs`.
- Use `rmcp::client::McpClient` with proper connection pooling and reconnection logic.
- Compile-time safety: derive `Serialize`, `Deserialize`, `Debug` on all MCP payloads; use typed tool schemas.
- Production readiness: graceful shutdown on SIGTERM, health-check endpoint, structured JSON logging.

### Key Methods / Functions to Implement
- `fn main() -> Result<(), Box<dyn std::error::Error>>` — parses config, starts MCP clients, builds the Rig agent, runs the interactive REPL or HTTP server loop.
- `async fn build_agent() -> Agent` — constructs the Rig `Agent` with Ollama backend, system prompt, and all discovered MCP tools.
- `async fn run_repl(agent: Agent)` or `async fn serve_http(agent: Agent)` — the main interactive loop.
- `async fn initialize_mcp_clients() -> (SurrealMcpClient, SaltApiMcpClient, ...)` — connects to every MCP server.
- Tool-calling hooks: `on_tool_call`, `on_tool_result`, `on_reflection` that always route through SurrealMCP.

### Memory Section – What We Are Doing
We are replacing ALL previous memory/history logic with SurrealMCP tools. There must be zero in-memory vectors, no custom `ConversationHistory` structs, no direct SurrealQL calls, and no leftover Python-style buffers.

- The agent discovers SurrealMCP tools at startup (`store_action_log`, `retrieve_recent_history`, `store_reflection`, `query_gig_outcomes`, `store_minion_event`, etc.).
- After every ReAct step, tool call, or high-level task completion, the agent MUST call the appropriate SurrealMCP tool to persist the full trace, reasoning, outcome, and any SaltStack state changes.
- System prompt must contain the strict instruction: “You MUST use SurrealMCP tools for ALL memory, history, reflections, and long-term storage. Never maintain local state or buffers.”
- Use Rig’s forced tool-calling to guarantee every memory operation is an explicit MCP call.
- On startup, the agent loads only the minimal context needed by calling `retrieve_recent_history(limit=10)` via SurrealMCP.
- This makes the entire agent stateless across restarts and gives us perfect auditability and self-improvement loops.

Generate the complete project skeleton (Cargo.toml, src/main.rs, src/agent.rs, src/mcp.rs, src/config.rs) with production-ready code that follows the pattern above. Use the most modern, idiomatic Rust practices available in 2026. Include comprehensive comments and a clear README.md. Begin implementation immediately.