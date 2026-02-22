use iced::widget::{column, container, markdown, row, text, Space};
use iced::{Element, Length, Theme};

use crate::app::Message;
use crate::state::{DisplayMessage, MessageRole};
use crate::theme::{self, AiosColors};
use crate::views::tool_card;

/// Renders a single chat message as a bubble.
///
/// - User messages are right-aligned with `USER_BUBBLE` background, plain text.
/// - Assistant messages are left-aligned with `ASSISTANT_BUBBLE` background, markdown rendered.
/// - Tool call / result messages are rendered as distinct cards via [`tool_card::view`].
pub fn view(msg: &DisplayMessage) -> Element<'_, Message> {
    match msg.role {
        MessageRole::ToolCall | MessageRole::ToolResult => {
            return tool_card::view(msg);
        }
        MessageRole::User | MessageRole::Assistant => {}
    }

    let timestamp_label = msg.timestamp.format("%H:%M").to_string();

    let content_element: Element<'_, Message> = match msg.role {
        MessageRole::User => text(&msg.text).size(14).into(),
        MessageRole::Assistant => render_assistant_markdown(msg),
        // Handled above with early return.
        MessageRole::ToolCall | MessageRole::ToolResult => unreachable!(),
    };

    let body = column![
        content_element,
        text(timestamp_label)
            .size(10)
            .color(AiosColors::TEXT_SECONDARY),
    ]
    .spacing(4);

    let bubble_style: fn(&Theme) -> container::Style = match msg.role {
        MessageRole::User => theme::container_user_bubble,
        MessageRole::Assistant => theme::container_assistant_bubble,
        MessageRole::ToolCall | MessageRole::ToolResult => unreachable!(),
    };

    let bubble = container(body)
        .padding(10)
        .max_width(520)
        .style(bubble_style);

    match msg.role {
        MessageRole::User => row![Space::new().width(Length::Fill), bubble]
            .spacing(8)
            .into(),
        MessageRole::Assistant => row![bubble, Space::new().width(Length::Fill)]
            .spacing(8)
            .into(),
        MessageRole::ToolCall | MessageRole::ToolResult => unreachable!(),
    }
}

/// Renders assistant message content as markdown.
///
/// If the message has pre-parsed markdown content, renders it with the Iced markdown widget.
/// Falls back to plain text if no parsed content is available.
fn render_assistant_markdown(msg: &DisplayMessage) -> Element<'_, Message> {
    match &msg.markdown_content {
        Some(content) => {
            let settings = markdown::Settings::with_text_size(
                14,
                markdown::Style::from_palette(Theme::TokyoNight.palette()),
            );

            markdown::view(content.items(), settings)
                .map(Message::OpenUrl)
        }
        None => text(&msg.text).size(14).into(),
    }
}
