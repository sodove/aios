use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::trust::TrustLevel;

/// A request to invoke a specific tool with arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: Uuid,
    pub name: String,
    pub arguments: serde_json::Value,
    pub trust_level: TrustLevel,
}

/// The result of a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: Uuid,
    pub output: String,
    pub is_error: bool,
}

/// Required confirmation level for tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustRequirement {
    /// Read-only operation, no confirmation needed.
    None,
    /// Modifying operation, single confirmation required.
    Confirm,
    /// Destructive operation, double confirmation required.
    DoubleConfirm,
}

/// Declares a tool that the agent can invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    /// JSON Schema describing the tool's parameters.
    pub parameters: serde_json::Value,
    pub trust_requirement: TrustRequirement,
}
