//! Type text into a browser input element.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Types text into an input element identified by a CSS selector.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserTypeTool;

#[async_trait]
impl Tool for BrowserTypeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_type".into(),
            description: "Type text into an input element on the current page".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "CSS selector of the input element"
                    },
                    "text": {
                        "type": "string",
                        "description": "The text to type into the element"
                    },
                    "clear_first": {
                        "type": "boolean",
                        "description": "Whether to clear existing text before typing (default: true)"
                    }
                },
                "required": ["selector", "text"]
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
