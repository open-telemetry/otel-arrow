// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental OTAP receiver that serves the Arrow gRPC endpoints directly on top of the `h2`
//! crate.  This variant keeps all request handling on the current thread so it can integrate with
//! the thread-per-core runtime without requiring `Send + Sync` futures.
//!
//! ToDo grpc-accept-encoding parsing: read client preference list, validate tokens, intersect with supported codecs, and propagate the chosen response codec through request handling.
//  ToDo Add snappy support. Wire in the matching decompress/encode routines with shared helpers for both request frames and response frames.
//  ToDo Error handling & metrics: surface clear statuses when the client requests unsupported codecs, log negotiation results, and add counters for negotiated/unsupported compression cases.
//  ToDo Tests: add unit/integration coverage for accept header parsing, per-codec request/response flows, and zstdarrow alias handling to prevent regressions.

use crate::compression::CompressionMethod;
use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::common;
use crate::otap_grpc::otlp::server::RouteResponse;
use crate::otap_grpc::{ArrowRequestStream, GrpcServerSettings, Settings, per_connection_limit};
use crate::otap_receiver::OtapReceiverMetrics;
use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;
use futures::future::{LocalBoxFuture, poll_fn};
use futures::stream::FuturesUnordered;
use futures::{Stream, StreamExt};
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
use otap_df_engine::control::{AckMsg, CallData, Context8u8, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces, from_record_messages};
use otap_df_pdata::proto::opentelemetry::arrow::v1::{
    BatchArrowRecords, BatchStatus, StatusCode as ProtoStatusCode,
};
use otap_df_pdata::{Consumer, OtapArrowRecords};
use otap_df_telemetry::metrics::MetricSet;
use prost::Message;
use serde::Deserialize;
use smallvec::smallvec;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::future::Future;
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::mem;
use std::ops::Add;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll, Waker};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::time::{sleep, Sleep, Instant as TokioInstant};
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tonic::transport::server::TcpIncoming;
use zstd::bulk::{Compressor as ZstdCompressor, Decompressor as ZstdDecompressor};

const OTEL_RECEIVER_URN: &str = "urn:otel:otap2:receiver";
const ARROW_LOGS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowLogsService/ArrowLogs";
const ARROW_METRICS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowMetricsService/ArrowMetrics";
const ARROW_TRACES_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowTracesService/ArrowTraces";
const MIN_DECOMPRESSED_CAPACITY: usize = 8 * 1024;
const MIN_COMPRESSED_CAPACITY: usize = 1024;

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
        let request_encodings =
            AcceptedGrpcEncodings::from_methods(&request_encoding_methods);
        let request_accept_header =
            build_accept_encoding_header(&request_encoding_methods);
        let response_methods = config.response_compression_methods();

        let router = Rc::new(ArrowRouter {
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
    arrow_router: Rc<ArrowRouter>,
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
    router: Rc<ArrowRouter>,
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
        let keepalive_armed = keepalive
            .as_ref()
            .is_some_and(Http2Keepalive::is_armed);

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

fn reset_sleep(timer: &mut Option<Pin<Box<Sleep>>>, duration: Duration) {
    if let Some(timer) = timer.as_mut() {
        timer.as_mut().reset(TokioInstant::now() + duration);
    }
}

struct ArrowRouter {
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

impl ArrowRouter {
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
        let response_encoding =
            negotiate_response_encoding(&self.response_methods, &client_accept);
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

        let timeout_duration = self.request_timeout;
        let mut timeout = timeout_duration.map(|dur| Box::pin(sleep(dur)));

        loop {
            let next_item = if let Some(timeout) = timeout.as_mut() {
                tokio::select! {
                    _ = timeout.as_mut() => {
                        if let Some(duration) = timeout_duration {
                            log::debug!("Request timed out after {:?}", duration);
                        }
                        send_error_trailers(
                            send_stream,
                            Status::deadline_exceeded("request timed out"),
                        );
                        return Ok(());
                    }
                    next = status_stream.next() => next
                }
            } else {
                status_stream.next().await
            };

            match next_item {
                Some(Ok(status)) => {
                    if let Some(duration) = timeout_duration {
                        reset_sleep(&mut timeout, duration);
                    }
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

fn parse_grpc_encoding(
    headers: &HeaderMap,
    accepted: &AcceptedGrpcEncodings,
) -> Result<GrpcEncoding, Status> {
    match headers.get(http::header::CONTENT_TYPE) {
        Some(value) if value.as_bytes().starts_with(b"application/grpc") => {}
        other => {
            log::error!("Rejecting stream due to invalid content-type: {other:?}");
            return Err(Status::invalid_argument(
                "missing application/grpc content-type",
            ));
        }
    }
    match headers.get("grpc-encoding") {
        None => Ok(GrpcEncoding::Identity),
        Some(value) => {
            let raw = value.to_str().map_err(|_| {
                log::error!("Non-UTF8 grpc-encoding header");
                Status::invalid_argument("invalid grpc-encoding header")
            })?;
            let trimmed = raw.trim();
            let ascii = trimmed.as_bytes();
            const PREFIX: &[u8] = b"zstdarrow";

            let encoding = if ascii.is_empty() || eq_ascii_case_insensitive(ascii, b"identity") {
                GrpcEncoding::Identity
            } else if eq_ascii_case_insensitive(ascii, b"zstd") {
                GrpcEncoding::Zstd
            } else if eq_ascii_case_insensitive(ascii, b"gzip") {
                GrpcEncoding::Gzip
            } else if eq_ascii_case_insensitive(ascii, b"deflate") {
                GrpcEncoding::Deflate
            } else if ascii.len() >= PREFIX.len()
                && starts_with_ascii_case_insensitive(ascii, PREFIX)
            {
                let tail = &ascii[PREFIX.len()..];
                if tail.len() == 1 && tail[0].is_ascii_digit() {
                    GrpcEncoding::Zstd
                } else {
                    log::error!("Unsupported grpc-encoding {}", trimmed);
                    return Err(Status::unimplemented("grpc compression not supported"));
                }
            } else {
                log::error!("Unsupported grpc-encoding {}", trimmed);
                return Err(Status::unimplemented("grpc compression not supported"));
            };

            if accepted.allows(encoding) {
                Ok(encoding)
            } else {
                log::error!(
                    "grpc-encoding {} not enabled in server configuration",
                    trimmed
                );
                Err(Status::unimplemented("grpc compression not supported"))
            }
        }
    }
}

fn eq_ascii_case_insensitive(value: &[u8], expected: &[u8]) -> bool {
    value.len() == expected.len()
        && value
            .iter()
            .zip(expected)
            .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
}

fn starts_with_ascii_case_insensitive(value: &[u8], prefix: &[u8]) -> bool {
    value.len() >= prefix.len()
        && value
            .iter()
            .zip(prefix)
            .all(|(lhs, rhs)| ascii_byte_eq_ignore_case(*lhs, *rhs))
}

fn ascii_byte_eq_ignore_case(lhs: u8, rhs: u8) -> bool {
    lhs == rhs || lhs.eq_ignore_ascii_case(&rhs)
}

fn parse_grpc_accept_encoding(headers: &HeaderMap) -> ClientAcceptEncodings {
    let Some(value) = headers.get("grpc-accept-encoding") else {
        return ClientAcceptEncodings::identity_only();
    };
    let raw = match value.to_str() {
        Ok(raw) => raw,
        Err(_) => return ClientAcceptEncodings::identity_only(),
    };

    let mut encodings = ClientAcceptEncodings {
        identity: false,
        zstd: false,
        gzip: false,
        deflate: false,
    };
    let mut recognized = false;

    for token in raw.split(',') {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ascii = trimmed.as_bytes();
        if eq_ascii_case_insensitive(ascii, b"identity") {
            encodings.identity = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"zstd") {
            encodings.zstd = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"gzip") {
            encodings.gzip = true;
            recognized = true;
        } else if eq_ascii_case_insensitive(ascii, b"deflate") {
            encodings.deflate = true;
            recognized = true;
        }
    }

    if recognized {
        encodings
    } else {
        ClientAcceptEncodings::identity_only()
    }
}

fn negotiate_response_encoding(
    configured: &[CompressionMethod],
    client: &ClientAcceptEncodings,
) -> GrpcEncoding {
    for method in configured {
        if client.supports(*method) {
            return match method {
                CompressionMethod::Zstd => GrpcEncoding::Zstd,
                CompressionMethod::Gzip => GrpcEncoding::Gzip,
                CompressionMethod::Deflate => GrpcEncoding::Deflate,
            };
        }
    }
    GrpcEncoding::Identity
}

#[derive(Clone, Copy)]
enum GrpcEncoding {
    Identity,
    Zstd,
    Gzip,
    Deflate,
    // ToDo Add support for Snappy to follow Go implementation
}

#[derive(Clone, Copy)]
struct AcceptedGrpcEncodings {
    zstd: bool,
    gzip: bool,
    deflate: bool,
}

impl AcceptedGrpcEncodings {
    fn from_methods(methods: &[CompressionMethod]) -> Self {
        let mut encodings = Self {
            zstd: false,
            gzip: false,
            deflate: false,
        };

        for method in methods {
            match method {
                CompressionMethod::Zstd => encodings.zstd = true,
                CompressionMethod::Gzip => encodings.gzip = true,
                CompressionMethod::Deflate => encodings.deflate = true,
            }
        }

        encodings
    }

    fn allows(self, encoding: GrpcEncoding) -> bool {
        match encoding {
            GrpcEncoding::Identity => true,
            GrpcEncoding::Zstd => self.zstd,
            GrpcEncoding::Gzip => self.gzip,
            GrpcEncoding::Deflate => self.deflate,
        }
    }
}

#[derive(Clone, Copy)]
struct ClientAcceptEncodings {
    identity: bool,
    zstd: bool,
    gzip: bool,
    deflate: bool,
}

impl ClientAcceptEncodings {
    fn identity_only() -> Self {
        Self {
            identity: true,
            zstd: false,
            gzip: false,
            deflate: false,
        }
    }

    fn supports(self, method: CompressionMethod) -> bool {
        match method {
            CompressionMethod::Zstd => self.zstd,
            CompressionMethod::Gzip => self.gzip,
            CompressionMethod::Deflate => self.deflate,
        }
    }
}

fn build_accept_encoding_header(methods: &[CompressionMethod]) -> HeaderValue {
    let mut tokens = Vec::with_capacity(methods.len() + 1);
    for method in methods {
        tokens.push(compression_method_token(*method));
    }
    // `identity` is always supported but least preferred.
    tokens.push("identity");
    let joined = tokens.join(",");
    HeaderValue::from_str(&joined).unwrap_or_else(|_| HeaderValue::from_static("identity"))
}

fn compression_method_token(method: CompressionMethod) -> &'static str {
    match method {
        CompressionMethod::Zstd => "zstd",
        CompressionMethod::Gzip => "gzip",
        CompressionMethod::Deflate => "deflate",
    }
}

fn grpc_encoding_token(encoding: GrpcEncoding) -> Option<&'static str> {
    match encoding {
        GrpcEncoding::Identity => None,
        GrpcEncoding::Zstd => Some("zstd"),
        GrpcEncoding::Gzip => Some("gzip"),
        GrpcEncoding::Deflate => Some("deflate"),
    }
}

struct GrpcStreamingBody {
    recv: h2::RecvStream,
    buffer: ChunkBuffer,
    current_frame: Option<FrameHeader>,
    finished: bool,
    encoding: GrpcEncoding,
    zstd: Option<ZstdDecompressor<'static>>,
    decompressed_buf: Vec<u8>,
}

#[derive(Clone, Copy)]
struct FrameHeader {
    length: usize,
    compressed: bool,
}

struct ChunkBuffer {
    chunks: VecDeque<Bytes>,
    len: usize,
}

impl ChunkBuffer {
    fn new() -> Self {
        Self {
            chunks: VecDeque::new(),
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn push(&mut self, chunk: Bytes) {
        if chunk.is_empty() {
            return;
        }
        self.len += chunk.len();
        self.chunks.push_back(chunk);
    }

    fn split_frame(&mut self, size: usize) -> Option<FrameBuf> {
        if size > self.len {
            return None;
        }
        if size == 0 {
            return Some(FrameBuf::new(VecDeque::new(), 0));
        }

        let mut needed = size;
        let mut parts = VecDeque::new();
        while needed > 0 {
            let mut chunk = self.chunks.pop_front()?;
            if chunk.len() > needed {
                let part = chunk.split_to(needed);
                self.len -= needed;
                parts.push_back(part);
                self.chunks.push_front(chunk);
                needed = 0;
            } else {
                needed -= chunk.len();
                self.len -= chunk.len();
                parts.push_back(chunk);
            }
        }
        Some(FrameBuf::new(parts, size))
    }
}

struct FrameBuf {
    chunks: VecDeque<Bytes>,
    remaining: usize,
}

impl FrameBuf {
    fn new(chunks: VecDeque<Bytes>, remaining: usize) -> Self {
        Self { chunks, remaining }
    }

    fn into_bytes(mut self) -> Bytes {
        match self.chunks.len() {
            0 => Bytes::new(),
            1 => self
                .chunks
                .pop_front()
                .expect("frame buffer length mismatch"),
            _ => {
                let mut buf = BytesMut::with_capacity(self.remaining);
                while let Some(chunk) = self.chunks.pop_front() {
                    buf.extend_from_slice(&chunk);
                }
                buf.freeze()
            }
        }
    }
}

impl Buf for FrameBuf {
    fn remaining(&self) -> usize {
        self.remaining
    }

    fn chunk(&self) -> &[u8] {
        self.chunks
            .front()
            .map(|bytes| bytes.as_ref())
            .unwrap_or(&[])
    }

    fn advance(&mut self, mut cnt: usize) {
        assert!(cnt <= self.remaining);
        self.remaining -= cnt;
        while cnt > 0 {
            let Some(front_len) = self.chunks.front().map(|b| b.len()) else {
                break;
            };
            if cnt < front_len {
                if let Some(front) = self.chunks.front_mut() {
                    front.advance(cnt);
                }
                break;
            } else {
                cnt -= front_len;
                let _ = self.chunks.pop_front();
            }
        }
    }
}

impl GrpcStreamingBody {
    fn new(recv: h2::RecvStream, encoding: GrpcEncoding) -> Self {
        Self {
            recv,
            buffer: ChunkBuffer::new(),
            current_frame: None,
            finished: false,
            encoding,
            zstd: None,
            decompressed_buf: Vec::new(),
        }
    }

    async fn fill_buffer(&mut self) -> Result<(), Status> {
        if self.finished {
            return Ok(());
        }
        match self.recv.data().await {
            Some(Ok(bytes)) => {
                let chunk_len = bytes.len();
                self.buffer.push(bytes);
                if let Err(err) = self.recv.flow_control().release_capacity(chunk_len) {
                    log::debug!("release_capacity failed: {err}");
                }
                Ok(())
            }
            Some(Err(err)) => Err(Status::internal(format!("h2 error: {err}"))),
            None => {
                self.finished = true;
                Ok(())
            }
        }
    }
    fn reserve_decompressed_capacity(&mut self, payload_len: usize) {
        let required_capacity = payload_len
            .saturating_mul(2)
            .max(MIN_DECOMPRESSED_CAPACITY);
        if self.decompressed_buf.capacity() < required_capacity {
            self.decompressed_buf
                .reserve(required_capacity - self.decompressed_buf.capacity());
        }
    }

    fn decompress(&mut self, payload: Bytes) -> Result<&[u8], Status> {
        match self.encoding {
            GrpcEncoding::Identity => {
                log::error!("Received compressed frame but grpc-encoding=identity");
                Err(Status::unimplemented("message compression not negotiated"))
            }
            GrpcEncoding::Zstd => self.decompress_zstd(payload),
            GrpcEncoding::Gzip => self.decompress_gzip(payload),
            GrpcEncoding::Deflate => self.decompress_deflate(payload),
        }
    }

    fn decompress_zstd(&mut self, payload: Bytes) -> Result<&[u8], Status> {
        self.ensure_zstd_decompressor()?;
        let mut required_capacity = self
            .decompressed_buf
            .capacity()
            .max(payload.len().saturating_mul(2))
            .max(MIN_DECOMPRESSED_CAPACITY);

        loop {
            if self.decompressed_buf.capacity() < required_capacity {
                self.decompressed_buf
                    .reserve(required_capacity - self.decompressed_buf.capacity());
            }
            self.decompressed_buf.clear();
            let result = {
                let decompressor = self
                    .zstd
                    .as_mut()
                    .expect("decompressor must be initialized");
                decompressor.decompress_to_buffer(payload.as_ref(), &mut self.decompressed_buf)
            };
            match result {
                Ok(_) => return Ok(self.decompressed_buf.as_slice()),
                Err(err) => {
                    let err_msg = err.to_string();
                    if err.kind() == io::ErrorKind::Other
                        && err_msg.contains("Destination buffer is too small")
                    {
                        required_capacity =
                            required_capacity.checked_mul(2).ok_or_else(|| {
                                log::error!(
                                    "zstd decompression failed: required buffer overflow"
                                );
                                Status::internal(
                                    "zstd decompression failed: output too large",
                                )
                            })?;
                        continue;
                    }
                    log::error!("zstd decompression failed: {err_msg}");
                    return Err(Status::internal(format!(
                        "zstd decompression failed: {err_msg}"
                    )));
                }
            }
        }
    }

    fn decompress_gzip(&mut self, payload: Bytes) -> Result<&[u8], Status> {
        self.reserve_decompressed_capacity(payload.len());
        self.decompressed_buf.clear();
        let mut decoder = GzDecoder::new(payload.as_ref());
        let _ = decoder
            .read_to_end(&mut self.decompressed_buf)
            .map_err(|err| {
                log::error!("gzip decompression failed: {err}");
                Status::internal(format!("gzip decompression failed: {err}"))
            })?;
        Ok(self.decompressed_buf.as_slice())
    }

    fn decompress_deflate(&mut self, payload: Bytes) -> Result<&[u8], Status> {
        self.reserve_decompressed_capacity(payload.len());
        self.decompressed_buf.clear();
        let mut decoder = ZlibDecoder::new(payload.as_ref());
        let _ = decoder
            .read_to_end(&mut self.decompressed_buf)
            .map_err(|err| {
                log::error!("deflate decompression failed: {err}");
                Status::internal(format!("deflate decompression failed: {err}"))
            })?;
        Ok(self.decompressed_buf.as_slice())
    }

    fn ensure_zstd_decompressor(&mut self) -> Result<(), Status> {
        if self.zstd.is_some() {
            return Ok(());
        }
        match ZstdDecompressor::new() {
            Ok(decoder) => {
                self.zstd = Some(decoder);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed to construct zstd decompressor: {err}");
                Err(Status::internal(format!(
                    "failed to initialize zstd decompressor: {err}"
                )))
            }
        }
    }
}

#[async_trait]
impl ArrowRequestStream for GrpcStreamingBody {
    async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
        loop {
            if self.current_frame.is_none() {
                if self.buffer.len() < 5 {
                    if self.finished {
                        return Ok(None);
                    }
                    self.fill_buffer().await?;
                    continue;
                }
                let header = self
                    .buffer
                    .split_frame(5)
                    .expect("buffer len checked above")
                    .into_bytes();
                let compressed = header[0] == 1;
                let len = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;
                self.current_frame = Some(FrameHeader {
                    length: len,
                    compressed,
                });
            }

            if let Some(header) = self.current_frame.take() {
                if self.buffer.len() < header.length {
                    if self.finished {
                        log::error!("Stream ended before full gRPC frame was received");
                        return Err(Status::internal("truncated gRPC frame"));
                    }
                    self.fill_buffer().await?;
                    self.current_frame = Some(header);
                    continue;
                }

                let payload = self
                    .buffer
                    .split_frame(header.length)
                    .expect("buffer len checked above");
                let decoded = if header.compressed {
                    let bytes = self.decompress(payload.into_bytes())?;
                    BatchArrowRecords::decode(bytes)
                } else {
                    BatchArrowRecords::decode(payload)
                };
                let message = decoded.map_err(|e| {
                    log::error!("Failed to decode BatchArrowRecords: {e}");
                    Status::invalid_argument(format!("failed to decode BatchArrowRecords: {e}"))
                })?;
                return Ok(Some(message));
            }
        }
    }
}

struct GrpcMessageEncoder {
    compression: GrpcEncoding,
    frame_buf: BytesMut,
    message_buf: BytesMut,
    compressed_buf: Vec<u8>,
    zstd: Option<ZstdCompressor<'static>>,
}

impl GrpcMessageEncoder {
    fn new(compression: GrpcEncoding) -> Self {
        Self {
            compression,
            frame_buf: BytesMut::with_capacity(512),
            message_buf: BytesMut::with_capacity(512),
            compressed_buf: Vec::new(),
            zstd: None,
        }
    }

    fn encode<M: Message>(&mut self, message: &M) -> Result<Bytes, Status> {
        self.message_buf.clear();
        message
            .encode(&mut self.message_buf)
            .map_err(|e| Status::internal(format!("failed to encode response: {e}")))?;
        let uncompressed = self.message_buf.split().freeze();

        match self.compression {
            GrpcEncoding::Identity => {
                self.finish_frame(false, uncompressed.as_ref())
            }
            GrpcEncoding::Zstd => {
                self.compress_zstd(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
            GrpcEncoding::Gzip => {
                self.compress_gzip(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
            GrpcEncoding::Deflate => {
                self.compress_deflate(uncompressed.as_ref())?;
                let mut payload = mem::take(&mut self.compressed_buf);
                let result = self.finish_frame(true, payload.as_slice());
                payload.clear();
                self.compressed_buf = payload;
                result
            }
        }
    }

    fn finish_frame(&mut self, compressed: bool, payload: &[u8]) -> Result<Bytes, Status> {
        let needed = 5 + payload.len();
        if self.frame_buf.capacity() < needed {
            self.frame_buf
                .reserve(needed - self.frame_buf.capacity());
        }
        self.frame_buf.clear();
        self.frame_buf.put_u8(u8::from(compressed));
        self.frame_buf.put_u32(payload.len() as u32);
        self.frame_buf.extend_from_slice(payload);
        Ok(self.frame_buf.split().freeze())
    }

    fn compress_zstd(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.ensure_zstd_encoder()?;
        let mut required_capacity = payload.len().max(MIN_COMPRESSED_CAPACITY);
        loop {
            if self.compressed_buf.len() != required_capacity {
                self.compressed_buf.resize(required_capacity, 0);
            }
            let result = {
                let encoder = self
                    .zstd
                    .as_mut()
                    .expect("zstd encoder must exist");
                encoder.compress_to_buffer(payload, self.compressed_buf.as_mut_slice())
            };
            match result {
                Ok(written) => {
                    self.compressed_buf.truncate(written);
                    return Ok(());
                }
                Err(err)
                    if err.kind() == io::ErrorKind::Other
                        && err.to_string().contains("Destination buffer is too small") =>
                {
                    required_capacity = required_capacity.checked_mul(2).ok_or_else(|| {
                        log::error!("zstd compression failed: required buffer overflow");
                        Status::internal("zstd compression failed: output too large")
                    })?;
                }
                Err(err) => {
                    log::error!("zstd compression failed: {err}");
                    return Err(Status::internal(format!(
                        "zstd compression failed: {err}"
                    )));
                }
            }
        }
    }

    fn compress_gzip(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.compressed_buf.clear();
        {
            let mut encoder =
                GzEncoder::new(&mut self.compressed_buf, Compression::default());
            encoder
                .write_all(payload)
                .and_then(|_| encoder.try_finish())
                .map_err(|err| {
                    log::error!("gzip compression failed: {err}");
                    Status::internal(format!("gzip compression failed: {err}"))
                })?;
        }
        Ok(())
    }

    fn compress_deflate(&mut self, payload: &[u8]) -> Result<(), Status> {
        self.compressed_buf.clear();
        {
            let mut encoder =
                ZlibEncoder::new(&mut self.compressed_buf, Compression::default());
            encoder
                .write_all(payload)
                .and_then(|_| encoder.try_finish())
                .map_err(|err| {
                    log::error!("deflate compression failed: {err}");
                    Status::internal(format!("deflate compression failed: {err}"))
                })?;
        }
        Ok(())
    }

    fn ensure_zstd_encoder(&mut self) -> Result<(), Status> {
        if self.zstd.is_some() {
            return Ok(());
        }
        match ZstdCompressor::new(0) {
            Ok(encoder) => {
                self.zstd = Some(encoder);
                Ok(())
            }
            Err(err) => {
                log::error!("Failed to construct zstd compressor: {err}");
                Err(Status::internal(format!(
                    "failed to initialize zstd compressor: {err}"
                )))
            }
        }
    }
}

fn stream_batch_statuses<S, T, F>(
    input_stream: S,
    effect_handler: local::EffectHandler<OtapPdata>,
    ack_registry: Option<AckRegistry>,
    otap_batch: F,
    max_in_flight_per_connection: usize,
) -> ArrowBatchStatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    let state = ArrowBatchStreamState::new(
        input_stream,
        effect_handler,
        ack_registry,
        otap_batch,
        max_in_flight_per_connection,
    );
    ArrowBatchStatusStream::new(state)
}

struct ArrowBatchStatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    state: Option<ArrowBatchStreamState<S, T, F>>,
    pending: Option<LocalBoxFuture<'static, (ArrowBatchStreamState<S, T, F>, StreamStep)>>,
    finished: bool,
}

impl<S, T, F> ArrowBatchStatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    fn new(state: ArrowBatchStreamState<S, T, F>) -> Self {
        Self {
            state: Some(state),
            pending: None,
            finished: false,
        }
    }

    fn drive_next(
        state: ArrowBatchStreamState<S, T, F>,
    ) -> LocalBoxFuture<'static, (ArrowBatchStreamState<S, T, F>, StreamStep)> {
        Box::pin(async move {
            let mut state = state;
            let step = state.next_item().await;
            (state, step)
        })
    }
}

impl<S, T, F> Stream for ArrowBatchStatusStream<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    type Item = Result<BatchStatus, Status>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.finished {
            return Poll::Ready(None);
        }

        if this.pending.is_none() {
            let state = match this.state.take() {
                Some(state) => state,
                None => {
                    this.finished = true;
                    return Poll::Ready(None);
                }
            };
            this.pending = Some(Self::drive_next(state));
        }

        match this
            .pending
            .as_mut()
            .expect("pending future must exist")
            .as_mut()
            .poll(cx)
        {
            Poll::Pending => Poll::Pending,
            Poll::Ready((state, step)) => {
                this.pending = None;
                match step {
                    StreamStep::Yield(item) => {
                        this.state = Some(state);
                        Poll::Ready(Some(item))
                    }
                    StreamStep::Done => {
                        this.finished = true;
                        this.state = None;
                        Poll::Ready(None)
                    }
                }
            }
        }
    }
}

struct ArrowBatchStreamState<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    input_stream: S,
    consumer: Consumer,
    effect_handler: local::EffectHandler<OtapPdata>,
    state: Option<AckRegistry>,
    otap_batch: F,
    in_flight: LocalFutureSet<LocalAckWaitFuture>,
    max_in_flight: usize,
    finished: bool,
    _marker: PhantomData<fn() -> T>,
}

enum StreamStep {
    Yield(Result<BatchStatus, Status>),
    Done,
}

enum PreparedBatch {
    Enqueued,
    Immediate(StreamStep),
}

impl<S, T, F> ArrowBatchStreamState<S, T, F>
where
    S: ArrowRequestStream + Unpin,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static + Unpin,
{
    fn new(
        input_stream: S,
        effect_handler: local::EffectHandler<OtapPdata>,
        state: Option<AckRegistry>,
        otap_batch: F,
        max_in_flight_per_connection: usize,
    ) -> Self {
        Self {
            input_stream,
            consumer: Consumer::default(),
            effect_handler,
            state,
            otap_batch,
            in_flight: LocalFutureSet::with_capacity(max_in_flight_per_connection.max(1)),
            max_in_flight: max_in_flight_per_connection.max(1),
            finished: false,
            _marker: PhantomData,
        }
    }

    async fn next_item(&mut self) -> StreamStep {
        if let Some(step) = self.fill_inflight().await {
            return step;
        }

        match poll_fn(|cx| self.in_flight.poll_next(cx)).await {
            Some(step) => {
                if matches!(step, StreamStep::Done) {
                    self.finished = true;
                }
                step
            }
            None => StreamStep::Done,
        }
    }

    async fn fill_inflight(&mut self) -> Option<StreamStep> {
        while !self.finished && self.in_flight.len() < self.max_in_flight {
            match self.input_stream.next_message().await {
                Ok(Some(batch)) => match self.enqueue_batch(batch).await {
                    PreparedBatch::Enqueued => continue,
                    PreparedBatch::Immediate(step) => return Some(step),
                },
                Ok(None) => {
                    self.finished = true;
                    break;
                }
                Err(status) => {
                    self.finished = true;
                    return Some(StreamStep::Yield(Err(status)));
                }
            }
        }
        None
    }

    async fn enqueue_batch(&mut self, mut batch: BatchArrowRecords) -> PreparedBatch {
        let batch_id = batch.batch_id;

        let batch = match self.consumer.consume_bar(&mut batch) {
            Ok(batch) => batch,
            Err(e) => {
                log::error!("Error decoding OTAP Batch: {e:?}. Closing stream");
                self.finished = true;
                return PreparedBatch::Immediate(StreamStep::Done);
            }
        };

        let batch = from_record_messages::<T>(batch);
        let otap_batch_as_otap_arrow_records = (self.otap_batch)(batch);
        let mut otap_pdata =
            OtapPdata::new(Context::default(), otap_batch_as_otap_arrow_records.into());

        let wait_token = if let Some(state) = self.state.clone() {
            match state.allocate() {
                None => {
                    log::error!("Too many concurrent requests");
                    return PreparedBatch::Immediate(StreamStep::Yield(Ok(
                        local_overloaded_status(batch_id),
                    )));
                }
                Some(token) => {
                    self.effect_handler.subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        token.to_calldata(),
                        &mut otap_pdata,
                    );
                    Some((state, token))
                }
            }
        } else {
            None
        };

        if let Err(e) = self.effect_handler.send_message(otap_pdata).await {
            log::error!("Failed to send to pipeline: {e}");
            self.finished = true;
            return PreparedBatch::Immediate(StreamStep::Done);
        };

        if let Some((state, token)) = wait_token {
            if let Err(_future) = self
                .in_flight
                .push(LocalAckWaitFuture::new(batch_id, token, state))
            {
                log::error!("In-flight future set unexpectedly full");
                return PreparedBatch::Immediate(StreamStep::Yield(Ok(local_overloaded_status(
                    batch_id,
                ))));
            }
            PreparedBatch::Enqueued
        } else {
            PreparedBatch::Immediate(StreamStep::Yield(Ok(local_success_status(batch_id))))
        }
    }
}

struct LocalFutureSet<F> {
    futures: FuturesUnordered<F>,
    capacity: usize,
}

impl<F> LocalFutureSet<F> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            capacity,
        }
    }

    fn len(&self) -> usize {
        self.futures.len()
    }

    fn push(&mut self, future: F) -> Result<(), F> {
        if self.len() >= self.capacity {
            Err(future)
        } else {
            self.futures.push(future);
            Ok(())
        }
    }

    fn poll_next(&mut self, cx: &mut TaskContext<'_>) -> Poll<Option<<F as Future>::Output>>
    where
        F: Future + Unpin,
    {
        Pin::new(&mut self.futures).poll_next(cx)
    }
}

struct LocalAckWaitFuture {
    batch_id: i64,
    token: AckToken,
    state: AckRegistry,
    completed: bool,
}

impl LocalAckWaitFuture {
    fn new(batch_id: i64, token: AckToken, state: AckRegistry) -> Self {
        Self {
            batch_id,
            token,
            state,
            completed: false,
        }
    }
}

impl Future for LocalAckWaitFuture {
    type Output = StreamStep;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.state.poll_slot(this.token, cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(LocalPollResult::Ack) => {
                this.completed = true;
                Poll::Ready(StreamStep::Yield(Ok(local_success_status(this.batch_id))))
            }
            Poll::Ready(LocalPollResult::Nack(reason)) => {
                this.completed = true;
                Poll::Ready(StreamStep::Yield(Ok(local_nack_status(
                    this.batch_id,
                    reason,
                ))))
            }
            Poll::Ready(LocalPollResult::Cancelled) => {
                this.completed = true;
                Poll::Ready(StreamStep::Done)
            }
        }
    }
}

impl Drop for LocalAckWaitFuture {
    fn drop(&mut self) {
        if !self.completed {
            self.state.cancel(self.token);
        }
    }
}

enum LocalPollResult {
    Ack,
    Nack(String),
    Cancelled,
}

#[derive(Clone)]
struct AckRegistry {
    inner: Rc<RefCell<AckRegistryInner>>,
}

struct AckRegistryInner {
    slots: Box<[AckSlot]>,
    free_stack: Vec<usize>,
}

impl AckRegistry {
    fn new(max_size: usize) -> Self {
        let mut slots = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            slots.push(AckSlot::new());
        }
        let mut free_stack = Vec::with_capacity(max_size);
        for idx in (0..max_size).rev() {
            free_stack.push(idx);
        }
        Self {
            inner: Rc::new(RefCell::new(AckRegistryInner {
                slots: slots.into_boxed_slice(),
                free_stack,
            })),
        }
    }

    fn allocate(&self) -> Option<AckToken> {
        let mut inner = self.inner.borrow_mut();
        let slot_index = inner.free_stack.pop()?;
        let slot = &mut inner.slots[slot_index];
        debug_assert!(matches!(slot.state, SlotState::Free));
        slot.generation = slot.generation.wrapping_add(1);
        slot.state = SlotState::Waiting(WaitingSlot::new());
        Some(AckToken {
            slot_index,
            generation: slot.generation,
        })
    }

    fn complete(&self, token: AckToken, result: Result<(), String>) -> RouteResponse {
        let mut inner = self.inner.borrow_mut();
        let Some(slot) = inner.slots.get_mut(token.slot_index) else {
            return RouteResponse::Invalid;
        };
        if slot.generation != token.generation {
            return RouteResponse::Expired;
        }
        match &mut slot.state {
            SlotState::Waiting(waiting) => {
                waiting.outcome = match result {
                    Ok(()) => AckOutcome::Ack,
                    Err(reason) => AckOutcome::Nack(reason),
                };
                if let Some(waker) = waiting.waker.take() {
                    waker.wake();
                }
                RouteResponse::Sent
            }
            SlotState::Free => RouteResponse::Expired,
        }
    }

    fn poll_slot(&self, token: AckToken, cx: &mut TaskContext<'_>) -> Poll<LocalPollResult> {
        let mut inner = self.inner.borrow_mut();
        let Some(slot) = inner.slots.get_mut(token.slot_index) else {
            return Poll::Ready(LocalPollResult::Cancelled);
        };
        if slot.generation != token.generation {
            return Poll::Ready(LocalPollResult::Cancelled);
        }
        match &mut slot.state {
            SlotState::Waiting(waiting) => match &mut waiting.outcome {
                AckOutcome::Pending => {
                    let replace = match &waiting.waker {
                        Some(existing) => !existing.will_wake(cx.waker()),
                        None => true,
                    };
                    if replace {
                        waiting.waker = Some(cx.waker().clone());
                    }
                    Poll::Pending
                }
                AckOutcome::Ack => {
                    slot.state = SlotState::Free;
                    inner.free_stack.push(token.slot_index);
                    Poll::Ready(LocalPollResult::Ack)
                }
                AckOutcome::Nack(reason) => {
                    let reason = mem::take(reason);
                    slot.state = SlotState::Free;
                    inner.free_stack.push(token.slot_index);
                    Poll::Ready(LocalPollResult::Nack(reason))
                }
            },
            SlotState::Free => Poll::Ready(LocalPollResult::Cancelled),
        }
    }

    fn cancel(&self, token: AckToken) {
        let mut inner = self.inner.borrow_mut();
        if let Some(slot) = inner.slots.get_mut(token.slot_index) {
            if slot.generation != token.generation {
                return;
            }
            if matches!(slot.state, SlotState::Waiting(_)) {
                slot.state = SlotState::Free;
                inner.free_stack.push(token.slot_index);
            }
        }
    }
}

struct AckSlot {
    generation: u32,
    state: SlotState,
}

impl AckSlot {
    fn new() -> Self {
        Self {
            generation: 0,
            state: SlotState::Free,
        }
    }
}

enum SlotState {
    Free,
    Waiting(WaitingSlot),
}

struct WaitingSlot {
    waker: Option<Waker>,
    outcome: AckOutcome,
}

impl WaitingSlot {
    fn new() -> Self {
        Self {
            waker: None,
            outcome: AckOutcome::Pending,
        }
    }
}

enum AckOutcome {
    Pending,
    Ack,
    Nack(String),
}

#[derive(Clone, Copy)]
struct AckToken {
    slot_index: usize,
    generation: u32,
}

impl AckToken {
    fn to_calldata(self) -> CallData {
        smallvec![
            Context8u8::from(self.slot_index as u64),
            Context8u8::from(self.generation as u64)
        ]
    }

    fn from_calldata(calldata: &CallData) -> Option<Self> {
        if calldata.len() < 2 {
            return None;
        }
        let slot_index = usize::try_from(u64::from(calldata[0])).ok()?;
        let generation = u64::from(calldata[1]) as u32;
        Some(Self {
            slot_index,
            generation,
        })
    }
}

#[derive(Clone, Default)]
struct AckRegistries {
    logs: Option<AckRegistry>,
    metrics: Option<AckRegistry>,
    traces: Option<AckRegistry>,
}

impl AckRegistries {
    fn new(
        logs: Option<AckRegistry>,
        metrics: Option<AckRegistry>,
        traces: Option<AckRegistry>,
    ) -> Self {
        Self {
            logs,
            metrics,
            traces,
        }
    }

    fn ack_registry_for_signal(&self, signal: SignalType) -> Option<&AckRegistry> {
        match signal {
            SignalType::Logs => self.logs.as_ref(),
            SignalType::Metrics => self.metrics.as_ref(),
            SignalType::Traces => self.traces.as_ref(),
        }
    }
}

fn route_local_ack_response(states: &AckRegistries, ack: AckMsg<OtapPdata>) -> RouteResponse {
    let Some(token) = AckToken::from_calldata(&ack.calldata) else {
        return RouteResponse::Invalid;
    };
    states
        .ack_registry_for_signal(ack.accepted.signal_type())
        .map(|state| state.complete(token, Ok(())))
        .unwrap_or(RouteResponse::None)
}

fn route_local_nack_response(
    states: &AckRegistries,
    mut nack: NackMsg<OtapPdata>,
) -> RouteResponse {
    let Some(token) = AckToken::from_calldata(&nack.calldata) else {
        return RouteResponse::Invalid;
    };
    let reason = mem::take(&mut nack.reason);
    states
        .ack_registry_for_signal(nack.refused.signal_type())
        .map(|state| state.complete(token, Err(reason)))
        .unwrap_or(RouteResponse::None)
}

fn local_success_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }
}

fn local_nack_status(batch_id: i64, reason: String) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: format!("Pipeline processing failed: {reason}"),
    }
}

fn local_overloaded_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: ProtoStatusCode::Unavailable as i32,
        status_message: "Pipeline processing failed: Too many concurrent requests".to_string(),
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
