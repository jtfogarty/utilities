use anyhow::{Result, anyhow};
use axum::Router;
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::ServerConfig;
use crate::tools::SaltService;

pub async fn start_server(config: ServerConfig) -> Result<()> {
    let bind_address = config.bind_address.clone();

    tracing::info!(
        bind_address = %bind_address,
        salt_api_url = %config.salt_api_url,
        "Starting saltapimcp (HTTP/MCP transport)"
    );

    let session_manager = Arc::new(LocalSessionManager::default());

    let mcp_service = StreamableHttpService::new(
        move || Ok(SaltService::new(config.clone())),
        session_manager,
        StreamableHttpServerConfig {
            stateful_mode: true,
            sse_keep_alive: None,
        },
    );

    let router = Router::new().nest_service("/mcp", mcp_service);

    let listener = TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("Failed to bind to {bind_address}: {e}"))?;

    tracing::info!("saltapimcp ready — MCP endpoint: http://{}/mcp", bind_address);

    axum::serve(listener, router).await?;

    Ok(())
}