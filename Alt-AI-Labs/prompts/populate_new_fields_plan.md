# Plan: `populate_new_fields` CLI Command

## Goal
Add a `populate_new_fields` subcommand to `x_bookmark_puller` that back-fills the `note_tweet` and `article` columns for existing `x_bookmarks` records by fetching each tweet from the X v2 API.

---

## Context

The current pull cycle already stores `note_tweet` and `article` when creating new bookmark records (see `tweet_note_tweet_and_article` at main.rs:608-613). However, bookmarks archived before those columns were added have `null` values. This command fills in those gaps.

---

## Steps

### 1. Add `populate_new_fields` as a CLI subcommand

**File:** `xbookmark-puller/src/main.rs`

- Replace the flat `Cli` struct (lines 95-104) with a `clap` subcommand enum:
  ```
  Cli
    └─ Subcommand::Pull        (existing default behavior, keeps --dry-run)
    └─ Subcommand::PopulateNewFields
    └─ Subcommand::TestSurrealWrite  (move --test-surreal-write here)
  ```
- Route to the appropriate handler in `main()`.

### 2. Add `get_tweet` tool to xapimcp

**File:** `xapimcp/src/x/mod.rs`

- Add a new function `get_tweet(token, tweet_id) -> Result<Value>`:
  ```
  GET https://api.x.com/2/tweets/{tweet_id}
      ?tweet.fields=note_tweet,article,text,created_at,author_id
  Authorization: Bearer {token}
  ```
- This is a single-tweet lookup; the X v2 endpoint rate limit for tweet lookup is **300 requests per 15 minutes** (app-level) or **900 per 15 minutes** (user-level with OAuth 2.0 user context).

**File:** `xapimcp/src/tools/mod.rs`

- Register a new MCP tool `get_tweet` with parameter `tweet_id` (string).
- Wire it to the `get_tweet` HTTP function.

### 3. Query all bookmark IDs from SurrealDB

**File:** `xbookmark-puller/src/main.rs`

- Add function `surreal_select_all_bookmark_ids(client) -> Vec<String>`:
  ```surrealql
  SELECT bookmark_id FROM x_bookmarks
  ```
  Parse the SurrealMCP response and collect all `bookmark_id` values.

- Optionally filter to only rows where `note_tweet IS NULL AND article IS NULL` to skip already-populated records:
  ```surrealql
  SELECT bookmark_id FROM x_bookmarks
  WHERE note_tweet IS NULL OR note_tweet = NONE
  ```

### 4. Fetch each tweet and update the record

**File:** `xbookmark-puller/src/main.rs`

- Add function `populate_new_fields(client_surreal, client_x, rate_limiter)`:
  1. Call `surreal_select_all_bookmark_ids` to get the list.
  2. Log total count.
  3. For each `bookmark_id`:
     a. Acquire a GET rate-limiter slot (reuse existing `acquire_get`).
     b. Call `get_tweet` MCP tool with the `bookmark_id`.
     c. Extract `note_tweet` and `article` from the response.
     d. If either field has data, update the SurrealDB record:
        ```surrealql
        UPDATE x_bookmarks:{bookmark_id} MERGE {
          note_tweet: <value_or_null>,
          article: <value_or_null>
        }
        ```
        Use SurrealMCP `update` tool with merge mode.
     e. Log progress (e.g., `"updated 42/1500"`).
  4. Log completion summary (total processed, updated, skipped, errors).

### 5. Rate limiting for tweet lookups

- **Reuse the existing GET governor** (180 per 15 min, 1 cell every 5 seconds).
  - The tweet lookup endpoint actually allows 300/15min for app-auth or 900/15min for user-auth, so the existing 180/15min governor is conservative and safe.
- **Reuse the existing 429 retry logic** (`x_error_is_documented_rate_limit` + 5-retry loop with 2s sleep).
- After every 150 lookups, optionally log a rate-limit checkpoint with remaining burst capacity.

### 6. Handle edge cases

| Case | Handling |
|------|----------|
| Tweet deleted or unavailable | Log warning, skip to next. Do not update the record. |
| `note_tweet` and `article` are both absent | Skip update, log at debug level. |
| Record already has both fields populated | Skip (filtered in the query, step 3). |
| Network / MCP failure mid-run | Bail with error; re-running the command is idempotent since the query filters already-populated rows. |
| Empty bookmark table | Log info "no bookmarks to process" and exit cleanly. |

### 7. Wire it all up in `main()`

```rust
match cli.command {
    Command::Pull { dry_run } => { /* existing run_pull_cycle logic */ }
    Command::PopulateNewFields => {
        // connect to SurrealMCP + xapimcp
        // init GET rate limiter (no DELETE limiter needed)
        // call populate_new_fields()
        // disconnect and exit
    }
    Command::TestSurrealWrite => { /* existing test logic */ }
}
```

This is a **one-shot command** (no scheduler loop). It runs to completion and exits.

---

## Files Changed

| File | Change |
|------|--------|
| `xapimcp/src/x/mod.rs` | Add `get_tweet` HTTP function |
| `xapimcp/src/tools/mod.rs` | Register `get_tweet` MCP tool |
| `xbookmark-puller/src/main.rs` | Refactor CLI to subcommands; add `surreal_select_all_bookmark_ids`, `surreal_update_bookmark_fields`, `populate_new_fields` functions; wire into `main()` |

---

## Testing

1. **`--test-surreal-write`** still works after CLI refactor.
2. **`populate_new_fields`** on a DB with a known bookmark (e.g., `1804016740822548855`) fetches and stores `note_tweet`/`article`.
3. **Re-running** `populate_new_fields` skips already-populated records (idempotent).
4. **Rate limiter** pacing is visible in logs (governor acquire messages at debug level).
5. **Deleted tweet** is handled gracefully (logged, not fatal).

---

## Execution Order

```
1. Add get_tweet to xapimcp          (independent)
2. Refactor CLI to subcommands       (main.rs)
3. Add SurrealDB query/update fns    (main.rs)
4. Add populate_new_fields fn        (main.rs, depends on 1-3)
5. Wire into main()                  (main.rs, depends on 2,4)
6. Test                              (depends on all above)
```

Steps 1 and 2 can be done in parallel.
