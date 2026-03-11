**Yes, this warning is expected — and it's the reason your `http::get()` is failing.**

You're running **SurrealDB embedded inside Rust compiled to WASM** (`wasm32-unknown-unknown` target, likely in the browser or a WASM runtime like wasm-bindgen / wasmtime in a browser-like context).

In this environment:

- There is **no `--allow-net` flag** you can pass (that's a CLI/server-only thing).
- The **default capability for outgoing network** is **denied**.
- The `http::get()` / `http::post()` etc. functions are **still executed by the SurrealQL runtime** inside the WASM module.
- So `surrealdb_core::ctx::context` correctly blocks the request and logs exactly the warning you're seeing:

```
Capabilities denied outgoing network connection attempt, target: 'fake-json-api.mock.beeceptor.com:443'
```

### How to fix it in Rust + WASM (3.x)

You must explicitly enable the network capability **when you create the embedded client**.

```rust
use surrealdb::Surreal;
use surrealdb::engine::local::Mem;           // or whatever storage you use (Mem, RocksDb, etc.)
use surrealdb::opt::capabilities::Capabilities;
use surrealdb::opt::capabilities::NetTarget;

// Build the exact allowlist you want
let caps = Capabilities::default()
    .with_net(NetTarget::from_host("fake-json-api.mock.beeceptor.com"))   // exact domain
    // .with_net(NetTarget::from_host_port("fake-json-api.mock.beeceptor.com", 443))
    // or allow multiple:
    // .with_net(NetTarget::from_hosts(vec!["example.com", "api.stripe.com"]))
    .with_allow_net_all(false);   // important: do NOT allow everything

// Create the embedded database with the capability
let db = Surreal::new::<Mem>(caps).await?;
// or for other engines: Surreal::new::<RocksDb>(caps).await? etc.

// Then use it normally
db.use_ns("test").use_db("test").await?;

let query = "INSERT INTO user (SELECT * FROM http::get('https://fake-json-api.mock.beeceptor.com/users'));";
let result: Vec<serde_json::Value> = db.query(query).await?.take(0)?;
```

### Important notes for WASM/browser

- In the **browser** (via the official `@surrealdb/wasm` or your own Rust→WASM build), the actual HTTP request is still made by the browser's `fetch()` under the hood. So CORS still applies, and the browser's own network restrictions apply.
- `NetTarget` supports:
  - `NetTarget::from_host("example.com")`
  - `NetTarget::from_host_port("example.com", 443)`
  - `NetTarget::from_ip("1.2.3.4")`
  - CIDR ranges, wildcards, etc. (check the `surrealdb::opt::capabilities::NetTarget` docs)
- Always start with **allow specific hosts only** — never `with_allow_net_all(true)` in production.

### Quick test

If you just want to confirm it works, temporarily do:

```rust
let caps = Capabilities::default().with_allow_net_all(true);
```

(then remove it again once you see it succeed).

This behavior has been consistent since the capabilities system was introduced and did **not** change in 3.x. The warning you see is the security system working as designed.

Let me know which storage engine you're using (`Mem`, `IndexedDb`, `RocksDb`, etc.) or if you're compiling with `wasm-bindgen`, and I can give you the exact one-liner.