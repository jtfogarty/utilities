//! Native Rig tool that walks `bookmark_annotations` rows whose `summary` is NONE / empty,
//! asks Ollama for a strict-JSON `{summary, extracted_urls}` payload, and writes the result
//! back through SurrealMCP. Every action is mirrored into the agent history table.
//!
//! Tool name advertised to the LLM: `summarize_unsummarized_bookmarks`.
//! Args (all optional): `{ "limit": <u32, default = config> }`.
//! The schema is intentionally narrow so the model cannot invent extra parameters.

use std::sync::Arc;
use std::time::Duration;

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use rmcp::model::{CallToolRequestParams, CallToolResult, RawContent};
use rmcp::service::ServerSink;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use crate::config::AppConfig;

/// Hard-coded summarizer system prompt. **Never** edit at runtime — the surrounding system
/// (deduping, schema validation, and downstream GitHub detection) depends on this exact wording.
pub const SUMMARIZER_SYSTEM_PROMPT: &str = "You are a precise bookmark summarizer. Given the full text of an X/Twitter bookmark, produce:\n- A single concise summary sentence (max 2 sentences) describing exactly what the bookmark is about.\n- Extract EVERY full URL mentioned (especially GitHub links). Return them as a clean JSON array.\nDo not add commentary. Output ONLY valid JSON:\n{\"summary\": \"...\", \"extracted_urls\": [\"https://...\"]}";

/// Tool name registered on the agent's `ToolServerHandle`.
pub const TOOL_NAME: &str = "summarize_unsummarized_bookmarks";

/// LLM-facing arguments for the tool. The schema is published via [`Tool::definition`] below.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SummarizeArgs {
    /// Optional maximum number of bookmarks to process in this invocation.
    /// Defaults to [`AppConfig::bookmark_summarize_default_limit`] when absent.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Per-record outcome surfaced back to the LLM.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessedBookmark {
    pub bookmark_id: String,
    pub summary: String,
    pub extracted_urls: Vec<String>,
    pub source: String,
}

/// Top-level tool output.
#[derive(Debug, Clone, Serialize)]
pub struct SummarizeOutput {
    pub processed: u32,
    pub skipped: u32,
    pub items: Vec<ProcessedBookmark>,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BookmarkProcessorError {
    #[error("SurrealMCP call_tool({0}) transport error: {1}")]
    SurrealTransport(&'static str, String),
    #[error("SurrealMCP {0} returned tool error: {1}")]
    SurrealToolError(&'static str, String),
    #[error("xapimcp call_tool({0}) error: {1}")]
    XapiTransport(String, String),
    #[error("Ollama HTTP error: {0}")]
    Ollama(String),
    #[error("Ollama JSON output was not parseable: {0}; raw={1}")]
    OllamaJson(serde_json::Error, String),
    #[error("SurrealDB query response could not be parsed: {0}")]
    Parse(String),
}

/// State the tool needs to execute. Cloned cheaply (Arc inside).
#[derive(Clone)]
pub struct BookmarkProcessor {
    inner: Arc<Inner>,
}

struct Inner {
    surreal_sink: ServerSink,
    x_sink: Option<ServerSink>,
    ollama_base_url: String,
    ollama_model: String,
    http: reqwest::Client,
    table: String,
    history_table: String,
    default_limit: u32,
    x_get_tweet_tool: String,
}

impl BookmarkProcessor {
    pub fn new(
        cfg: &AppConfig,
        surreal_sink: ServerSink,
        x_sink: Option<ServerSink>,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("reqwest client builder cannot fail with default settings");
        Self {
            inner: Arc::new(Inner {
                surreal_sink,
                x_sink,
                ollama_base_url: cfg.ollama_base_url.trim_end_matches('/').to_string(),
                ollama_model: cfg.ollama_summarize_model.clone(),
                http,
                table: cfg.bookmark_annotations_table.clone(),
                history_table: cfg.surreal_history_table.clone(),
                default_limit: cfg.bookmark_summarize_default_limit,
                x_get_tweet_tool: cfg.x_get_tweet_tool.clone(),
            }),
        }
    }

    /// Public entry point used by both the Rig tool dispatch and the optional reactor loop.
    pub async fn process(
        &self,
        limit: Option<u32>,
    ) -> Result<SummarizeOutput, BookmarkProcessorError> {
        let limit = limit.unwrap_or(self.inner.default_limit).clamp(1, 500);
        let candidates = self.fetch_candidates(limit).await?;
        info!(
            candidate_count = candidates.len(),
            limit, "summarize_unsummarized_bookmarks: fetched candidate batch"
        );

        let mut processed = Vec::new();
        let mut skipped: u32 = 0;

        for cand in candidates {
            match self.process_one(&cand).await {
                Ok(item) => processed.push(item),
                Err(e) => {
                    warn!(
                        bookmark_id = %cand.bookmark_id,
                        error = %e,
                        "bookmark summarize failed; skipping record"
                    );
                    self.log_history(
                        "bookmark_summary_error",
                        Some(&cand.bookmark_id),
                        json!({ "error": e.to_string() }),
                    )
                    .await;
                    skipped += 1;
                }
            }
        }

        let processed_count = processed.len() as u32;
        let message = format!(
            "summarized {processed_count} bookmark(s); skipped {skipped}"
        );
        Ok(SummarizeOutput {
            processed: processed_count,
            skipped,
            items: processed,
            message,
        })
    }

    async fn process_one(
        &self,
        cand: &Candidate,
    ) -> Result<ProcessedBookmark, BookmarkProcessorError> {
        let (content, source) = self.gather_content(cand).await?;
        if content.trim().is_empty() {
            return Err(BookmarkProcessorError::Parse(format!(
                "no usable content for bookmark {} (notes empty and xapimcp unavailable / failed)",
                cand.bookmark_id
            )));
        }

        let llm = self.call_ollama(&content).await?;
        let mut merged_urls: Vec<String> = cand
            .existing_urls
            .iter()
            .map(|u| normalize_url(u))
            .collect();
        for u in &llm.extracted_urls {
            let n = normalize_url(u);
            if !merged_urls.iter().any(|x| x == &n) {
                merged_urls.push(n);
            }
        }

        self.update_record(&cand.bookmark_id, &llm.summary, &merged_urls)
            .await?;
        self.log_history(
            "bookmark_summary_ok",
            Some(&cand.bookmark_id),
            json!({
                "summary": llm.summary,
                "extracted_urls": merged_urls,
                "content_source": source,
                "model": self.inner.ollama_model,
            }),
        )
        .await;

        Ok(ProcessedBookmark {
            bookmark_id: cand.bookmark_id.clone(),
            summary: llm.summary,
            extracted_urls: merged_urls,
            source,
        })
    }

    async fn fetch_candidates(
        &self,
        limit: u32,
    ) -> Result<Vec<Candidate>, BookmarkProcessorError> {
        let q = format!(
            "SELECT bookmark_id, urls FROM {} WHERE summary IS NONE OR summary = '' OR summary = NONE LIMIT $lim;",
            self.inner.table
        );
        let mut params = serde_json::Map::new();
        params.insert("lim".to_string(), json!(limit));
        let mut args = serde_json::Map::new();
        args.insert("query".to_string(), json!(q));
        args.insert("parameters".to_string(), Value::Object(params));

        let text = self.surreal_call_text("query", args).await?;
        debug!(
            raw_len = text.len(),
            "fetch_candidates: raw SurrealMCP response"
        );

        let ids = parse_string_field(&text, "bookmark_id");
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut out = Vec::with_capacity(ids.len());
        for bid in ids {
            let urls = self.fetch_urls(&bid).await.unwrap_or_default();
            let notes = self.fetch_notes(&bid).await.unwrap_or_default();
            out.push(Candidate {
                bookmark_id: bid,
                notes,
                existing_urls: urls,
            });
        }
        Ok(out)
    }

    async fn fetch_notes(&self, bookmark_id: &str) -> Result<String, BookmarkProcessorError> {
        let q = format!(
            "SELECT notes FROM {} WHERE bookmark_id = $bid LIMIT 1;",
            self.inner.table
        );
        let mut params = serde_json::Map::new();
        params.insert("bid".to_string(), json!(bookmark_id));
        let mut args = serde_json::Map::new();
        args.insert("query".to_string(), json!(q));
        args.insert("parameters".to_string(), Value::Object(params));

        let text = self.surreal_call_text("query", args).await?;
        Ok(parse_string_field(&text, "notes")
            .into_iter()
            .next()
            .unwrap_or_default())
    }

    async fn fetch_urls(&self, bookmark_id: &str) -> Result<Vec<String>, BookmarkProcessorError> {
        let q = format!(
            "SELECT urls FROM {} WHERE bookmark_id = $bid LIMIT 1;",
            self.inner.table
        );
        let mut params = serde_json::Map::new();
        params.insert("bid".to_string(), json!(bookmark_id));
        let mut args = serde_json::Map::new();
        args.insert("query".to_string(), json!(q));
        args.insert("parameters".to_string(), Value::Object(params));

        let text = self.surreal_call_text("query", args).await?;
        Ok(parse_array_strings_field(&text, "urls"))
    }

    async fn gather_content(
        &self,
        cand: &Candidate,
    ) -> Result<(String, String), BookmarkProcessorError> {
        let notes = cand.notes.trim();
        if notes.len() >= 30 {
            return Ok((notes.to_string(), "notes".to_string()));
        }

        if let Some(x) = &self.inner.x_sink {
            match self
                .call_xapi_get_tweet(x, &cand.bookmark_id)
                .await
            {
                Ok(tweet_text) if !tweet_text.trim().is_empty() => {
                    let combined = if notes.is_empty() {
                        tweet_text
                    } else {
                        format!("{notes}\n\n---\nTweet:\n{tweet_text}")
                    };
                    return Ok((combined, "xapimcp".to_string()));
                }
                Ok(_) => warn!(bookmark_id = %cand.bookmark_id, "xapimcp returned empty tweet text"),
                Err(e) => warn!(bookmark_id = %cand.bookmark_id, %e, "xapimcp get_tweet failed; using notes only"),
            }
        }

        if !notes.is_empty() {
            return Ok((notes.to_string(), "notes_short".to_string()));
        }

        Err(BookmarkProcessorError::Parse(format!(
            "bookmark {} has no notes and no xapimcp content",
            cand.bookmark_id
        )))
    }

    async fn call_xapi_get_tweet(
        &self,
        x: &ServerSink,
        bookmark_id: &str,
    ) -> Result<String, BookmarkProcessorError> {
        let mut args = serde_json::Map::new();
        args.insert("tweet_id".to_string(), json!(bookmark_id));

        let req = CallToolRequestParams::new(self.inner.x_get_tweet_tool.clone())
            .with_arguments(args);
        let res = x.call_tool(req).await.map_err(|e| {
            BookmarkProcessorError::XapiTransport(
                self.inner.x_get_tweet_tool.clone(),
                e.to_string(),
            )
        })?;
        if res.is_error == Some(true) {
            return Err(BookmarkProcessorError::XapiTransport(
                self.inner.x_get_tweet_tool.clone(),
                flatten(&res),
            ));
        }
        let text = flatten(&res);
        Ok(extract_tweet_text(&text))
    }

    async fn update_record(
        &self,
        bookmark_id: &str,
        summary: &str,
        urls: &[String],
    ) -> Result<(), BookmarkProcessorError> {
        let q = format!(
            "UPDATE {} SET summary = $summary, extracted_urls = $urls, updated_at = time::now() WHERE bookmark_id = $bid;",
            self.inner.table
        );
        let mut params = serde_json::Map::new();
        params.insert("summary".to_string(), json!(summary));
        params.insert("urls".to_string(), json!(urls));
        params.insert("bid".to_string(), json!(bookmark_id));
        let mut args = serde_json::Map::new();
        args.insert("query".to_string(), json!(q));
        args.insert("parameters".to_string(), Value::Object(params));

        self.surreal_call_text("query", args).await?;
        Ok(())
    }

    async fn surreal_call_text(
        &self,
        tool: &'static str,
        args: serde_json::Map<String, Value>,
    ) -> Result<String, BookmarkProcessorError> {
        let req = CallToolRequestParams::new(tool).with_arguments(args);
        let res = self
            .inner
            .surreal_sink
            .call_tool(req)
            .await
            .map_err(|e| BookmarkProcessorError::SurrealTransport(tool, e.to_string()))?;
        if res.is_error == Some(true) {
            return Err(BookmarkProcessorError::SurrealToolError(
                tool,
                flatten(&res),
            ));
        }
        Ok(flatten(&res))
    }

    async fn log_history(&self, phase: &str, bookmark_id: Option<&str>, extra: Value) {
        let mut row = serde_json::Map::new();
        row.insert("ts".into(), json!(chrono::Utc::now().to_rfc3339()));
        row.insert("phase".into(), json!(phase));
        row.insert("source".into(), json!("salt-master-agent::bookmark_processor"));
        if let Some(b) = bookmark_id {
            row.insert("bookmark_id".into(), json!(b));
        }
        row.insert("extra".into(), extra);

        let mut args = serde_json::Map::new();
        args.insert("target".into(), json!(self.inner.history_table));
        args.insert("data".into(), Value::Object(row));

        let req = CallToolRequestParams::new("create").with_arguments(args);
        if let Err(e) = self.inner.surreal_sink.call_tool(req).await {
            warn!(phase, %e, "bookmark_processor: history log create failed (non-fatal)");
        }
    }

    async fn call_ollama(&self, content: &str) -> Result<LlmOutput, BookmarkProcessorError> {
        let url = format!("{}/api/chat", self.inner.ollama_base_url);
        let body = json!({
            "model": self.inner.ollama_model,
            "stream": false,
            "format": "json",
            "options": { "temperature": 0.0 },
            "messages": [
                { "role": "system", "content": SUMMARIZER_SYSTEM_PROMPT },
                { "role": "user", "content": content },
            ],
        });

        let resp = self
            .inner
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| BookmarkProcessorError::Ollama(e.to_string()))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let t = resp.text().await.unwrap_or_default();
            return Err(BookmarkProcessorError::Ollama(format!("HTTP {s}: {t}")));
        }

        let v: Value = resp
            .json()
            .await
            .map_err(|e| BookmarkProcessorError::Ollama(e.to_string()))?;
        let raw = v
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                BookmarkProcessorError::Ollama(format!(
                    "Ollama response missing message.content: {v}"
                ))
            })?
            .trim()
            .to_string();

        parse_llm_json(&raw)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct LlmOutput {
    summary: String,
    #[serde(default)]
    extracted_urls: Vec<String>,
}

fn parse_llm_json(raw: &str) -> Result<LlmOutput, BookmarkProcessorError> {
    let cleaned = strip_code_fence(raw);
    serde_json::from_str::<LlmOutput>(&cleaned)
        .map_err(|e| BookmarkProcessorError::OllamaJson(e, raw.to_string()))
}

fn strip_code_fence(s: &str) -> String {
    let t = s.trim();
    if let Some(rest) = t.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            return rest[..end].trim().to_string();
        }
    }
    t.to_string()
}

#[derive(Debug, Clone)]
struct Candidate {
    bookmark_id: String,
    notes: String,
    existing_urls: Vec<String>,
}

fn flatten(r: &CallToolResult) -> String {
    r.content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_url(u: &str) -> String {
    let t = u.trim().trim_end_matches(['.', ',', ')']);
    if t.starts_with("http://") || t.starts_with("https://") {
        t.to_string()
    } else if t.contains('.') && !t.contains(' ') {
        format!("https://{t}")
    } else {
        t.to_string()
    }
}

/// xapimcp's `get_tweet` returns a JSON-stringified Twitter API response. Try to extract the
/// `text` and `note_tweet.text` fields; fall back to the entire blob.
fn extract_tweet_text(blob: &str) -> String {
    let trimmed = blob.trim();
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        let mut parts: Vec<String> = Vec::new();
        if let Some(t) = v.pointer("/data/text").and_then(|x| x.as_str()) {
            parts.push(t.to_string());
        } else if let Some(t) = v.pointer("/text").and_then(|x| x.as_str()) {
            parts.push(t.to_string());
        }
        if let Some(t) = v
            .pointer("/data/note_tweet/text")
            .and_then(|x| x.as_str())
        {
            parts.push(t.to_string());
        } else if let Some(t) = v.pointer("/note_tweet/text").and_then(|x| x.as_str()) {
            parts.push(t.to_string());
        }
        if !parts.is_empty() {
            return parts.join("\n");
        }
    }
    trimmed.to_string()
}

// ---------------------------------------------------------------------------
// SurrealDB Debug-format parsers (because SurrealMCP serializes results as
// `format!("{value:?}")` of a `surrealdb::Value`).
// ---------------------------------------------------------------------------

/// Extracts the unescaped string contents of every `"<field>": Strand("...")` /
/// `"<field>": String("...")` pair found in a SurrealMCP debug-formatted response.
pub fn parse_string_field(text: &str, field: &str) -> Vec<String> {
    let key = format!("\"{field}\"");
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut search = 0;
    while let Some(idx) = text[search..].find(&key) {
        let abs = search + idx + key.len();
        let rest = &text[abs..];
        let mut matched = false;
        for prefix in [": Strand(\"", ":Strand(\"", ": String(\"", ":String(\""] {
            if let Some(p_idx) = rest.find(prefix)
                && p_idx <= 4
            {
                let val_start = abs + p_idx + prefix.len() - 1;
                if let Some((val, end)) = parse_debug_string(bytes, val_start) {
                    out.push(val);
                    search = end;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            search = abs;
        }
    }
    out
}

/// Extracts the contents of the *first* `"<field>": Array([...])` value as a list of strings,
/// transparently unescaping `Strand("…")` / `String("…")` items inside.
///
/// Tolerates the SurrealMCP `IndexedResults` Debug shape, which wraps SurrealDB arrays as
/// `Array(Array([...]))` (newer surrealdb crates) **or** as `Array([...])` (older).
pub fn parse_array_strings_field(text: &str, field: &str) -> Vec<String> {
    let key = format!("\"{field}\"");
    let bytes = text.as_bytes();
    let mut search = 0;
    while let Some(idx) = text[search..].find(&key) {
        let abs = search + idx + key.len();
        let rest = &text.as_bytes()[abs..];
        let mut cursor = 0usize;
        while cursor < rest.len() && matches!(rest[cursor], b' ' | b':') {
            cursor += 1;
        }
        if cursor + 6 <= rest.len() && &rest[cursor..cursor + 6] == b"Array(" {
            let mut p = cursor + 6;
            while p + 6 <= rest.len() && &rest[p..p + 6] == b"Array(" {
                p += 6;
            }
            if p < rest.len() && rest[p] == b'[' {
                let array_start = abs + p + 1;
                let mut depth = 1usize;
                let mut i = array_start;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'[' => depth += 1,
                        b']' => depth -= 1,
                        b'"' => {
                            if let Some((_, end)) = parse_debug_string(bytes, i) {
                                i = end;
                                continue;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                let array_end = i.saturating_sub(1);
                let inner = &text[array_start..array_end];
                let mut out = Vec::new();
                let inner_bytes = inner.as_bytes();
                let mut j = 0;
                while j < inner_bytes.len() {
                    if inner_bytes[j] == b'"'
                        && let Some((s, end_rel)) = parse_debug_string(inner_bytes, j)
                    {
                        out.push(s);
                        j = end_rel;
                        continue;
                    }
                    j += 1;
                }
                return out;
            }
        }
        search = abs;
    }
    Vec::new()
}

/// Reads a Rust-Debug-formatted string literal whose opening `"` lives at `start`.
/// Returns `(unescaped_value, byte_index_just_past_closing_quote)`.
fn parse_debug_string(bytes: &[u8], start: usize) -> Option<(String, usize)> {
    if bytes.get(start) != Some(&b'"') {
        return None;
    }
    let mut i = start + 1;
    let mut out = Vec::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            let n = bytes[i + 1];
            match n {
                b'"' => out.push(b'"'),
                b'\\' => out.push(b'\\'),
                b'n' => out.push(b'\n'),
                b't' => out.push(b'\t'),
                b'r' => out.push(b'\r'),
                b'0' => out.push(0),
                _ => {
                    out.push(b);
                    out.push(n);
                }
            }
            i += 2;
        } else if b == b'"' {
            return Some((String::from_utf8_lossy(&out).into_owned(), i + 1));
        } else {
            out.push(b);
            i += 1;
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Schema bootstrap: idempotent `DEFINE FIELD` + `DEFINE FUNCTION` statements.
// ---------------------------------------------------------------------------

/// SurrealQL applied at startup (once per process) when
/// `apply_bookmark_schema_on_startup` is true. All statements are `OVERWRITE` so reruns are safe.
pub fn schema_sql(table: &str) -> String {
    format!(
        "DEFINE FIELD OVERWRITE summary ON TABLE {table} TYPE option<string>;\n\
         DEFINE FIELD OVERWRITE extracted_urls ON TABLE {table} TYPE option<array<string>>;\n\
         DEFINE FIELD OVERWRITE updated_at ON TABLE {table} TYPE option<datetime>;\n\
         DEFINE FUNCTION OVERWRITE fn::mark_as_processed(\
            $table: string, $bookmark_id: string, $summary: string, $extracted_urls: array<string>\
         ) {{\n\
            UPDATE type::table($table) \
                SET summary = $summary, \
                    extracted_urls = $extracted_urls, \
                    updated_at = time::now() \
                WHERE bookmark_id = $bookmark_id;\n\
            RETURN $bookmark_id;\n\
         }};",
    )
}

/// Apply the schema additions through SurrealMCP `query`. Logs and swallows errors so
/// startup never fails because the schema function syntax shifted between SurrealDB versions.
pub async fn ensure_schema(cfg: &AppConfig, surreal_sink: &ServerSink) {
    let sql = schema_sql(&cfg.bookmark_annotations_table);
    let mut args = serde_json::Map::new();
    args.insert("query".into(), json!(sql));
    let req = CallToolRequestParams::new("query").with_arguments(args);
    match surreal_sink.call_tool(req).await {
        Ok(r) if r.is_error != Some(true) => {
            info!(table = %cfg.bookmark_annotations_table, "bookmark schema applied");
        }
        Ok(r) => warn!(?r, "bookmark schema apply returned tool error (continuing)"),
        Err(e) => warn!(%e, "bookmark schema apply transport error (continuing)"),
    }
}

/// Apply `use_namespace` / `use_database` if configured. Required because SurrealMCP starts
/// with no NS/DB selected when the server is launched in standalone mode.
pub async fn select_namespace_database(cfg: &AppConfig, surreal_sink: &ServerSink) {
    if let Some(ns) = cfg.surreal_namespace.as_deref() {
        let mut args = serde_json::Map::new();
        args.insert("namespace".into(), json!(ns));
        let req = CallToolRequestParams::new("use_namespace").with_arguments(args);
        if let Err(e) = surreal_sink.call_tool(req).await {
            warn!(%e, namespace = ns, "use_namespace failed (continuing)");
        }
    }
    if let Some(db) = cfg.surreal_database.as_deref() {
        let mut args = serde_json::Map::new();
        args.insert("database".into(), json!(db));
        let req = CallToolRequestParams::new("use_database").with_arguments(args);
        if let Err(e) = surreal_sink.call_tool(req).await {
            warn!(%e, database = db, "use_database failed (continuing)");
        }
    }
}

// ---------------------------------------------------------------------------
// Rig `Tool` impl — exposes `summarize_unsummarized_bookmarks` to the LLM.
// ---------------------------------------------------------------------------

impl Tool for BookmarkProcessor {
    const NAME: &'static str = TOOL_NAME;
    type Error = BookmarkProcessorError;
    type Args = SummarizeArgs;
    type Output = SummarizeOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: TOOL_NAME.to_string(),
            description: "Scan SurrealDB `bookmark_annotations` for rows with no `summary`, ask the local Ollama model to produce a strict JSON {summary, extracted_urls}, then UPDATE each row through SurrealMCP. Process up to `limit` records (default from config, capped at 500). Returns the per-record outcome.".to_string(),
            parameters: json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 500,
                        "description": "Maximum number of unsummarized bookmarks to process in this call."
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.process(args.limit).await
    }
}

// ---------------------------------------------------------------------------
// Tests (no external services required — parsers + prompts only).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_DEBUG: &str = "Ok(Array(Array([Object({\"id\": Thing { tb: \"bookmark_annotations\", id: String(\"byg5m9jepf7wss8flmu2\") }, \"bookmark_id\": Strand(\"1788009838951235802\"), \"notes\": Strand(\"This shows that the Book of Revelation is a big chiastic structure\"), \"urls\": Array([Strand(\"t.co/sNW4AbdUSL\")])})])))";

    #[test]
    fn parses_bookmark_id_and_notes() {
        let ids = parse_string_field(EXAMPLE_DEBUG, "bookmark_id");
        assert_eq!(ids, vec!["1788009838951235802".to_string()]);
        let notes = parse_string_field(EXAMPLE_DEBUG, "notes");
        assert_eq!(
            notes,
            vec!["This shows that the Book of Revelation is a big chiastic structure".to_string()]
        );
    }

    #[test]
    fn parses_urls_array() {
        let urls = parse_array_strings_field(EXAMPLE_DEBUG, "urls");
        assert_eq!(urls, vec!["t.co/sNW4AbdUSL".to_string()]);
    }

    #[test]
    fn parses_escaped_quotes_in_notes() {
        let blob = "Ok([Object({\"notes\": Strand(\"He said \\\"hello\\\" to me\")})])";
        let notes = parse_string_field(blob, "notes");
        assert_eq!(notes, vec!["He said \"hello\" to me".to_string()]);
    }

    #[test]
    fn handles_empty_array() {
        let blob = "Ok([Object({\"urls\": Array([])})])";
        let urls = parse_array_strings_field(blob, "urls");
        assert!(urls.is_empty());
    }

    #[test]
    fn llm_json_parses_clean_payload() {
        let raw = "{\"summary\": \"A short summary.\", \"extracted_urls\": [\"https://github.com/foo/bar\"]}";
        let o = parse_llm_json(raw).unwrap();
        assert_eq!(o.summary, "A short summary.");
        assert_eq!(o.extracted_urls, vec!["https://github.com/foo/bar".to_string()]);
    }

    #[test]
    fn llm_json_strips_code_fence() {
        let raw = "```json\n{\"summary\": \"x\", \"extracted_urls\": []}\n```";
        let o = parse_llm_json(raw).unwrap();
        assert_eq!(o.summary, "x");
    }

    #[test]
    fn normalize_url_adds_scheme_when_missing() {
        assert_eq!(
            normalize_url("github.com/foo/bar"),
            "https://github.com/foo/bar".to_string()
        );
        assert_eq!(
            normalize_url("https://github.com/foo/bar"),
            "https://github.com/foo/bar".to_string()
        );
        assert_eq!(
            normalize_url("https://github.com/foo/bar."),
            "https://github.com/foo/bar".to_string()
        );
    }

    #[test]
    fn schema_sql_mentions_required_fields_and_function() {
        let sql = schema_sql("bookmark_annotations");
        assert!(sql.contains("summary ON TABLE bookmark_annotations"));
        assert!(sql.contains("extracted_urls ON TABLE bookmark_annotations"));
        assert!(sql.contains("DEFINE FUNCTION OVERWRITE fn::mark_as_processed"));
    }

    #[test]
    fn extract_tweet_text_pulls_from_data_text() {
        let blob = r#"{"data":{"id":"1","text":"hello world","note_tweet":{"text":"hello world long"}}}"#;
        let t = extract_tweet_text(blob);
        assert!(t.contains("hello world"));
        assert!(t.contains("hello world long"));
    }

    #[test]
    fn example_bookmark_round_trip_via_parsers() {
        let ids = parse_string_field(EXAMPLE_DEBUG, "bookmark_id");
        let notes = parse_string_field(EXAMPLE_DEBUG, "notes");
        let urls = parse_array_strings_field(EXAMPLE_DEBUG, "urls");
        assert_eq!(ids[0], "1788009838951235802");
        assert!(notes[0].contains("Book of Revelation"));
        assert_eq!(urls, vec!["t.co/sNW4AbdUSL".to_string()]);
    }

    /// Exact shape returned by the live SurrealMCP `query` tool (captured 2026-05-03 against
    /// `bookmarks/v1`): `IndexedResults` wrapper, `Array(Array([…]))` and `Object(Object({…}))`
    /// double-wrapping, `String("…")` instead of `Strand("…")`, and `RecordId(RecordId { … })`.
    const LIVE_DEBUG: &str = "IndexedResults { results: {0: (DbResultStats { execution_time: Some(380.633µs), query_type: Some(Other) }, Ok(Array(Array([Object(Object({\"bookmark_id\": String(\"1788009838951235802\"), \"created_at\": String(\"2026-04-07T23:17:34.060Z\"), \"id\": RecordId(RecordId { table: Table(\"bookmark_annotations\"), key: String(\"byg5m9jepf7wss8flmu2\") }), \"notes\": String(\"This shows that the Book of Revelation is a big chiastic structure\"), \"updated_at\": String(\"2026-04-08T01:09:25.945Z\"), \"urls\": Array(Array([String(\"t.co/sNW4AbdUSL\")]))}))]))) }, live_queries: {} }";

    #[test]
    fn live_indexedresults_wrapped_response_is_parsed() {
        let ids = parse_string_field(LIVE_DEBUG, "bookmark_id");
        assert_eq!(ids, vec!["1788009838951235802".to_string()]);

        let notes = parse_string_field(LIVE_DEBUG, "notes");
        assert_eq!(
            notes,
            vec!["This shows that the Book of Revelation is a big chiastic structure".to_string()]
        );

        let urls = parse_array_strings_field(LIVE_DEBUG, "urls");
        assert_eq!(urls, vec!["t.co/sNW4AbdUSL".to_string()]);
    }

    #[test]
    fn live_double_wrapped_array_with_multiple_items() {
        let blob = "Ok(Array(Array([Object(Object({\"urls\": Array(Array([String(\"https://github.com/a/b\"), String(\"https://github.com/c/d\")]))}))])))";
        let urls = parse_array_strings_field(blob, "urls");
        assert_eq!(
            urls,
            vec![
                "https://github.com/a/b".to_string(),
                "https://github.com/c/d".to_string()
            ]
        );
    }
}
