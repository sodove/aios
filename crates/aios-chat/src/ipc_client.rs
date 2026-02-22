use std::sync::Arc;

use aios_common::ipc::IpcWriter;
use aios_common::{ChatMessage, IpcPayload};
use futures::channel::mpsc;
use futures::SinkExt;
use tokio::sync::Mutex;

/// Socket path resolution: `AIOS_SOCKET` env var or platform default.
pub fn socket_path() -> String {
    std::env::var("AIOS_SOCKET").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/tmp/aios-agent.sock".to_owned()
        } else {
            format!("/run/user/{}/aios-agent.sock", 1000)
        }
    })
}

/// Events produced by the IPC background worker and forwarded to the app.
#[derive(Clone)]
pub enum IpcEvent {
    /// Connection established; carries a shared writer handle.
    Connected(Arc<Mutex<IpcWriter>>),
    /// Connection attempt failed or lost; carries a human-readable reason.
    Disconnected(String),
    /// A complete chat response was received from the agent.
    ChatResponse(ChatMessage),
    /// A streaming chunk was received.
    StreamChunk {
        request_id: uuid::Uuid,
        delta: String,
        done: bool,
    },
    /// The agent reported an error.
    AgentError { message: String },
}

impl std::fmt::Debug for IpcEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected(_) => f.debug_tuple("Connected").field(&"<IpcWriter>").finish(),
            Self::Disconnected(reason) => {
                f.debug_tuple("Disconnected").field(reason).finish()
            }
            Self::ChatResponse(msg) => f.debug_tuple("ChatResponse").field(msg).finish(),
            Self::StreamChunk {
                request_id,
                delta,
                done,
            } => f
                .debug_struct("StreamChunk")
                .field("request_id", request_id)
                .field("delta", delta)
                .field("done", done)
                .finish(),
            Self::AgentError { message } => {
                f.debug_struct("AgentError").field("message", message).finish()
            }
        }
    }
}

/// Creates a long-lived `Stream<Item = IpcEvent>` that:
///
/// 1. Connects to the agent socket.
/// 2. Sends `Register { client_type: Chat }`.
/// 3. Waits for `RegisterAck`.
/// 4. Enters a read loop, forwarding agent messages as `IpcEvent`s.
/// 5. On any error, emits `Disconnected`, waits 2 seconds, and retries.
///
/// This function is designed to be used with `Subscription::run`.
pub fn ipc_worker() -> impl futures::Stream<Item = IpcEvent> {
    iced::stream::channel(64, async move |mut output: mpsc::Sender<IpcEvent>| {
        loop {
            if let Err(reason) = run_ipc_session(&mut output).await {
                let _ = output
                    .send(IpcEvent::Disconnected(reason.clone()))
                    .await;
                tracing::warn!("IPC session ended: {reason}. Reconnecting in 2 s...");
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    })
}

/// A single connect-register-read session. Returns `Err(reason)` when the
/// session must be retried.
async fn run_ipc_session(output: &mut mpsc::Sender<IpcEvent>) -> Result<(), String> {
    use aios_common::{ClientType, IpcClient, IpcMessage};

    let path = socket_path();
    tracing::info!("Connecting to agent at {path}...");

    let conn = IpcClient::connect(&path)
        .await
        .map_err(|e| format!("connect failed: {e}"))?;

    let (mut reader, writer) = conn.into_split();

    // -- Register --
    let register_msg = IpcMessage {
        id: uuid::Uuid::new_v4(),
        payload: IpcPayload::Register {
            client_type: ClientType::Chat,
        },
    };

    let writer = Arc::new(Mutex::new(writer));
    {
        let mut w = writer.lock().await;
        w.send(&register_msg)
            .await
            .map_err(|e| format!("register send failed: {e}"))?;
    }

    // -- Wait for RegisterAck --
    let ack = reader
        .recv()
        .await
        .map_err(|e| format!("register ack recv failed: {e}"))?;

    match ack.payload {
        IpcPayload::RegisterAck { success: true } => {
            tracing::info!("Registered with agent successfully");
        }
        IpcPayload::RegisterAck { success: false } => {
            return Err("agent rejected registration".to_owned());
        }
        IpcPayload::Error { message, .. } => {
            return Err(format!("agent error during registration: {message}"));
        }
        other => {
            return Err(format!("unexpected payload during registration: {other:?}"));
        }
    }

    // -- Notify app that we are connected --
    let _ = output.send(IpcEvent::Connected(Arc::clone(&writer))).await;

    // -- Read loop --
    loop {
        let msg = reader
            .recv()
            .await
            .map_err(|e| format!("read error: {e}"))?;

        let event = match msg.payload {
            IpcPayload::ChatResponse { message } => IpcEvent::ChatResponse(message),
            IpcPayload::StreamChunk {
                request_id,
                delta,
                done,
            } => IpcEvent::StreamChunk {
                request_id,
                delta,
                done,
            },
            IpcPayload::Error { message, .. } => IpcEvent::AgentError { message },
            IpcPayload::Ping => {
                // Respond with Pong.
                let pong = IpcMessage {
                    id: uuid::Uuid::new_v4(),
                    payload: IpcPayload::Pong,
                };
                let mut w = writer.lock().await;
                let _ = w.send(&pong).await;
                continue;
            }
            IpcPayload::Pong => continue,
            _ => {
                tracing::debug!("Ignoring unexpected IPC payload: {:?}", msg.payload);
                continue;
            }
        };

        if output.send(event).await.is_err() {
            // Receiver dropped -- app shutting down.
            return Ok(());
        }
    }
}


