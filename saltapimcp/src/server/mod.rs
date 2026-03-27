use anyhow::Result;
use crate::config::ServerConfig;
use crate::tools::SaltService;
use tokio::io::{stdin, stdout};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting saltapimcp (stdio transport) — Salt API at {}", config.salt_api_url);

    let service = SaltService::new(config);

    let server = rmcp::serve_server(service, (stdin(), stdout())).await?;

    tracing::info!("saltapimcp ready — LLM agents can now call salt_execute");
    let _ = server.waiting().await;
    Ok(())
}