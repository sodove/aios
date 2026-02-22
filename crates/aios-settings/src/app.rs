use iced::{Element, Task};

use crate::commands;
use crate::theme;
use crate::views::{ai, display, network, ollama, sidebar};

/// Active settings tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Network,
    Display,
    Ollama,
    Ai,
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
    /// Models available to pull (fetched from Ollama library API).
    pub available_models: Vec<String>,
    pub progress: Option<String>,
    pub error: Option<String>,
}

/// State for AI Provider tab.
#[derive(Debug, Clone)]
pub struct AiState {
    pub provider: String,   // "ollama", "openai", "claude"
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub saved: bool,
    pub error: Option<String>,
}

impl Default for AiState {
    fn default() -> Self {
        Self {
            provider: "ollama".to_owned(),
            api_key: String::new(),
            model: String::new(),
            base_url: String::new(),
            saved: false,
            error: None,
        }
    }
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
    OllamaRefreshDone { running: bool, models: Vec<String>, available: Vec<String> },
    OllamaStart,
    OllamaStop,
    OllamaPull(String),
    OllamaRemove(String),
    OllamaActionDone(bool, String),

    // AI Provider
    AiLoadConfig,
    AiConfigLoaded(String, String, String, String), // provider, api_key, model, base_url
    AiSelectProvider(String),
    AiApiKeyChanged(String),
    AiModelChanged(String),
    AiBaseUrlChanged(String),
    AiSave,
    AiSaveDone(bool, String),
}

pub struct SettingsApp {
    pub active_tab: Tab,
    pub network: NetworkState,
    pub display: DisplayState,
    pub ollama: OllamaState,
    pub ai: AiState,
}

impl SettingsApp {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            active_tab: Tab::Network,
            network: NetworkState::default(),
            display: DisplayState::default(),
            ollama: OllamaState::default(),
            ai: AiState::default(),
        };
        // Auto-refresh on start
        let tasks = Task::batch([
            Task::perform(async { do_wifi_scan() }, |(nets, status)| Message::WifiScanDone(nets, status)),
            Task::perform(async { do_display_refresh() }, Message::DisplayRefreshDone),
            Task::perform(async { do_ollama_refresh() }, |(running, models, available)| {
                Message::OllamaRefreshDone { running, models, available }
            }),
            Task::perform(async { load_ai_config() }, |(p, k, m, u)| Message::AiConfigLoaded(p, k, m, u)),
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
                return Task::perform(async { do_ollama_refresh() }, |(running, models, available)| {
                    Message::OllamaRefreshDone { running, models, available }
                });
            }
            Message::OllamaRefreshDone { running, models, available } => {
                self.ollama.running = running;
                self.ollama.models = models;
                self.ollama.available_models = available;
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
                    return Task::perform(async { do_ollama_refresh() }, |(running, models, available)| {
                        Message::OllamaRefreshDone { running, models, available }
                    });
                } else {
                    self.ollama.error = Some(msg);
                    self.ollama.progress = None;
                }
            }

            // -- AI Provider --
            Message::AiLoadConfig => {
                return Task::perform(async { load_ai_config() }, |(p, k, m, u)| {
                    Message::AiConfigLoaded(p, k, m, u)
                });
            }
            Message::AiConfigLoaded(provider, api_key, model, base_url) => {
                self.ai.provider = provider;
                self.ai.api_key = api_key;
                self.ai.model = model;
                self.ai.base_url = base_url;
                self.ai.saved = false;
            }
            Message::AiSelectProvider(p) => {
                self.ai.provider = p;
                self.ai.saved = false;
            }
            Message::AiApiKeyChanged(v) => {
                self.ai.api_key = v;
                self.ai.saved = false;
            }
            Message::AiModelChanged(v) => {
                self.ai.model = v;
                self.ai.saved = false;
            }
            Message::AiBaseUrlChanged(v) => {
                self.ai.base_url = v;
                self.ai.saved = false;
            }
            Message::AiSave => {
                let provider = self.ai.provider.clone();
                let api_key = self.ai.api_key.clone();
                let model = self.ai.model.clone();
                let base_url = self.ai.base_url.clone();
                return Task::perform(
                    async move { save_ai_config(&provider, &api_key, &model, &base_url) },
                    |(ok, msg)| Message::AiSaveDone(ok, msg),
                );
            }
            Message::AiSaveDone(success, msg) => {
                if success {
                    self.ai.saved = true;
                    self.ai.error = None;
                } else {
                    self.ai.error = Some(msg);
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
            Tab::Ai => ai::view(&self.ai),
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

fn do_ollama_refresh() -> (bool, Vec<String>, Vec<String>) {
    let status = commands::ollama_status();
    let running = status.success && status.output.trim() == "active";

    let models_result = commands::ollama_list_models();
    let models: Vec<String> = if models_result.success {
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

    // Fetch available models from Ollama library API (offline-only: size > 0)
    let available = fetch_available_models(&models);

    (running, models, available)
}

/// Fetch popular offline models from the Ollama library API.
/// Falls back to a curated list if the API is unreachable.
fn fetch_available_models(installed: &[String]) -> Vec<String> {
    let fallback = vec![
        "llama3.2", "llama3.1", "mistral", "qwen2.5",
        "gemma2", "phi4-mini", "deepseek-r1", "codellama",
    ];

    let result = std::process::Command::new("curl")
        .args(["-sf", "--connect-timeout", "5", "https://ollama.com/api/tags"])
        .output();

    let output = match result {
        Ok(o) if o.status.success() => o.stdout,
        _ => {
            return fallback
                .into_iter()
                .filter(|m| !installed.iter().any(|i| i.starts_with(m)))
                .map(|s| s.to_owned())
                .collect();
        }
    };

    let parsed: Result<serde_json::Value, _> = serde_json::from_slice(&output);
    match parsed {
        Ok(json) => {
            // API returns {"models": [...]} — unwrap the wrapper object
            let mut models: Vec<String> = json
                .get("models")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
                .filter(|m| {
                    m.get("size").and_then(|s| s.as_u64()).unwrap_or(0) > 0
                })
                .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_owned()))
                .filter(|name| !installed.iter().any(|i| i.starts_with(name.as_str())))
                .take(20)
                .collect();
            if models.is_empty() {
                models = fallback
                    .into_iter()
                    .filter(|m| !installed.iter().any(|i| i.starts_with(m)))
                    .map(|s| s.to_owned())
                    .collect();
            }
            models
        }
        Err(_) => fallback
            .into_iter()
            .filter(|m| !installed.iter().any(|i| i.starts_with(m)))
            .map(|s| s.to_owned())
            .collect(),
    }
}

/// Config path: ~/.config/aios/agent.toml
fn ai_config_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from(".config"))
        .join("aios")
        .join("agent.toml")
}

fn load_ai_config() -> (String, String, String, String) {
    let path = ai_config_path();
    if !path.exists() {
        return ("ollama".to_owned(), String::new(), String::new(), "http://localhost:11434".to_owned());
    }
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let config: serde_json::Value = toml::from_str(&content).unwrap_or_default();

    let provider = config.get("provider")
        .and_then(|p| p.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("ollama")
        .to_owned();
    let api_key = config.get("provider")
        .and_then(|p| p.get("api_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let model = config.get("provider")
        .and_then(|p| p.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let base_url = config.get("provider")
        .and_then(|p| p.get("base_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    (provider, api_key, model, base_url)
}

fn save_ai_config(provider: &str, api_key: &str, model: &str, base_url: &str) -> (bool, String) {
    let path = ai_config_path();

    // Read existing config to preserve agent section
    let mut config: toml::Value = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_else(|_| toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Update provider section
    let table = config.as_table_mut().unwrap();
    let mut prov = toml::map::Map::new();
    prov.insert("type".to_owned(), toml::Value::String(provider.to_owned()));
    prov.insert("api_key".to_owned(), toml::Value::String(api_key.to_owned()));
    prov.insert("model".to_owned(), toml::Value::String(model.to_owned()));
    if !base_url.is_empty() {
        prov.insert("base_url".to_owned(), toml::Value::String(base_url.to_owned()));
    }
    table.insert("provider".to_owned(), toml::Value::Table(prov));

    // Ensure agent section exists with defaults
    if !table.contains_key("agent") {
        let uid = std::env::var("UID")
            .or_else(|_| std::env::var("EUID"))
            .unwrap_or_else(|_| "1000".to_owned());
        let mut agent = toml::map::Map::new();
        agent.insert("socket_path".to_owned(), toml::Value::String(format!("/run/user/{uid}/aios-agent.sock")));
        agent.insert("audit_log".to_owned(), toml::Value::String("/var/log/aios/actions.log".to_owned()));
        agent.insert("max_destructive_per_minute".to_owned(), toml::Value::Integer(3));
        table.insert("agent".to_owned(), toml::Value::Table(agent));
    }

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match toml::to_string_pretty(&config) {
        Ok(content) => match std::fs::write(&path, &content) {
            Ok(()) => (true, "Saved! Restart aios-agent to apply.".to_owned()),
            Err(e) => (false, format!("Write error: {e}")),
        },
        Err(e) => (false, format!("Serialize error: {e}")),
    }
}
