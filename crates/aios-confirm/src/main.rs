mod app;
mod theme;
mod views;

use app::AiosConfirm;
use iced::Theme;
use theme::ConfirmTheme;

fn main() -> Result<(), iced::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aios_confirm=info".into()),
        )
        .init();

    tracing::info!("aios-confirm starting...");

    iced::application(AiosConfirm::new, AiosConfirm::update, AiosConfirm::view)
        .title("AIOS Confirm")
        .window_size((500.0, 400.0))
        .centered()
        .resizable(false)
        .theme(Theme::Dark)
        .style(|_state: &AiosConfirm, _theme: &Theme| iced::theme::Style {
            background_color: ConfirmTheme::BG,
            text_color: ConfirmTheme::TEXT,
        })
        .run()
}
