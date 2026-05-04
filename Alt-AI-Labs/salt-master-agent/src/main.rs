//! `salt-master-agent` — Ollama + Rig + rmcp. Stateless process; durable memory only via MCP (SurrealMCP).

mod agent;
mod agents;
mod config;
mod error;
mod mcp;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::agent::SurrealAuditHook;
use crate::agents::bookmark_processor::{self, BookmarkProcessor};
use crate::config::AppConfig;
use crate::mcp::McpRuntime;
use rig::completion::Prompt;
use rig::providers::ollama;

#[derive(Parser, Debug)]
#[command(name = "salt-master-agent", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive line-oriented loop (stdin/stdout).
    Repl,
    /// HTTP API with `GET /health` and `POST /v1/prompt`.
    Serve {
        /// Overrides `http_bind` for this process (also `SMA_HTTP_BIND`).
        #[arg(long, env = "SMA_HTTP_BIND")]
        bind: Option<String>,
        /// Force-enable the bookmark reactor loop for this process (overrides config).
        #[arg(long)]
        process_bookmarks: bool,
    },
    /// One-shot: process unsummarized bookmarks and exit (no agent / HTTP).
    ProcessBookmarks {
        /// How many records to process this run.
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Apply the bookmark schema (`summary`, `extracted_urls`, `fn::mark_as_processed`) and exit.
    InitBookmarkSchema,
}

#[derive(Clone)]
struct HttpState {
    agent: Arc<rig::agent::Agent<ollama::CompletionModel, SurrealAuditHook>>,
}

#[derive(Deserialize)]
struct PromptRequest {
    prompt: String,
}

#[derive(Serialize)]
struct PromptResponse {
    reply: String,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "salt-master-agent",
    }))
}

async fn post_prompt(
    State(state): State<HttpState>,
    Json(body): Json<PromptRequest>,
) -> impl IntoResponse {
    if body.prompt.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "empty prompt").into_response();
    }
    match state.agent.prompt(&body.prompt).await {
        Ok(reply) => {
            if let Some(h) = &state.agent.hook {
                h.persist_reflection(&body.prompt, &reply).await;
            }
            (StatusCode::OK, Json(PromptResponse { reply })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("agent error: {e:#}"),
        )
            .into_response(),
    }
}

fn init_tracing(filter: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().json())
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut s) = signal(SignalKind::terminate()) {
            let _ = s.recv().await;
        }
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
    tracing::info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let mut cfg = AppConfig::load().map_err(anyhow::Error::from)?;
    init_tracing(&cfg.log_filter);

    let mcp = mcp::initialize_mcp_clients(&cfg)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(sink) = mcp.surreal_sink() {
        bookmark_processor::select_namespace_database(&cfg, &sink).await;
        if cfg.apply_bookmark_schema_on_startup {
            bookmark_processor::ensure_schema(&cfg, &sink).await;
        }
    }

    let bookmark_processor_opt = mcp
        .surreal_sink()
        .map(|s| BookmarkProcessor::new(&cfg, s, mcp.x_sink()));

    if let Some(bp) = &bookmark_processor_opt {
        if let Err(e) = mcp.tool_server_handle().add_tool(bp.clone()).await {
            tracing::warn!(%e, "failed to register summarize_unsummarized_bookmarks tool");
        } else {
            tracing::info!("registered tool: {}", bookmark_processor::TOOL_NAME);
        }
    } else {
        tracing::warn!(
            "SurrealMCP not connected; `summarize_unsummarized_bookmarks` tool not registered"
        );
    }

    match cli.command {
        Commands::Repl => {
            let agent = agent::build_agent(&cfg, &mcp)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            spawn_reactor_if_enabled(&cfg, bookmark_processor_opt.clone());
            agent::run_repl(agent).await?;
        }
        Commands::Serve {
            bind,
            process_bookmarks,
        } => {
            if let Some(b) = bind {
                cfg.http_bind = b;
            }
            if process_bookmarks {
                cfg.bookmark_reactor_enabled = true;
            }
            let agent = agent::build_agent(&cfg, &mcp)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            spawn_reactor_if_enabled(&cfg, bookmark_processor_opt.clone());
            let state = HttpState {
                agent: Arc::new(agent),
            };
            let app = Router::new()
                .route("/health", get(health))
                .route("/v1/prompt", post(post_prompt))
                .layer(TraceLayer::new_for_http())
                .with_state(state);

            let listener = tokio::net::TcpListener::bind(&cfg.http_bind)
                .await
                .map_err(|e| anyhow::anyhow!("bind {}: {e}", cfg.http_bind))?;
            tracing::info!(addr = %cfg.http_bind, "listening (POST /v1/prompt, GET /health)");

            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        Commands::ProcessBookmarks { limit } => {
            let bp = bookmark_processor_opt.ok_or_else(|| {
                anyhow::anyhow!("SurrealMCP not configured (set SMA_MCP__SURREAL_URL)")
            })?;
            let out = bp.process(Some(limit)).await?;
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        Commands::InitBookmarkSchema => {
            let sink = mcp.surreal_sink().ok_or_else(|| {
                anyhow::anyhow!("SurrealMCP not configured (set SMA_MCP__SURREAL_URL)")
            })?;
            bookmark_processor::ensure_schema(&cfg, &sink).await;
            println!("bookmark schema applied to table `{}`", cfg.bookmark_annotations_table);
        }
    }

    Ok(())
}

fn spawn_reactor_if_enabled(cfg: &AppConfig, bp: Option<BookmarkProcessor>) {
    if !cfg.bookmark_reactor_enabled {
        return;
    }
    let Some(bp) = bp else {
        tracing::warn!("bookmark reactor enabled but SurrealMCP not connected");
        return;
    };
    let interval = Duration::from_secs(cfg.bookmark_reactor_interval_seconds.max(15));
    let limit = cfg.bookmark_summarize_default_limit;
    tracing::info!(
        every_seconds = interval.as_secs(),
        limit, "starting bookmark summarizer reactor"
    );
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(interval);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            match bp.process(Some(limit)).await {
                Ok(out) => {
                    if out.processed > 0 || out.skipped > 0 {
                        tracing::info!(
                            processed = out.processed,
                            skipped = out.skipped,
                            "reactor tick"
                        );
                    } else {
                        tracing::debug!("reactor tick: no unsummarized bookmarks");
                    }
                }
                Err(e) => tracing::warn!(%e, "reactor tick failed"),
            }
        }
    });
}

// Silence unused-import warning when no caller uses `McpRuntime` outside this file.
#[allow(dead_code)]
fn _force_use(_: &McpRuntime) {}
