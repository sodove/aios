use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Fill, Length};

use crate::app::Message;
use crate::theme::{self, ConfirmTheme};

/// Renders the idle waiting screen displayed when no confirmation request is active.
///
/// Includes debug/simulate buttons for testing the UI without a live IPC connection.
pub fn view() -> Element<'static, Message> {
    // Close button in top-right
    let close_btn = button(text("X").size(14).color(ConfirmTheme::TEXT_MUTED))
        .on_press(Message::CloseWindow)
        .padding([4, 10])
        .style(theme::simulate_button);

    let header = row![
        Space::new().width(Length::Fill),
        close_btn,
    ];

    let title = text("AIOS Confirm")
        .size(24)
        .color(ConfirmTheme::TEXT);

    let subtitle = text("Waiting for requests...")
        .size(14)
        .color(ConfirmTheme::TEXT_MUTED);

    let simulate_normal = button(
        text("Simulate Normal").size(13),
    )
    .style(theme::simulate_button)
    .on_press(Message::SimulateNormalRequest)
    .padding([8, 16]);

    let simulate_critical = button(
        text("Simulate Critical").size(13),
    )
    .style(theme::simulate_button)
    .on_press(Message::SimulateCriticalRequest)
    .padding([8, 16]);

    let content = column![
        header,
        Space::new().height(40),
        title,
        Space::new().height(8),
        subtitle,
        Space::new().height(32),
        simulate_normal,
        Space::new().height(8),
        simulate_critical,
    ]
    .align_x(iced::Center);

    container(content)
        .padding(16)
        .center(Fill)
        .style(theme::dark_container)
        .into()
}
