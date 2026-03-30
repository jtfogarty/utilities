use crate::{
    config::ServerConfig,
    tools::{DeleteBookmarkRequest, GetMyBookmarksRequest, GetRepliesRequest},
    x,
};
use anyhow::Result;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router, ServerHandler, ServiceExt,
};
use tokio::io::{stdin, stdout};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting xapimcp (stdio transport) — X.com API ready");

    let server = XServer::new(config);
    let service = server.serve((stdin(), stdout())).await?;

    tracing::info!("xapimcp ready — LLM agents can now call get_my_bookmarks, delete_bookmark, get_replies_to_bookmark");
    service.waiting().await?;
    Ok(())
}

#[derive(Clone)]
pub struct XServer {
    config: ServerConfig,
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
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get your personal X.com bookmarks (returns full tweet data)")]
    async fn get_my_bookmarks(
        &self,
        params: Parameters<GetMyBookmarksRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let data = x::get_my_bookmarks(&self.config, params.0.pagination_token).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmarks retrieved".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Delete one of your X.com bookmarks by tweet ID")]
    async fn delete_bookmark(
        &self,
        params: Parameters<DeleteBookmarkRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let data = x::delete_bookmark(&self.config, params.0.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmark deleted".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get replies/comments to one of your bookmarked tweets")]
    async fn get_replies_to_bookmark(
        &self,
        params: Parameters<GetRepliesRequest>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let data = x::get_replies_to_tweet(&self.config, params.0.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Replies retrieved".to_string());
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}
