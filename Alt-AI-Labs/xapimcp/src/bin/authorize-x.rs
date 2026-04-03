// Load crate `.env` before reading env vars (same as main binary).
#[path = "../dotenv_load.rs"]
mod dotenv_load;

use axum::{Router, extract::Query, routing::get};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
    basic::BasicClient, AuthUrl, TokenUrl, ClientId, ClientSecret, RedirectUrl,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

fn env_one_of(keys: &[&str]) -> anyhow::Result<String> {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            let t = v.trim();
            if !t.is_empty() {
                return Ok(t.to_string());
            }
        }
    }
    anyhow::bail!("Missing required env var (checked: {})", keys.join(", "))
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

fn x_oauth_scopes() -> Vec<Scope> {
    // bookmark.write is required for DELETE bookmarks.
    [
        "tweet.read",
        "users.read",
        "bookmark.read",
        "bookmark.write",
        "offline.access",
    ]
    .into_iter()
    .map(|s| Scope::new(s.to_string()))
    .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv_load::load();

    // Laptop-only helper. Requires a browser for interactive login.
    // Prefer CONSUMER_* first: some `.env` files set X_CLIENT_ID to a username by mistake.
    let client_id = env_one_of(&["X_CONSUMER_KEY", "X_CLIENT_ID"])?;
    let client_secret = env_one_of(&["X_CONSUMER_SECRET", "X_CLIENT_SECRET", "X_CONSUMBER_SECRET"])?;

    let redirect_uri = "http://127.0.0.1:8080/callback".to_string();
    let auth_url = AuthUrl::new("https://x.com/i/oauth2/authorize".to_string())?;
    let token_url = TokenUrl::new("https://api.x.com/2/oauth2/token".to_string())?;

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri.clone())?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut auth_req = client.authorize_url(CsrfToken::new_random);
    for s in x_oauth_scopes() {
        auth_req = auth_req.add_scope(s);
    }
    let (auth_url, csrf_state) = auth_req.set_pkce_challenge(pkce_challenge).url();

    println!();
    println!("Open this URL in your browser to authorize xapimcp:");
    println!("{auth_url}");
    println!();
    println!("Waiting for callback on {redirect_uri} ...");
    println!();

    let (tx, rx) = oneshot::channel::<CallbackQuery>();
    // Axum 0.8 handlers must be `Clone`; wrap the oneshot sender so the route closure only captures `Arc`.
    let tx_slot: Arc<Mutex<Option<oneshot::Sender<CallbackQuery>>>> = Arc::new(Mutex::new(Some(tx)));

    let app = Router::new().route(
        "/callback",
        get({
            let tx_slot = tx_slot.clone();
            move |Query(q): Query<CallbackQuery>| {
                let tx_slot = tx_slot.clone();
                async move {
                    if let Some(sender) = tx_slot.lock().await.take() {
                        let _ = sender.send(q);
                    }
                    "Authorization received. You can close this tab."
                }
            }
        }),
    );

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);
    let server_handle = tokio::spawn(async move {
        let _ = server.await;
    });

    let CallbackQuery { code, state } = rx.await?;
    if state != *csrf_state.secret() {
        anyhow::bail!("CSRF state mismatch (expected {}, got {})", csrf_state.secret(), state);
    }

    let token_res = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.secret().to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let refresh = token_res
        .refresh_token()
        .map(|t| t.secret().to_string())
        .ok_or_else(|| anyhow::anyhow!("No refresh token returned. Ensure you requested offline.access scope and your X app allows it."))?;

    println!();
    println!("==== COPY THIS TO YOUR HEADLESS SERVER CONFIG ====");
    println!("X_REFRESH_TOKEN={refresh}");
    println!("==================================================");
    println!();
    println!("Next:");
    println!("- Set `X_CLIENT_ID`/`X_CLIENT_SECRET` (or `X_CONSUMER_KEY`/`X_CONSUMER_SECRET`) on the server");
    println!("- Set `X_REFRESH_TOKEN` OR write it to a file and set `X_REFRESH_TOKEN_FILE`");
    println!("- Restart xapimcp");

    server_handle.abort();
    Ok(())
}

