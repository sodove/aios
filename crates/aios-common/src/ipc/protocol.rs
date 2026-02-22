use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::error::AiosError;
use crate::types::message::ChatMessage;
use crate::types::trust::TrustLevel;

/// IPC message envelope with a unique identifier and typed payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: Uuid,
    #[serde(flatten)]
    pub payload: IpcPayload,
}

/// All possible IPC message payloads exchanged between AIOS components.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcPayload {
    // -- Chat --
    ChatRequest {
        message: String,
        conversation_id: Uuid,
    },
    ChatResponse {
        message: ChatMessage,
    },
    StreamChunk {
        request_id: Uuid,
        delta: String,
        done: bool,
    },

    // -- Tool confirmation --
    ConfirmRequest {
        action_id: Uuid,
        action_type: String,
        description: String,
        command: String,
        trust_level: TrustLevel,
    },
    ConfirmResponse {
        action_id: Uuid,
        approved: bool,
        reason: Option<String>,
    },

    // -- Client registration --
    Register {
        client_type: ClientType,
    },
    RegisterAck {
        success: bool,
    },

    // -- System --
    SystemInfo {
        info: serde_json::Value,
    },
    Error {
        message: String,
        code: Option<String>,
    },
    Ping,
    Pong,
}

/// Identifies the kind of IPC client connecting to the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Chat,
    Dock,
    Confirm,
}

/// Length-prefixed JSON codec for IPC messages.
///
/// Wire format: `[4-byte BE u32 length][JSON bytes]`
///
/// The 4-byte prefix carries the byte length of the JSON payload that follows,
/// encoded as a big-endian unsigned 32-bit integer.
pub struct LengthPrefixedCodec;

impl LengthPrefixedCodec {
    /// Maximum allowed message size (16 MiB).
    const MAX_MESSAGE_SIZE: u32 = 16 * 1024 * 1024;

    /// Encode an [`IpcMessage`] into a length-prefixed byte buffer.
    ///
    /// Returns a `Vec<u8>` containing the 4-byte BE length header followed by
    /// the JSON-serialised message body.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::Json`] if serialisation fails, or
    /// [`AiosError::Protocol`] if the serialised message exceeds the maximum
    /// allowed size.
    pub fn encode(msg: &IpcMessage) -> Result<Vec<u8>, AiosError> {
        let json = serde_json::to_vec(msg)?;

        let len: u32 = u32::try_from(json.len()).map_err(|_| {
            AiosError::Protocol(format!("message too large: {} bytes", json.len()))
        })?;

        if len > Self::MAX_MESSAGE_SIZE {
            return Err(AiosError::Protocol(format!(
                "message size {len} exceeds maximum {}",
                Self::MAX_MESSAGE_SIZE
            )));
        }

        let mut buf = Vec::with_capacity(4 + json.len());
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&json);
        Ok(buf)
    }

    /// Decode an [`IpcMessage`] from an async reader.
    ///
    /// Reads the 4-byte BE length header, then reads exactly that many bytes
    /// of JSON and deserialises the result.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::ConnectionClosed`] on EOF,
    /// [`AiosError::Protocol`] if the declared size exceeds the limit,
    /// [`AiosError::Io`] on read failures, or [`AiosError::Json`] on parse
    /// failures.
    pub async fn decode<R: AsyncRead + Unpin>(reader: &mut R) -> Result<IpcMessage, AiosError> {
        let mut len_buf = [0u8; 4];

        match reader.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Err(AiosError::ConnectionClosed);
            }
            Err(e) => return Err(AiosError::Io(e)),
        }

        let len = u32::from_be_bytes(len_buf);

        if len > Self::MAX_MESSAGE_SIZE {
            return Err(AiosError::Protocol(format!(
                "incoming message size {len} exceeds maximum {}",
                Self::MAX_MESSAGE_SIZE
            )));
        }

        let mut json_buf = vec![0u8; len as usize];
        reader.read_exact(&mut json_buf).await?;

        let msg: IpcMessage = serde_json::from_slice(&json_buf)?;
        Ok(msg)
    }

    /// Write an [`IpcMessage`] to an async writer.
    ///
    /// Encodes the message and flushes the writer.
    ///
    /// # Errors
    ///
    /// Propagates encoding errors or I/O write errors.
    pub async fn write<W: AsyncWrite + Unpin>(
        writer: &mut W,
        msg: &IpcMessage,
    ) -> Result<(), AiosError> {
        let bytes = Self::encode(msg)?;
        writer.write_all(&bytes).await?;
        writer.flush().await?;
        Ok(())
    }
}
