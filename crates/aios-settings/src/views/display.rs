use iced::widget::{button, column, container, row, text, Space};
use iced::{Element, Length};

use crate::app::{DisplayState, Message};
use crate::theme;

pub fn view(state: &DisplayState) -> Element<'_, Message> {
    let title = text("Display").size(20).color(theme::SettingsColors::TEXT_PRIMARY);

    let refresh_btn = button(text("Refresh").size(13))
        .on_press(Message::DisplayRefresh)
        .padding([6, 14])
        .style(theme::action_button);

    let header = row![title, Space::new().width(Length::Fill), refresh_btn]
        .align_y(iced::Alignment::Center);

    let mut content = column![header].spacing(12).padding(16);

    if state.loading {
        content = content.push(
            text("Loading displays...").size(13).color(theme::SettingsColors::ACCENT),
        );
    } else if state.outputs.is_empty() {
        content = content.push(
            text("No displays found. Click Refresh.")
                .size(13)
                .color(theme::SettingsColors::TEXT_SECONDARY),
        );
    } else {
        for output in &state.outputs {
            let info = format!(
                "{}: {}x{} @ {:.0}Hz (scale {})",
                output.name, output.width, output.height, output.refresh, output.scale
            );
            content = content.push(
                text(info).size(14).color(theme::SettingsColors::TEXT_PRIMARY),
            );

            // Mode selection buttons
            if !output.modes.is_empty() {
                let mut modes_row = row![].spacing(6);
                for mode in &output.modes {
                    let label = format!("{}x{}@{:.0}", mode.width, mode.height, mode.refresh);
                    let is_current = mode.width == output.width
                        && mode.height == output.height
                        && (mode.refresh - output.refresh).abs() < 1.0;
                    modes_row = modes_row.push(
                        button(text(label).size(11))
                            .on_press(Message::DisplaySetMode {
                                output: output.name.clone(),
                                width: mode.width,
                                height: mode.height,
                                refresh: mode.refresh,
                            })
                            .padding([4, 8])
                            .style(if is_current {
                                theme::sidebar_tab_active as fn(&iced::Theme, _) -> _
                            } else {
                                theme::sidebar_tab_inactive
                            }),
                    );
                }
                content = content.push(modes_row);
            }
        }
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
