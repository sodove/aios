use aios_common::TrustLevel;
use iced::{Element, Task as IcedTask};
use uuid::Uuid;

use crate::views::{confirm_dialog, critical_dialog, waiting_view};

/// Root application state for the AIOS Confirm dialog.
pub struct AiosConfirm {
    state: ConfirmState,
}

/// The current state of the confirmation dialog.
enum ConfirmState {
    /// Idle -- no active confirmation request.
    Waiting,

    /// Showing a standard (non-destructive) confirmation dialog.
    Normal {
        action_id: Uuid,
        action_type: String,
        description: String,
        command: String,
        trust_level: TrustLevel,
    },

    /// Showing a critical (destructive) confirmation dialog that requires
    /// the user to type "DELETE" before the confirm button activates.
    Critical {
        action_id: Uuid,
        action_type: String,
        description: String,
        command: String,
        trust_level: TrustLevel,
        confirm_input: String,
    },
}

/// Messages exchanged within the Iced application.
#[derive(Debug, Clone)]
pub enum Message {
    // -- Simulation (debug/testing without IPC) --
    SimulateNormalRequest,
    SimulateCriticalRequest,

    // -- Dialog interactions --
    Approve,
    Reject,
    ConfirmInputChanged(String),

    // -- Post-response (will be used when IPC is wired up) --
    #[allow(dead_code)]
    ResponseSent,

    /// User clicked the close (X) button.
    CloseWindow,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Action type keywords that indicate a destructive / dangerous operation.
#[allow(dead_code)]
const CRITICAL_KEYWORDS: &[&str] = &[
    "delete", "remove", "drop", "exec", "shell", "format",
];

/// Determines whether a confirmation request should use the critical dialog.
///
/// A request is considered critical if:
/// - The `action_type` contains any of the [`CRITICAL_KEYWORDS`], **or**
/// - The `trust_level` is [`TrustLevel::WebContent`] (any action from
///   web content is inherently untrusted).
#[allow(dead_code)]
fn is_critical(action_type: &str, trust_level: &TrustLevel) -> bool {
    if *trust_level == TrustLevel::WebContent {
        return true;
    }
    let lower = action_type.to_lowercase();
    CRITICAL_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

// ---------------------------------------------------------------------------
// Iced lifecycle
// ---------------------------------------------------------------------------

impl AiosConfirm {
    /// Creates the initial application state.
    pub fn new() -> (Self, IcedTask<Message>) {
        let app = Self {
            state: ConfirmState::Waiting,
        };
        (app, IcedTask::none())
    }

    /// Processes an incoming [`Message`] and returns a follow-up task.
    pub fn update(&mut self, message: Message) -> IcedTask<Message> {
        match message {
            Message::SimulateNormalRequest => {
                tracing::info!("simulating normal confirmation request");
                let action_id = Uuid::new_v4();
                self.state = ConfirmState::Normal {
                    action_id,
                    action_type: "file_write".into(),
                    description: "Write file /home/user/notes.txt".into(),
                    command: "echo \"hello\" > notes.txt".into(),
                    trust_level: TrustLevel::User,
                };
            }

            Message::SimulateCriticalRequest => {
                tracing::info!("simulating critical confirmation request");
                let action_id = Uuid::new_v4();
                self.state = ConfirmState::Critical {
                    action_id,
                    action_type: "file_delete".into(),
                    description: "Delete file /home/user/important.doc".into(),
                    command: "rm /home/user/important.doc".into(),
                    trust_level: TrustLevel::WebContent,
                    confirm_input: String::new(),
                };
            }

            Message::Approve => {
                let (action_id, action_type) = match &self.state {
                    ConfirmState::Normal { action_id, action_type, .. } => {
                        (*action_id, action_type.clone())
                    }
                    ConfirmState::Critical { action_id, action_type, .. } => {
                        (*action_id, action_type.clone())
                    }
                    ConfirmState::Waiting => return IcedTask::none(),
                };
                tracing::info!(
                    action_id = %action_id,
                    action_type = %action_type,
                    "action APPROVED by user",
                );
                self.state = ConfirmState::Waiting;
            }

            Message::Reject => {
                let (action_id, action_type) = match &self.state {
                    ConfirmState::Normal { action_id, action_type, .. } => {
                        (*action_id, action_type.clone())
                    }
                    ConfirmState::Critical { action_id, action_type, .. } => {
                        (*action_id, action_type.clone())
                    }
                    ConfirmState::Waiting => return IcedTask::none(),
                };
                tracing::info!(
                    action_id = %action_id,
                    action_type = %action_type,
                    "action REJECTED by user",
                );
                self.state = ConfirmState::Waiting;
            }

            Message::ConfirmInputChanged(value) => {
                if let ConfirmState::Critical { confirm_input, .. } = &mut self.state {
                    *confirm_input = value;
                }
            }

            Message::ResponseSent => {
                self.state = ConfirmState::Waiting;
            }

            Message::CloseWindow => {
                return iced::window::close(iced::window::Id::MAIN);
            }
        }

        IcedTask::none()
    }

    /// Produces the view tree for the current state.
    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            ConfirmState::Waiting => waiting_view::view(),

            ConfirmState::Normal {
                action_type,
                description,
                command,
                trust_level,
                ..
            } => confirm_dialog::view(action_type, description, command, trust_level),

            ConfirmState::Critical {
                action_type,
                description,
                command,
                trust_level,
                confirm_input,
                ..
            } => critical_dialog::view(
                action_type,
                description,
                command,
                trust_level,
                confirm_input,
            ),
        }
    }
}

/// Determines whether a request should show the critical dialog.
///
/// Will be used when IPC is wired up to route incoming `ConfirmRequest`
/// messages to the appropriate dialog variant.
#[allow(dead_code)]
pub fn request_is_critical(action_type: &str, trust_level: &TrustLevel) -> bool {
    is_critical(action_type, trust_level)
}
