//! Read the current page content from the browser.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Reads the DOM / rendered content of the current browser page.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserReadPageTool;

#[async_trait]
impl Tool for BrowserReadPageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_read_page".into(),
            description: "Read the rendered content of the current browser page".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "Optional CSS selector to limit the read scope"
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
