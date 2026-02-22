//! Control display brightness.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Reads or sets screen brightness via `/sys/class/backlight`.
pub struct BrightnessTool;

/// Find the first backlight device directory under `/sys/class/backlight/`.
async fn find_backlight_dir() -> std::io::Result<std::path::PathBuf> {
    let mut entries = tokio::fs::read_dir("/sys/class/backlight").await?;
    if let Some(entry) = entries.next_entry().await? {
        Ok(entry.path())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no backlight device found",
        ))
    }
}

#[async_trait]
impl Tool for BrightnessTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "brightness".to_string(),
            description: "Get or set display brightness (0-100)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "value": {
                        "type": "integer",
                        "description": "Brightness value 0-100. Omit to read current brightness."
                    }
                },
                "required": []
            }),
            trust_requirement: TrustRequirement::Confirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::Confirm
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let bl_dir = match find_backlight_dir().await {
            Ok(d) => d,
            Err(e) => {
                return Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Error finding backlight device: {e}"),
                    is_error: true,
                });
            }
        };

        let max_brightness_path = bl_dir.join("max_brightness");
        let brightness_path = bl_dir.join("brightness");

        let max_raw = match tokio::fs::read_to_string(&max_brightness_path).await {
            Ok(s) => s,
            Err(e) => {
                return Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Error reading max_brightness: {e}"),
                    is_error: true,
                });
            }
        };
        let max_val: u64 = max_raw.trim().parse().unwrap_or(100);

        if let Some(value) = args.get("value").and_then(|v| v.as_u64()) {
            // Set brightness.
            let clamped = value.min(100);
            let raw = max_val * clamped / 100;
            match tokio::fs::write(&brightness_path, raw.to_string()).await {
                Ok(()) => Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Brightness set to {clamped}%"),
                    is_error: false,
                }),
                Err(e) => Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Error writing brightness: {e}"),
                    is_error: true,
                }),
            }
        } else {
            // Read current brightness.
            let cur_raw = match tokio::fs::read_to_string(&brightness_path).await {
                Ok(s) => s,
                Err(e) => {
                    return Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: format!("Error reading brightness: {e}"),
                        is_error: true,
                    });
                }
            };
            let cur_val: u64 = cur_raw.trim().parse().unwrap_or(0);
            let percent = if max_val > 0 {
                cur_val * 100 / max_val
            } else {
                0
            };
            Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Current brightness: {percent}%"),
                is_error: false,
            })
        }
    }
}
