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
    /// Current keyboard layout, e.g. "EN" or "RU".
    pub(crate) kbd_layout: String,
}

impl DockApp {
    /// Bootstrap the dock application state.
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            clock: current_time(),
            wifi_connected: true,
            battery_percent: None,
            volume_percent: 50,
            kbd_layout: current_kbd_layout(),
        };

        // On Wayland, clients cannot set their own window position.
        // We spawn a background thread that retries swaymsg IPC until
        // the window is found and positioned at the bottom.
        std::thread::spawn(|| {
            for attempt in 1..=5 {
                std::thread::sleep(std::time::Duration::from_millis(600 * attempt));
                if position_dock_via_sway() {
                    tracing::info!("Dock positioned successfully on attempt {attempt}");
                    return;
                }
                tracing::warn!("Dock positioning attempt {attempt} failed, retrying...");
            }
            tracing::error!("Failed to position dock after 5 attempts");
        });

        (state, Task::none())
    }

    /// Process an incoming message and return a follow-up task.
    #[allow(clippy::needless_pass_by_value)] // iced requires owned Message
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.clock = current_time();
                self.kbd_layout = current_kbd_layout();
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

/// Query sway for the active keyboard layout via `swaymsg -t get_inputs`.
///
/// Returns a short label like "EN" or "RU".
fn current_kbd_layout() -> String {
    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_inputs", "-r"])
        .output()
        .ok();

    if let Some(out) = output {
        if let Ok(inputs) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            // Find first keyboard input with xkb_active_layout_name
            for input in &inputs {
                if input.get("type").and_then(|v| v.as_str()) != Some("keyboard") {
                    continue;
                }
                if let Some(layout) = input
                    .get("xkb_active_layout_name")
                    .and_then(|v| v.as_str())
                {
                    return layout_to_short(layout);
                }
            }
        }
    }

    "EN".to_owned()
}

/// Convert a full layout name (e.g. "English (US)", "Russian") to a short label.
fn layout_to_short(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("russian") || lower.contains("ru") {
        "RU".to_owned()
    } else if lower.contains("english") || lower.contains("us") {
        "EN".to_owned()
    } else if lower.contains("german") || lower.contains("de") {
        "DE".to_owned()
    } else if lower.contains("french") || lower.contains("fr") {
        "FR".to_owned()
    } else {
        // Take first 2 chars uppercase as fallback
        name.chars().take(2).collect::<String>().to_uppercase()
    }
}

/// Use swaymsg IPC to position the dock at the bottom of the focused output.
///
/// Returns `true` if the move command succeeded.
fn position_dock_via_sway() -> bool {
    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
        .ok();

    let Some(out) = output else {
        tracing::warn!("swaymsg not available");
        return false;
    };

    let (x, _y, w, h) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout)
        .ok()
        .and_then(|outputs| {
            let focused = outputs.iter().find(|o| {
                o.get("focused").and_then(|v| v.as_bool()).unwrap_or(false)
            })?;
            let rect = focused.get("rect")?;
            let x = rect.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let y = rect.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let w = rect.get("width").and_then(|v| v.as_f64()).unwrap_or(1920.0);
            let h = rect.get("height").and_then(|v| v.as_f64()).unwrap_or(1080.0);
            Some((x, y, w, h))
        })
        .unwrap_or((0.0, 0.0, 1920.0, 1080.0));

    let dock_x = x as i32;
    let dock_y = (_y + h - 48.0) as i32;
    let dock_w = w as i32;

    // Use PID matching — 100% reliable since we know our own PID.
    let pid = std::process::id();
    let sel = format!("[pid={pid}]");

    tracing::info!("Positioning dock via swaymsg {sel}: ({dock_x}, {dock_y}) width {dock_w}");

    // Force floating (for_window rules may not have matched).
    let cmds = [
        format!("{sel} floating enable"),
        format!("{sel} sticky enable"),
        format!("{sel} resize set width {dock_w} height 48"),
        format!("{sel} move absolute position {dock_x} {dock_y}"),
    ];

    let mut ok = true;
    for cmd in &cmds {
        match std::process::Command::new("swaymsg").arg(cmd).output() {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                tracing::warn!("swaymsg `{cmd}` failed: {err}");
                ok = false;
            }
            Err(e) => {
                tracing::warn!("swaymsg `{cmd}` error: {e}");
                ok = false;
            }
        }
    }
    ok
}
