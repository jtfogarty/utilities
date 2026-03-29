use crate::{config::ServerConfig, tools::XTools};
use anyhow::Result;
use rmcp::{ServerHandler, ServiceExt, model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo}};

pub async fn start_server(config: ServerConfig) -> Result<()> {
    tracing::info!("Starting xapimcp (stdio transport) — X.com API ready");

    let tools = XTools::new();

    let service = rmcp::stdio()
        .serve(XServer { config, tools })
        .await?;

    tracing::info!("xapimcp ready — LLM agents can now call get_my_bookmarks, delete_bookmark, get_replies_to_bookmark");
    service.waiting().await?;
    Ok(())
}

#[derive(Clone)]
struct XServer {
    config: ServerConfig,
    tools: rmcp::handler::server::router::tool::ToolRouter<XTools>,
}

impl ServerHandler for XServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "X.com MCP server. Use the three bookmark tools to manage your personal X bookmarks and their replies.".to_string(),
            ),
        }
    }
}