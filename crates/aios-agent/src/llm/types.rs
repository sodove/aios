use aios_common::{ChatMessage, ToolDefinition};

/// Request to an LLM provider.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// Conversation messages to send to the model.
    pub messages: Vec<ChatMessage>,
    /// Tool definitions available to the model.
    pub tools: Vec<ToolDefinition>,
    /// System prompt to prepend.
    pub system_prompt: String,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Sampling temperature (0.0 -- 2.0).
    pub temperature: f32,
}

/// Non-streaming response from an LLM provider.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// The assistant message produced by the model.
    pub message: ChatMessage,
    /// Whether the response contains tool calls (used in later steps).
    #[allow(dead_code)]
    pub has_tool_calls: bool,
}

/// A single chunk from a streaming response (used in later steps).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StreamDelta {
    /// Incremental text content.
    pub delta: String,
    /// Whether this is the final chunk.
    pub done: bool,
}
