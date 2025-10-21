// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementations of OTLP grpc service servers that can produce Otap Pdata.
//!
//! The Pdata it produces contain the serialized protobuf messages. This means that we can
//! use these servers to receive telemetry data and deserialize it lazily only if some pipeline
//! requires it

use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::task::Poll;

use crate::accessory::slots::{Key as SlotKey, State as SlotsState};
use crate::pdata::{Context, OtapPdata, OtlpProtoBytes};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_config::experimental::SignalType;
use otap_df_engine::control::{CallData, NackMsg};
use otap_df_engine::shared::receiver::EffectHandler;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use prost::Message;
use prost::bytes::Buf;
use tokio::sync::oneshot;
use tonic::Status;
use tonic::body::Body;
use tonic::codec::{Codec, DecodeBuf, Decoder, EnabledCompressionEncodings, EncodeBuf, Encoder};
use tonic::server::{Grpc, NamedService, UnaryService};

/// Shared state for binding requests with responses.  This map is
/// generally optional depending on wait_for_result: true, we do not
/// create or use the state when ack/nack is not required.
#[derive(Clone)]
pub struct SharedState(Arc<Mutex<SlotsState<oneshot::Sender<Result<(), NackMsg<OtapPdata>>>>>>);

impl SharedState {
    fn new(max_size: usize) -> Self {
        Self(Arc::new(Mutex::new(SlotsState::new(max_size))))
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

impl SharedState {
    /// Internal helper to route responses to slots
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
        let chan = self
            .0
            .lock()
            .map(|mut state| state.take(key))
            .ok()
            .flatten();

        // Try to send.
        if chan.and_then(|sender| sender.send(result).ok()).is_some() {
            RouteResponse::Sent
        } else {
            RouteResponse::Expired
        }
    }
}

/// Common settings for OTLP receivers.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
    /// Request compression allowed
    pub accept_compression_encodings: EnabledCompressionEncodings,
    /// Response compression used
    pub send_compression_encodings: EnabledCompressionEncodings,
}

/// Tonic `Codec` implementation that returns the bytes of the serialized message
struct OtlpBytesCodec {
    signal: SignalType,
}

impl OtlpBytesCodec {
    fn new(signal: SignalType) -> Self {
        Self { signal }
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
        OtlpBytesDecoder::new(self.signal)
    }
}

/// Tonic codec `Encoder` implementation that encodes protobuf serialized otlp service responses
struct OtlpResponseEncoder {
    signal: SignalType,
}

impl OtlpResponseEncoder {
    fn new(signal: SignalType) -> Self {
        Self { signal }
    }
}

impl Encoder for OtlpResponseEncoder {
    type Error = Status;
    type Item = ();

    fn encode(&mut self, _item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        match self.signal {
            SignalType::Logs => {
                let response = ExportLogsServiceResponse {
                    partial_success: None,
                };
                response.encode(dst)
            }
            SignalType::Metrics => {
                let response = ExportMetricsServiceResponse {
                    partial_success: None,
                };
                response.encode(dst)
            }
            SignalType::Traces => {
                let response = ExportTraceServiceResponse {
                    partial_success: None,
                };
                response.encode(dst)
            }
        }
        .map_err(|e| Status::internal(format!("unexpected error encoding response: {e}")))
    }
}

/// Tonic codec `Decoder` implementation that decodes OtapBatch from protobuf request bytes
struct OtlpBytesDecoder {
    signal: SignalType,
}

impl OtlpBytesDecoder {
    fn new(signal: SignalType) -> Self {
        Self { signal }
    }
}

impl Decoder for OtlpBytesDecoder {
    type Item = OtapPdata;

    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let buf = src.chunk();
        let result = match self.signal {
            SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(buf.to_vec()),
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(buf.to_vec()),
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(buf.to_vec()),
        };
        src.advance(buf.len());
        Ok(Some(OtapPdata::new(Context::default(), result.into())))
    }
}

/// Returns a new gRPC service with OTLP bytes codec for the
/// appropriate signal.  Note! This is an inexpensive call, called for
/// each request instead of a Clone + Sync + Send trait binding that
/// would require Arc<Mutex<_>>.
fn new_grpc(signal: SignalType, settings: Settings) -> Grpc<OtlpBytesCodec> {
    let codec = OtlpBytesCodec::new(signal);
    Grpc::new(codec).apply_compression_config(
        settings.accept_compression_encodings,
        settings.send_compression_encodings,
    )
}

/// Tonic service handler for decoded requests of the appropriate
/// signal.  Like new_grpc, these are inexpensive to create and do
/// not require Arc<Mutex<_>>.
struct OtapBatchService {
    effect_handler: EffectHandler<OtapPdata>,
    state: Option<SharedState>,
}

impl OtapBatchService {
    fn new(effect_handler: EffectHandler<OtapPdata>, state: Option<SharedState>) -> Self {
        Self {
            effect_handler,
            state,
        }
    }
}

/// Guard mechanism for cancelling a slot when Tonic timeout
/// drops the future.
struct SlotGuard {
    key: SlotKey,
    state: SharedState,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.state.0.lock() {
            state.cancel(self.key);
        }
    }
}

impl UnaryService<OtapPdata> for OtapBatchService {
    type Response = ();
    type Future = BoxFuture<'static, Result<tonic::Response<Self::Response>, Status>>;

    fn call(&mut self, request: tonic::Request<OtapPdata>) -> Self::Future {
        let mut otap_batch = request.into_inner();

        let effect_handler = self.effect_handler.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let cancel_rx = if let Some(state) = state {
                // Try to allocate a slot (under the mutex) for calldata.
                let (key, rx) = match state
                    .0
                    .lock()
                    .map(|mut state| state.allocate(|| oneshot::channel()))
                {
                    Err(_) => return Err(Status::internal("Mutex poisoned")),
                    Ok(None) => {
                        return Err(Status::resource_exhausted("Too many concurrent requests"));
                    }
                    Ok(Some(pair)) => pair,
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

            // Send and wait for Ack/Nack
            match effect_handler.send_message(otap_batch).await {
                Ok(_) => {}
                Err(e) => {
                    return Err(Status::internal(format!("Failed to send to pipeline: {e}")));
                }
            };

            // If backpressure, await a response. The guard will cancel and return the
            // slot if Tonic times-out this task.
            if let Some((_cancel_guard, rx)) = cancel_rx {
                match rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(nack)) => {
                        // TODO: Use more specific status codes based on nack reason/type
                        // when more detailed error information is available from the pipeline
                        return Err(Status::unavailable(format!(
                            "Pipeline processing failed: {}",
                            nack.reason
                        )));
                    }
                    Err(_) => {
                        return Err(Status::internal("Response channel closed unexpectedly"));
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
    _ = headers.insert(
        Status::GRPC_STATUS,
        (tonic::Code::Unimplemented as i32).into(),
    );
    _ = headers.insert(
        http::header::CONTENT_TYPE,
        tonic::metadata::GRPC_CONTENT_TYPE,
    );
    response
}

/// common server functionality
#[derive(Clone)]
pub struct ServerCommon {
    effect_handler: EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
}

impl ServerCommon {
    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<SharedState> {
        self.state.clone()
    }

    fn new(effect_handler: EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| SharedState::new(settings.max_concurrent_requests)),
            settings: settings.clone(),
        }
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
    pub fn new(effect_handler: EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings),
        }
    }
}

impl tower_service::Service<Request<Body>> for LogsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::LOGS_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                let mut grpc = new_grpc(SignalType::Logs, common.settings);
                let service = OtapBatchService::new(common.effect_handler, common.state);
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
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
    pub fn new(effect_handler: EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings),
        }
    }
}

impl tower_service::Service<Request<Body>> for MetricsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::METRICS_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                let mut grpc = new_grpc(SignalType::Metrics, common.settings);
                let service = OtapBatchService::new(common.effect_handler, common.state);
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
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
    pub fn new(effect_handler: EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            common: ServerCommon::new(effect_handler, settings),
        }
    }
}

impl tower_service::Service<Request<Body>> for TraceServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::TRACE_SERVICE_EXPORT_PATH => {
                let common = self.common.clone();
                let mut grpc = new_grpc(SignalType::Traces, common.settings);
                let service = OtapBatchService::new(common.effect_handler, common.state);
                Box::pin(async move { Ok(grpc.unary(service, req).await) })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl NamedService for TraceServiceServer {
    const NAME: &'static str = super::TRACE_SERVICE_NAME;
}
