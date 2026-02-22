//! Read the contents of a file.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Reads a file and returns its contents as a UTF-8 string.
pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_read".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the file to read"
                    }
                },
                "required": ["path"]
            }),
            trust_requirement: TrustRequirement::None,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::None
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'path' argument"))?;

        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: content,
                is_error: false,
            }),
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error reading file: {e}"),
                is_error: true,
            }),
        }
    }
}
