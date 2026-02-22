use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::tool::{ToolCall, ToolResult};
use super::trust::TrustLevel;

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub role: Role,
    pub content: MessageContent,
    pub trust_level: TrustLevel,
    pub timestamp: DateTime<Utc>,
}

/// The role of a message author within a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

/// Typed content of a chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { text: String },
    ToolUse { tool_calls: Vec<ToolCall> },
    ToolResult { results: Vec<ToolResult> },
}
