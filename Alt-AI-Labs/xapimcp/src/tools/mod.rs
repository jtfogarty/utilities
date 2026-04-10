use serde::{Deserialize, Serialize};

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
pub struct GetTweetRequest {
    #[schemars(description = "Tweet ID to look up")]
    pub tweet_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema, Serialize)]
pub struct GetRepliesRequest {
    #[schemars(description = "Tweet ID of the bookmarked tweet")]
    pub tweet_id: String,
}
