use std::pin::Pin;

use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;

use aios_common::ProviderConfig;

use super::types::{LlmRequest, LlmResponse, StreamDelta};
use super::LlmProvider;

/// Stub provider for Ollama (not yet implemented).
pub struct OllamaProvider;

impl OllamaProvider {
    /// Create a new Ollama provider from configuration.
    ///
    /// Currently a no-op since Ollama support is not implemented.
    pub fn new(_config: &ProviderConfig) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, _req: &LlmRequest) -> Result<LlmResponse> {
        anyhow::bail!("Ollama provider not yet implemented")
    }

    async fn complete_stream(
        &self,
        _req: &LlmRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamDelta>> + Send>>> {
        anyhow::bail!("Ollama provider not yet implemented")
    }

    fn supports_tools(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
