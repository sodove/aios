use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

use aios_common::ProviderType;

use crate::app::{Message, OobeState, OobeStep};
use crate::theme::{self, AiosColors};

/// Top-level OOBE view dispatcher -- renders the appropriate step.
pub fn view(state: &OobeState) -> Element<'_, Message> {
    let step_content: Element<'_, Message> = match state.step {
        OobeStep::Welcome => welcome_view(),
        OobeStep::SelectProvider => select_provider_view(),
        OobeStep::EnterApiKey => enter_api_key_view(state),
        OobeStep::OllamaSetup => ollama_setup_view(state),
        OobeStep::OllamaModelSelect => ollama_model_select_view(state),
        OobeStep::Complete => complete_view(state),
    };

    container(step_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::container_oobe_content)
        .into()
}

/// Welcome step -- greeting and start/skip buttons.
fn welcome_view() -> Element<'static, Message> {
    let title = text("AIOS")
        .size(36)
        .color(AiosColors::ACCENT);

    let greeting = text("Привет! Я AIOS -- твой ИИ-ассистент.")
        .size(16)
        .color(AiosColors::TEXT_PRIMARY);

    let description = text(
        "Давай настроим систему.\nДля работы мне нужен доступ к языковой модели (LLM).",
    )
    .size(14)
    .color(AiosColors::TEXT_SECONDARY);

    let start_btn = button(
        text("Начать настройку").size(15),
    )
    .on_press(Message::OobeNext)
    .padding([10, 24])
    .style(theme::send_button);

    let skip_btn = button(
        text("Пропустить").size(13),
    )
    .on_press(Message::OobeSkip)
    .padding([8, 20])
    .style(theme::oobe_secondary_button);

    let content = column![
        title,
        Space::new().height(16),
        greeting,
        Space::new().height(8),
        description,
        Space::new().height(32),
        start_btn,
        Space::new().height(12),
        skip_btn,
    ]
    .align_x(Alignment::Center)
    .max_width(420);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Provider selection step.
fn select_provider_view() -> Element<'static, Message> {
    let heading = text("Выбери провайдера LLM:")
        .size(20)
        .color(AiosColors::TEXT_PRIMARY);

    let claude_card = provider_card(
        "Claude (Anthropic)",
        "claude-sonnet-4-20250514",
        ProviderType::Claude,
    );

    let openai_card = provider_card(
        "ChatGPT (OpenAI)",
        "gpt-4o",
        ProviderType::OpenAi,
    );

    let ollama_card = provider_card(
        "Ollama (локальный)",
        "Без API-ключа, работает локально",
        ProviderType::Ollama,
    );

    let content = column![
        heading,
        Space::new().height(24),
        claude_card,
        Space::new().height(10),
        openai_card,
        Space::new().height(10),
        ollama_card,
    ]
    .align_x(Alignment::Center)
    .max_width(420);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// A single provider card rendered as a clickable button.
fn provider_card(
    name: &str,
    subtitle: &str,
    provider: ProviderType,
) -> Element<'static, Message> {
    let label = text(name.to_owned())
        .size(16)
        .color(AiosColors::TEXT_PRIMARY);

    let sub = text(subtitle.to_owned())
        .size(12)
        .color(AiosColors::TEXT_SECONDARY);

    let inner = column![label, sub].spacing(4).padding(14);

    let card = container(inner)
        .width(Length::Fill)
        .style(theme::container_oobe_card);

    button(card)
        .on_press(Message::OobeSelectProvider(provider))
        .width(Length::Fill)
        .style(theme::oobe_card_button)
        .into()
}

/// API key input step.
fn enter_api_key_view(state: &OobeState) -> Element<'_, Message> {
    let provider_name = match state.selected_provider {
        Some(ProviderType::Claude) => "Claude",
        Some(ProviderType::OpenAi) => "OpenAI",
        _ => "провайдера",
    };

    let heading = text(format!("Введи API-ключ для {provider_name}:"))
        .size(20)
        .color(AiosColors::TEXT_PRIMARY);

    let placeholder = match state.selected_provider {
        Some(ProviderType::Claude) => "sk-ant-...",
        Some(ProviderType::OpenAi) => "sk-...",
        _ => "API key",
    };

    let input = text_input(placeholder, &state.api_key_input)
        .on_input(Message::OobeApiKeyChanged)
        .on_submit(Message::OobeSubmitApiKey)
        .padding(10)
        .size(14)
        .style(theme::input_style);

    let hint = text("Ключ хранится локально в ~/.config/aios/agent.toml")
        .size(12)
        .color(AiosColors::TEXT_SECONDARY);

    let can_submit = !state.api_key_input.trim().is_empty();

    let back_btn = button(text("Назад").size(14))
        .on_press(Message::OobeBack)
        .padding([8, 20])
        .style(theme::oobe_secondary_button);

    let save_btn = button(text("Сохранить").size(14))
        .on_press_maybe(if can_submit {
            Some(Message::OobeSubmitApiKey)
        } else {
            None
        })
        .padding([8, 20])
        .style(theme::send_button);

    let buttons = row![back_btn, Space::new().width(Length::Fill), save_btn]
        .align_y(Alignment::Center);

    let content = column![
        heading,
        Space::new().height(20),
        input,
        Space::new().height(8),
        hint,
        Space::new().height(28),
        buttons,
    ]
    .max_width(420);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Ollama setup step -- shows installation check status.
fn ollama_setup_view(state: &OobeState) -> Element<'_, Message> {
    let heading = text("Ollama Setup")
        .size(22)
        .color(AiosColors::ACCENT);

    let status_msg = state
        .ollama_status
        .as_deref()
        .unwrap_or("Checking Ollama installation...");

    let status = text(status_msg.to_owned())
        .size(14)
        .color(AiosColors::TEXT_SECONDARY);

    let content = column![
        heading,
        Space::new().height(24),
        status,
    ]
    .align_x(Alignment::Center)
    .max_width(420);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Ollama model selection step -- shows fetched models and custom input.
fn ollama_model_select_view(state: &OobeState) -> Element<'_, Message> {
    let heading = text("Выбери модель")
        .size(22)
        .color(AiosColors::ACCENT);

    let mut content = column![heading, Space::new().height(24)]
        .align_x(Alignment::Center)
        .max_width(420)
        .spacing(8);

    if let Some(status_msg) = &state.ollama_status {
        let status = text(status_msg.clone())
            .size(13)
            .color(AiosColors::TEXT_SECONDARY);
        content = content.push(status);
        content = content.push(Space::new().height(8));
    }

    // Show fetched models as cards
    if !state.available_models.is_empty() {
        let subtitle = text("Популярные модели:")
            .size(14)
            .color(AiosColors::TEXT_SECONDARY);
        content = content.push(subtitle);

        for model in &state.available_models {
            let label = text(model.clone())
                .size(15)
                .color(AiosColors::TEXT_PRIMARY);
            let inner = container(label)
                .width(Length::Fill)
                .padding(14)
                .style(theme::container_oobe_card);
            let btn = button(inner)
                .on_press(Message::OobeOllamaSelectModel(model.clone()))
                .width(Length::Fill)
                .style(theme::oobe_card_button);
            content = content.push(btn);
        }

        content = content.push(Space::new().height(12));
    }

    // Custom model input
    let custom_label = text("Или введи имя модели:")
        .size(14)
        .color(AiosColors::TEXT_SECONDARY);

    let custom_input = text_input("например: codellama:7b", &state.custom_model_input)
        .on_input(Message::OobeOllamaCustomModelChanged)
        .padding(10)
        .size(14)
        .style(theme::input_style);

    let can_pull_custom = !state.custom_model_input.trim().is_empty();
    let pull_btn = button(text("Pull").size(14))
        .on_press_maybe(if can_pull_custom {
            Some(Message::OobeOllamaSelectModel(state.custom_model_input.trim().to_owned()))
        } else {
            None
        })
        .padding([8, 20])
        .style(theme::send_button);

    let custom_row = row![custom_input, Space::new().width(8), pull_btn]
        .align_y(Alignment::Center);

    content = content.push(custom_label);
    content = content.push(custom_row);

    // Back button
    let back_btn = button(text("Назад").size(14))
        .on_press(Message::OobeBack)
        .padding([8, 20])
        .style(theme::oobe_secondary_button);

    content = content.push(Space::new().height(16));
    content = content.push(back_btn);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

/// Completion step -- shows the chosen provider and a start button.
fn complete_view(state: &OobeState) -> Element<'static, Message> {
    let checkmark = text("Настройка завершена!")
        .size(22)
        .color(AiosColors::SUCCESS);

    let provider_label = match state.selected_provider {
        Some(ProviderType::Claude) => "Claude",
        Some(ProviderType::OpenAi) => "OpenAI",
        Some(ProviderType::Ollama) => "Ollama",
        None => "по умолчанию",
    };

    let ollama_model_name = state.ollama_model.clone().unwrap_or_else(|| "llama3".to_owned());
    let model_label = match state.selected_provider {
        Some(ProviderType::Claude) => "claude-sonnet-4-20250514".to_owned(),
        Some(ProviderType::OpenAi) => "gpt-4o".to_owned(),
        Some(ProviderType::Ollama) => ollama_model_name,
        None => "claude-sonnet-4-20250514".to_owned(),
    };

    let info = column![
        text(format!("Провайдер: {provider_label}")).size(14).color(AiosColors::TEXT_PRIMARY),
        text(format!("Модель: {model_label}")).size(14).color(AiosColors::TEXT_SECONDARY),
    ]
    .spacing(4);

    let suggestions_header = text("Попробуй попросить меня что-нибудь:")
        .size(14)
        .color(AiosColors::TEXT_PRIMARY);

    let suggestions = column![
        text("  - \"Открой google.com\"").size(13).color(AiosColors::TEXT_SECONDARY),
        text("  - \"Покажи содержимое /home\"").size(13).color(AiosColors::TEXT_SECONDARY),
        text("  - \"Какая сейчас погода?\"").size(13).color(AiosColors::TEXT_SECONDARY),
    ]
    .spacing(2);

    let start_btn = button(text("Начать общение").size(15))
        .on_press(Message::OobeComplete)
        .padding([10, 24])
        .style(theme::send_button);

    let content = column![
        checkmark,
        Space::new().height(20),
        info,
        Space::new().height(24),
        suggestions_header,
        Space::new().height(8),
        suggestions,
        Space::new().height(32),
        start_btn,
    ]
    .align_x(Alignment::Center)
    .max_width(420);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
