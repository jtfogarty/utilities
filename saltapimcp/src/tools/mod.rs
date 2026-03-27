use crate::salt;
use rmcp::{
    handler::server::tool::ToolRouter,
    model::{CallToolResult, ToolContent},
    schemars, tool, tool_router, Parameters,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SaltTools;

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
    pub fn new() -> ToolRouter<Self> {
        Self::tool_router()
    }

    #[tool(description = "Execute any Salt command via the existing salt-api")]
    async fn salt_execute(
        &self,
        Parameters(request): Parameters<SaltExecuteRequest>,
        config: &crate::config::ServerConfig, // injected by server
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let token = salt::get_token(config).await?;

        let payload = serde_json::json!({
            "client": request.client.unwrap_or_else(|| "local".to_string()),
            "tgt": request.tgt,
            "fun": request.fun,
            "arg": request.arg.unwrap_or_default(),
        });

        let resp = salt::http_client()
            .post(format!("{}/run", config.salt_api_url))
            .header("X-Auth-Token", token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| rmcp::model::ErrorData {
                code: (-32603).into(),
                message: Cow::from(format!("Salt API call failed: {}", e)),
                data: None,
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| rmcp::model::ErrorData {
            code: (-32603).into(),
            message: Cow::from(format!("Failed to parse Salt response: {}", e)),
            data: None,
        })?;

        let result_text = serde_json::to_string_pretty(&body)
            .unwrap_or_else(|_| "Command executed successfully".to_string());

        Ok(CallToolResult::success(vec![ToolContent::text(result_text)]))
    }
}