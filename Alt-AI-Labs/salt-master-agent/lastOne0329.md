**SaltStack + LLM Master Agent Project Update**  
**Last Updated:** March 29, 2026

### Status: Memory Replacement with SurrealMCP – Excellent Progress

The implementation you just described is **very close to complete**. The architecture now correctly follows our MCP-first design:

- No in-memory history structs remain.
- All persistence flows through `SurrealMemoryTools` resolution + `call_mcp_tool`.
- `SurrealAuditHook` intercepts tool calls/results and reflections.
- Startup pulls recent history via `retrieve_recent_history(10)`.
- System prompt enforces SurrealMCP usage.
- Fallback to generic `insert`/`select`/`query` works with stock SurrealMCP (which only exposes those core tools today).
- Config overrides and heuristics make it robust when we later add semantic tools (`store_action_log`, etc.).

**Current State of Memory Using SurrealDB**  
Memory **is now using SurrealDB exclusively via SurrealMCP**.  
Every action, reflection, tool call, and history entry is persisted by the hook calling resolved MCP tools (or falling back to `insert`/`select`). The LLM sees the full SurrealMCP tool list and is forced (via `ToolChoice::Required` + strict prompt) to stay within the MCP contract. On restart, context bootstraps from the DB. This satisfies “Replace all previous memory/history logic with SurrealMCP tools.”

The only remaining gap is semantic richness: stock SurrealMCP gives raw `insert`/`select`/`query`, so the hooks use those today. When we enhance SurrealMCP with domain-specific tools (`store_action_log`, `persist_reflection`, `retrieve_recent_history`, `store_minion_event`, etc.), the resolver will pick them up automatically (or via config overrides) and the LLM will prefer the cleaner, typed interfaces.

### Verification Steps (Run These Now)

On the Alt AI Master node (where SurrealMCP + SurrealDB are running):

1. Confirm SurrealMCP is exposing tools:
   ```bash
   # If it has a health or tools endpoint, or just run the agent and watch logs
   ```

2. Start the agent in REPL mode:
   ```bash
   cd /Users/jefffogarty/Documents/projects/utilities/Alt-AI-Labs/salt-master-agent
   cargo run --release -- repl
   ```

3. Test the memory flow with these prompts (one at a time):
   - “Remember that I want to build 3 Polymarket trading bots using SaltStack minions.”
   - “What is my most recent goal or remembered intent?”
   - “Store a reflection: The last deployment attempt failed because minion X was offline.”
   - “Show recent history.”

Watch the structured JSON logs for:
- `on_tool_call` / `on_tool_result` → `store_action_log` or `insert` calls
- `persist_reflection` calls
- `retrieve_recent_history` on startup or explicit request

If you see clean MCP tool calls going to SurrealMCP and data appearing in SurrealDB (check with Surrealist or `surreal sql`), the replacement is live and working.

### Recommended Immediate Next Actions

**Option A (Quick Polish – Do This First)**  
Enhance the resolution heuristics and fallback messages so the LLM knows exactly which tools are available right now.

Paste this into Cursor as a targeted continuation prompt:

```
You are continuing work on salt-master-agent (rmcp 0.16 + rig-core 0.33).

The SurrealMemoryTools resolution and SurrealAuditHook are implemented.
Improve the memory layer for better transparency and future-proofing:

1. In mcp.rs → resolve_surreal_memory_tools: After resolution, log (tracing::info!) the exact tool names chosen (e.g. "Using store_action_log: 'insert', retrieve: 'select'"). If no semantic tools found, log "Falling back to generic SurrealMCP CRUD tools".

2. In agent.rs → SurrealAuditHook and SYSTEM_PREAMBLE: Dynamically include the resolved tool names in the system prompt so the LLM knows precisely what to call (e.g. "For storing actions use the tool named 'insert' with table 'agent_actions' ...").

3. Add a new MCP tool call at startup (after retrieve_recent_history) that calls a simple "list_tables" or "query" to confirm SurrealDB connectivity and log the available tables (agent_history, reflections, etc.).

4. In persist_row / persist_reflection: If fallback to insert/select is used, add a small structured note in the payload so we can later query "records inserted via fallback".

Keep changes minimal, compile-safe, and idiomatic. Generate exact updated functions with diffs.

Then run cargo clippy --bin salt-master-agent --fix --allow-dirty --allow-staged
```

**Option B (Bigger Step)**  
Start extending SurrealMCP itself with the semantic tools we want (`store_action_log`, `persist_reflection`, etc.) so the resolver picks the nice names instead of raw CRUD. This is the clean long-term state.

Let me know which option you want to tackle, or drop the output/logs from the REPL memory test prompts above and I’ll give precise debugging/fixes.

Once this memory layer is rock-solid and tested, we can move straight to wiring `saltapimcp` tools into the same `McpRuntime` and start executing real SaltStack commands from natural language.

Ready when you are — what’s the next command or file you want to iterate on?