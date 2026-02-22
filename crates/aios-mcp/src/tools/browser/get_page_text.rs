//! Extract text content from the current browser page.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

const MCP_STUB_MSG: &str =
    "Chrome MCP integration not yet available. \
     This tool requires the Chrome MCP extension to be installed and connected.";

/// Extracts the visible text content from the current browser page.
///
/// Unlike [`BrowserReadPageTool`](super::read_page::BrowserReadPageTool) which
/// returns raw DOM content, this tool returns only the human-readable text.
///
/// **Stub** -- requires Chrome MCP extension integration.
pub struct BrowserGetPageTextTool;

#[async_trait]
impl Tool for BrowserGetPageTextTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "browser_get_page_text".into(),
            description:
                "Extract the visible text content from the current browser page (no HTML tags)"
                    .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "selector": {
                        "type": "string",
                        "description": "Optional CSS selector to limit extraction to a specific element"
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
