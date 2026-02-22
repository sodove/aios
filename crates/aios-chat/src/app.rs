use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use iced::widget::markdown;
use iced::{Element, Subscription, Task};
use tokio::sync::Mutex;
use uuid::Uuid;

use aios_common::ipc::IpcWriter;
use aios_common::{
    AiosConfig, ChatMessage, IpcMessage, IpcPayload, MessageContent, ProviderConfig, ProviderType,
};

use crate::ipc_client::{self, IpcEvent};
use crate::state::{ConnectionStatus, DisplayMessage, ToolStatus};
use crate::views::{chat_view, oobe};

/// Root application state for the AIOS Chat UI.
pub struct AiosChat {
    messages: Vec<DisplayMessage>,
    input_text: String,
    connection_status: ConnectionStatus,
    /// Shared writer handle for sending messages to the agent.
    writer: Option<Arc<Mutex<IpcWriter>>>,
    /// Sent with every `ChatRequest`.
    conversation_id: Uuid,
    /// Accumulator for the current streaming assistant response.
    streaming_message: Option<StreamingMessage>,
    /// OOBE wizard state. `None` means normal chat mode.
    oobe_state: Option<OobeState>,
}

/// State for the OOBE (first boot) setup wizard.
pub struct OobeState {
    /// Current wizard step.
    pub step: OobeStep,
    /// Provider chosen by the user (set during `SelectProvider`).
    pub selected_provider: Option<ProviderType>,
    /// API key text input buffer.
    pub api_key_input: String,
    /// Selected Ollama model name.
    pub ollama_model: Option<String>,
    /// Status message during Ollama setup.
    pub ollama_status: Option<String>,
    /// Whether a model pull is in progress.
    pub pulling: bool,
    /// Animated progress value (0.0 -- 100.0) for the indeterminate bar.
    pub pull_progress: f32,
    /// Available models fetched from Ollama library.
    pub available_models: Vec<String>,
    /// Custom model name typed by user.
    pub custom_model_input: String,
}

/// Steps in the OOBE setup wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OobeStep {
    /// Welcome / greeting screen.
    Welcome,
    /// LLM provider selection.
    SelectProvider,
    /// API key entry (skipped for Ollama).
    EnterApiKey,
    /// Ollama: checking if installed, installing if needed.
    OllamaSetup,
    /// Ollama: selecting which model to pull.
    OllamaModelSelect,
    /// Setup complete -- summary before entering chat.
    Complete,
}

/// Tracks an in-progress streaming response from the agent.
struct StreamingMessage {
    id: Uuid,
    request_id: Uuid,
    text: String,
}

/// All messages the UI can produce.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    /// The user typed in the input field.
    InputChanged(String),
    /// The user pressed Enter or clicked Send.
    SendMessage,
    /// A clickable link inside a rendered markdown block was clicked.
    OpenUrl(markdown::Uri),
    /// An IPC lifecycle event from the background worker.
    Ipc(IpcEvent),
    /// Async IPC send completed (Ok) or failed (Err reason).
    SendCompleted(Result<(), String>),

    // -- OOBE wizard messages --

    /// Advance from Welcome to SelectProvider.
    OobeNext,
    /// User selected a provider on the SelectProvider screen.
    OobeSelectProvider(ProviderType),
    /// User typed into the API key field.
    OobeApiKeyChanged(String),
    /// User submitted the API key.
    OobeSubmitApiKey,
    /// Ollama installation check completed.
    OobeOllamaChecked { installed: bool },
    /// Available models list fetched from Ollama library.
    OobeOllamaModelsLoaded(Vec<String>),
    /// User typed into custom model input.
    OobeOllamaCustomModelChanged(String),
    /// User selected an Ollama model to pull.
    OobeOllamaSelectModel(String),
    /// Ollama model pull progress update (0.0 -- 100.0).
    OobeOllamaPullProgress(f32),
    /// Ollama model pull completed.
    OobeOllamaModelPulled(Result<(), String>),
    /// Navigate back to the previous OOBE step.
    OobeBack,
    /// User chose to skip the OOBE wizard entirely.
    OobeSkip,
    /// Exit OOBE and enter normal chat mode.
    OobeComplete,
    /// Config file was saved (or failed) asynchronously.
    OobeConfigSaved(Result<(), String>),

    /// User clicked the close (X) button.
    CloseWindow,
}

impl AiosChat {
    /// Bootstrap the application state. Returns `(state, initial_command)`.
    ///
    /// If no configuration file exists at `~/.config/aios/agent.toml`, the
    /// application starts in OOBE (first-boot) mode.
    pub fn new() -> (Self, Task<Message>) {
        let oobe_state = if config_path().exists() {
            None
        } else {
            Some(OobeState {
                step: OobeStep::Welcome,
                selected_provider: None,
                api_key_input: String::new(),
                ollama_model: None,
                ollama_status: None,
                pulling: false,
                pull_progress: 0.0,
                available_models: Vec::new(),
                custom_model_input: String::new(),
            })
        };

        let state = Self {
            messages: Vec::new(),
            input_text: String::new(),
            connection_status: ConnectionStatus::Connecting,
            writer: None,
            conversation_id: Uuid::new_v4(),
            streaming_message: None,
            oobe_state,
        };
        // The IPC worker subscription handles connection automatically.
        (state, Task::none())
    }

    /// Process an incoming UI message and return a command.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // -- Normal chat messages --
            Message::InputChanged(value) => {
                self.input_text = value;
            }
            Message::SendMessage => {
                return self.handle_send();
            }
            Message::OpenUrl(url) => {
                tracing::info!("Opening URL: {url}");
            }
            Message::Ipc(event) => {
                return self.handle_ipc_event(event);
            }
            Message::SendCompleted(result) => {
                if let Err(reason) = result {
                    tracing::error!("Failed to send message: {reason}");
                    self.messages.push(DisplayMessage::assistant(
                        Uuid::new_v4(),
                        format!("*Send error:* {reason}"),
                        Utc::now(),
                    ));
                }
            }

            Message::CloseWindow => {
                return iced::exit();
            }

            // -- OOBE wizard messages --
            Message::OobeNext => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.step = OobeStep::SelectProvider;
                }
            }
            Message::OobeSelectProvider(provider) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.selected_provider = Some(provider);
                    if provider == ProviderType::Ollama {
                        oobe.step = OobeStep::OllamaSetup;
                        oobe.ollama_status = Some("Starting Ollama service...".to_owned());
                        return Task::perform(
                            async {
                                // Ollama is pre-installed in the ISO. Just check it exists and start the service.
                                let installed = std::process::Command::new("ollama")
                                    .arg("--version")
                                    .output()
                                    .map(|o| o.status.success())
                                    .unwrap_or(false);
                                if installed {
                                    let _ = std::process::Command::new("systemctl")
                                        .args(["start", "ollama"])
                                        .output();
                                    // Give service a moment to start
                                    std::thread::sleep(std::time::Duration::from_secs(2));
                                }
                                installed
                            },
                            |installed| Message::OobeOllamaChecked { installed },
                        );
                    }
                    oobe.step = OobeStep::EnterApiKey;
                }
            }
            Message::OobeApiKeyChanged(value) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.api_key_input = value;
                }
            }
            Message::OobeSubmitApiKey => {
                return self.save_oobe_config();
            }
            Message::OobeOllamaChecked { installed } => {
                if let Some(oobe) = &mut self.oobe_state {
                    if installed {
                        oobe.step = OobeStep::OllamaModelSelect;
                        oobe.ollama_status = Some("Loading available models...".to_owned());
                        // Fetch available models from Ollama library
                        return Task::perform(
                            async {
                                fetch_ollama_models().await
                            },
                            Message::OobeOllamaModelsLoaded,
                        );
                    } else {
                        oobe.ollama_status = Some("Ollama not found. You can install it from Settings.".to_owned());
                        oobe.step = OobeStep::OllamaModelSelect;
                    }
                }
            }
            Message::OobeOllamaModelsLoaded(models) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.available_models = models;
                    oobe.ollama_status = None;
                }
            }
            Message::OobeOllamaCustomModelChanged(value) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.custom_model_input = value;
                }
            }
            Message::OobeOllamaSelectModel(model) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.ollama_model = Some(model.clone());
                    oobe.ollama_status = Some(format!("Pulling {model}..."));
                    oobe.pulling = true;
                    oobe.pull_progress = 0.0;
                    return Task::perform(
                        async move {
                            let output = tokio::task::spawn_blocking(move || {
                                std::process::Command::new("ollama")
                                    .args(["pull", &model])
                                    .output()
                            })
                            .await
                            .unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
                            match output {
                                Ok(o) if o.status.success() => Ok(()),
                                Ok(o) => Err(String::from_utf8_lossy(&o.stderr).to_string()),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::OobeOllamaModelPulled,
                    );
                }
            }
            Message::OobeOllamaPullProgress(_) => {
                // Tick from subscription — animate the progress bar
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.pull_progress = (oobe.pull_progress + 2.0) % 100.0;
                }
            }
            Message::OobeOllamaModelPulled(result) => {
                if let Some(oobe) = &mut self.oobe_state {
                    oobe.pulling = false;
                    oobe.pull_progress = 0.0;
                    match result {
                        Ok(()) => {
                            oobe.ollama_status = Some("Model ready!".to_owned());
                        }
                        Err(e) => {
                            oobe.ollama_status = Some(format!("Pull failed: {e}. You can try again from Settings."));
                        }
                    }
                    return self.save_oobe_config();
                }
            }
            Message::OobeBack => {
                if let Some(oobe) = &mut self.oobe_state {
                    match oobe.step {
                        OobeStep::EnterApiKey => oobe.step = OobeStep::SelectProvider,
                        OobeStep::SelectProvider => oobe.step = OobeStep::Welcome,
                        OobeStep::OllamaSetup | OobeStep::OllamaModelSelect => {
                            oobe.step = OobeStep::SelectProvider;
                            oobe.ollama_status = None;
                        }
                        _ => {}
                    }
                }
            }
            Message::OobeSkip => {
                // Save a default config with an empty API key (echo mode).
                return self.save_default_config();
            }
            Message::OobeComplete => {
                self.oobe_state = None;
                self.messages.push(DisplayMessage::assistant(
                    Uuid::new_v4(),
                    "Привет! Чем могу помочь?".to_owned(),
                    Utc::now(),
                ));
            }
            Message::OobeConfigSaved(result) => {
                match result {
                    Ok(()) => {
                        if let Some(oobe) = &mut self.oobe_state {
                            oobe.step = OobeStep::Complete;
                        }
                        // Restart aios-agent so it picks up the new config
                        let _ = std::process::Command::new("systemctl")
                            .args(["--user", "restart", "aios-agent"])
                            .spawn();
                    }
                    Err(reason) => {
                        tracing::error!("Failed to save config: {reason}");
                        // Stay on the current step; the user can retry.
                    }
                }
            }
        }
        Task::none()
    }

    /// Declarative subscription: runs the IPC background worker when alive.
    pub fn subscription(&self) -> Subscription<Message> {
        let ipc = Subscription::run(ipc_client::ipc_worker).map(Message::Ipc);

        // Animate progress bar while pulling a model
        let is_pulling = self
            .oobe_state
            .as_ref()
            .map_or(false, |o| o.pulling);

        if is_pulling {
            let tick = iced::time::every(std::time::Duration::from_millis(200))
                .map(|_| Message::OobeOllamaPullProgress(0.0));
            Subscription::batch([ipc, tick])
        } else {
            ipc
        }
    }

    /// Build the view tree for the current state.
    ///
    /// When OOBE is active, the setup wizard is shown instead of the chat.
    pub fn view(&self) -> Element<'_, Message> {
        if let Some(oobe_state) = &self.oobe_state {
            return oobe::view(oobe_state);
        }
        chat_view::view(self)
    }

    // -- Accessors used by views --

    pub fn messages(&self) -> &[DisplayMessage] {
        &self.messages
    }

    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection_status
    }

    pub fn can_send(&self) -> bool {
        !self.input_text.trim().is_empty()
            && self.connection_status == ConnectionStatus::Connected
    }

    /// Returns the OOBE state if the wizard is active.
    #[allow(dead_code)]
    pub fn oobe_state(&self) -> Option<&OobeState> {
        self.oobe_state.as_ref()
    }

    // -- Internal helpers --

    /// Handle `Message::SendMessage`: validate, enqueue user message, and
    /// fire an async IPC send.
    fn handle_send(&mut self) -> Task<Message> {
        let text = self.input_text.trim().to_owned();
        if text.is_empty() {
            return Task::none();
        }

        let Some(writer) = self.writer.clone() else {
            // Not connected -- do nothing (button should be disabled).
            tracing::warn!("SendMessage while disconnected; ignoring");
            return Task::none();
        };

        // Add the user message to the display list.
        self.messages
            .push(DisplayMessage::user(Uuid::new_v4(), text.clone(), Utc::now()));

        // Clear input.
        self.input_text.clear();

        // Build IPC message.
        let conversation_id = self.conversation_id;
        let ipc_msg = IpcMessage {
            id: Uuid::new_v4(),
            payload: IpcPayload::ChatRequest {
                message: text,
                conversation_id,
            },
        };

        // Fire and forget via async task.
        Task::perform(
            async move {
                let mut w = writer.lock().await;
                w.send(&ipc_msg)
                    .await
                    .map_err(|e| format!("{e}"))
            },
            Message::SendCompleted,
        )
    }

    /// Handle an event coming from the IPC background subscription.
    fn handle_ipc_event(&mut self, event: IpcEvent) -> Task<Message> {
        match event {
            IpcEvent::Connected(writer) => {
                tracing::info!("IPC connected");
                self.connection_status = ConnectionStatus::Connected;
                self.writer = Some(writer);
            }
            IpcEvent::Disconnected(reason) => {
                tracing::warn!("IPC disconnected: {reason}");
                self.connection_status = ConnectionStatus::Disconnected;
                self.writer = None;
            }
            IpcEvent::ChatResponse(chat_msg) => {
                self.append_chat_response(&chat_msg);
            }
            IpcEvent::StreamChunk {
                request_id,
                delta,
                done,
            } => {
                self.handle_stream_chunk(request_id, &delta, done);
            }
            IpcEvent::AgentError { message } => {
                tracing::error!("Agent error: {message}");
                self.messages.push(DisplayMessage::assistant(
                    Uuid::new_v4(),
                    format!("*Agent error:* {message}"),
                    Utc::now(),
                ));
            }
        }
        Task::none()
    }

    /// Append a complete `ChatResponse` as one or more `DisplayMessage`s.
    ///
    /// Text content becomes a single assistant message. Tool use and tool
    /// result payloads are expanded into individual tool cards.
    fn append_chat_response(&mut self, chat_msg: &ChatMessage) {
        match &chat_msg.content {
            MessageContent::Text { text } => {
                self.messages.push(DisplayMessage::assistant(
                    chat_msg.id,
                    text.clone(),
                    chat_msg.timestamp,
                ));
            }
            MessageContent::ToolUse { tool_calls } => {
                for tc in tool_calls {
                    let args_pretty = serde_json::to_string_pretty(&tc.arguments)
                        .unwrap_or_else(|_| tc.arguments.to_string());
                    self.messages.push(DisplayMessage::tool_call(
                        tc.id,
                        tc.name.clone(),
                        args_pretty,
                        chat_msg.timestamp,
                    ));
                }
            }
            MessageContent::ToolResult { results } => {
                for tr in results {
                    // Try to resolve the tool name from a matching pending
                    // ToolCall card; fall back to "tool" if none found.
                    let tool_name = self
                        .messages
                        .iter()
                        .rev()
                        .find(|m| m.id == tr.call_id)
                        .and_then(|m| m.tool_name.clone())
                        .unwrap_or_else(|| "tool".to_owned());

                    // Update the matching ToolCall card status.
                    if let Some(call_msg) = self
                        .messages
                        .iter_mut()
                        .rev()
                        .find(|m| m.id == tr.call_id)
                    {
                        let new_status = if tr.is_error {
                            ToolStatus::Failed
                        } else {
                            ToolStatus::Completed
                        };
                        call_msg.set_tool_status(new_status);
                    }

                    self.messages.push(DisplayMessage::tool_result(
                        tr.call_id,
                        tool_name,
                        tr.output.clone(),
                        tr.is_error,
                        chat_msg.timestamp,
                    ));
                }
            }
        }
    }

    /// Handle an incremental streaming chunk from the agent.
    fn handle_stream_chunk(&mut self, request_id: Uuid, delta: &str, done: bool) {
        let streaming = self
            .streaming_message
            .get_or_insert_with(|| StreamingMessage {
                id: Uuid::new_v4(),
                request_id,
                text: String::new(),
            });

        // If request_id changed, finalize the previous and start fresh.
        if streaming.request_id != request_id {
            self.finalize_streaming();
            self.streaming_message = Some(StreamingMessage {
                id: Uuid::new_v4(),
                request_id,
                text: String::new(),
            });
        }

        let streaming = self.streaming_message.as_mut().expect("just created");
        streaming.text.push_str(delta);

        // Update or insert the display message for this stream.
        if let Some(display_msg) = self.messages.iter_mut().rev().find(|m| m.id == streaming.id) {
            display_msg.update_text(streaming.text.clone());
        } else {
            self.messages.push(DisplayMessage::assistant(
                streaming.id,
                streaming.text.clone(),
                Utc::now(),
            ));
        }

        if done {
            self.streaming_message = None;
        }
    }

    /// Finalize an in-progress streaming message so we stop appending to it.
    fn finalize_streaming(&mut self) {
        self.streaming_message = None;
    }

    // -- OOBE config persistence --

    /// Build an `AiosConfig` from current OOBE selections and save it.
    fn save_oobe_config(&self) -> Task<Message> {
        let Some(oobe) = &self.oobe_state else {
            return Task::none();
        };

        let provider_type = oobe.selected_provider.unwrap_or(ProviderType::Claude);
        let api_key = oobe.api_key_input.trim().to_owned();

        let (model, base_url) = match provider_type {
            ProviderType::Claude => ("claude-sonnet-4-20250514".to_owned(), None),
            ProviderType::OpenAi => ("gpt-4o".to_owned(), None),
            ProviderType::Ollama => {
                let model = oobe.ollama_model.clone().unwrap_or_else(|| "llama3".to_owned());
                (model, Some("http://localhost:11434".to_owned()))
            }
        };

        let config = AiosConfig {
            provider: ProviderConfig {
                provider_type,
                api_key,
                model,
                base_url,
            },
            ..AiosConfig::default()
        };

        Task::perform(write_config(config), Message::OobeConfigSaved)
    }

    /// Save a default config with an empty API key (echo / skip mode).
    fn save_default_config(&self) -> Task<Message> {
        let config = AiosConfig::default();
        Task::perform(write_config(config), Message::OobeConfigSaved)
    }
}

/// Returns the canonical config file path: `~/.config/aios/agent.toml`.
fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("aios")
        .join("agent.toml")
}

/// Serialize `config` as TOML and write it to [`config_path()`].
///
/// Creates the parent directory if it does not exist.
async fn write_config(config: AiosConfig) -> Result<(), String> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("failed to create config directory: {e}"))?;
    }

    let toml_str =
        toml::to_string_pretty(&config).map_err(|e| format!("failed to serialize config: {e}"))?;

    tokio::fs::write(&path, toml_str)
        .await
        .map_err(|e| format!("failed to write config file: {e}"))?;

    tracing::info!("Config saved to {}", path.display());
    Ok(())
}

/// Get available Ollama models: locally installed + offline models from Ollama API.
///
/// Strategy:
/// 1. List locally installed models via `ollama list`
/// 2. Fetch from `https://ollama.com/api/tags`, keep only offline models (size > 0)
/// 3. Fallback to curated list if API is unreachable
async fn fetch_ollama_models() -> Vec<String> {
    let mut models = Vec::new();

    // 1. Locally installed models
    let local_result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("ollama")
            .arg("list")
            .output()
            .ok()
    })
    .await;

    if let Ok(Some(output)) = local_result {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) {
                if let Some(name) = line.split_whitespace().next() {
                    if !name.is_empty() {
                        models.push(name.to_owned());
                    }
                }
            }
        }
    }

    // 2. Fetch from Ollama library API, filter offline-only (size > 0)
    let api_result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("curl")
            .args(["-sS", "--max-time", "10", "https://ollama.com/api/tags"])
            .output()
    })
    .await;

    let mut got_api = false;
    if let Ok(Ok(output)) = api_result {
        if output.status.success() {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(api_models) = json.get("models").and_then(|m| m.as_array()) {
                    let names: Vec<String> = api_models
                        .iter()
                        .filter(|m| {
                            // size == 0 means online-only model; skip those
                            m.get("size")
                                .and_then(|s| s.as_u64())
                                .unwrap_or(0)
                                > 0
                        })
                        .filter_map(|m| {
                            m.get("name").and_then(|n| n.as_str()).map(String::from)
                        })
                        .take(20)
                        .collect();
                    if !names.is_empty() {
                        got_api = true;
                        for name in names {
                            if !models.contains(&name) {
                                models.push(name);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback curated list if API was unreachable
    if !got_api {
        let recommended = [
            "llama3.2:3b",
            "llama3.1:8b",
            "mistral:7b",
            "qwen2.5:7b",
            "gemma2:9b",
            "phi4-mini",
            "deepseek-r1:8b",
            "codellama:7b",
        ];
        for rec in recommended {
            if !models.iter().any(|m| m == rec) {
                models.push(rec.to_owned());
            }
        }
    }

    models
}

