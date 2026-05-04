use crate::config::ServerConfig;
use crate::salt;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo, Implementation},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct SaltService {
    pub config: ServerConfig,
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SaltService {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Execute any Salt command via the existing salt-api")]
    pub async fn salt_execute(
        &self,
        params: Parameters<SaltExecuteRequest>,   // <-- surrealmcp style (no destructuring here)
    ) -> Result<CallToolResult, McpError> {
        let request = params.0;   // destructure once inside the function

        let token = salt::get_token(&self.config).await?;

        let payload = serde_json::json!({
            "client": request.client.unwrap_or_else(|| "local".to_string()),
            "tgt": request.tgt,
            "fun": request.fun,
            "arg": request.arg.unwrap_or_default(),
        });

        let resp = salt::http_client()
            .post(format!("{}/", self.config.salt_api_url))
            .header("X-Auth-Token", token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: Cow::from(format!("Salt API call failed: {}", e)),
                data: None,
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: Cow::from(format!("Failed to parse Salt response: {}", e)),
            data: None,
        })?;

        let result_text = serde_json::to_string_pretty(&body)
            .unwrap_or_else(|_| "Command executed successfully".to_string());

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }
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

#[tool_handler]
impl ServerHandler for SaltService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions("SaltStack MCP server. Use the salt_execute tool to run any Salt command via the local salt-api.")
    }
}