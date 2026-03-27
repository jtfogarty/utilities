use crate::config::ServerConfig;
use once_cell::sync::Lazy;
use reqwest::Client;
use rmcp::ErrorData as McpError;
use std::borrow::Cow;
use tokio::sync::Mutex;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);
static TOKEN: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub async fn get_token(config: &ServerConfig) -> Result<String, McpError> {
    let mut token_guard = TOKEN.lock().await;
    if let Some(t) = &*token_guard {
        return Ok(t.clone());
    }

    let resp = HTTP_CLIENT
        .post(format!("{}/login", config.salt_api_url))
        .json(&serde_json::json!({
            "username": &config.salt_user,
            "password": &config.salt_pass,
            "eauth": &config.salt_eauth,
        }))
        .send()
        .await
        .map_err(|e| McpError {
            code: -32603,
            message: Cow::from(format!("Login failed: {}", e)),
            data: None,
        })?;

    let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
        code: -32603,
        message: Cow::from(format!("Login response error: {}", e)),
        data: None,
    })?;

    let token = body["return"][0]["token"]
        .as_str()
        .ok_or_else(|| McpError {
            code: -32603,
            message: Cow::from("No token in login response"),
            data: None,
        })?
        .to_string();

    *token_guard = Some(token.clone());
    Ok(token)
}

pub fn http_client() -> &'static Client {
    &HTTP_CLIENT
}