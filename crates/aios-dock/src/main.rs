mod app;
mod launcher;
mod theme;
mod views;

use app::DockApp;

use iced::window;

/// Dock panel height in logical pixels.
const DOCK_HEIGHT: f32 = 48.0;

/// Default dock width (will stretch to screen width on Wayland via layer-shell).
const DOCK_WIDTH: f32 = 1024.0;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_dock=info".into()),
        )
        .init();

    tracing::info!("aios-dock starting...");

    // TODO: On Linux/Wayland, replace with iced_layershell for proper
    // layer-shell anchoring (Bottom + Left + Right, exclusive zone 48).
    // iced_layershell does not compile on macOS, so we use a regular
    // iced window positioned at the bottom of the screen.

    iced::application(DockApp::new, DockApp::update, DockApp::view)
        .title("AIOS Dock")
        .theme(iced::Theme::TokyoNight)
        .window_size((DOCK_WIDTH, DOCK_HEIGHT))
        .position(window::Position::SpecificWith(position_at_bottom))
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

/// Compute the initial window position so the dock sits at the bottom-center
/// of the primary monitor.
///
/// `window_size` is the dock dimensions, `monitor_size` is the screen resolution.
fn position_at_bottom(window_size: iced::Size, monitor_size: iced::Size) -> iced::Point {
    let x = (monitor_size.width - window_size.width) / 2.0;
    let y = monitor_size.height - window_size.height;
    iced::Point::new(x, y)
}
