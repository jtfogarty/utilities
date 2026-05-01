use anyhow::{Result, anyhow};
use axum::{
    Router,
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
};
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::ServerConfig;
use crate::tools::SaltService;

// Bearer-token authentication middleware.
///
/// If `expected_token` is `None (no AUTH_TOKEN configured) all requests pass through.
/// The `/health` endpoint is always exempt so Load-balancers can probe without credentials.
async fn bearer_auth(
    expected_token: Option<String>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no token is configured, skip auth entirely.
    let expected = match &expected_token {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(next.run(request).await),
    };

    // Always allow the health endpoint through unauthenticated.
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Extract and validate the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    match auth_header {
        Some(value) if value.starts_with("Bearer ") => {
            let token = &value["Bearer ".len()..];
            if token == expected.as_str() {
                Ok(next.run(request).await)
            } else {
                tracing::warn!("Rejected request: invalid bearer token");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            tracing::warn!("Rejected request: missing or invalid Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn start_server(config: ServerConfig) -> Result<()> {
    let bind_address = config.bind_address.clone();
    let auth_token = config.auth_token.clone();

    if auth_token.as_ref().is_some_and(|t| !t.is_empty()) {
        tracing::info!("Bearer-token Authentication is ENABLED");
    } else {
        tracing::info!("Bearer-token Authentication is DISABLED - all requests will be accepted");
    }

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

    let router = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(middleware::from_fn(move |req, next| {
            bearer_auth(auth_token.clone(), req, next)
    }));

    let listener = TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("Failed to bind to {bind_address}: {e}"))?;

    tracing::info!("saltapimcp ready — MCP endpoint: http://{}/mcp", bind_address);

    axum::serve(listener, router).await?;

    Ok(())
}
