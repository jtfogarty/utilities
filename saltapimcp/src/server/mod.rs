use crate::{config::ServerConfig, tools::SaltTools};
use anyhow::Result;
use rmcp::serve_server;
use tokio::io::{stdin, stdout};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting saltapimcp (stdio transport) — Salt API at {}", config.salt_api_url);

    let tools = SaltTools::new(config);

    let service = serve_server(tools, (stdin(), stdout())).await?;

    tracing::info!("saltapimcp ready — LLM agents can now call salt_execute");
    let _ = service.waiting().await;
    Ok(())
}