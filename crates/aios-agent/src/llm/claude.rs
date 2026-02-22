use std::num::NonZeroU16;
use std::pin::Pin;

use aios_common::{
    ChatMessage, MessageContent, ProviderConfig, Role as AiosRole, TrustLevel,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::Stream;
use misanthropic::prompt::message::{Content, Role as ClaudeRole};
use misanthropic::{Client, Prompt};
use uuid::Uuid;

use super::types::{LlmRequest, LlmResponse, StreamDelta};
use super::LlmProvider;

/// Claude provider backed by the `misanthropic` crate.
pub struct ClaudeProvider {
    client: Client,
    model: String,
}

impl ClaudeProvider {
    /// Create a new Claude provider from the shared configuration.
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let client = Client::new(config.api_key.clone())
            .map_err(|e| anyhow::anyhow!("Invalid Anthropic API key: {e}"))?;

        Ok(Self {
            client,
            model: config.model.clone(),
        })
    }

    /// Convert our `ChatMessage` to misanthropic's `prompt::Message`.
    fn convert_message(msg: &ChatMessage) -> Option<misanthropic::prompt::Message<'static>> {
        let role = match msg.role {
            AiosRole::User | AiosRole::Tool => ClaudeRole::User,
            AiosRole::Assistant => ClaudeRole::Assistant,
            // System messages go into the system prompt, not the message list.
            AiosRole::System => return None,
        };

        let text = extract_text(&msg.content);

        Some(misanthropic::prompt::Message {
            role,
            content: Content::text(text),
        })
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, req: &LlmRequest) -> Result<LlmResponse> {
        // Build messages, filtering out system messages (they go in .system()).
        let messages: Vec<misanthropic::prompt::Message<'static>> = req
            .messages
            .iter()
            .filter_map(Self::convert_message)
            .collect();

        // Determine max_tokens (clamped to NonZeroU16 range).
        let max_tokens = NonZeroU16::new(
            u16::try_from(req.max_tokens.min(u32::from(u16::MAX)))
                .unwrap_or(4096),
        )
        .unwrap_or(NonZeroU16::new(4096).expect("4096 is nonzero"));

        // Build the prompt using the model string directly via serde.
        // The misanthropic crate only has a fixed enum of models, so we
        // serialize our model string and attempt to deserialize it into
        // their Model enum. If that fails, fall back to the default model.
        let model: misanthropic::Model =
            serde_json::from_value(serde_json::Value::String(self.model.clone()))
                .unwrap_or_default();

        let prompt = Prompt::default()
            .model(model)
            .system(req.system_prompt.as_str())
            .messages(messages)
            .max_tokens(max_tokens)
            .temperature(Some(req.temperature));

        let response = self
            .client
            .message(&prompt)
            .await
            .map_err(|e| anyhow::anyhow!("Claude API error: {e}"))
            .context("Claude message request failed")?;

        // Extract text from the response message.
        let text = response.message.content.to_string();
        let has_tool_calls = response
            .stop_reason
            .as_ref()
            .is_some_and(|r| matches!(r, misanthropic::response::StopReason::ToolUse));

        let chat_message = ChatMessage {
            id: Uuid::new_v4(),
            role: AiosRole::Assistant,
            content: MessageContent::Text { text },
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
        // TODO: Implement streaming via client.stream().
        // For now, return an error indicating streaming is not yet supported.
        anyhow::bail!("Claude streaming is not yet implemented; use complete() instead")
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "claude"
    }
}

/// Extract plain text from a `MessageContent` value.
fn extract_text(content: &MessageContent) -> String {
    match content {
        MessageContent::Text { text } => text.clone(),
        MessageContent::ToolUse { tool_calls } => {
            serde_json::to_string(tool_calls).unwrap_or_default()
        }
        MessageContent::ToolResult { results } => results
            .iter()
            .map(|r| r.output.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
