// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement receivers.

use crate::error::Error;
use std::borrow::Cow;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Common implementation across both `!Send` and `Send` effect handlers.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore {
    pub(crate) node_name: Cow<'static, str>,
}

impl EffectHandlerCore {
    #[must_use]
    pub(crate) fn node_name(&self) -> &str {
        &self.node_name
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
    pub(crate) fn tcp_listener<PData, S: AsRef<str>>(
        &self,
        addr: SocketAddr,
        receiver_name: S,
    ) -> Result<TcpListener, Error<PData>> {
        // Helper closure to convert errors.
        let err = |error: std::io::Error| Error::IoError {
            node: receiver_name.as_ref().to_owned(),
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
        .map_err(err)?;

        // Allows multiple sockets to bind to an address/port combination even if a socket in the
        // TIME_WAIT state currently occupies that combination.
        // Goal: Restarting the server quickly without waiting for the OS to release a port.
        sock.set_reuse_address(true).map_err(err)?;
        // Explicitly allows multiple sockets to simultaneously bind and listen to the exact same
        // IP and port. Incoming connections or packets are distributed between the sockets
        // (load balancing).
        // Goal: Load balancing incoming connections.
        sock.set_reuse_port(true).map_err(err)?;
        sock.set_nonblocking(true).map_err(err)?;
        sock.bind(&addr.into()).map_err(err)?;
        sock.listen(8192).map_err(err)?;

        TcpListener::from_std(sock.into()).map_err(err)
    }
}
