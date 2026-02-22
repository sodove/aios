mod app;
mod launcher;
mod theme;
mod views;

use app::DockApp;

use iced::window;

/// Dock panel height in logical pixels.
const DOCK_HEIGHT: f32 = 48.0;

/// Query sway for the focused output dimensions.
/// Returns (x, y, width, height) of the focused output, or defaults for 1920x1080 at (0,0).
fn get_focused_output() -> (f32, f32, f32, f32) {
    let output = std::process::Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
        .ok();

    if let Some(out) = output {
        if let Ok(outputs) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            if let Some(focused) = outputs.iter().find(|o| {
                o.get("focused").and_then(|v| v.as_bool()).unwrap_or(false)
            }) {
                if let Some(rect) = focused.get("rect") {
                    let x = rect.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let y = rect.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let w = rect.get("width").and_then(|v| v.as_f64()).unwrap_or(1920.0) as f32;
                    let h = rect.get("height").and_then(|v| v.as_f64()).unwrap_or(1080.0) as f32;
                    return (x, y, w, h);
                }
            }
        }
    }

    (0.0, 0.0, 1920.0, 1080.0)
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_dock=info".into()),
        )
        .init();

    tracing::info!("aios-dock starting...");

    let (out_x, out_y, out_w, out_h) = get_focused_output();
    let dock_width = out_w;
    let dock_x = out_x;
    let dock_y = out_y + out_h - DOCK_HEIGHT;

    tracing::info!(
        "Dock: output at ({out_x},{out_y}) size {out_w}x{out_h}, placing dock at ({dock_x},{dock_y}) width {dock_width}"
    );

    iced::application(DockApp::new, DockApp::update, DockApp::view)
        .title("AIOS Dock")
        .theme(iced::Theme::TokyoNight)
        .window_size((dock_width, DOCK_HEIGHT))
        .position(window::Position::Specific(iced::Point::new(dock_x, dock_y)))
        .level(window::Level::AlwaysOnTop)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .antialiasing(true)
        .subscription(|_state| {
            iced::time::every(std::time::Duration::from_secs(5)).map(|_| app::Message::Tick)
        })
        .run()
}
