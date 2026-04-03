//! X OAuth2 (User Context) for headless xapimcp.
//!
//! ## Workflow (one time + headless forever)
//! 1) On your laptop (with a browser available), run:
//!    `cargo run --bin authorize-x`
//! 2) Complete the X authorization in your browser.
//! 3) The tool prints an **X_REFRESH_TOKEN**.
//! 4) Copy that refresh token to your headless server config (env var or a file).
//! 5) The headless `xapimcp` service uses the refresh token to mint short-lived access tokens
//!    automatically and will persist refresh-token rotation (X may rotate refresh tokens).
//!
//! No interactive auth is performed on the headless server.

use oauth2::{
    AuthUrl, ClientId, ClientSecret, RefreshToken, RequestTokenError, TokenResponse, TokenUrl,
    basic::{BasicErrorResponse, BasicErrorResponseType, BasicClient},
    reqwest::AsyncHttpClientError,
};
use rmcp::model::ErrorData as McpError;

#[derive(Debug, Clone)]
pub struct XOAuthClient {
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: String,
}

impl XOAuthClient {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, redirect_uri: impl Into<String>) -> Result<Self, McpError> {
        let client_id = client_id.into();
        let client_secret = client_secret.into();
        let redirect_uri = redirect_uri.into();

        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);

        if redirect_uri.trim().is_empty() {
            return Err(McpError::internal_error(
                "redirect_uri must be non-empty".to_string(),
                None,
            ));
        }

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
        })
    }

    fn oauth_client(&self) -> Result<BasicClient, McpError> {
        let auth_url = AuthUrl::new("https://x.com/i/oauth2/authorize".to_string())
            .map_err(|e| McpError::internal_error(format!("Invalid authorize URL: {e}"), None))?;
        let token_url = TokenUrl::new("https://api.x.com/2/oauth2/token".to_string())
            .map_err(|e| McpError::internal_error(format!("Invalid token URL: {e}"), None))?;

        Ok(BasicClient::new(
            self.client_id.clone(),
            Some(self.client_secret.clone()),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            oauth2::RedirectUrl::new(self.redirect_uri.clone()).map_err(|e| {
                McpError::internal_error(format!("Invalid redirect URI: {e}"), None)
            })?,
        ))
    }
}

fn format_refresh_token_error(
    e: RequestTokenError<AsyncHttpClientError, BasicErrorResponse>,
) -> String {
    match e {
        RequestTokenError::ServerResponse(resp) => {
            let mut s = format!("X POST /2/oauth2/token: {resp}");
            match resp.error() {
                BasicErrorResponseType::InvalidGrant => {
                    s.push_str(" — `invalid_grant`: refresh token revoked/expired/wrong app, or Client ID+secret on this host do not match the X app that minted the token. Fix env and re-run `cargo run --bin authorize-x` on a laptop.");
                }
                BasicErrorResponseType::InvalidClient => {
                    s.push_str(" — `invalid_client`: check X_CLIENT_ID and X_CLIENT_SECRET (no extra whitespace; must be the confidential client for this app).");
                }
                BasicErrorResponseType::InvalidRequest => {
                    s.push_str(" — `invalid_request` + bad token: refresh value is often corrupted (BOM, quotes, extra lines in X_REFRESH_TOKEN_FILE) or no longer valid. Re-copy the token or run `authorize-x` again.");
                }
                _ => {}
            }
            s
        }
        RequestTokenError::Request(re) => format!("X POST /2/oauth2/token HTTP error: {re}"),
        RequestTokenError::Parse(pe, body) => format!(
            "X POST /2/oauth2/token: failed to parse response ({pe}); body={}",
            String::from_utf8_lossy(&body)
        ),
        RequestTokenError::Other(msg) => format!("X POST /2/oauth2/token: {msg}"),
    }
}

pub async fn refresh_access_token(
    client: &XOAuthClient,
    refresh_token: &str,
) -> Result<(String, Option<String>), McpError> {
    let rt = refresh_token.trim();
    if rt.is_empty() {
        return Err(McpError::internal_error(
            "No refresh token found — run `authorize-x` on your laptop first and copy the token to the server".to_string(),
            None,
        ));
    }

    let oauth = client.oauth_client()?;
    let refresh_token = RefreshToken::new(rt.to_string());

    let token_res = oauth
        .exchange_refresh_token(&refresh_token)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| McpError::internal_error(format_refresh_token_error(e), None))?;

    let access = token_res.access_token().secret().to_string();
    let new_refresh = token_res
        .refresh_token()
        .map(|t| t.secret().to_string());

    Ok((access, new_refresh))
}

