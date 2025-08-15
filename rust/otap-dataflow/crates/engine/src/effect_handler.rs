// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::control::{PipelineControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::node::{Index, NodeId};
use otap_df_channel::error::SendError;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};

/// Common implementation of all effect handlers.
///
/// Note: This implementation is `Send`.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore {
    pub(crate) node: NodeId,
    // ToDo refactor the code to avoid using Option here.
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender>,
}

impl EffectHandlerCore {
    /// Creates a new EffectHandlerCore with node_id.
    pub(crate) fn new(node: NodeId) -> Self {
        Self {
            node,
            pipeline_ctrl_msg_sender: None,
        }
    }

    pub(crate) fn set_pipeline_ctrl_msg_sender(
        &mut self,
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender,
    ) {
        self.pipeline_ctrl_msg_sender = Some(pipeline_ctrl_msg_sender);
    }

    /// Returns the name of the node associated with this effect handler.
    #[must_use]
    pub(crate) fn node_id(&self) -> NodeId {
        self.node.clone()
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for all nodes in the pipeline
    /// to output informational messages without blocking the async runtime.
    pub(crate) async fn info(&self, message: &str) {
        use tokio::io::{AsyncWriteExt, stdout};
        let mut out = stdout();
        let formatted_message = format!("{message}\n");
        // Ignore write errors as they're typically not recoverable for stdout
        let _ = out.write_all(formatted_message.as_bytes()).await;
        let _ = out.flush().await;
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
        receiver_id: NodeId,
    ) -> Result<TcpListener, Error<PData>> {
        // Helper closure to convert errors.
        let into_engine_error = |error: std::io::Error| Error::IoError {
            node: receiver_id.clone(),
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
        receiver_id: NodeId,
    ) -> Result<UdpSocket, Error<PData>> {
        // Helper closure to convert errors.
        let into_engine_error = |error: std::io::Error| Error::IoError {
            node: receiver_id.clone(),
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

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: The timer can only be started once per node.
    pub async fn start_periodic_timer<PData>(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        let pipeline_ctrl_msg_sender = self.pipeline_ctrl_msg_sender.clone()
            .expect("[Internal Error] Node request sender not set. This is a bug in the pipeline engine implementation.");
        pipeline_ctrl_msg_sender
            .send(PipelineControlMsg::StartTimer {
                node_id: self.node.index,
                duration,
            })
            .await
            .map_err(Error::PipelineControlMsgError)?;

        Ok(TimerCancelHandle {
            node_id: self.node.index,
            pipeline_ctrl_msg_sender,
        })
    }
}

/// Handle to cancel a running timer.
pub struct TimerCancelHandle {
    node_id: Index,
    pipeline_ctrl_msg_sender: PipelineCtrlMsgSender,
}

impl TimerCancelHandle {
    /// Cancels the timer.
    pub async fn cancel(self) -> Result<(), SendError<PipelineControlMsg>> {
        self.pipeline_ctrl_msg_sender
            .send(PipelineControlMsg::CancelTimer {
                node_id: self.node_id,
            })
            .await
    }
}
