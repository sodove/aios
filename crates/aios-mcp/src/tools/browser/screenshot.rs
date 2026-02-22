//! Take a screenshot of the current browser page.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Captures a screenshot of the current browser page or a specific element.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserScreenshotTool;

#[async_trait]
impl Tool for BrowserScreenshotTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_screenshot".into(),
            description: "Take a screenshot of the current browser page or a specific element"
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "Optional CSS selector to screenshot a specific element"
                    },
                    "full_page": {
                        "type": "boolean",
                        "description": "Whether to capture the full scrollable page (default: false)"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "File path to save the screenshot (PNG format)"
                    }
                },
                "required": []
            }),
            trust_requirement: TrustRequirement::None,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::None
    }

    async fn execute(&self, _args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        Ok(ToolResult {
            call_id: ctx.call_id,
            output: MCP_STUB_MSG.into(),
            is_error: true,
        })
    }
}
