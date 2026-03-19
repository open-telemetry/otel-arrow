// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Common foundation of all effect handlers.

use crate::Interests;
use crate::control::{AckMsg, NackMsg, PipelineControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::node::NodeId;
use futures::channel::oneshot;
use otap_df_channel::error::SendError;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::reporter::MetricsReporter;
use std::future::Future;
use std::io;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

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

/// Boxed async reader type owned by the engine boundary.
pub type LocalAsyncBufRead = dyn tokio::io::AsyncBufRead + Unpin;
/// Engine-owned buffered reader for runtime local sockets/streams.
pub type LocalBufReader<R> = tokio::io::BufReader<R>;
/// Extension trait for buffered reads owned by the engine boundary.
pub use tokio::io::AsyncBufReadExt as LocalAsyncBufReadExt;

/// Create a buffered reader owned by the engine boundary.
#[must_use]
pub fn local_buf_reader<R>(reader: R) -> LocalBufReader<R>
where
    R: tokio::io::AsyncRead,
{
    tokio::io::BufReader::new(reader)
}

/// Convert a standard TCP listener into the engine runtime's async listener.
pub fn runtime_tcp_listener(listener: TcpListener) -> io::Result<tokio::net::TcpListener> {
    tokio::net::TcpListener::from_std(listener)
}

/// Convert a standard UDP socket into the engine runtime's async socket.
pub fn runtime_udp_socket(socket: UdpSocket) -> io::Result<tokio::net::UdpSocket> {
    tokio::net::UdpSocket::from_std(socket)
}

/// Sleep for the requested duration on the engine-owned runtime.
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Sleep until the requested deadline on the engine-owned runtime.
pub async fn sleep_until(deadline: Instant) {
    tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
}

/// Wait for a future with a timeout on the engine-owned runtime.
pub async fn timeout<F>(
    duration: Duration,
    future: F,
) -> Result<F::Output, tokio::time::error::Elapsed>
where
    F: Future,
{
    tokio::time::timeout(duration, future).await
}

/// Handle for a task spawned onto the engine-owned local runtime.
#[must_use = "Dropping the task handle detaches the task"]
pub struct LocalTaskHandle<T> {
    inner: tokio::task::JoinHandle<T>,
}

impl<T> LocalTaskHandle<T> {
    /// Abort the underlying task.
    pub fn abort(&self) {
        self.inner.abort();
    }

    /// Returns true if the task has finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }
}

impl<T> Future for LocalTaskHandle<T> {
    type Output = Result<T, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

/// Sender side of a local oneshot channel created by the engine boundary.
pub type LocalOneshotSender<T> = oneshot::Sender<T>;
/// Receiver side of a local oneshot channel created by the engine boundary.
pub type LocalOneshotReceiver<T> = oneshot::Receiver<T>;

/// Cancellation token owned by the engine boundary.
#[derive(Clone, Default)]
pub struct SharedCancellationToken {
    inner: CancellationToken,
}

impl SharedCancellationToken {
    /// Create a new cancellation token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: CancellationToken::new(),
        }
    }

    /// Cancel the token and all of its clones.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Wait until the token is cancelled.
    pub async fn cancelled(&self) {
        self.inner.cancelled().await;
    }

    /// Clone the inner cancellation token for engine-owned transport code.
    #[must_use]
    pub fn clone_inner(&self) -> CancellationToken {
        self.inner.clone()
    }
}

/// Shared semaphore owned by the engine boundary.
#[derive(Clone)]
pub struct SharedSemaphore {
    inner: Arc<tokio::sync::Semaphore>,
}

impl SharedSemaphore {
    /// Create a new semaphore with the given capacity.
    #[must_use]
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Semaphore::new(permits)),
        }
    }

    /// Clone the inner semaphore handle for engine-owned transport code.
    #[must_use]
    pub fn clone_inner(&self) -> Arc<tokio::sync::Semaphore> {
        self.inner.clone()
    }
}

/// Interval driven by the engine-owned runtime.
pub struct LocalInterval {
    inner: tokio::time::Interval,
}

impl LocalInterval {
    /// Wait for the next tick.
    pub async fn tick(&mut self) {
        _ = self.inner.tick().await;
    }

    /// Reset the interval to start a new period from now.
    pub fn reset(&mut self) {
        self.inner.reset();
    }
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
    pub(crate) pipeline_ctrl_msg_sender: Option<PipelineCtrlMsgSender<PData>>,
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
            pipeline_ctrl_msg_sender: None,
            metrics_reporter,
            source_tag: SourceTagging::Disabled,
            node_interests: Interests::empty(),
        }
    }

    /// Sets the pipeline control message sender for this effect handler.
    pub fn set_pipeline_ctrl_msg_sender(
        &mut self,
        pipeline_ctrl_msg_sender: PipelineCtrlMsgSender<PData>,
    ) {
        self.pipeline_ctrl_msg_sender = Some(pipeline_ctrl_msg_sender);
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
        use std::io::{Write, stdout};
        let mut out = stdout();
        // Ignore write errors as they're typically not recoverable for stdout
        let _ = out.write_all(message.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
    }

    /// Sleep for the requested duration on the engine-owned runtime.
    pub(crate) async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    /// Sleep until the requested deadline on the engine-owned runtime.
    pub(crate) async fn sleep_until(&self, deadline: Instant) {
        tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)).await;
    }

    /// Create an interval on the engine-owned runtime.
    pub(crate) fn interval(&self, period: Duration) -> LocalInterval {
        LocalInterval {
            inner: tokio::time::interval(period),
        }
    }

    /// Create an interval starting at the given deadline on the engine-owned runtime.
    pub(crate) fn interval_at(&self, start: Instant, period: Duration) -> LocalInterval {
        LocalInterval {
            inner: tokio::time::interval_at(tokio::time::Instant::from_std(start), period),
        }
    }

    /// Yield to other tasks on the engine-owned runtime.
    pub(crate) async fn yield_now(&self) {
        tokio::task::yield_now().await;
    }

    /// Spawn a task onto the engine-owned local runtime.
    pub(crate) fn spawn_local<F>(&self, future: F) -> LocalTaskHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
    {
        LocalTaskHandle {
            inner: tokio::task::spawn_local(future),
        }
    }

    /// Wait for a future with a timeout on the engine-owned runtime.
    pub(crate) async fn timeout<F>(
        &self,
        duration: Duration,
        future: F,
    ) -> Result<F::Output, tokio::time::error::Elapsed>
    where
        F: Future,
    {
        tokio::time::timeout(duration, future).await
    }

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    ///
    pub(crate) fn tcp_listener(
        &self,
        addr: SocketAddr,
        receiver_id: NodeId,
    ) -> Result<TcpListener, Error> {
        // Helper closure to convert errors.
        let into_engine_error = |error: io::Error| Error::IoError {
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

        Ok(sock.into())
    }

    /// Creates a non-blocking UDP socket on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create UDP
    /// sockets via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    ///
    #[allow(dead_code)]
    pub(crate) fn udp_socket(
        &self,
        addr: SocketAddr,
        receiver_id: NodeId,
    ) -> Result<UdpSocket, Error> {
        // Helper closure to convert errors.
        let into_engine_error = |error: io::Error| Error::IoError {
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

        Ok(sock.into())
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

    /// Send an AckMsg to the pipeline controller for context unwinding.
    /// This will skip if there are no frames.
    pub async fn route_ack(&self, ack: AckMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        if ack.accepted.has_frames() {
            self.send_pipeline_ctrl_msg(PipelineControlMsg::DeliverAck { ack })
                .await
                .map(|_| ())
                .map_err(|e| Error::PipelineControlMsgError {
                    error: e.to_string(),
                })
        } else {
            Ok(())
        }
    }

    /// Send a NackMsg to the pipeline controller for context unwinding.
    /// Same semantics as `route_ack()`.
    pub async fn route_nack(&self, nack: NackMsg<PData>) -> Result<(), Error>
    where
        PData: crate::Unwindable,
    {
        if nack.refused.has_frames() {
            self.send_pipeline_ctrl_msg(PipelineControlMsg::DeliverNack { nack })
                .await
                .map(|_| ())
                .map_err(|e| Error::PipelineControlMsgError {
                    error: e.to_string(),
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
