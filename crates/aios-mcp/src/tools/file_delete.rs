//! Delete a file from the filesystem.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Deletes a single file. This is a destructive operation requiring double
/// confirmation.
pub struct FileDeleteTool;

#[async_trait]
impl Tool for FileDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_delete".to_string(),
            description: "Delete a file (destructive, requires double confirmation)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the file to delete"
                    }
                },
                "required": ["path"]
            }),
            trust_requirement: TrustRequirement::DoubleConfirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::DoubleConfirm
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'path' argument"))?;

        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Successfully deleted {path}"),
                is_error: false,
            }),
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error deleting file: {e}"),
                is_error: true,
            }),
        }
    }
}
