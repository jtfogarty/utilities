use clap::Args;
use serde::{Deserialize, Serialize};
use rmcp::model::ErrorData as McpError;
use std::path::PathBuf;

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Your X.com numeric user ID. Omit to resolve automatically via GET /2/users/me.
    #[arg(long, env = "X_USER_ID")]
    pub x_user_id: Option<String>,

    /// X OAuth2 client id (aka consumer key). Used for refresh-token flow.
    #[arg(long, env = "X_CLIENT_ID")]
    pub x_client_id: Option<String>,

    /// Legacy alias for client id.
    #[arg(long, env = "X_CONSUMER_KEY")]
    pub x_consumer_key: Option<String>,

    /// X OAuth2 client secret (aka consumer secret). Used for refresh-token flow.
    #[arg(long, env = "X_CLIENT_SECRET")]
    pub x_client_secret: Option<String>,

    /// Legacy alias for client secret.
    #[arg(long, env = "X_CONSUMER_SECRET")]
    pub x_consumer_secret: Option<String>,

    /// Common typo for `X_CONSUMER_SECRET` (still read from env for compatibility).
    #[arg(long, env = "X_CONSUMBER_SECRET")]
    pub x_consumber_secret: Option<String>,

    /// X OAuth2 refresh token (laptop-seeded, headless forever).
    #[arg(long, env = "X_REFRESH_TOKEN")]
    pub x_refresh_token: Option<String>,

    /// Optional file path holding the refresh token. If set, xapimcp reads this file
    /// (falling back to X_REFRESH_TOKEN). On refresh-token rotation, xapimcp overwrites this file.
    #[arg(long, env = "X_REFRESH_TOKEN_FILE")]
    pub x_refresh_token_file: Option<PathBuf>,

    /// Bind address for Streaable HTTP transport (0.0.0.0:8090).
    // If omitted the server runs in stdio mode (stdin/stdout).
    #[arg(long, env = "X_MCP_BIND_ADDRESS", default_value = "0.0.0.0:8090")]
    #[serde(skip)]
    pub bind_address: Option<String>,
}

fn normalize_refresh_token(raw: &str) -> String {
    let s = raw.trim();
    let s = s.strip_prefix('\u{FEFF}').unwrap_or(s).trim();
    // Refresh tokens are a single line; ignore accidental extra lines / trailing junk.
    let line = s.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    let mut t = line.to_string();
    if t.len() >= 2 {
        let quoted_double = t.starts_with('"') && t.ends_with('"');
        let quoted_single = t.starts_with('\'') && t.ends_with('\'');
        if quoted_double || quoted_single {
            t = t[1..t.len() - 1].trim().to_string();
        }
    }
    t
}

impl ServerConfig {
    fn client_id(&self) -> Result<String, McpError> {
        // Prefer explicit OAuth2 names from the X portal. Legacy CONSUMER_KEY still works if
        // CLIENT_ID is unset (but a stale CONSUMER_KEY must not override a valid X_CLIENT_ID).
        let v = self
            .x_client_id
            .as_deref()
            .or(self.x_consumer_key.as_deref())
            .unwrap_or("")
            .trim()
            .to_string();
        if v.is_empty() {
            return Err(McpError::internal_error(
                "X_CLIENT_ID or X_CONSUMER_KEY must be set (OAuth2 app client id)".to_string(),
                None,
            ));
        }
        Ok(v)
    }

    fn client_secret(&self) -> Result<String, McpError> {
        let v = self
            .x_client_secret
            .as_deref()
            .or(self.x_consumer_secret.as_deref())
            .or(self.x_consumber_secret.as_deref())
            .unwrap_or("")
            .trim()
            .to_string();
        if v.is_empty() {
            return Err(McpError::internal_error(
                "X_CLIENT_SECRET or X_CONSUMER_SECRET must be set (OAuth2 app client secret)".to_string(),
                None,
            ));
        }
        Ok(v)
    }

    fn load_refresh_token(&self) -> Result<String, McpError> {
        if let Some(s) = self.x_refresh_token.as_deref() {
            let t = normalize_refresh_token(s);
            if !t.is_empty() {
                return Ok(t);
            }
        }

        if let Some(path) = self.x_refresh_token_file.as_ref() {
            let text = std::fs::read_to_string(path).map_err(|e| {
                McpError::internal_error(
                    format!(
                        "Failed to read X_REFRESH_TOKEN_FILE ({}): {}",
                        path.display(),
                        e
                    ),
                    None,
                )
            })?;
            let t = normalize_refresh_token(&text);
            if !t.is_empty() {
                return Ok(t);
            }
        }

        Err(McpError::internal_error(
            "No refresh token found — run `authorize-x` on your laptop first and copy the token to the server".to_string(),
            None,
        ))
    }

    fn persist_refresh_token(&self, new_refresh_token: &str) -> Result<(), McpError> {
        let t = new_refresh_token.trim();
        if t.is_empty() {
            return Ok(());
        }

        let Some(path) = self.x_refresh_token_file.as_ref() else {
            return Ok(());
        };

        use std::io::Write;

        #[cfg(unix)]
        fn set_mode(p: &std::path::Path) -> std::io::Result<()> {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(p)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(p, perms)
        }

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|e| {
                McpError::internal_error(
                    format!(
                        "Failed to write X_REFRESH_TOKEN_FILE ({}): {}",
                        path.display(),
                        e
                    ),
                    None,
                )
            })?;
        f.write_all(t.as_bytes()).map_err(|e| {
            McpError::internal_error(
                format!(
                    "Failed to write X_REFRESH_TOKEN_FILE ({}): {}",
                    path.display(),
                    e
                ),
                None,
            )
        })?;
        f.write_all(b"\n").ok();

        #[cfg(unix)]
        {
            set_mode(path).ok();
        }

        Ok(())
    }

    pub async fn get_valid_x_access_token(&self) -> Result<String, McpError> {
        let refresh_token = self.load_refresh_token()?;
        let client = crate::x_oauth::XOAuthClient::new(
            self.client_id()?,
            self.client_secret()?,
            // Redirect is unused for refresh, but XOAuthClient expects it.
            "http://127.0.0.1:8080/callback",
        )?;

        let (access, new_refresh) =
            crate::x_oauth::refresh_access_token(&client, &refresh_token).await?;
        if let Some(nrt) = new_refresh {
            let _ = self.persist_refresh_token(&nrt);
        }
        Ok(access)
    }

    /// Fills `x_user_id` via GET /2/users/me when unset or empty.
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

    pub fn user_id(&self) -> Result<&str, McpError> {
        let Some(s) = self.x_user_id.as_deref() else {
            return Err(McpError::internal_error(
                "X_USER_ID is not set — run ensure_x_user_id after X auth succeeds".to_string(),
                None,
            ));
        };
        let t = s.trim();
        if t.is_empty() {
            return Err(McpError::internal_error(
                "X_USER_ID is not set — run ensure_x_user_id after X auth succeeds".to_string(),
                None,
            ));
        }
        Ok(t)
    }
}