use std::pin::Pin;

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

use aios_common::{ChatMessage, MessageContent, ProviderConfig, Role};

use super::types::{LlmRequest, LlmResponse, StreamDelta};
use super::LlmProvider;

/// Ollama provider — talks to a local Ollama instance via its HTTP API.
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

/// A single message in the Ollama chat API format.
#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Request body for `POST /api/chat`.
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// Response from `POST /api/chat` (non-streaming).
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

impl OllamaProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let base_url = if config.base_url.is_empty() {
            "http://localhost:11434".to_owned()
        } else {
            config.base_url.trim_end_matches('/').to_owned()
        };

        let model = if config.model.is_empty() {
            "llama3.2".to_owned()
        } else {
            config.model.clone()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client for Ollama")?;

        tracing::info!(base_url = %base_url, model = %model, "Ollama provider initialized");

        Ok(Self {
            base_url,
            model,
            client,
        })
    }

    /// Convert internal ChatMessage to Ollama API format.
    fn convert_messages(system_prompt: &str, messages: &[ChatMessage]) -> Vec<OllamaMessage> {
        let mut out = Vec::new();

        // System prompt as first message
        if !system_prompt.is_empty() {
            out.push(OllamaMessage {
                role: "system".to_owned(),
                content: system_prompt.to_owned(),
            });
        }

        for msg in messages {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => "system",
                Role::Tool => "user", // Ollama doesn't have tool role; map to user
            };

            let content = match &msg.content {
                MessageContent::Text { text } => text.clone(),
                MessageContent::ToolUse { tool_calls } => {
                    serde_json::to_string(tool_calls).unwrap_or_default()
                }
                MessageContent::ToolResult { results } => {
                    serde_json::to_string(results).unwrap_or_default()
                }
            };

            if !content.is_empty() {
                out.push(OllamaMessage {
                    role: role.to_owned(),
                    content,
                });
            }
        }

        out
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, req: &LlmRequest) -> Result<LlmResponse> {
        let messages = Self::convert_messages(&req.system_prompt, &req.messages);

        let body = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(req.temperature),
                num_predict: if req.max_tokens > 0 {
                    Some(req.max_tokens)
                } else {
                    None
                },
            }),
        };

        let url = format!("{}/api/chat", self.base_url);

        tracing::debug!(url = %url, model = %self.model, "Sending request to Ollama");

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to connect to Ollama — is it running?")?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama returned {status}: {body_text}");
        }

        let chat_resp: OllamaChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        let message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            role: Role::Assistant,
            content: MessageContent::Text {
                text: chat_resp.message.content,
            },
            trust_level: aios_common::TrustLevel::Trusted,
            timestamp: chrono::Utc::now(),
        };

        Ok(LlmResponse {
            message,
            has_tool_calls: false,
        })
    }

    async fn complete_stream(
        &self,
        _req: &LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamDelta>> + Send>>> {
        anyhow::bail!("Ollama streaming not yet implemented")
    }

    fn supports_tools(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
