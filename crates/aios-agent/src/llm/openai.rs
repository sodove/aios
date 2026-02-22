use std::pin::Pin;

use aios_common::{
    ChatMessage, MessageContent, ProviderConfig, Role as AiosRole, TrustLevel,
};
use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestToolMessage,
        ChatCompletionRequestUserMessage, ChatCompletionTool, ChatCompletionTools,
        CreateChatCompletionRequest, FunctionObject,
    },
    Client,
};
use async_trait::async_trait;
use chrono::Utc;
use futures::Stream;
use uuid::Uuid;

use super::types::{LlmRequest, LlmResponse, StreamDelta};
use super::LlmProvider;

/// OpenAI provider backed by the `async-openai` crate.
pub struct OpenAiProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider from the shared configuration.
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let mut openai_config = OpenAIConfig::new().with_api_key(&config.api_key);

        if let Some(base_url) = &config.base_url {
            openai_config = openai_config.with_api_base(base_url);
        }

        let client = Client::with_config(openai_config);

        Ok(Self {
            client,
            model: config.model.clone(),
        })
    }

    /// Convert our `ChatMessage` to async-openai's `ChatCompletionRequestMessage`.
    fn convert_message(msg: &ChatMessage) -> Option<ChatCompletionRequestMessage> {
        match msg.role {
            AiosRole::System => {
                let text = extract_text(&msg.content);
                Some(ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage::from(text.as_str()),
                ))
            }
            AiosRole::User => {
                let text = extract_text(&msg.content);
                Some(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage::from(text.as_str()),
                ))
            }
            AiosRole::Assistant => {
                let text = extract_text(&msg.content);
                Some(ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage::from(text.as_str()),
                ))
            }
            AiosRole::Tool => {
                // Tool results need a tool_call_id. For now, use the message id
                // as a best-effort mapping.
                let text = extract_text(&msg.content);
                Some(ChatCompletionRequestMessage::Tool(
                    ChatCompletionRequestToolMessage {
                        content: text.into(),
                        tool_call_id: msg.id.to_string(),
                    },
                ))
            }
        }
    }

    /// Convert our `ToolDefinition` to async-openai's `ChatCompletionTools`.
    fn convert_tool(tool: &aios_common::ToolDefinition) -> ChatCompletionTools {
        ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObject {
                name: tool.name.clone(),
                description: Some(tool.description.clone()),
                parameters: Some(tool.parameters.clone()),
                strict: None,
            },
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, req: &LlmRequest) -> Result<LlmResponse> {
        // Build message list: system prompt first, then conversation history.
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::with_capacity(
            req.messages.len() + 1,
        );

        messages.push(ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage::from(req.system_prompt.as_str()),
        ));

        for msg in &req.messages {
            if let Some(converted) = Self::convert_message(msg) {
                messages.push(converted);
            }
        }

        // Build tool definitions.
        let tools: Option<Vec<ChatCompletionTools>> = if req.tools.is_empty() {
            None
        } else {
            Some(req.tools.iter().map(Self::convert_tool).collect())
        };

        #[allow(deprecated)]
        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_completion_tokens: Some(req.max_tokens),
            temperature: Some(req.temperature),
            tools,
            ..Default::default()
        };

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("OpenAI chat completion request failed")?;

        // Extract the first choice.
        let choice = response
            .choices
            .into_iter()
            .next()
            .context("OpenAI returned no choices")?;

        let response_msg = choice.message;
        let has_tool_calls = response_msg.tool_calls.is_some();

        let content_text = response_msg.content.unwrap_or_default();

        let chat_message = ChatMessage {
            id: Uuid::new_v4(),
            role: AiosRole::Assistant,
            content: MessageContent::Text { text: content_text },
            trust_level: TrustLevel::System,
            timestamp: Utc::now(),
        };

        Ok(LlmResponse {
            message: chat_message,
            has_tool_calls,
        })
    }

    async fn complete_stream(
        &self,
        _req: &LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamDelta>> + Send>>> {
        // TODO: Implement streaming via client.chat().create_stream().
        // For now, return an error indicating streaming is not yet supported.
        anyhow::bail!(
            "OpenAI streaming is not yet implemented; use complete() instead"
        )
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "openai"
    }
}

/// Extract plain text from a `MessageContent` value.
fn extract_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text { text } => text.clone(),
        MessageContent::ToolUse { tool_calls } => {
            // Serialize tool calls as JSON for context.
            serde_json::to_string(tool_calls).unwrap_or_default()
        }
        MessageContent::ToolResult { results } => {
            // Concatenate tool outputs.
            results
                .iter()
                .map(|r| r.output.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}
