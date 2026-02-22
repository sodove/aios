use aios_common::TrustLevel;
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color};

/// Visual constants for the AIOS Confirm dialog.
///
/// Dark theme palette designed for high-contrast confirmation dialogs
/// that demand user attention and convey trust level through color.
pub struct ConfirmTheme;

impl ConfirmTheme {
    // -- Base palette --

    pub const BG: Color = Color::from_rgb(0.12, 0.13, 0.17);
    pub const TEXT: Color = Color::from_rgb(0.87, 0.89, 0.93);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.57, 0.63);
    pub const WARNING: Color = Color::from_rgb(1.0, 0.76, 0.03);
    pub const DANGER: Color = Color::from_rgb(0.91, 0.30, 0.24);
    pub const APPROVE: Color = Color::from_rgb(0.47, 0.56, 1.0);
    pub const CANCEL: Color = Color::from_rgb(0.35, 0.36, 0.42);
    pub const COMMAND_BG: Color = Color::from_rgb(0.08, 0.08, 0.12);

    // -- Trust level colors --

    pub const TRUST_USER: Color = Color::from_rgb(0.30, 0.69, 0.31);
    pub const TRUST_SYSTEM: Color = Color::from_rgb(0.26, 0.54, 0.90);
    pub const TRUST_WEB: Color = Color::from_rgb(0.91, 0.30, 0.24);
    pub const TRUST_MEMORY: Color = Color::from_rgb(1.0, 0.76, 0.03);

    /// Returns the color associated with the given trust level.
    pub fn trust_color(trust: &TrustLevel) -> Color {
        match trust {
            TrustLevel::User => Self::TRUST_USER,
            TrustLevel::System => Self::TRUST_SYSTEM,
            TrustLevel::WebContent => Self::TRUST_WEB,
            TrustLevel::Memory => Self::TRUST_MEMORY,
        }
    }

    /// Returns a human-readable label for the trust level.
    pub fn trust_label(trust: &TrustLevel) -> &'static str {
        match trust {
            TrustLevel::User => "User",
            TrustLevel::System => "System",
            TrustLevel::WebContent => "WebContent",
            TrustLevel::Memory => "Memory",
        }
    }
}

// ---------------------------------------------------------------------------
// Container styles
// ---------------------------------------------------------------------------

/// Dark background container for the main window area.
pub fn dark_container(_theme: &iced::Theme) -> container::Style {
    container::Style::default()
        .background(Background::Color(ConfirmTheme::BG))
        .color(ConfirmTheme::TEXT)
}

/// Container styled as a code block for displaying commands.
pub fn command_container(_theme: &iced::Theme) -> container::Style {
    container::Style::default()
        .background(Background::Color(ConfirmTheme::COMMAND_BG))
        .color(ConfirmTheme::TEXT)
        .border(Border {
            color: ConfirmTheme::CANCEL,
            width: 1.0,
            radius: 4.0.into(),
        })
}

/// Container with a colored left border for trust level indication.
pub fn trust_badge_container(trust: &TrustLevel) -> impl Fn(&iced::Theme) -> container::Style {
    let color = ConfirmTheme::trust_color(trust);
    move |_theme: &iced::Theme| {
        container::Style::default()
            .background(Background::Color(Color {
                a: 0.15,
                ..color
            }))
            .border(Border {
                color,
                width: 2.0,
                radius: 4.0.into(),
            })
    }
}

/// Container with a red danger border for critical dialogs.
pub fn danger_container(_theme: &iced::Theme) -> container::Style {
    container::Style::default()
        .background(Background::Color(Color {
            a: 0.10,
            ..ConfirmTheme::DANGER
        }))
        .border(Border {
            color: ConfirmTheme::DANGER,
            width: 1.0,
            radius: 4.0.into(),
        })
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

/// Approve / allow button style (accent blue).
pub fn approve_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(ConfirmTheme::APPROVE)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(lighten(ConfirmTheme::APPROVE, 0.15))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(darken(ConfirmTheme::APPROVE, 0.10))),
            ..base
        },
        _ => base,
    }
}

/// Cancel / reject button style (gray).
pub fn cancel_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(ConfirmTheme::CANCEL)),
        text_color: ConfirmTheme::TEXT,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(lighten(ConfirmTheme::CANCEL, 0.10))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(darken(ConfirmTheme::CANCEL, 0.10))),
            ..base
        },
        _ => base,
    }
}

/// Danger / confirm-destructive button style (red, used in critical dialog).
pub fn danger_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(ConfirmTheme::DANGER)),
        text_color: Color::WHITE,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(lighten(ConfirmTheme::DANGER, 0.15))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(darken(ConfirmTheme::DANGER, 0.10))),
            ..base
        },
        _ => base,
    }
}

/// Disabled button style (dimmed, non-interactive appearance).
pub fn disabled_button(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.20, 0.21, 0.25))),
        text_color: ConfirmTheme::TEXT_MUTED,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    }
}

/// Simulate / debug button style (subtle outline).
pub fn simulate_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: ConfirmTheme::TEXT_MUTED,
        border: Border {
            color: ConfirmTheme::CANCEL,
            width: 1.0,
            radius: 6.0.into(),
        },
        shadow: iced::Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
            text_color: ConfirmTheme::TEXT,
            ..base
        },
        _ => base,
    }
}

// ---------------------------------------------------------------------------
// Text input styles
// ---------------------------------------------------------------------------

/// Style for the "type DELETE" confirmation input.
pub fn confirm_input(_theme: &iced::Theme, _status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(ConfirmTheme::COMMAND_BG),
        border: Border {
            color: ConfirmTheme::DANGER,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: ConfirmTheme::TEXT_MUTED,
        placeholder: ConfirmTheme::TEXT_MUTED,
        value: ConfirmTheme::TEXT,
        selection: ConfirmTheme::APPROVE,
    }
}

// ---------------------------------------------------------------------------
// Color utilities
// ---------------------------------------------------------------------------

/// Lighten a color by a relative factor (0.0 = unchanged, 1.0 = white).
fn lighten(color: Color, factor: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * factor,
        g: color.g + (1.0 - color.g) * factor,
        b: color.b + (1.0 - color.b) * factor,
        a: color.a,
    }
}

/// Darken a color by a relative factor (0.0 = unchanged, 1.0 = black).
fn darken(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * (1.0 - factor),
        g: color.g * (1.0 - factor),
        b: color.b * (1.0 - factor),
        a: color.a,
    }
}
