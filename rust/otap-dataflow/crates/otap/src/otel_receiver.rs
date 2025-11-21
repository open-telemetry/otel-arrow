// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental receiver that serves OTLP and OTAP gRPC endpoints directly on top of the `h2`
//! crate.
//!
//! This receiver keeps all request handling on the current thread so it can integrate with a
//! thread-per-core runtime without requiring `Send + Sync` futures.
//!
//! Design goals:
//! - Support OTLP and OTAP Arrow over gRPC with minimal dependencies.
//! - Avoid `Send + Sync` bounds on request handlers so we can integrate with a single threaded executor.
//! - No `Arc` or `Mutex` in the hot path.
//! - Minimize heap allocations on the hot path.
//!
//! ToDo: Add snappy support.
//! ToDo: Improve error handling and metrics: surface clear statuses when the client requests
//!       unsupported codecs, log negotiation results, and add counters for negotiated and
//!       unsupported compression cases.
//! ToDo: Add support for Unix domain sockets as a transport option.

mod ack;
mod encoder;
pub(crate) mod grpc;
mod response_templates;
mod router;
mod status;
mod stream;

#[cfg(test)]
mod test_common;

#[cfg(test)]
mod otlp_tests;

#[cfg(test)]
mod otap_tests;

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::common;
use crate::otap_grpc::{GrpcServerSettings, Settings, per_connection_limit};
use crate::otap_receiver::OtapReceiverMetrics;
use crate::otel_receiver::router::{GrpcRequestRouter, respond_with_error};
use crate::pdata::OtapPdata;
use ack::{AckRegistries, AckRegistry, route_ack_response, route_nack_response};
use async_trait::async_trait;
use bytes::Bytes;
use encoder::ResponseEncoderPool;
use futures::{FutureExt, Stream, StreamExt, stream::FuturesUnordered};
use grpc::{AcceptedGrpcEncodings, build_accept_encoding_header};
use h2::server::{self, SendResponse};
use h2::{Ping, PingPong};
use http::Request;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::admitter::{AdmitDecision, Admitter, ConnectionGuard};
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::metrics::MetricSet;
use response_templates::ResponseTemplates;
use serde::Deserialize;
use status::Status;
use std::cell::RefCell;
use std::fmt;
use std::future::Future;
use std::io;
use std::ops::Add;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time::{Sleep, sleep};
use tokio_util::sync::CancellationToken;
use tonic::transport::server::TcpIncoming;

/// URN used to register this receiver implementation in the engine.
const OTEL_RECEIVER_URN: &str = "urn:otel:otel:receiver";

/// Configuration for the experimental OTEL receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC server settings reused across receivers.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,
}

/// Experimental OTEL receiver that speaks OTLP and OTAP Arrow over pure `h2`.
pub struct OtelReceiver {
    config: Config,
    metrics: MetricSet<OtapReceiverMetrics>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Registers the receiver in the distributed slice used by the engine.
pub static OTEL_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTEL_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        let mut receiver = OtelReceiver::from_config(pipeline, &node_config.config)?;
        receiver.tune_max_concurrent_requests(receiver_config.output_pdata_channel.capacity);
        Ok(ReceiverWrapper::local(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl OtelReceiver {
    /// Builds a receiver instance from the user configuration stored on the node.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let metrics = pipeline_ctx.register_metrics::<OtapReceiverMetrics>();
        Ok(Self { config, metrics })
    }

    fn tune_max_concurrent_requests(&mut self, downstream_capacity: usize) {
        common::tune_max_concurrent_requests(&mut self.config.grpc, downstream_capacity);
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for OtelReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Admission control and TCP listener.
        // ToDo The hardcoded limits here are intentionally conservative and will be made tunable soon in a follow-up PR
        let admitter = Admitter::new(
            100000,
            self.config.grpc.max_concurrent_streams.unwrap_or(100),
            100000,
        );

        let config = Rc::new(self.config.grpc.clone());
        let listener = effect_handler.tcp_listener(config.listening_addr)?;
        let mut incoming = config.build_tcp_incoming(listener);

        let settings = Settings {
            max_concurrent_requests: config.max_concurrent_requests,
            wait_for_result: config.wait_for_result,
        };
        let max_in_flight_per_connection = per_connection_limit(&settings);

        // Per signal Ack registries.
        // If `wait_for_result` is disabled we skip creating registries entirely and respond
        // immediately after enqueueing work into the pipeline.
        let logs_ack_registry = settings
            .wait_for_result
            .then(|| AckRegistry::new(settings.max_concurrent_requests));
        let metrics_ack_registry = settings
            .wait_for_result
            .then(|| AckRegistry::new(settings.max_concurrent_requests));
        let traces_ack_registry = settings
            .wait_for_result
            .then(|| AckRegistry::new(settings.max_concurrent_requests));
        let ack_registries = AckRegistries::new(
            logs_ack_registry.clone(),
            metrics_ack_registry.clone(),
            traces_ack_registry.clone(),
        );

        // Compression configuration and response templates. We precompute:
        // - which request encodings are allowed,
        // - the `grpc-accept-encoding` header we advertise, and
        // - response header templates for each configured response compression method.
        let request_encoding_methods = config.request_compression_methods();
        let request_encodings = AcceptedGrpcEncodings::from_methods(&request_encoding_methods);
        let request_accept_header = build_accept_encoding_header(&request_encoding_methods);

        let response_methods = config.response_compression_methods();
        let response_templates = response_methods.iter().copied().fold(
            ResponseTemplates::new(request_accept_header.clone()),
            |acc, method| acc.with_method(method, &request_accept_header),
        );

        // The encoder pool size is tied to the number of concurrent requests.
        let encoder_pool_capacity = settings.max_concurrent_requests.max(1);
        let response_encoders = ResponseEncoderPool::new(&response_methods, encoder_pool_capacity);

        // Router shared by all h2 streams on this thread. The router holds shared state.
        let router = Rc::new(GrpcRequestRouter {
            effect_handler: effect_handler.clone(),
            logs_ack_registry,
            metrics_ack_registry,
            traces_ack_registry,
            max_in_flight_per_connection,
            request_encodings,
            request_accept_header: request_accept_header.clone(),
            response_methods,
            request_timeout: config.timeout,
            response_encoders,
            response_templates,
            // Shared zstd decompressor used by gRPC bodies on the current core.
            zstd_decompressor: RefCell::new(None),
        });

        // Telemetry and cancellation.
        let cancel_token = CancellationToken::new();
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // Local enum that lets us treat control and server completion uniformly without relying on
        // `select!` macros (so no branch cancellation).
        enum DriverEvent {
            Control(Result<TerminalState, Error>),
            Server(Result<(), io::Error>),
        }

        // Control plane loop:
        // - handles shutdown,
        // - exposes metrics snapshots,
        // - routes Ack and Nack control messages back into the Ack registries.
        let control_loop = Box::pin(async {
            loop {
                match ctrl_msg_recv.recv().await {
                    Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                        let snapshot = self.metrics.snapshot();
                        _ = telemetry_cancel_handle.cancel().await;
                        return Ok(TerminalState::new(deadline, [snapshot]));
                    }
                    Ok(NodeControlMsg::CollectTelemetry { metrics_reporter }) => {
                        // Best effort metrics push; errors are ignored.
                        _ = metrics_reporter.report(&mut self.metrics);
                    }
                    Ok(NodeControlMsg::Ack(ack)) => {
                        let result = route_ack_response(&ack_registries, ack);
                        common::handle_route_response(
                            result,
                            &mut self.metrics,
                            |metrics| metrics.acks_sent.inc(),
                            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                        );
                    }
                    Ok(NodeControlMsg::Nack(nack)) => {
                        let result = route_nack_response(&ack_registries, nack);
                        common::handle_route_response(
                            result,
                            &mut self.metrics,
                            |metrics| metrics.nacks_sent.inc(),
                            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                        );
                    }
                    Err(e) => return Err(Error::ChannelRecvError(e)),
                    // Other control messages can be added here when needed.
                    _ => {}
                }
            }
        });

        // Data plane loop that accepts TCP connections and drives the h2 server.
        let grpc_loop = Box::pin(run_grpc_server(
            &mut incoming,
            config,
            router,
            cancel_token.clone(),
            admitter.clone(),
        ));

        // We manually poll both futures and stop as soon as either finishes.
        let mut control_future = control_loop;
        let mut server_future = grpc_loop;

        let first_loop_done = futures::future::poll_fn(|cx| {
            if let Poll::Ready(res) = control_future.as_mut().poll(cx) {
                return Poll::Ready(DriverEvent::Control(res));
            }
            if let Poll::Ready(res) = server_future.as_mut().poll(cx) {
                return Poll::Ready(DriverEvent::Server(res));
            }
            Poll::Pending
        })
        .await;

        let server_done = match first_loop_done {
            DriverEvent::Control(ctrl_msg_result) => {
                cancel_token.cancel();
                return ctrl_msg_result;
            }
            DriverEvent::Server(server_result) => {
                if let Err(error) = server_result {
                    log::error!("OTEL H2 receiver server loop failed: {error}");
                    let source_detail = format_error_sources(&error);
                    return Err(Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Transport,
                        error: error.to_string(),
                        source_detail,
                    });
                }
                true
            }
        };

        drop(control_future);
        drop(server_future);

        if server_done {
            return Ok(TerminalState::new(
                Instant::now().add(Duration::from_secs(1)),
                [self.metrics],
            ));
        }

        Ok(TerminalState::new(
            Instant::now().add(Duration::from_secs(1)),
            [self.metrics],
        ))
    }
}

/// Build an `h2::server::Builder` from the configured gRPC settings.
///
/// Only values that are explicitly configured are applied. Everything else uses
/// the default from the `h2` library.
fn build_h2_builder(settings: &GrpcServerSettings) -> server::Builder {
    let mut builder = server::Builder::new();
    if let Some(window) = settings.initial_stream_window_size {
        let _ = builder.initial_window_size(window);
    }
    if let Some(window) = settings.initial_connection_window_size {
        let _ = builder.initial_connection_window_size(window);
    }
    if let Some(frame) = settings.max_frame_size {
        let _ = builder.max_frame_size(frame);
    }
    builder
}

/// Top level h2 server loop.
///
/// This function:
/// - accepts new TCP connections from `incoming`,
/// - applies admission control for each connection,
/// - spawns a per connection task that drives the h2 state machine, and
/// - listens for cancellation so we can drain gracefully on shutdown.
async fn run_grpc_server(
    incoming: &mut TcpIncoming,
    grpc_config: Rc<GrpcServerSettings>,
    grpc_router: Rc<GrpcRequestRouter>,
    cancel: CancellationToken,
    admitter: Admitter,
) -> Result<(), io::Error> {
    // Track all per TCP connection tasks.
    let mut tcp_conn_tasks: JoinSet<()> = JoinSet::new();
    let mut accepting = true;
    let h2_builder = build_h2_builder(&grpc_config);
    let mut cancel_wait = Box::pin(cancel.cancelled());

    loop {
        // Drain completed connection tasks without awaiting the whole set.
        // This lives outside the select style loop to avoid interfering with accept.
        while let Some(res) = tcp_conn_tasks.join_next().now_or_never().flatten() {
            if let Err(join_err) = res {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!("h2 connection task join error: {join_err}");
                }
            }
        }

        enum ServerEvent {
            Cancel,
            Accept(Result<tokio::net::TcpStream, io::Error>),
            IncomingClosed,
        }

        let event = futures::future::poll_fn(|cx| {
            if cancel_wait.as_mut().poll(cx).is_ready() {
                return Poll::Ready(ServerEvent::Cancel);
            }

            // Propagate any join errors from connection tasks as debug logs.
            if !tcp_conn_tasks.is_empty() {
                if let Poll::Ready(Some(Err(join_err))) = tcp_conn_tasks.poll_join_next(cx) {
                    if log::log_enabled!(log::Level::Debug) {
                        log::debug!("h2 connection task join error: {join_err}");
                    }
                }
            }

            if accepting {
                match StreamExt::poll_next_unpin(incoming, cx) {
                    Poll::Ready(Some(res)) => return Poll::Ready(ServerEvent::Accept(res)),
                    Poll::Ready(None) => return Poll::Ready(ServerEvent::IncomingClosed),
                    Poll::Pending => {}
                }
            }

            Poll::Pending
        })
        .await;

        match event {
            ServerEvent::Cancel => break,
            ServerEvent::IncomingClosed => {
                // No more connections from the listener. We keep running until all
                // existing connection tasks finish.
                accepting = false;
            }
            ServerEvent::Accept(res) => match res {
                Ok(tcp_conn) => {
                    // Admission control runs before we spawn the connection task.
                    match admitter.try_admit_connection() {
                        AdmitDecision::Admitted(conn_guard) => {
                            let h2_builder = h2_builder.clone();
                            let router = Rc::clone(&grpc_router);
                            let keepalive_interval = grpc_config.http2_keepalive_interval;
                            let keepalive_timeout = grpc_config.http2_keepalive_timeout;

                            // Each connection holds its admission guard until it finishes.
                            // The AbortHandler from the admitter is currently unused.
                            _ = tcp_conn_tasks.spawn_local(async move {
                                if let Err(err) = handle_tcp_conn(
                                    tcp_conn,
                                    h2_builder,
                                    router,
                                    conn_guard,
                                    keepalive_interval,
                                    keepalive_timeout,
                                )
                                .await
                                {
                                    if log::log_enabled!(log::Level::Debug) {
                                        log::debug!("h2 connection ended with error: {err}");
                                    }
                                }
                            });
                        }
                        AdmitDecision::Busy => {
                            // Soft backpressure: drop the stream so the kernel backlog can absorb spikes.
                            if log::log_enabled!(log::Level::Trace) {
                                log::trace!("Connection admission busy; pausing accepts briefly");
                            }
                            drop(tcp_conn);
                            // Yield to avoid a tight accept/reject loop.
                            tokio::task::yield_now().await;
                        }
                        AdmitDecision::Reject { message } => {
                            // Hard policy style rejection (circuit breaker etc).
                            if log::log_enabled!(log::Level::Warn) {
                                log::warn!("Connection admission rejected: {message}");
                            }
                            drop(tcp_conn);
                        }
                    }
                }
                Err(err) => return Err(err),
            },
        }

        // Once no more accepts will arrive and all tasks are drained we can exit.
        if !accepting && tcp_conn_tasks.is_empty() {
            break;
        }
    }

    // Graceful drain after cancellation or listener close.
    while let Some(join_res) = tcp_conn_tasks.join_next().await {
        if let Err(join_err) = join_res {
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("h2 connection task join error: {join_err}");
            }
        }
    }

    Ok(())
}

/// Drives a single TCP connection through the h2 server state machine.
///
/// Responsibilities:
/// - perform the h2 handshake,
/// - accept inbound streams (HTTP/2 requests),
/// - enforce per connection stream admission, and
/// - keep the connection alive via HTTP/2 PING when configured.
async fn handle_tcp_conn(
    socket: tokio::net::TcpStream,
    builder: server::Builder,
    router: Rc<GrpcRequestRouter>,
    // Keeps one connection slot while the connection is alive.
    tcp_conn_guard: ConnectionGuard,
    keepalive_interval: Option<Duration>,
    keepalive_timeout: Option<Duration>,
) -> Result<(), h2::Error> {
    // HTTP/2 handshake.
    let mut http2_conn = builder.handshake(socket).await?;
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("h2 handshake established");
    }

    let keepalive = Http2Keepalive::new(
        http2_conn.ping_pong(),
        keepalive_interval,
        keepalive_timeout,
    );

    // Wrap the connection in a pinned future so we can build a stream over `poll_accept`.
    let mut http2_conn = Box::pin(http2_conn);
    let mut accept_stream = futures::stream::poll_fn(move |cx| http2_conn.as_mut().poll_accept(cx));

    // Keepalive ticks are driven by a custom stream so we reuse timers.
    let mut keepalive_stream = KeepaliveStream::new(keepalive);
    let mut in_flight = FuturesUnordered::new();
    let mut accepting = true;
    let mut idle_spins: u8 = 0;

    let trace_enabled = log::log_enabled!(log::Level::Trace);
    let debug_enabled = log::log_enabled!(log::Level::Debug);

    loop {
        // Keepalive is only armed when there are no in flight request tasks.
        keepalive_stream.set_idle(in_flight.is_empty());

        let next_event = futures::future::poll_fn(|cx| {
            // 1. Drain completed in flight request tasks.
            if let Poll::Ready(Some(_)) = Pin::new(&mut in_flight).poll_next(cx) {
                return Poll::Ready(Some(StreamEvent::Task));
            }

            // 2. Drive keepalive ticks if configured.
            if keepalive_stream.is_active() {
                if let Poll::Ready(ev) = Pin::new(&mut keepalive_stream).poll_next(cx) {
                    return Poll::Ready(ev);
                }
            }

            // 3. Accept new streams on this connection.
            if accepting {
                match Pin::new(&mut accept_stream).poll_next(cx) {
                    Poll::Ready(Some(res)) => {
                        return Poll::Ready(Some(StreamEvent::Accept(Box::new(res))));
                    }
                    Poll::Ready(None) => {
                        return Poll::Ready(Some(StreamEvent::AcceptClosed));
                    }
                    Poll::Pending => {}
                }
            }

            Poll::Pending
        })
        .await;

        match next_event {
            Some(StreamEvent::Accept(result)) => {
                idle_spins = 0;
                match *result {
                    Ok((request, respond)) => {
                        // Per stream admission for this connection.
                        match tcp_conn_guard.try_open_stream() {
                            AdmitDecision::Admitted(stream_guard) => {
                                let router = router.clone();
                                in_flight.push(async move {
                                    if trace_enabled {
                                        log::trace!("New h2 stream: {}", request.uri().path());
                                    }
                                    if let Err(status) =
                                        router.route_grpc_request(request, respond).await
                                    {
                                        if debug_enabled {
                                            log::debug!("Request failed: {}", status);
                                        }
                                    }
                                    // Release per stream admission slot.
                                    drop(stream_guard);
                                });
                            }
                            AdmitDecision::Busy => {
                                // Per connection stream capacity is full: reply with RESOURCE_EXHAUSTED.
                                respond_with_error(
                                    respond,
                                    Status::resource_exhausted("stream capacity exhausted"),
                                    &router.request_accept_header,
                                );
                            }
                            AdmitDecision::Reject { message } => {
                                // Policy level rejection of this stream.
                                respond_with_error(
                                    respond,
                                    Status::unavailable(message),
                                    &router.request_accept_header,
                                );
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            Some(StreamEvent::AcceptClosed) => {
                idle_spins = 0;
                accepting = false;
            }
            Some(StreamEvent::Keepalive(result)) => {
                idle_spins = 0;
                if let Err(err) = result {
                    if log::log_enabled!(log::Level::Debug) {
                        log::debug!("h2 keepalive failed: {err}");
                    }
                    break;
                }
            }
            Some(StreamEvent::Task) => {
                idle_spins = 0;
            }
            None => {
                // No work this tick. After a couple of tight spins yield to avoid burning CPU.
                idle_spins = idle_spins.saturating_add(1);
                if idle_spins >= 2 {
                    tokio::task::yield_now().await;
                    idle_spins = 0;
                }
            }
        }

        // Exit once there are no more streams to accept, no pending tasks, and keepalive is idle.
        if !accepting && in_flight.is_empty() && !keepalive_stream.is_active() {
            break;
        }
    }

    Ok(())
}

/// Tracks whether a connection needs an HTTP/2 PING to keep the client alive.
///
/// The keepalive logic is intentionally simple:
/// - only runs when the connection is idle,
/// - arms a timer for the configured interval, and
/// - waits for a matching PONG within the configured timeout.
struct Http2Keepalive {
    ping_pong: PingPong,
    interval: Duration,
    timeout: Duration,
    sleep: Option<Pin<Box<Sleep>>>,
}

impl Http2Keepalive {
    fn new(
        ping_pong: Option<PingPong>,
        interval: Option<Duration>,
        timeout: Option<Duration>,
    ) -> Option<Self> {
        let (ping_pong, interval, timeout) = match (ping_pong, interval, timeout) {
            (Some(ping_pong), Some(interval), Some(timeout)) if !interval.is_zero() => {
                (ping_pong, interval, timeout)
            }
            _ => return None,
        };
        Some(Self {
            ping_pong,
            interval,
            timeout,
            sleep: None,
        })
    }

    /// Updates the idle state of the connection.
    ///
    /// When the connection becomes idle we arm a sleep; once the connection becomes active
    /// again we drop it so the next idle period starts a fresh timer.
    fn update_idle_state(&mut self, idle: bool) {
        if idle {
            if self.sleep.is_none() {
                self.sleep = Some(Box::pin(sleep(self.interval)));
            }
        } else if self.sleep.is_some() {
            self.sleep = None;
        }
    }

    fn is_armed(&self) -> bool {
        self.sleep.is_some()
    }

    /// Waits for the next keepalive interval and fires a PING.
    ///
    /// If the PING does not complete within the configured timeout this returns an error
    /// and the connection is closed.
    async fn poll_tick(&mut self) -> Result<(), Http2KeepaliveError> {
        let mut sleeper = self
            .sleep
            .take()
            .expect("keepalive polled without being armed");
        sleeper.as_mut().await;
        self.sleep = Some(Box::pin(sleep(self.interval)));

        match tokio::time::timeout(self.timeout, self.ping_pong.ping(Ping::opaque())).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => Err(Http2KeepaliveError::Ping(err)),
            Err(_) => Err(Http2KeepaliveError::Timeout),
        }
    }
}

enum Http2KeepaliveError {
    Timeout,
    Ping(h2::Error),
}

impl fmt::Display for Http2KeepaliveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "keepalive timeout waiting for PONG"),
            Self::Ping(err) => write!(f, "keepalive ping failed: {err}"),
        }
    }
}

/// Events that can drive progress on a connection:
/// - a newly accepted stream,
/// - the accept side closing,
/// - a keepalive tick firing, or
/// - a previously spawned request task finishing.
enum StreamEvent {
    Accept(Box<Result<(Request<h2::RecvStream>, SendResponse<Bytes>), h2::Error>>),
    AcceptClosed,
    Keepalive(Result<(), Http2KeepaliveError>),
    Task,
}

/// Stream wrapper over `Http2Keepalive` that plugs into the same poll loop as new streams.
///
/// The keepalive stream is only active when a configured keepalive exists and the
/// connection is idle.
struct KeepaliveStream {
    keepalive: Option<Http2Keepalive>,
    tick: Option<KeepaliveTick>,
    idle: bool,
    idle_streak: u8,
}

/// Future alias used by `KeepaliveStream` for a single keepalive cycle.
type KeepaliveTick =
    Pin<Box<dyn Future<Output = (Result<(), Http2KeepaliveError>, Http2Keepalive)> + 'static>>;

impl KeepaliveStream {
    fn new(keepalive: Option<Http2Keepalive>) -> Self {
        Self {
            keepalive,
            tick: None,
            idle: true,
            idle_streak: 0,
        }
    }

    /// Mark the connection as idle or active.
    ///
    /// When we observe consecutive idle polls the underlying keepalive is armed.
    fn set_idle(&mut self, idle: bool) {
        self.idle = idle;
        if !idle {
            // Drop any pending tick so the next idle cycle re arms from scratch.
            self.tick = None;
            self.idle_streak = 0;
        } else {
            self.idle_streak = self.idle_streak.saturating_add(1);
        }
    }

    fn is_active(&self) -> bool {
        self.keepalive.is_some() || self.tick.is_some()
    }
}

impl Stream for KeepaliveStream {
    type Item = StreamEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if !this.is_active() {
            return Poll::Pending;
        }

        loop {
            // First drive an already armed keepalive tick to completion.
            if let Some(tick) = this.tick.as_mut() {
                match tick.as_mut().poll(cx) {
                    Poll::Ready((res, ka)) => {
                        this.tick = None;
                        this.keepalive = Some(ka);
                        return Poll::Ready(Some(StreamEvent::Keepalive(res)));
                    }
                    Poll::Pending => return Poll::Pending,
                }
            }

            let Some(mut keepalive) = this.keepalive.take() else {
                return Poll::Pending;
            };

            // We only arm keepalive after a couple of consecutive idle polls.
            if !this.idle || this.idle_streak < 2 {
                this.keepalive = Some(keepalive);
                return Poll::Pending;
            }

            keepalive.update_idle_state(true);
            if keepalive.is_armed() {
                this.tick = Some(Box::pin(async move {
                    let res = keepalive.poll_tick().await;
                    (res, keepalive)
                }));
                // Immediately poll the newly created tick.
                continue;
            } else {
                this.keepalive = Some(keepalive);
                return Poll::Pending;
            }
        }
    }
}
