use crate::config::ServerConfig;
use anyhow::Context;
use anyhow::Result;
use reqwest::{Client, Url};
use rmcp::model::ErrorData as McpError;
use std::sync::LazyLock;
use tracing::info;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub fn http_client() -> &'static Client {
    &HTTP_CLIENT
}

pub async fn fetch_authenticated_user_id(config: &ServerConfig) -> anyhow::Result<String> {
    let url = "https://api.x.com/2/users/me?user.fields=id";
    let token = config
        .get_valid_x_access_token()
        .await
        .map_err(|e| anyhow::anyhow!(e.message.into_owned()))?;
    let resp = http_client()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .context("X API GET /2/users/me failed")?;

    let status = resp.status();
    let body_text = resp.text().await.context("read users/me body")?;
    if !status.is_success() {
        anyhow::bail!("GET /2/users/me {}: {}", status, body_text);
    }

    let v: serde_json::Value =
        serde_json::from_str(&body_text).context("parse users/me JSON")?;
    let id = v["data"]["id"]
        .as_str()
        .context("users/me response missing data.id")?;
    Ok(id.to_string())
}

pub async fn get_my_bookmarks(
    config: &ServerConfig,
    pagination_token: Option<String>,
) -> Result<serde_json::Value, McpError> {
    info!(
        x_user_id = config.user_id(),
        x_access_token = "present",
        "xapimcp auth context for get_my_bookmarks (token redacted)"
    );
    let token = config.get_valid_x_access_token().await?;
    let base = format!("https://api.x.com/2/users/{}/bookmarks", config.user_id());
    let mut url = Url::parse_with_params(
        &base,
        &[
            ("max_results", "100"),
            (
                "tweet.fields",
                "created_at,author_id,conversation_id,text,public_metrics,attachments,referenced_tweets,entities,lang",
            ),
            ("expansions", "author_id,attachments.media_keys"),
            ("user.fields", "id,name,username,verified,public_metrics"),
            (
                "media.fields",
                "media_key,type,url,preview_image_url,width,height,duration_ms,public_metrics",
            ),
        ],
    )
    .map_err(|e| McpError::internal_error(format!("Invalid bookmarks URL: {}", e), None))?;
    if let Some(token) = pagination_token {
        url.query_pairs_mut().append_pair("pagination_token", &token);
    }

    let resp = http_client()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            McpError::internal_error(format!("X API bookmarks request failed: {}", e), None)
        })?;

    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| {
        McpError::internal_error(format!("Failed to read X API bookmarks response: {}", e), None)
    })?;
    if !status.is_success() {
        return Err(McpError::internal_error(
            format!("X API GET /bookmarks {}: {}", status, body_text),
            None,
        ));
    }
    let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
        McpError::internal_error(format!("Failed to parse X API response: {}", e), None)
    })?;

    Ok(body)
}

pub async fn delete_bookmark(config: &ServerConfig, tweet_id: String) -> Result<serde_json::Value, McpError> {
    info!(
        x_user_id = config.user_id(),
        x_access_token = "present",
        tweet_id = %tweet_id,
        "xapimcp auth context for delete_bookmark (token redacted)"
    );
    let token = config.get_valid_x_access_token().await?;
    let base = format!("https://api.x.com/2/users/{}/bookmarks", config.user_id());
    let mut url =
        Url::parse(&base).map_err(|e| McpError::internal_error(format!("Invalid delete URL: {}", e), None))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| McpError::internal_error("Delete URL does not support path segments", None))?;
        segments.push(&tweet_id);
    }

    let resp = http_client()
        .delete(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            McpError::internal_error(format!("X API delete bookmark failed: {}", e), None)
        })?;

    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| {
        McpError::internal_error(format!("Failed to read X API delete response: {}", e), None)
    })?;
    if !status.is_success() {
        return Err(McpError::internal_error(
            format!("X API DELETE /bookmarks/{{id}} {}: {}", status, body_text),
            None,
        ));
    }
    let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
        McpError::internal_error(format!("Failed to parse delete response: {}", e), None)
    })?;

    Ok(body)
}

pub async fn get_replies_to_tweet(
    config: &ServerConfig,
    tweet_id: String,
) -> Result<serde_json::Value, McpError> {
    info!(
        x_user_id = config.user_id(),
        x_access_token = "present",
        tweet_id = %tweet_id,
        "xapimcp auth context for get_replies_to_tweet (token redacted)"
    );
    let token = config.get_valid_x_access_token().await?;
    let query = format!("conversation_id:{}", tweet_id);
    let url = Url::parse_with_params(
        "https://api.x.com/2/tweets/search/recent",
        &[
            ("query", query.as_str()),
            ("max_results", "100"),
            (
                "tweet.fields",
                "created_at,author_id,text,conversation_id,public_metrics,attachments,referenced_tweets,entities,lang",
            ),
            ("expansions", "author_id,attachments.media_keys"),
            ("user.fields", "id,name,username,verified,public_metrics"),
            (
                "media.fields",
                "media_key,type,url,preview_image_url,width,height,duration_ms,public_metrics",
            ),
        ],
    )
    .map_err(|e| McpError::internal_error(format!("Invalid replies URL: {}", e), None))?;

    let resp = http_client()
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            McpError::internal_error(format!("X API replies search failed: {}", e), None)
        })?;

    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| {
        McpError::internal_error(format!("Failed to read X API replies response: {}", e), None)
    })?;
    if !status.is_success() {
        return Err(McpError::internal_error(
            format!("X API GET /tweets/search/recent {}: {}", status, body_text),
            None,
        ));
    }
    let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|e| {
        McpError::internal_error(format!("Failed to parse replies response: {}", e), None)
    })?;

    Ok(body)
}