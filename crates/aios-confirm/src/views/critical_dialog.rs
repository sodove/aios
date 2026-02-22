use aios_common::TrustLevel;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Color, Element, Fill};

use crate::app::Message;
use crate::theme::{self, ConfirmTheme};

/// The exact string the user must type to confirm a destructive action.
const CONFIRM_KEYWORD: &str = "DELETE";

/// Renders the critical (destructive) confirmation dialog.
///
/// Requires the user to type "DELETE" before the confirm button becomes active.
/// Uses red/danger theming to clearly signal the irreversible nature of the action.
pub fn view<'a>(
    action_type: &'a str,
    description: &'a str,
    command: &'a str,
    trust_level: &'a TrustLevel,
    confirm_input: &'a str,
) -> Element<'a, Message> {
    let header = text("DANGEROUS ACTION")
        .size(20)
        .color(ConfirmTheme::DANGER);

    let type_row = row![
        text("Type: ").size(13).color(ConfirmTheme::TEXT_MUTED),
        text(action_type).size(13).color(ConfirmTheme::DANGER),
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

    // WebContent gets an extra prominent warning.
    let web_warning: Option<Element<'_, Message>> =
        if *trust_level == TrustLevel::WebContent {
            Some(
                container(
                    text("WebContent source -- exercise extreme caution!")
                        .size(13)
                        .color(Color::WHITE),
                )
                .padding(8)
                .width(Fill)
                .style(theme::danger_container)
                .into(),
            )
        } else {
            None
        };

    let irreversible_warning = container(
        text("This action is irreversible!")
            .size(13)
            .color(ConfirmTheme::DANGER),
    )
    .padding(8)
    .width(Fill)
    .style(theme::danger_container);

    let input_label = text(format!("Type \"{CONFIRM_KEYWORD}\" to confirm:"))
        .size(13)
        .color(ConfirmTheme::TEXT_MUTED);

    let input_field = text_input("", confirm_input)
        .on_input(Message::ConfirmInputChanged)
        .padding(10)
        .size(14)
        .style(theme::confirm_input);

    let confirmed = confirm_input == CONFIRM_KEYWORD;

    let cancel_btn = button(text("Cancel").size(14))
        .style(theme::cancel_button)
        .on_press(Message::Reject)
        .padding([10, 24]);

    let confirm_btn = if confirmed {
        button(text("Confirm").size(14))
            .style(theme::danger_button)
            .on_press(Message::Approve)
            .padding([10, 24])
    } else {
        button(text("Confirm").size(14))
            .style(theme::disabled_button)
            .padding([10, 24])
    };

    let buttons = row![
        cancel_btn,
        Space::new().width(Fill),
        confirm_btn,
    ]
    .width(Fill);

    let mut content = column![
        header,
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
    ]
    .width(Fill);

    if let Some(warning) = web_warning {
        content = content
            .push(Space::new().height(8))
            .push(warning);
    }

    content = content
        .push(Space::new().height(8))
        .push(irreversible_warning)
        .push(Space::new().height(12))
        .push(input_label)
        .push(Space::new().height(4))
        .push(input_field)
        .push(Space::new().height(16))
        .push(buttons);

    container(content)
        .padding(24)
        .width(Fill)
        .height(Fill)
        .style(theme::dark_container)
        .into()
}
