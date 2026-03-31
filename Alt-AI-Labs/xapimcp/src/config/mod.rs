use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// X.com API consumer key (OAuth 1.0a app key).
    #[arg(long, env = "X_CONSUMER_KEY")]
    pub x_consumer_key: String,

    /// X.com API consumer secret (OAuth 1.0a app secret).
    #[arg(long, env = "X_CONSUMER_SECRET")]
    pub x_consumer_secret: Option<String>,

    /// Backward-compatible typo env var still supported if X_CONSUMER_SECRET is not set.
    #[arg(long, env = "X_CONSUMBER_SECRET")]
    pub x_consumber_secret: Option<String>,

    /// X.com OAuth 1.0a user access token.
    #[arg(long, env = "X_ACCESS_TOKEN")]
    pub x_access_token: String,

    /// X.com OAuth 1.0a user access token secret.
    #[arg(long, env = "X_ACCESS_TOKEN_SECRET")]
    pub x_access_token_secret: String,

    /// Your X.com numeric user ID. Omit to resolve automatically via GET /2/users/me (OAuth 1.0a).
    #[arg(long, env = "X_USER_ID")]
    pub x_user_id: Option<String>,

    /// Bind address for Streaable HTTP transport (0.0.0.0:8090).
    // If omitted the server runs in stdio mode (stdin/stdout).
    #[arg(long, env = "X_MCP_BIND_ADDRESS", default_value = "0.0.0.0:8090")]
    #[serde(skip)]
    pub bind_address: Option<String>,
}

impl ServerConfig {
    pub fn consumer_secret(&self) -> anyhow::Result<&str> {
        if let Some(s) = self.x_consumer_secret.as_deref() {
            let t = s.trim();
            if !t.is_empty() {
                return Ok(t);
            }
        }
        if let Some(s) = self.x_consumber_secret.as_deref() {
            let t = s.trim();
            if !t.is_empty() {
                return Ok(t);
            }
        }
        anyhow::bail!("X_CONSUMER_SECRET must be set (or legacy X_CONSUMBER_SECRET)");
    }

    /// Fills `x_user_id` via GET /2/users/me when unset or empty (OAuth 1.0a user token).
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
            self.x_user_id = Some(crate::x::fetch_authenticated_user_id(self).await?);
        }
        Ok(())
    }

    pub fn user_id(&self) -> &str {
        self.x_user_id
            .as_deref()
            .expect("x_user_id must be set after ensure_x_user_id")
    }
}