use serde::{Deserialize, Serialize};

/// Top-level AIOS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiosConfig {
    pub provider: ProviderConfig,
    pub agent: AgentConfig,
}

/// LLM provider connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
}

/// Supported LLM provider backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    OpenAi,
    Claude,
    Ollama,
}

/// Agent runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub socket_path: String,
    pub audit_log: String,
    pub max_destructive_per_minute: u32,
}

impl Default for AiosConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig {
                provider_type: ProviderType::Ollama,
                api_key: String::new(),
                model: "llama3.2".to_string(),
                base_url: Some("http://localhost:11434".to_string()),
            },
            agent: AgentConfig {
                socket_path: format!("/run/user/{}/aios-agent.sock", 1000),
                audit_log: "/var/log/aios/actions.log".to_string(),
                max_destructive_per_minute: 3,
            },
        }
    }
}
