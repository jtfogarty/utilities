use crate::{
    config::ServerConfig,
    tools::{DeleteBookmarkRequest, GetMyBookmarksRequest, GetRepliesRequest, GetTweetRequest},
    x,
};
use anyhow::{Result, anyhow};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router, ServerHandler, ServiceExt,
};
use std::sync::Arc;
use tokio::io::{stdin, stdout};
use tokio::sync::Mutex;

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("xapimcp starting (X auth is lazy: failures surface on tool calls, not process exit)");
    match config.bind_address.clone() {
        Some(addr) => start_http_server(config, &addr).await,
        None => start_stdio_server(config).await,
    }
}

async fn start_stdio_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting xapimcp (stdio transport)");
    let server = XServer::new(Arc::new(Mutex::new(config)));
    let service = server.serve((stdin(), stdout())).await?;
    tracing::info!("xapimcp ready (stdio)");
    service.waiting().await?;
    Ok(())
}

async fn start_http_server(config: ServerConfig, bind_address: &str) -> Result<()> {
    use axum::{Router, routing::get};
    use axum::http::StatusCode;
    use rmcp::transport::{
        StreamableHttpServerConfig,
        streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
    };
    use tower_http::trace::TraceLayer;

    tracing::info!(bind_address,"Starting xapimcp (Streamable HTTP transport)");

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("Failed to bind to {bind_address}: {e}"))?;

    let session_manager = Arc::new(LocalSessionManager::default());

    let shared_config = Arc::new(Mutex::new(config));
    let mcp_service = StreamableHttpService::new(
        {
            let shared_config = shared_config.clone();
            move || Ok(XServer::new(shared_config.clone()))
        },
        session_manager,
        StreamableHttpServerConfig {
            stateful_mode: true,
            sse_keep_alive: None,
        },
    );

    let router = Router::new()
        .nest_service("/mcp", mcp_service)
        .route("/health", get(|| async { StatusCode::OK }))
        .layer(TraceLayer::new_for_http());

    tracing::info!("xapimcp ready (Streamable HTTP) — listening on {bind_address}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutdown signal received");
}

#[derive(Clone)]
pub struct XServer {
    config: Arc<Mutex<ServerConfig>>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for XServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "X.com MCP server. Use the three bookmark tools to manage your personal X bookmarks and their replies."
                    .to_string(),
            ),
        }
    }
}

#[tool_router(router = tool_router)]
impl XServer {
    pub fn new(config: Arc<Mutex<ServerConfig>>) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    async fn ensure_x_ready(&self) -> Result<(), rmcp::ErrorData> {
        let mut cfg = self.config.lock().await;
        cfg.ensure_x_user_id()
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get your personal X.com bookmarks (returns full tweet data)")]
    async fn get_my_bookmarks(
        &self,
        params: Parameters<GetMyBookmarksRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.ensure_x_ready().await?;
        let cfg = self.config.lock().await;
        let data = x::get_my_bookmarks(&cfg, params.0.pagination_token).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmarks retrieved".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Delete one of your X.com bookmarks by tweet ID")]
    async fn delete_bookmark(
        &self,
        params: Parameters<DeleteBookmarkRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.ensure_x_ready().await?;
        let cfg = self.config.lock().await;
        let data = x::delete_bookmark(&cfg, params.0.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmark deleted".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Look up a single tweet by ID (returns note_tweet and article fields)")]
    async fn get_tweet(
        &self,
        params: Parameters<GetTweetRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.ensure_x_ready().await?;
        let cfg = self.config.lock().await;
        let data = x::get_tweet(&cfg, params.0.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Tweet retrieved".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get replies/comments to one of your bookmarked tweets")]
    async fn get_replies_to_bookmark(
        &self,
        params: Parameters<GetRepliesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.ensure_x_ready().await?;
        let cfg = self.config.lock().await;
        let data = x::get_replies_to_tweet(&cfg, params.0.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Replies retrieved".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}
