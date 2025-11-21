// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! gRPC routing for OTAP Arrow and OTLP protobuf endpoints.
//!
//! This module owns:
//! - request header parsing and compression negotiation,
//! - dispatch to OTAP streaming handlers and OTLP unary handlers,
//! - the small per request context that holds encoders and response templates.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use super::ack::{AckCompletionFuture, AckPollResult, AckRegistry};
use super::encoder::{EncoderGuard, GrpcResponseFrameEncoder, ResponseEncoderPool};
use super::grpc::{
    AcceptedGrpcEncodings, GrpcStreamingBody, RequestTimeout, negotiate_response_encoding,
    parse_grpc_accept_encoding, parse_grpc_encoding,
};
use super::response_templates::ResponseTemplates;
use super::status::Status;
use super::stream::stream_batch_statuses;
use crate::compression::CompressionMethod;
use crate::otel_receiver::grpc;
use crate::pdata::{Context, OtapPdata};
use bytes::Bytes;
use h2::server::SendResponse;
use http::{HeaderMap, HeaderValue, Request, Response, StatusCode as HttpStatusCode};
use otap_df_config::SignalType;
use otap_df_engine::local::receiver as local;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_pdata::otap::{Logs, Metrics, OtapBatchStore, Traces};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use otap_df_pdata::{OtapArrowRecords, OtlpProtoBytes};
use zstd::bulk::Decompressor;

/// OTAP gRPC service paths.
const ARROW_LOGS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowLogsService/ArrowLogs";
const ARROW_METRICS_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowMetricsService/ArrowMetrics";
const ARROW_TRACES_SERVICE: &str =
    "/opentelemetry.proto.experimental.arrow.v1.ArrowTracesService/ArrowTraces";

/// OTLP gRPC service paths.
const OTLP_LOGS_SERVICE: &str = "/opentelemetry.proto.collector.logs.v1.LogsService/Export";
const OTLP_METRICS_SERVICE: &str =
    "/opentelemetry.proto.collector.metrics.v1.MetricsService/Export";
const OTLP_TRACES_SERVICE: &str = "/opentelemetry.proto.collector.trace.v1.TraceService/Export";

/// Routes each inbound gRPC request to the appropriate OTLP or OTAP handler.
///
/// A single instance lives per core and is shared across all h2 connections. It keeps:
/// - the effect handler into the pipeline,
/// - per signal Ack registries,
/// - request and response compression configuration,
/// - timeouts, encoder pool, and response header templates, and
/// - a shared zstd decompressor used by the streaming body decoder.
pub(crate) struct GrpcRequestRouter {
    pub(crate) effect_handler: local::EffectHandler<OtapPdata>,
    pub(crate) logs_ack_registry: Option<AckRegistry>,
    pub(crate) metrics_ack_registry: Option<AckRegistry>,
    pub(crate) traces_ack_registry: Option<AckRegistry>,
    pub(crate) max_in_flight_per_connection: usize,
    pub(crate) request_encodings: AcceptedGrpcEncodings,
    pub(crate) request_accept_header: HeaderValue,
    pub(crate) response_methods: Vec<CompressionMethod>,
    pub(crate) request_timeout: Option<Duration>,
    pub(crate) response_encoders: ResponseEncoderPool,
    pub(crate) response_templates: ResponseTemplates,
    // zstd decompressor shared by every stream on this core.
    pub(crate) zstd_decompressor: RefCell<Option<Decompressor<'static>>>,
    /// Maximum decoded message size in bytes (from GrpcServerSettings::max_decoding_message_size)
    pub(crate) max_decoding_message_size: u32,
}

/// Per request gRPC context used by the router while serving a single RPC.
struct RequestContext<'a> {
    request_encoding: grpc::GrpcEncoding,
    response: Response<()>,
    response_encoder: EncoderGuard<'a>,
}

impl GrpcRequestRouter {
    /// Validates the incoming headers and builds the response template and encoder for this call.
    ///
    /// This step:
    /// - parses the request `grpc-encoding`,
    /// - negotiates the response encoding against server preferences and client accept list, and
    /// - selects the appropriate prebuilt response header template.
    fn prepare_request<'a>(&'a self, headers: &HeaderMap) -> Result<RequestContext<'a>, Status> {
        let request_encoding = parse_grpc_encoding(headers, &self.request_encodings)?;
        let client_accept = parse_grpc_accept_encoding(headers);
        let response_encoding = negotiate_response_encoding(&self.response_methods, &client_accept);
        let response = self
            .response_templates
            .get_ok(CompressionMethod::from_grpc_encoding(response_encoding))
            .ok_or_else(|| Status::internal("failed to build response"))?;
        let response_encoder = self.response_encoders.checkout(response_encoding);

        Ok(RequestContext {
            request_encoding,
            response,
            response_encoder,
        })
    }

    /// Routes a single gRPC request to the correct signal specific handler based on the path.
    pub(crate) async fn route_grpc_request(
        self: Rc<Self>,
        request: Request<h2::RecvStream>,
        respond: SendResponse<Bytes>,
    ) -> Result<(), Status> {
        let path = request.uri().path();
        match path {
            ARROW_LOGS_SERVICE => {
                self.serve_otap_stream::<Logs>(
                    request,
                    respond,
                    OtapArrowRecords::Logs,
                    self.logs_ack_registry.clone(),
                )
                .await
            }
            ARROW_METRICS_SERVICE => {
                self.serve_otap_stream::<Metrics>(
                    request,
                    respond,
                    OtapArrowRecords::Metrics,
                    self.metrics_ack_registry.clone(),
                )
                .await
            }
            ARROW_TRACES_SERVICE => {
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
                log::warn!("Unknown OTEL gRPC path {}", path);
                respond_with_error(
                    respond,
                    Status::unimplemented("unknown method"),
                    &self.request_accept_header,
                );
                Ok(())
            }
        }
    }

    /// Serves an OTAP Arrow bidirectional stream.
    ///
    /// Batches from the client are decoded and sent into the pipeline. Ack and Nack outcomes
    /// are converted back into `BatchStatus` messages on the same stream.
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
        let mut ctx = self.prepare_request(request.headers())?;
        let recv_stream = request.into_body();
        let body = GrpcStreamingBody::new(
            recv_stream,
            ctx.request_encoding,
            Rc::clone(self),
            self.max_decoding_message_size as usize,
        );

        let mut status_stream = stream_batch_statuses::<GrpcStreamingBody, T, _>(
            body,
            self.effect_handler.clone(),
            ack_registry,
            otap_batch,
            self.max_in_flight_per_connection,
        );

        let mut send_stream = respond
            .send_response(ctx.response, false)
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
                    let bytes = ctx.response_encoder.encode(&status)?;
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

    /// Serves a unary OTLP Export call (Logs, Metrics, or Traces).
    ///
    /// The request body is optionally compressed and contains a single Export request which
    /// is forwarded into the pipeline. Depending on `wait_for_result` we either:
    /// - wait for an Ack or Nack and convert it to a gRPC status, or
    /// - return an empty `Export*ServiceResponse` immediately after enqueueing.
    async fn serve_otlp_unary(
        self: &Rc<Self>,
        request: Request<h2::RecvStream>,
        mut respond: SendResponse<Bytes>,
        signal: SignalType,
        ack_registry: Option<AckRegistry>,
    ) -> Result<(), Status> {
        let (parts, body) = request.into_parts();
        let mut ctx = self.prepare_request(&parts.headers)?;
        let mut recv_stream = GrpcStreamingBody::new(
            body,
            ctx.request_encoding,
            Rc::clone(self),
            self.max_decoding_message_size as usize,
        );
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

        // Wrap the raw request protobuf bytes in pipeline pdata.
        let mut otap_pdata = OtapPdata::new(
            Context::default(),
            otlp_proto_bytes(signal, request_bytes).into(),
        );

        // Optional Ack tracking depending on `wait_for_result`.
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

        // If we are not waiting for results we can respond immediately.
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

        let mut send_stream = respond
            .send_response(ctx.response, false)
            .map_err(|e| Status::internal(format!("failed to send response headers: {e}")))?;

        let payload = encode_otlp_response(signal, &mut ctx.response_encoder)?;
        if let Err(e) = send_stream.send_data(payload, false) {
            log::debug!("send_data failed: {e}");
            return Ok(());
        }

        send_ok_trailers(send_stream);
        Ok(())
    }
}

/// Sends trailers for a successful gRPC response (status code 0).
fn send_ok_trailers(mut stream: h2::SendStream<Bytes>) {
    let mut trailers = HeaderMap::new();
    let _ = trailers.insert("grpc-status", HeaderValue::from_static("0"));
    if let Err(e) = stream.send_trailers(trailers) {
        log::debug!("send_trailers failed: {e}");
    }
}

/// Sends trailers for a failed gRPC response with the provided status code and message.
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

/// Sends a unary gRPC error response with an empty body and error trailers.
///
/// We always respond with HTTP 200 and encode the error in `grpc-status` and `grpc-message`
/// trailers as per the gRPC spec.
pub(crate) fn respond_with_error(
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

/// Wraps raw OTLP protobuf bytes into the typed enum used by the pipeline.
fn otlp_proto_bytes(signal: SignalType, bytes: Bytes) -> OtlpProtoBytes {
    match signal {
        SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(bytes),
        SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(bytes),
        SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(bytes),
    }
}

/// Builds the empty `Export*ServiceResponse` payload for a successful OTLP call.
fn encode_otlp_response(
    signal: SignalType,
    encoder: &mut GrpcResponseFrameEncoder,
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
