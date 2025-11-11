// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Experimental OTAP receiver that serves the Arrow gRPC endpoints directly on top of the `h2`
//! crate.  This variant keeps all request handling on the current thread so it can integrate with
//! the thread-per-core runtime without requiring `Send + Sync` futures.
//!
//! ToDo grpc-accept-encoding parsing: read client preference list, validate tokens, intersect with supported codecs, and propagate the chosen response codec through request handling.
//  ToDo Preferred response encoding: store negotiated codec in per-stream state (fall back to identity), set the response header accordingly, and ensure encode_message flips the compression bit and runs the right compressor.
//  ToDo Codec support matrix: extend GrpcEncoding to include gzip & snappy. Wire in the matching decompress/encode routines with shared helpers for both request frames and response frames.
//  ToDo Error handling & metrics: surface clear statuses when the client requests unsupported codecs, log negotiation results, and add counters for negotiated/unsupported compression cases.
//  ToDo Tests: add unit/integration coverage for accept header parsing, per-codec request/response flows, and zstdarrow alias handling to prevent regressions.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::common;
use crate::otap_grpc::otlp::server::RouteResponse;
use crate::otap_grpc::{ArrowRequestStream, GrpcServerSettings, Settings, per_connection_limit};
use crate::otap_receiver::OtapReceiverMetrics;
use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{Stream, StreamExt};
use futures::future::{LocalBoxFuture, poll_fn};
use h2::server::{self, SendResponse};
use http::{HeaderMap, HeaderValue, Request, Response, StatusCode as HttpStatusCode};
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, CallData, Context8u8, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    BatchArrowRecords, BatchStatus, StatusCode as ProtoStatusCode,
};
use otel_arrow_rust::{
    Consumer,
    otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces, from_record_messages},
};
use prost::Message;
use serde::Deserialize;
use smallvec::smallvec;
use std::cell::RefCell;
use std::future::Future;
use std::io::Cursor;
use std::mem;
use std::marker::PhantomData;
use std::ops::Add;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll, Waker};
use std::time::{Duration, Instant};
use tokio::task::spawn_local;
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tonic::transport::server::TcpIncoming;
use zstd::stream::decode_all;

const OTEL_RECEIVER_URN: &str = "urn:otel:otap2:receiver";
const ARROW_LOGS_SERVICE: &str = "/opentelemetry.proto.experimental.arrow.v1.ArrowLogsService/ArrowLogs";
const ARROW_METRICS_SERVICE: &str = "/opentelemetry.proto.experimental.arrow.v1.ArrowMetricsService/ArrowMetrics";
const ARROW_TRACES_SERVICE: &str = "/opentelemetry.proto.experimental.arrow.v1.ArrowTracesService/ArrowTraces";

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

        let router = Rc::new(ArrowRouter {
            effect_handler: effect_handler.clone(),
            logs_ack_registry: logs_ack_registry,
            metrics_ack_registry: metrics_ack_registry,
            traces_ack_registry: traces_ack_registry,
            max_in_flight_per_connection: max_in_flight,
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

            server_result = run_h2_server(
                &mut incoming,
                Rc::clone(&config),
                Rc::clone(&router),
                cancel_token.clone(),
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

async fn run_h2_server(
    incoming: &mut TcpIncoming,
    config: Rc<GrpcServerSettings>,
    router: Rc<ArrowRouter>,
    cancel: CancellationToken,
) -> Result<(), std::io::Error> {
    let mut connections: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            Some(res) = incoming.next() => {
                let socket = res?;
                let peer = socket
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string());
                // log::info!("Accepted OTAP H2 connection from {}", peer);
                if let Err(e) = socket.set_nodelay(config.tcp_nodelay) {
                    log::warn!("Failed to set TCP_NODELAY: {e}");
                }
                let builder = build_h2_builder(&config);
                let router = Rc::clone(&router);

                let handle = spawn_local(async move {
                    if let Err(err) = handle_connection(socket, builder, router).await {
                        log::debug!("H2 connection ended with error: {err}");
                    }
                });
                connections.push(handle);
            }
            else => break,
        }

        connections.retain(|handle| !handle.is_finished());
    }

    for handle in connections {
        if !handle.is_finished() {
            if let Err(err) = handle.await {
                log::debug!("H2 connection task join error: {err}");
            }
        }
    }

    Ok(())
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

async fn handle_connection(
    socket: tokio::net::TcpStream,
    builder: server::Builder,
    router: Rc<ArrowRouter>,
) -> Result<(), h2::Error> {
    let mut connection = builder.handshake(socket).await?;
    log::trace!("H2 handshake established");
    while let Some(result) = connection.accept().await {
        let (request, respond) = result?;
        let router = router.clone();
        let _ = spawn_local(async move {
            log::trace!("Received new H2 stream for path {}", request.uri().path());
            if let Err(status) = router.handle_request(request, respond).await {
                log::debug!("Request failed: {}", status);
            }
        });
    }
    Ok(())
}

struct ArrowRouter {
    effect_handler: local::EffectHandler<OtapPdata>,
    logs_ack_registry: Option<AckRegistry>,
    metrics_ack_registry: Option<AckRegistry>,
    traces_ack_registry: Option<AckRegistry>,
    max_in_flight_per_connection: usize,
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
                respond_with_error(respond, Status::unimplemented("unknown method"));
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
        let encoding = parse_grpc_encoding(request.headers())?;
        let recv_stream = request.into_body();
        let body = GrpcStreamingBody::new(recv_stream, encoding);

        let mut status_stream = stream_batch_statuses::<GrpcStreamingBody, T, _>(
            body,
            self.effect_handler.clone(),
            ack_registry,
            otap_batch,
            self.max_in_flight_per_connection,
        );

        let response = Response::builder()
            .status(HttpStatusCode::OK)
            .header("content-type", "application/grpc")
            .header("grpc-encoding", "identity")        // support compressed responses later
            .body(())
            .unwrap();
        let mut send_stream = respond
            .send_response(response, false)
            .map_err(|e| Status::internal(format!("failed to send response headers: {e}")))?;

        while let Some(next) = status_stream.next().await {
            match next {
                Ok(status) => {
                    // log::info!(
                    //     "Sending batch status id={} code={}",
                    //     status.batch_id,
                    //     status.status_code
                    // );
                    let bytes = encode_message(&status)?;
                    if let Err(e) = send_stream.send_data(bytes, false) {
                        log::debug!("send_data failed: {e}");
                        return Ok(());
                    }
                }
                Err(status) => {
                    log::error!("Stream aborted with status {}", status);
                    send_error_trailers(send_stream, status);
                    return Ok(());
                }
            }
        }

        send_ok_trailers(send_stream);
        Ok(())
    }
}

fn parse_grpc_encoding(headers: &HeaderMap) -> Result<GrpcEncoding, Status> {
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
            let enc = value
                .to_str()
                .map_err(|_| {
                    log::error!("Non-UTF8 grpc-encoding header");
                    Status::invalid_argument("invalid grpc-encoding header")
                })?
                .trim()
                .to_ascii_lowercase();
            let enc_str = enc.as_str();
            const PREFIX: &str = "zstdarrow";

            if enc_str.is_empty() || enc_str == "identity" {
                Ok(GrpcEncoding::Identity)
            } else if enc_str == "zstd" {
                Ok(GrpcEncoding::Zstd)
            } else if enc_str.starts_with(PREFIX) {
                let tail = &enc_str[PREFIX.len()..];
                if tail.len() == 1 && tail.as_bytes()[0].is_ascii_digit() {
                    Ok(GrpcEncoding::Zstd)
                } else {
                    log::error!("Unsupported grpc-encoding {}", enc_str);
                    Err(Status::unimplemented("grpc compression not supported"))
                }
            } else {
                log::error!("Unsupported grpc-encoding {}", enc_str);
                Err(Status::unimplemented("grpc compression not supported"))
            }

        }
    }
}

#[derive(Clone, Copy)]
enum GrpcEncoding {
    Identity,
    Zstd,
    // ToDo: Gzip, Snappy (to follow Go implementation)
}

struct GrpcStreamingBody {
    recv: h2::RecvStream,
    buffer: BytesMut,
    current_frame: Option<FrameHeader>,
    finished: bool,
    encoding: GrpcEncoding,
}

#[derive(Clone, Copy)]
struct FrameHeader {
    length: usize,
    compressed: bool,
}

impl GrpcStreamingBody {
    fn new(recv: h2::RecvStream, encoding: GrpcEncoding) -> Self {
        Self {
            recv,
            buffer: BytesMut::with_capacity(1024),
            current_frame: None,
            finished: false,
            encoding,
        }
    }

    async fn fill_buffer(&mut self) -> Result<(), Status> {
        if self.finished {
            return Ok(());
        }
        match self.recv.data().await {
            Some(Ok(bytes)) => {
                self.buffer.extend_from_slice(&bytes);
                if let Err(err) = self.recv.flow_control().release_capacity(bytes.len()) {
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
    fn decompress(&self, payload: Bytes) -> Result<Bytes, Status> {
        match self.encoding {
            GrpcEncoding::Identity => {
                log::error!("Received compressed frame but grpc-encoding=identity");
                Err(Status::unimplemented("message compression not negotiated"))
            }
            GrpcEncoding::Zstd => {
                let cursor = Cursor::new(payload.as_ref());
                decode_all(cursor).map(Bytes::from).map_err(|e| {
                    log::error!("zstd decompression failed: {e}");
                    Status::internal(format!("zstd decompression failed: {e}"))
                })
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
                let compressed = self.buffer[0] == 1;
                let len = u32::from_be_bytes([
                    self.buffer[1],
                    self.buffer[2],
                    self.buffer[3],
                    self.buffer[4],
                ]) as usize;
                self.buffer.advance(5);
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

                let payload = self.buffer.split_to(header.length).freeze();
                let message_bytes = if header.compressed {
                    self.decompress(payload)?
                } else {
                    payload
                };
                let message = BatchArrowRecords::decode(message_bytes.clone()).map_err(|e| {
                    log::error!("Failed to decode BatchArrowRecords: {e}");
                    Status::invalid_argument(format!("failed to decode BatchArrowRecords: {e}"))
                })?;
                return Ok(Some(message));
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

        loop {
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
                Poll::Pending => return Poll::Pending,
                Poll::Ready((state, step)) => {
                    this.pending = None;
                    match step {
                        StreamStep::Yield(item) => {
                            this.state = Some(state);
                            return Poll::Ready(Some(item));
                        }
                        StreamStep::Done => {
                            this.finished = true;
                            this.state = None;
                            return Poll::Ready(None);
                        }
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
                return PreparedBatch::Immediate(StreamStep::Yield(Ok(
                    local_overloaded_status(batch_id),
                )));
            }
            PreparedBatch::Enqueued
        } else {
            PreparedBatch::Immediate(StreamStep::Yield(Ok(local_success_status(
                batch_id,
            ))))
        }
    }
}

struct LocalFutureSet<F> {
    slots: Vec<Option<F>>,
    active: usize,
}

impl<F> LocalFutureSet<F> {
    fn with_capacity(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);
        Self { slots, active: 0 }
    }

    fn len(&self) -> usize {
        self.active
    }

    fn push(&mut self, future: F) -> Result<(), F> {
        if self.active == self.slots.len() {
            return Err(future);
        }
        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(future);
                self.active += 1;
                return Ok(());
            }
        }
        Err(future)
    }

    fn poll_next(
        &mut self,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Option<<F as Future>::Output>>
    where
        F: Future + Unpin,
    {
        if self.active == 0 {
            return Poll::Ready(None);
        }
        for slot in &mut self.slots {
            if let Some(future) = slot.as_mut() {
                let mut pinned = Pin::new(future);
                match pinned.as_mut().poll(cx) {
                    Poll::Ready(output) => {
                        *slot = None;
                        self.active -= 1;
                        return Poll::Ready(Some(output));
                    }
                    Poll::Pending => {}
                }
            }
        }
        Poll::Pending
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

    fn ack_registry_for_signal(
        &self,
        signal: SignalType,
    ) -> Option<&AckRegistry> {
        match signal {
            SignalType::Logs => self.logs.as_ref(),
            SignalType::Metrics => self.metrics.as_ref(),
            SignalType::Traces => self.traces.as_ref(),
        }
    }
}

fn route_local_ack_response(
    states: &AckRegistries,
    ack: AckMsg<OtapPdata>,
) -> RouteResponse {
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

fn encode_message<M: Message>(message: &M) -> Result<Bytes, Status> {
    let mut buf = BytesMut::with_capacity(5 + message.encoded_len());
    buf.put_u8(0);
    buf.put_u32(message.encoded_len() as u32);
    message
        .encode(&mut buf)
        .map_err(|e| Status::internal(format!("failed to encode response: {e}")))?;
    Ok(buf.freeze())
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

fn respond_with_error(mut respond: SendResponse<Bytes>, status: Status) {
    let response = Response::builder()
        .status(HttpStatusCode::OK)
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();
    match respond.send_response(response, false) {
        Ok(stream) => send_error_trailers(stream, status),
        Err(e) => log::debug!("failed to send error response: {e}"),
    }
}
