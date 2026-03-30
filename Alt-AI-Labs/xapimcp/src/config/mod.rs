use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// X.com API v2 Bearer token (OAuth 2.0 user access token with bookmark.read + bookmark.write scopes)
    #[arg(long, env = "X_BEARER_TOKEN")]
    pub x_bearer_token: String,

    /// Your X.com numeric user ID. Omit to resolve automatically via GET /2/users/me (same bearer token).
    #[arg(long, env = "X_USER_ID")]
    pub x_user_id: Option<String>,

    /// Bind address for Streaable HTTP transport (0.0.0.0:8090).
    // If omitted the server runs in stdio mode (stdin/stdout).
    #[arg(long, env = "X_MCP_BIND_ADDRESS", default_value = "0.0.0.0:8090")]
    #[serde(skip)]
    pub bind_address: Option<String>,
}

impl ServerConfig {
    /// Fills `x_user_id` via GET /2/users/me when unset or empty (OAuth2 user token).
    pub async fn ensure_x_user_id(&mut self) -> anyhow::Result<()> {
        if let Some(ref s) = self.x_user_id {
            let t = s.trim();
            if t.is_empty() {
                self.x_user_id = None;
            } else if t != s {
                self.x_user_id = Some(t.to_string());
            }
        }
        if self.x_user_id.is_none() {
            self.x_user_id = Some(
                crate::x::fetch_authenticated_user_id(&self.x_bearer_token).await?,
            );
        }
        Ok(())
    }

    pub fn user_id(&self) -> &str {
        self.x_user_id
            .as_deref()
            .expect("x_user_id must be set after ensure_x_user_id")
    }
}