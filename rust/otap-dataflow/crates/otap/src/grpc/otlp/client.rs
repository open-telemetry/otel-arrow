// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

//! Implementations of OTLP grpc service clients that can accept serialized bytes as the
//! request payload.
//!
//! The motivation behind this client implementation is that our telemetry pipelines will be
//! able to receive GRPC OTLP requests, and if there's no need to serialize them, we can keep
//! the payload serialized as protobuf before then forwarding using these clients.

use http::uri::PathAndQuery;
use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otap_df_otlp::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use prost::Message;
use prost::bytes::BufMut;
use std::marker::PhantomData;
use tonic::body::Body;
use tonic::client::{Grpc, GrpcService};
use tonic::codec::{Codec, CompressionEncoding, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::transport::{Channel, Endpoint};
use tonic::{GrpcMethod, Status};

/// Codec for converting OTLP Requests and Responses for our client implementations
#[derive(Default)]
struct OtlpRequestCodec<T> {
    _pd: PhantomData<T>,
}

impl<T> Codec for OtlpRequestCodec<T>
where
    T: Message + Default + Send + 'static,
{
    type Encode = Vec<u8>;
    type Decode = T;

    type Encoder = OtlpRequestEncoder;
    type Decoder = OtlpResponseDecoder<T>;

    fn decoder(&mut self) -> Self::Decoder {
        OtlpResponseDecoder::default()
    }

    fn encoder(&mut self) -> Self::Encoder {
        OtlpRequestEncoder::default()
    }
}

/// Encoder implementation for encoding Requests for our client implementations.
///
/// This encoder expects that the OTLP Bytes for the response were passed directly into
/// the grpc client, so it simply puts the bytes directly into the buffer without doing
/// any serialization
#[derive(Default)]
struct OtlpRequestEncoder {}

impl Encoder for OtlpRequestEncoder {
    type Error = Status;
    type Item = Vec<u8>;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        dst.put(item.as_ref());
        Ok(())
    }
}

/// Decoder implementation for our decoding GRPC responses.
///
/// This is generic over the type of prost message it decodes
#[derive(Default)]
struct OtlpResponseDecoder<T> {
    _pd: PhantomData<T>,
}

impl<T> Decoder for OtlpResponseDecoder<T>
where
    T: Message + Default + Send + 'static,
{
    type Item = T;
    type Error = Status;

    fn decode(&mut self, buf: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let item = Message::decode(buf)
            .map(Some)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(item)
    }
}

/// Generic implementation of OTLP Service
pub struct OtlpServiceClient<T, Resp, S> {
    inner: Grpc<T>,
    _pd: PhantomData<(Resp, S)>,
}

impl<Resp, S> OtlpServiceClient<Channel, Resp, S>
where
    S: ServiceDescriptor,
    Resp: Message + Default + Send + 'static,
{
    /// Attempt to create a new client by connecting to a given endpoint.
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: TryInto<Endpoint>,
        D::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let conn = Endpoint::new(dst)?.connect().await?;
        Ok(Self::new(conn))
    }
}

impl<T, Resp, S> OtlpServiceClient<T, Resp, S>
where
    T: GrpcService<Body>,
    T::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T::ResponseBody: Send + 'static,
    <T::ResponseBody as tonic::transport::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    S: ServiceDescriptor,
    Resp: Message + Default + Send + 'static,
{
    /// create a new instance of `[LogsServiceClient]`
    pub fn new(inner: T) -> Self {
        let inner = Grpc::new(inner);
        Self {
            inner,
            _pd: PhantomData,
        }
    }

    /// Compress requests with the given encoding.
    ///
    /// This requires the server to support it otherwise it might respond with an
    /// error.
    #[must_use]
    pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.send_compressed(encoding);
        self
    }

    /// Enable decompressing responses.
    #[must_use]
    pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
        self.inner = self.inner.accept_compressed(encoding);
        self
    }

    /// Send the serialized grpc request
    pub async fn export(
        &mut self,
        request: impl tonic::IntoRequest<Vec<u8>>,
    ) -> Result<tonic::Response<Resp>, Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| Status::unknown(format!("Service was not ready: {}", e.into())))?;

        let codec = OtlpRequestCodec::<Resp>::default();
        let path = PathAndQuery::from_static(S::EXPORT_PATH);
        let mut req = request.into_request();
        _ = req
            .extensions_mut()
            .insert(GrpcMethod::new(S::SERVICE_NAME, "Export"));

        self.inner.unary(req, path, codec).await
    }
}

/// trait that supplies static strings including the service name and the export path
pub trait ServiceDescriptor {
    /// path of the export ipc
    const EXPORT_PATH: &str;

    /// name of the grpc service
    const SERVICE_NAME: &str;
}

/// descriptor of LogsService
pub struct LogsServiceDescriptor {}

impl ServiceDescriptor for LogsServiceDescriptor {
    const EXPORT_PATH: &str = super::LOGS_SERVICE_EXPORT_PATH;
    const SERVICE_NAME: &str = super::LOGS_SERVICE_NAME;
}

/// descriptor of MetricsService
pub struct MetricsServiceDescriptor {}

impl ServiceDescriptor for MetricsServiceDescriptor {
    const EXPORT_PATH: &str = super::METRICS_SERVICE_EXPORT_PATH;
    const SERVICE_NAME: &str = super::METRICS_SERVICE_NAME;
}

/// descriptor of TracesService
pub struct TraceServiceDescriptor {}

impl ServiceDescriptor for TraceServiceDescriptor {
    const EXPORT_PATH: &str = super::TRACE_SERVICE_EXPORT_PATH;
    const SERVICE_NAME: &str = super::TRACE_SERVICE_NAME;
}

/// Implementation of OTLP Logs Service client that can accept and send pre-serialized requests
pub type LogsServiceClient<T> =
    OtlpServiceClient<T, ExportLogsServiceResponse, LogsServiceDescriptor>;

/// Implementation of OTLP Metrics Service client that can accept and send pre-serialized requests
pub type MetricsServiceClient<T> =
    OtlpServiceClient<T, ExportMetricsServiceResponse, MetricsServiceDescriptor>;

/// Implementation of OTLP Traces Service client that can accept and send pre-serialized requests
pub type TraceServiceClient<T> =
    OtlpServiceClient<T, ExportTraceServiceResponse, TraceServiceDescriptor>;
