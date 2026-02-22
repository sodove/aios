use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};

use crate::app::{Message, NetworkState};
use crate::theme;

pub fn view(state: &NetworkState) -> Element<'_, Message> {
    let title = text("Network").size(20).color(theme::SettingsColors::TEXT_PRIMARY);

    let scan_btn = button(text("Scan").size(13))
        .on_press(Message::WifiScan)
        .padding([6, 14])
        .style(theme::action_button);

    let header = row![title, Space::new().width(Length::Fill), scan_btn]
        .align_y(iced::Alignment::Center);

    let mut content = column![header].spacing(12).padding(16);

    // Status line
    if !state.status.is_empty() {
        content = content.push(
            text(&state.status).size(12).color(theme::SettingsColors::TEXT_SECONDARY)
        );
    }

    // Network list
    if state.networks.is_empty() && !state.loading {
        content = content.push(
            text("No networks found. Click Scan to search.")
                .size(13)
                .color(theme::SettingsColors::TEXT_SECONDARY),
        );
    } else if state.loading {
        content = content.push(
            text("Scanning...").size(13).color(theme::SettingsColors::ACCENT),
        );
    } else {
        let mut list = column![].spacing(6);
        for net in &state.networks {
            let in_use = if net.connected { " *" } else { "" };
            let label = format!("{}  {}%  {}{}", net.ssid, net.signal, net.security, in_use);
            let net_row = button(text(label).size(13))
                .on_press(Message::SelectNetwork(net.ssid.clone()))
                .width(Length::Fill)
                .padding([8, 12])
                .style(if Some(&net.ssid) == state.selected_ssid.as_ref() {
                    theme::sidebar_tab_active as fn(&iced::Theme, _) -> _
                } else {
                    theme::sidebar_tab_inactive
                });
            list = list.push(net_row);
        }
        content = content.push(scrollable(list).height(Length::Fill));
    }

    // Password input + connect/disconnect buttons
    if let Some(ssid) = &state.selected_ssid {
        let mut action_row = row![].spacing(8).align_y(iced::Alignment::Center);

        // Only show password input for secured networks
        let selected_net = state.networks.iter().find(|n| &n.ssid == ssid);
        let is_secured = selected_net.map_or(false, |n| n.security != "--" && !n.security.is_empty());
        let is_connected = selected_net.map_or(false, |n| n.connected);

        if is_secured && !is_connected {
            let pwd_input = text_input("Password...", &state.password_input)
                .on_input(Message::PasswordChanged)
                .on_submit(Message::WifiConnect)
                .secure(true)
                .padding(8)
                .size(13)
                .width(200)
                .style(theme::input_style);
            action_row = action_row.push(pwd_input);
        }

        if is_connected {
            action_row = action_row.push(
                button(text("Disconnect").size(13))
                    .on_press(Message::WifiDisconnect)
                    .padding([6, 14])
                    .style(theme::danger_button),
            );
        } else {
            action_row = action_row.push(
                button(text("Connect").size(13))
                    .on_press(Message::WifiConnect)
                    .padding([6, 14])
                    .style(theme::action_button),
            );
        }

        content = content.push(action_row);
    }

    // Error display
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
