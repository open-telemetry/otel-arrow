// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP/HTTP receiver implementation.
//!
//! This module implements the OTLP/HTTP protocol endpoints:
//! - `POST /v1/logs`
//! - `POST /v1/metrics`
//! - `POST /v1/traces`
//!
//! The receiver keeps request payloads in their serialized protobuf form and forwards them into
//! the pipeline as `OtapPdata`, matching the OTLP/gRPC receiver's lazy-decoding strategy.

use crate::otap_grpc::common::AckRegistry;
use crate::otap_grpc::otlp::server_new::AckSlot;
use crate::pdata::{Context, OtapPdata};
use crate::socket_options;

use bytes::Bytes;
use http::{HeaderValue, Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::Body;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use otap_df_config::SignalType;
use otap_df_config::byte_units;
use otap_df_engine::shared::receiver::EffectHandler;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceResponse;
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceResponse;
use otap_df_telemetry::metrics::MetricSet;
use parking_lot::Mutex;
use prost::Message;
use prost_types::Any;
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::{Semaphore, oneshot};
use zstd::stream::read::Decoder as ZstdDecoder;

#[cfg(feature = "experimental-tls")]
use crate::tls_utils::build_tls_acceptor;
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsServerConfig;

/// OTLP protobuf content type
const PROTOBUF_CONTENT_TYPE: &str = "application/x-protobuf";

/// Settings for the OTLP/HTTP server.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HttpServerSettings {
    /// Listening address for OTLP/HTTP (typically `0.0.0.0:4318`).
    pub listening_addr: SocketAddr,

    /// Maximum number of concurrent in-flight requests.
    /// Defaults to `0`, which means the receiver adopts the downstream pdata channel capacity.
    #[serde(default = "default_max_concurrent_requests")]
    pub max_concurrent_requests: usize,

    /// Whether newly accepted sockets should have `TCP_NODELAY` enabled.
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,

    /// TCP keepalive timeout for accepted sockets.
    #[serde(default = "default_tcp_keepalive", with = "humantime_serde")]
    pub tcp_keepalive: Option<Duration>,

    /// Interval between TCP keepalive probes.
    #[serde(default = "default_tcp_keepalive_interval", with = "humantime_serde")]
    pub tcp_keepalive_interval: Option<Duration>,

    /// Number of TCP keepalive probes before a connection is declared dead.
    #[serde(default = "default_tcp_keepalive_retries")]
    pub tcp_keepalive_retries: Option<u32>,

    /// Maximum request body size, in bytes.
    /// Accepts plain integers or suffixed strings such as `4MiB`.
    ///
    /// This limit applies to:
    /// - Uncompressed bodies: the wire size (bytes received over network)
    /// - Compressed bodies: BOTH the compressed wire size AND decompressed size
    ///
    /// Example scenarios with `max_request_body_size = 4MiB`:
    /// - 5MiB compressed payload → rejected during collection (wire size > 4MiB)
    /// - 2MiB compressed payload expanding to 10MiB → rejected during decompression
    /// - 2MiB compressed payload expanding to 3MiB → accepted
    ///
    /// This dual enforcement provides defense-in-depth against both bandwidth abuse
    /// and decompression bombs (zip bombs).
    #[serde(
        default = "default_max_request_body_size",
        deserialize_with = "deserialize_body_size"
    )]
    pub max_request_body_size: u32,

    /// Whether to wait for the immediate downstream ACK/NACK before responding.
    #[serde(default = "default_wait_for_result")]
    pub wait_for_result: bool,

    /// Timeout for processing a request (send + optional wait).
    ///
    /// If unset in configuration, defaults to a conservative value to reduce the
    /// risk of slow-client (Slowloris-style) denial of service.
    #[serde(default = "default_http_timeout", with = "humantime_serde")]
    pub timeout: Option<Duration>,

    /// Whether to accept gzip/deflate HTTP request bodies (via `Content-Encoding`).
    ///
    /// When disabled, non-identity encodings are rejected with `415 Unsupported Media Type`.
    #[serde(default = "default_accept_compressed_requests")]
    pub accept_compressed_requests: bool,

    /// Optional TLS configuration.
    ///
    /// This is only available when the `experimental-tls` feature is enabled.
    #[cfg(feature = "experimental-tls")]
    #[serde(default)]
    pub tls: Option<TlsServerConfig>,
}

impl Default for HttpServerSettings {
    fn default() -> Self {
        Self {
            listening_addr: "127.0.0.1:4318".parse().expect("valid default addr"),
            max_concurrent_requests: default_max_concurrent_requests(),
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: default_tcp_keepalive_interval(),
            tcp_keepalive_retries: default_tcp_keepalive_retries(),
            max_request_body_size: default_max_request_body_size(),
            wait_for_result: default_wait_for_result(),
            timeout: default_http_timeout(),
            accept_compressed_requests: default_accept_compressed_requests(),
            #[cfg(feature = "experimental-tls")]
            tls: None,
        }
    }
}

const fn default_max_concurrent_requests() -> usize {
    0
}

const fn default_tcp_nodelay() -> bool {
    true
}

const fn default_tcp_keepalive() -> Option<Duration> {
    Some(Duration::from_secs(45))
}

const fn default_tcp_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(15))
}

const fn default_tcp_keepalive_retries() -> Option<u32> {
    Some(5)
}

const fn default_max_request_body_size() -> u32 {
    // Keep this aligned with tonic's default max-decoding size (4MiB) for symmetry.
    4 * 1024 * 1024
}

fn deserialize_body_size<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(byte_units::deserialize(deserializer)?.unwrap_or(default_max_request_body_size()))
}

const fn default_wait_for_result() -> bool {
    false
}

const fn default_accept_compressed_requests() -> bool {
    true
}

fn default_http_timeout() -> Option<Duration> {
    // Conservative default chosen to mitigate slow-client DoS while keeping parity
    // with common collector defaults.
    Some(Duration::from_secs(30))
}

/// Tune max concurrent requests relative to downstream capacity.
pub fn tune_max_concurrent_requests(config: &mut HttpServerSettings, downstream_capacity: usize) {
    let safe_capacity = downstream_capacity.max(1);
    if config.max_concurrent_requests == 0 || config.max_concurrent_requests > safe_capacity {
        config.max_concurrent_requests = safe_capacity;
    }

    // Safety: ensure timeouts are enabled by default even if older configs relied
    // on `timeout: null` or omitted the field entirely.
    if config.timeout.is_none() {
        config.timeout = default_http_timeout();
    }
}

fn content_type_is_protobuf(headers: &http::HeaderMap) -> bool {
    headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            let base = v.split(';').next().unwrap_or("").trim();
            base.eq_ignore_ascii_case(PROTOBUF_CONTENT_TYPE)
        })
}

fn response_bytes(signal: SignalType) -> Bytes {
    fn encode<T: Message + Default>() -> Bytes {
        let mut buf = Vec::with_capacity(T::default().encoded_len());
        T::default().encode(&mut buf).expect("encode response");
        Bytes::from(buf)
    }

    static LOGS: OnceLock<Bytes> = OnceLock::new();
    static METRICS: OnceLock<Bytes> = OnceLock::new();
    static TRACES: OnceLock<Bytes> = OnceLock::new();

    match signal {
        SignalType::Logs => LOGS
            .get_or_init(encode::<ExportLogsServiceResponse>)
            .clone(),
        SignalType::Metrics => METRICS
            .get_or_init(encode::<ExportMetricsServiceResponse>)
            .clone(),
        SignalType::Traces => TRACES
            .get_or_init(encode::<ExportTraceServiceResponse>)
            .clone(),
    }
}

fn ok_response(signal: SignalType) -> Response<Full<Bytes>> {
    let mut resp = Response::new(Full::new(response_bytes(signal)));
    *resp.status_mut() = StatusCode::OK;
    let headers = resp.headers_mut();
    _ = headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static(PROTOBUF_CONTENT_TYPE),
    );
    resp
}

/// Google RPC status format for HTTP error responses.
///
/// This follows the Google RPC error model, encoding status codes and messages
/// in protobuf format for consistency with gRPC error handling.
/// See: https://github.com/googleapis/googleapis/blob/master/google/rpc/status.proto
#[derive(Clone, PartialEq, ::prost::Message)]
pub(crate) struct RpcStatus {
    #[prost(int32, tag = "1")]
    pub(crate) code: i32,
    #[prost(string, tag = "2")]
    pub(crate) message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<Any>,
}

fn rpc_status_response(
    status: StatusCode,
    grpc_code: i32,
    message: impl Into<String>,
) -> Response<Full<Bytes>> {
    let status_msg = RpcStatus {
        code: grpc_code,
        message: message.into(),
        details: Vec::new(),
    };

    let mut buf = Vec::with_capacity(status_msg.encoded_len());
    status_msg.encode(&mut buf).expect("encode RpcStatus");

    let mut resp = Response::new(Full::new(Bytes::from(buf)));
    *resp.status_mut() = status;
    _ = resp.headers_mut().insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static(PROTOBUF_CONTENT_TYPE),
    );
    resp
}

fn empty_response(status: StatusCode) -> Response<Full<Bytes>> {
    let mut resp = Response::new(Full::new(Bytes::new()));
    *resp.status_mut() = status;
    resp
}

fn limited_body_too_large() -> Response<Full<Bytes>> {
    // Align with Go collector's OTLP/HTTP receiver behavior: 400 for oversized bodies.
    rpc_status_response(StatusCode::BAD_REQUEST, 3, "request body too large")
}

fn unsupported_media_type() -> Response<Full<Bytes>> {
    empty_response(StatusCode::UNSUPPORTED_MEDIA_TYPE)
}

fn method_not_allowed() -> Response<Full<Bytes>> {
    empty_response(StatusCode::METHOD_NOT_ALLOWED)
}

fn not_found() -> Response<Full<Bytes>> {
    empty_response(StatusCode::NOT_FOUND)
}

fn service_unavailable() -> Response<Full<Bytes>> {
    rpc_status_response(StatusCode::SERVICE_UNAVAILABLE, 14, "service unavailable")
}

fn internal_error() -> Response<Full<Bytes>> {
    rpc_status_response(StatusCode::INTERNAL_SERVER_ERROR, 13, "internal error")
}

fn map_path_to_signal(path: &str) -> Option<SignalType> {
    match path {
        "/v1/logs" => Some(SignalType::Logs),
        "/v1/metrics" => Some(SignalType::Metrics),
        "/v1/traces" => Some(SignalType::Traces),
        _ => None,
    }
}

#[allow(clippy::result_large_err)]
fn decode_content_encoding(
    accept_compressed_requests: bool,
    headers: &http::HeaderMap,
    body: Bytes,
    max_len: usize,
) -> Result<Bytes, Response<Full<Bytes>>> {
    let encoding = headers
        .get(http::header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("identity")
        .trim();

    if encoding.eq_ignore_ascii_case("identity") || encoding.is_empty() {
        return Ok(body);
    }

    if !accept_compressed_requests {
        return Err(unsupported_media_type());
    }

    // Decompress into a bounded buffer.
    if encoding.eq_ignore_ascii_case("gzip") {
        let mut decoder = flate2::read::GzDecoder::new(body.as_ref());
        let decoded = read_to_end_limited(&mut decoder, max_len).map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("decoded body too large") {
                // Size limit exceeded
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "decompressed_size_exceeded",
                    encoding = "gzip",
                    max_len = max_len,
                    error = err_msg.as_str()
                );
                limited_body_too_large()
            } else {
                // Format/corruption error
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "invalid_encoding",
                    encoding = "gzip",
                    error = err_msg.as_str()
                );
                rpc_status_response(
                    StatusCode::BAD_REQUEST,
                    3,
                    format!("gzip decode failed: {err_msg}"),
                )
            }
        })?;
        Ok(Bytes::from(decoded))
    } else if encoding.eq_ignore_ascii_case("deflate") {
        let mut decoder = flate2::read::ZlibDecoder::new(body.as_ref());
        let decoded = read_to_end_limited(&mut decoder, max_len).map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("decoded body too large") {
                // Size limit exceeded
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "decompressed_size_exceeded",
                    encoding = "deflate",
                    max_len = max_len,
                    error = err_msg.as_str()
                );
                limited_body_too_large()
            } else {
                // Format/corruption error
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "invalid_encoding",
                    encoding = "deflate",
                    error = err_msg.as_str()
                );
                rpc_status_response(
                    StatusCode::BAD_REQUEST,
                    3,
                    format!("deflate decode failed: {err_msg}"),
                )
            }
        })?;
        Ok(Bytes::from(decoded))
    } else if encoding.eq_ignore_ascii_case("zstd") {
        let mut decoder = ZstdDecoder::new(body.as_ref()).map_err(|e| {
            let err_msg = e.to_string();
            otap_df_telemetry::otel_debug!(
                "HttpRequestRejected",
                reason = "invalid_encoding",
                encoding = "zstd",
                error = err_msg.as_str()
            );
            rpc_status_response(
                StatusCode::BAD_REQUEST,
                3,
                format!("zstd decode failed: {err_msg}"),
            )
        })?;

        let decoded = read_to_end_limited(&mut decoder, max_len).map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("decoded body too large") {
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "decompressed_size_exceeded",
                    encoding = "zstd",
                    max_len = max_len,
                    error = err_msg.as_str()
                );
                limited_body_too_large()
            } else {
                otap_df_telemetry::otel_debug!(
                    "HttpRequestRejected",
                    reason = "invalid_encoding",
                    encoding = "zstd",
                    error = err_msg.as_str()
                );
                rpc_status_response(
                    StatusCode::BAD_REQUEST,
                    3,
                    format!("zstd decode failed: {err_msg}"),
                )
            }
        })?;

        Ok(Bytes::from(decoded))
    } else {
        Err(unsupported_media_type())
    }
}

fn read_to_end_limited<R: std::io::Read>(
    reader: &mut R,
    max_len: usize,
) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break; // EOF reached - success
        }

        // Check AFTER reading if we would exceed the limit.
        // We allow reading past the limit into 'buf' to detect overflow,
        // but we reject appending it to 'out'.
        if out.len().saturating_add(n) > max_len {
            return Err(std::io::Error::other("decoded body too large"));
        }

        out.extend_from_slice(&buf[..n]);
    }
    Ok(out)
}

#[derive(Clone)]
struct HttpHandler {
    effect_handler: EffectHandler<OtapPdata>,
    ack_registry: AckRegistry,
    metrics: Arc<Mutex<MetricSet<crate::otlp_receiver::OtlpReceiverMetrics>>>,
    settings: HttpServerSettings,
    semaphore: Arc<Semaphore>,
}

impl HttpHandler {
    async fn handle(self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let Some(signal) = map_path_to_signal(req.uri().path()) else {
            return Ok(not_found());
        };

        if req.method() != Method::POST {
            return Ok(method_not_allowed());
        }

        if !content_type_is_protobuf(req.headers()) {
            return Ok(unsupported_media_type());
        }

        let timeout = self.settings.timeout;
        let path = req.uri().path().to_string();
        let path_for_fut = path.clone();

        // Acquire permit with timeout for fair queuing across protocols.
        // Uses the request timeout if configured, otherwise a short default to avoid
        // indefinite queuing.
        //
        // Important: this is done inside the request future so the overall request timeout
        // (if enabled) accounts for time spent waiting for a permit.
        let permit_timeout = self.settings.timeout.unwrap_or(Duration::from_secs(5));

        let fut = async move {
            let permit_result =
                tokio::time::timeout(permit_timeout, self.semaphore.clone().acquire_owned()).await;

            let _permit = match permit_result {
                Ok(Ok(permit)) => permit,
                Ok(Err(_)) => {
                    // Semaphore closed (shouldn't happen)
                    otap_df_telemetry::otel_error!(
                        "HttpSemaphoreClosed",
                        path = path_for_fut.as_str()
                    );
                    self.metrics.lock().rejected_requests.inc();
                    return Err(internal_error());
                }
                Err(_) => {
                    // Timeout waiting for permit
                    otap_df_telemetry::otel_warn!(
                        "HttpRequestRejected",
                        reason = "concurrency_limit_timeout",
                        path = path_for_fut.as_str(),
                        max_concurrent = self.settings.max_concurrent_requests,
                        timeout_ms = permit_timeout.as_millis() as u64
                    );
                    self.metrics.lock().rejected_requests.inc();
                    return Err(service_unavailable());
                }
            };

            self.metrics.lock().requests_started.inc();

            let max_len = self.settings.max_request_body_size as usize;

            let (parts, body) = req.into_parts();
            let headers = parts.headers;

            // Collect request body.
            let size_hint = body.size_hint();
            if let Some(upper) = size_hint.upper() {
                if (upper as usize) > max_len {
                    otap_df_telemetry::otel_debug!(
                        "HttpRequestRejected",
                        reason = "body_too_large_hint",
                        max_len = max_len,
                        size_hint = upper,
                        path = parts.uri.path().to_string()
                    );
                    self.metrics.lock().rejected_requests.inc();
                    return Err(limited_body_too_large());
                }
            }

            let collected = Limited::new(body, max_len).collect().await.map_err(|e| {
                if e.downcast_ref::<LengthLimitError>().is_some() {
                    otap_df_telemetry::otel_debug!(
                        "HttpRequestRejected",
                        reason = "body_too_large_collected",
                        max_len = max_len,
                        path = parts.uri.path().to_string()
                    );
                    self.metrics.lock().rejected_requests.inc();
                    return limited_body_too_large();
                }

                otap_df_telemetry::otel_warn!("HttpBodyCollectionFailed", error = e.to_string());
                internal_error()
            })?;
            let body = collected.to_bytes();
            self.metrics.lock().request_bytes.add(body.len() as u64);

            let body = decode_content_encoding(
                self.settings.accept_compressed_requests,
                &headers,
                body,
                max_len,
            )?;

            let context = if self.settings.wait_for_result {
                Context::with_capacity(1)
            } else {
                Context::default()
            };

            let payload = match signal {
                SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(body),
                SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(body),
                SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(body),
            };

            let mut pdata = OtapPdata::new(context, payload.into());

            let cancel_rx = if self.settings.wait_for_result {
                let state = match signal {
                    SignalType::Logs => self.ack_registry.logs.clone(),
                    SignalType::Metrics => self.ack_registry.metrics.clone(),
                    SignalType::Traces => self.ack_registry.traces.clone(),
                };

                let Some(state) = state else {
                    return Err(internal_error());
                };

                let (key, rx) = {
                    let mut guard = state.0.lock();
                    match guard.allocate(|| oneshot::channel()) {
                        None => {
                            self.metrics.lock().rejected_requests.inc();
                            return Err(service_unavailable());
                        }
                        Some(pair) => pair,
                    }
                };

                // Register calldata in the context.
                self.effect_handler.subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    key.into(),
                    &mut pdata,
                );

                Some((SlotGuard { key, state }, rx))
            } else {
                None
            };

            if self.effect_handler.send_message(pdata).await.is_err() {
                otap_df_telemetry::otel_warn!(
                    "HttpPipelineSendFailed",
                    path = parts.uri.path().to_string(),
                    signal = format!("{:?}", signal)
                );
                return Err(service_unavailable());
            }

            if let Some((_guard, rx)) = cancel_rx {
                match rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(nack)) => {
                        otap_df_telemetry::otel_debug!(
                            "HttpRequestNacked",
                            reason = nack.reason.as_str(),
                            signal = format!("{:?}", signal)
                        );
                        // Include the nack reason in the response for parity with gRPC
                        return Err(rpc_status_response(
                            StatusCode::SERVICE_UNAVAILABLE,
                            14, // gRPC UNAVAILABLE code
                            format!("Pipeline processing failed: {}", nack.reason),
                        ));
                    }
                    Err(_) => {
                        return Err(internal_error());
                    }
                }
            }

            self.metrics.lock().requests_completed.inc();
            Ok(ok_response(signal))
        };

        let result = if let Some(timeout_duration) = timeout {
            match tokio::time::timeout(timeout_duration, fut).await {
                Ok(inner) => inner,
                Err(_) => {
                    otap_df_telemetry::otel_warn!(
                        "HttpRequestTimeout",
                        path = path.as_str(),
                        timeout_ms = timeout_duration.as_millis() as u64,
                        signal = format!("{:?}", signal)
                    );
                    Err(service_unavailable())
                }
            }
        } else {
            fut.await
        };

        Ok(result.unwrap_or_else(|resp| resp))
    }
}

struct SlotGuard {
    key: crate::accessory::slots::Key,
    state: AckSlot,
}

impl Drop for SlotGuard {
    fn drop(&mut self) {
        self.state.0.lock().cancel(self.key);
    }
}

/// Starts the OTLP/HTTP server.
///
/// # Parameters
///
/// - `shared_semaphore`: Optional semaphore shared with other protocol servers
///   (e.g., gRPC). When provided, all protocols draw from the same concurrency
///   pool. When `None`, creates a dedicated semaphore based on `settings.max_concurrent_requests`.
pub async fn serve(
    effect_handler: EffectHandler<OtapPdata>,
    settings: HttpServerSettings,
    ack_registry: AckRegistry,
    metrics: Arc<Mutex<MetricSet<crate::otlp_receiver::OtlpReceiverMetrics>>>,
    shared_semaphore: Option<Arc<Semaphore>>,
) -> std::io::Result<()> {
    let listener = effect_handler
        .tcp_listener(settings.listening_addr)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    // Use shared semaphore if provided, otherwise create a dedicated one
    let semaphore = shared_semaphore.unwrap_or_else(|| {
        debug_assert!(
            settings.max_concurrent_requests > 0,
            "tune_max_concurrent_requests should have been called before serve()"
        );
        Arc::new(Semaphore::new(settings.max_concurrent_requests.max(1)))
    });

    #[cfg(feature = "experimental-tls")]
    let maybe_tls_acceptor = build_tls_acceptor(settings.tls.as_ref()).await?;

    loop {
        let (stream, peer_addr) = listener.accept().await?;

        // Apply socket options. If this fails for a single connection, log and skip
        // that connection rather than terminating the entire server.
        let stream = match socket_options::apply_socket_options(
            stream,
            settings.tcp_nodelay,
            settings.tcp_keepalive,
            settings.tcp_keepalive_interval,
            settings.tcp_keepalive_retries,
        ) {
            Ok(s) => s,
            Err(e) => {
                otap_df_telemetry::otel_warn!(
                    "HttpSocketOptionsFailed",
                    peer = peer_addr.to_string(),
                    error = e.to_string(),
                    message = "Failed to apply socket options to connection, skipping"
                );
                continue; // Skip this connection, continue accepting others
            }
        };

        let handler = HttpHandler {
            effect_handler: effect_handler.clone(),
            ack_registry: ack_registry.clone(),
            metrics: metrics.clone(),
            settings: settings.clone(),
            semaphore: semaphore.clone(),
        };

        #[cfg(feature = "experimental-tls")]
        {
            if let Some(acceptor) = maybe_tls_acceptor.clone() {
                _ = tokio::spawn(async move {
                    let tls_stream = match acceptor.accept(stream).await {
                        Ok(s) => s,
                        Err(_) => return,
                    };
                    let io = TokioIo::new(tls_stream);
                    let conn = hyper::server::conn::http1::Builder::new()
                        .serve_connection(io, service_fn(move |req| handler.clone().handle(req)));
                    let _ = conn.await;
                });
                continue;
            }
        }

        _ = tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handler.clone().handle(req)));
            let _ = conn.await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_paths() {
        assert_eq!(map_path_to_signal("/v1/logs"), Some(SignalType::Logs));
        assert_eq!(map_path_to_signal("/v1/metrics"), Some(SignalType::Metrics));
        assert_eq!(map_path_to_signal("/v1/traces"), Some(SignalType::Traces));
        assert_eq!(map_path_to_signal("/nope"), None);
    }
}
