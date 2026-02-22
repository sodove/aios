//! Find an element on the current browser page.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Finds a DOM element by CSS selector or `XPath`.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserFindTool;

#[async_trait]
impl Tool for BrowserFindTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_find".into(),
            description: "Find an element on the current page by CSS selector or XPath".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector to locate the element"
                    },
                    "xpath": {
                        "type": "string",
                        "description": "XPath expression to locate the element (alternative to selector)"
                    }
                },
                "required": ["selector"]
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
