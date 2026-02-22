//! Append-only audit logger for tool execution events.
//!
//! Writes one JSON object per line (JSON Lines / NDJSON) to a configurable
//! file path.  Every tool invocation -- whether approved, rejected, or
//! timed-out -- is recorded so that the full action history can be
//! reconstructed later for security review.

use std::path::PathBuf;

use aios_common::{AuditEntry, AuditResult, ToolCall, ToolResult as ToolExecResult};
use chrono::Utc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Persistent, append-only audit logger backed by a JSON Lines file.
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new logger that appends entries to `log_path`.
    ///
    /// The file (and its parent directories) are created lazily on the first
    /// write, so construction never fails.
    pub fn new(log_path: impl Into<PathBuf>) -> Self {
        Self {
            log_path: log_path.into(),
        }
    }

    /// Record a tool execution that was **rejected** by the user or by a
    /// missing Confirm client.
    pub async fn log_rejected(&self, tool_call: &ToolCall) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
            trust_level: tool_call.trust_level,
            user_approved: false,
            result: AuditResult::Rejected,
            details: None,
        };
        self.append(&entry).await;
    }

    /// Record a confirmation that **timed out**.
    pub async fn log_timeout(&self, tool_call: &ToolCall) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
            trust_level: tool_call.trust_level,
            user_approved: false,
            result: AuditResult::Timeout,
            details: None,
        };
        self.append(&entry).await;
    }

    /// Record a tool execution that was **rate-limited**.
    pub async fn log_rate_limited(&self, tool_call: &ToolCall) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
            trust_level: tool_call.trust_level,
            user_approved: false,
            result: AuditResult::Error("rate limit exceeded".to_owned()),
            details: Some("Destructive action rate limit exceeded".to_owned()),
        };
        self.append(&entry).await;
    }

    /// Record a successful tool execution.
    pub async fn log_success(&self, tool_call: &ToolCall, result: &ToolExecResult) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
            trust_level: tool_call.trust_level,
            user_approved: true,
            result: if result.is_error {
                AuditResult::Error(result.output.clone())
            } else {
                AuditResult::Ok
            },
            details: Some(truncate_output(&result.output, 4096)),
        };
        self.append(&entry).await;
    }

    /// Record a tool whose execution produced an unrecoverable error.
    pub async fn log_error(&self, tool_call: &ToolCall, error: &str) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            action: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
            trust_level: tool_call.trust_level,
            user_approved: true,
            result: AuditResult::Error(error.to_owned()),
            details: None,
        };
        self.append(&entry).await;
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Serialise `entry` to JSON and append it as a single line.
    async fn append(&self, entry: &AuditEntry) {
        if let Err(e) = self.try_append(entry).await {
            tracing::error!("Failed to write audit log: {e:#}");
        }
    }

    async fn try_append(&self, entry: &AuditEntry) -> anyhow::Result<()> {
        if let Some(parent) = self.log_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await?;

        let json = serde_json::to_string(entry)?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        Ok(())
    }
}

/// Truncate tool output to at most `max_len` bytes (UTF-8 safe).
fn truncate_output(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_owned()
    } else {
        // Find a char boundary at or before `max_len`.
        let mut end = max_len;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        let mut truncated = s[..end].to_owned();
        truncated.push_str("...[truncated]");
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_output("hello", 100), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(5000);
        let result = truncate_output(&long, 4096);
        assert!(result.len() < 5000);
        assert!(result.ends_with("...[truncated]"));
    }

    #[test]
    fn truncate_multibyte_safe() {
        // 3-byte UTF-8 chars
        let s = "\u{2603}".repeat(2000); // snowman
        let result = truncate_output(&s, 100);
        // Must be valid UTF-8 and not panic
        assert!(result.len() <= 120); // 100 + "...[truncated]" len
    }
}
