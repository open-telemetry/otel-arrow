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

use crate::accessory::slots::{Config as SlotsConfig, SlotKey, State as SlotsState};
use crate::pdata::{OtapPdata, OtlpProtoBytes};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_engine::control::{AckMsg, CallData, Context8u8, NackMsg};
use otap_df_engine::shared::receiver::EffectHandler;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use prost::Message;
use prost::bytes::Buf;
use smallvec::smallvec;
use tokio::sync::oneshot;
use tonic::Status;
use tonic::body::Body;
use tonic::codec::{
    Codec, CompressionEncoding, DecodeBuf, Decoder, EnabledCompressionEncodings, EncodeBuf, Encoder,
};
use tonic::server::{Grpc, NamedService, UnaryService};

/// Shared state for correlating requests with responses using slots
pub type SharedCorrelationState =
    Arc<Mutex<SlotsState<oneshot::Sender<Result<(), NackMsg<OtapPdata>>>>>>;

/// Route an Ack response back to the appropriate correlation slot
pub fn route_ack_response(state: &SharedCorrelationState, ack: AckMsg<OtapPdata>) {
    route_response(state, ack.calldata, Ok(()));
}

/// Route a Nack response back to the appropriate correlation slot
pub fn route_nack_response(state: &SharedCorrelationState, mut nack: NackMsg<OtapPdata>) {
    let calldata = std::mem::take(&mut nack.calldata);
    route_response(state, calldata, Err(nack));
}

/// Internal helper to route responses to slots
fn route_response(
    state: &SharedCorrelationState,
    calldata: CallData,
    result: Result<(), NackMsg<OtapPdata>>,
) {
    // Decode slot key from calldata
    if calldata.len() != 2 {
        return; // Invalid calldata format
    }

    let slot_index: usize = calldata[0].try_into().unwrap_or(0);
    let slot_generation: usize = calldata[1].try_into().unwrap_or(0);
    let slot_key = SlotKey::new(
        crate::accessory::slots::SlotIndex::from_usize(slot_index),
        crate::accessory::slots::SlotGeneration::from_usize(slot_generation),
    );

    // Try to get the channel from the slot
    if let Ok(mut state) = state.lock() {
        if let Some(sender) = state.get_if_current(slot_key) {
            // Send the result back to the waiting gRPC handler
            let _ = sender.send(result);
        }
    }
}

/// identifier
#[derive(Clone)]
enum Signal {
    Logs,
    Metrics,
    Traces,
}

/// Tonic `Codec` implementation that returns the bytes of the serialized message
struct OtlpBytesCodec {
    signal: Signal,
}

impl OtlpBytesCodec {
    fn new(signal: Signal) -> Self {
        Self { signal }
    }
}

impl Codec for OtlpBytesCodec {
    type Decode = OtapPdata;
    type Encode = ();

    type Encoder = OtlpResponseEncoder;
    type Decoder = OtapBatchDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        OtlpResponseEncoder::new(self.signal.clone())
    }

    fn decoder(&mut self) -> Self::Decoder {
        OtapBatchDecoder::new(self.signal.clone())
    }
}

/// Tonic codec `Encoder` implementation that encodes protobuf serialized otlp service responses
struct OtlpResponseEncoder {
    signal: Signal,
}

impl OtlpResponseEncoder {
    fn new(signal: Signal) -> Self {
        Self { signal }
    }
}

impl Encoder for OtlpResponseEncoder {
    type Error = Status;
    type Item = ();

    fn encode(&mut self, _item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        match self.signal {
            Signal::Logs => {
                let response = ExportLogsServiceResponse {
                    partial_success: None,
                };
                response.encode(dst)
            }
            Signal::Metrics => {
                let response = ExportMetricsServiceResponse {
                    partial_success: None,
                };
                response.encode(dst)
            }
            Signal::Traces => {
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
struct OtapBatchDecoder {
    signal: Signal,
}

impl OtapBatchDecoder {
    fn new(signal: Signal) -> Self {
        Self { signal }
    }
}

impl Decoder for OtapBatchDecoder {
    type Item = OtapPdata;

    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let buf = src.chunk();
        let result = match self.signal {
            Signal::Logs => OtlpProtoBytes::ExportLogsRequest(buf.to_vec()),
            Signal::Metrics => OtlpProtoBytes::ExportMetricsRequest(buf.to_vec()),
            Signal::Traces => OtlpProtoBytes::ExportTracesRequest(buf.to_vec()),
        };
        src.advance(buf.len());
        Ok(Some(OtapPdata::new_todo_context(result.into())))
    }
}

/// implementation of tonic service that handles the decoded request (the OtapBatch).
struct OtapBatchService {
    effect_handler: EffectHandler<OtapPdata>,
    state: SharedCorrelationState,
}

impl OtapBatchService {
    fn new(effect_handler: EffectHandler<OtapPdata>, state: SharedCorrelationState) -> Self {
        Self {
            effect_handler,
            state,
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
            // Create oneshot channel for response
            let (tx, rx) = oneshot::channel();

            // Allocate a slot for this request
            let slot_key = {
                let mut state = state.lock().unwrap();
                match state.allocate_slot(tx) {
                    Some(key) => key,
                    None => {
                        return Err(Status::resource_exhausted("Too many concurrent requests"));
                    }
                }
            };

            // Create guard to clean up slot if this future is cancelled/dropped
            struct SlotGuard {
                slot_key: SlotKey,
                state: SharedCorrelationState,
            }

            impl Drop for SlotGuard {
                fn drop(&mut self) {
                    if let Ok(mut state) = self.state.lock() {
                        state.cancel(self.slot_key);
                    }
                }
            }

            let _guard = SlotGuard {
                slot_key,
                state: state.clone(),
            };

            // Create CallData with slot index and generation
            let call_data: CallData = smallvec![
                Context8u8::from(slot_key.index().as_usize()),
                Context8u8::from(slot_key.generation().as_usize()),
            ];

            // Subscribe to Ack/Nack responses
            effect_handler.subscribe_to(
                Interests::ACKS | Interests::NACKS,
                call_data,
                &mut otap_batch,
            );

            // Send the message to the pipeline
            match effect_handler.send_message(otap_batch).await {
                Ok(_) => {
                    // Wait for Ack or Nack response
                    match rx.await {
                        Ok(Ok(())) => Ok(tonic::Response::new(())),
                        Ok(Err(nack)) => Err(Status::internal(format!(
                            "Pipeline processing failed: {}",
                            nack.reason
                        ))),
                        Err(_) => Err(Status::internal(
                            "Response channel closed unexpectedly",
                        )),
                    }
                }
                Err(e) => {
                    // Failed to send to pipeline - clean up via guard drop
                    Err(Status::internal(format!("Failed to send to pipeline: {e}")))
                }
            }
        })
    }
}

/// handle the grpc service request
async fn handle_service_request(
    req: Request<Body>,
    signal: Signal,
    effect_handler: EffectHandler<OtapPdata>,
    state: SharedCorrelationState,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
) -> Response<Body> {
    let codec = OtlpBytesCodec::new(signal);
    let mut grpc = Grpc::new(codec)
        .apply_compression_config(accept_compression_encodings, send_compression_encodings);
    grpc.unary(OtapBatchService::new(effect_handler, state), req)
        .await
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

/// implementation of OTLP bytes -> OTAP GRPC server for logs
#[derive(Clone)]
pub struct LogsServiceServer {
    effect_handler: EffectHandler<OtapPdata>,
    state: SharedCorrelationState,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl LogsServiceServer {
    /// create a new instance of `LogsServiceServer`
    #[must_use]
    pub fn new(effect_handler: EffectHandler<OtapPdata>) -> Self {
        let config = SlotsConfig { max_slots: 1000 };
        let state = Arc::new(Mutex::new(SlotsState::new(config)));
        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get the correlation state for routing responses
    #[must_use]
    pub fn state(&self) -> SharedCorrelationState {
        self.state.clone()
    }

    /// compress responses with the given encoding if the client supports it
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.accept_compression_encodings.enable(encoding);
        self
    }

    /// enable decompressing requests with the given encoding
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.send_compression_encodings.enable(encoding);
        self
    }
}

impl tower_service::Service<Request<Body>> for LogsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::LOGS_SERVICE_EXPORT_PATH => {
                let effect_handler = self.effect_handler.clone();
                let state = self.state.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Logs,
                        effect_handler,
                        state,
                        accept_compression_encodings,
                        send_compression_encodings,
                    )
                    .await;
                    Ok(res)
                })
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
    effect_handler: EffectHandler<OtapPdata>,
    state: SharedCorrelationState,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl MetricsServiceServer {
    /// create a new instance of `MetricsServiceServer`
    #[must_use]
    pub fn new(effect_handler: EffectHandler<OtapPdata>) -> Self {
        let config = SlotsConfig { max_slots: 1000 };
        let state = Arc::new(Mutex::new(SlotsState::new(config)));
        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get the correlation state for routing responses
    #[must_use]
    pub fn state(&self) -> SharedCorrelationState {
        self.state.clone()
    }

    /// compress responses with the given encoding if the client supports it
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.accept_compression_encodings.enable(encoding);
        self
    }

    /// enable decompressing requests with the given encoding
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.send_compression_encodings.enable(encoding);
        self
    }
}

impl tower_service::Service<Request<Body>> for MetricsServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::METRICS_SERVICE_EXPORT_PATH => {
                let effect_handler = self.effect_handler.clone();
                let state = self.state.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Metrics,
                        effect_handler,
                        state,
                        accept_compression_encodings,
                        send_compression_encodings,
                    )
                    .await;
                    Ok(res)
                })
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
    effect_handler: EffectHandler<OtapPdata>,
    state: SharedCorrelationState,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl TraceServiceServer {
    /// create a new instance of `TracesServiceServer`
    #[must_use]
    pub fn new(effect_handler: EffectHandler<OtapPdata>) -> Self {
        let config = SlotsConfig { max_slots: 1000 };
        let state = Arc::new(Mutex::new(SlotsState::new(config)));
        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get the correlation state for routing responses
    #[must_use]
    pub fn state(&self) -> SharedCorrelationState {
        self.state.clone()
    }

    /// compress responses with the given encoding if the client supports it
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.accept_compression_encodings.enable(encoding);
        self
    }

    /// enable decompressing requests with the given encoding
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.send_compression_encodings.enable(encoding);
        self
    }
}

impl tower_service::Service<Request<Body>> for TraceServiceServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            super::TRACE_SERVICE_EXPORT_PATH => {
                let effect_handler = self.effect_handler.clone();
                let state = self.state.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Traces,
                        effect_handler,
                        state,
                        accept_compression_encodings,
                        send_compression_encodings,
                    )
                    .await;
                    Ok(res)
                })
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
