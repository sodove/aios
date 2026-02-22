//! Click an element in the browser.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Clicks on a DOM element identified by a CSS selector.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserClickTool;

#[async_trait]
impl Tool for BrowserClickTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_click".into(),
            description: "Click an element on the current page identified by CSS selector".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector of the element to click"
                    }
                },
                "required": ["selector"]
            }),
            trust_requirement: TrustRequirement::Confirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::Confirm
    }

    async fn execute(&self, _args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        Ok(ToolResult {
            call_id: ctx.call_id,
            output: MCP_STUB_MSG.into(),
            is_error: true,
        })
    }
}
