# Upgrading saltapimcp & surrealmcp to rmcp 1.6.0

Reference: [rmcp 1.6.0 docs](https://docs.rs/rmcp/1.6.0/rmcp/) | [1.x migration guide](https://github.com/modelcontextprotocol/rust-sdk/discussions/716)

## Overview

| Crate | Current rmcp | Target rmcp |
|---|---|---|
| **saltapimcp** | `0.6.0` | `1.6.0` |
| **surrealmcp** | `0.6.4` | `1.6.0` |

The jump from 0.6.x to 1.6.0 spans multiple breaking changes. The biggest is that most public model structs became `#[non_exhaustive]` in 1.0.0 and now require builder-style construction instead of struct literals.

---

## 1. Cargo.toml Changes

### saltapimcp/Cargo.toml

```toml
# Before
rmcp = { version = "0.6.0", features = ["server", "macros", "transport-streamable-http-server"] }

# After
rmcp = { version = "1.6.0", features = ["server", "macros", "transport-streamable-http-server"] }
```

### surrealmcp/Cargo.toml

```toml
# Before
rmcp = { version = "0.6.4", features = [
    "server",
    "macros",
    "transport-streamable-http-server",
    "transport-worker",
] }

# After
rmcp = { version = "1.6.0", features = [
    "server",
    "macros",
    "transport-streamable-http-server",
    "transport-worker",
] }
```

All existing feature names are unchanged in 1.6.0. The SSE transport features were removed in 0.11.0 but neither project uses them.

---

## 2. `ServerInfo` Construction (Both Projects)

The `ServerInfo` struct is now `#[non_exhaustive]`. Struct literal construction no longer compiles.

### saltapimcp — `src/tools/mod.rs`

```rust
// Before
fn get_info(&self) -> ServerInfo {
    ServerInfo {
        protocol_version: ProtocolVersion::V_2024_11_05,
        capabilities: ServerCapabilities::builder().enable_tools().build(),
        server_info: Implementation::from_build_env(),
        instructions: Some(
            "SaltStack MCP server. Use the salt_execute tool to run any Salt command via the local salt-api.".to_string(),
        ),
    }
}

// After
fn get_info(&self) -> ServerInfo {
    ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
        .with_server_info(Implementation::from_build_env())
        .with_instructions("SaltStack MCP server. Use the salt_execute tool to run any Salt command via the local salt-api.")
}
```

Note: `ProtocolVersion` no longer needs to be set explicitly — `ServerInfo::new()` defaults to the latest supported version. If you need to pin it, use `.with_protocol_version(ProtocolVersion::V_2024_11_05)`.

### surrealmcp — `src/tools/mod.rs`

```rust
// Before
fn get_info(&self) -> ServerInfo {
    ServerInfo {
        capabilities: ServerCapabilities::builder()
            .enable_resources()
            .enable_prompts()
            .enable_tools()
            .build(),
        instructions: Some(include_str!("../../server.md").to_string()),
        ..Default::default()
    }
}

// After
fn get_info(&self) -> ServerInfo {
    ServerInfo::new(
        ServerCapabilities::builder()
            .enable_resources()
            .enable_prompts()
            .enable_tools()
            .build(),
    )
    .with_instructions(include_str!("../../server.md"))
}
```

---

## 3. `InitializeResult` Return Type (surrealmcp)

The `initialize` method returns `InitializeResult` which is the same type as `ServerInfo`. The same builder pattern applies.

```rust
// Before (in surrealmcp src/tools/mod.rs, ServerHandler impl)
async fn initialize(
    &self,
    _req: rmcp::model::InitializeRequestParam,
    ctx: RequestContext<RoleServer>,
) -> Result<rmcp::model::InitializeResult, McpError> {
    // ... auth logic ...
    Ok(self.get_info())
}

// After — no change needed to the method body since get_info() already
// returns the correct type. Just make sure the param type name is correct.
// In 1.x it may be `InitializeRequestParams` (with trailing 's').
```

Check whether the type was renamed to `InitializeRequestParams` (with a trailing `s`). If the compiler errors on `InitializeRequestParam`, rename it.

---

## 4. Prompt-Related Types (surrealmcp)

### `Prompt` struct

```rust
// Before (src/prompts/mod.rs)
Prompt {
    name: generator.name().to_string(),
    title: Some(generator.summary().to_string()),
    description: Some(generator.description().to_string()),
    arguments: Some(generator.arguments()),
    icons: None,
}

// After
Prompt::new(
    generator.name(),
    Some(generator.description().to_string()),
    generator.arguments(),
)
.with_title(generator.summary())
```

### `PromptArgument` struct

```rust
// Before
PromptArgument {
    name: "query_type".to_string(),
    title: Some("Query Type".to_string()),
    description: Some("The type of query ...".to_string()),
    required: Some(true),
}

// After
PromptArgument::new("query_type")
    .with_title("Query Type")
    .with_description("The type of query ...")
    .with_required(true)
```

### `GetPromptResult` struct

```rust
// Before (src/tools/mod.rs, get_prompt handler)
Ok(rmcp::model::GetPromptResult {
    description: Some(description),
    messages,
})

// After
Ok(rmcp::model::GetPromptResult::new(messages)
    .with_description(description))
```

---

## 5. Resource-Related Types (surrealmcp)

### `RawResource` struct

```rust
// Before (src/resources/mod.rs)
let raw = RawResource {
    size: Some(size),
    uri: self.uri().to_string(),
    name: self.name().to_string(),
    title: Some(self.name().to_string()),
    mime_type: Some(self.mime_type().to_string()),
    description: Some(self.description().to_string()),
    icons: None,
};

// After — use the new constructor
// RawResource::new(uri, name) then chain .with_*() builders
let raw = RawResource::new(self.uri(), self.name())
    .with_title(self.name())
    .with_mime_type(self.mime_type())
    .with_description(self.description())
    .with_size(size);
```

Check whether `.with_size()` exists. If not, the field may need to be set differently or may have been removed from the builder. The `Annotated::new(raw, None)` wrapper should still work.

### `ListResourcesResult`

```rust
// Before
Ok(rmcp::model::ListResourcesResult {
    resources,
    next_cursor: None,
})

// After — check if it now requires a `meta` field or builder
// In 1.x, list results gained a `meta: None` field.
// If the struct is non_exhaustive, use a constructor:
Ok(rmcp::model::ListResourcesResult {
    resources,
    next_cursor: None,
    meta: None,
})
// Or if that doesn't compile:
// Ok(rmcp::model::ListResourcesResult::new(resources))
```

### `ListPromptsResult`

Same pattern as `ListResourcesResult` — add the `meta: None` field if the compiler requires it, or switch to a builder if the struct is `#[non_exhaustive]`.

---

## 6. `ReadResourceRequestParam` (surrealmcp)

```rust
// Before
async fn read_resource(
    &self,
    req: rmcp::model::ReadResourceRequestParam,
    ...

// After — check if renamed to ReadResourceRequestParams (with 's')
// The field access req.uri stays the same.
```

---

## 7. `PaginatedRequestParam` (surrealmcp)

```rust
// Before
async fn list_prompts(
    &self,
    _req: Option<rmcp::model::PaginatedRequestParam>,
    ...

// After — may be renamed to PaginatedRequestParams
```

---

## 8. `StreamableHttpService` Type Parameter (Both Projects)

In rmcp 1.3.0, the default type parameter was removed from `StreamableHttpService`. If you were relying on the default, you may need to add an explicit type parameter or adjust the usage. However, since both projects use `StreamableHttpService::new()` with a closure returning `Result<impl ServerHandler, _>`, this should continue to work via type inference. If the compiler complains, add the explicit generic:

```rust
// If needed:
let mcp_service = StreamableHttpService::<SaltService>::new(
    move || Ok(SaltService::new(config.clone())),
    session_manager,
    StreamableHttpServerConfig { ... },
);
```

---

## 9. `Implementation` (saltapimcp)

If `Implementation::from_build_env()` still exists, no change. If not:

```rust
// Fallback
Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
```

---

## 10. `ErrorData` / `McpError` Changes

The `ErrorData` type alias should still work. If any `match` arms on `RmcpError` or similar error enums exist, add a `_ => {}` catch-all since these are now `#[non_exhaustive]`.

```rust
// If you match on RmcpError anywhere, add:
_ => { /* handle unknown future variants */ }
```

The `McpError::internal_error(message, data)` pattern used throughout both projects should remain valid.

---

## 11. `meta` Field on List Results

Starting from 0.6.2+, several result types gained an optional `_meta` field. In 1.x with `#[non_exhaustive]`, you may need to use builders or add `meta: None` to struct literals that still compile. The affected types include:

- `ListPromptsResult`
- `ListResourcesResult`
- `ListToolsResult`
- `CallToolResult`
- `ReadResourceResult`

Since `CallToolResult::success()` is used everywhere, tools should be fine. For the list handlers, add `meta: None` or switch to builders.

---

## 12. `serve_server` vs `ServiceExt` (surrealmcp stdio/unix modes)

```rust
// Before
rmcp::serve_server(service.clone(), (tokio::io::stdin(), tokio::io::stdout())).await

// After — in 1.x, the preferred pattern is:
use rmcp::ServiceExt;
service.clone().serve((tokio::io::stdin(), tokio::io::stdout())).await
```

`rmcp::serve_server` may still exist as a convenience, but if it was removed, switch to the `ServiceExt::serve()` method.

---

## Migration Checklist

### Both Projects
- [ ] Bump `rmcp` version in `Cargo.toml` to `"1.6.0"`
- [ ] Update `ServerInfo` construction to use `ServerInfo::new().with_*()`
- [ ] Verify `StreamableHttpService::new()` compiles (type inference)
- [ ] Run `cargo build` and follow compiler errors for any remaining struct-literal breakages

### saltapimcp Only
- [ ] Update `get_info()` in `src/tools/mod.rs`
- [ ] Verify `Implementation::from_build_env()` still exists, else use `Implementation::new()`

### surrealmcp Only
- [ ] Update `get_info()` in `src/tools/mod.rs`
- [ ] Update `initialize()` param type if renamed (`InitializeRequestParam` → `InitializeRequestParams`)
- [ ] Update `Prompt` construction in `src/prompts/mod.rs`
- [ ] Update `PromptArgument` construction in `src/prompts/mod.rs`
- [ ] Update `GetPromptResult` construction in `src/tools/mod.rs`
- [ ] Update `RawResource` construction in `src/resources/mod.rs`
- [ ] Add `meta: None` to `ListPromptsResult`, `ListResourcesResult` if needed
- [ ] Update `PaginatedRequestParam` → `PaginatedRequestParams` if renamed
- [ ] Update `ReadResourceRequestParam` → `ReadResourceRequestParams` if renamed
- [ ] Check `rmcp::serve_server` still exists; if not, use `ServiceExt::serve()`
- [ ] Add `_ => {}` catch-all to any `match` on rmcp error enums

### Final Verification
- [ ] `cargo build` succeeds for both crates
- [ ] `cargo test` passes
- [ ] `cargo clippy` clean

---

## Version Timeline (0.6.x → 1.6.0)

Key breaking changes by version:

| Version | Breaking Change |
|---|---|
| **0.7.0** | Auth error handling changes |
| **0.8.0** | Default schema generation for no-param tools |
| **0.11.0** | SSE transport removed (not used by our projects) |
| **0.12.0** | `cached_schema_for_type` merged into `schema_for_type` |
| **1.0.0** | All model structs become `#[non_exhaustive]`, builder constructors required |
| **1.3.0** | Default type param removed from `StreamableHttpService`; `local` feature added |
| **1.4.0** | `#[tool_router(server_handler)]` shorthand; auto-generated `get_info` |
| **1.5.0** | `2025-11-25` protocol version support added |
| **1.6.0** | Origin header validation, runtime tool disabling, session resumability |
