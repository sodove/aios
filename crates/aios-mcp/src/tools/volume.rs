//! Control audio volume.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Gets or sets the default audio sink volume via `wpctl`.
pub struct VolumeTool;

#[async_trait]
impl Tool for VolumeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "volume".to_string(),
            description: "Get or set audio volume (0-100)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "value": {
                        "type": "integer",
                        "description": "Volume percentage 0-100. Omit to read current volume."
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
        if let Some(value) = args.get("value").and_then(|v| v.as_u64()) {
            let clamped = value.min(100);
            let fraction = format!("{:.2}", f64::from(clamped as u32) / 100.0);

            let output = tokio::process::Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &fraction])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Volume set to {clamped}%"),
                    is_error: false,
                }),
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: format!("wpctl failed: {stderr}"),
                        is_error: true,
                    })
                }
                Err(e) => Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Error running wpctl: {e}"),
                    is_error: true,
                }),
            }
        } else {
            // Read current volume.
            let output = tokio::process::Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()
                .await;

            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: stdout.trim().to_string(),
                        is_error: false,
                    })
                }
                Ok(out) => {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Ok(ToolResult {
                        call_id: ctx.call_id,
                        output: format!("wpctl failed: {stderr}"),
                        is_error: true,
                    })
                }
                Err(e) => Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: format!("Error running wpctl: {e}"),
                    is_error: true,
                }),
            }
        }
    }
}
