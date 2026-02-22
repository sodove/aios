pub mod audit;
pub mod error;
pub mod ipc;
pub mod types;

pub use audit::{AuditEntry, AuditResult};
pub use error::AiosError;
pub use ipc::{ClientType, IpcClient, IpcConnection, IpcMessage, IpcPayload, IpcServer};
pub use types::config::{AgentConfig, AiosConfig, ProviderConfig, ProviderType};
pub use types::message::{ChatMessage, MessageContent, Role};
pub use types::tool::{ToolCall, ToolDefinition, ToolResult, TrustRequirement};
pub use types::trust::TrustLevel;
