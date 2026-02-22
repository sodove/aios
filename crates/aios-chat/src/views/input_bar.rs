use iced::widget::{button, container, row, text, text_input};
use iced::{Element, Length};

use crate::app::Message;
use crate::theme;

/// Renders the bottom input bar with a text field and a send button.
pub fn view<'a>(input_text: &str, can_send: bool) -> Element<'a, Message> {
    let input = text_input("Type a message...", input_text)
        .on_input(Message::InputChanged)
        .on_submit(Message::SendMessage)
        .padding(10)
        .size(14)
        .style(theme::input_style);

    let send_btn = button(text("Send").size(14))
        .on_press_maybe(if can_send {
            Some(Message::SendMessage)
        } else {
            None
        })
        .padding([8, 16])
        .style(theme::send_button);

    let bar = row![input, send_btn]
        .spacing(8)
        .align_y(iced::Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .padding(12)
        .style(theme::container_secondary)
        .into()
}
