// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental OTAP receiver that serves the Arrow gRPC endpoints directly on top of the `h2`
//! crate.  This variant keeps all request handling on the current thread so it can integrate with
//! the thread-per-core runtime without requiring `Send + Sync` futures.
//!
//! ToDo grpc-accept-encoding parsing: read client preference list, validate tokens, intersect with supported codecs, and propagate the chosen response codec through request handling.
//! ToDo Add snappy support. Wire in the matching decompress/encode routines with shared helpers for both request frames and response frames.
//! ToDo Error handling & metrics: surface clear statuses when the client requests unsupported codecs, log negotiation results, and add counters for negotiated/unsupported compression cases.
//! ToDo Tests: add unit/integration coverage for accept header parsing, per-codec request/response flows, and zstdarrow alias handling to prevent regressions.
//! ToDo Add support for Unix domain sockets as a transport option.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::compression::CompressionMethod;
use crate::otap_grpc::common;
use crate::otap_grpc::{GrpcServerSettings, Settings, per_connection_limit};
use crate::otap_receiver::OtapReceiverMetrics;
use ack::{AckRegistries, AckRegistry, route_local_ack_response, route_local_nack_response};
use futures::StreamExt;
use grpc::{
    AcceptedGrpcEncodings, GrpcMessageEncoder, GrpcStreamingBody, RequestTimeout,
    build_accept_encoding_header, grpc_encoding_token, negotiate_response_encoding,
    parse_grpc_accept_encoding, parse_grpc_encoding,
};
use stream::stream_batch_statuses;

mod ack;
mod grpc;
mod stream;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use bytes::Bytes;
use h2::server::{self, SendResponse};
use h2::{Ping, PingPong};
use http::{HeaderMap, HeaderValue, Request, Response, StatusCode as HttpStatusCode};
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
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otap_df_telemetry::metrics::MetricSet;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::ops::Add;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time::{Sleep, sleep};
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tonic::transport::server::TcpIncoming;

const OTEL_RECEIVER_URN: &str = "urn:otel:otap2:receiver";
const ARROW_LOGS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowLogsService/ArrowLogs";
const ARROW_METRICS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowMetricsService/ArrowMetrics";
const ARROW_TRACES_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowTracesService/ArrowTraces";
/// Configuration for the experimental H2 receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC server settings reused across receivers.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,
}

/// Experimental OTLP+OTAP receiver powered directly by the `h2` crate.
/// Note: Only OTAP Arrow payloads are supported in this version.
pub struct OtelReceiver {
    config: Config,
    metrics: MetricSet<OtapReceiverMetrics>,
}

#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
/// Registers the experimental H2 OTAP receiver factory.
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

        let router = Rc::new(GrpcRequestRouter {
            effect_handler: effect_handler.clone(),
            logs_ack_registry,
            metrics_ack_registry,
            traces_ack_registry,
            max_in_flight_per_connection: max_in_flight,
            request_encodings,
            request_accept_header,
            response_methods,
            request_timeout: config.timeout,
        });

        let cancel_token = CancellationToken::new();

        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // log::info!("OTAP H2 receiver starting on {}", config.listening_addr);

        tokio::select! {
            biased;
            ctrl_msg_result = async {
                loop {
                    match ctrl_msg_recv.recv().await {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            // log::info!("OTAP H2 receiver received shutdown signal");
                            cancel_token.cancel();
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { metrics_reporter }) => {
                            _ = metrics_reporter.report(&mut self.metrics);
                        }
                        Ok(NodeControlMsg::Ack(ack)) => {
                            let resp = route_local_ack_response(&ack_registries, ack);
                            common::handle_route_response(
                                resp,
                                &mut self.metrics,
                                |metrics| metrics.acks_sent.inc(),
                                |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                            );
                        }
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let resp = route_local_nack_response(&ack_registries, nack);
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
            } => {
                return ctrl_msg_result;
            }

            server_result = run_grpc_server(
                &mut incoming,
                Rc::clone(&config),
                Rc::clone(&router),
                cancel_token.clone(),
                admitter.clone(),
            ) => {
                if let Err(error) = server_result {
                    log::error!("OTAP H2 receiver server loop failed: {error}");
                    let source_detail = format_error_sources(&error);
                    return Err(Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Transport,
                        error: error.to_string(),
                        source_detail,
                    });
                }
            }
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
    arrow_router: Rc<GrpcRequestRouter>,
    cancel: CancellationToken,
    admitter: Admitter,
) -> Result<(), io::Error> {
    // Track all per-tcp-connection tasks.
    let mut tcp_conn_tasks: JoinSet<()> = JoinSet::new();
    let mut accepting = true;
    let h2_builder = build_h2_builder(&grpc_config);

    loop {
        tokio::select! {
            // 1) Cancellation: stop accepting and break to drain
            _ = cancel.cancelled() => break,

            // 2) Accept next TCP connection while accepting (and under cap, if any)
            res = incoming.next(), if accepting => {
                match res {
                    Some(Ok(tcp_conn)) => {
                        if let Err(e) = tcp_conn.set_nodelay(grpc_config.tcp_nodelay) {   // ToDo check it's already done in the TCPIncoming?
                            if log::log_enabled!(log::Level::Warn) {
                                log::warn!("Failed to set TCP_NODELAY: {e}");
                            }
                        }

                        // Admit a connection before spawning the task.
                        match admitter.try_admit_connection() {
                            AdmitDecision::Admitted(conn_guard) => {
                                let h2_builder = h2_builder.clone();
                                let router = Rc::clone(&arrow_router);
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
                    Some(Err(err)) => return Err(err),
                    None => {
                        accepting = false;
                    }
                }
            }

            // 3) Observe progress/completion of any connection task
            maybe_done = tcp_conn_tasks.join_next(), if !tcp_conn_tasks.is_empty() => {
                if let Some(Err(join_err)) = maybe_done {
                    if log::log_enabled!(log::Level::Debug) {
                        log::debug!("H2 connection task join error: {join_err}");
                    }
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
    let mut keepalive = Http2Keepalive::new(
        http2_conn.ping_pong(),
        keepalive_interval,
        keepalive_timeout,
    );

    let mut stream_tasks: JoinSet<()> = JoinSet::new();
    let mut accepting = true;

    loop {
        if let Some(ka) = keepalive.as_mut() {
            ka.update_idle_state(stream_tasks.is_empty());
        }
        let keepalive_armed = keepalive.as_ref().is_some_and(Http2Keepalive::is_armed);

        tokio::select! {
            // Accept next H2 stream while accepting
            result = http2_conn.accept(), if accepting => {
                match result {
                    Some(Ok((request, respond))) => {
                        // Try to open a *stream* on this connection.
                        match tcp_conn_guard.try_open_stream() {
                            AdmitDecision::Admitted(stream_guard) => {
                                let router = router.clone();
                                // Keep `stream_guard` alive for the request lifetime.
                                // Ignore the AbortHandler for now.
                                _ = stream_tasks.spawn_local(async move {
                                    if log::log_enabled!(log::Level::Trace) {
                                        // `request.uri()` is cheap, but we still avoid doing it if TRACE is off.
                                        log::trace!("New H2 stream: {}", request.uri().path());
                                    }
                                    if let Err(status) = router.handle_request(request, respond).await {
                                        if log::log_enabled!(log::Level::Debug) {
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
                    Some(Err(err)) => return Err(err),
                    None => accepting = false,
                }
            }

            // Join completed stream tasks
            maybe_done = stream_tasks.join_next(), if !stream_tasks.is_empty() => {
                if let Some(Err(join_err)) = maybe_done {
                    if log::log_enabled!(log::Level::Debug) {
                        log::debug!("stream task join error: {join_err}");
                    }
                }
            }

            keepalive_result = async {
                if let Some(ka) = keepalive.as_mut() {
                    ka.poll_tick().await
                } else {
                    unreachable!("keepalive polled without being armed");
                }
            }, if keepalive_armed => {
                match keepalive_result {
                    Ok(()) => {}
                    Err(err) => {
                        if log::log_enabled!(log::Level::Debug) {
                            log::debug!("H2 keepalive failed: {err}");
                        }
                        break;
                    }
                }
            }
        }

        // Exit when no more streams will arrive and all tasks are done
        if !accepting && stream_tasks.is_empty() {
            break;
        }
    }

    // Drain in-flight stream tasks
    while let Some(res) = stream_tasks.join_next().await {
        if let Err(join_err) = res {
            if log::log_enabled!(log::Level::Debug) {
                log::debug!("stream task join error: {join_err}");
            }
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

/// Routes each inbound gRPC request to the appropriate OTAP signal stream.
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
}

impl GrpcRequestRouter {
    async fn handle_request(
        self: Rc<Self>,
        request: Request<h2::RecvStream>,
        respond: SendResponse<Bytes>,
    ) -> Result<(), Status> {
        let path = request.uri().path();
        match path {
            ARROW_LOGS_SERVICE => {
                // log::info!("Handling ArrowLogs stream");
                self.serve_stream::<Logs>(
                    request,
                    respond,
                    OtapArrowRecords::Logs,
                    self.logs_ack_registry.clone(),
                )
                .await
            }
            ARROW_METRICS_SERVICE => {
                // log::info!("Handling ArrowMetrics stream");
                self.serve_stream::<Metrics>(
                    request,
                    respond,
                    OtapArrowRecords::Metrics,
                    self.metrics_ack_registry.clone(),
                )
                .await
            }
            ARROW_TRACES_SERVICE => {
                // log::info!("Handling ArrowTraces stream");
                self.serve_stream::<Traces>(
                    request,
                    respond,
                    OtapArrowRecords::Traces,
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

    async fn serve_stream<T>(
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

#[cfg(test)]
mod tests {
    use super::local;
    use super::{
        AcceptedGrpcEncodings, AckRegistry, GrpcMessageEncoder, GrpcStreamingBody,
        OTEL_RECEIVER_URN, OtelReceiver, RequestTimeout, build_accept_encoding_header,
        negotiate_response_encoding, parse_grpc_accept_encoding, parse_grpc_encoding,
        stream_batch_statuses,
    };
    use crate::compression::CompressionMethod;
    use crate::otap_grpc::ArrowRequestStream;
    use crate::otap_mock::create_otap_batch;
    use crate::otel_receiver::ack::AckToken;
    use crate::otel_receiver::grpc::{
        BodyStream, BodyStreamError, GrpcEncoding, MIN_COMPRESSED_CAPACITY,
    };
    use crate::pdata::OtapPdata;
    use async_stream::stream;
    use async_trait::async_trait;
    use bytes::{BufMut, Bytes, BytesMut};
    use flate2::Compression;
    use flate2::read::{GzDecoder, ZlibDecoder};
    use flate2::write::{GzEncoder, ZlibEncoder};
    use futures::StreamExt;
    use http::{HeaderMap, HeaderValue};
    use otap_df_channel::mpsc;
    use otap_df_config::PortName;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::Producer;
    use otap_df_pdata::otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode as ProtoStatusCode,
        arrow_logs_service_client::ArrowLogsServiceClient,
        arrow_metrics_service_client::ArrowMetricsServiceClient,
        arrow_traces_service_client::ArrowTracesServiceClient,
    };
    use otap_df_telemetry::reporter::MetricsReporter;
    use prost::Message as _;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::future::Future;
    use std::io::{Read, Write};
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;
    use tokio::task::yield_now;
    use tokio::time::{Duration, timeout};
    use tokio_stream::wrappers::UnboundedReceiverStream;
    use tonic::Status;
    use zstd::bulk::Compressor as ZstdCompressor;

    fn base_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        let _ = headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/grpc"),
        );
        headers
    }

    #[test]
    fn test_parse_grpc_encoding_variants() {
        let accepted = AcceptedGrpcEncodings::from_methods(&[
            CompressionMethod::Zstd,
            CompressionMethod::Gzip,
        ]);
        let mut headers = base_headers();
        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("zstd"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Zstd)
        ));

        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("gzip"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Gzip)
        ));

        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("zstdarrow1"));
        assert!(matches!(
            parse_grpc_encoding(&headers, &accepted),
            Ok(GrpcEncoding::Zstd)
        ));
    }

    #[test]
    fn test_parse_grpc_encoding_respects_config() {
        let accepted = AcceptedGrpcEncodings::from_methods(&[CompressionMethod::Deflate]);
        let mut headers = base_headers();
        let _ = headers.insert("grpc-encoding", HeaderValue::from_static("gzip"));
        assert!(parse_grpc_encoding(&headers, &accepted).is_err());
    }

    #[test]
    fn test_parse_grpc_accept_encoding() {
        let mut headers = HeaderMap::new();
        let _ = headers.insert(
            "grpc-accept-encoding",
            HeaderValue::from_static("gzip,zstd, identity "),
        );
        let parsed = parse_grpc_accept_encoding(&headers);
        assert!(parsed.identity);
        assert!(parsed.zstd);
        assert!(parsed.gzip);
        assert!(!parsed.deflate);
    }

    #[test]
    fn test_negotiate_response_encoding_prefers_config_order() {
        let mut client_headers = HeaderMap::new();
        let _ = client_headers.insert(
            "grpc-accept-encoding",
            HeaderValue::from_static("zstd,gzip"),
        );
        let client = parse_grpc_accept_encoding(&client_headers);
        let cfg = vec![CompressionMethod::Gzip, CompressionMethod::Zstd];
        assert!(matches!(
            negotiate_response_encoding(&cfg, &client),
            GrpcEncoding::Gzip
        ));

        let cfg = vec![CompressionMethod::Zstd];
        assert!(matches!(
            negotiate_response_encoding(&cfg, &client),
            GrpcEncoding::Zstd
        ));
    }

    #[test]
    fn test_build_accept_encoding_header_includes_identity() {
        let value =
            build_accept_encoding_header(&[CompressionMethod::Zstd, CompressionMethod::Gzip]);
        assert_eq!(value.to_str().unwrap(), "zstd,gzip,identity");
    }

    #[tokio::test]
    async fn request_timeout_triggers_after_inactivity() {
        let mut timeout = RequestTimeout::new(Some(Duration::from_millis(50)));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<&'static str, ()>>();
        let mut stream = UnboundedReceiverStream::new(rx);

        let _producer = tokio::spawn(async move {
            let _ = tx.send(Ok("first"));
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(Ok("second"));
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = tx.send(Ok("third"));
        });

        assert!(timeout.next_with(&mut stream).await.unwrap().is_some());
        tokio::time::sleep(Duration::from_millis(15)).await;
        assert!(timeout.next_with(&mut stream).await.unwrap().is_some());
        assert!(timeout.next_with(&mut stream).await.is_err());
    }

    #[tokio::test]
    async fn request_timeout_disabled_when_unset() {
        let mut timeout = RequestTimeout::new(None);
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<&'static str, ()>>();
        let mut stream = UnboundedReceiverStream::new(rx);

        let _producer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            let _ = tx.send(Ok("done"));
        });

        tokio::time::sleep(Duration::from_millis(35)).await;
        let next = timeout.next_with(&mut stream).await.unwrap();
        assert!(next.is_some());
    }

    #[test]
    fn test_grpc_message_encoder_identity_frame_layout() {
        let mut encoder = GrpcMessageEncoder::new(GrpcEncoding::Identity);
        let message = BatchStatus {
            batch_id: 42,
            status_code: 7,
            status_message: "ok".to_string(),
        };
        let encoded = encoder.encode(&message).expect("identity encode");
        assert_eq!(encoded[0], 0);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        assert_eq!(
            encoded[5..],
            message.encode_to_vec(),
            "payload matches prost encoding"
        );
    }

    #[test]
    fn test_grpc_message_encoder_gzip_round_trip() {
        let mut encoder = GrpcMessageEncoder::new(GrpcEncoding::Gzip);
        let message = BatchStatus {
            batch_id: 99,
            status_code: 14,
            status_message: "compressed".to_string(),
        };
        let encoded = encoder.encode(&message).expect("gzip encode");
        assert_eq!(encoded[0], 1);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        let mut decoder = GzDecoder::new(&encoded[5..]);
        let mut decompressed = Vec::new();
        let _ = decoder.read_to_end(&mut decompressed).expect("gunzip");
        assert_eq!(decompressed, message.encode_to_vec());
    }

    #[test]
    fn test_grpc_message_encoder_deflate_round_trip() {
        let mut encoder = GrpcMessageEncoder::new(GrpcEncoding::Deflate);
        let message = BatchStatus {
            batch_id: 7,
            status_code: 3,
            status_message: "deflated".to_string(),
        };
        let encoded = encoder.encode(&message).expect("deflate encode");
        assert_eq!(encoded[0], 1);
        let len = u32::from_be_bytes(encoded[1..5].try_into().unwrap()) as usize;
        assert_eq!(len, encoded.len() - 5);
        let mut decoder = ZlibDecoder::new(&encoded[5..]);
        let mut decompressed = Vec::new();
        let _ = decoder.read_to_end(&mut decompressed).expect("inflate");
        assert_eq!(decompressed, message.encode_to_vec());
    }

    struct FakeArrowStream {
        batches: VecDeque<BatchArrowRecords>,
    }

    impl FakeArrowStream {
        fn new(batches: VecDeque<BatchArrowRecords>) -> Self {
            Self { batches }
        }
    }

    #[async_trait]
    impl ArrowRequestStream for FakeArrowStream {
        async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
            Ok(self.batches.pop_front())
        }
    }

    fn build_test_effect_handler(
        channel_capacity: usize,
    ) -> (local::EffectHandler<OtapPdata>, mpsc::Receiver<OtapPdata>) {
        let (tx, rx) = mpsc::Channel::new(channel_capacity);
        let mut senders = HashMap::new();
        let default_port: PortName = PortName::from("default");
        let _ = senders.insert(default_port.clone(), LocalSender::MpscSender(tx));
        let (ctrl_tx, _ctrl_rx) = pipeline_ctrl_msg_channel(4);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let effect_handler = local::EffectHandler::new(
            test_node("otel_receiver_status_test"),
            senders,
            Some(default_port),
            ctrl_tx,
            metrics_reporter,
        );
        (effect_handler, rx)
    }

    fn arrow_batches(
        payload_type: ArrowPayloadType,
        batch_count: usize,
    ) -> VecDeque<BatchArrowRecords> {
        let mut queue = VecDeque::with_capacity(batch_count);
        let mut producer = Producer::new();
        for batch_index in 0..batch_count {
            let mut batch = create_otap_batch(batch_index as i64, payload_type);
            let bar = producer
                .produce_bar(&mut batch)
                .expect("failed to encode arrow batch");
            queue.push_back(bar);
        }
        queue
    }

    async fn drive_ack_pipeline(
        pdata_rx: mpsc::Receiver<OtapPdata>,
        ack_registry: AckRegistry,
        total_batches: usize,
    ) -> (usize, usize) {
        let mut success = 0;
        let mut failure = 0;
        for idx in 0..total_batches {
            let pdata = pdata_rx
                .recv()
                .await
                .expect("pdata channel closed unexpectedly");
            let calldata = pdata
                .current_calldata()
                .expect("missing calldata for wait_for_result");
            let token = AckToken::from_calldata(&calldata).expect("invalid ack token");

            if idx % 11 == 0 {
                tokio::time::sleep(Duration::from_micros(((idx % 5) + 1) as u64 * 20)).await;
                let _ = ack_registry.complete(token, Err(format!("failure #{idx}")));
                failure += 1;
            } else {
                if idx % 7 == 0 {
                    tokio::time::sleep(Duration::from_micros(((idx % 3) + 1) as u64 * 10)).await;
                } else if idx % 3 == 0 {
                    yield_now().await;
                }
                let _ = ack_registry.complete(token, Ok(()));
                success += 1;
            }
        }
        (success, failure)
    }

    async fn run_status_stream_load_test<T>(
        payload_type: ArrowPayloadType,
        otap_batch: fn(T) -> OtapArrowRecords,
    ) where
        T: OtapBatchStore + 'static,
    {
        const TOTAL_BATCHES: usize = 1024;
        const MAX_CONCURRENT_REQUESTS: usize = 256;
        const MAX_IN_FLIGHT: usize = 64;

        let stream = FakeArrowStream::new(arrow_batches(payload_type, TOTAL_BATCHES));
        let (effect_handler, pdata_rx) = build_test_effect_handler(TOTAL_BATCHES);
        let ack_registry = AckRegistry::new(MAX_CONCURRENT_REQUESTS);

        let mut status_stream = stream_batch_statuses::<_, T, _>(
            stream,
            effect_handler,
            Some(ack_registry.clone()),
            otap_batch,
            MAX_IN_FLIGHT,
        );

        let ack_task = drive_ack_pipeline(pdata_rx, ack_registry.clone(), TOTAL_BATCHES);

        let status_task = async {
            let mut successes = 0;
            let mut failures = 0;
            let mut ids = HashSet::with_capacity(TOTAL_BATCHES);
            while let Some(next) = status_stream.next().await {
                let status = next.expect("receiver should not emit tonic errors");
                assert!(
                    ids.insert(status.batch_id),
                    "duplicate status for batch {}",
                    status.batch_id
                );
                match status.status_code {
                    code if code == ProtoStatusCode::Ok as i32 => {
                        assert_eq!(status.status_message, "Successfully received");
                        successes += 1;
                    }
                    code if code == ProtoStatusCode::Unavailable as i32 => {
                        assert!(
                            status
                                .status_message
                                .starts_with("Pipeline processing failed:"),
                            "unexpected failure message {}",
                            status.status_message
                        );
                        failures += 1;
                    }
                    other => panic!("unexpected status code {other}"),
                }
            }
            assert_eq!(ids.len(), TOTAL_BATCHES);
            (successes, failures)
        };

        let ((expected_successes, expected_failures), (actual_successes, actual_failures)) =
            tokio::join!(ack_task, status_task);

        assert_eq!(actual_successes, expected_successes);
        assert_eq!(actual_failures, expected_failures);
        assert_eq!(actual_successes + actual_failures, TOTAL_BATCHES);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_ack_nack_load() {
        run_status_stream_load_test::<Metrics>(
            ArrowPayloadType::MultivariateMetrics,
            OtapArrowRecords::Metrics,
        )
        .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_logs_load() {
        run_status_stream_load_test::<Logs>(ArrowPayloadType::Logs, OtapArrowRecords::Logs).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn stream_batch_statuses_handles_large_traces_load() {
        run_status_stream_load_test::<Traces>(ArrowPayloadType::Spans, OtapArrowRecords::Traces)
            .await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn grpc_streaming_body_randomized_frames() {
        async fn run_case(encoding: GrpcEncoding, seed: u64) {
            let mut rng = StdRng::seed_from_u64(seed);
            for iteration in 0..32 {
                let frame_count = rng.random_range(1..=8);
                let mut expected_ids = Vec::with_capacity(frame_count);
                let mut chunk_queue: VecDeque<Result<Bytes, &'static str>> = VecDeque::new();
                let mut expected_release = 0usize;

                for frame_idx in 0..frame_count {
                    let batch_id = (iteration * 100 + frame_idx) as i64;
                    expected_ids.push(batch_id);
                    let batch = BatchArrowRecords {
                        batch_id,
                        ..Default::default()
                    };

                    let frame = build_body_frame(&batch, encoding);
                    for chunk in split_frame_into_chunks(frame, &mut rng) {
                        expected_release += chunk.len();
                        chunk_queue.push_back(Ok(chunk));
                    }
                }

                let (stream, state_handle) = MockRecvStream::new(chunk_queue);
                let mut body = GrpcStreamingBody::with_stream(stream, encoding);
                let mut observed_ids = Vec::new();
                while let Some(batch) = body
                    .next_message()
                    .await
                    .expect("fuzzer should decode batches")
                {
                    observed_ids.push(batch.batch_id);
                }
                drop(body);

                assert_eq!(
                    observed_ids, expected_ids,
                    "encoding {:?} iteration {}",
                    encoding, iteration
                );
                let released = state_handle
                    .lock()
                    .expect("state lock poisoned")
                    .released_bytes;
                assert_eq!(
                    released, expected_release,
                    "flow control release mismatch for {:?}",
                    encoding
                );
            }
        }

        run_case(GrpcEncoding::Identity, 0x1111).await;
        run_case(GrpcEncoding::Gzip, 0x2222).await;
        run_case(GrpcEncoding::Deflate, 0x3333).await;
        run_case(GrpcEncoding::Zstd, 0x4444).await;
    }

    fn build_body_frame(batch: &BatchArrowRecords, encoding: GrpcEncoding) -> Bytes {
        let payload = batch.encode_to_vec();
        let (compressed, encoded_payload) = match encoding {
            GrpcEncoding::Identity => (false, payload),
            GrpcEncoding::Gzip => (true, compress_payload_gzip(&payload)),
            GrpcEncoding::Deflate => (true, compress_payload_deflate(&payload)),
            GrpcEncoding::Zstd => (true, compress_payload_zstd(&payload)),
        };
        let mut frame = BytesMut::with_capacity(5 + encoded_payload.len());
        frame.put_u8(u8::from(compressed));
        frame.put_u32(encoded_payload.len() as u32);
        frame.extend_from_slice(&encoded_payload);
        frame.freeze()
    }

    fn compress_payload_gzip(payload: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload).expect("gzip write");
        encoder.finish().expect("gzip finish")
    }

    fn compress_payload_deflate(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(payload).expect("deflate write");
        encoder.finish().expect("deflate finish")
    }

    fn compress_payload_zstd(payload: &[u8]) -> Vec<u8> {
        let mut encoder = ZstdCompressor::new(0).expect("zstd encoder");
        let mut buffer = vec![0u8; payload.len().max(MIN_COMPRESSED_CAPACITY)];
        let written = encoder
            .compress_to_buffer(payload, buffer.as_mut_slice())
            .expect("zstd compress");
        buffer.truncate(written);
        buffer
    }

    fn split_frame_into_chunks(frame: Bytes, rng: &mut StdRng) -> Vec<Bytes> {
        let mut offset = 0;
        let mut chunks = Vec::new();
        while offset < frame.len() {
            let remaining = frame.len() - offset;
            let max_chunk = remaining.clamp(1, 64);
            let step = rng.random_range(1..=max_chunk);
            chunks.push(frame.slice(offset..offset + step));
            offset += step;
        }
        chunks
    }

    struct MockStreamState {
        released_bytes: usize,
    }

    struct MockRecvStream {
        chunks: VecDeque<Result<Bytes, &'static str>>,
        state: Arc<Mutex<MockStreamState>>,
    }

    impl MockRecvStream {
        fn new(
            chunks: VecDeque<Result<Bytes, &'static str>>,
        ) -> (Self, Arc<Mutex<MockStreamState>>) {
            let state = Arc::new(Mutex::new(MockStreamState { released_bytes: 0 }));
            (
                Self {
                    chunks,
                    state: state.clone(),
                },
                state,
            )
        }
    }

    #[async_trait]
    impl BodyStream for MockRecvStream {
        async fn next_chunk(&mut self) -> Option<Result<Bytes, BodyStreamError>> {
            yield_now().await;
            self.chunks
                .pop_front()
                .map(|res| res.map_err(|err| err.to_string()))
        }

        fn release_capacity(&mut self, released: usize) -> Result<(), BodyStreamError> {
            if let Ok(mut state) = self.state.lock() {
                state.released_bytes += released;
            }
            Ok(())
        }
    }

    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut arrow_metrics_client =
                    ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect metrics client");
                #[allow(tail_expr_drop_order)]
                let metrics_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut metrics_records =
                            create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                        let bar = producer.produce_bar(&mut metrics_records).unwrap();
                        yield bar
                    }
                };
                let metrics_response = arrow_metrics_client
                    .arrow_metrics(metrics_stream)
                    .await
                    .expect("metrics request failed");
                validate_batch_responses(
                    metrics_response.into_inner(),
                    0,
                    "Successfully received",
                    "metrics",
                )
                .await;

                let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect logs client");
                #[allow(tail_expr_drop_order)]
                let logs_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                        let bar = producer.produce_bar(&mut logs_records).unwrap();
                        yield bar;
                    }
                };
                let logs_response = arrow_logs_client
                    .arrow_logs(logs_stream)
                    .await
                    .expect("logs request failed");
                validate_batch_responses(
                    logs_response.into_inner(),
                    0,
                    "Successfully received",
                    "logs",
                )
                .await;

                let mut arrow_traces_client =
                    ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect traces client");
                #[allow(tail_expr_drop_order)]
                let traces_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut traces_records =
                            create_otap_batch(batch_id, ArrowPayloadType::Spans);
                        let bar = producer.produce_bar(&mut traces_records).unwrap();
                        yield bar;
                    }
                };
                let traces_response = arrow_traces_client
                    .arrow_traces(traces_stream)
                    .await
                    .expect("traces request failed");
                validate_batch_responses(
                    traces_response.into_inner(),
                    0,
                    "Successfully received",
                    "traces",
                )
                .await;

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("shutdown send failed");
            })
        }
    }

    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                for batch_id in 0..3 {
                    let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("metrics timeout")
                        .expect("missing metrics");
                    let metrics_records: OtapArrowRecords = metrics_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("metrics conversion");
                    let _expected_metrics =
                        create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                    assert!(matches!(metrics_records, _expected_metrics));
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("metrics ack send failed");
                    }
                }

                for batch_id in 0..3 {
                    let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("logs timeout")
                        .expect("missing logs");
                    let logs_records: OtapArrowRecords = logs_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("logs conversion");
                    let _expected_logs = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    assert!(matches!(logs_records, _expected_logs));
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("logs ack send failed");
                    }
                }

                for batch_id in 0..3 {
                    let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("traces timeout")
                        .expect("missing traces");
                    let traces_records: OtapArrowRecords = traces_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("traces conversion");
                    let _expected_traces = create_otap_batch(batch_id, ArrowPayloadType::Spans);
                    assert!(matches!(traces_records, _expected_traces));
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(traces_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("traces ack send failed");
                    }
                }
            })
        }
    }

    fn nack_scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut arrow_metrics_client =
                    ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect metrics client");
                #[allow(tail_expr_drop_order)]
                let metrics_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut metrics_records =
                            create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                        let bar = producer.produce_bar(&mut metrics_records).unwrap();
                        yield bar
                    }
                };
                let metrics_response = arrow_metrics_client
                    .arrow_metrics(metrics_stream)
                    .await
                    .expect("metrics request failed");
                validate_batch_responses(
                    metrics_response.into_inner(),
                    14,
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for metrics"
                    ),
                    "metrics",
                )
                .await;

                let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect logs client");
                #[allow(tail_expr_drop_order)]
                let logs_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                        let bar = producer.produce_bar(&mut logs_records).unwrap();
                        yield bar;
                    }
                };
                let logs_response = arrow_logs_client
                    .arrow_logs(logs_stream)
                    .await
                    .expect("logs request failed");
                validate_batch_responses(
                    logs_response.into_inner(),
                    14,
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for logs"
                    ),
                    "logs",
                )
                .await;

                let mut arrow_traces_client =
                    ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect traces client");
                #[allow(tail_expr_drop_order)]
                let traces_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut traces_records =
                            create_otap_batch(batch_id, ArrowPayloadType::Spans);
                        let bar = producer.produce_bar(&mut traces_records).unwrap();
                        yield bar;
                    }
                };
                let traces_response = arrow_traces_client
                    .arrow_traces(traces_stream)
                    .await
                    .expect("traces request failed");
                validate_batch_responses(
                    traces_response.into_inner(),
                    14,
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for traces"
                    ),
                    "traces",
                )
                .await;

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("shutdown send failed");
            })
        }
    }

    fn nack_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                for _batch_id in 0..3 {
                    let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("metrics timeout")
                        .expect("missing metrics");
                    let nack = NackMsg::new("Test NACK reason for metrics", metrics_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("metrics nack send failed");
                    }
                }

                for _batch_id in 0..3 {
                    let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("logs timeout")
                        .expect("missing logs");
                    let nack = NackMsg::new("Test NACK reason for logs", logs_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("logs nack send failed");
                    }
                }

                for _batch_id in 0..3 {
                    let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("traces timeout")
                        .expect("missing traces");
                    let nack = NackMsg::new("Test NACK reason for traces", traces_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("traces nack send failed");
                    }
                }
            })
        }
    }

    async fn validate_batch_responses<S>(
        mut inbound_stream: S,
        expected_status_code: i32,
        expected_status_message: &str,
        signal_name: &str,
    ) where
        S: futures::Stream<Item = Result<BatchStatus, Status>> + Unpin,
    {
        use futures::StreamExt;
        let mut received_batch_ids = HashSet::new();
        while let Some(result) = inbound_stream.next().await {
            assert!(
                result.is_ok(),
                "Expected successful response for {}",
                signal_name
            );
            let batch_status = result.unwrap();
            let batch_id = batch_status.batch_id;
            assert!(
                received_batch_ids.insert(batch_id),
                "Duplicate response for batch {} ({})",
                batch_id,
                signal_name
            );
            assert_eq!(
                batch_status.status_code, expected_status_code,
                "Unexpected status code for {} batch {}",
                signal_name, batch_id
            );
            assert_eq!(
                batch_status.status_message, expected_status_message,
                "Unexpected status message for {} batch {}",
                signal_name, batch_id
            );
        }
        assert_eq!(
            received_batch_ids,
            (0..3).collect::<HashSet<_>>(),
            "Missing responses for {}",
            signal_name
        );
    }

    fn pick_free_port() -> u16 {
        portpicker::pick_unused_port().expect("No free ports")
    }

    #[test]
    fn test_otel_receiver() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = pick_free_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::MetricsRegistryHandle;
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config = json!({ "listening_addr": addr.to_string() });
        let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
        receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);

        let receiver = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation(validation_procedure());
    }

    #[test]
    fn test_otel_receiver_ack() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = pick_free_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::MetricsRegistryHandle;
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config = json!({
            "listening_addr": addr.to_string(),
            "wait_for_result": true
        });
        let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
        receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
        let receiver = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation_concurrent(validation_procedure());
    }

    #[test]
    fn test_otel_receiver_nack() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = pick_free_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTEL_RECEIVER_URN));
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::MetricsRegistryHandle;
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config = json!({
            "listening_addr": addr.to_string(),
            "wait_for_result": true
        });
        let mut receiver = OtelReceiver::from_config(pipeline_ctx, &config).unwrap();
        receiver.tune_max_concurrent_requests(test_runtime.config().output_pdata_channel.capacity);
        let receiver = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(nack_scenario(grpc_endpoint))
            .run_validation_concurrent(nack_validation_procedure());
    }

    #[test]
    fn test_otel_receiver_config_parsing() {
        use crate::compression::CompressionMethod;
        use serde_json::json;

        let metrics_registry_handle = otap_df_telemetry::registry::MetricsRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config_with_max_concurrent_requests = json!({
            "listening_addr": "127.0.0.1:4417",
            "max_concurrent_requests": 5000
        });
        let receiver =
            OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
                .unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4417"
        );
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 5000);
        assert!(!receiver.config.grpc.wait_for_result);
        assert!(receiver.config.grpc.request_compression.is_none());
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert!(receiver.config.grpc.timeout.is_none());

        let config_minimal = json!({ "listening_addr": "127.0.0.1:4418" });
        let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_minimal).unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4418"
        );
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 0);
        assert!(!receiver.config.grpc.wait_for_result);
        assert!(receiver.config.grpc.request_compression.is_none());
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert!(receiver.config.grpc.timeout.is_none());

        let config_full_gzip = json!({
            "listening_addr": "127.0.0.1:4419",
            "compression_method": "gzip",
            "max_concurrent_requests": 2500,
            "wait_for_result": true,
            "timeout": "30s"
        });
        let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_full_gzip).unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4419"
        );
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 2500);
        assert!(receiver.config.grpc.wait_for_result);
        assert_eq!(
            receiver.config.grpc.request_compression,
            Some(vec![CompressionMethod::Gzip])
        );
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert_eq!(receiver.config.grpc.timeout, Some(Duration::from_secs(30)));

        let config_with_zstd = json!({
            "listening_addr": "127.0.0.1:4420",
            "compression_method": "zstd",
            "wait_for_result": false
        });
        let receiver = OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_zstd).unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4420"
        );
        assert!(!receiver.config.grpc.wait_for_result);
        assert_eq!(
            receiver.config.grpc.request_compression,
            Some(vec![CompressionMethod::Zstd])
        );
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert!(receiver.config.grpc.timeout.is_none());

        let config_with_deflate = json!({
            "listening_addr": "127.0.0.1:4421",
            "compression_method": "deflate"
        });
        let receiver =
            OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_deflate).unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4421"
        );
        assert_eq!(
            receiver.config.grpc.request_compression,
            Some(vec![CompressionMethod::Deflate])
        );
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert!(receiver.config.grpc.timeout.is_none());

        let config_with_response_only = json!({
            "listening_addr": "127.0.0.1:4422",
            "response_compression_method": "gzip"
        });
        let receiver =
            OtelReceiver::from_config(pipeline_ctx.clone(), &config_with_response_only).unwrap();
        assert_eq!(
            receiver.config.grpc.listening_addr.to_string(),
            "127.0.0.1:4422"
        );
        assert!(receiver.config.grpc.request_compression.is_none());
        assert_eq!(
            receiver.config.grpc.response_compression,
            Some(vec![CompressionMethod::Gzip])
        );
        assert_eq!(
            receiver.config.grpc.preferred_response_compression(),
            Some(CompressionMethod::Gzip)
        );
        assert!(receiver.config.grpc.timeout.is_none());
    }
}
