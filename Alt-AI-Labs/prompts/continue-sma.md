**SaltStack + LLM Master Agent Project Update**  
**Last Updated:** March 29, 2026

### Status: salt-master-agent Binary Created & Built Successfully

Great work — the skeleton delivered from `Alt-AI-Labs/prompts/salt-master-agent.md` is solid and production-oriented. Cargo build + clippy passing on the first try with `rmcp 0.16` pinned to match `rig-core 0.33` is exactly the kind of clean, version-coherent Rust we want.

### Where Is the Memory Using SurrealDB?

**Short answer:**  
Memory is **not yet fully wired** to SurrealDB. It is prepared and partially stubbed via the `SurrealAuditHook` in `src/agent.rs`, but the actual persistence still routes through MCP tool calls that need to be connected to your running **SurrealMCP** instance.

Here’s the precise current state and what “replace all previous memory/history logic” looks like in practice right now:

#### 1. Where Memory Logic Lives Today (in the Generated Code)
- `src/agent.rs` contains:
  - `SurrealAuditHook` that implements `rig::agent::PromptHook`
  - Hooks for:
    - `on_tool_call`
    - `on_tool_result`
    - `persist_reflection` after each turn
  - Startup logic that does a “select for recent history”
- The system prompt (set during `AgentBuilder`) includes the strict rule: “You MUST use SurrealMCP tools for ALL memory, history, reflections, and long-term storage. Never maintain local state or buffers.”
- No in-memory `Vec<ChatMessage>`, no custom history structs, no direct `surrealdb` crate calls — this part of the replacement is already enforced by design.

#### 2. How SurrealDB Is Actually Used (The MCP Layer)
All real SurrealDB interaction happens **exclusively through SurrealMCP tools**, not direct DB connections inside `salt-master-agent`.

- In `src/mcp.rs` you have `McpClientHandler::connect` for each endpoint, including the SurrealMCP one (configured via `SMA_mcp__surreal_url` or similar in `config.rs`).
- At agent build time (`build_agent()`), Rig + rmcp discovers the tools exposed by SurrealMCP automatically:
  - `store_action_log`
  - `retrieve_recent_history`
  - `store_reflection`
  - `query_gig_outcomes`
  - `store_minion_event`
  - etc. (whatever tools your running SurrealMCP instance advertises)
- The `SurrealAuditHook` is designed to call those discovered tools (or trigger the LLM to call them via forced `ToolChoice::Required`).
- On startup, the agent calls the history retrieval tool to bootstrap context instead of loading from local memory.

This is exactly the “MCP-first, SurrealMCP as single source of truth” pattern we decided on.

#### 3. What Is Still Missing / Needs Wiring (Immediate Next Steps)
The hook exists, but the actual MCP client for SurrealMCP must be:
- Registered in `initialize_mcp_clients()` (in `mcp.rs`)
- Passed into `AgentBuilder` so its tools are discovered and available
- Explicitly used inside `SurrealAuditHook` methods (or the LLM is forced to call the tools directly)

To complete the replacement:

**Action 1 – Verify SurrealMCP is running**
On the Alt AI Master node:
```bash
# Check SurrealMCP service
systemctl status surreal-mcp   # or however you start it
# or
curl http://localhost:8080/health   # adjust port as configured
```

**Action 2 – Update config to point to SurrealMCP**
Add/edit `salt-master-agent.toml` or env vars:
```toml
[mcp.surreal]
url = "http://localhost:8080"   # or your SurrealMCP HTTP endpoint
# namespace, database, auth if needed
```

**Action 3 – Wire SurrealMCP client in code**
In `src/mcp.rs` and `src/agent.rs`, ensure the SurrealMCP `McpClientHandler` is created and its tools are added to the agent (Rig’s `.with_mcp_client()` or equivalent with `rmcp 0.16`).

**Action 4 – Test the memory flow**
Run:
```bash
cargo run --release -- repl
```
Then give a prompt like:
> Remember that my goal is to build 3 Polymarket trading bots. Now list recent history.

The agent should:
- Use a SurrealMCP tool to store the intent (via hook or direct tool call)
- Use a SurrealMCP tool to retrieve history
- Show no local state leakage

Once that works cleanly, the replacement is complete — all history, reflections, audit logs, and cluster events live only in SurrealDB via SurrealMCP.

### Recommended Next Move
Open `src/agent.rs` and `src/mcp.rs` in Cursor/Antigravity.

Paste this targeted follow-up prompt into Cursor (or I can refine it):

```
You are continuing work on salt-master-agent. 
The SurrealAuditHook and memory replacement design is already in place.
Finish wiring the SurrealMCP client:
1. In mcp.rs, add SurrealMCP connection with the same retry/backoff pattern as the others.
2. In agent.rs, register the SurrealMCP client so its tools (store_*, retrieve_*, persist_reflection) are discovered.
3. Ensure SurrealAuditHook actually invokes the discovered SurrealMCP tools in on_tool_call / on_tool_result / persist_reflection.
4. Update the system prompt if needed and add a startup call to retrieve_recent_history(10).
5. Keep everything compile-time safe with rmcp 0.16 + rig-core 0.33.
Generate the exact diffs or updated functions.
```

Run `cargo check` after each change.

Once this is wired and tested, memory is 100% SurrealDB-backed via MCP, and we can move on to integrating `saltapimcp` tools and the X bookmark puller.

Drop the output of `cargo run --release -- repl` with a simple memory test prompt if you want me to debug the flow live, or share the relevant sections of `agent.rs` / `mcp.rs` for precise patch guidance.

What do you want to tackle first — wiring the SurrealMCP client, or a quick test run?