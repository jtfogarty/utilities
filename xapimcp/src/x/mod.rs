use crate::config::ServerConfig;
use anyhow::Result;
use reqwest::Client;
use rmcp::model::ErrorData as McpError;
use std::borrow::Cow;
use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub fn http_client() -> &'static Client {
    &HTTP_CLIENT
}

pub async fn get_my_bookmarks(
    config: &ServerConfig,
    pagination_token: Option<String>,
) -> Result<serde_json::Value, McpError> {
    let mut url = format!(
        "https://api.x.com/2/users/{}/bookmarks?max_results=100&tweet.fields=created_at,author_id,conversation_id,text",
        config.x_user_id
    );
    if let Some(token) = pagination_token {
        url.push_str(&format!("&pagination_token={}", token));
    }

    let resp = http_client()
        .get(&url)
        .bearer_auth(&config.x_bearer_token)
        .send()
        .await
        .map_err(|e| McpError {
            code: (-32603).into(),
            message: Cow::from(format!("X API bookmarks request failed: {}", e)),
            data: None,
        })?;

    let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
        code: (-32603).into(),
        message: Cow::from(format!("Failed to parse X API response: {}", e)),
        data: None,
    })?;

    Ok(body)
}

pub async fn delete_bookmark(config: &ServerConfig, tweet_id: String) -> Result<serde_json::Value, McpError> {
    let url = format!("https://api.x.com/2/users/{}/bookmarks/{}", config.x_user_id, tweet_id);

    let resp = http_client()
        .delete(&url)
        .bearer_auth(&config.x_bearer_token)
        .send()
        .await
        .map_err(|e| McpError {
            code: (-32603).into(),
            message: Cow::from(format!("X API delete bookmark failed: {}", e)),
            data: None,
        })?;

    let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
        code: (-32603).into(),
        message: Cow::from(format!("Failed to parse delete response: {}", e)),
        data: None,
    })?;

    Ok(body)
}

pub async fn get_replies_to_tweet(
    config: &ServerConfig,
    tweet_id: String,
) -> Result<serde_json::Value, McpError> {
    let url = format!(
        "https://api.x.com/2/tweets/search/recent?query=conversation_id:{}&max_results=100&tweet.fields=created_at,author_id,text,conversation_id",
        tweet_id
    );

    let resp = http_client()
        .get(&url)
        .bearer_auth(&config.x_bearer_token)
        .send()
        .await
        .map_err(|e| McpError {
            code: (-32603).into(),
            message: Cow::from(format!("X API replies search failed: {}", e)),
            data: None,
        })?;

    let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
        code: (-32603).into(),
        message: Cow::from(format!("Failed to parse replies response: {}", e)),
        data: None,
    })?;

    Ok(body)
}