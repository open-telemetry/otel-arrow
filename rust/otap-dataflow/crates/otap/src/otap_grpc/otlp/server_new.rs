// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementations of OTLP gRPC service servers that produce `OtapPdata` for the pipeline.
//!
//! Request lifecycle:
//! - decode: wrap incoming OTLP bytes into `OtapPdata` without deserializing
//! - subscribe (optional): when `wait_for_result` is set, register ACK/NACK interests with calldata
//! - send: forward the payload into the pipeline
//! - wait (optional): block until an ACK/NACK arrives through the routed slot
//! - respond: return success or convert NACK/channel errors into gRPC status

use std::convert::Infallible;
use std::fmt::Display;
use std::mem;
use std::sync::Arc;
use std::task::Poll;

use crate::accessory::slots::{Key as SlotKey, State as SlotsState};
use crate::bearer_authorization::{AuthorizationRejection, authorize_bearer};
use crate::otlp_metrics::{OtlpProtocol, OtlpReceiverMetrics};
use crate::pdata::{Context, OtapPdata};
use crate::rate_limit_layer::{
    grpc_rate_limit_burst_exceeded_status, grpc_rate_limit_saturated_status, grpc_rate_limit_status,
};
use bytes::{BufMut, Bytes};
use futures::future::BoxFuture;
use http::{Request, Response};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::transport_headers::TransportHeaders;
use otel_arrow_dfe_engine::admission::{AdmissionContext, AdmissionDecision, SharedAdmissionGate};
use otel_arrow_dfe_engine::capability::auth::AuthorizedIdentity;
use otel_arrow_dfe_engine::control::{CallData, NackMsg};
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer;
use otel_arrow_dfe_engine::shared::receiver::EffectHandler;
use otel_arrow_dfe_engine::{
    Interests, MessageSourceSharedEffectHandlerExtension, ProducerEffectHandlerExtension,
};

use crate::nack_status::classify_nack;
use otel_arrow_dfe_pdata::OtapPayload;
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use otel_arrow_dfe_telemetry::common_attributes::ReceiverRejectionErrorType;
use parking_lot::Mutex;
use prost::Message;
use prost::bytes::Buf;
use std::sync::OnceLock;
use tokio::sync::oneshot;
use tonic::body::Body;
use tonic::codec::{Codec, DecodeBuf, Decoder, EnabledCompressionEncodings, EncodeBuf, Encoder};
use tonic::server::{Grpc, NamedService, UnaryService};
use tonic::{Code, Status};
use tower::{Layer, Service};

use crate::otap_grpc::common::peer_addr_from_extensions;

/// Tracks outstanding request subscriptions for a single signal so ACK/NACK responses can be routed
/// back to the waiting caller. When `wait_for_result` is disabled the receiver skips creating this
/// map entirely, avoiding extra allocations on the hot path.
#[derive(Clone)]
pub struct AckSlot(
    // parking_lot mutex keeps the hot ACK/NACK path lock-free from poisoning.
    pub(crate) Arc<Mutex<SlotsState<oneshot::Sender<Result<(), NackMsg<OtapPdata>>>>>>,
);

/// Receiver side of a wait-for-result subscription slot.
pub type AckSlotReceiver = oneshot::Receiver<Result<(), NackMsg<OtapPdata>>>;

impl AckSlot {
    /// Build a new per-signal slot map sized for the configured concurrency.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self(Arc::new(Mutex::new(SlotsState::new(max_size))))
    }

    /// Returns true when there are no outstanding wait-for-result slots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.lock().is_empty()
    }

    /// Completes all outstanding waiters with a shutdown Nack.
    pub fn force_shutdown(&self, signal: SignalType, reason: &str) {
        self.0.lock().drain(|sender| {
            let _ = sender.send(Err(NackMsg::new(
                reason,
                OtapPdata::new_todo_context(OtapPayload::empty(signal)),
            )));
        });
    }

    /// Allocate a raw slot key plus its paired receiver.
    pub(crate) fn allocate_slot(&self) -> Option<(SlotKey, AckSlotReceiver)> {
        let mut guard = self.0.lock();
        guard.allocate(oneshot::channel)
    }

    /// Allocate a wait-for-result subscription in calldata form.
    #[must_use]
    pub fn allocate_waiter(&self) -> Option<(CallData, AckSlotReceiver)> {
        self.allocate_slot().map(|(key, rx)| (key.into(), rx))
    }

    /// Cancel an outstanding wait-for-result subscription.
    pub(crate) fn cancel_slot(&self, key: SlotKey) {
        self.0.lock().cancel(key);
    }
}

/// The outcome from RouteResponse
pub enum RouteResponse {
    /// The Ack/Nack was sent.
    Sent,
    /// The Ack/Nack may have timed out.
    Expired,
    /// No subscription was found.
    None,
    /// The Ack/Nack had invalid call data.
    Invalid,
}

impl AckSlot {
    /// Routes the final outcome into the registered slot matching the provided `CallData`.
    #[must_use]
    pub fn route_response(
        &self,
        calldata: CallData,
        result: Result<(), NackMsg<OtapPdata>>,
    ) -> RouteResponse {
        // Decode slot key from calldata
        let key: SlotKey = match calldata.try_into() {
            Ok(data) => data,
            Err(_) => return RouteResponse::Invalid,
        };

        // Try to take the channel from the slot under the mutex.
        let chan = self.0.lock().take(key);

        // Try to send.
        if chan.and_then(|sender| sender.send(result).ok()).is_some() {
            RouteResponse::Sent
        } else {
            RouteResponse::Expired
        }
    }
}

/// Common settings for OTLP receivers.
/// Per-signal server settings derived from user configuration and shared with the services.
#[derive(Clone, Debug)]
pub struct OtlpServerSettings {
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
    /// Maximum size for inbound gRPC messages.
    /// ToDo: Note the Collector calls this max_recv_msg_size_mib,
    /// consider max_receive_message_size to reduce difference, add
    /// serde::from byte sizes.
    /// https://github.com/open-telemetry/opentelemetry-collector/blob/152042ebfa9d67731b23ae3cb5b23f585e13d2a2/config/configgrpc/configgrpc.go#L183
    pub max_decoding_message_size: Option<usize>,
    /// Request compression allowed
    pub request_compression_encodings: EnabledCompressionEncodings,
    /// Response compression used
    pub response_compression_encodings: EnabledCompressionEncodings,
}

/// Encodes a default response for repeated use.
fn encode_response<T: Message + Default>() -> Bytes {
    let mut buf = Vec::with_capacity(T::default().encoded_len());
    T::default().encode(&mut buf).expect("encode response");
    Bytes::from(buf)
}

/// Precomputed empty responses per signal to avoid per-call prost encoding.
fn precomputed_response(signal: SignalType) -> &'static [u8] {
    static LOGS: OnceLock<Bytes> = OnceLock::new();
    static METRICS: OnceLock<Bytes> = OnceLock::new();
    static TRACES: OnceLock<Bytes> = OnceLock::new();

    match signal {
        SignalType::Logs => LOGS
            .get_or_init(encode_response::<ExportLogsServiceResponse>)
            .as_ref(),
        SignalType::Metrics => METRICS
            .get_or_init(encode_response::<ExportMetricsServiceResponse>)
            .as_ref(),
        SignalType::Traces => TRACES
            .get_or_init(encode_response::<ExportTraceServiceResponse>)
            .as_ref(),
    }
}

fn pipeline_send_status<E: Display>(err: E) -> Status {
    Status::internal(format!("Failed to send to pipeline: {err}"))
}

/// Converts a pipeline [`NackMsg`] into a [`tonic::Status`] for the gRPC
/// response.
///
/// The status code is chosen by [`classify_nack`]: permanent client rejections
/// map to `INVALID_ARGUMENT`, other permanent failures to `INTERNAL`, and
/// transient failures to `UNAVAILABLE`, following the OTLP gRPC status code
/// conventions defined in
/// <https://opentelemetry.io/docs/specs/otlp/#otlpgrpc-response>.
fn nack_to_status(nack: NackMsg<OtapPdata>) -> Status {
    let message = format!("Pipeline processing failed: {}", nack.reason);
    classify_nack(nack.permanent, nack.cause).to_tonic_status(message)
}

fn response_channel_closed_status() -> Status {
    Status::internal("Response channel closed unexpectedly")
}

/// Tonic `Codec` implementation that returns the bytes of the serialized message
/// Custom tonic codec that keeps OTLP request bodies as raw bytes and writes minimal responses.
#[derive(Clone)]
struct GrpcRateLimitContext {
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    rate_limiter: SharedAdmissionGate,
}

struct OtlpBytesCodec {
    /// Which OTLP signal this service handles.
    signal: SignalType,
    /// Whether to pre-reserve a context frame (when wait_for_result is on).
    preallocate_frame: bool,
}

impl OtlpBytesCodec {
    fn new(signal: SignalType, preallocate_frame: bool) -> Self {
        Self {
            signal,
            preallocate_frame,
        }
    }
}

impl Codec for OtlpBytesCodec {
    type Decode = OtapPdata;
    type Encode = ();

    type Encoder = OtlpResponseEncoder;
    type Decoder = OtlpBytesDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        OtlpResponseEncoder::new(self.signal)
    }

    fn decoder(&mut self) -> Self::Decoder {
        OtlpBytesDecoder::new(self.signal, self.preallocate_frame)
    }
}

/// Tonic codec `Encoder` implementation that encodes protobuf serialized otlp service responses
struct OtlpResponseEncoder {
    signal: SignalType,
}

impl OtlpResponseEncoder {
    const fn new(signal: SignalType) -> Self {
        Self { signal }
    }
}

impl Encoder for OtlpResponseEncoder {
    type Error = Status;
    type Item = ();

    fn encode(&mut self, _item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        // Reuse precomputed protobuf responses to avoid per-request prost encoding
        // and heap allocations.
        let bytes = precomputed_response(self.signal);
        dst.put_slice(bytes);
        Ok(())
    }
}

/// Tonic codec `Decoder` implementation that decodes OtapBatch from protobuf request bytes
struct OtlpBytesDecoder {
    signal: SignalType,
    preallocate_frame: bool,
}

impl OtlpBytesDecoder {
    fn new(signal: SignalType, preallocate_frame: bool) -> Self {
        Self {
            signal,
            preallocate_frame,
        }
    }
}

impl Decoder for OtlpBytesDecoder {
    type Item = OtapPdata;

    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.remaining();
        // Use copy_to_bytes so accepted requests copy once while advancing the buffer.
        let bytes = src.copy_to_bytes(len);
        let result = match self.signal {
            SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(bytes),
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(bytes),
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(bytes),
        };
        let context = if self.preallocate_frame {
            // Pre-reserve a single frame since wait_for_result uses one slot.
            Context::with_capacity(1)
        } else {
            Context::default()
        };
        Ok(Some(OtapPdata::new(context, result.into())))
    }
}

/// Returns a new gRPC service with OTLP bytes codec for the
/// appropriate signal.  Note! This is an inexpensive call, called for
/// each request instead of a Clone + Sync + Send trait binding that
/// would require Arc<Mutex<_>>.
fn new_grpc(signal: SignalType, settings: OtlpServerSettings) -> Grpc<OtlpBytesCodec> {
    let codec = OtlpBytesCodec::new(signal, settings.wait_for_result);
    let mut grpc = Grpc::new(codec);
    if let Some(limit) = settings.max_decoding_message_size {
        grpc = grpc.max_decoding_message_size(limit);
    }
    grpc.apply_compression_config(
        settings.request_compression_encodings,
        settings.response_compression_encodings,
    )
}

/// Tonic service handler for decoded requests of the appropriate
/// signal.  Like new_grpc, these are inexpensive to create and do
/// not require Arc<Mutex<_>>.
struct OtapBatchService {
    effect_handler: Option<EffectHandler<OtapPdata>>,
    state: Option<AckSlot>,
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    signal: SignalType,
    rate_limit: Option<GrpcRateLimitContext>,
}

impl OtapBatchService {
    const fn new(
        effect_handler: EffectHandler<OtapPdata>,
        state: Option<AckSlot>,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        signal: SignalType,
        rate_limit: Option<GrpcRateLimitContext>,
    ) -> Self {
        Self {
            effect_handler: Some(effect_handler),
            state,
            metrics,
            signal,
            rate_limit,
        }
    }
}

/// Records request completion when the gRPC future returns or is cancelled.
struct RequestCompletionGuard {
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    signal: SignalType,
}

impl Drop for RequestCompletionGuard {
    fn drop(&mut self) {
        self.metrics
            .lock()
            .record_request_completed(self.signal, OtlpProtocol::Grpc);
    }
}

/// Guard mechanism for cancelling a slot when Tonic timeout
/// drops the future.
pub(crate) struct SlotGuard {
    pub(crate) key: SlotKey,
    pub(crate) state: AckSlot,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.state.cancel_slot(self.key);
    }
}

fn required_rate_limit_payload_bytes(size: Option<usize>) -> Result<u64, Status> {
    let size = size.ok_or_else(|| {
        Status::internal("OTLP gRPC payload does not expose its encoded byte size")
    })?;
    u64::try_from(size)
        .map_err(|_| Status::internal("OTLP gRPC payload byte size exceeds the supported range"))
}

impl UnaryService<OtapPdata> for OtapBatchService {
    type Response = ();
    type Future = BoxFuture<'static, Result<tonic::Response<Self::Response>, Status>>;

    fn call(&mut self, request: tonic::Request<OtapPdata>) -> Self::Future {
        let (metadata, extensions, mut otap_batch) = request.into_parts();
        let payload_size = otap_batch.num_bytes();

        // Keep the final weighted admission decision at the request-service
        // boundary, where both transport metadata and the raw payload weight
        // are available. Tonic has already assembled and decompressed the frame;
        // this placement adds only the byte-buffer handoff and OtapPdata wrapper
        // before rejection. Do not move this into the decoder: request metadata
        // is unavailable there, which would block future tenant-key resolution.
        if let Some(rate_limit) = &self.rate_limit {
            let payload_bytes = match required_rate_limit_payload_bytes(payload_size) {
                Ok(payload_bytes) => payload_bytes,
                Err(status) => return Box::pin(std::future::ready(Err(status))),
            };
            match rate_limit
                .rate_limiter
                .admit(payload_bytes, AdmissionContext::for_signal(self.signal))
            {
                AdmissionDecision::Admit => {}
                AdmissionDecision::WouldThrottle => {}
                AdmissionDecision::Throttle { retry_after_secs } => {
                    rate_limit.metrics.lock().record_rejection(
                        OtlpProtocol::Grpc,
                        ReceiverRejectionErrorType::RateLimit,
                    );
                    return Box::pin(std::future::ready(Err(grpc_rate_limit_status(
                        retry_after_secs,
                    ))));
                }
                AdmissionDecision::Oversized => {
                    rate_limit.metrics.lock().record_rejection(
                        OtlpProtocol::Grpc,
                        ReceiverRejectionErrorType::RateLimit,
                    );
                    return Box::pin(std::future::ready(Err(
                        grpc_rate_limit_burst_exceeded_status(),
                    )));
                }
            }
        }

        // Payload size is required only by byte admission. When admission is
        // disabled, missing optional size telemetry must never reject traffic.
        let payload_bytes = payload_size.and_then(|size| u64::try_from(size).ok());

        // Propagate the receiver-observed peer address so downstream processors
        // (e.g. k8sattributes) can correlate telemetry with the originating socket.
        if let Some(addr) = peer_addr_from_extensions(&extensions) {
            otap_batch.set_peer_addr(addr);
        }

        let effect_handler = self
            .effect_handler
            .take()
            .expect("`OtapBatchService` is not reused for multiple calls");

        // Capture transport headers synchronously before moving the effect handler
        // into the async block, avoiding a clone of the capture policy.
        if let Some(policy) = effect_handler.capture_policy() {
            let mut transport_headers = TransportHeaders::new();

            // Collect all metadata pairs, decoding binary values so we store
            // raw bytes rather than the base64 wire encoding (which would be
            // double-encoded on downstream gRPC propagation).
            let pairs: Vec<(&str, Vec<u8>)> = metadata
                .iter()
                .filter_map(|kv| match kv {
                    tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                        Some((key.as_str(), value.as_bytes().to_vec()))
                    }
                    tonic::metadata::KeyAndValueRef::Binary(key, value) => value
                        .to_bytes()
                        .ok()
                        .map(|decoded| (key.as_str(), decoded.to_vec())),
                })
                .collect();

            let _stats = policy.capture_from_pairs(
                pairs.iter().map(|(k, v)| (*k, v.as_slice())),
                &mut transport_headers,
            );
            if !transport_headers.is_empty() {
                otap_batch.set_transport_headers(transport_headers);
            }
        }

        let state = self.state.clone();
        let metrics = self.metrics.clone();
        let signal = self.signal;
        Box::pin(async move {
            let cancel_rx = if let Some(state) = state {
                let (key, rx) = match state.allocate_slot() {
                    None => {
                        metrics.lock().record_rejection(
                            OtlpProtocol::Grpc,
                            ReceiverRejectionErrorType::ConcurrencyLimit,
                        );
                        return Err(Status::resource_exhausted("Too many concurrent requests"));
                    }
                    Some(pair) => pair,
                };

                // Enter the subscription. Slot key becomes calldata.
                effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    key.into(),
                    &mut otap_batch,
                );
                Some((SlotGuard { key, state }, rx))
            } else {
                None
            };

            metrics
                .lock()
                .record_request_admitted(signal, OtlpProtocol::Grpc, payload_bytes);
            let _completion_guard = RequestCompletionGuard {
                metrics: metrics.clone(),
                signal,
            };

            // Send and wait for Ack/Nack
            match effect_handler
                .send_message_with_source_node(otap_batch)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    return Err(pipeline_send_status(e));
                }
            };

            // If backpressure, await a response. The guard will cancel and return the
            // slot if Tonic times-out this task.
            if let Some((_cancel_guard, rx)) = cancel_rx {
                match rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(nack)) => {
                        return Err(nack_to_status(nack));
                    }
                    Err(_) => {
                        return Err(response_channel_closed_status());
                    }
                }
            }

            Ok(tonic::Response::new(()))
        })
    }
}

/// generate a response for a path the grpc server does not know about
fn unimplemented_resp() -> Response<Body> {
    let mut response = Response::new(Body::default());
    let headers = response.headers_mut();
    _ = headers.insert(Status::GRPC_STATUS, (Code::Unimplemented as i32).into());
    _ = headers.insert(
        http::header::CONTENT_TYPE,
        tonic::metadata::GRPC_CONTENT_TYPE,
    );
    response
}

async fn authorize_request(
    authorizer: &dyn BearerTokenAuthorizer,
    metrics: &Arc<Mutex<OtlpReceiverMetrics>>,
    headers: &http::HeaderMap,
    timeout: std::time::Duration,
) -> Result<AuthorizedIdentity, AuthorizationRejection> {
    let result = authorize_bearer(authorizer, headers, Some(timeout)).await;
    if let Err(rejection) = &result {
        metrics
            .lock()
            .record_rejection(OtlpProtocol::Grpc, (*rejection).error_type());
    }
    result
}

fn authorization_status(rejection: AuthorizationRejection) -> Status {
    match rejection {
        AuthorizationRejection::Unauthenticated => Status::unauthenticated(rejection.message()),
        AuthorizationRejection::PermissionDenied => Status::permission_denied(rejection.message()),
        AuthorizationRejection::Unavailable => Status::unavailable(rejection.message()),
    }
}

/// Applies bearer authorization before a gRPC request reaches an OTLP service.
#[derive(Clone)]
pub struct AuthorizationLayer {
    authorizer: Arc<dyn BearerTokenAuthorizer>,
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    timeout: std::time::Duration,
}

impl AuthorizationLayer {
    /// Creates an authorization layer from a bound authorizer capability.
    #[must_use]
    pub fn new(
        authorizer: Arc<dyn BearerTokenAuthorizer>,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            authorizer,
            metrics,
            timeout,
        }
    }
}

impl<S> Layer<S> for AuthorizationLayer {
    type Service = AuthorizationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthorizationService {
            inner,
            authorizer: self.authorizer.clone(),
            metrics: self.metrics.clone(),
            timeout: self.timeout,
        }
    }
}

/// gRPC service that authorizes every request before dispatching it.
#[derive(Clone)]
pub struct AuthorizationService<S> {
    inner: S,
    authorizer: Arc<dyn BearerTokenAuthorizer>,
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    timeout: std::time::Duration,
}

impl<S> Service<Request<Body>> for AuthorizationService<S>
where
    S: Service<Request<Body>, Response = Response<Body>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = mem::replace(&mut self.inner, clone);
        let authorizer = self.authorizer.clone();
        let metrics = self.metrics.clone();
        let timeout = self.timeout;

        Box::pin(async move {
            let _authorized_identity = match authorize_request(
                authorizer.as_ref(),
                &metrics,
                req.headers(),
                timeout,
            )
            .await
            {
                Ok(identity) => identity,
                Err(rejection) => return Ok(authorization_status(rejection).into_http()),
            };
            inner.call(req).await
        })
    }
}

/// common server functionality
#[derive(Clone)]
pub struct ServerCommon {
    effect_handler: EffectHandler<OtapPdata>,
    state: Option<AckSlot>,
    settings: OtlpServerSettings,
    metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    rate_limiter: Option<SharedAdmissionGate>,
}

impl ServerCommon {
    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<AckSlot> {
        self.state.clone()
    }

    fn new(
        effect_handler: EffectHandler<OtapPdata>,
        settings: &OtlpServerSettings,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        rate_limiter: Option<SharedAdmissionGate>,
        state: Option<AckSlot>,
    ) -> Self {
        Self {
            effect_handler,
            state,
            settings: settings.clone(),
            metrics,
            rate_limiter,
        }
    }

    fn exhausted_rate_limit_response(&self) -> Option<Response<Body>> {
        let rate_limiter = self.rate_limiter.as_ref()?;
        if !rate_limiter.refuse_if_instance_saturated() {
            return None;
        }

        self.metrics
            .lock()
            .record_rejection(OtlpProtocol::Grpc, ReceiverRejectionErrorType::RateLimit);
        Some(grpc_rate_limit_saturated_status().into_http())
    }

    fn grpc_rate_limit_context(&self) -> Option<GrpcRateLimitContext> {
        self.rate_limiter
            .clone()
            .map(|rate_limiter| GrpcRateLimitContext {
                metrics: self.metrics.clone(),
                rate_limiter,
            })
    }
}

/// implementation of OTLP bytes -> OTAP GRPC server for logs
#[derive(Clone)]
pub struct LogsServiceServer {
    /// common support for OTLP servers
    pub common: ServerCommon,
}

impl LogsServiceServer {
    /// create a new instance of `LogsServiceServer`
    #[must_use]
    pub fn new(
        effect_handler: EffectHandler<OtapPdata>,
        settings: &OtlpServerSettings,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        rate_limiter: Option<SharedAdmissionGate>,
        state: Option<AckSlot>,
    ) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings, metrics, rate_limiter, state),
        }
    }
}

impl Service<Request<Body>> for LogsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::LOGS_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                // The outer layer avoids entering this service when exhaustion
                // is already visible. Re-check here because another service
                // clone can charge the shared bucket between poll_ready and call.
                if let Some(response) = common.exhausted_rate_limit_response() {
                    return Box::pin(async move { Ok(response) });
                }
                let mut grpc = new_grpc(SignalType::Logs, common.settings.clone());
                let rate_limit = common.grpc_rate_limit_context();
                let service = OtapBatchService::new(
                    common.effect_handler,
                    common.state,
                    common.metrics.clone(),
                    SignalType::Logs,
                    rate_limit,
                );
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }
}

impl NamedService for LogsServiceServer {
    const NAME: &'static str = super::LOGS_SERVICE_NAME;
}

/// implementation of OTLP bytes -> OTAP Pdata GRPC server for metrics
#[derive(Clone)]
pub struct MetricsServiceServer {
    /// common support for OTLP servers
    pub common: ServerCommon,
}

impl MetricsServiceServer {
    /// create a new instance of `MetricsServiceServer`
    #[must_use]
    pub fn new(
        effect_handler: EffectHandler<OtapPdata>,
        settings: &OtlpServerSettings,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        rate_limiter: Option<SharedAdmissionGate>,
        state: Option<AckSlot>,
    ) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings, metrics, rate_limiter, state),
        }
    }
}

impl Service<Request<Body>> for MetricsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::METRICS_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                if let Some(response) = common.exhausted_rate_limit_response() {
                    return Box::pin(async move { Ok(response) });
                }
                let mut grpc = new_grpc(SignalType::Metrics, common.settings.clone());
                let rate_limit = common.grpc_rate_limit_context();
                let service = OtapBatchService::new(
                    common.effect_handler,
                    common.state,
                    common.metrics.clone(),
                    SignalType::Metrics,
                    rate_limit,
                );
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }
}

impl NamedService for MetricsServiceServer {
    const NAME: &'static str = super::METRICS_SERVICE_NAME;
}

/// implementation of OTLP bytes -> OTAP GRPC server for traces
#[derive(Clone)]
pub struct TraceServiceServer {
    /// common support for OTLP servers
    pub common: ServerCommon,
}

impl TraceServiceServer {
    /// create a new instance of `TracesServiceServer`
    #[must_use]
    pub fn new(
        effect_handler: EffectHandler<OtapPdata>,
        settings: &OtlpServerSettings,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
        rate_limiter: Option<SharedAdmissionGate>,
        state: Option<AckSlot>,
    ) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings, metrics, rate_limiter, state),
        }
    }
}

impl Service<Request<Body>> for TraceServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::TRACE_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                if let Some(response) = common.exhausted_rate_limit_response() {
                    return Box::pin(async move { Ok(response) });
                }
                let mut grpc = new_grpc(SignalType::Traces, common.settings.clone());
                let rate_limit = common.grpc_rate_limit_context();
                let service = OtapBatchService::new(
                    common.effect_handler,
                    common.state,
                    common.metrics.clone(),
                    SignalType::Traces,
                    rate_limit,
                );
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }
}

impl NamedService for TraceServiceServer {
    const NAME: &'static str = super::TRACE_SERVICE_NAME;
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as BearerTokenAuthorizerCapability;
    use otel_arrow_dfe_engine::capability::auth::{AuthzDecision, BearerToken, DenyReason};
    use otel_arrow_dfe_engine::capability::{CapabilityError, CapabilityErrorSource};
    use otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel;
    use otel_arrow_dfe_engine::shared::message::SharedSender;
    use otel_arrow_dfe_engine::testing::test_node;
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use std::collections::HashMap;
    use tokio::sync::mpsc as tokio_mpsc;
    use tonic::Code;

    const TEST_AUTHORIZATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

    struct TestAuthorizer;

    #[async_trait::async_trait]
    impl BearerTokenAuthorizer for TestAuthorizer {
        async fn authorize(
            &self,
            credential: &BearerToken,
        ) -> Result<AuthzDecision, CapabilityError> {
            Ok(match credential.expose_token() {
                "allowed" => AuthzDecision::allow_anonymous(),
                "invalid" => AuthzDecision::deny(DenyReason::InvalidCredential),
                _ => AuthzDecision::deny(DenyReason::NotPermitted),
            })
        }
    }

    /// An authorizer that cannot reach its backing identity service, so it
    /// reaches no decision at all.
    struct FailingAuthorizer;

    #[async_trait::async_trait]
    impl BearerTokenAuthorizer for FailingAuthorizer {
        async fn authorize(
            &self,
            _credential: &BearerToken,
        ) -> Result<AuthzDecision, CapabilityError> {
            Err(
                CapabilityErrorSource::<BearerTokenAuthorizerCapability>::new("test-ext".into())
                    .error("token review backend unreachable"),
            )
        }
    }

    struct PendingAuthorizer;

    #[async_trait::async_trait]
    impl BearerTokenAuthorizer for PendingAuthorizer {
        async fn authorize(
            &self,
            _credential: &BearerToken,
        ) -> Result<AuthzDecision, CapabilityError> {
            std::future::pending().await
        }
    }

    fn new_test_metrics() -> Arc<Mutex<OtlpReceiverMetrics>> {
        let registry = TelemetryRegistryHandle::new();
        let controller = otel_arrow_dfe_engine::context::ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        Arc::new(Mutex::new(OtlpReceiverMetrics::register(&pipeline_ctx)))
    }

    /// Scenario: gRPC authorization receives missing, non-bearer, allowed,
    /// invalid, and policy-denied credentials.
    /// Guarantees: Requests are admitted only on allow; authentication and
    /// policy failures map to the expected gRPC status codes.
    #[tokio::test]
    async fn maps_authorization_outcomes() {
        let authorizer = TestAuthorizer;
        let metrics = new_test_metrics();
        let mut headers = http::HeaderMap::new();

        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("missing credential must be rejected");
        assert_eq!(authorization_status(response).code(), Code::Unauthenticated);

        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Basic dXNlcjpwYXNz"),
        );
        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("non-bearer credential must be rejected");
        assert_eq!(authorization_status(response).code(), Code::Unauthenticated);

        _ = headers.remove(http::header::AUTHORIZATION);
        _ = headers.append(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer allowed"),
        );
        _ = headers.append(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer allowed"),
        );
        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("duplicate credentials must be rejected");
        assert_eq!(authorization_status(response).code(), Code::Unauthenticated);

        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer allowed"),
        );
        assert!(
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT,)
                .await
                .is_ok()
        );

        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer invalid"),
        );
        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("invalid credential must be rejected");
        assert_eq!(authorization_status(response).code(), Code::Unauthenticated);

        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer denied"),
        );
        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("policy-denied credential must be rejected");
        assert_eq!(
            authorization_status(response).code(),
            Code::PermissionDenied
        );

        let metrics = metrics.lock();
        assert_eq!(
            metrics
                .rejections_for(
                    OtlpProtocol::Grpc,
                    ReceiverRejectionErrorType::Authentication,
                )
                .requests
                .get(),
            4
        );
        assert_eq!(
            metrics
                .rejections_for(
                    OtlpProtocol::Grpc,
                    ReceiverRejectionErrorType::PermissionDenied,
                )
                .requests
                .get(),
            1
        );
    }

    /// Scenario: gRPC authorization is attempted while the authorizer cannot
    /// reach its backing identity service and returns an error rather than a
    /// decision.
    /// Guarantees: An undetermined authorization fails closed with gRPC
    /// UNAVAILABLE (14) rather than admitting the request.
    #[tokio::test]
    async fn undetermined_authorization_fails_closed() {
        let authorizer = FailingAuthorizer;
        let metrics = new_test_metrics();
        let mut headers = http::HeaderMap::new();
        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer allowed"),
        );

        let response =
            authorize_request(&authorizer, &metrics, &headers, TEST_AUTHORIZATION_TIMEOUT)
                .await
                .expect_err("an undetermined decision must not admit the request");
        assert_eq!(authorization_status(response).code(), Code::Unavailable);
        assert_eq!(
            metrics
                .lock()
                .rejections_for(
                    OtlpProtocol::Grpc,
                    ReceiverRejectionErrorType::AuthorizationUnavailable,
                )
                .requests
                .get(),
            1
        );
    }

    /// Scenario: A bearer authorizer does not complete within the receiver's
    /// authorization deadline.
    /// Guarantees: The receiver cancels the capability call, returns gRPC
    /// UNAVAILABLE, and records an authorization-unavailable rejection.
    #[tokio::test]
    async fn authorization_timeout_fails_closed() {
        let authorizer = PendingAuthorizer;
        let metrics = new_test_metrics();
        let mut headers = http::HeaderMap::new();
        _ = headers.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Bearer pending"),
        );

        let response = authorize_request(
            &authorizer,
            &metrics,
            &headers,
            std::time::Duration::from_millis(10),
        )
        .await
        .expect_err("a timed-out decision must not admit the request");
        assert_eq!(authorization_status(response).code(), Code::Unavailable);
        assert_eq!(
            metrics
                .lock()
                .rejections_for(
                    OtlpProtocol::Grpc,
                    ReceiverRejectionErrorType::AuthorizationUnavailable,
                )
                .requests
                .get(),
            1
        );
    }

    fn new_test_service(
        state: Option<AckSlot>,
        metrics: Arc<Mutex<OtlpReceiverMetrics>>,
    ) -> (OtapBatchService, tokio_mpsc::Receiver<OtapPdata>) {
        let (msg_tx, msg_rx) = tokio_mpsc::channel(1);
        let mut senders = HashMap::new();
        let _ = senders.insert("default".into(), SharedSender::mpsc(msg_tx));
        let (ctrl_tx, _ctrl_rx) = runtime_ctrl_msg_channel(1);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let effect_handler = EffectHandler::new(
            test_node("grpc_admission_metrics"),
            senders,
            None,
            ctrl_tx,
            metrics_reporter,
        );
        (
            OtapBatchService::new(effect_handler, state, metrics, SignalType::Logs, None),
            msg_rx,
        )
    }

    fn make_nack(permanent: bool) -> NackMsg<OtapPdata> {
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into());
        if permanent {
            NackMsg::new_permanent("permanent failure", pdata)
        } else {
            NackMsg::new("transient failure", pdata)
        }
    }

    /// Scenario: a future payload variant has no encoded byte size.
    /// Guarantees: byte admission fails closed, while best-effort payload telemetry represents
    /// the unknown measurement as absent instead of recording a real zero-byte measurement.
    #[test]
    fn grpc_rate_weight_is_required_only_when_admission_is_enabled() {
        let best_effort_telemetry = None::<usize>.and_then(|size| u64::try_from(size).ok());
        assert_eq!(best_effort_telemetry, None);
        let status = required_rate_limit_payload_bytes(None)
            .expect_err("enabled byte admission must fail closed");

        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("does not expose"));
    }

    /// Scenario: a permanent downstream NACK is converted to a gRPC status.
    /// Guarantees: the client receives `INTERNAL` with both the generic pipeline
    /// failure context and the specific permanent failure reason.
    #[test]
    fn test_nack_to_status_permanent_returns_internal() {
        let nack = make_nack(true);
        let status = nack_to_status(nack);
        assert_eq!(status.code(), Code::Internal);
        assert!(
            status.message().contains("Pipeline processing failed"),
            "message: {}",
            status.message()
        );
        assert!(
            status.message().contains("permanent failure"),
            "message: {}",
            status.message()
        );
    }

    /// Scenario: a transient downstream NACK is converted to a gRPC status.
    /// Guarantees: the client receives `UNAVAILABLE` with both the generic pipeline
    /// failure context and the retryable failure reason.
    #[test]
    fn test_nack_to_status_transient_returns_unavailable() {
        let nack = make_nack(false);
        let status = nack_to_status(nack);
        assert_eq!(status.code(), Code::Unavailable);
        assert!(
            status.message().contains("Pipeline processing failed"),
            "message: {}",
            status.message()
        );
        assert!(
            status.message().contains("transient failure"),
            "message: {}",
            status.message()
        );
    }

    /// Scenario: a permanent NACK classified as a client refusal is converted
    /// to a gRPC status.
    /// Guarantees: the client receives non-retryable `INVALID_ARGUMENT` rather
    /// than the server-fault `INTERNAL` used for other permanent failures.
    #[test]
    fn test_nack_to_status_refused_returns_invalid_argument() {
        use otel_arrow_dfe_engine::control::NackCause;
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into());
        let nack = NackMsg::new_permanent_with_cause("bad request", pdata, NackCause::Refused);
        let status = nack_to_status(nack);
        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(
            status.message().contains("bad request"),
            "message: {}",
            status.message()
        );
    }

    /// Scenario: A non-empty gRPC request does not require an acknowledgement slot.
    /// Guarantees: Successful admission records its started, completed, and payload-byte values.
    #[tokio::test]
    async fn admitted_grpc_request_records_payload_bytes() {
        let metrics = new_test_metrics();
        let (mut service, mut msg_rx) = new_test_service(None, metrics.clone());
        let payload = Bytes::from_static(b"grpc-payload");
        let payload_bytes = payload.len() as u64;
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(payload).into());

        let result = UnaryService::call(&mut service, tonic::Request::new(pdata)).await;

        assert!(result.is_ok());
        let _ = msg_rx.recv().await.expect("request forwarded downstream");
        let metrics = metrics.lock();
        let requests = metrics.requests_for(SignalType::Logs, OtlpProtocol::Grpc);
        assert_eq!(requests.started.get(), 1);
        assert_eq!(requests.completed.get(), 1);
        assert_eq!(requests.payload_size.get(), payload_bytes);
    }

    /// Scenario: A non-empty gRPC request cannot allocate its acknowledgement slot.
    /// Guarantees: The request is rejected without recording admission, completion, or payload bytes.
    #[tokio::test]
    async fn rejected_grpc_request_does_not_record_payload_bytes() {
        let metrics = new_test_metrics();
        let (mut service, mut msg_rx) = new_test_service(Some(AckSlot::new(0)), metrics.clone());
        let payload = Bytes::from_static(b"grpc-rejected-payload");
        let pdata = OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(payload).into());

        let result = UnaryService::call(&mut service, tonic::Request::new(pdata)).await;

        assert_eq!(
            result.expect_err("request rejected").code(),
            Code::ResourceExhausted
        );
        assert!(msg_rx.try_recv().is_err());
        let metrics = metrics.lock();
        let requests = metrics.requests_for(SignalType::Logs, OtlpProtocol::Grpc);
        assert_eq!(requests.started.get(), 0);
        assert_eq!(requests.completed.get(), 0);
        assert_eq!(requests.payload_size.get(), 0);
        assert_eq!(
            metrics
                .rejections_for(
                    OtlpProtocol::Grpc,
                    ReceiverRejectionErrorType::ConcurrencyLimit,
                )
                .requests
                .get(),
            1
        );
    }
}
