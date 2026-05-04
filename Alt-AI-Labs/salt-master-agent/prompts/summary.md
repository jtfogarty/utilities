
Using the surrealDB MCP, bookmarks namespace, v1 database, confirm all table names and structures 

**Cursor Prompt — Implement X Bookmark LLM Summarization Feature**

Project context: We are in the pure-Rust `salt-master-agent` binary (Rig + rmcp + SurrealMCP + Ollama). Bookmarks are already being loaded into SurrealDB table `bookmark_annotations`. Example record:

```json
{
  "bookmark_id": "1788009838951235802",
  "created_at": "2026-04-07T23:17:34.060Z",
  "id": "bookmark_annotations:byg5m9jepf7wss8flmu2",
  "notes": "This shows that the Book of Revelation is a big chiastic structure",
  "updated_at": "2026-04-08T01:09:25.945Z",
  "urls": ["t.co/sNW4AbdUSL"]
}
```

The actual bookmark **content** (full tweet text) is available either in the `notes` field or via xapimcp (we will query it). Goal: automatically generate a clean summary of what the tweet/bookmark is about and ensure any URLs (especially GitHub) are stored.

**Requirements — implement exactly:**

1. **Database change** (via SurrealMCP):
   - Add two new optional fields to the `bookmark_annotations` table definition / schema:
     - `summary: string` (the LLM-generated 1-2 sentence summary).
     - `extracted_urls: array<string>` (full clean URLs; merge with existing `urls` field).
   - Create a helper function `mark_as_processed` that sets `summary` and `extracted_urls` and updates `updated_at`.

2. **New Rust module** (`src/agents/bookmark_processor.rs` or add to existing agent code):
   - Use the existing SurrealMCP client to query all records where `summary IS NONE OR summary = ""`.
   - For each record, retrieve the full bookmark **content** (prefer `notes` if it contains the tweet text; otherwise call xapimcp to fetch by `bookmark_id`).
   - Call the local Ollama LLM (`llama3.1:8b`) with this **exact system prompt** (hard-coded, never change):

     ```
     You are a precise bookmark summarizer. Given the full text of an X/Twitter bookmark, produce:
     - A single concise summary sentence (max 2 sentences) describing exactly what the bookmark is about.
     - Extract EVERY full URL mentioned (especially GitHub links). Return them as a clean JSON array.
     Do not add commentary. Output ONLY valid JSON:
     {"summary": "...", "extracted_urls": ["https://..."]}
     ```

   - Parse the JSON response.
   - Update the record in SurrealDB using SurrealMCP (type-safe).
   - Log every action to SurrealDB history for audit/self-improvement.

3. **Expose as MCP tool**:
   - Add a new discoverable tool to the Rig agent: `summarize_unsummarized_bookmarks` (no parameters, or optional `limit: u32`).
   - Tool schema must be strict (use rmcp macros).
   - The tool should process up to the limit (default 10) and return a summary of what was summarized.

4. **Integration**:
   - Add the tool to the main agent’s tool registry in `main.rs`.
   - (Optional but recommended) Add a simple CLI flag `--process-bookmarks` and a background reactor-style loop (every 5 min) that auto-calls the tool on new bookmarks.
   - Use the existing error handling and logging patterns.

5. **Testing**:
   - Include a test that uses the example bookmark text provided by the user.
   - Verify summary is stored and URLs are extracted correctly.

Use existing patterns from SurrealMCP and xapimcp. Keep everything compile-time safe, no `unwrap`, full error propagation. Single static binary, no new dependencies.

Implement this now and show me the diff when complete.

---

Paste that entire block into Cursor (or your Cursor agent) and let it generate the code. It will slot perfectly into our current architecture.

This feature is now officially “In Progress” and will be marked complete once the PR is merged and tested with your live bookmarks. Let me know when it’s running and we’ll immediately hook it into the next autonomous step (GitHub project detection).