use serde::{Deserialize, Serialize};

/// Data provenance marker for the security model.
///
/// Every piece of data flowing through the system is tagged with a trust level
/// that determines what actions can be performed without explicit user confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Direct user input via UI.
    User,
    /// System data (files, configs).
    System,
    /// Content from web pages (untrusted).
    WebContent,
    /// Data retrieved from RAG memory.
    Memory,
}
