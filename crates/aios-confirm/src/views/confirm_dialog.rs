use aios_common::TrustLevel;
use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Fill};

use crate::app::Message;
use crate::theme::{self, ConfirmTheme};

/// Renders the standard (non-destructive) confirmation dialog.
///
/// Displays the action type, description, command, and trust level
/// with color-coded indicators. Offers "Cancel" and "Allow" buttons.
pub fn view<'a>(
    action_type: &'a str,
    description: &'a str,
    command: &'a str,
    trust_level: &'a TrustLevel,
) -> Element<'a, Message> {
    let header = text("Confirm action")
        .size(20)
        .color(ConfirmTheme::WARNING);

    let close_btn = button(text("X").size(14).color(ConfirmTheme::TEXT_MUTED))
        .on_press(Message::Reject)
        .padding([4, 10])
        .style(theme::cancel_button);

    let top_row = row![
        header,
        Space::new().width(Fill),
        close_btn,
    ]
    .align_y(iced::Alignment::Center);

    let type_row = row![
        text("Type: ").size(13).color(ConfirmTheme::TEXT_MUTED),
        text(action_type).size(13).color(ConfirmTheme::TEXT),
    ];

    let desc_label = text(description)
        .size(14)
        .color(ConfirmTheme::TEXT);

    let command_block = container(
        text(command)
            .size(13)
            .color(ConfirmTheme::TEXT),
    )
    .padding(12)
    .width(Fill)
    .style(theme::command_container);

    let trust_color = ConfirmTheme::trust_color(trust_level);
    let trust_label = ConfirmTheme::trust_label(trust_level);

    let trust_row = container(
        row![
            text("Source: ").size(13).color(ConfirmTheme::TEXT_MUTED),
            text(trust_label).size(13).color(trust_color),
        ],
    )
    .padding(8)
    .style(theme::trust_badge_container(trust_level));

    let cancel_btn = button(text("Cancel").size(14))
        .style(theme::cancel_button)
        .on_press(Message::Reject)
        .padding([10, 24]);

    let approve_btn = button(text("Allow").size(14))
        .style(theme::approve_button)
        .on_press(Message::Approve)
        .padding([10, 24]);

    let buttons = row![
        cancel_btn,
        Space::new().width(Fill),
        approve_btn,
    ]
    .width(Fill);

    let content = column![
        top_row,
        Space::new().height(12),
        type_row,
        Space::new().height(8),
        desc_label,
        Space::new().height(12),
        text("Command:").size(12).color(ConfirmTheme::TEXT_MUTED),
        Space::new().height(4),
        command_block,
        Space::new().height(12),
        trust_row,
        Space::new().height(20),
        buttons,
    ]
    .width(Fill);

    container(content)
        .padding(24)
        .width(Fill)
        .height(Fill)
        .style(theme::dark_container)
        .into()
}
