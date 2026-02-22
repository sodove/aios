//! Dark theme styles for AIOS Settings -- matches dock/chat Tokyo Night palette.

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color};

pub struct SettingsColors;

impl SettingsColors {
    pub const BG_PRIMARY: Color = Color::from_rgb(0.10, 0.11, 0.15);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.13, 0.14, 0.18);
    pub const BG_SIDEBAR: Color = Color::from_rgb(0.08, 0.09, 0.12);
    pub const BG_INPUT: Color = Color::from_rgb(0.16, 0.17, 0.22);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.87, 0.89, 0.93);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
    pub const ACCENT: Color = Color::from_rgb(0.47, 0.56, 1.0);
    pub const SUCCESS: Color = Color::from_rgb(0.30, 0.78, 0.47);
    pub const DANGER: Color = Color::from_rgb(0.85, 0.30, 0.30);
    pub const SIDEBAR_ACTIVE: Color = Color::from_rgba(0.47, 0.56, 1.0, 0.15);
    pub const SIDEBAR_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);
}

pub fn container_primary(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SettingsColors::BG_PRIMARY)),
        text_color: Some(SettingsColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

pub fn container_secondary(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SettingsColors::BG_SECONDARY)),
        text_color: Some(SettingsColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

pub fn container_sidebar(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SettingsColors::BG_SIDEBAR)),
        text_color: Some(SettingsColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

pub fn sidebar_tab_active(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(SettingsColors::SIDEBAR_ACTIVE)),
        text_color: SettingsColors::ACCENT,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

pub fn sidebar_tab_inactive(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: SettingsColors::TEXT_SECONDARY,
        border: Border {
            radius: 6.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(SettingsColors::SIDEBAR_HOVER)),
            text_color: SettingsColors::TEXT_PRIMARY,
            ..base
        },
        _ => base,
    }
}

pub fn action_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(SettingsColors::ACCENT)),
        text_color: Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.55, 0.64, 1.0))),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.47, 0.56, 1.0, 0.4))),
            text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.4),
            ..base
        },
        _ => base,
    }
}

pub fn danger_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(SettingsColors::DANGER)),
        text_color: Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.40, 0.40))),
            ..base
        },
        _ => base,
    }
}

pub fn close_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: SettingsColors::TEXT_SECONDARY,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.85, 0.30, 0.30, 0.3))),
            text_color: Color::from_rgb(0.95, 0.40, 0.40),
            ..base
        },
        _ => base,
    }
}

pub fn input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(SettingsColors::BG_INPUT),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
        },
        icon: SettingsColors::TEXT_SECONDARY,
        placeholder: SettingsColors::TEXT_SECONDARY,
        value: SettingsColors::TEXT_PRIMARY,
        selection: SettingsColors::ACCENT,
    };
    match status {
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: SettingsColors::ACCENT,
                width: 1.5,
                ..base.border
            },
            ..base
        },
        _ => base,
    }
}
