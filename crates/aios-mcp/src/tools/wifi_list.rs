//! List available Wi-Fi networks.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Lists available Wi-Fi networks using `nmcli`.
pub struct WifiListTool;

#[async_trait]
impl Tool for WifiListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "wifi_list".to_string(),
            description: "List available Wi-Fi networks".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            trust_requirement: TrustRequirement::None,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::None
    }

    async fn execute(&self, _args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let output = tokio::process::Command::new("nmcli")
            .args(["dev", "wifi", "list"])
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if out.status.success() {
                    Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: stdout.to_string(),
                        is_error: false,
                    })
                } else {
                    Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: format!("nmcli failed: {stderr}"),
                        is_error: true,
                    })
                }
            }
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error running nmcli: {e}"),
                is_error: true,
            }),
        }
    }
}
