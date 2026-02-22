//! List entries in a directory.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Lists files and directories inside a given directory path.
pub struct FileListTool;

#[async_trait]
impl Tool for FileListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_list".to_string(),
            description: "List files and directories in a given path".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the directory to list"
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

        match tokio::fs::read_dir(path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_type = entry.file_type().await;
                    let kind = match file_type {
                        Ok(ft) if ft.is_dir() => "dir",
                        Ok(ft) if ft.is_symlink() => "symlink",
                        _ => "file",
                    };
                    items.push(json!({
                        "name": entry.file_name().to_string_lossy().to_string(),
                        "type": kind,
                    }));
                }
                let output = serde_json::to_string_pretty(&items)
                    .unwrap_or_else(|e| format!("Error serializing entries: {e}"));
                Ok(ToolResult {
                    call_id: ctx.call_id,
                    output,
                    is_error: false,
                })
            }
            Err(e) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error listing directory: {e}"),
                is_error: true,
            }),
        }
    }
}
