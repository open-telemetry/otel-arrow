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

use crate::pdata::{OtapPdata, OtlpProtoBytes};
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use crate::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_config::experimental::SignalType;
use otap_df_engine::control::CtxVal;
use otap_df_engine::{
    AckMsg, CallData, Interests, NackMsg, ProducerEffectHandlerExtension,
    shared::receiver::EffectHandler,
};
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

/// Configuration for servers
pub struct ServerConfig {
    /// Maximum number of slots
    max_slots: usize,
}

/// Data stored in each correlation slot
struct SlotData {
    /// Channel to send response back to gRPC handler
    channel: oneshot::Sender<Result<(), NackMsg<OtapPdata>>>,

    /// Coutner
    generation: SlotGeneration,
}

/// Treated as an index uinto slots
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SlotIndex(usize);

/// A unique value each time the SlotIndex is used
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SlotGeneration(usize);

/// The value placed in CallData
#[derive(Clone, Copy, Debug)]
pub struct SlotKey(SlotIndex, SlotGeneration);

impl SlotIndex {
    fn as_usize(self) -> usize {
        self.0
    }
}

impl SlotGeneration {
    fn as_usize(self) -> usize {
        self.0
    }

    fn increment(self) -> Self {
        SlotGeneration(self.0.wrapping_add(1))
    }
}

impl SlotKey {
    fn new(index: SlotIndex, generation: SlotGeneration) -> Self {
        Self(index, generation)
    }

    fn index(self) -> SlotIndex {
        self.0
    }

    fn generation(self) -> SlotGeneration {
        self.1
    }
}

enum GenMem {
    Current(SlotData),
    Available(SlotGeneration),
}

/// State for correlating gRPC requests with pipeline Ack/Nack responses
pub struct ServerState {
    /// Current slots array, can safely grow because this means we retain
    /// the generation; to support shrink would require not recycling
    /// generation numbers.
    ///
    /// Functionally maps SlotIndex to Option<SlotData>, however uses
    /// default values for all fields to avoid overhead. This is safe
    /// because the oneshot has an inner Option.
    slots: Vec<GenMem>,

    /// Free slots, use to push/pop free SlotIndex values. When the
    /// slots Vec grows we will add all the new SlotIndex values to
    /// this set.
    free_slots: Vec<SlotIndex>,

    /// Server configuration.
    config: ServerConfig,
}

/// Shared correlation state between gRPC handlers and effect handlers
pub type SharedCorrelationState = Arc<Mutex<ServerState>>;

/// Route an Ack/Nack response back to the appropriate correlation slot
pub fn route_response(
    state: &SharedCorrelationState,
    calldata: Option<CallData>,
    result: Result<(), NackMsg<OtapPdata>>,
) {
    let calldata = match calldata {
        Some(data) => data,
        None => {
            return;
        }
    };

    let slot_index = SlotIndex(calldata[0].into());
    let generation = SlotGeneration(calldata[1].into());
    let slot_key = SlotKey::new(slot_index, generation);

    let mut state = state.lock().unwrap();
    state.deliver_response(slot_key, result);
}

/// Route the ack
pub fn route_ack_response(state: &SharedCorrelationState, ack: AckMsg<OtapPdata>) {
    route_response(state, ack.calldata, Ok(()));
}

/// Route a Nack response back to the appropriate correlation slot  
pub fn route_nack_response(state: &SharedCorrelationState, mut nack: NackMsg<OtapPdata>) {
    let calldata = nack.calldata.take();
    route_response(state, calldata, Err(nack))
}

impl ServerConfig {
    /// Create default server configuration
    pub fn default() -> Self {
        Self { max_slots: 1000 }
    }
}

impl ServerState {
    /// Create new correlation state with specified limits
    fn new(config: ServerConfig) -> Self {
        Self {
            slots: Vec::new(),
            free_slots: Vec::new(),
            config,
        }
    }

    /// Deliver response to a specific slot
    pub(crate) fn deliver_response(
        &mut self,
        slot_key: SlotKey,
        result: Result<(), NackMsg<OtapPdata>>,
    ) {
        let slot_index = slot_key.index().as_usize();
        let slice = self.slots.as_mut_slice();
        let slotref = &mut slice[slot_index];

        let mut replace = match slotref {
            GenMem::Available(_) => {
                return;
            }
            GenMem::Current(data) => {
                if slot_key.generation() != data.generation {
                    return;
                }
                GenMem::Available(data.generation.increment())
            }
        };

        std::mem::swap(slotref, &mut replace);

        match replace {
            GenMem::Current(data) => {
                let _ = data.channel.send(result);
            }
            _ => {}
        }
    }

    /// Free a slot when a gRPC request is cancelled
    pub(crate) fn free_slot(&mut self, slot_key: SlotKey) {
        let slot_index = slot_key.index();
        let slice = self.slots.as_mut_slice();
        let slotref = &mut slice[slot_index.as_usize()];

        let mut replace = match slotref {
            GenMem::Available(_) => {
                return;
            }
            GenMem::Current(data) => {
                if slot_key.generation() != data.generation {
                    return;
                }
                GenMem::Available(data.generation.increment())
            }
        };

        std::mem::swap(slotref, &mut replace);
        self.free_slots.push(slot_index);
    }

    /// Allocate a slot and return SlotKey or None if at capacity
    fn allocate_slot(
        &mut self,
        channel: oneshot::Sender<Result<(), NackMsg<OtapPdata>>>,
    ) -> Option<SlotKey> {
        if let Some(slot_index) = self.free_slots.pop() {
            let slice = self.slots.as_mut_slice();
            let slotref = &mut slice[slot_index.as_usize()];

            match slotref {
                GenMem::Available(slotgen) => {
                    let generation = *slotgen;
                    let mut repl = GenMem::Current(SlotData {
                        channel,
                        generation,
                    });
                    std::mem::swap(slotref, &mut repl);
                    return Some(SlotKey::new(slot_index, generation));
                }
                _ => {}
            }
        };

        // If no free slots and we can still grow
        if self.slots.len() < self.config.max_slots {
            let slot_index = SlotIndex(self.slots.len());
            let generation = SlotGeneration(1);

            self.slots.push(GenMem::Current(SlotData {
                channel,
                generation,
            }));

            Some(SlotKey::new(slot_index, generation))
        } else {
            None // At capacity
        }
    }
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
struct OtapBatchDecoder {
    signal: SignalType,
}

impl OtapBatchDecoder {
    fn new(signal: SignalType) -> Self {
        Self { signal }
    }
}

impl Decoder for OtapBatchDecoder {
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
            let (tx, rx) = oneshot::channel();

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
                        state.free_slot(self.slot_key);
                    }
                }
            }

            let _guard = SlotGuard {
                slot_key,
                state: state.clone(),
            };

            // Create CallData with correlation information
            let call_data = smallvec![
                CtxVal::from(slot_key.index().as_usize()),
                CtxVal::from(slot_key.generation().as_usize()),
            ];

            // Subscribe to Ack/Nack responses
            effect_handler.subscribe_to(
                Interests::ACKS | Interests::NACKS,
                call_data,
                &mut otap_batch,
            );

            // Send message to pipeline
            match effect_handler.send_message(otap_batch).await {
                Ok(_) => {
                    // Wait for Ack/Nack response
                    match rx.await {
                        Ok(Ok(())) => Ok(tonic::Response::new(())),
                        Ok(Err(nack_msg)) => {
                            let status = if nack_msg.permanent {
                                Status::invalid_argument(nack_msg.reason)
                            } else {
                                Status::unavailable(nack_msg.reason)
                            };
                            Err(status)
                        }
                        Err(_) => Err(Status::internal("Response channel closed unexpectedly")),
                    }
                }
                Err(e) => {
                    // Failed to send to pipeline - clean up slot via drop
                    Err(Status::internal(
                        format!("Failed to send to pipeline: {e}",),
                    ))
                }
            }
        })
    }
}

/// handle the grpc service request
async fn handle_service_request(
    req: Request<Body>,
    signal: SignalType,
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
        let config = ServerConfig::default();
        let state = Arc::new(Mutex::new(ServerState::new(config)));

        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get correlation state for sharing with effect handlers
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
                        SignalType::Logs,
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
        let config = ServerConfig::default();
        let state = Arc::new(Mutex::new(ServerState::new(config)));

        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get correlation state for sharing with effect handlers
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
                        SignalType::Metrics,
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
        let config = ServerConfig::default();
        let state = Arc::new(Mutex::new(ServerState::new(config)));

        Self {
            effect_handler,
            state,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
    }

    /// Get correlation state for sharing with effect handlers
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
                        SignalType::Traces,
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
