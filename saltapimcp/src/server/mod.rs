use crate::{config::ServerConfig, tools::SaltTools};
use anyhow::Result;
use rmcp::{ServerHandler, ServiceExt, model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo}};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting saltapimcp (stdio transport) — Salt API at {}", config.salt_api_url);

    let tools = SaltTools::new();

    let service = rmcp::stdio()
        .serve(SaltServer { config, tools })
        .await?;

    tracing::info!("saltapimcp ready — LLM agents can now call salt_execute");
    service.waiting().await?;
    Ok(())
}

#[derive(Clone)]
struct SaltServer {
    config: ServerConfig,
    tools: rmcp::handler::server::router::tool::ToolRouter<SaltTools>,
}

impl ServerHandler for SaltServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "SaltStack MCP server. Use the salt_execute tool to run any command on your Salt infrastructure.".to_string(),
            ),
        }
    }
}