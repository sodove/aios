//! Main dock bar layout -- horizontal panel with app icons and system tray.

use iced::widget::{container, row, Space};
use iced::{Element, Length};

use crate::app::{AppId, DockApp, Message};
use crate::theme;
use crate::views::{app_icon, system_tray};

/// Renders the full dock bar.
///
/// ```text
/// +------+------+------------------+---+---+---+-------+
/// | Chat | Web  |    (spacer)      |WiFi|Vol|Bat| 15:30 |
/// +------+------+------------------+---+---+---+-------+
/// ```
pub fn view(state: &DockApp) -> Element<'_, Message> {
    let chat_icon = app_icon::view("Chat", AppId::Chat);
    let web_icon = app_icon::view("Web", AppId::Browser);

    let app_icons = row![chat_icon, web_icon]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    let spacer = Space::new().width(Length::Fill);

    let tray = system_tray::view(state);

    let bar = row![app_icons, spacer, tray]
        .spacing(12)
        .padding([4, 12])
        .align_y(iced::Alignment::Center);

    container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::dock_bar)
        .into()
}
