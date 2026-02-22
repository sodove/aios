use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Element, Length};

use crate::app::{Message, OllamaState};
use crate::theme;

pub fn view(state: &OllamaState) -> Element<'_, Message> {
    let title = text("Ollama").size(20).color(theme::SettingsColors::TEXT_PRIMARY);

    let status_color = if state.running {
        theme::SettingsColors::SUCCESS
    } else {
        theme::SettingsColors::DANGER
    };
    let status_text = if state.running { "Running" } else { "Stopped" };

    let header = row![
        title,
        Space::new().width(8),
        text(status_text).size(12).color(status_color),
        Space::new().width(Length::Fill),
        button(text("Refresh").size(13))
            .on_press(Message::OllamaRefresh)
            .padding([6, 14])
            .style(theme::action_button),
        Space::new().width(4),
        button(text(if state.running { "Stop" } else { "Start" }).size(13))
            .on_press(if state.running { Message::OllamaStop } else { Message::OllamaStart })
            .padding([6, 14])
            .style(if state.running { theme::danger_button as fn(&iced::Theme, _) -> _ } else { theme::action_button }),
    ]
    .align_y(iced::Alignment::Center);

    let mut content = column![header].spacing(12).padding(16);

    // Installed models
    content = content.push(
        text("Installed Models").size(16).color(theme::SettingsColors::TEXT_PRIMARY),
    );

    if state.models.is_empty() {
        content = content.push(
            text("No models installed.")
                .size(13)
                .color(theme::SettingsColors::TEXT_SECONDARY),
        );
    } else {
        let mut list = column![].spacing(4);
        for model in &state.models {
            let model_row = row![
                text(model).size(13).color(theme::SettingsColors::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                button(text("Remove").size(11))
                    .on_press(Message::OllamaRemove(model.clone()))
                    .padding([4, 8])
                    .style(theme::danger_button),
            ]
            .align_y(iced::Alignment::Center);
            list = list.push(model_row);
        }
        content = content.push(scrollable(list).height(Length::Fill));
    }

    // Pull popular models
    content = content.push(
        text("Pull Model").size(16).color(theme::SettingsColors::TEXT_PRIMARY),
    );

    let popular = ["llama3", "mistral", "phi3", "gemma2"];
    let mut pull_row = row![].spacing(6);
    for model in popular {
        let already_installed = state.models.iter().any(|m| m.starts_with(model));
        let btn = button(text(model).size(12))
            .padding([6, 12])
            .style(if already_installed {
                theme::sidebar_tab_active as fn(&iced::Theme, _) -> _
            } else {
                theme::action_button
            });

        pull_row = pull_row.push(if already_installed {
            btn
        } else {
            btn.on_press(Message::OllamaPull(model.to_owned()))
        });
    }
    content = content.push(pull_row);

    // Status/progress message
    if let Some(msg) = &state.progress {
        content = content.push(
            text(msg).size(12).color(theme::SettingsColors::ACCENT),
        );
    }

    if let Some(err) = &state.error {
        content = content.push(
            text(err).size(12).color(theme::SettingsColors::DANGER),
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::container_primary)
        .into()
}
