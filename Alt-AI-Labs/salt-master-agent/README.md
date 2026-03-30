# salt-master-agent

Stateless Rust binary that drives a **SaltStack-focused** LLM agent: local **Ollama** for reasoning, **Rig** for agent loops and forced tool calling, and **rmcp** (aligned with Rig’s MCP version) for every capability. No in-process conversation history; durable memory and audit trails go through **SurrealMCP** (and other MCP servers) only.

## Stack

| Layer | Choice |
|--------|--------|
| LLM | Ollama via `rig`’s Ollama provider |
| Agent | `rig` `AgentBuilder`, `ToolChoice::Required` when any MCP tool is registered |
| MCP client | `rmcp` **0.16** (must match `rig-core`’s `rmcp` dependency — do not bump independently) |
| Tool aggregation | Single shared `ToolServerHandle`; one `McpClientHandler` per MCP URL |
| Config | `figment`: optional `salt-master-agent.toml` + `SMA_*` environment |
| HTTP | `axum`: `GET /health`, `POST /v1/prompt` |
| Logging | `tracing-subscriber` JSON layers + `RUST_LOG` / `SMA_log_filter` |

## Building

```bash
cd salt-master-agent
cargo build --release
```

For a **fully static** Linux binary, use a musl target (example):

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

(macOS binaries are not truly “static” in the ELF sense; use musl/Linux for single-file deploy artifacts.)

## Configuration

1. **Optional file** `salt-master-agent.toml` in the working directory (merged first, then overridden by env).
2. **Environment** variables prefixed with `SMA_`. Nested keys use `__` (Figment), e.g. `SMA_mcp__surreal_url`.

Important fields (defaults in code — see `src/config.rs`):

| Key | Meaning |
|-----|---------|
| `ollama_base_url` | Ollama API root, default `http://localhost:11434` |
| `ollama_model` | Model name, default `llama3.2` |
| `mcp.surreal_url` | SurrealMCP streamable HTTP endpoint |
| `mcp.surreal_auth_header` | Optional `Authorization` header value (e.g. `Bearer …`) |
| `mcp.salt_url` / `mcp.x_url` / `mcp.github_url` | Optional additional MCP servers |
| `surreal_trace_table` | Table used by hooks for `insert` audit rows |
| `surreal_history_table` | Table queried on startup via `select` (expects a `ts` column for ordering) |
| `surreal_tool_store_action_log` | Optional: exact MCP tool name for hook audit rows (must exist in `tools/list`) |
| `surreal_tool_retrieve_recent_history` | Optional: exact MCP tool for startup history |
| `surreal_tool_persist_reflection` | Optional: exact MCP tool for end-of-turn reflections |
| `skip_startup_history` | Skip startup `select` if the table is not ready yet |
| `http_bind` | `serve` listen address, default `127.0.0.1:7099` |

### Example `salt-master-agent.toml`

```toml
ollama_base_url = "http://localhost:11434"
ollama_model = "qwen2.5:14b"

[mcp]
surreal_url = "http://127.0.0.1:8080/mcp"
salt_url = "http://127.0.0.1:8081/mcp"
```

## Running

**REPL** (stdin/stdout):

```bash
cargo run --release -- repl
```

**HTTP** (health + JSON prompt API):

```bash
cargo run --release -- serve
# curl -s localhost:7099/health
# curl -s localhost:7099/v1/prompt -H 'content-type: application/json' -d '{"prompt":"status of salt minions"}'
```

Graceful shutdown: **Ctrl+C** or **SIGTERM** (Unix) while serving.

## MCP tool namespace

All MCP tools from every server are registered on **one** `ToolServerHandle`. **Tool names must be unique across servers.** If two servers expose the same name, rename tools on the server side or split deployments.

## SurrealDB schema (minimal)

Hooks and startup history assume SurrealMCP’s **`insert`** and **`select`** tools. Your trace/history tables should at least support the rows written by this agent (e.g. a `ts` field for ordering). The default table names are `agent_mcp_trace` and `agent_history`; override with config.

## Related repos in `Alt-AI-Labs`

- `surrealmcp` — SurrealDB MCP server  
- `saltapimcp` — Salt API MCP server  
- `xapimcp`, `githubmcp` — additional tool sources  

Prompt that defined this crate: `prompts/salt-master-agent.md`.
