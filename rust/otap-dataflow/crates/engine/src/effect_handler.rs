// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::control::{PipelineControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use otap_df_channel::error::SendError;
use otap_df_config::NodeId;
use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};
use tokio::time::Instant;

/// Common implementation of all effect handlers.
///
/// Note: This implementation is `Send`.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore {
    pub(crate) node_id: NodeId,
    // ToDo refactor the code to avoid using Option here.
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender>,
}

impl EffectHandlerCore {
    /// Creates a new EffectHandlerCore with node_id.
    pub(crate) fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            pipeline_ctrl_msg_sender: None,
        }
    }

    pub(crate) fn set_pipeline_ctrl_msg_sender(
        &mut self,
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender,
    ) {
        self.pipeline_ctrl_msg_sender = Some(pipeline_ctrl_msg_sender);
    }

    /// Returns the id of the node associated with this effect handler.
    #[must_use]
    pub(crate) fn node_id(&self) -> NodeId {
        self.node_id.clone()
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

    /// Generic helper for sending timer control messages.
    async fn send_timer_message<PData, F>(
        &self,
        create_message: F,
    ) -> Result<TimerCancelHandle, Error<PData>>
    where
        F: FnOnce(NodeId) -> PipelineControlMsg,
    {
        let pipeline_ctrl_msg_sender = self.pipeline_ctrl_msg_sender.clone()
            .expect("[Internal Error] Node request sender not set. This is a bug in the pipeline engine implementation.");

        let message = create_message(self.node_id.clone());

        pipeline_ctrl_msg_sender
            .send(message)
            .await
            .map_err(Error::PipelineControlMsgError)?;

        Ok(TimerCancelHandle {
            node_id: self.node_id.clone(),
            pipeline_ctrl_msg_sender,
        })
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Currently, only one timer can only be started at once per node, recurring or non-recurring.
    pub async fn start_periodic_timer<PData>(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.send_timer_message(|node_id| PipelineControlMsg::StartTimer { node_id, duration })
            .await
    }

    /// Schedules a cancellable non-recurring timer that emits TimerTick on the control channel at the specified time.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Currently, only one timer can only be started at once per node, recurring or non-recurring.
    pub async fn run_at_timer<PData>(
        &self,
        when: Instant,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.send_timer_message(|node_id| PipelineControlMsg::RunAtTimer { node_id, when })
            .await
    }
}

/// Handle to cancel a running timer.
pub struct TimerCancelHandle {
    node_id: NodeId,
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

#[cfg(test)]
mod tests {
    #![allow(missing_docs)]
    use super::*;
    use crate::control::{PipelineControlMsg, PipelineCtrlMsgReceiver, pipeline_ctrl_msg_channel};
    use tokio::time::{Duration, Instant};

    /// Helper function to set up a test environment
    fn setup_test_core() -> (EffectHandlerCore, PipelineCtrlMsgReceiver) {
        let (ctrl_tx, ctrl_rx) = pipeline_ctrl_msg_channel(10);
        let mut core = EffectHandlerCore::new("test_node".into());
        core.set_pipeline_ctrl_msg_sender(ctrl_tx);

        (core, ctrl_rx)
    }

    /// Helper function to assert that a specific control message was received
    async fn assert_control_message(
        ctrl_rx: &mut PipelineCtrlMsgReceiver,
        expected_message: impl Fn(PipelineControlMsg) -> bool + Send,
        message_description: &str,
    ) {
        let msg = ctrl_rx
            .recv()
            .await
            .expect("Control message should be valid");

        assert!(
            expected_message(msg.clone()),
            "Expected {}, got {:?}",
            message_description,
            msg
        );
    }

    #[tokio::test]
    async fn test_run_at_timer() {
        let (core, mut ctrl_rx) = setup_test_core();

        let target_time = Instant::now() + Duration::from_millis(100);
        let _handle = core.run_at_timer::<()>(target_time).await.unwrap();

        assert_control_message(
            &mut ctrl_rx,
            |msg| {
                matches!(msg, PipelineControlMsg::RunAtTimer { node_id, when } 
                if node_id == "test_node" && when == target_time)
            },
            "RunAtTimer message with correct node_id and time",
        )
        .await;
    }

    #[tokio::test]
    async fn test_start_periodic_timer() {
        let (core, mut ctrl_rx) = setup_test_core();

        let period = Duration::from_millis(200);
        let _handle = core.start_periodic_timer::<()>(period).await.unwrap();

        assert_control_message(
            &mut ctrl_rx,
            |msg| {
                matches!(msg, PipelineControlMsg::StartTimer { node_id, duration }
                if node_id == "test_node" && duration == period)
            },
            "StartTimer message with correct node_id and duration",
        )
        .await;
    }

    #[tokio::test]
    async fn test_cancel_timer() {
        let (core, mut ctrl_rx) = setup_test_core();

        let handle = core
            .start_periodic_timer::<()>(Duration::from_millis(100))
            .await
            .unwrap();

        // Clear the StartTimer message
        let _ = ctrl_rx.recv().await;

        // Cancel the timer
        handle.cancel().await.unwrap();

        assert_control_message(
            &mut ctrl_rx,
            |msg| {
                matches!(msg, PipelineControlMsg::CancelTimer { node_id }
                if node_id == "test_node")
            },
            "CancelTimer message with correct node_id",
        )
        .await;
    }
}
