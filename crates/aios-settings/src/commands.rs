//! System command execution for settings tabs.
//! All commands run directly without AI dependency.

use std::process::Command;

/// Result of a system command execution.
#[derive(Debug, Clone)]
pub struct CmdResult {
    pub success: bool,
    pub output: String,
}

fn run_cmd(program: &str, args: &[&str]) -> CmdResult {
    match Command::new(program).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            CmdResult {
                success: output.status.success(),
                output: if stdout.is_empty() { stderr } else { stdout },
            }
        }
        Err(e) => CmdResult {
            success: false,
            output: format!("Failed to run {program}: {e}"),
        },
    }
}

// -- Network commands (nmcli) --

pub fn wifi_scan() -> CmdResult {
    run_cmd("nmcli", &["-t", "-f", "SSID,SIGNAL,SECURITY,IN-USE", "dev", "wifi", "list", "--rescan", "yes"])
}

pub fn wifi_connect(ssid: &str, password: &str) -> CmdResult {
    if password.is_empty() {
        run_cmd("nmcli", &["dev", "wifi", "connect", ssid])
    } else {
        run_cmd("nmcli", &["dev", "wifi", "connect", ssid, "password", password])
    }
}

pub fn wifi_disconnect() -> CmdResult {
    run_cmd("nmcli", &["dev", "disconnect", "wlan0"])
}

pub fn network_status() -> CmdResult {
    run_cmd("nmcli", &["-t", "-f", "DEVICE,TYPE,STATE,CONNECTION", "dev", "status"])
}

// -- Display commands (swaymsg) --

pub fn display_list() -> CmdResult {
    run_cmd("swaymsg", &["-t", "get_outputs", "-r"])
}

pub fn display_set_mode(output_name: &str, width: u32, height: u32, hz: f32) -> CmdResult {
    let mode = format!("{width}x{height}@{hz:.3}Hz");
    run_cmd("swaymsg", &["output", output_name, "mode", &mode])
}

// -- Ollama commands --

pub fn ollama_status() -> CmdResult {
    run_cmd("systemctl", &["is-active", "ollama"])
}

pub fn ollama_start() -> CmdResult {
    run_cmd("systemctl", &["start", "ollama"])
}

pub fn ollama_stop() -> CmdResult {
    run_cmd("systemctl", &["stop", "ollama"])
}

pub fn ollama_list_models() -> CmdResult {
    run_cmd("ollama", &["list"])
}

pub fn ollama_pull(model: &str) -> CmdResult {
    run_cmd("ollama", &["pull", model])
}

pub fn ollama_remove(model: &str) -> CmdResult {
    run_cmd("ollama", &["rm", model])
}
