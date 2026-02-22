use std::sync::Arc;

use aios_common::{AiosError, IpcMessage, IpcPayload, IpcServer};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::router;
use crate::state::{AgentState, ConnectedClient};

/// Run the IPC server loop: accept connections and spawn per-client handlers.
pub async fn run_server(
    server: IpcServer,
    state: Arc<RwLock<AgentState>>,
) -> anyhow::Result<()> {
    tracing::info!("IPC server listening for connections");

    loop {
        match server.accept().await {
            Ok(connection) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(connection, state).await {
                        tracing::error!("Client handler error: {e}");
                    }
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {e}");
            }
        }
    }
}

/// Handle a single connected client through its full lifecycle.
async fn handle_client(
    connection: aios_common::IpcConnection,
    state: Arc<RwLock<AgentState>>,
) -> anyhow::Result<()> {
    let client_id = Uuid::new_v4();
    let (mut reader, writer) = connection.into_split();

    tracing::info!(%client_id, "New client connected");

    // The first message must be a Register; otherwise we disconnect.
    let first_msg = reader.recv().await?;
    let client_type = match &first_msg.payload {
        IpcPayload::Register { client_type } => *client_type,
        _ => {
            tracing::warn!(%client_id, "First message was not Register, disconnecting");
            return Ok(());
        }
    };

    tracing::info!(%client_id, ?client_type, "Client registered");

    // Store the client in shared state.
    let writer = Mutex::new(writer);
    {
        let mut state_guard = state.write().await;
        state_guard.clients.insert(
            client_id,
            ConnectedClient {
                client_type,
                writer,
            },
        );
    }

    // Send RegisterAck back to the client.
    {
        let state_guard = state.read().await;
        if let Some(client) = state_guard.clients.get(&client_id) {
            let ack = IpcMessage {
                id: Uuid::new_v4(),
                payload: IpcPayload::RegisterAck { success: true },
            };
            client.writer.lock().await.send(&ack).await?;
        }
    }

    // Main message loop.
    loop {
        match reader.recv().await {
            Ok(msg) => {
                if let Some(response) = router::route_message(msg, client_id, &state).await {
                    let state_guard = state.read().await;
                    if let Some(client) = state_guard.clients.get(&client_id)
                        && let Err(e) = client.writer.lock().await.send(&response).await
                    {
                        tracing::error!(%client_id, "Failed to send response: {e}");
                        break;
                    }
                }
            }
            Err(AiosError::ConnectionClosed) => {
                tracing::info!(%client_id, "Client disconnected");
                break;
            }
            Err(e) => {
                tracing::error!(%client_id, "Read error: {e}");
                break;
            }
        }
    }

    // Cleanup: remove client from shared state.
    {
        let mut state_guard = state.write().await;
        state_guard.clients.remove(&client_id);
    }

    Ok(())
}
