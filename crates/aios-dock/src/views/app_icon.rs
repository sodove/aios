//! A single application icon button for the dock bar.

use iced::widget::{button, center, text};
use iced::{Element, Length};

use crate::app::{AppId, Message};
use crate::theme;

/// Renders a single dock icon as a fixed-size button with a text label.
///
/// The button sends `Message::LaunchApp(id)` when pressed.
pub fn view(label: &str, id: AppId) -> Element<'static, Message> {
    let content = center(text(label.to_owned()).size(13))
        .width(Length::Fill)
        .height(Length::Fill);

    button(content)
        .width(48.0)
        .height(40.0)
        .padding(4)
        .style(theme::app_icon_button)
        .on_press(Message::LaunchApp(id))
        .into()
}
