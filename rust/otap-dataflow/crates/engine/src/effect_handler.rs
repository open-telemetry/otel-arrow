// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::error::Error;
use std::borrow::Cow;
use std::net::SocketAddr;
use tokio::net::{TcpListener, UdpSocket};

/// Common implementation of all effect handlers.
///
/// Note: This implementation is `Send`.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore {
    pub(crate) node_name: Cow<'static, str>,
}

impl EffectHandlerCore {
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
        sock.listen(8192).map_err(into_engine_error)?;

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
