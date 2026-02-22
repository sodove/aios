mod app;
mod ipc_client;
mod state;
mod theme;
mod views;

use app::AiosChat;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_chat=info".into()),
        )
        .init();

    tracing::info!("aios-chat starting...");

    iced::application(AiosChat::new, AiosChat::update, AiosChat::view)
        .subscription(AiosChat::subscription)
        .title("AIOS Chat")
        .theme(iced::Theme::TokyoNight)
        .window_size((800.0, 600.0))
        .centered()
        .antialiasing(true)
        .run()
}
