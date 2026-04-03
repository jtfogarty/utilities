//! Load `.env` before `std::env::var` / clap env parsing.
//!
//! 1. `CARGO_MANIFEST_DIR/.env` (crate root — works even when cwd is elsewhere)
//! 2. `.env` in current working directory (overrides)

pub fn load() {
    let crate_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_filename(&crate_env);
    let _ = dotenvy::dotenv();
}
