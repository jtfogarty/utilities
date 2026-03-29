use crate::{config::ServerConfig, x};
use rmcp::{
    handler::server::tool::ToolRouter,
    model::{CallToolResult, ToolContent},
    schemars, tool, tool_router, Parameters,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct XTools;

#[derive(Debug, Deserialize, schemars::JsonSchema, Serialize)]
pub struct GetMyBookmarksRequest {
    #[schemars(description = "Optional pagination token for next page")]
    pub pagination_token: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema, Serialize)]
pub struct DeleteBookmarkRequest {
    #[schemars(description = "Tweet ID of the bookmark to delete")]
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema, Serialize)]
pub struct GetRepliesRequest {
    #[schemars(description = "Tweet ID of the bookmarked tweet")]
    pub tweet_id: String,
}

#[tool_router]
impl XTools {
    pub fn new() -> ToolRouter<Self> {
        Self::tool_router()
    }

    #[tool(description = "Get your personal X.com bookmarks (returns full tweet data)")]
    async fn get_my_bookmarks(
        &self,
        Parameters(request): Parameters<GetMyBookmarksRequest>,
        config: &ServerConfig,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let data = x::get_my_bookmarks(config, request.pagination_token).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmarks retrieved".to_string());
        Ok(CallToolResult::success(vec![ToolContent::text(text)]))
    }

    #[tool(description = "Delete one of your X.com bookmarks by tweet ID")]
    async fn delete_bookmark(
        &self,
        Parameters(request): Parameters<DeleteBookmarkRequest>,
        config: &ServerConfig,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let data = x::delete_bookmark(config, request.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Bookmark deleted".to_string());
        Ok(CallToolResult::success(vec![ToolContent::text(text)]))
    }

    #[tool(description = "Get replies/comments to one of your bookmarked tweets")]
    async fn get_replies_to_bookmark(
        &self,
        Parameters(request): Parameters<GetRepliesRequest>,
        config: &ServerConfig,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let data = x::get_replies_to_tweet(config, request.tweet_id).await?;
        let text = serde_json::to_string_pretty(&data)
            .unwrap_or_else(|_| "Replies retrieved".to_string());
        Ok(CallToolResult::success(vec![ToolContent::text(text)]))
    }
}