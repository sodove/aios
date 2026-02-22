use std::sync::Arc;

use aios_common::{
    ChatMessage, IpcMessage, IpcPayload, MessageContent, Role, ToolResult, TrustLevel,
};
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::llm::system_prompt::default_system_prompt;
use crate::llm::types::LlmRequest;
use crate::state::{AgentState, Conversation};
use crate::tool_executor;

/// Default maximum tokens for LLM responses.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Default sampling temperature.
const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Maximum number of tool-call round-trips before the agent forces a text
/// response.  This prevents infinite loops when the LLM keeps requesting
/// tools without ever producing a final answer.
const MAX_TOOL_ITERATIONS: u32 = 10;

/// Route an incoming IPC message and optionally produce a response.
pub async fn route_message(
    msg: IpcMessage,
    _client_id: Uuid,
    state: &Arc<RwLock<AgentState>>,
) -> Option<IpcMessage> {
    match msg.payload {
        IpcPayload::Register { client_type } => {
            tracing::info!(?client_type, "Client registered via router");
            // Registration is already handled in server.rs before routing,
            // but we still return an ack for safety.
            Some(IpcMessage {
                id: Uuid::new_v4(),
                payload: IpcPayload::RegisterAck { success: true },
            })
        }

        IpcPayload::ChatRequest {
            message,
            conversation_id,
        } => {
            tracing::info!(%conversation_id, "Chat request received");

            // Store the user message in the conversation.
            let user_msg = ChatMessage {
                id: Uuid::new_v4(),
                role: Role::User,
                content: MessageContent::Text {
                    text: message.clone(),
                },
                trust_level: TrustLevel::User,
                timestamp: Utc::now(),
            };

            {
                let mut state_guard = state.write().await;
                let conversation = state_guard
                    .conversations
                    .entry(conversation_id)
                    .or_insert_with(|| Conversation {
                        id: conversation_id,
                        messages: Vec::new(),
                    });
                conversation.messages.push(user_msg);
            }

            // Run the agentic loop: LLM call -> tool execution -> repeat.
            let assistant_msg = agentic_loop(state, conversation_id, &message).await;

            // Store the final assistant message.
            {
                let mut state_guard = state.write().await;
                if let Some(conversation) = state_guard.conversations.get_mut(&conversation_id) {
                    conversation.messages.push(assistant_msg.clone());
                }
            }

            Some(IpcMessage {
                id: Uuid::new_v4(),
                payload: IpcPayload::ChatResponse {
                    message: assistant_msg,
                },
            })
        }

        IpcPayload::ConfirmResponse {
            action_id,
            approved,
            ..
        } => {
            tracing::info!(%action_id, %approved, "Confirm response received");
            let mut state_guard = state.write().await;
            if let Some(sender) = state_guard.pending_confirms.remove(&action_id) {
                if sender.send(approved).is_err() {
                    tracing::warn!(
                        %action_id,
                        "Confirm response arrived but the waiting task was already gone"
                    );
                }
            } else {
                tracing::warn!(%action_id, "No pending confirmation found for this action_id");
            }
            None
        }

        IpcPayload::Ping => Some(IpcMessage {
            id: Uuid::new_v4(),
            payload: IpcPayload::Pong,
        }),

        other => {
            tracing::warn!(?other, "Unhandled message type");
            None
        }
    }
}

// --------------------------------------------------------------------------
// Agentic loop
// --------------------------------------------------------------------------

/// Run the agentic loop: call the LLM, execute any requested tools, feed the
/// results back, and repeat until the LLM produces a text response or the
/// iteration limit is reached.
async fn agentic_loop(
    state: &Arc<RwLock<AgentState>>,
    conversation_id: Uuid,
    raw_message: &str,
) -> ChatMessage {
    // Check if there is an LLM provider at all.
    let has_provider = {
        let state_guard = state.read().await;
        state_guard.llm_provider.is_some()
    };

    if !has_provider {
        tracing::debug!("No LLM provider configured, using echo mode");
        return echo_response(raw_message);
    }

    for iteration in 0..MAX_TOOL_ITERATIONS {
        let llm_response = call_llm(state, conversation_id).await;

        let response_msg = match llm_response {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("LLM request failed: {e:#}");
                return ChatMessage {
                    id: Uuid::new_v4(),
                    role: Role::Assistant,
                    content: MessageContent::Text {
                        text: format!("Sorry, I encountered an error: {e}"),
                    },
                    trust_level: TrustLevel::System,
                    timestamp: Utc::now(),
                };
            }
        };

        // If the LLM returned text, we are done.
        if matches!(&response_msg.content, MessageContent::Text { .. }) {
            return response_msg;
        }

        // The LLM returned tool calls -- execute them.
        let tool_calls = match &response_msg.content {
            MessageContent::ToolUse { tool_calls } => tool_calls.clone(),
            _ => return response_msg, // Unexpected variant; return as-is.
        };

        tracing::info!(
            iteration,
            count = tool_calls.len(),
            "LLM requested tool calls"
        );

        // Store the assistant tool-use message in the conversation.
        {
            let mut state_guard = state.write().await;
            if let Some(conv) = state_guard.conversations.get_mut(&conversation_id) {
                conv.messages.push(response_msg);
            }
        }

        // Execute each tool call and collect results.
        let mut results: Vec<ToolResult> = Vec::with_capacity(tool_calls.len());
        for tc in &tool_calls {
            // We need to read registry and audit_logger from state for each call.
            // To avoid holding the lock across an async tool execution, we clone
            // the registry reference pattern -- but ToolRegistry is not Clone.
            // Instead, we pass the full state Arc and let execute_tool_call
            // acquire the lock internally.
            let result = {
                let state_guard = state.read().await;
                let registry = &state_guard.tool_registry;
                let audit_logger = &state_guard.audit_logger;
                tool_executor::execute_tool_call(tc, registry, state, audit_logger).await
            };
            results.push(result);
        }

        // Build a tool-result message and push it into the conversation.
        let tool_result_msg = ChatMessage {
            id: Uuid::new_v4(),
            role: Role::Tool,
            content: MessageContent::ToolResult { results },
            trust_level: TrustLevel::System,
            timestamp: Utc::now(),
        };

        {
            let mut state_guard = state.write().await;
            if let Some(conv) = state_guard.conversations.get_mut(&conversation_id) {
                conv.messages.push(tool_result_msg);
            }
        }

        // Continue the loop -- the next LLM call will include the tool results.
    }

    // Iteration limit reached.  Force a text response.
    tracing::warn!("Agentic loop reached {MAX_TOOL_ITERATIONS} iterations, forcing text response");
    force_text_response(state, conversation_id).await
}

/// Call the LLM with the current conversation history and tool definitions.
async fn call_llm(
    state: &Arc<RwLock<AgentState>>,
    conversation_id: Uuid,
) -> anyhow::Result<ChatMessage> {
    let (history, tool_defs) = {
        let state_guard = state.read().await;
        let history = state_guard
            .conversations
            .get(&conversation_id)
            .map(|c| c.messages.clone())
            .unwrap_or_default();
        let tool_defs = state_guard.tool_registry.definitions();
        (history, tool_defs)
    };

    let llm_request = LlmRequest {
        messages: history,
        tools: tool_defs,
        system_prompt: default_system_prompt(),
        max_tokens: DEFAULT_MAX_TOKENS,
        temperature: DEFAULT_TEMPERATURE,
    };

    let state_guard = state.read().await;
    let provider = state_guard
        .llm_provider
        .as_ref()
        .expect("LLM provider must exist when agentic_loop runs");
    let response = provider.complete(&llm_request).await?;
    Ok(response.message)
}

/// Ask the LLM one more time but without tools, forcing a text answer.
async fn force_text_response(
    state: &Arc<RwLock<AgentState>>,
    conversation_id: Uuid,
) -> ChatMessage {
    let history = {
        let state_guard = state.read().await;
        state_guard
            .conversations
            .get(&conversation_id)
            .map(|c| c.messages.clone())
            .unwrap_or_default()
    };

    let llm_request = LlmRequest {
        messages: history,
        tools: Vec::new(), // No tools -> LLM must respond with text.
        system_prompt: default_system_prompt(),
        max_tokens: DEFAULT_MAX_TOKENS,
        temperature: DEFAULT_TEMPERATURE,
    };

    let result = {
        let state_guard = state.read().await;
        if let Some(provider) = &state_guard.llm_provider {
            provider.complete(&llm_request).await
        } else {
            return echo_response("(iteration limit reached)");
        }
    };

    match result {
        Ok(response) => response.message,
        Err(e) => {
            tracing::error!("Force-text LLM call failed: {e:#}");
            ChatMessage {
                id: Uuid::new_v4(),
                role: Role::Assistant,
                content: MessageContent::Text {
                    text: format!("Sorry, I encountered an error: {e}"),
                },
                trust_level: TrustLevel::System,
                timestamp: Utc::now(),
            }
        }
    }
}

/// Produce a simple echo response (fallback when no LLM provider is configured).
fn echo_response(message: &str) -> ChatMessage {
    ChatMessage {
        id: Uuid::new_v4(),
        role: Role::Assistant,
        content: MessageContent::Text {
            text: format!("Echo: {message}"),
        },
        trust_level: TrustLevel::System,
        timestamp: Utc::now(),
    }
}
