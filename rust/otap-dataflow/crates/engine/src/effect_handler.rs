// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::error::Error;
use crate::message::ControlMsg;
use std::borrow::Cow;
use std::net::SocketAddr;
use tokio::net::{TcpListener, UdpSocket};

/// Default TCP listen backlog size
const DEFAULT_TCP_LISTEN_BACKLOG: i32 = 8192;

/// Type alias for local (not Send) effect handler core
pub(crate) type LocalEffectHandlerCore = EffectHandlerCore<crate::message::Sender<ControlMsg>>;

/// Type alias for shared (Send) effect handler core  
pub(crate) type SharedEffectHandlerCore = EffectHandlerCore<tokio::sync::mpsc::Sender<ControlMsg>>;

/// Common implementation of all effect handlers.
///
/// Generic over sender type to support both Send and !Send variants.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore<S> {
    pub(crate) node_name: Cow<'static, str>,
    pub(crate) control_sender: Option<S>,
}

impl<S> EffectHandlerCore<S> {
    /// Returns the name of the node associated with this effect handler.
    #[must_use]
    pub(crate) fn node_name(&self) -> Cow<'static, str> {
        self.node_name.clone()
    }

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    ///
    /// ToDo: return a std::net::TcpListener instead of a tokio::net::tcp::TcpListener to avoid leaking our current dependency on Tokio.
    pub(crate) fn tcp_listener<PData>(
        &self,
        addr: SocketAddr,
        receiver_name: impl Into<Cow<'static, str>>,
    ) -> Result<TcpListener, Error<PData>> {
        let node_name: Cow<'static, str> = receiver_name.into();
        // Helper closure to convert errors.
        let into_engine_error = |error: std::io::Error| Error::IoError {
            node: node_name.clone(),
            error,
        };

        // Create a SO_REUSEADDR + SO_REUSEPORT listener.
        let sock = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::STREAM,
            None,
        )
        .map_err(into_engine_error)?;

        // Allows multiple sockets to bind to an address/port combination even if a socket in the
        // TIME_WAIT state currently occupies that combination.
        // Goal: Restarting the server quickly without waiting for the OS to release a port.
        sock.set_reuse_address(true).map_err(into_engine_error)?;
        // Explicitly allows multiple sockets to simultaneously bind and listen to the exact same
        // IP and port. Incoming connections or packets are distributed between the sockets
        // (load balancing).
        // Goal: Load balancing incoming connections.
        sock.set_reuse_port(true).map_err(into_engine_error)?;
        sock.set_nonblocking(true).map_err(into_engine_error)?;
        sock.bind(&addr.into()).map_err(into_engine_error)?;
        sock.listen(DEFAULT_TCP_LISTEN_BACKLOG)
            .map_err(into_engine_error)?;

        TcpListener::from_std(sock.into()).map_err(into_engine_error)
    }

    /// Creates a non-blocking UDP socket on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create UDP
    /// sockets via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    ///
    /// ToDo: return a std::net::UdpSocket instead of a tokio::net::UdpSocket to avoid leaking our current dependency on Tokio.
    #[allow(dead_code)]
    pub(crate) fn udp_socket<PData>(
        &self,
        addr: SocketAddr,
        receiver_name: impl Into<Cow<'static, str>>,
    ) -> Result<UdpSocket, Error<PData>> {
        let node_name: Cow<'static, str> = receiver_name.into();
        // Helper closure to convert errors.
        let into_engine_error = |error: std::io::Error| Error::IoError {
            node: node_name.clone(),
            error,
        };

        // Create a SO_REUSEADDR + SO_REUSEPORT UDP socket.
        let sock = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::DGRAM,
            None,
        )
        .map_err(into_engine_error)?;

        // Goal: Restarting the server quickly without waiting for the OS to release a port.
        sock.set_reuse_address(true).map_err(into_engine_error)?;
        // Explicitly allows multiple sockets to simultaneously bind to the exact same
        // IP and port. Incoming packets are distributed between the sockets
        // (load balancing).
        // Goal: Load balancing incoming packets.
        sock.set_reuse_port(true).map_err(into_engine_error)?;
        sock.set_nonblocking(true).map_err(into_engine_error)?;
        sock.bind(&addr.into()).map_err(into_engine_error)?;

        UdpSocket::from_std(sock.into()).map_err(into_engine_error)
    }
}

// Implementation for the Sender enum (used in local contexts)
impl EffectHandlerCore<crate::message::Sender<ControlMsg>> {
    /// Sends an ACK control message upstream to indicate successful processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the control message could not be sent.
    pub(crate) async fn send_ack<PData>(&self, id: u64) -> Result<(), Error<PData>> {
        if let Some(ref sender) = self.control_sender {
            sender
                .send(ControlMsg::Ack { id })
                .await
                .map_err(|_| Error::ProcessorError {
                    processor: self.node_name.clone(),
                    error: format!("Failed to send ACK control message for id {id}"),
                })?;
        }
        Ok(())
    }

    /// Sends a NACK control message upstream to indicate failed processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the control message could not be sent.
    pub(crate) async fn send_nack<PData>(&self, id: u64, reason: &str) -> Result<(), Error<PData>> {
        if let Some(ref sender) = self.control_sender {
            sender
                .send(ControlMsg::Nack {
                    id,
                    reason: reason.to_owned(),
                })
                .await
                .map_err(|_| Error::ProcessorError {
                    processor: self.node_name.clone(),
                    error: format!("Failed to send NACK control message for id {id}: {reason}"),
                })?;
        }
        Ok(())
    }
}

// Implementation for tokio Sender (used in shared contexts)
impl EffectHandlerCore<tokio::sync::mpsc::Sender<ControlMsg>> {
    /// Sends an ACK control message upstream to indicate successful processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the control message could not be sent.
    pub(crate) async fn send_ack<PData>(&self, id: u64) -> Result<(), Error<PData>> {
        if let Some(ref sender) = self.control_sender {
            sender
                .send(ControlMsg::Ack { id })
                .await
                .map_err(|_| Error::ProcessorError {
                    processor: self.node_name.clone(),
                    error: format!("Failed to send ACK control message for id {id}"),
                })?;
        }
        Ok(())
    }

    /// Sends a NACK control message upstream to indicate failed processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the control message could not be sent.
    pub(crate) async fn send_nack<PData>(&self, id: u64, reason: &str) -> Result<(), Error<PData>> {
        if let Some(ref sender) = self.control_sender {
            sender
                .send(ControlMsg::Nack {
                    id,
                    reason: reason.to_owned(),
                })
                .await
                .map_err(|_| Error::ProcessorError {
                    processor: self.node_name.clone(),
                    error: format!("Failed to send NACK control message for id {id}: {reason}"),
                })?;
        }
        Ok(())
    }
}
