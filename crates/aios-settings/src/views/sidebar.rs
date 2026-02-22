use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::app::{Message, Tab};
use crate::theme;

pub fn view(active_tab: Tab) -> Element<'static, Message> {
    let tabs = [
        (Tab::Network, "Network"),
        (Tab::Display, "Display"),
        (Tab::Ollama, "Ollama"),
        (Tab::Ai, "AI Provider"),
    ];

    let mut col = column![].spacing(4).padding(8);

    for (tab, label) in tabs {
        let style = if tab == active_tab {
            theme::sidebar_tab_active as fn(&iced::Theme, button::Status) -> button::Style
        } else {
            theme::sidebar_tab_inactive
        };

        col = col.push(
            button(text(label).size(14))
                .on_press(Message::SwitchTab(tab))
                .width(Length::Fill)
                .padding([8, 12])
                .style(style),
        );
    }

    container(col)
        .width(140)
        .height(Length::Fill)
        .style(theme::container_sidebar)
        .into()
}
