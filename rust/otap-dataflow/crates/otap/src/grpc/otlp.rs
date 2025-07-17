// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

//! crate containing GRPC Server implementations for the OTLP services that
//! convert the received OTLP signals into OTAP

use std::convert::Infallible;
use std::task::Poll;

use futures::future::BoxFuture;
use http::{Request, Response};
use otap_df_engine::shared::receiver as shared;
use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otap_df_otlp::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use otel_arrow_rust::otap::OtapBatch;
use prost::{Message, bytes::Buf};
use tonic::Status;
use tonic::body::Body;
use tonic::codec::{Codec, CompressionEncoding, Decoder, EnabledCompressionEncodings, Encoder};
use tonic::server::{Grpc, NamedService, UnaryService};

use crate::encoder::encode_logs_otap_batch;
use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;

#[derive(Clone)]
enum Signal {
    Logs,
    Metrics,
    Traces,
}

/// Tonic `Codec` implementation that decodes OtapBatches from grpc requests bytes and and encodes
/// otlp service responses.
struct OtapBatchCodec {
    signal: Signal,
}

impl OtapBatchCodec {
    fn new(signal: Signal) -> Self {
        Self { signal }
    }
}

impl Codec for OtapBatchCodec {
    type Decode = OtapBatch;
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

    fn encode(
        &mut self,
        _item: Self::Item,
        dst: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
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
    type Item = OtapBatch;

    type Error = Status;

    fn decode(
        &mut self,
        src: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let buf = src.chunk();
        let view_impl = RawLogsData::new(buf);
        let res = match self.signal {
            Signal::Logs => encode_logs_otap_batch(&view_impl),
            // TODO - implement for other signal types
            _ => {
                return Err(Status::unimplemented("signal type not yet implemented"));
            }
        };
        src.advance(buf.len());
        match res {
            Ok(batch) => Ok(Some(batch)),
            Err(e) => Err(Status::internal(format!("Internal Error: {e}"))),
        }
    }
}

/// implementation of tonic service that handles the decoded request (the OtapBatch).
struct OtapBatchService {
    effect_handler: shared::EffectHandler<OtapBatch>,
}

impl OtapBatchService {
    fn new(effect_handler: shared::EffectHandler<OtapBatch>) -> Self {
        Self { effect_handler }
    }
}

impl UnaryService<OtapBatch> for OtapBatchService {
    type Response = ();
    type Future = BoxFuture<'static, Result<tonic::Response<Self::Response>, Status>>;

    fn call(&mut self, request: tonic::Request<OtapBatch>) -> Self::Future {
        let otap_batch = request.into_inner();

        let effect_handler = self.effect_handler.clone();
        Box::pin(async move {
            match effect_handler.send_message(otap_batch).await {
                Ok(result) => Ok(tonic::Response::new(result)),
                Err(e) => Err(Status::internal(format!(
                    "internal error handling request: {e}",
                ))),
            }
        })
    }
}

/// handle the grpc service request
async fn handle_service_request(
    req: Request<Body>,
    signal: Signal,
    effect_handler: shared::EffectHandler<OtapBatch>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
) -> Response<Body> {
    let codec = OtapBatchCodec::new(signal);
    let mut grpc = Grpc::new(codec)
        .apply_compression_config(accept_compression_encodings, send_compression_encodings);
    grpc.unary(OtapBatchService::new(effect_handler), req).await
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
    effect_handler: shared::EffectHandler<OtapBatch>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl LogsServiceServer {
    /// create a new instance of `LogsServiceServer`
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapBatch>) -> Self {
        Self {
            effect_handler,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
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
            "/opentelemetry.proto.collector.logs.v1.LogsService/Export" => {
                let effect_handler = self.effect_handler.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Logs,
                        effect_handler,
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

const LOGS_SERVICE_NAME: &str = "opentelemetry.proto.collector.logs.v1.LogsService";
impl NamedService for LogsServiceServer {
    const NAME: &'static str = LOGS_SERVICE_NAME;
}

/// implementation of OTLP bytes -> OTAP GRPC server for metrics
#[derive(Clone)]
pub struct MetricsServiceServer {
    effect_handler: shared::EffectHandler<OtapBatch>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl MetricsServiceServer {
    /// create a new instance of `MetricsServiceServer`
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapBatch>) -> Self {
        Self {
            effect_handler,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
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
            "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export" => {
                let effect_handler = self.effect_handler.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Metrics,
                        effect_handler,
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

const METRICS_SERVICE_NAME: &str = "opentelemetry.proto.collector.metrics.v1.MetricsService";
impl NamedService for MetricsServiceServer {
    const NAME: &'static str = METRICS_SERVICE_NAME;
}

/// implementation of OTLP bytes -> OTAP GRPC server for traces
#[derive(Clone)]
pub struct TraceServiceServer {
    effect_handler: shared::EffectHandler<OtapBatch>,
    accept_compression_encodings: EnabledCompressionEncodings,
    send_compression_encodings: EnabledCompressionEncodings,
}

impl TraceServiceServer {
    /// create a new instance of `TracesServiceServer`
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapBatch>) -> Self {
        Self {
            effect_handler,
            accept_compression_encodings: Default::default(),
            send_compression_encodings: Default::default(),
        }
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
            "/opentelemetry.proto.collector.trace.v1.TraceService/Export" => {
                let effect_handler = self.effect_handler.clone();
                let accept_compression_encodings = self.accept_compression_encodings;
                let send_compression_encodings = self.send_compression_encodings;
                Box::pin(async move {
                    let res = handle_service_request(
                        req,
                        Signal::Traces,
                        effect_handler,
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

const TRACES_SERVICE_NAME: &str = "opentelemetry.proto.collector.trace.v1.TraceService";
impl NamedService for TraceServiceServer {
    const NAME: &'static str = TRACES_SERVICE_NAME;
}
