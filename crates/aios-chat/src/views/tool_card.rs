use iced::widget::{column, container, row, text, Space};
use iced::{Element, Length, Theme};

use crate::app::Message;
use crate::state::{DisplayMessage, ToolStatus};
use crate::theme::{self, AiosColors};

/// Renders a tool call or tool result as a visually distinct card.
///
/// Cards are color-coded by status:
/// - **Pending**: amber border and background
/// - **Completed**: green border and background
/// - **Failed / Rejected**: red border and background
pub fn view(msg: &DisplayMessage) -> Element<'_, Message> {
    let status = msg.tool_status.unwrap_or(ToolStatus::Pending);
    let tool_name = msg.tool_name.as_deref().unwrap_or("unknown");

    let (icon, status_label) = status_decoration(status);

    // Header row: icon + tool name
    let header = row![
        text(icon).size(14),
        text(tool_name)
            .size(14)
            .color(AiosColors::TEXT_PRIMARY),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // Body content differs between call and result cards.
    let body = build_body(msg, status, status_label);

    // Timestamp
    let timestamp_label = msg.timestamp.format("%H:%M").to_string();

    let card_content = column![header, body, text(timestamp_label).size(10).color(AiosColors::TEXT_SECONDARY)]
        .spacing(4);

    let style: fn(&Theme) -> container::Style = match status {
        ToolStatus::Pending => theme::container_tool_pending,
        ToolStatus::Completed => theme::container_tool_completed,
        ToolStatus::Failed | ToolStatus::Rejected => theme::container_tool_failed,
    };

    let card = container(card_content)
        .padding(10)
        .max_width(520)
        .style(style);

    // Tool cards are left-aligned like assistant messages.
    row![card, Space::new().width(Length::Fill)]
        .spacing(8)
        .into()
}

/// Returns (icon, status_label) for the given tool status.
fn status_decoration(status: ToolStatus) -> (&'static str, &'static str) {
    match status {
        ToolStatus::Pending => ("[~]", "Pending..."),
        ToolStatus::Completed => ("[ok]", "Completed"),
        ToolStatus::Failed => ("[err]", "Failed"),
        ToolStatus::Rejected => ("[x]", "Rejected"),
    }
}

/// Builds the body section of a tool card.
fn build_body<'a>(
    msg: &'a DisplayMessage,
    status: ToolStatus,
    status_label: &'a str,
) -> Element<'a, Message> {
    match status {
        ToolStatus::Pending => {
            // Show pretty-printed arguments and a pending indicator.
            let mut col = column![].spacing(2);
            if let Some(args) = &msg.tool_args {
                col = col.push(
                    text(args)
                        .size(12)
                        .color(AiosColors::TEXT_SECONDARY),
                );
            }
            col = col.push(
                text(status_label)
                    .size(11)
                    .color(AiosColors::TOOL_PENDING_BORDER),
            );
            col.into()
        }
        ToolStatus::Completed => {
            // Show (possibly truncated) output.
            let mut col = column![].spacing(2);
            if !msg.text.is_empty() {
                col = col.push(
                    text(&msg.text)
                        .size(12)
                        .color(AiosColors::TEXT_SECONDARY),
                );
            }
            col.into()
        }
        ToolStatus::Failed | ToolStatus::Rejected => {
            // Show error output.
            let label = if status == ToolStatus::Rejected {
                "Action rejected by user"
            } else if msg.text.is_empty() {
                "Tool execution failed"
            } else {
                &msg.text
            };
            text(label)
                .size(12)
                .color(AiosColors::TOOL_FAILED_BORDER)
                .into()
        }
    }
}
