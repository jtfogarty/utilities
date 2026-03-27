use crate::config::ServerConfig;
use crate::salt;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::wrapper::parameters::Parameters,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo, Implementation, ProtocolVersion},
    schemars, tool, tool_handler,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct SaltService {
    config: ServerConfig,
}

#[derive(Debug, Deserialize, schemars::JsonSchema, Serialize)]
pub struct SaltExecuteRequest {
    #[schemars(description = "Target minions (glob, list, grain, etc.)")]
    pub tgt: String,
    #[schemars(description = "Salt function (e.g. test.ping, cmd.run, pkg.install)")]
    pub fun: String,
    #[schemars(description = "Optional arguments")]
    pub arg: Option<Vec<String>>,
    #[schemars(description = "Client type: local, local_async, runner, etc.")]
    pub client: Option<String>,
}

impl SaltService {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }
}

#[tool_handler]
impl ServerHandler for SaltService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "SaltStack MCP server. Use the salt_execute tool to run any Salt command via the local salt-api.".to_string(),
            ),
        }
    }
}

impl SaltService {
    #[tool(description = "Execute any Salt command via the existing salt-api")]
    async fn salt_execute(
        &self,
        Parameters(request): Parameters<SaltExecuteRequest>,
    ) -> Result<CallToolResult, McpError> {
        let token = salt::get_token(&self.config).await?;

        let payload = serde_json::json!({
            "client": request.client.unwrap_or_else(|| "local".to_string()),
            "tgt": request.tgt,
            "fun": request.fun,
            "arg": request.arg.unwrap_or_default(),
        });

        let resp = salt::http_client()
            .post(format!("{}/run", self.config.salt_api_url))
            .header("X-Auth-Token", token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| McpError {
                code: ErrorCode(-32603),
                message: Cow::from(format!("Salt API call failed: {}", e)),
                data: None,
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
            code: ErrorCode(-32603),
            message: Cow::from(format!("Failed to parse Salt response: {}", e)),
            data: None,
        })?;

        let result_text = serde_json::to_string_pretty(&body)
            .unwrap_or_else(|_| "Command executed successfully".to_string());

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }
}