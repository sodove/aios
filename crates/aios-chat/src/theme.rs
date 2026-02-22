use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Vector};

/// Dark theme color palette for AIOS Chat.
pub struct AiosColors;

impl AiosColors {
    pub const BG_PRIMARY: Color = Color::from_rgb(0.10, 0.11, 0.15);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.13, 0.14, 0.18);
    pub const BG_INPUT: Color = Color::from_rgb(0.16, 0.17, 0.22);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.87, 0.89, 0.93);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
    pub const ACCENT: Color = Color::from_rgb(0.47, 0.56, 1.0);
    pub const USER_BUBBLE: Color = Color::from_rgb(0.20, 0.25, 0.45);
    pub const ASSISTANT_BUBBLE: Color = Color::from_rgb(0.15, 0.16, 0.20);

    // -- OOBE wizard colors --

    /// Background for OOBE provider selection cards.
    pub const OOBE_CARD_BG: Color = Color::from_rgb(0.16, 0.17, 0.22);
    /// Border for OOBE provider selection cards (idle).
    pub const OOBE_CARD_BORDER: Color = Color::from_rgb(0.25, 0.27, 0.35);
    /// Success indicator color.
    pub const SUCCESS: Color = Color::from_rgb(0.30, 0.78, 0.48);

    // -- Tool card colors --

    /// Background for pending tool calls (dark amber tint).
    pub const TOOL_PENDING_BG: Color = Color::from_rgb(0.22, 0.20, 0.12);
    /// Border for pending tool calls (amber).
    pub const TOOL_PENDING_BORDER: Color = Color::from_rgb(0.75, 0.65, 0.20);

    /// Background for completed tool results (dark green tint).
    pub const TOOL_COMPLETED_BG: Color = Color::from_rgb(0.12, 0.20, 0.14);
    /// Border for completed tool results (green).
    pub const TOOL_COMPLETED_BORDER: Color = Color::from_rgb(0.30, 0.70, 0.40);

    /// Background for failed/rejected tool results (dark red tint).
    pub const TOOL_FAILED_BG: Color = Color::from_rgb(0.22, 0.12, 0.12);
    /// Border for failed/rejected tool results (red).
    pub const TOOL_FAILED_BORDER: Color = Color::from_rgb(0.80, 0.30, 0.30);
}

// ---------------------------------------------------------------------------
// Container styles
// ---------------------------------------------------------------------------

/// Primary background for the root container.
pub fn container_primary(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::BG_PRIMARY)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

/// Secondary background for the header and input bar areas.
pub fn container_secondary(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::BG_SECONDARY)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

/// User message bubble background.
pub fn container_user_bubble(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::USER_BUBBLE)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 12.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Assistant message bubble background.
pub fn container_assistant_bubble(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::ASSISTANT_BUBBLE)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 12.0.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Tool card in `Pending` state (amber border, dark amber background).
pub fn container_tool_pending(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::TOOL_PENDING_BG)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.5,
            color: AiosColors::TOOL_PENDING_BORDER,
        },
        ..container::Style::default()
    }
}

/// Tool card in `Completed` state (green border, dark green background).
pub fn container_tool_completed(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::TOOL_COMPLETED_BG)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.5,
            color: AiosColors::TOOL_COMPLETED_BORDER,
        },
        ..container::Style::default()
    }
}

/// Tool card in `Failed` or `Rejected` state (red border, dark red background).
pub fn container_tool_failed(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::TOOL_FAILED_BG)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.5,
            color: AiosColors::TOOL_FAILED_BORDER,
        },
        ..container::Style::default()
    }
}

// ---------------------------------------------------------------------------
// OOBE styles
// ---------------------------------------------------------------------------

/// Container style for an OOBE provider card (unselected).
pub fn container_oobe_card(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::OOBE_CARD_BG)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        border: Border {
            radius: 10.0.into(),
            width: 1.5,
            color: AiosColors::OOBE_CARD_BORDER,
        },
        ..container::Style::default()
    }
}

/// Centered OOBE content area container.
pub fn container_oobe_content(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(AiosColors::BG_PRIMARY)),
        text_color: Some(AiosColors::TEXT_PRIMARY),
        ..container::Style::default()
    }
}

/// Secondary (outline) button used for "Skip" and "Back" actions.
pub fn oobe_secondary_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: AiosColors::TEXT_SECONDARY,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: AiosColors::OOBE_CARD_BORDER,
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            text_color: AiosColors::TEXT_PRIMARY,
            border: Border {
                color: AiosColors::TEXT_PRIMARY,
                ..base.border
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
            text_color: AiosColors::TEXT_PRIMARY,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Color::from_rgba(0.55, 0.58, 0.65, 0.4),
            ..base
        },
    }
}

/// Provider card button style (transparent, no border -- the container handles visuals).
pub fn oobe_card_button(_theme: &iced::Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: AiosColors::TEXT_PRIMARY,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: true,
    }
}

// ---------------------------------------------------------------------------
// Text input style
// ---------------------------------------------------------------------------

/// Custom style for the message input field.
pub fn input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(AiosColors::BG_INPUT),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
        },
        icon: AiosColors::TEXT_SECONDARY,
        placeholder: AiosColors::TEXT_SECONDARY,
        value: AiosColors::TEXT_PRIMARY,
        selection: AiosColors::ACCENT,
    };

    match status {
        text_input::Status::Active | text_input::Status::Disabled => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: AiosColors::ACCENT,
                width: 1.5,
                ..base.border
            },
            ..base
        },
    }
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

/// Send button style.
pub fn send_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(AiosColors::ACCENT)),
        text_color: Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.55, 0.64, 1.0))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.38, 0.47, 0.90))),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.47, 0.56, 1.0, 0.4))),
            text_color: Color::from_rgba(1.0, 1.0, 1.0, 0.4),
            ..base
        },
    }
}

/// Close button style — transparent background, red hover highlight.
pub fn close_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: AiosColors::TEXT_SECONDARY,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.85, 0.30, 0.30, 0.3))),
            text_color: Color::from_rgb(0.95, 0.40, 0.40),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color::from_rgba(0.85, 0.30, 0.30, 0.5))),
            text_color: Color::WHITE,
            ..base
        },
        button::Status::Disabled => base,
    }
}

// ---------------------------------------------------------------------------
// Scrollable style
// ---------------------------------------------------------------------------

/// Dark scrollable style matching the primary background.
pub fn scrollable_dark(_theme: &iced::Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_border = Border {
        radius: 4.0.into(),
        ..Border::default()
    };

    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.15)),
            border: scroller_border,
        },
    };

    let auto_scroll = scrollable::AutoScroll {
        background: Background::Color(AiosColors::BG_SECONDARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::ZERO,
            blur_radius: 2.0,
        },
        icon: AiosColors::TEXT_SECONDARY,
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll,
        },
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            ..
        } => {
            let hovered_rail = scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.30)),
                    ..rail.scroller
                },
                ..rail
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: if is_vertical_scrollbar_hovered {
                    hovered_rail
                } else {
                    rail
                },
                horizontal_rail: if is_horizontal_scrollbar_hovered {
                    hovered_rail
                } else {
                    rail
                },
                gap: None,
                auto_scroll,
            }
        }
        scrollable::Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            ..
        } => {
            let dragged_rail = scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: Background::Color(AiosColors::ACCENT),
                    ..rail.scroller
                },
                ..rail
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: if is_vertical_scrollbar_dragged {
                    dragged_rail
                } else {
                    rail
                },
                horizontal_rail: if is_horizontal_scrollbar_dragged {
                    dragged_rail
                } else {
                    rail
                },
                gap: None,
                auto_scroll,
            }
        }
    }
}
