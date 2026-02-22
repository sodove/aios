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

    let model_label = match state.selected_provider {
        Some(ProviderType::Claude) => "claude-sonnet-4-20250514",
        Some(ProviderType::OpenAi) => "gpt-4o",
        Some(ProviderType::Ollama) => "llama3",
        None => "claude-sonnet-4-20250514",
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
