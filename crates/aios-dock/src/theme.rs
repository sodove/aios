//! Dark theme styles for the AIOS Dock panel.

use iced::widget::{button, container};
use iced::{Background, Border, Color};

/// Color palette for the dock panel.
pub struct DockColors;

impl DockColors {
    /// Semi-transparent dark background for the dock bar.
    pub const DOCK_BG: Color = Color::from_rgba(0.10, 0.11, 0.15, 0.95);

    /// Subtle background for app icon buttons (idle state).
    pub const ICON_BG: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);

    /// Highlighted background for app icon buttons (hovered state).
    pub const ICON_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);

    /// Pressed state background for app icon buttons.
    pub const ICON_PRESSED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.22);

    /// Primary text color (light gray).
    pub const TEXT: Color = Color::from_rgb(0.87, 0.89, 0.93);

    /// Secondary/muted text color.
    pub const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.58, 0.65);

    /// Accent color for active/connected indicators (reserved for future use).
    #[allow(dead_code)]
    pub const ACCENT: Color = Color::from_rgb(0.47, 0.56, 1.0);

    /// Green indicator (e.g. Wi-Fi connected).
    pub const STATUS_OK: Color = Color::from_rgb(0.30, 0.78, 0.47);

    /// Gray indicator (e.g. Wi-Fi disconnected).
    pub const STATUS_OFF: Color = Color::from_rgb(0.45, 0.47, 0.52);
}

// ---------------------------------------------------------------------------
// Container styles
// ---------------------------------------------------------------------------

/// Style for the root dock bar container.
pub fn dock_bar(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(DockColors::DOCK_BG)),
        text_color: Some(DockColors::TEXT),
        ..container::Style::default()
    }
}

// ---------------------------------------------------------------------------
// Button styles
// ---------------------------------------------------------------------------

/// Style for app icon buttons in the dock.
pub fn app_icon_button(_theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(DockColors::ICON_BG)),
        text_color: DockColors::TEXT,
        border: Border {
            radius: 8.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(DockColors::ICON_HOVER)),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(DockColors::ICON_PRESSED)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(DockColors::ICON_BG)),
            text_color: DockColors::TEXT_MUTED,
            ..base
        },
    }
}
