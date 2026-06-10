**SaltStack + LLM Master Agent Project - Dependency & Version Synchronization**  
**Status:** May 03, 2026 | Ready for unified Cargo workspace alignment

### Current Project Versions Table

| Crate Name            | Current Version | Suggested Next Version | Notes / Recommendation |
|-----------------------|-----------------|------------------------|------------------------|
| **salt-master-agent** | 0.1.1           | **0.2.0**              | Major bump: now fully Rig + rmcp MCP-only. Align edition to 2024. |
| **surrealmcp**        | 0.4.4           | **0.5.0**              | Stable SurrealDB 3.x integration. Ready for 0.5.0 with new features. |
| **saltapimcp**        | 0.2.0           | **0.3.0**              | Core Salt REST wrapper. Bump after Reactor support. |
| **xapimcp**           | 0.1.0           | **0.2.0**              | X/Twitter API MCP. Update edition + add rate-limiting. |
| **xbookmark-puller**  | 0.1.0           | **0.2.0**              | Depends on xapimcp + SurrealMCP. Ready for scheduled pulls. |

### Recommended Synchronization Plan (Pure-Rust Alignment)

Create a **workspace** `Cargo.toml` at the root to keep everything in sync:

```toml
[workspace]
members = [
    "salt-master-agent",
    "surrealmcp",
    "saltapimcp",
    "xapimcp",
    "xbookmark-puller",
]
resolver = "2"

[workspace.package]
edition = "2024"          # Standardize on 2024
rust-version = "1.85"     # Current stable baseline
authors = ["Jeff Fogarty <jtfogarty>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourorg/salt-master-agent"

[workspace.dependencies]
# Core shared crates (pinned to latest stable as of May 2026)
anyhow = "1.0"
axum = "0.8.9"
clap = { version = "4.6.1", features = ["derive", "env"] }
reqwest = { version = "0.13.3", default-features = false, features = ["json", "http2", "rustls-tls"] }
rmcp = { version = "1.6.0", features = ["server", "macros", "transport-streamable-http-server"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.52.1", features = ["full", "signal"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

#### Key Updates to Apply Across Projects

- **rig-core**: Already at `0.36` in `salt-master-agent` → keep (latest).
- **surrealdb**: `3.0.5` (latest) in surrealmcp.
- **axum**: Upgrade all to `0.8.9`.
- **tokio**: Standardize on `1.52.1`.
- **clap**: `4.6.1`.
- **reqwest**: `0.13.3`.
- **rmcp**: `1.6.0` (already excellent across projects).

**Next Actions (Immediate):**
1. Add the workspace `Cargo.toml`.
2. Update each project's `[package]` to inherit from workspace where possible + bump versions as suggested.
3. Run `cargo update` in root.
4. Test `cargo build --workspace`.
5. Commit the synchronized state.

This gives us a clean, reproducible, production-grade Rust monorepo aligned with the latest ecosystem (as of May 2026). Let me know if you want the full updated TOML snippets for each crate! 🚀