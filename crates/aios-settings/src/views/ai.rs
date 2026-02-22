use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length};

use crate::app::{AiState, Message};
use crate::theme;

pub fn view(state: &AiState) -> Element<'_, Message> {
    let title = text("AI Provider").size(20).color(theme::SettingsColors::TEXT_PRIMARY);

    let header = row![title].align_y(iced::Alignment::Center);

    let mut content = column![header].spacing(16).padding(16);

    // Provider selection buttons
    content = content.push(
        text("Provider").size(14).color(theme::SettingsColors::TEXT_SECONDARY),
    );

    let providers = [("ollama", "Ollama"), ("open_ai", "OpenAI"), ("claude", "Claude")];
    let mut provider_row = row![].spacing(8);
    for (id, label) in providers {
        let is_active = state.provider == id;
        let style = if is_active {
            theme::sidebar_tab_active as fn(&iced::Theme, _) -> _
        } else {
            theme::action_button
        };
        let btn = button(text(label).size(13))
            .padding([8, 16])
            .style(style);
        provider_row = provider_row.push(if is_active {
            btn
        } else {
            btn.on_press(Message::AiSelectProvider(id.to_owned()))
        });
    }
    content = content.push(provider_row);

    // API Key (hidden for Ollama, shown for OpenAI/Claude)
    if state.provider != "ollama" {
        content = content.push(
            text("API Key").size(14).color(theme::SettingsColors::TEXT_SECONDARY),
        );
        content = content.push(
            text_input("sk-...", &state.api_key)
                .on_input(Message::AiApiKeyChanged)
                .padding(10)
                .size(13)
                .secure(true),
        );
    }

    // Model
    content = content.push(
        text("Model").size(14).color(theme::SettingsColors::TEXT_SECONDARY),
    );

    // Show installed Ollama models as clickable cards
    if state.provider == "ollama" && !state.installed_models.is_empty() {
        let mut model_row = row![].spacing(6);
        let mut count = 0;
        let mut model_col = column![].spacing(6);

        for model_name in &state.installed_models {
            let is_selected = state.model == *model_name;
            let style = if is_selected {
                theme::sidebar_tab_active as fn(&iced::Theme, _) -> _
            } else {
                theme::action_button
            };
            let btn = button(text(model_name.clone()).size(12))
                .padding([6, 12])
                .style(style)
                .on_press(Message::AiPickModel(model_name.clone()));
            model_row = model_row.push(btn);
            count += 1;
            if count % 3 == 0 {
                model_col = model_col.push(model_row);
                model_row = row![].spacing(6);
            }
        }
        if count % 3 != 0 {
            model_col = model_col.push(model_row);
        }
        content = content.push(model_col);

        content = content.push(
            text("Или введи вручную:").size(12).color(theme::SettingsColors::TEXT_SECONDARY),
        );
    }

    let model_placeholder = match state.provider.as_str() {
        "ollama" => "llama3.2:3b",
        "open_ai" => "gpt-4o",
        "claude" => "claude-sonnet-4-20250514",
        _ => "model name",
    };

    content = content.push(
        text_input(model_placeholder, &state.model)
            .on_input(Message::AiModelChanged)
            .padding(10)
            .size(13),
    );

    // Base URL (optional, mainly for Ollama custom host or OpenAI-compatible)
    content = content.push(
        text("Base URL (optional)").size(14).color(theme::SettingsColors::TEXT_SECONDARY),
    );

    let url_placeholder = match state.provider.as_str() {
        "ollama" => "http://localhost:11434",
        "open_ai" => "https://api.openai.com/v1",
        "claude" => "https://api.anthropic.com",
        _ => "",
    };

    content = content.push(
        text_input(url_placeholder, &state.base_url)
            .on_input(Message::AiBaseUrlChanged)
            .padding(10)
            .size(13),
    );

    // Save button
    content = content.push(Space::new().height(8));

    let save_btn = button(text("Save").size(14))
        .padding([10, 24])
        .style(theme::action_button)
        .on_press(Message::AiSave);

    let mut save_row = row![save_btn].spacing(12).align_y(iced::Alignment::Center);

    if state.saved {
        save_row = save_row.push(
            text("Saved & applied!")
                .size(12)
                .color(theme::SettingsColors::SUCCESS),
        );
    }

    content = content.push(save_row);

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
