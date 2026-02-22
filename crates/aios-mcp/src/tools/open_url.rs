//! Open a URL in the default browser.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Opens a URL in Chromium (or another configured browser).
pub struct OpenUrlTool;

#[async_trait]
impl Tool for OpenUrlTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "open_url".to_string(),
            description: "Open a URL in the browser".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to open"
                    }
                },
                "required": ["url"]
            }),
            trust_requirement: TrustRequirement::Confirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::Confirm
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'url' argument"))?;

        let output = tokio::process::Command::new("chromium")
            .arg(url)
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Opened {url} in browser"),
                is_error: false,
            }),
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Browser failed: {stderr}"),
                    is_error: true,
                })
            }
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error launching browser: {e}"),
                is_error: true,
            }),
        }
    }
}
