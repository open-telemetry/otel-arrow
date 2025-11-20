// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental receiver that serves OTLP and OTAP endpoints directly on top of the `h2`crate.
//! This variant keeps all request handling on the current thread so it can integrate with the
//! thread-per-core runtime without requiring `Send + Sync` futures.
//!
//! Design goals:
//! - Support OTLP and OTAP Arrow over gRPC with minimal dependencies.
//! - Avoid `Send + Sync` bounds on request handlers to integrate with thread-per-core runtime.
//! - No Arc, Mutex dependencies in the hot path.
//! - Low use of heap allocations in the hot path.
//!
//! ToDo Add snappy support. Wire in the matching decompress/encode routines with shared helpers for both request frames and response frames.
//! ToDo Improve error handling & metrics: surface clear statuses when the client requests unsupported codecs, log negotiation results, and add counters for negotiated/unsupported compression cases.
//! ToDo Add support for Unix domain sockets as a transport option.

mod ack;
pub(crate) mod grpc;
mod response_templates;
mod stream;

use crate::OTAP_RECEIVER_FACTORIES;
use crate::compression::CompressionMethod;
use crate::otap_grpc::common;
use crate::otap_grpc::{GrpcServerSettings, Settings, per_connection_limit};
use crate::otap_receiver::OtapReceiverMetrics;
use crate::pdata::{Context, OtapPdata};
use response_templates::ResponseTemplates;
use ack::{
    AckCompletionFuture, AckPollResult, AckRegistries, AckRegistry, route_ack_response,
    route_nack_response,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::{FutureExt, Stream, StreamExt};
use grpc::{
    AcceptedGrpcEncodings, GrpcMessageEncoder, GrpcStreamingBody, RequestTimeout,
    build_accept_encoding_header, grpc_encoding_token, negotiate_response_encoding,
    parse_grpc_accept_encoding, parse_grpc_encoding,
};
use h2::server::{self, SendResponse};
use h2::{Ping, PingPong};
use http::{HeaderMap, HeaderValue, Request, Response, StatusCode as HttpStatusCode};
use linkme::distributed_slice;
use otap_df_config::SignalType;
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
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use otap_df_telemetry::metrics::MetricSet;
use serde::Deserialize;
use std::fmt;
use std::future::Future;
use std::io;
use std::ops::Add;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};
use stream::stream_batch_statuses;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{Sleep, sleep};
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tonic::transport::server::TcpIncoming;

const OTEL_RECEIVER_URN: &str = "urn:otel:otel:receiver";

const ARROW_LOGS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowLogsService/ArrowLogs";
const ARROW_METRICS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowMetricsService/ArrowMetrics";
const ARROW_TRACES_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowTracesService/ArrowTraces";

const OTLP_LOGS_SERVICE: &str = "/opentelemetry.proto.collector.logs.v1.LogsService/Export";
const OTLP_METRICS_SERVICE: &str =
    "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export";
const OTLP_TRACES_SERVICE: &str = "/opentelemetry.proto.collector.trace.v1.TraceService/Export";

/// Configuration for the OTEL receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC server settings reused across receivers.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,
}

/// Experimental OTEL receiver powered directly by the `h2` crate.
pub struct OtelReceiver {
    config: Config,
    metrics: MetricSet<OtapReceiverMetrics>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Registers the experimental OTEL receiver factory.
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
    /// Builds a receiver instance from a user configuration blob.
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
        let max_in_flight = per_connection_limit(&settings);

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

        let request_encoding_methods = config.request_compression_methods();
        let request_encodings = AcceptedGrpcEncodings::from_methods(&request_encoding_methods);
        let request_accept_header = build_accept_encoding_header(&request_encoding_methods);
        let response_methods = config.response_compression_methods();
        let response_templates = response_methods
            .iter()
            .copied()
            .fold(ResponseTemplates::new(request_accept_header.clone()), |acc, method| {
                acc.with_method(method, &request_accept_header)
            });

        let router = Rc::new(GrpcRequestRouter {
            effect_handler: effect_handler.clone(),
            logs_ack_registry,
            metrics_ack_registry,
            traces_ack_registry,
            max_in_flight_per_connection: max_in_flight,
            request_encodings,
            request_accept_header: request_accept_header.clone(),
            response_methods,
            request_timeout: config.timeout,
            response_templates,
        });

        let cancel_token = CancellationToken::new();

        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // log::info!("OTAP H2 receiver starting on {}", config.listening_addr);

        enum StartEvent {
            Ctrl(Result<TerminalState, Error>),
            Server(Result<(), io::Error>),
        }

        let ctrl_fut = Box::pin(async {
            loop {
                match ctrl_msg_recv.recv().await {
                    Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                        // log::info!("OTAP H2 receiver received shutdown signal");
                        let snapshot = self.metrics.snapshot();
                        _ = telemetry_cancel_handle.cancel().await;
                        return Ok(TerminalState::new(deadline, [snapshot]));
                    }
                    Ok(NodeControlMsg::CollectTelemetry { metrics_reporter }) => {
                        _ = metrics_reporter.report(&mut self.metrics);
                    }
                    Ok(NodeControlMsg::Ack(ack)) => {
                        let resp = route_ack_response(&ack_registries, ack);
                        common::handle_route_response(
                            resp,
                            &mut self.metrics,
                            |metrics| metrics.acks_sent.inc(),
                            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                        );
                    }
                    Ok(NodeControlMsg::Nack(nack)) => {
                        let resp = route_nack_response(&ack_registries, nack);
                        common::handle_route_response(
                            resp,
                            &mut self.metrics,
                            |metrics| metrics.nacks_sent.inc(),
                            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                        );
                    }
                    Err(e) => return Err(Error::ChannelRecvError(e)),
                    _ => {}
                }
            }
        });

        let server_fut = Box::pin(run_grpc_server(
            &mut incoming,
            config,
            router,
            cancel_token.clone(),
            admitter.clone(),
        ));

        let server_done;
        let mut ctrl_fut = ctrl_fut;
        let mut server_fut = server_fut;

        let event = futures::future::poll_fn(|cx| {
            if let Poll::Ready(res) = ctrl_fut.as_mut().poll(cx) {
                return Poll::Ready(StartEvent::Ctrl(res));
            }
            if let Poll::Ready(res) = server_fut.as_mut().poll(cx) {
                return Poll::Ready(StartEvent::Server(res));
            }
            Poll::Pending
        })
        .await;

        match event {
            StartEvent::Ctrl(ctrl_msg_result) => {
                cancel_token.cancel();
                return ctrl_msg_result;
            }
            StartEvent::Server(server_result) => {
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
                server_done = true;
            }
        }

        drop(ctrl_fut);
        drop(server_fut);

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

async fn run_grpc_server(
    incoming: &mut TcpIncoming,
    grpc_config: Rc<GrpcServerSettings>,
    grpc_router: Rc<GrpcRequestRouter>,
    cancel: CancellationToken,
    admitter: Admitter,
) -> Result<(), io::Error> {
    // Track all per-tcp-connection tasks.
    let mut tcp_conn_tasks: JoinSet<()> = JoinSet::new();
    let mut accepting = true;
    let h2_builder = build_h2_builder(&grpc_config);
    let mut cancel_wait = Box::pin(cancel.cancelled());

    loop {
        // Drain completed connection tasks without awaiting.
        // This is outside the select! to avoid cancelling a pending accept.
        while let Some(res) = tcp_conn_tasks.join_next().now_or_never().flatten() {
            if let Err(join_err) = res {
                if log::log_enabled!(log::Level::Debug) {
                    log::debug!("H2 connection task join error: {join_err}");
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

            if !tcp_conn_tasks.is_empty() {
                if let Poll::Ready(Some(res)) = tcp_conn_tasks.poll_join_next(cx) {
                    if let Err(join_err) = res {
                        if log::log_enabled!(log::Level::Debug) {
                            log::debug!("H2 connection task join error: {join_err}");
                        }
                    }
                    // Continue polling for other events this tick.
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
            ServerEvent::IncomingClosed => accepting = false,
            ServerEvent::Accept(res) => {
                match res {
                    Ok(tcp_conn) => {
                        // Admit a connection before spawning the task.
                        match admitter.try_admit_connection() {
                            AdmitDecision::Admitted(conn_guard) => {
                                let h2_builder = h2_builder.clone();
                                let router = Rc::clone(&grpc_router);
                                let keepalive_interval = grpc_config.http2_keepalive_interval;
                                let keepalive_timeout = grpc_config.http2_keepalive_timeout;

                                // Hold `conn_guard` inside the task for the connection lifetime.
                                // Ignore the AbortHandler for now.
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
                                            log::debug!("H2 connection ended with error: {err}");
                                        }
                                    }
                                });
                            }
                            AdmitDecision::Busy => {
                                // Soft backpressure: do not spawn; let the kernel backlog absorb.
                                if log::log_enabled!(log::Level::Trace) {
                                    log::trace!("Connection admission busy; pausing accepts briefly");
                                }
                                drop(tcp_conn);
                                // Yield to avoid a tight loop.
                                tokio::task::yield_now().await;
                            }
                            AdmitDecision::Reject { message } => {
                                // Hard reject (circuit breaker, policy). Drop the TCP stream.
                                if log::log_enabled!(log::Level::Warn) {
                                    log::warn!("Connection admission rejected: {message}");
                                }
                                drop(tcp_conn);
                            }
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        // If no more accepts will arrive and all tasks are done, we can exit
        if !accepting && tcp_conn_tasks.is_empty() {
            break;
        }
    }

    // Graceful drain after cancellation or incoming end
    while let Some(join_res) = tcp_conn_tasks.join_next().await {
        if let Err(join_err) = join_res {
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("H2 connection task join error: {join_err}");
            }
        }
    }

    Ok(())
}

async fn handle_tcp_conn(
    socket: tokio::net::TcpStream,
    builder: server::Builder,
    router: Rc<GrpcRequestRouter>,
    // IMPORTANT: this keeps one connection slot while the connection is alive.
    tcp_conn_guard: ConnectionGuard,
    keepalive_interval: Option<Duration>,
    keepalive_timeout: Option<Duration>,
) -> Result<(), h2::Error> {
    // HTTP/2 handshake
    let mut http2_conn = builder.handshake(socket).await?;
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("H2 handshake established");
    }
    let keepalive = Http2Keepalive::new(
        http2_conn.ping_pong(),
        keepalive_interval,
        keepalive_timeout,
    );

    // Drive accepts through a stream so we keep wakers alive across polls.
    let mut http2_conn = Box::pin(http2_conn);
    let mut accept_stream =
        futures::stream::poll_fn(move |cx| http2_conn.as_mut().poll_accept(cx));
    // Drive keepalive ticks through a stream to avoid recreating futures.
    let mut keepalive_stream = KeepaliveStream::new(keepalive);
    let mut tasks_stream = TaskJoinStream::new();
    let mut accepting = true;
    let mut idle_spins: u8 = 0;
    let trace_enabled = log::log_enabled!(log::Level::Trace);
    let debug_enabled = log::log_enabled!(log::Level::Debug);

    loop {
        // Keepalive only armed when idle; piggyback on task stream emptiness.
        keepalive_stream.set_idle(tasks_stream.is_empty());

        let next_event = futures::future::poll_fn(|cx| {
            // Drain completed tasks first.
            if let Poll::Ready(ev) = Pin::new(&mut tasks_stream).poll_next(cx) {
                if ev.is_some() {
                    return Poll::Ready(ev);
                }
            }

            if keepalive_stream.is_active() {
                if let Poll::Ready(ev) = Pin::new(&mut keepalive_stream).poll_next(cx) {
                    return Poll::Ready(ev);
                }
            }

            if accepting {
                match Pin::new(&mut accept_stream).poll_next(cx) {
                    Poll::Ready(Some(res)) => {
                        return Poll::Ready(Some(StreamEvent::Accept(res)));
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
                match result {
                    Ok((request, respond)) => {
                        // Try to open a *stream* on this connection.
                        match tcp_conn_guard.try_open_stream() {
                            AdmitDecision::Admitted(stream_guard) => {
                                let router = router.clone();
                                // Keep `stream_guard` alive for the request lifetime.
                                // Ignore the AbortHandler for now.
                                tasks_stream.spawn_local(async move {
                                    if trace_enabled {
                                        // `request.uri()` is cheap, but we still avoid doing it if TRACE is off.
                                        log::trace!("New H2 stream: {}", request.uri().path());
                                    }
                                    if let Err(status) = router.route_grpc_request(request, respond).await {
                                        if debug_enabled {
                                            log::debug!("Request failed: {}", status);
                                        }
                                    }
                                    // release stream slot
                                    drop(stream_guard);
                                });
                            }
                            AdmitDecision::Busy => {
                                // Per-connection stream capacity is full: reply immediately.
                                respond_with_error(
                                    respond,
                                    Status::resource_exhausted(
                                        "stream capacity exhausted",
                                    ),
                                    &router.request_accept_header,
                                );
                            }
                            AdmitDecision::Reject { message } => {
                                // Breaker/policy: reject this stream immediately.
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
                match result {
                    Ok(()) => {}
                    Err(err) => {
                        if log::log_enabled!(log::Level::Debug) {
                            log::debug!("H2 keepalive failed: {err}");
                        }
                        break;
                    }
                }
            }
            Some(StreamEvent::Task(result)) => {
                idle_spins = 0;
                if let Err(join_err) = result {
                    if log::log_enabled!(log::Level::Debug) {
                        log::debug!("stream task join error: {join_err}");
                    }
                }
            }
            None => {
                idle_spins = idle_spins.saturating_add(1);
                if idle_spins >= 2 {
                    tokio::task::yield_now().await;
                    idle_spins = 0;
                }
            }
        }

        // Exit when no more streams will arrive and all tasks are done
        if !accepting && tasks_stream.is_empty() && !keepalive_stream.is_active() {
            break;
        }
    }

    Ok(())
}

/// Tracks whether a connection needs a HTTP/2 PING/PONG to keep the client alive.
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

enum StreamEvent {
    Accept(Result<(Request<h2::RecvStream>, SendResponse<Bytes>), h2::Error>),
    AcceptClosed,
    Keepalive(Result<(), Http2KeepaliveError>),
    Task(Result<(), JoinError>),
}

struct KeepaliveStream {
    keepalive: Option<Http2Keepalive>,
    tick: Option<KeepaliveTick>,
    idle: bool,
    idle_streak: u8,
}

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

    fn set_idle(&mut self, idle: bool) {
        self.idle = idle;
        if !idle {
            // Drop any pending tick so the next idle cycle re-arms from scratch.
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

            let Some(mut ka) = this.keepalive.take() else {
                return Poll::Pending;
            };

            if !this.idle || this.idle_streak < 2 {
                this.keepalive = Some(ka);
                return Poll::Pending;
            }

            ka.update_idle_state(true);
            if ka.is_armed() {
                this.tick = Some(Box::pin(async move {
                    let res = ka.poll_tick().await;
                    (res, ka)
                }));
                // Poll the newly created tick in the same call.
                continue;
            } else {
                this.keepalive = Some(ka);
                return Poll::Pending;
            }
        }
    }
}

struct TaskJoinStream {
    tasks: JoinSet<()>,
    inflight: usize,
}

impl TaskJoinStream {
    fn new() -> Self {
        Self {
            tasks: JoinSet::new(),
            inflight: 0,
        }
    }

    fn spawn_local(&mut self, fut: impl Future<Output = ()> + 'static) {
        self.inflight += 1;
        _ = self.tasks.spawn_local(fut);
    }

    fn is_empty(&self) -> bool {
        self.inflight == 0
    }
}

impl Stream for TaskJoinStream {
    type Item = StreamEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.tasks.poll_join_next(cx) {
            Poll::Ready(Some(res)) => {
                if this.inflight > 0 {
                    this.inflight -= 1;
                }
                Poll::Ready(Some(StreamEvent::Task(res)))
            }
            Poll::Ready(None) => Poll::Pending,
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Routes each inbound gRPC request to the appropriate OTLP+OTAP signal stream.
struct GrpcRequestRouter {
    effect_handler: local::EffectHandler<OtapPdata>,
    logs_ack_registry: Option<AckRegistry>,
    metrics_ack_registry: Option<AckRegistry>,
    traces_ack_registry: Option<AckRegistry>,
    max_in_flight_per_connection: usize,
    request_encodings: AcceptedGrpcEncodings,
    request_accept_header: HeaderValue,
    response_methods: Vec<CompressionMethod>,
    request_timeout: Option<Duration>,
    response_templates: ResponseTemplates,
}

impl GrpcRequestRouter {
    async fn route_grpc_request(
        self: Rc<Self>,
        request: Request<h2::RecvStream>,
        respond: SendResponse<Bytes>,
    ) -> Result<(), Status> {
        let path = request.uri().path();
        match path {
            ARROW_LOGS_SERVICE => {
                // log::info!("Handling ArrowLogs stream");
                self.serve_otap_stream::<Logs>(
                    request,
                    respond,
                    OtapArrowRecords::Logs,
                    self.logs_ack_registry.clone(),
                )
                .await
            }
            ARROW_METRICS_SERVICE => {
                // log::info!("Handling ArrowMetrics stream");
                self.serve_otap_stream::<Metrics>(
                    request,
                    respond,
                    OtapArrowRecords::Metrics,
                    self.metrics_ack_registry.clone(),
                )
                .await
            }
            ARROW_TRACES_SERVICE => {
                // log::info!("Handling ArrowTraces stream");
                self.serve_otap_stream::<Traces>(
                    request,
                    respond,
                    OtapArrowRecords::Traces,
                    self.traces_ack_registry.clone(),
                )
                .await
            }
            OTLP_LOGS_SERVICE => {
                self.serve_otlp_unary(
                    request,
                    respond,
                    SignalType::Logs,
                    self.logs_ack_registry.clone(),
                )
                .await
            }
            OTLP_METRICS_SERVICE => {
                self.serve_otlp_unary(
                    request,
                    respond,
                    SignalType::Metrics,
                    self.metrics_ack_registry.clone(),
                )
                .await
            }
            OTLP_TRACES_SERVICE => {
                self.serve_otlp_unary(
                    request,
                    respond,
                    SignalType::Traces,
                    self.traces_ack_registry.clone(),
                )
                .await
            }
            _ => {
                log::warn!("Unknown OTAP Arrow path {}", path);
                respond_with_error(
                    respond,
                    Status::unimplemented("unknown method"),
                    &self.request_accept_header,
                );
                Ok(())
            }
        }
    }

    async fn serve_otap_stream<T>(
        self: &Rc<Self>,
        request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        otap_batch: fn(T) -> OtapArrowRecords,
        ack_registry: Option<AckRegistry>,
    ) -> Result<(), Status>
    where
        T: OtapBatchStore + 'static,
    {
        let encoding = parse_grpc_encoding(request.headers(), &self.request_encodings)?;
        let client_accept = parse_grpc_accept_encoding(request.headers());
        let response_encoding = negotiate_response_encoding(&self.response_methods, &client_accept);
        let mut response_encoder = GrpcMessageEncoder::new(response_encoding);
        let recv_stream = request.into_body();
        let body = GrpcStreamingBody::new(recv_stream, encoding);

        let mut status_stream = stream_batch_statuses::<GrpcStreamingBody, T, _>(
            body,
            self.effect_handler.clone(),
            ack_registry,
            otap_batch,
            self.max_in_flight_per_connection,
        );

        let mut response_builder = Response::builder()
            .status(HttpStatusCode::OK)
            .header("content-type", "application/grpc")
            .header("grpc-accept-encoding", self.request_accept_header.clone());
        if let Some(token) = grpc_encoding_token(response_encoding) {
            response_builder = response_builder.header("grpc-encoding", token);
        }
        let response = response_builder
            .body(())
            .map_err(|e| Status::internal(format!("failed to build response: {e}")))?;
        let mut send_stream = respond
            .send_response(response, false)
            .map_err(|e| Status::internal(format!("failed to send response headers: {e}")))?;

        let mut request_timeout = RequestTimeout::new(self.request_timeout);

        loop {
            let next_item = match request_timeout.next_with(&mut status_stream).await {
                Ok(item) => item,
                Err(()) => {
                    if let Some(duration) = self.request_timeout {
                        log::debug!("Request timed out after {:?}", duration);
                    }
                    send_error_trailers(
                        send_stream,
                        Status::deadline_exceeded("request timed out"),
                    );
                    return Ok(());
                }
            };

            match next_item {
                Some(Ok(status)) => {
                    // log::info!(
                    //     "Sending batch status id={} code={}",
                    //     status.batch_id,
                    //     status.status_code
                    // );
                    let bytes = response_encoder.encode(&status)?;
                    if let Err(e) = send_stream.send_data(bytes, false) {
                        log::debug!("send_data failed: {e}");
                        return Ok(());
                    }
                }
                Some(Err(status)) => {
                    log::error!("Stream aborted with status {}", status);
                    send_error_trailers(send_stream, status);
                    return Ok(());
                }
                None => break,
            }
        }

        send_ok_trailers(send_stream);
        Ok(())
    }

    async fn serve_otlp_unary(
        self: &Rc<Self>,
        request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        signal: SignalType,
        ack_registry: Option<AckRegistry>,
    ) -> Result<(), Status> {
        let (parts, body) = request.into_parts();
        let encoding = parse_grpc_encoding(&parts.headers, &self.request_encodings)?;
        let client_accept = parse_grpc_accept_encoding(&parts.headers);
        let response_encoding = negotiate_response_encoding(&self.response_methods, &client_accept);
        let mut response_encoder = GrpcMessageEncoder::new(response_encoding);
        let mut recv_stream = GrpcStreamingBody::new(body, encoding);
        let mut request_timeout = RequestTimeout::new(self.request_timeout);

        let request_bytes = match request_timeout
            .with_future(recv_stream.next_message_bytes())
            .await
        {
            Ok(Ok(Some(bytes))) => bytes,
            Ok(Ok(None)) => {
                respond_with_error(
                    respond,
                    Status::invalid_argument("missing request body"),
                    &self.request_accept_header,
                );
                return Ok(());
            }
            Ok(Err(status)) => {
                respond_with_error(respond, status, &self.request_accept_header);
                return Ok(());
            }
            Err(()) => {
                if let Some(duration) = self.request_timeout {
                    log::debug!("Request timed out after {:?}", duration);
                }
                respond_with_error(
                    respond,
                    Status::deadline_exceeded("request timed out"),
                    &self.request_accept_header,
                );
                return Ok(());
            }
        };

        let mut otap_pdata = OtapPdata::new(
            Context::default(),
            otlp_proto_bytes(signal, request_bytes).into(),
        );

        let wait_token = if let Some(state) = ack_registry.as_ref() {
            match state.allocate() {
                Some(token) => {
                    self.effect_handler.subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        token.to_calldata(),
                        &mut otap_pdata,
                    );
                    Some((state.clone(), token))
                }
                None => {
                    respond_with_error(
                        respond,
                        Status::resource_exhausted("too many concurrent requests"),
                        &self.request_accept_header,
                    );
                    return Ok(());
                }
            }
        } else {
            None
        };

        if let Err(err) = self.effect_handler.send_message(otap_pdata).await {
            log::error!("Failed to send to pipeline: {err}");
            respond_with_error(
                respond,
                Status::internal("failed to send to pipeline"),
                &self.request_accept_header,
            );
            return Ok(());
        }

        if let Some((state, token)) = wait_token {
            let ack_future = AckCompletionFuture::new(token, state);
            let ack_result = match request_timeout.with_future(ack_future).await {
                Ok(result) => result,
                Err(()) => {
                    if let Some(duration) = self.request_timeout {
                        log::debug!("Request timed out after {:?}", duration);
                    }
                    respond_with_error(
                        respond,
                        Status::deadline_exceeded("request timed out"),
                        &self.request_accept_header,
                    );
                    return Ok(());
                }
            };
            match ack_result {
                AckPollResult::Ack => {}
                AckPollResult::Nack(reason) => {
                    respond_with_error(
                        respond,
                        Status::unavailable(format!("Pipeline processing failed: {reason}")),
                        &self.request_accept_header,
                    );
                    return Ok(());
                }
                AckPollResult::Cancelled => {
                    respond_with_error(
                        respond,
                        Status::internal("request cancelled"),
                        &self.request_accept_header,
                    );
                    return Ok(());
                }
            }
        }

        let response = self
            .response_templates
            .get_ok(CompressionMethod::from_grpc_encoding(response_encoding))
            .ok_or_else(|| Status::internal("failed to build response"))?;
        let mut send_stream = respond
            .send_response(response, false)
            .map_err(|e| Status::internal(format!("failed to send response headers: {e}")))?;

        let payload = encode_otlp_response(signal, &mut response_encoder)?;
        if let Err(e) = send_stream.send_data(payload, false) {
            log::debug!("send_data failed: {e}");
            return Ok(());
        }

        send_ok_trailers(send_stream);
        Ok(())
    }
}

fn send_ok_trailers(mut stream: h2::SendStream<Bytes>) {
    let mut trailers = HeaderMap::new();
    let _ = trailers.insert("grpc-status", HeaderValue::from_static("0"));
    if let Err(e) = stream.send_trailers(trailers) {
        log::debug!("send_trailers failed: {e}");
    }
}

fn send_error_trailers(mut stream: h2::SendStream<Bytes>, status: Status) {
    let mut trailers = HeaderMap::new();
    let _ = trailers.insert(
        "grpc-status",
        HeaderValue::from_str(&(status.code() as i32).to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("2")),
    );
    if !status.message().is_empty() {
        if let Ok(value) = HeaderValue::from_str(status.message()) {
            let _ = trailers.insert("grpc-message", value);
        }
    }
    if let Err(e) = stream.send_trailers(trailers) {
        log::debug!("send_trailers failed: {e}");
    }
}

fn respond_with_error(
    mut respond: SendResponse<Bytes>,
    status: Status,
    accept_header: &HeaderValue,
) {
    let response = match Response::builder()
        .status(HttpStatusCode::OK)
        .header("content-type", "application/grpc")
        .header("grpc-accept-encoding", accept_header.clone())
        .body(())
    {
        Ok(response) => response,
        Err(e) => {
            log::debug!("failed to build error response: {e}");
            return;
        }
    };

    match respond.send_response(response, false) {
        Ok(stream) => send_error_trailers(stream, status),
        Err(e) => log::debug!("failed to send error response: {e}"),
    }
}

fn otlp_proto_bytes(signal: SignalType, bytes: Bytes) -> OtlpProtoBytes {
    match signal {
        SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(bytes),
        SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(bytes),
        SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(bytes),
    }
}

fn encode_otlp_response(
    signal: SignalType,
    encoder: &mut GrpcMessageEncoder,
) -> Result<Bytes, Status> {
    match signal {
        SignalType::Logs => encoder.encode(&ExportLogsServiceResponse {
            partial_success: None,
        }),
        SignalType::Metrics => encoder.encode(&ExportMetricsServiceResponse {
            partial_success: None,
        }),
        SignalType::Traces => encoder.encode(&ExportTraceServiceResponse {
            partial_success: None,
        }),
    }
}

#[cfg(test)]
mod test_common;

#[cfg(test)]
mod otlp_tests;

#[cfg(test)]
mod otap_tests;
