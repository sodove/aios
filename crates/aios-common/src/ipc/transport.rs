use std::path::Path;

use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::{UnixListener, UnixStream};

use crate::error::AiosError;

use super::protocol::{IpcMessage, LengthPrefixedCodec};

/// A Unix domain socket server that accepts IPC connections.
pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    /// Bind a new IPC server to the given Unix socket path.
    ///
    /// Removes any stale socket file at `path` before binding.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::Io`] if the socket cannot be created.
    pub fn bind(path: impl AsRef<Path>) -> Result<Self, AiosError> {
        let path = path.as_ref();

        // Remove stale socket file if it exists.
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| {
                AiosError::Ipc(format!("failed to remove stale socket {}: {e}", path.display()))
            })?;
        }

        // Ensure the parent directory exists.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AiosError::Ipc(format!(
                    "failed to create socket directory {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let listener = UnixListener::bind(path)?;
        Ok(Self { listener })
    }

    /// Accept the next incoming IPC connection.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::Io`] on accept failure.
    pub async fn accept(&self) -> Result<IpcConnection, AiosError> {
        let (stream, _addr) = self.listener.accept().await?;
        Ok(IpcConnection { stream })
    }
}

/// An IPC client that connects to an agent via a Unix domain socket.
pub struct IpcClient;

impl IpcClient {
    /// Connect to the IPC server at the given Unix socket path.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::Io`] if the connection cannot be established.
    pub async fn connect(path: impl AsRef<Path>) -> Result<IpcConnection, AiosError> {
        let stream = UnixStream::connect(path).await?;
        Ok(IpcConnection { stream })
    }
}

/// A bidirectional IPC connection over a Unix domain socket.
pub struct IpcConnection {
    stream: UnixStream,
}

impl IpcConnection {
    /// Send an IPC message over this connection.
    ///
    /// # Errors
    ///
    /// Returns encoding or I/O errors.
    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), AiosError> {
        let (_, mut writer) = self.stream.split();
        LengthPrefixedCodec::write(&mut writer, msg).await
    }

    /// Receive the next IPC message from this connection.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::ConnectionClosed`] on EOF, or decoding/I/O errors.
    pub async fn recv(&mut self) -> Result<IpcMessage, AiosError> {
        let (mut reader, _) = self.stream.split();
        LengthPrefixedCodec::decode(&mut reader).await
    }

    /// Split this connection into independent reader and writer halves
    /// for concurrent send/receive operations.
    pub fn into_split(self) -> (IpcReader, IpcWriter) {
        let (read_half, write_half) = tokio::io::split(self.stream);
        (IpcReader { inner: read_half }, IpcWriter { inner: write_half })
    }
}

/// The read half of a split IPC connection.
pub struct IpcReader {
    inner: ReadHalf<UnixStream>,
}

impl IpcReader {
    /// Receive the next IPC message.
    ///
    /// # Errors
    ///
    /// Returns [`AiosError::ConnectionClosed`] on EOF, or decoding/I/O errors.
    pub async fn recv(&mut self) -> Result<IpcMessage, AiosError> {
        LengthPrefixedCodec::decode(&mut self.inner).await
    }
}

/// The write half of a split IPC connection.
pub struct IpcWriter {
    inner: WriteHalf<UnixStream>,
}

impl IpcWriter {
    /// Send an IPC message.
    ///
    /// # Errors
    ///
    /// Returns encoding or I/O errors.
    pub async fn send(&mut self, msg: &IpcMessage) -> Result<(), AiosError> {
        LengthPrefixedCodec::write(&mut self.inner, msg).await
    }
}
