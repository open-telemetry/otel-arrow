// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::error::Error;
use crate::message::ControlMsg;
use std::borrow::Cow;
use std::net::SocketAddr;
use tokio::net::{TcpListener, UdpSocket};

/// Default TCP listen backlog size
const DEFAULT_TCP_LISTEN_BACKLOG: i32 = 8192;

/// Common implementation of all effect handlers (local variant - not Send).
#[derive(Clone)]
pub(crate) struct LocalEffectHandlerCore {
    pub(crate) node_name: Cow<'static, str>,
    pub(crate) control_sender: Option<crate::message::Sender<ControlMsg>>,
}

/// Common implementation of all effect handlers (shared variant - Send).
#[derive(Clone)]
pub(crate) struct SharedEffectHandlerCore {
    pub(crate) node_name: Cow<'static, str>,
    pub(crate) control_sender: Option<tokio::sync::mpsc::Sender<ControlMsg>>,
}

impl LocalEffectHandlerCore {
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

impl SharedEffectHandlerCore {
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
                })?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{ControlMsg, Sender};
    use otap_df_channel::mpsc;
    use std::net::{IpAddr, Ipv4Addr};
    use tokio::sync::mpsc as tokio_mpsc;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct TestData {
        id: u64,
        payload: String,
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_creation() {
        let (sender, _receiver) = mpsc::Channel::new(100);
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(Sender::Local(sender)),
        };

        assert_eq!(core.node_name(), "test_node");
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_creation() {
        let (sender, _receiver) = tokio_mpsc::channel(100);
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(sender),
        };

        assert_eq!(core.node_name(), "test_node");
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_send_ack() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(Sender::Local(sender)),
        };

        let result = core.send_ack::<TestData>(123).await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(received_msg, ControlMsg::Ack { id: 123 });
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_send_nack() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(Sender::Local(sender)),
        };

        let result = core.send_nack::<TestData>(456, "Test error").await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(
            received_msg,
            ControlMsg::Nack {
                id: 456,
                reason: "Test error".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_send_ack() {
        let (sender, mut receiver) = tokio_mpsc::channel(100);
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(sender),
        };

        let result = core.send_ack::<TestData>(789).await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(received_msg, ControlMsg::Ack { id: 789 });
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_send_nack() {
        let (sender, mut receiver) = tokio_mpsc::channel(100);
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: Some(sender),
        };

        let result = core.send_nack::<TestData>(101, "Another test error").await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(
            received_msg,
            ControlMsg::Nack {
                id: 101,
                reason: "Another test error".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_send_ack_no_sender() {
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let result = core.send_ack::<TestData>(123).await;
        assert!(result.is_ok()); // Should succeed even without a sender
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_send_nack_no_sender() {
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let result = core.send_nack::<TestData>(456, "Test error").await;
        assert!(result.is_ok()); // Should succeed even without a sender
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_tcp_listener() {
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let listener = core.tcp_listener::<TestData>(addr, "test_receiver");

        assert!(listener.is_ok());
        let listener = listener.unwrap();
        assert!(listener.local_addr().is_ok());
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_tcp_listener() {
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let listener = core.tcp_listener::<TestData>(addr, "test_receiver");

        assert!(listener.is_ok());
        let listener = listener.unwrap();
        assert!(listener.local_addr().is_ok());
    }

    #[tokio::test]
    async fn test_local_effect_handler_core_udp_socket() {
        let core = LocalEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let socket = core.udp_socket::<TestData>(addr, "test_receiver");

        assert!(socket.is_ok());
        let socket = socket.unwrap();
        assert!(socket.local_addr().is_ok());
    }

    #[tokio::test]
    async fn test_shared_effect_handler_core_udp_socket() {
        let core = SharedEffectHandlerCore {
            node_name: "test_node".into(),
            control_sender: None,
        };

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
        let socket = core.udp_socket::<TestData>(addr, "test_receiver");

        assert!(socket.is_ok());
        let socket = socket.unwrap();
        assert!(socket.local_addr().is_ok());
    }
}
