//! Tool execution trait and context.

use aios_common::{ToolDefinition, ToolResult, TrustRequirement};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

/// Context passed to every tool invocation.
///
/// Carries the call identifier and will be extended in Step 3.3
/// with an audit logger and confirmation channel.
pub struct ToolContext {
    /// Unique identifier of the tool call this execution belongs to.
    pub call_id: Uuid,
    // TODO: Step 3.3 - audit logger, confirm channel
}

/// Trait that all tools must implement.
///
/// Each tool declares its JSON Schema definition, required trust level,
/// and an async `execute` method that performs the actual work.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the JSON Schema definition of this tool, including its name,
    /// description, parameter schema, and trust requirement.
    fn definition(&self) -> ToolDefinition;

    /// Returns the confirmation level required before this tool can execute.
    fn trust_requirement(&self) -> TrustRequirement;

    /// Execute the tool with the given arguments.
    ///
    /// Implementations must **never panic**. All errors are returned as
    /// [`ToolResult`] with `is_error: true` or via the `Result` wrapper
    /// for truly unrecoverable situations.
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult>;
}
