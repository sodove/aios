mod app;
mod launcher;
mod theme;
mod views;

use app::DockApp;

/// Dock panel height in logical pixels.
const DOCK_HEIGHT: f32 = 48.0;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_dock=info".into()),
        )
        .init();

    tracing::info!("aios-dock starting...");

    // On Wayland, Position::Specific is ignored by the compositor.
    // The dock positions itself via swaymsg after the window is created (see app.rs).
    iced::application(DockApp::new, DockApp::update, DockApp::view)
        .title("AIOS Dock")
        .theme(iced::Theme::TokyoNight)
        .window_size((1920.0, DOCK_HEIGHT))
        .level(iced::window::Level::AlwaysOnTop)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .antialiasing(true)
        .subscription(|_state| {
            iced::time::every(std::time::Duration::from_secs(5)).map(|_| app::Message::Tick)
        })
        .run()
}
