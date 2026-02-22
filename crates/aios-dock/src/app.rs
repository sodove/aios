//! Core application state, messages, and logic for the AIOS Dock.

use iced::{Element, Task};

use crate::launcher;
use crate::views::dock_bar;

/// Identifies a launchable application in the dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppId {
    Chat,
    Browser,
    Terminal,
    Settings,
}

/// All messages the dock UI can produce.
#[derive(Debug, Clone)]
pub enum Message {
    /// Periodic tick -- refreshes clock and system status.
    Tick,
    /// User clicked an app icon to launch it.
    LaunchApp(AppId),
}

/// Root application state for the dock panel.
pub struct DockApp {
    /// Current clock string, e.g. "15:30".
    pub(crate) clock: String,
    /// Whether Wi-Fi is connected (hardcoded for MVP).
    pub(crate) wifi_connected: bool,
    /// Battery percentage, if available (`None` on desktop).
    pub(crate) battery_percent: Option<u8>,
    /// Volume percentage (hardcoded for MVP).
    pub(crate) volume_percent: u8,
}

impl DockApp {
    /// Bootstrap the dock application state.
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            clock: current_time(),
            wifi_connected: true,
            battery_percent: None,
            volume_percent: 50,
        };
        (state, Task::none())
    }

    /// Process an incoming message and return a follow-up task.
    #[allow(clippy::needless_pass_by_value)] // iced requires owned Message
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock = current_time();
                // WiFi, battery, volume -- hardcoded until IPC to aios-agent is wired.
            }
            Message::LaunchApp(app) => match app {
                AppId::Chat => launcher::launch_chat(),
                AppId::Browser => launcher::launch_browser(),
                AppId::Terminal => launcher::launch_terminal(),
                AppId::Settings => launcher::launch_settings(),
            },
        }
        Task::none()
    }

    /// Build the view tree for the current dock state.
    pub fn view(&self) -> Element<'_, Message> {
        dock_bar::view(self)
    }
}

/// Returns the current local time formatted as `HH:MM`.
fn current_time() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}
