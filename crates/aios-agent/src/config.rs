use std::path::PathBuf;

use aios_common::AiosConfig;
use anyhow::{Context, Result};

/// Returns the default config path: `~/.config/aios/agent.toml`.
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("aios")
        .join("agent.toml")
}

/// Load config from TOML file, or return default if not found.
pub fn load_config() -> Result<AiosConfig> {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config from {}", path.display()))?;
        let config: AiosConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse config from {}", path.display()))?;
        Ok(config)
    } else {
        tracing::warn!("Config not found at {}, using defaults", path.display());
        Ok(AiosConfig::default())
    }
}

/// Save config to TOML file, creating parent directories as needed.
#[allow(dead_code)]
pub fn save_config(config: &AiosConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}
