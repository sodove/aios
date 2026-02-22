use std::io;

/// Central error type for the AIOS project.
///
/// Covers IPC communication, LLM provider interactions, tool execution,
/// configuration parsing, and user-facing confirmation flows.
#[derive(Debug, thiserror::Error)]
pub enum AiosError {
    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("IPC connection closed")]
    ConnectionClosed,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("LLM provider error: {0}")]
    Provider(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Confirmation timeout")]
    ConfirmTimeout,

    #[error("Action rejected by user")]
    ActionRejected,

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
