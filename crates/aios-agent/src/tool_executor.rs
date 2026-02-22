//! Tool execution pipeline with confirmation, rate limiting, and audit logging.
//!
//! This module bridges the LLM tool-call mechanism with the MCP tool registry.
//! When the LLM returns a `ToolUse` message the router delegates here to:
//!
//! 1. Look up the tool in the [`ToolRegistry`].
//! 2. Check whether user confirmation is required ([`TrustRequirement`]).
//! 3. Enforce rate limits for destructive actions.
//! 4. Send a `ConfirmRequest` to the connected Confirm client and wait.
//! 5. Execute the tool and return a [`ToolResult`].
//! 6. Log every step to the audit trail.

use std::sync::Arc;
use std::time::Duration;

use aios_common::{
    ClientType, IpcMessage, IpcPayload, ToolCall, ToolResult, TrustRequirement,
};
use aios_mcp::executor::ToolContext;
use aios_mcp::registry::ToolRegistry;
use tokio::sync::{oneshot, RwLock};
use uuid::Uuid;

use crate::audit::AuditLogger;
use crate::state::AgentState;

/// Timeout for waiting on user confirmation via the Confirm client.
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(60);

/// Execute a single tool call through the full pipeline:
/// lookup -> rate limit -> confirm -> execute -> audit.
pub async fn execute_tool_call(
    tool_call: &ToolCall,
    registry: &ToolRegistry,
    state: &Arc<RwLock<AgentState>>,
    audit_logger: &AuditLogger,
) -> ToolResult {
    // 1. Look up the tool.
    let Some(tool) = registry.get(&tool_call.name) else {
        tracing::warn!(tool = %tool_call.name, "Unknown tool requested");
        return ToolResult {
            call_id: tool_call.id,
            output: format!("Unknown tool: {}", tool_call.name),
            is_error: true,
        };
    };

    let trust_req = tool.trust_requirement();

    // 2. Rate-limit destructive actions.
    if trust_req == TrustRequirement::DoubleConfirm {
        let allowed = {
            let mut state_guard = state.write().await;
            state_guard.rate_limiter.check_and_record()
        };
        if !allowed {
            tracing::warn!(tool = %tool_call.name, "Destructive action rate limit exceeded");
            audit_logger.log_rate_limited(tool_call).await;
            return ToolResult {
                call_id: tool_call.id,
                output: "Rate limit exceeded for destructive actions. Please wait before retrying."
                    .to_owned(),
                is_error: true,
            };
        }
    }

    // 3. Request user confirmation if the trust requirement demands it.
    if trust_req != TrustRequirement::None {
        let definition = tool.definition();
        match request_confirmation(state, tool_call, &definition.description).await {
            ConfirmOutcome::Approved => {
                tracing::info!(tool = %tool_call.name, "Action approved by user");
            }
            ConfirmOutcome::Rejected => {
                tracing::info!(tool = %tool_call.name, "Action rejected by user");
                audit_logger.log_rejected(tool_call).await;
                return ToolResult {
                    call_id: tool_call.id,
                    output: "Action rejected by user".to_owned(),
                    is_error: true,
                };
            }
            ConfirmOutcome::Timeout => {
                tracing::warn!(tool = %tool_call.name, "Confirmation timed out");
                audit_logger.log_timeout(tool_call).await;
                return ToolResult {
                    call_id: tool_call.id,
                    output: "Confirmation timed out (60s)".to_owned(),
                    is_error: true,
                };
            }
            ConfirmOutcome::NoClient => {
                tracing::warn!(tool = %tool_call.name, "No confirm client connected");
                audit_logger.log_rejected(tool_call).await;
                return ToolResult {
                    call_id: tool_call.id,
                    output: "No confirmation client connected. Cannot execute this action."
                        .to_owned(),
                    is_error: true,
                };
            }
            ConfirmOutcome::SendFailed => {
                tracing::error!(tool = %tool_call.name, "Failed to send confirm request");
                audit_logger.log_error(tool_call, "IPC send failed").await;
                return ToolResult {
                    call_id: tool_call.id,
                    output: "Internal error: failed to contact confirmation client".to_owned(),
                    is_error: true,
                };
            }
        }
    }

    // 4. Execute the tool.
    let ctx = ToolContext {
        call_id: tool_call.id,
    };

    let result = match tool.execute(tool_call.arguments.clone(), &ctx).await {
        Ok(r) => r,
        Err(e) => {
            let error_msg = format!("Execution error: {e:#}");
            audit_logger.log_error(tool_call, &error_msg).await;
            return ToolResult {
                call_id: tool_call.id,
                output: error_msg,
                is_error: true,
            };
        }
    };

    // 5. Audit the result.
    audit_logger.log_success(tool_call, &result).await;
    result
}

// --------------------------------------------------------------------------
// Confirmation flow
// --------------------------------------------------------------------------

/// Possible outcomes of a confirmation request.
enum ConfirmOutcome {
    Approved,
    Rejected,
    Timeout,
    NoClient,
    SendFailed,
}

/// Send a `ConfirmRequest` to the connected Confirm client and wait for the
/// user's decision.  Returns the outcome.
async fn request_confirmation(
    state: &Arc<RwLock<AgentState>>,
    tool_call: &ToolCall,
    description: &str,
) -> ConfirmOutcome {
    let action_id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();

    // Register the pending confirmation before sending the IPC message so
    // that a fast response cannot arrive before the entry exists.
    {
        let mut state_guard = state.write().await;
        state_guard.pending_confirms.insert(action_id, tx);
    }

    // Build the IPC message.
    let confirm_msg = IpcMessage {
        id: Uuid::new_v4(),
        payload: IpcPayload::ConfirmRequest {
            action_id,
            action_type: tool_call.name.clone(),
            description: description.to_owned(),
            command: serde_json::to_string_pretty(&tool_call.arguments).unwrap_or_default(),
            trust_level: tool_call.trust_level,
        },
    };

    // Find the Confirm client and send.
    let send_ok = {
        let state_guard = state.read().await;
        if let Some(client) = state_guard.find_client(ClientType::Confirm) {
            match client.writer.lock().await.send(&confirm_msg).await {
                Ok(()) => true,
                Err(e) => {
                    tracing::error!("Failed to send confirm request via IPC: {e}");
                    false
                }
            }
        } else {
            // Clean up the pending entry since nobody will answer.
            drop(state_guard);
            let mut state_guard = state.write().await;
            state_guard.pending_confirms.remove(&action_id);
            return ConfirmOutcome::NoClient;
        }
    };

    if !send_ok {
        let mut state_guard = state.write().await;
        state_guard.pending_confirms.remove(&action_id);
        return ConfirmOutcome::SendFailed;
    }

    // Wait for the response with a timeout.
    match tokio::time::timeout(CONFIRM_TIMEOUT, rx).await {
        Ok(Ok(true)) => ConfirmOutcome::Approved,
        Ok(Ok(false)) => ConfirmOutcome::Rejected,
        Ok(Err(_)) => {
            // Channel dropped -- the confirm client disconnected.
            tracing::warn!("Confirm channel dropped before response");
            ConfirmOutcome::Rejected
        }
        Err(_) => {
            // Timeout -- clean up the pending entry.
            let mut state_guard = state.write().await;
            state_guard.pending_confirms.remove(&action_id);
            ConfirmOutcome::Timeout
        }
    }
}
