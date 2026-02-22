use chrono::{DateTime, Utc};
use iced::widget::markdown;
use uuid::Uuid;

/// Maximum characters to display for tool result output before truncation.
const TOOL_OUTPUT_MAX_LEN: usize = 500;

/// A single message prepared for display in the chat UI.
#[derive(Debug)]
pub struct DisplayMessage {
    /// Unique identifier; used for keyed rendering and IPC correlation.
    pub id: Uuid,
    pub role: MessageRole,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    /// Pre-parsed markdown content for assistant messages.
    /// Stored to avoid re-parsing on every frame.
    pub markdown_content: Option<markdown::Content>,
    /// Tool name, present for `ToolCall` and `ToolResult` roles.
    pub tool_name: Option<String>,
    /// Pretty-printed JSON arguments for tool calls.
    pub tool_args: Option<String>,
    /// Whether the tool result was an error. Preserved for use in extended
    /// tool card views (e.g. collapsed vs expanded error details).
    #[allow(dead_code)]
    pub tool_is_error: Option<bool>,
    /// Current status of a tool interaction card.
    pub tool_status: Option<ToolStatus>,
}

impl DisplayMessage {
    /// Creates a new user message (no markdown parsing).
    pub fn user(id: Uuid, text: String, timestamp: DateTime<Utc>) -> Self {
        Self {
            id,
            role: MessageRole::User,
            text,
            timestamp,
            markdown_content: None,
            tool_name: None,
            tool_args: None,
            tool_is_error: None,
            tool_status: None,
        }
    }

    /// Creates a new assistant message with pre-parsed markdown.
    pub fn assistant(id: Uuid, text: String, timestamp: DateTime<Utc>) -> Self {
        let markdown_content = Some(markdown::Content::parse(&text));
        Self {
            id,
            role: MessageRole::Assistant,
            text,
            timestamp,
            markdown_content,
            tool_name: None,
            tool_args: None,
            tool_is_error: None,
            tool_status: None,
        }
    }

    /// Creates a tool call card in `Pending` state.
    pub fn tool_call(
        id: Uuid,
        name: String,
        args_json: String,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            role: MessageRole::ToolCall,
            text: String::new(),
            timestamp,
            markdown_content: None,
            tool_name: Some(name),
            tool_args: Some(args_json),
            tool_is_error: None,
            tool_status: Some(ToolStatus::Pending),
        }
    }

    /// Creates a tool result card with `Completed` or `Failed` status.
    ///
    /// Long output is truncated to [`TOOL_OUTPUT_MAX_LEN`] characters.
    pub fn tool_result(
        id: Uuid,
        name: String,
        output: String,
        is_error: bool,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let truncated = truncate_output(&output);
        let status = if is_error {
            ToolStatus::Failed
        } else {
            ToolStatus::Completed
        };
        Self {
            id,
            role: MessageRole::ToolResult,
            text: truncated,
            timestamp,
            markdown_content: None,
            tool_name: Some(name),
            tool_args: None,
            tool_is_error: Some(is_error),
            tool_status: Some(status),
        }
    }

    /// Update the text of a message (used during streaming) and re-parse
    /// markdown content for assistant messages.
    pub fn update_text(&mut self, new_text: String) {
        self.text = new_text;
        if self.role == MessageRole::Assistant {
            self.markdown_content = Some(markdown::Content::parse(&self.text));
        }
    }

    /// Mark a tool call card as having received its result.
    pub fn set_tool_status(&mut self, status: ToolStatus) {
        self.tool_status = Some(status);
    }
}

/// Truncate tool output to [`TOOL_OUTPUT_MAX_LEN`] characters, appending an
/// ellipsis marker when truncation occurs.
fn truncate_output(output: &str) -> String {
    if output.len() <= TOOL_OUTPUT_MAX_LEN {
        output.to_owned()
    } else {
        let mut truncated = output[..TOOL_OUTPUT_MAX_LEN].to_owned();
        truncated.push_str("... (truncated)");
        truncated
    }
}

/// The author role of a displayed message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    /// A tool invocation request from the agent.
    ToolCall,
    /// A tool execution result returned to the agent.
    ToolResult,
}

/// Lifecycle status of a tool interaction card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool call sent, waiting for result.
    Pending,
    /// Result received, execution succeeded.
    Completed,
    /// Result received, execution failed.
    Failed,
    /// User rejected the tool call in the confirm dialog.
    Rejected,
}

/// Current connection status to the AIOS agent backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

impl ConnectionStatus {
    /// Returns a human-readable label for the status indicator.
    pub fn label(self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
        }
    }
}
