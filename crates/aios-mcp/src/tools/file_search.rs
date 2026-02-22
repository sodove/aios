//! Search for files by name pattern in a directory tree.

use std::path::Path;

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::executor::{Tool, ToolContext};

/// Recursively searches a directory tree for files whose names match a glob-like
/// pattern (simple `*` wildcard only).
pub struct FileSearchTool;

/// Check whether `name` matches `pattern` with simple `*` wildcard support.
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    // Split by `*` and verify that all fragments appear in order.
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut remaining = name;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            // First segment must be a prefix.
            if let Some(rest) = remaining.strip_prefix(part) {
                remaining = rest;
            } else {
                return false;
            }
        } else if let Some(pos) = remaining.find(part) {
            remaining = &remaining[pos + part.len()..];
        } else {
            return false;
        }
    }

    // If the pattern does not end with `*`, remaining must be empty.
    if !pattern.ends_with('*') && !remaining.is_empty() {
        return false;
    }

    true
}

/// Recursively walk `dir` collecting paths whose file name matches `pattern`.
fn walk_dir(dir: &Path, pattern: &str, results: &mut Vec<String>, max: usize) {
    if results.len() >= max {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if results.len() >= max {
            return;
        }
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && matches_pattern(name, pattern)
        {
            results.push(path.to_string_lossy().to_string());
        }
        if path.is_dir() {
            walk_dir(&path, pattern, results, max);
        }
    }
}

#[async_trait]
impl Tool for FileSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "file_search".to_string(),
            description: "Search for files by name pattern in a directory tree".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Root directory to search from"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "File name pattern with optional * wildcards (e.g. '*.rs')"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 100)"
                    }
                },
                "required": ["path", "pattern"]
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

        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing 'pattern' argument"))?;

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;

        let root = Path::new(path).to_path_buf();
        let pattern_owned = pattern.to_string();

        // Run blocking walk on a dedicated thread to avoid blocking the runtime.
        let results =
            tokio::task::spawn_blocking(move || {
                let mut results = Vec::new();
                walk_dir(&root, &pattern_owned, &mut results, max_results);
                results
            })
            .await
            .unwrap_or_default();

        let output = serde_json::to_string_pretty(&results)
            .unwrap_or_else(|e| format!("Error serializing results: {e}"));

        Ok(ToolResult {
            call_id: ctx.call_id,
            output,
            is_error: false,
        })
    }
}
