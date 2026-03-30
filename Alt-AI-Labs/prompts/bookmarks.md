You are building a reliable, asynchronous X bookmark archiver binary.

Create a complete standalone Rust binary called `xbookmark-puller` in the Alt-AI-Labs folder.

**Requirements:**
- Runs continuously as a systemd service on the SaltMaster 24/7.
- Periodically calls the `pull_x_bookmarks` tool from xapimcp (configurable interval via environment variable `PULL_INTERVAL_MINUTES`, default 15; handle pagination and respect rate limits).
- For every bookmark returned:
  - Extract the unique `bookmark_id` (the X tweet_id / bookmark id).
  - Use SurrealMCP to check if a record with that `bookmark_id` already exists in table `x_bookmarks`.
  - If it does NOT exist:
    - Store the bookmark as a structured record via SurrealMCP (table: `x_bookmarks`, fields: `bookmark_id` [string, primary key], `text`, `url`, `author`, `created_at`, `raw_json`, `pulled_at` [timestamp], `processed` [bool = false]).
    - Immediately after successful insert, call the delete tool from xapimcp to remove the bookmark from the X account.
  - If it already exists, skip and do not delete again.
- No analysis, planning, or further actions — only pull → dedupe → store → delete.
- Include proper logging (tracing), error handling, exponential backoff, and graceful shutdown on SIGTERM.
- Generate the full `Cargo.toml` (with all needed dependencies) and complete `src/main.rs`.

After generating the code, also output the exact shell commands (or saltapimcp tool calls) needed to:
1. Copy the binary to the SaltMaster.
2. Compile it.
3. Install and enable the systemd service so it starts automatically.

Output ONLY the code files and the deployment steps. Begin.
