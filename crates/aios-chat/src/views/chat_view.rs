use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::app::{AiosChat, Message};
use crate::state::ConnectionStatus;
use crate::theme::{self, AiosColors};
use crate::views::{input_bar, message_bubble};

/// Renders the full chat layout: header, scrollable message list, and input bar.
pub fn view(state: &AiosChat) -> Element<'_, Message> {
    let header = header_row(state.connection_status());
    let messages = message_list(state);
    let input = input_bar::view(state.input_text(), state.can_send());

    let content = column![header, messages, input];

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::container_primary)
        .into()
}

/// The top header bar with the application title and connection status.
fn header_row(status: ConnectionStatus) -> Element<'static, Message> {
    let title = text("AIOS Chat").size(18).color(AiosColors::TEXT_PRIMARY);

    let status_color = match status {
        ConnectionStatus::Connected => AiosColors::ACCENT,
        ConnectionStatus::Connecting => AiosColors::TEXT_SECONDARY,
        ConnectionStatus::Disconnected => iced::Color::from_rgb(0.85, 0.30, 0.30),
    };

    let status_label = text(status.label()).size(12).color(status_color);

    let close_btn = button(text("X").size(14).color(AiosColors::TEXT_SECONDARY))
        .on_press(Message::CloseWindow)
        .padding([4, 10])
        .style(theme::close_button);

    let bar = row![
        title,
        Space::new().width(Length::Fill),
        status_label,
        close_btn
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .padding(12)
        .style(theme::container_secondary)
        .into()
}

/// The scrollable list of chat messages.
fn message_list(state: &AiosChat) -> Element<'_, Message> {
    let messages = state.messages();

    let content: Element<'_, Message> = if messages.is_empty() {
        container(
            text("No messages yet. Start a conversation!")
                .size(14)
                .color(AiosColors::TEXT_SECONDARY),
        )
        .center(Length::Fill)
        .into()
    } else {
        let mut col = column![].spacing(8).padding([8, 12]);
        for msg in messages {
            col = col.push(message_bubble::view(msg));
        }
        col.into()
    };

    scrollable(container(content).width(Length::Fill))
        .height(Length::Fill)
        .style(theme::scrollable_dark)
        .into()
}
