use crate::salt;
use rmcp::{
    ErrorData as McpError,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use crate::config::ServerConfig;

#[derive(Debug, Clone)]
pub struct SaltTools {
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

#[tool_router]
impl SaltTools {
    pub fn new(config: ServerConfig) -> ToolRouter<Self> {
        Self::tool_router()
    }

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
                code: -32603,
                message: Cow::from(format!("Salt API call failed: {}", e)),
                data: None,
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| McpError {
            code: -32603,
            message: Cow::from(format!("Failed to parse Salt response: {}", e)),
            data: None,
        })?;

        let result_text = serde_json::to_string_pretty(&body)
            .unwrap_or_else(|_| "Command executed successfully".to_string());

        Ok(CallToolResult::success(vec![Content::text(result_text)]))
    }
}