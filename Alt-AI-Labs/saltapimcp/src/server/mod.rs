use anyhow::{Result, anyhow};
use axum::{
    Router,
    extract::Request,
    http::{StatusCode, header},
    middleware::{self, Next},
    response::Response,
};
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::ServerConfig;
use crate::tools::SaltService;

/// Build the list of allowed `Host` header values for rmcp's DNS-rebinding
/// guard. Always includes loopback. Adds the concrete bind IP:port when bound
/// to a non-loopback, non-wildcard address (e.g. a LAN IP).
fn build_allowed_hosts(bind_address: &str) -> Result<Vec<String>> {
    let bind: SocketAddr = bind_address
        .parse()
        .map_err(|e| anyhow!("Invalid bind address {bind_address}: {e}"))?;
    let port = bind.port();

    let mut hosts: Vec<String> = vec![
        "localhost".into(),
        format!("localhost:{port}"),
        "127.0.0.1".into(),
        format!("127.0.0.1:{port}"),
        "::1".into(),
        format!("[::1]:{port}"),
    ];

    let ip = bind.ip();
    if !ip.is_unspecified() && !ip.is_loopback() {
        let ip_str = ip.to_string();
        let host_port = match ip {
            IpAddr::V4(_) => format!("{ip_str}:{port}"),
            IpAddr::V6(_) => format!("[{ip_str}]:{port}"),
        };
        hosts.push(ip_str);
        hosts.push(host_port);
    }

    hosts.sort();
    hosts.dedup();
    Ok(hosts)
}

// Bearer-token authentication middleware.
///
/// If `expected_token` is `None (no AUTH_TOKEN configured) all requests pass through.
/// The `/health` endpoint is always exempt so Load-balancers can probe without credentials.
async fn bearer_auth(
    expected_token: Option<String>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no token is configured, skip auth entirely.
    let expected = match &expected_token {
        Some(t) if !t.is_empty() => t,
        _ => return Ok(next.run(request).await),
    };

    // Always allow the health endpoint through unauthenticated.
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Extract and validate the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    match auth_header {
        Some(value) if value.starts_with("Bearer ") => {
            let token = &value["Bearer ".len()..];
            if token == expected.as_str() {
                Ok(next.run(request).await)
            } else {
                tracing::warn!("Rejected request: invalid bearer token");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            tracing::warn!("Rejected request: missing or invalid Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn start_server(config: ServerConfig) -> Result<()> {
    let bind_address = config.bind_address.clone();
    let auth_token = config.auth_token.clone();

    if auth_token.as_ref().is_some_and(|t| !t.is_empty()) {
        tracing::info!("Bearer-token Authentication is ENABLED");
    } else {
        tracing::info!("Bearer-token Authentication is DISABLED - all requests will be accepted");
    }

    tracing::info!(
        bind_address = %bind_address,
        salt_api_url = %config.salt_api_url,
        "Starting saltapimcp (HTTP/MCP transport)"
    );

    // Build the allowed-hosts list for rmcp's DNS-rebinding guard. rmcp 1.6
    // defaults to loopback only, which 403s any request to a LAN IP before
    // auth even runs.
    let allowed_hosts = build_allowed_hosts(&bind_address)?;
    tracing::info!(
        allowed_hosts = ?allowed_hosts,
        "Configured Streamable HTTP allowed hosts"
    );

    let session_manager = Arc::new(LocalSessionManager::default());

    let http_config = StreamableHttpServerConfig::default().with_allowed_hosts(allowed_hosts);

    let mcp_service = StreamableHttpService::new(
        move || Ok(SaltService::new(config.clone())),
        session_manager,
        http_config,
    );

    let router = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(middleware::from_fn(move |req, next| {
            bearer_auth(auth_token.clone(), req, next)
    }));

    let listener = TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow!("Failed to bind to {bind_address}: {e}"))?;

    tracing::info!("saltapimcp ready — MCP endpoint: http://{}/mcp", bind_address);

    axum::serve(listener, router).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_allowed_hosts;

    #[test]
    fn allowed_hosts_for_lan_ip_bind() {
        let hosts = build_allowed_hosts("10.10.3.8:8765").unwrap();
        assert!(hosts.iter().any(|h| h == "localhost"));
        assert!(hosts.iter().any(|h| h == "localhost:8765"));
        assert!(hosts.iter().any(|h| h == "127.0.0.1"));
        assert!(hosts.iter().any(|h| h == "127.0.0.1:8765"));
        assert!(hosts.iter().any(|h| h == "::1"));
        assert!(hosts.iter().any(|h| h == "[::1]:8765"));
        assert!(hosts.iter().any(|h| h == "10.10.3.8"));
        assert!(hosts.iter().any(|h| h == "10.10.3.8:8765"));
    }

    #[test]
    fn allowed_hosts_for_wildcard_bind() {
        let hosts = build_allowed_hosts("0.0.0.0:8765").unwrap();
        assert!(hosts.iter().any(|h| h == "localhost:8765"));
        assert!(hosts.iter().any(|h| h == "127.0.0.1:8765"));
        assert!(!hosts.iter().any(|h| h == "0.0.0.0"));
    }

    #[test]
    fn allowed_hosts_for_loopback_bind_skips_duplicate_ip() {
        let hosts = build_allowed_hosts("127.0.0.1:8765").unwrap();
        let count_127 = hosts.iter().filter(|h| h.as_str() == "127.0.0.1").count();
        assert_eq!(count_127, 1, "loopback should not be added twice");
        let count_port = hosts
            .iter()
            .filter(|h| h.as_str() == "127.0.0.1:8765")
            .count();
        assert_eq!(count_port, 1);
    }

    #[test]
    fn allowed_hosts_for_ipv6_bind() {
        let hosts = build_allowed_hosts("[2001:db8::1]:8765").unwrap();
        assert!(hosts.iter().any(|h| h == "2001:db8::1"));
        assert!(hosts.iter().any(|h| h == "[2001:db8::1]:8765"));
    }

    #[test]
    fn allowed_hosts_rejects_garbage_bind() {
        assert!(build_allowed_hosts("not-a-socket").is_err());
    }
}
