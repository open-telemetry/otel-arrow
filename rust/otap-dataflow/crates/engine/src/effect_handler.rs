// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::Interests;
use crate::control::{
    AckMsg, NackMsg, PipelineResultMsg, PipelineResultMsgSender, RuntimeControlMsg,
    RuntimeCtrlMsgSender,
};
use crate::error::Error;
use crate::node::NodeId;
use otap_df_channel::error::SendError;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, UdpSocket};

/// SourceTagging indicates whether the Context will contain empty source frames.
#[derive(Clone, Copy)]
pub enum SourceTagging {
    /// Disabled means no source node-id will be automatically
    /// inserted for nodes that do not not otherwise subscribe to
    /// Ack/Nack.
    Disabled,

    /// Enabled means a source node_id will be automatically entered
    /// by creating a new frame in as messagees are sent.
    Enabled,
}

impl SourceTagging {
    /// Indicates that source tagging is required.
    #[must_use]
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Common implementation of all effect handlers.
///
/// Note: This implementation is `Send`.
#[derive(Clone)]
pub(crate) struct EffectHandlerCore<PData> {
    pub(crate) node_id: NodeId,
    // ToDo refactor the code to avoid using Option here.
    pub(crate) runtime_ctrl_msg_sender: Option<RuntimeCtrlMsgSender<PData>>,
    pub(crate) pipeline_result_msg_sender: Option<PipelineResultMsgSender<PData>>,
    #[allow(dead_code)]
    // Will be used in the future. ToDo report metrics from channel and messages.
    pub(crate) metrics_reporter: MetricsReporter,
    /// The outgoing message source tagging mode.
    pub(crate) source_tag: SourceTagging,
    /// Precomputed node interests derived from metric level.
    node_interests: Interests,
}

impl<PData> EffectHandlerCore<PData> {
    /// Creates a new EffectHandlerCore with node_id and a metrics reporter.
    pub(crate) const fn new(node_id: NodeId, metrics_reporter: MetricsReporter) -> Self {
        Self {
            node_id,
            runtime_ctrl_msg_sender: None,
            pipeline_result_msg_sender: None,
            metrics_reporter,
            source_tag: SourceTagging::Disabled,
            node_interests: Interests::empty(),
        }
    }

    /// Sets the runtime control message sender for this effect handler.
    pub fn set_runtime_ctrl_msg_sender(
        &mut self,
        runtime_ctrl_msg_sender: RuntimeCtrlMsgSender<PData>,
    ) {
        self.runtime_ctrl_msg_sender = Some(runtime_ctrl_msg_sender);
    }

    /// Sets the pipeline result message sender for this effect handler.
    pub fn set_pipeline_result_msg_sender(
        &mut self,
        pipeline_result_msg_sender: PipelineResultMsgSender<PData>,
    ) {
        self.pipeline_result_msg_sender = Some(pipeline_result_msg_sender);
    }

    /// Sets whether outgoing messages need source node tagging.
    pub fn set_source_tagging(&mut self, value: SourceTagging) {
        self.source_tag = value;
    }

    /// Returns outgoing messages source tagging mode.
    #[must_use]
    pub const fn source_tagging(&self) -> SourceTagging {
        self.source_tag
    }

    /// Sets the precomputed node interests for this effect handler.
    pub fn set_node_interests(&mut self, interests: Interests) {
        self.node_interests = interests;
    }

    /// Returns the precomputed node interests.
    ///
    /// Includes SOURCE_TAGGING when source tagging is enabled.
    #[must_use]
    pub fn node_interests(&self) -> Interests {
        if self.source_tag.enabled() {
            self.node_interests | Interests::SOURCE_TAGGING
        } else {
            self.node_interests
        }
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

    /// Re-usable function to send a runtime control message. This returns a reference
    /// to the sender to place in a cancelation, for example.
    async fn send_runtime_ctrl_msg(
        &self,
        msg: RuntimeControlMsg<PData>,
    ) -> Result<RuntimeCtrlMsgSender<PData>, SendError<RuntimeControlMsg<PData>>> {
        let runtime_ctrl_msg_sender = self.runtime_ctrl_msg_sender.clone()
            .expect("[Internal Error] Node request sender not set. This is a bug in the pipeline engine implementation.");
        runtime_ctrl_msg_sender.send(msg).await?;
        Ok(runtime_ctrl_msg_sender)
    }

    /// Re-usable function to send a pipeline result message.
    async fn send_pipeline_result_msg(
        &self,
        msg: PipelineResultMsg<PData>,
    ) -> Result<PipelineResultMsgSender<PData>, SendError<PipelineResultMsg<PData>>> {
        let pipeline_result_msg_sender = self.pipeline_result_msg_sender.clone()
            .expect("[Internal Error] Node return sender not set. This is a bug in the pipeline engine implementation.");
        pipeline_result_msg_sender.send(msg).await?;
        Ok(pipeline_result_msg_sender)
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: The timer can only be started once per node.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle<PData>, Error> {
        let runtime_ctrl_msg_sender = self
            .send_runtime_ctrl_msg(RuntimeControlMsg::StartTimer {
                node_id: self.node_id.index,
                duration,
            })
            .await
            .map_err(|e| Error::RuntimeMsgError {
                error: e.to_string(),
            })?;

        Ok(TimerCancelHandle {
            node_id: self.node_id.index,
            runtime_ctrl_msg_sender,
        })
    }

    /// Starts a cancellable periodic telemetry collection timer that emits CollectTelemetry on the control channel.
    /// Returns a handle that can be used to cancel the telemetry timer.
    pub async fn start_periodic_telemetry(
        &self,
        duration: Duration,
    ) -> Result<TelemetryTimerCancelHandle<PData>, Error> {
        let runtime_ctrl_msg_sender = self
            .send_runtime_ctrl_msg(RuntimeControlMsg::StartTelemetryTimer {
                node_id: self.node_id.index,
                duration,
            })
            .await
            .map_err(|e| Error::RuntimeMsgError {
                error: e.to_string(),
            })?;

        Ok(TelemetryTimerCancelHandle {
            node_id: self.node_id.clone(),
            runtime_ctrl_msg_sender,
        })
    }

    /// Send an AckMsg to the runtime control manager for context unwinding.
    /// This will skip if there are no frames.
    pub async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        if ack.accepted.has_frames() {
            self.send_pipeline_result_msg(PipelineResultMsg::DeliverAck { ack })
                .await
                .map(|_| ())
                .map_err(|e| Error::RuntimeMsgError {
                    error: e.to_string(),
                })
        } else {
            Ok(())
        }
    }

    /// Send a NackMsg to the runtime control manager for context unwinding.
    /// Same semantics as `route_ack()`.
    pub async fn route_nack(&self, nack: NackMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        if nack.refused.has_frames() {
            self.send_pipeline_result_msg(PipelineResultMsg::DeliverNack { nack })
                .await
                .map(|_| ())
                .map_err(|e| Error::RuntimeMsgError {
                    error: e.to_string(),
                })
        } else {
            Ok(())
        }
    }

    /// Delay a message.
    pub async fn delay_data(&self, when: Instant, data: Box<PData>) -> Result<(), PData> {
        self.send_runtime_ctrl_msg(RuntimeControlMsg::DelayData {
            node_id: self.node_id().index,
            when,
            data,
        })
        .await
        .map(|_| ())
        .map_err(|e| -> PData {
            match e.inner() {
                RuntimeControlMsg::DelayData { data, .. } => *data,
                _ => unreachable!(),
            }
        })
    }

    /// Notifies the runtime control manager that this receiver has completed
    /// ingress drain.
    pub async fn notify_receiver_drained(&self) -> Result<(), Error> {
        self.send_runtime_ctrl_msg(RuntimeControlMsg::ReceiverDrained {
            node_id: self.node_id().index,
        })
        .await
        .map(|_| ())
        .map_err(|e| Error::RuntimeMsgError {
            error: e.to_string(),
        })
    }
}

/// Handle to cancel a running timer.
pub struct TimerCancelHandle<PData> {
    node_id: usize,
    runtime_ctrl_msg_sender: RuntimeCtrlMsgSender<PData>,
}

impl<PData> TimerCancelHandle<PData> {
    /// Cancels the timer.
    pub async fn cancel(self) -> Result<(), SendError<RuntimeControlMsg<PData>>> {
        self.runtime_ctrl_msg_sender
            .send(RuntimeControlMsg::CancelTimer {
                node_id: self.node_id,
            })
            .await
    }
}

/// Handle to cancel a running telemetry timer.
pub struct TelemetryTimerCancelHandle<PData> {
    node_id: NodeId,
    runtime_ctrl_msg_sender: RuntimeCtrlMsgSender<PData>,
}

impl<PData> TelemetryTimerCancelHandle<PData> {
    /// Cancels the telemetry collection timer.
    pub async fn cancel(self) -> Result<(), SendError<RuntimeControlMsg<PData>>> {
        self.runtime_ctrl_msg_sender
            .send(RuntimeControlMsg::CancelTelemetryTimer {
                node_id: self.node_id.index,
                _temp: std::marker::PhantomData,
            })
            .await
    }
}
