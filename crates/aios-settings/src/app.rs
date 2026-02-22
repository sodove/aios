use iced::{Element, Task};

use crate::commands;
use crate::theme;
use crate::views::{display, network, ollama, sidebar};

/// Active settings tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Network,
    Display,
    Ollama,
}

/// Wi-Fi network entry parsed from nmcli output.
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub connected: bool,
}

/// Display output info parsed from swaymsg.
#[derive(Debug, Clone)]
pub struct DisplayOutput {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh: f32,
    pub scale: f32,
    pub modes: Vec<DisplayMode>,
}

#[derive(Debug, Clone)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh: f32,
}

/// State for Network tab.
#[derive(Debug, Default)]
pub struct NetworkState {
    pub networks: Vec<WifiNetwork>,
    pub selected_ssid: Option<String>,
    pub password_input: String,
    pub status: String,
    pub loading: bool,
    pub error: Option<String>,
}

/// State for Display tab.
#[derive(Debug, Default)]
pub struct DisplayState {
    pub outputs: Vec<DisplayOutput>,
    pub loading: bool,
    pub error: Option<String>,
}

/// State for Ollama tab.
#[derive(Debug, Default)]
pub struct OllamaState {
    pub running: bool,
    pub models: Vec<String>,
    pub progress: Option<String>,
    pub error: Option<String>,
}

/// All messages the settings UI can produce.
#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    CloseWindow,

    // Network
    WifiScan,
    WifiScanDone(Vec<WifiNetwork>, String),
    SelectNetwork(String),
    PasswordChanged(String),
    WifiConnect,
    WifiDisconnect,
    WifiActionDone(bool, String),

    // Display
    DisplayRefresh,
    DisplayRefreshDone(Vec<DisplayOutput>),
    DisplaySetMode { output: String, width: u32, height: u32, refresh: f32 },
    DisplayActionDone(bool, String),

    // Ollama
    OllamaRefresh,
    OllamaRefreshDone { running: bool, models: Vec<String> },
    OllamaStart,
    OllamaStop,
    OllamaPull(String),
    OllamaRemove(String),
    OllamaActionDone(bool, String),
}

pub struct SettingsApp {
    pub active_tab: Tab,
    pub network: NetworkState,
    pub display: DisplayState,
    pub ollama: OllamaState,
}

impl SettingsApp {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            active_tab: Tab::Network,
            network: NetworkState::default(),
            display: DisplayState::default(),
            ollama: OllamaState::default(),
        };
        // Auto-refresh on start
        let tasks = Task::batch([
            Task::perform(async { do_wifi_scan() }, |(nets, status)| Message::WifiScanDone(nets, status)),
            Task::perform(async { do_display_refresh() }, Message::DisplayRefreshDone),
            Task::perform(async { do_ollama_refresh() }, |(running, models)| {
                Message::OllamaRefreshDone { running, models }
            }),
        ]);
        (state, tasks)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
            }
            Message::CloseWindow => {
                return iced::exit();
            }

            // -- Network --
            Message::WifiScan => {
                self.network.loading = true;
                self.network.error = None;
                return Task::perform(async { do_wifi_scan() }, |(nets, status)| {
                    Message::WifiScanDone(nets, status)
                });
            }
            Message::WifiScanDone(networks, status) => {
                self.network.loading = false;
                self.network.networks = networks;
                self.network.status = status;
            }
            Message::SelectNetwork(ssid) => {
                self.network.selected_ssid = Some(ssid);
                self.network.password_input.clear();
            }
            Message::PasswordChanged(val) => {
                self.network.password_input = val;
            }
            Message::WifiConnect => {
                if let Some(ssid) = self.network.selected_ssid.clone() {
                    let password = self.network.password_input.clone();
                    return Task::perform(
                        async move {
                            let r = commands::wifi_connect(&ssid, &password);
                            (r.success, r.output)
                        },
                        |(ok, msg)| Message::WifiActionDone(ok, msg),
                    );
                }
            }
            Message::WifiDisconnect => {
                return Task::perform(
                    async {
                        let r = commands::wifi_disconnect();
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::WifiActionDone(ok, msg),
                );
            }
            Message::WifiActionDone(success, msg) => {
                if success {
                    self.network.error = None;
                    self.network.status = msg;
                    // Refresh list after action
                    return Task::perform(async { do_wifi_scan() }, |(nets, status)| {
                        Message::WifiScanDone(nets, status)
                    });
                } else {
                    self.network.error = Some(msg);
                }
            }

            // -- Display --
            Message::DisplayRefresh => {
                self.display.loading = true;
                self.display.error = None;
                return Task::perform(async { do_display_refresh() }, Message::DisplayRefreshDone);
            }
            Message::DisplayRefreshDone(outputs) => {
                self.display.loading = false;
                self.display.outputs = outputs;
            }
            Message::DisplaySetMode { output, width, height, refresh } => {
                return Task::perform(
                    async move {
                        let r = commands::display_set_mode(&output, width, height, refresh);
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::DisplayActionDone(ok, msg),
                );
            }
            Message::DisplayActionDone(success, msg) => {
                if success {
                    self.display.error = None;
                    return Task::perform(async { do_display_refresh() }, Message::DisplayRefreshDone);
                } else {
                    self.display.error = Some(msg);
                }
            }

            // -- Ollama --
            Message::OllamaRefresh => {
                return Task::perform(async { do_ollama_refresh() }, |(running, models)| {
                    Message::OllamaRefreshDone { running, models }
                });
            }
            Message::OllamaRefreshDone { running, models } => {
                self.ollama.running = running;
                self.ollama.models = models;
                self.ollama.progress = None;
            }
            Message::OllamaStart => {
                self.ollama.progress = Some("Starting Ollama...".to_owned());
                return Task::perform(
                    async {
                        let r = commands::ollama_start();
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::OllamaActionDone(ok, msg),
                );
            }
            Message::OllamaStop => {
                self.ollama.progress = Some("Stopping Ollama...".to_owned());
                return Task::perform(
                    async {
                        let r = commands::ollama_stop();
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::OllamaActionDone(ok, msg),
                );
            }
            Message::OllamaPull(model) => {
                self.ollama.progress = Some(format!("Pulling {model}..."));
                self.ollama.error = None;
                return Task::perform(
                    async move {
                        let r = commands::ollama_pull(&model);
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::OllamaActionDone(ok, msg),
                );
            }
            Message::OllamaRemove(model) => {
                self.ollama.progress = Some(format!("Removing {model}..."));
                return Task::perform(
                    async move {
                        let r = commands::ollama_remove(&model);
                        (r.success, r.output)
                    },
                    |(ok, msg)| Message::OllamaActionDone(ok, msg),
                );
            }
            Message::OllamaActionDone(success, msg) => {
                if success {
                    self.ollama.error = None;
                    return Task::perform(async { do_ollama_refresh() }, |(running, models)| {
                        Message::OllamaRefreshDone { running, models }
                    });
                } else {
                    self.ollama.error = Some(msg);
                    self.ollama.progress = None;
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, row, text, Space};
        use iced::Length;

        // Title bar with close button
        let title_bar = {
            let title = text("AIOS Settings").size(18).color(theme::SettingsColors::TEXT_PRIMARY);
            let close_btn = button(text("X").size(14).color(theme::SettingsColors::TEXT_SECONDARY))
                .on_press(Message::CloseWindow)
                .padding([4, 10])
                .style(theme::close_button);
            container(
                row![title, Space::new().width(Length::Fill), close_btn]
                    .align_y(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .padding(12)
            .style(theme::container_secondary)
        };

        let sidebar_view = sidebar::view(self.active_tab);

        let tab_content: Element<'_, Message> = match self.active_tab {
            Tab::Network => network::view(&self.network),
            Tab::Display => display::view(&self.display),
            Tab::Ollama => ollama::view(&self.ollama),
        };

        let body = row![sidebar_view, tab_content];

        container(column![title_bar, body])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_primary)
            .into()
    }
}

// -- Async helpers --

fn do_wifi_scan() -> (Vec<WifiNetwork>, String) {
    let status_result = commands::network_status();
    let scan_result = commands::wifi_scan();

    let networks = if scan_result.success {
        parse_wifi_list(&scan_result.output)
    } else {
        Vec::new()
    };

    (networks, status_result.output.trim().to_owned())
}

fn parse_wifi_list(output: &str) -> Vec<WifiNetwork> {
    let mut networks = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 4 {
            let ssid = parts[0].trim().to_owned();
            if ssid.is_empty() {
                continue;
            }
            let signal = parts[1].trim().parse::<u8>().unwrap_or(0);
            let security = parts[2].trim().to_owned();
            let connected = parts[3].trim() == "*";
            networks.push(WifiNetwork {
                ssid,
                signal,
                security,
                connected,
            });
        }
    }
    // Deduplicate by SSID, keep highest signal
    networks.sort_by(|a, b| b.signal.cmp(&a.signal));
    networks.dedup_by(|a, b| a.ssid == b.ssid);
    networks
}

fn do_display_refresh() -> Vec<DisplayOutput> {
    let result = commands::display_list();
    if !result.success {
        return Vec::new();
    }
    parse_sway_outputs(&result.output)
}

fn parse_sway_outputs(json_str: &str) -> Vec<DisplayOutput> {
    let Ok(outputs) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) else {
        return Vec::new();
    };

    outputs
        .iter()
        .filter_map(|o| {
            let name = o["name"].as_str()?.to_owned();
            let current = o.get("current_mode")?;
            let width = current["width"].as_u64()? as u32;
            let height = current["height"].as_u64()? as u32;
            let refresh = current["refresh"].as_f64()? as f32 / 1000.0;
            let scale = o["scale"].as_f64().unwrap_or(1.0) as f32;

            let modes = o["modes"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            Some(DisplayMode {
                                width: m["width"].as_u64()? as u32,
                                height: m["height"].as_u64()? as u32,
                                refresh: m["refresh"].as_f64()? as f32 / 1000.0,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            Some(DisplayOutput {
                name,
                width,
                height,
                refresh,
                scale,
                modes,
            })
        })
        .collect()
}

fn do_ollama_refresh() -> (bool, Vec<String>) {
    let status = commands::ollama_status();
    let running = status.success && status.output.trim() == "active";

    let models_result = commands::ollama_list_models();
    let models = if models_result.success {
        models_result
            .output
            .lines()
            .skip(1) // skip header
            .filter_map(|line| {
                let name = line.split_whitespace().next()?;
                if name.is_empty() {
                    None
                } else {
                    Some(name.to_owned())
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    (running, models)
}
