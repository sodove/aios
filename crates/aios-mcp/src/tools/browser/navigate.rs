//! Navigate the browser to a URL.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Opens a URL in the Chromium browser.
///
/// Unlike other browser tools this one works without the Chrome MCP
/// extension -- it simply spawns a Chromium process with the target URL.
pub struct BrowserNavigateTool;

#[async_trait]
impl Tool for BrowserNavigateTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_navigate".into(),
            description: "Open a URL in the Chromium browser and navigate to it".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to navigate to (must include scheme, e.g. https://)"
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
            .ok_or_else(|| anyhow::anyhow!("missing required 'url' argument"))?;

        // Spawn Chromium in the background -- we do not wait for it to exit
        // because a browser process stays alive until the user closes it.
        let spawn_result = tokio::process::Command::new("chromium")
            .arg("--ozone-platform-hint=auto")
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match spawn_result {
            Ok(_child) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Navigated to {url} in Chromium"),
                is_error: false,
            }),
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Failed to launch Chromium: {e}"),
                is_error: true,
            }),
        }
    }
}
