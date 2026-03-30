use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// X.com API v2 Bearer token (OAuth 2.0 user access token with bookmark.read + bookmark.write scopes)
    #[arg(long, env = "X_BEARER_TOKEN")]
    pub x_bearer_token: String,

    /// Your X.com numeric user ID (find it at https://x.com/whoami or via /2/users/me)
    #[arg(long, env = "X_USER_ID")]
    pub x_user_id: String,

    /// Bind address for Streaable HTTP transport (0.0.0.0:8090).
    // If omitted the server runs in stdio mode (stdin/stdout).
    #[arg(long, env = "X_MCP_BIND_ADDRESS", default_value = "0.0.0.0:8090")]
    #[serde(skip)]
    pub bind_address: Option<String>,
}