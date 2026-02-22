use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use aios_common::ipc::IpcWriter;
use aios_common::{ChatMessage, ClientType};
use aios_mcp::registry::ToolRegistry;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

use crate::audit::AuditLogger;
use crate::llm::LlmProvider;

/// A registered client with its IPC writer half.
pub struct ConnectedClient {
    #[allow(dead_code)]
    pub client_type: ClientType,
    pub writer: Mutex<IpcWriter>,
}

/// A conversation with accumulated message history.
pub struct Conversation {
    #[allow(dead_code)]
    pub id: Uuid,
    pub messages: Vec<ChatMessage>,
}

/// Sliding-window rate limiter for destructive tool actions.
///
/// Tracks timestamps of recent destructive executions and rejects new ones
/// when the configured per-minute threshold is reached.
pub struct RateLimiter {
    /// Timestamps of recent destructive actions (within the last 60 s).
    window: VecDeque<Instant>,
    /// Maximum allowed destructive actions per 60-second window.
    max_per_minute: u32,
}

impl RateLimiter {
    /// Create a rate limiter with the given per-minute cap.
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            window: VecDeque::new(),
            max_per_minute,
        }
    }

    /// Try to record a new destructive action.
    ///
    /// Returns `true` if the action is allowed, `false` if the rate limit
    /// has been reached.  When allowed, the current timestamp is pushed into
    /// the sliding window.
    pub fn check_and_record(&mut self) -> bool {
        let now = Instant::now();
        // `Instant` is guaranteed to be at least 60 s after epoch in practice,
        // but `checked_sub` avoids a pedantic clippy lint.
        let one_minute_ago = now
            .checked_sub(std::time::Duration::from_secs(60))
            .unwrap_or(now);

        // Evict entries older than 60 s.
        while self
            .window
            .front()
            .is_some_and(|&ts| ts < one_minute_ago)
        {
            self.window.pop_front();
        }

        #[allow(clippy::cast_possible_truncation)] // window len is capped by max_per_minute (u32)
        let current_count = self.window.len() as u32;
        if current_count >= self.max_per_minute {
            return false;
        }

        self.window.push_back(now);
        true
    }
}

/// Central mutable state of the agent process.
pub struct AgentState {
    pub clients: HashMap<Uuid, ConnectedClient>,
    pub conversations: HashMap<Uuid, Conversation>,
    /// The active LLM provider. `None` when no valid API key is configured,
    /// in which case the agent falls back to echo mode.
    pub llm_provider: Option<Box<dyn LlmProvider>>,
    /// Registry of all available MCP tools.
    pub tool_registry: ToolRegistry,
    /// Pending confirmation requests awaiting a `ConfirmResponse`.
    /// Maps `action_id` to a one-shot sender that resolves the waiting
    /// `execute_tool_call` future.
    pub pending_confirms: HashMap<Uuid, oneshot::Sender<bool>>,
    /// Rate limiter for destructive tool actions.
    pub rate_limiter: RateLimiter,
    /// Audit logger shared across all tool executions.
    pub audit_logger: AuditLogger,
}

impl AgentState {
    /// Create a new agent state with no LLM provider (echo mode).
    pub fn new(audit_logger: AuditLogger, max_destructive_per_minute: u32) -> Self {
        Self {
            clients: HashMap::new(),
            conversations: HashMap::new(),
            llm_provider: None,
            tool_registry: ToolRegistry::with_defaults(),
            pending_confirms: HashMap::new(),
            rate_limiter: RateLimiter::new(max_destructive_per_minute),
            audit_logger,
        }
    }

    /// Create a new agent state with the given LLM provider.
    pub fn with_provider(
        provider: Box<dyn LlmProvider>,
        audit_logger: AuditLogger,
        max_destructive_per_minute: u32,
    ) -> Self {
        Self {
            clients: HashMap::new(),
            conversations: HashMap::new(),
            llm_provider: Some(provider),
            tool_registry: ToolRegistry::with_defaults(),
            pending_confirms: HashMap::new(),
            rate_limiter: RateLimiter::new(max_destructive_per_minute),
            audit_logger,
        }
    }

    /// Find the first connected client matching a given type.
    pub fn find_client(&self, client_type: ClientType) -> Option<&ConnectedClient> {
        self.clients.values().find(|c| c.client_type == client_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limiter_allows_within_limit() {
        let mut rl = RateLimiter::new(3);
        assert!(rl.check_and_record());
        assert!(rl.check_and_record());
        assert!(rl.check_and_record());
        // Fourth should be rejected.
        assert!(!rl.check_and_record());
    }

    #[test]
    fn rate_limiter_zero_limit_rejects_all() {
        let mut rl = RateLimiter::new(0);
        assert!(!rl.check_and_record());
    }
}
