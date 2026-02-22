pub mod claude;
pub mod ollama;
pub mod openai;
pub mod system_prompt;
pub mod types;

use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;

use self::types::{LlmRequest, LlmResponse, StreamDelta};

/// Trait abstracting an LLM provider.
///
/// Implementors must be safe to share across threads (`Send + Sync`) since the
/// provider instance lives inside `AgentState` behind an `Arc<RwLock<_>>`.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Non-streaming completion. Sends a request and waits for the full
    /// response.
    async fn complete(&self, req: &LlmRequest) -> Result<LlmResponse>;

    /// Streaming completion. Returns a stream of incremental deltas.
    /// Not yet used -- will be wired in a later step.
    #[allow(dead_code)]
    async fn complete_stream(
        &self,
        req: &LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamDelta>> + Send>>>;

    /// Whether this provider supports tool / function calling.
    /// Not yet used -- will be wired in a later step.
    #[allow(dead_code)]
    fn supports_tools(&self) -> bool;

    /// Provider name for logging and diagnostics.
    fn name(&self) -> &str;
}

/// Factory function: create a boxed `LlmProvider` from the shared config.
pub fn create_provider(config: &aios_common::ProviderConfig) -> Result<Box<dyn LlmProvider>> {
    match config.provider_type {
        aios_common::ProviderType::OpenAi => {
            Ok(Box::new(openai::OpenAiProvider::new(config)?))
        }
        aios_common::ProviderType::Claude => {
            Ok(Box::new(claude::ClaudeProvider::new(config)?))
        }
        aios_common::ProviderType::Ollama => {
            Ok(Box::new(ollama::OllamaProvider::new(config)?))
        }
    }
}
