mod app;
mod commands;
mod theme;
mod views;

use app::SettingsApp;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_settings=info".into()),
        )
        .init();

    tracing::info!("aios-settings starting...");

    iced::application(SettingsApp::new, SettingsApp::update, SettingsApp::view)
        .title("AIOS Settings")
        .theme(iced::Theme::TokyoNight)
        .window_size((700.0, 500.0))
        .centered()
        .antialiasing(true)
        .run()
}
