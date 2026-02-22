mod audit;
mod config;
mod llm;
mod router;
mod server;
mod state;
mod tool_executor;

use std::sync::Arc;

use aios_common::IpcServer;
use anyhow::Result;
use tokio::sync::RwLock;

use crate::audit::AuditLogger;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_agent=info".into()),
        )
        .init();

    tracing::info!("aios-agent starting...");

    let config = config::load_config()?;
    tracing::info!(socket = %config.agent.socket_path, "Loaded configuration");

    let audit_logger = AuditLogger::new(&config.agent.audit_log);
    let max_destructive = config.agent.max_destructive_per_minute;

    // Create the LLM provider from config. If the API key is empty (and provider
    // is not Ollama, which doesn't need one), fall back to echo mode and warn.
    let needs_api_key = config.provider.provider_type != aios_common::ProviderType::Ollama;
    let state = if needs_api_key && config.provider.api_key.is_empty() {
        tracing::warn!(
            "No API key configured for {:?} provider -- running in echo mode",
            config.provider.provider_type,
        );
        Arc::new(RwLock::new(state::AgentState::new(
            audit_logger,
            max_destructive,
        )))
    } else {
        match llm::create_provider(&config.provider) {
            Ok(provider) => {
                tracing::info!(
                    provider = provider.name(),
                    "LLM provider initialized successfully",
                );
                Arc::new(RwLock::new(state::AgentState::with_provider(
                    provider,
                    audit_logger,
                    max_destructive,
                )))
            }
            Err(e) => {
                tracing::error!("Failed to initialize LLM provider: {e:#}");
                tracing::warn!("Falling back to echo mode");
                Arc::new(RwLock::new(state::AgentState::new(
                    audit_logger,
                    max_destructive,
                )))
            }
        }
    };

    let ipc_server = IpcServer::bind(&config.agent.socket_path)?;
    tracing::info!(path = %config.agent.socket_path, "IPC server bound");

    server::run_server(ipc_server, state).await?;

    Ok(())
}
