//! Execute a shell command.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Executes an arbitrary shell command via `sh -c`. This is a destructive
/// operation requiring double confirmation.
pub struct ShellExecTool;

#[async_trait]
impl Tool for ShellExecTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "shell_exec".to_string(),
            description: "Execute a shell command (destructive, requires double confirmation)"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Optional working directory for the command"
                    },
                    "timeout_ms": {
                        "type": "integer",
                        "description": "Timeout in milliseconds (default: 30000)"
                    }
                },
                "required": ["command"]
            }),
            trust_requirement: TrustRequirement::DoubleConfirm,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::DoubleConfirm
    }

    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'command' argument"))?;

        let working_dir = args.get("working_dir").and_then(|v| v.as_str());

        let timeout_ms = args
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(30_000);

        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(command);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(timeout_ms),
            cmd.output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let combined = json!({
                    "exit_code": exit_code,
                    "stdout": stdout.as_ref(),
                    "stderr": stderr.as_ref(),
                });

                Ok(ToolResult {
                    call_id: ctx.call_id,
                    output: combined.to_string(),
                    is_error: !output.status.success(),
                })
            }
            Ok(Err(e)) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Error executing command: {e}"),
                is_error: true,
            }),
            Err(_) => Ok(ToolResult {
                call_id: ctx.call_id,
                output: format!("Command timed out after {timeout_ms}ms"),
                is_error: true,
            }),
        }
    }
}
