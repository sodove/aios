//! Connect to a Wi-Fi network.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Connects to a Wi-Fi network by SSID, optionally with a password.
pub struct WifiConnectTool;

#[async_trait]
impl Tool for WifiConnectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "wifi_connect".to_string(),
            description: "Connect to a Wi-Fi network by SSID".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "ssid": {
                        "type": "string",
                        "description": "SSID of the network to connect to"
                    },
                    "password": {
                        "type": "string",
                        "description": "Password for the network (optional for open networks)"
                    }
                },
                "required": ["ssid"]
            }),
            trust_requirement: TrustRequirement::Confirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::Confirm
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let ssid = args
            .get("ssid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'ssid' argument"))?;

        let password = args.get("password").and_then(|v| v.as_str());

        let mut cmd = tokio::process::Command::new("nmcli");
        cmd.args(["dev", "wifi", "connect", ssid]);

        if let Some(pw) = password {
            cmd.args(["password", pw]);
        }

        let output = cmd.output().await;

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
                        output: format!("Failed to connect: {stderr}"),
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
