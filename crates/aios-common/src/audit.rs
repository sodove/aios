use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::trust::TrustLevel;

/// An immutable record of an agent action for the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub arguments: serde_json::Value,
    pub trust_level: TrustLevel,
    pub user_approved: bool,
    pub result: AuditResult,
    pub details: Option<String>,
}

/// Outcome of an audited action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResult {
    Ok,
    Error(String),
    Rejected,
    Timeout,
}
