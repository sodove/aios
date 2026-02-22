//! Gather system information.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Collects system information: CPU, memory, disk, and battery status.
/// Returns data as a JSON object.
pub struct SystemInfoTool;

/// Read a file and return its contents, or an empty string on error.
async fn read_or_empty(path: &str) -> String {
    tokio::fs::read_to_string(path)
        .await
        .unwrap_or_default()
}

/// Run a command and return stdout, or an empty string on error.
async fn run_or_empty(program: &str, args: &[&str]) -> String {
    tokio::process::Command::new(program)
        .args(args)
        .output()
        .await
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

#[async_trait]
impl Tool for SystemInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "system_info".to_string(),
            description: "Get system information (CPU, memory, disk, battery)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            trust_requirement: TrustRequirement::None,
        }
    }

    fn trust_requirement(&self) -> TrustRequirement {
        TrustRequirement::None
    }

    async fn execute(&self, _args: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let cpuinfo = read_or_empty("/proc/cpuinfo").await;
        let meminfo = read_or_empty("/proc/meminfo").await;
        let df_output = run_or_empty("df", &["-h"]).await;

        // Try to read battery status from common sysfs paths.
        let battery_status = read_or_empty(
            "/sys/class/power_supply/BAT0/status",
        )
        .await;
        let battery_capacity = read_or_empty(
            "/sys/class/power_supply/BAT0/capacity",
        )
        .await;

        // Extract CPU model name (first occurrence).
        let cpu_model = cpuinfo
            .lines()
            .find(|l| l.starts_with("model name"))
            .and_then(|l| l.split(':').nth(1))
            .map(str::trim)
            .unwrap_or("unknown")
            .to_string();

        // Extract total and available memory.
        let extract_mem = |key: &str| -> String {
            meminfo
                .lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split(':').nth(1))
                .map(str::trim)
                .unwrap_or("unknown")
                .to_string()
        };

        let mem_total = extract_mem("MemTotal");
        let mem_available = extract_mem("MemAvailable");

        let info = json!({
            "cpu_model": cpu_model,
            "memory": {
                "total": mem_total,
                "available": mem_available,
            },
            "disk": df_output.trim(),
            "battery": {
                "status": battery_status.trim(),
                "capacity": battery_capacity.trim(),
            },
        });

        Ok(ToolResult {
            call_id: ctx.call_id,
            output: serde_json::to_string_pretty(&info)
                .unwrap_or_else(|e| format!("Error serializing system info: {e}")),
            is_error: false,
        })
    }
}
