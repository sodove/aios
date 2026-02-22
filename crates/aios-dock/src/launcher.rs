//! Application launcher -- spawns external processes for dock icons.

use std::process::Command;

/// Attempts to launch the `aios-chat` binary.
///
/// Logs an error if the binary cannot be found or started, but never panics.
pub fn launch_chat() {
    if let Err(e) = Command::new("aios-chat").spawn() {
        tracing::error!("Failed to launch aios-chat: {e}");
    }
}

/// Attempts to launch a web browser (Chromium with Wayland hints on Linux,
/// or the default `open` command on macOS).
///
/// Logs an error if the browser cannot be started, but never panics.
pub fn launch_browser() {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg("https://google.com").spawn()
    } else {
        Command::new("chromium")
            .arg("--ozone-platform-hint=auto")
            .spawn()
    };

    if let Err(e) = result {
        tracing::error!("Failed to launch browser: {e}");
    }
}

/// Attempts to launch the `foot` terminal emulator.
pub fn launch_terminal() {
    if let Err(e) = Command::new("foot").spawn() {
        tracing::error!("Failed to launch foot: {e}");
    }
}

/// Attempts to launch the `aios-settings` binary.
pub fn launch_settings() {
    if let Err(e) = Command::new("aios-settings").spawn() {
        tracing::error!("Failed to launch aios-settings: {e}");
    }
}
