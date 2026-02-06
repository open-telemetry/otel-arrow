// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::control::{AckMsg, NackMsg, PipelineControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::node::NodeId;
use otap_df_channel::error::SendError;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, UdpSocket};

/// Common implementation of all effect handlers.
///
/// Note: This implementation is `Send`.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore<PData> {
    pub(crate) node_id: NodeId,
    // ToDo refactor the code to avoid using Option here.
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender<PData>>,
    #[allow(dead_code)]
    // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) metrics_reporter: MetricsReporter,
}

impl<PData> EffectHandlerCore<PData> {
    /// Creates a new EffectHandlerCore with node_id and a metrics reporter.
    pub(crate) fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        Self {
            node_id,
            pipeline_ctrl_msg_sender: None,
            metrics_reporter,
        }
    }

    /// Sets the pipeline control message sender for this effect handler.
    pub fn set_pipeline_ctrl_msg_sender(
        &mut self,
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender<PData>,
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
        // Ignore write errors as they're typically not recoverable for stdout
        let _ = out.write_all(message.as_bytes()).await;
        let _ = out.write_all(b"\n").await;
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
    pub(crate) fn tcp_listener(
        &self,
        addr: SocketAddr,
        receiver_id: NodeId,
    ) -> Result<TcpListener, Error> {
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
        // TODO: Investigate adding set_reuse_port support for Windows.
        #[cfg(unix)]
        {
            sock.set_reuse_port(true).map_err(into_engine_error)?;
        }
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
    pub(crate) fn udp_socket(
        &self,
        addr: SocketAddr,
        receiver_id: NodeId,
    ) -> Result<UdpSocket, Error> {
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
        // TODO: Investigate adding set_reuse_port support for Windows.
        #[cfg(unix)]
        {
            sock.set_reuse_port(true).map_err(into_engine_error)?;
        }
        sock.set_nonblocking(true).map_err(into_engine_error)?;
        sock.bind(&addr.into()).map_err(into_engine_error)?;

        UdpSocket::from_std(sock.into()).map_err(into_engine_error)
    }

    /// Reports the provided metrics to the engine.
    #[allow(dead_code)] // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) fn report_metrics<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), TelemetryError> {
        self.metrics_reporter.report(metrics)
    }

    /// Re-usable function to send a pipeline control message. This returns a reference
    /// to the sender to place in a cancelation, for example.
    async fn send_pipeline_ctrl_msg(
        &self,
        msg: PipelineControlMsg<PData>,
    ) -> Result<PipelineCtrlMsgSender<PData>, SendError<PipelineControlMsg<PData>>> {
        let pipeline_ctrl_msg_sender = self.pipeline_ctrl_msg_sender.clone()
            .expect("[Internal Error] Node request sender not set. This is a bug in the pipeline engine implementation.");
        pipeline_ctrl_msg_sender.send(msg).await?;
        Ok(pipeline_ctrl_msg_sender)
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: The timer can only be started once per node.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle<PData>, Error> {
        let pipeline_ctrl_msg_sender = self
            .send_pipeline_ctrl_msg(PipelineControlMsg::StartTimer {
                node_id: self.node_id.index,
                duration,
            })
            .await
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })?;

        Ok(TimerCancelHandle {
            node_id: self.node_id.index,
            pipeline_ctrl_msg_sender,
        })
    }

    /// Starts a cancellable periodic telemetry collection timer that emits CollectTelemetry on the control channel.
    /// Returns a handle that can be used to cancel the telemetry timer.
    pub async fn start_periodic_telemetry(
        &self,
        duration: Duration,
    ) -> Result<TelemetryTimerCancelHandle<PData>, Error> {
        let pipeline_ctrl_msg_sender = self
            .send_pipeline_ctrl_msg(PipelineControlMsg::StartTelemetryTimer {
                node_id: self.node_id.index,
                duration,
            })
            .await
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })?;

        Ok(TelemetryTimerCancelHandle {
            node_id: self.node_id.clone(),
            pipeline_ctrl_msg_sender,
        })
    }

    /// Send a AckMsg using a context-transfer function.  The context
    /// transfer function applies PData-specific logic to discover the
    /// next recipient in the chain of Acks, if any.  When there is a
    /// recipient, this returns its node_id and the AckMsg prepared for
    /// delivery with the recipient's calldata.
    pub async fn route_ack<Transfer>(
        &self,
        ack_in: AckMsg<PData>,
        transfer: Transfer,
    ) -> Result<(), Error>
    where
        Transfer: FnOnce(AckMsg<PData>) -> Option<(usize, AckMsg<PData>)>,
    {
        if let Some((node_id, ack)) = transfer(ack_in) {
            self.send_pipeline_ctrl_msg(PipelineControlMsg::DeliverAck { node_id, ack })
                .await
                .map(|_| ())
                .map_err(|e| {
                    eprintln!(
                        "[DEBUG] route_ack failed: node={:?} error={}",
                        self.node_id, e
                    );
                    Error::PipelineControlMsgError {
                        error: e.to_string(),
                    }
                })
        } else {
            Ok(())
        }
    }

    /// Send a NackMsg using a context-transfer function.  The context
    /// transfer function applies PData-specific logic to discover the
    /// next recipient in the chain of Nacks, if any.  When there is a
    /// recipient, this returns its node_id and the NackMsg prepared for
    /// delivery with the recipient's calldata.
    pub async fn route_nack<Transfer>(
        &self,
        nack_in: NackMsg<PData>,
        transfer: Transfer,
    ) -> Result<(), Error>
    where
        Transfer: FnOnce(NackMsg<PData>) -> Option<(usize, NackMsg<PData>)>,
    {
        if let Some((node_id, nack)) = transfer(nack_in) {
            self.send_pipeline_ctrl_msg(PipelineControlMsg::DeliverNack { node_id, nack })
                .await
                .map(|_| ())
                .map_err(|e| {
                    eprintln!(
                        "[DEBUG] route_nack failed: node={:?} error={}",
                        self.node_id, e
                    );
                    Error::PipelineControlMsgError {
                        error: e.to_string(),
                    }
                })
        } else {
            Ok(())
        }
    }

    /// Delay a message.
    pub async fn delay_data(&self, when: Instant, data: Box<PData>) -> Result<(), PData> {
        self.send_pipeline_ctrl_msg(PipelineControlMsg::DelayData {
            node_id: self.node_id().index,
            when,
            data,
        })
        .await
        .map(|_| ())
        .map_err(|e| -> PData {
            match e.inner() {
                PipelineControlMsg::DelayData { data, .. } => *data,
                _ => unreachable!(),
            }
        })
    }
}

/// Handle to cancel a running timer.
pub struct TimerCancelHandle<PData> {
    node_id: usize,
    pipeline_ctrl_msg_sender: PipelineCtrlMsgSender<PData>,
}

impl<PData> TimerCancelHandle<PData> {
    /// Cancels the timer.
    pub async fn cancel(self) -> Result<(), SendError<PipelineControlMsg<PData>>> {
        self.pipeline_ctrl_msg_sender
            .send(PipelineControlMsg::CancelTimer {
                node_id: self.node_id,
            })
            .await
    }
}

/// Handle to cancel a running telemetry timer.
pub struct TelemetryTimerCancelHandle<PData> {
    node_id: NodeId,
    pipeline_ctrl_msg_sender: PipelineCtrlMsgSender<PData>,
}

impl<PData> TelemetryTimerCancelHandle<PData> {
    /// Cancels the telemetry collection timer.
    pub async fn cancel(self) -> Result<(), SendError<PipelineControlMsg<PData>>> {
        self.pipeline_ctrl_msg_sender
            .send(PipelineControlMsg::CancelTelemetryTimer {
                node_id: self.node_id.index,
                _temp: std::marker::PhantomData,
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{AckMsg, NackMsg, pipeline_ctrl_msg_channel};
    use crate::testing::test_node;

    /// Documents the current behavior of route_nack when the control channel is closed.
    ///
    /// This simulates the race condition that occurs during pipeline shutdown:
    /// 1. A node receives data and starts processing (e.g., sending a NACK)
    /// 2. During the async operation, upstream nodes complete and drop their channel clones
    /// 3. The PipelineCtrlMsgManager exits when all senders are gone
    /// 4. The NACK send fails because the channel receiver is gone
    ///
    /// Current behavior: Returns PipelineControlMsgError
    ///
    /// Trade-offs to consider:
    /// - Returning error (current): Fail-fast, visible for debugging, caller can decide
    /// - Returning Ok: ACKs/NACKs are best-effort, prevents cascade failures during shutdown
    ///
    /// This causes test flakiness in durable_buffer_processor_tests::test_durable_buffer_recovery_after_outage
    /// when the error exporter tries to NACK data while the pipeline is shutting down.
    #[tokio::test]
    async fn test_route_nack_returns_error_when_channel_closed() {
        let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
        let metrics_reporter = metrics_system.reporter();
        let node_id = test_node("test_node");

        let mut effect_handler: EffectHandlerCore<String> =
            EffectHandlerCore::new(node_id, metrics_reporter);

        // Create a channel and immediately drop the receiver to simulate shutdown
        let (tx, rx) = pipeline_ctrl_msg_channel::<String>(10);
        effect_handler.set_pipeline_ctrl_msg_sender(tx);
        drop(rx); // Simulate the PipelineCtrlMsgManager exiting

        // Create a NACK message
        let nack = NackMsg::new("test error", "test data".to_string());

        // This transfer function always returns a recipient (node 0)
        let transfer = |nack: NackMsg<String>| -> Option<(usize, NackMsg<String>)> {
            Some((0, nack))
        };

        let result = effect_handler.route_nack(nack, transfer).await;

        // Document current behavior: returns error when channel is closed
        assert!(
            result.is_err(),
            "route_nack currently returns an error when channel is closed: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::PipelineControlMsgError { .. }),
            "Expected PipelineControlMsgError, got: {:?}",
            err
        );
    }

    /// Documents the current behavior of route_ack when the control channel is closed.
    /// See test_route_nack_returns_error_when_channel_closed for full context.
    #[tokio::test]
    async fn test_route_ack_returns_error_when_channel_closed() {
        let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
        let metrics_reporter = metrics_system.reporter();
        let node_id = test_node("test_node");

        let mut effect_handler: EffectHandlerCore<String> =
            EffectHandlerCore::new(node_id, metrics_reporter);

        // Create a channel and immediately drop the receiver to simulate shutdown
        let (tx, rx) = pipeline_ctrl_msg_channel::<String>(10);
        effect_handler.set_pipeline_ctrl_msg_sender(tx);
        drop(rx); // Simulate the PipelineCtrlMsgManager exiting

        // Create an ACK message
        let ack = AckMsg::new("test data".to_string());

        // This transfer function always returns a recipient (node 0)
        let transfer = |ack: AckMsg<String>| -> Option<(usize, AckMsg<String>)> { Some((0, ack)) };

        let result = effect_handler.route_ack(ack, transfer).await;

        // Document current behavior: returns error when channel is closed
        assert!(
            result.is_err(),
            "route_ack currently returns an error when channel is closed: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(
            matches!(err, Error::PipelineControlMsgError { .. }),
            "Expected PipelineControlMsgError, got: {:?}",
            err
        );
    }
}
