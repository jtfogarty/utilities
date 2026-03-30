//! `salt-master-agent` — Ollama + Rig + rmcp. Stateless process; durable memory only via MCP (SurrealMCP).

mod agent;
mod config;
mod error;
mod mcp;

use std::sync::Arc;

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
use crate::config::AppConfig;
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
    },
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
    let agent = agent::build_agent(&cfg, &mcp)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;

    match cli.command {
        Commands::Repl => {
            agent::run_repl(agent).await?;
        }
        Commands::Serve { bind } => {
            if let Some(b) = bind {
                cfg.http_bind = b;
            }
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
    }

    Ok(())
}
