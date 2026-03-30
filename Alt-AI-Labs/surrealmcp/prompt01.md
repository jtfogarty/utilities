review the below for correctness

# Upgrade SurrealDB SDK: v2.3.7 → v3.0.0

## Context

Fixes: Cannot connect to SurrealDB v3.0.0 due to WebSocket SubProtocol error.

SurrealDB v3.0.0 (released Feb 17, 2026) changed its WebSocket subprotocol negotiation. The current codebase uses `surrealdb` Rust SDK v2.3.7, which doesn't handle the new v3 protocol, causing:

SubProtocol error: Server sent no subprotocol

## Goal

Upgrade the `surrealdb` crate from `2.3.7` → `3.0.0` and adapt all call sites to the new API.

---

## Breaking Changes to Address

| What Changed | v2 (2.3.7) | v3 (3.0.0) |
|---|---|---|
| `Value` location | `surrealdb::Value` | `surrealdb::types::Value` |
| `sql` module | `surrealdb::sql::ToSql` | Removed — use `surrealdb_types::ToSql` (separate crate) |
| `Value` Display | `val.to_string()` via `Display` | Not implemented — must use `val.to_sql()` via `ToSql` |
| Query response type | `surrealdb::Response` | `surrealdb::IndexedResults` |
| Auth `Root` fields | `&str` (borrowed) | `String` (owned) |
| Value parsing | `Value::from_str()` via `FromStr` | `surrealdb::parse::value()` |
| Value formatting | `val.to_string()` via `Display` | `val.to_sql()` via `ToSql` |
| Response `.take()` | `.take::<T>()` requires `Deserialize` | `.take::<T>()` requires `SurrealValue` |
| WebSocket protocol | Pre-v3 subprotocol | v3 subprotocol negotiation |

---

## Required Changes Per File

### `Cargo.toml`
- Bump `surrealdb` version from `2.3.7` → `3.0.0`
- Add `surrealdb-types = "3.0.4"` as a direct dependency
  - Required because `ToSql` (used in `src/utils/mod.rs`) lives in the standalone `surrealdb-types` crate and is **not** re-exported through the `surrealdb` crate
  - The `surrealdb` crate pulls it in transitively, but the trait must be explicitly imported from `surrealdb_types::ToSql` to be usable
  - Pin the version to match what `surrealdb = "3.0.0"` resolves transitively (check `Cargo.lock` after first build)
- All existing feature flags remain unchanged

### `src/db/mod.rs`
- Update `Root` auth struct fields from `&str` (borrowed) → `String` (owned)

### `src/engine/mod.rs`
- Move `Value` import to `surrealdb::types::Value`
- Replace `surrealdb::Response` with `surrealdb::IndexedResults`

### `src/tools/mod.rs`
- Update `Value` import to `surrealdb::types::Value`
- For `ListNamespaces` / `ListDatabases`: use a JSON roundtrip to extract results
  - Reason: `IndexedResults::take()` now requires the `SurrealValue` derive macro, which depends on `surrealdb_types` as a direct crate. To avoid adding that dependency, take the raw `Value`, serialize to JSON, then deserialize into the target struct. Overhead is negligible for these low-frequency metadata calls.

### `src/utils/mod.rs`
- Replace `Value::from_str()` (via `FromStr`) → `surrealdb::parse::value()`
- Replace `.to_string()` (via `Display`) → `.to_sql()` (via `ToSql` trait)
  - `Value` in surrealdb-types 3.x does **not** implement `Display`, so `.to_string()` will not compile
  - Import `ToSql` as `use surrealdb_types::ToSql;` (requires the `surrealdb-types` direct dependency above)
- Update return type of `convert_json_to_surreal` from `surrealdb::Value` → `surrealdb::types::Value`
  - `surrealdb::sql` module no longer exists in 3.x; `surrealdb::Value` is also gone
  - `surrealdb::types::Value` is the correct path (same type used in `tools/mod.rs` and `engine/mod.rs`)

---

## What Does NOT Change

The following APIs are stable across v2 → v3 and require no changes:

- `any::connect()`
- Feature flags
- `Surreal<Any>` client type
- `.signin()`, `.use_ns()`, `.use_db()`
- `.authenticate()`, `.query()`, `.bind()`

---

## Validation

- [ ] `cargo check` passes
- [ ] All unit tests pass (`cargo test`)
- [ ] Live connection test against a SurrealDB v3.0.0 instance succeeds

---

## Notes

- No package version bump is strictly required, but bumping to `0.5.0

