//! System tray area: clock, Wi-Fi status, volume, battery.

use iced::widget::{row, text};
use iced::Element;

use crate::app::{DockApp, Message};
use crate::theme::DockColors;

/// Renders the system tray section of the dock (right side).
///
/// Layout: `WiFi | Vol | Bat | HH:MM`
pub fn view(state: &DockApp) -> Element<'_, Message> {
    let wifi_color = if state.wifi_connected {
        DockColors::STATUS_OK
    } else {
        DockColors::STATUS_OFF
    };

    let wifi_label = if state.wifi_connected {
        "WiFi"
    } else {
        "WiFi Off"
    };

    let wifi = text(wifi_label).size(12).color(wifi_color);

    let volume = text(format!("Vol {}%", state.volume_percent))
        .size(12)
        .color(DockColors::TEXT_MUTED);

    let mut items = row![wifi, volume].spacing(12).align_y(iced::Alignment::Center);

    if let Some(bat) = state.battery_percent {
        let bat_color = if bat > 20 {
            DockColors::TEXT
        } else {
            DockColors::STATUS_OFF
        };
        items = items.push(text(format!("Bat {bat}%")).size(12).color(bat_color));
    }

    let clock = text(state.clock.as_str().to_owned())
        .size(14)
        .color(DockColors::TEXT);

    items = items.push(clock);

    items.into()
}
