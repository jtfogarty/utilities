You are an expert Rust systems engineer working on the saltapimcp project.

Project Context:
- saltapimcp is a standalone pure-Rust binary/service that exposes SaltStack functionality via the Model Context Protocol (MCP) so the LLM Master Agent (Rig + Ollama) can control the cluster.
- It already wraps the Salt REST API (using reqwest for auth, commands, state.apply, etc.).
- It integrates with SurrealMCP to store events/history/cluster state in SurrealDB.
- Uses Tokio for async, serde/serde_json, tracing for logging, rmcp (or equivalent) for MCP server, config via TOML/CLI/env.
- Follow the exact existing code style, module layout, error handling, and MCP tool definition patterns already in the project.

We are now implementing **Salt Reactor logic** inside saltapimcp.

What is Salt Reactor?
Salt Reactor is a real, core SaltStack feature (not hallucinated). The Salt Master has an internal ZeroMQ event bus. Every operation fires a tagged event (e.g. salt/minion/*/start, salt/key, salt/job/*/ret, salt/beacon/*, custom events). The Reactor listens for tag patterns and runs reactions (wheel.key.accept, local.state.apply, runner.*, etc.).

Why we need it in saltapimcp:
- Turns the system from "agent tells Salt what to do" into true self-aware, autonomous infrastructure.
- Automatic minion detection/provisioning, event storage in SurrealDB for LLM reflection, dynamic rules, and no polling.
- Events flow → SurrealMCP → LLM Master Agent can react or schedule work (e.g. new minion → auto-accept key + highstate + notify agent).

How it must work (exact requirements for error-free first compile):

1. Configuration
   Add to existing config.toml:
   ```toml
   [reactor]
   enabled = true
   salt_api_base_url = "https://saltmaster:8000"
   salt_api_events_url = "https://saltmaster:8000/events"  # SSE endpoint
   auth_token = "..."  # or eauth fallback
   reconnect_backoff_ms = 1000
   ```

2. Event Listener (background Tokio task)
   - Connect to GET {salt_api_events_url} as Server-Sent Events (Content-Type: text/event-stream).
   - Use authentication (X-Auth-Token header; login via /login if token missing).
   - Implement automatic reconnection with exponential backoff.
   - Parse each SSE message into:
     ```rust
     #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
     pub struct SaltEvent {
         pub tag: String,
         pub data: serde_json::Value,
         pub timestamp: chrono::DateTime<chrono::Utc>,
         pub id: Option<String>,  // for dedup
     }
     ```
   - Recommended crates to add to Cargo.toml (add these only):
     ```toml
     eventsource-client = "0.13"  # or reqwest-eventsource if you prefer
     glob = "0.3"
     chrono = { version = "0.4", features = ["serde"] }
     thiserror = "1"
     ```

3. Reactor Rules Engine
   - Load rules from config + allow runtime registration via MCP.
   - Rule example in TOML:
     ```toml
     [[reactor.rules]]
     name = "auto_accept_new_minion"
     tag_pattern = "salt/minion/*/start"
     enabled = true
     actions = [
       { type = "wheel", module = "key.accept", args = { "match" = "{{ data.id }}" } },
       { type = "local", tgt = "{{ data.id }}", module = "state.apply", args = ["base"] }
     ]
     ```
   - Simple glob matching on tag (use `glob` crate).

4. Reaction Executor
   - When rule matches: store event in SurrealDB (table `salt_events`) via existing SurrealMCP.
   - Execute actions by reusing the existing Salt API client in saltapimcp (wheel.key.*, local.state.apply, local.cmd.run, runner.*).
   - Support at minimum the four reaction types from Salt Reactor (local, runner, wheel, caller).

5. New MCP Tools (must be discoverable by the LLM agent)
   - `get_recent_events(limit: u32, tag_filter: Option<String>) -> Vec<SaltEvent>`
   - `list_reactor_rules() -> Vec<ReactorRule>`
   - `register_reactor_rule(rule: ReactorRule) -> bool`
   - `remove_reactor_rule(name: String) -> bool`
   - `get_event_stream_status() -> ConnectionStatus`
   - Use exact same MCP tool macro/schema pattern already in the project (typed, JSON schema, forced calling).

Technical / Production Requirements (must compile and run error-free first time):
- Event listener runs in its own spawned Tokio task; does not block MCP server.
- Graceful shutdown (Ctrl+C stops listener cleanly).
- Full structured tracing logs for every event received, rule match, action taken, and errors.
- Robust error handling with thiserror (no unwrap/panic in hot path).
- Deduplication and rate limiting if events flood.
- Store every event in SurrealDB with proper indexing for fast LLM queries.
- Start listener automatically if `reactor.enabled = true`.
- Add health-check via existing MCP or HTTP endpoint.

Deliverables I expect:
1. First, show exact Cargo.toml additions and new file structure (src/reactor/mod.rs, src/reactor/events.rs, src/reactor/rules.rs, etc.).
2. Then provide full code for all new files + any minimal changes to main.rs or existing modules.
3. Include comprehensive comments explaining every part.
4. Make sure it integrates perfectly with the existing Salt API client and SurrealMCP connection.

Generate production-ready, idiomatic Rust (2026 standards) that will compile and run cleanly on the first try. Do not add extra features beyond these requirements.