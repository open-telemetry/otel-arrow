// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP exporter via HTTP
//!
//! This exporter sends telemetry data to an OTLP server using the HTTP Protocol.
//!
//! ToDo:
//! - Proxy settings
//! - JSON encoding payloads (currently only proto is supported and it's not configurable)
//! - Unit test metrics reporting

use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use http::{HeaderMap, HeaderValue, StatusCode};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error as EngineError, ExporterErrorKind};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::wiring_contract::WiringContract;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
#[cfg(test)]
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::otlp::logs::LogsProtoBytesEncoder;
use otap_df_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otap_df_pdata::otlp::traces::TracesProtoBytesEncoder;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::{
    ExportLogsPartialSuccess, ExportLogsServiceResponse,
};
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::{
    ExportMetricsPartialSuccess, ExportMetricsServiceResponse,
};
use otap_df_pdata::proto::opentelemetry::collector::trace::v1::{
    ExportTracePartialSuccess, ExportTraceServiceResponse,
};
use otap_df_pdata::{OtapPayload, OtapPayloadHelpers};
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
use otap_df_telemetry::metrics::MeasurementMetricSet;
use otap_df_telemetry::{otel_debug, otel_info, otel_warn};
use prost::Message as _;
use reqwest::{Client, Response};
use secrecy::ExposeSecret;

use self::config::Config;
use crate::exporters::otlp_grpc_exporter::InFlightExports;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterPDataExportMetrics;
use otap_df_otap::otlp_http::client_settings::{HttpClientError, HttpClientSettings};
use otap_df_otap::otlp_http::{LOGS_PATH, METRICS_PATH, PROTOBUF_CONTENT_TYPE, TRACES_PATH};
use otap_df_otap::pdata::{Context, OtapPdata};

use self::bearer_auth::BearerAuth;

mod bearer_auth;
mod config;

/// The URN for the OTLP HTTP exporter
pub const OTLP_HTTP_EXPORTER_URN: &str = "urn:otel:exporter:otlp_http";

/// Exporter that sends OTLP data via HTTP
pub struct OtlpHttpExporter {
    config: Config,
    pdata_metrics: MeasurementMetricSet<ExporterPDataExportMetrics>,
    /// Optional bearer token provider resolved from the
    /// `bearer_token_provider` capability. When bound, a fresh
    /// `Authorization: Bearer <token>` is injected on every outgoing
    /// request; when absent, the exporter behaves exactly as before.
    token_provider: Option<Box<dyn BearerTokenProvider>>,
}

/// Declare the OTLP HTTP Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTLP_HTTP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTLP_HTTP_EXPORTER_URN,
    create: factory_create,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config,
};

/// Validates the OTLP HTTP exporter configuration at config load time.
///
/// Runs before any node is started (initial load and live reconfigure), so bad
/// configuration is rejected fast and attributed to the offending node rather
/// than surfacing as an opaque client error at startup.
fn validate_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    let cfg: Config =
        serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?;
    cfg.http
        .validate()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(())
}

fn factory_create(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &ExporterConfig,
    capabilities: &otap_df_engine::capability::registry::Capabilities,
) -> Result<ExporterWrapper<OtapPdata>, ConfigError> {
    // Optionally resolve a bound bearer token provider. Absent binding keeps the
    // default (no-auth) behavior; a bound provider (e.g. the `azure_identity_auth`
    // extension) supplies refreshed OAuth tokens.
    let token_provider = capabilities
        .optional_local::<otap_df_engine::capability::auth::bearer_token_provider::BearerTokenProvider>()
        .map_err(|e| ConfigError::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(ExporterWrapper::local(
        OtlpHttpExporter::from_config(pipeline, &node_config.config, token_provider)?,
        node,
        node_config,
        exporter_config,
    ))
}

impl OtlpHttpExporter {
    /// create a new instance of the `[OtlpHttpExporter]` from json config value
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
        token_provider: Option<Box<dyn BearerTokenProvider>>,
    ) -> Result<Self, ConfigError> {
        let pdata_metrics = ExporterPDataExportMetrics::register(&pipeline_ctx);

        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // validate the endpoint URL
        _ = reqwest::Url::parse(&config.endpoint).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("invalid endpoint URL: {e}"),
        })?;

        // validate the endpoint overrides if supplied
        if let Some(endpoint) = config.logs_endpoint.as_ref() {
            _ = reqwest::Url::parse(endpoint).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid logs endpoint URL: {e}"),
            })?;
        }
        if let Some(endpoint) = config.metrics_endpoint.as_ref() {
            _ = reqwest::Url::parse(endpoint).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid metrics endpoint URL: {e}"),
            })?;
        }
        if let Some(endpoint) = config.traces_endpoint.as_ref() {
            _ = reqwest::Url::parse(endpoint).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid traces endpoint URL: {e}"),
            })?;
        }

        if let Some(tls) = &config.http.tls {
            // server_name not currently supported
            if let Some(_server_name) = &tls.server_name {
                return Err(ConfigError::InvalidUserConfig {
                    error: "TLS configuration error: server_name_override is not supported by \
                    the current Rust OTLP HTTP client implementation (reqwest/rustls) remove \
                    server_name_override."
                        .into(),
                });
            }

            if let Some(true) = tls.insecure {
                // Keeping with the same behaviour in the golang collector: if this is
                // configured, but the endpoints are start with https, we still send the
                // request using https. Just warn about the ignored config mismatch.
                let wants_https = config.endpoint.starts_with("https://")
                    || config
                        .logs_endpoint
                        .as_ref()
                        .map(|e| e.starts_with("https://"))
                        .unwrap_or(false)
                    || config
                        .metrics_endpoint
                        .as_ref()
                        .map(|e| e.starts_with("https://"))
                        .unwrap_or(false)
                    || config
                        .traces_endpoint
                        .as_ref()
                        .map(|e| e.starts_with("https://"))
                        .unwrap_or(false);
                if wants_https {
                    otel_warn!(
                        "otlp.exporter.http.validate_insecure_flag",
                        message = "config setting http.tls.insecure = true is ignored. \
                            requests will still be sent with TLS to endpoints configured \
                            with scheme https"
                    )
                }
            }
        }

        Ok(Self {
            config,
            pdata_metrics,
            token_provider,
        })
    }
}

#[derive(Debug)]
struct CompletedExport {
    result: Result<ServiceResponse, ServiceRequestError>,
    context: Context,
    saved_payload: OtapPayload,
    signal_type: SignalType,
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for OtlpHttpExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, EngineError> {
        let logs_endpoint = Rc::new(
            self.config
                .logs_endpoint
                .clone()
                .unwrap_or(format!("{}{}", self.config.endpoint, LOGS_PATH)),
        );
        let metrics_endpoint = Rc::new(
            self.config
                .metrics_endpoint
                .clone()
                .unwrap_or(format!("{}{}", self.config.endpoint, METRICS_PATH)),
        );
        let traces_endpoint = Rc::new(
            self.config
                .traces_endpoint
                .clone()
                .unwrap_or(format!("{}{}", self.config.endpoint, TRACES_PATH)),
        );

        otel_info!(
            "otlp.exporter.http.start",
            logs_endpoint = logs_endpoint.as_str(),
            metrics_endpoint = metrics_endpoint.as_str(),
            traces_endpoint = traces_endpoint.as_str(),
        );

        let max_in_flight = self.config.max_in_flight.max(1);
        let mut client_pool =
            HttpClientPool::try_new(&self.config.http, self.config.client_pool_size)
                .await
                .map_err(|e| EngineError::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    kind: ExporterErrorKind::Configuration,
                    error: "unable to initialize HTTP client pool".into(),
                    source_detail: e.to_string(),
                })?;

        let mut inflight_exports = InFlightExports::new();

        let mut logs_proto_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_proto_encoder = TracesProtoBytesEncoder::new();
        let mut proto_buffer = ProtoBuffer::default();

        let compression = self.config.http.compression();
        // Buffer to hold compressed bytes. We re-use this scratch space to place
        // the compressed bytes and then allocate an exact sized Bytes to copy
        // them into for exporting.
        //
        // The rationale is that this buffer will eventually settle on a size
        // that fits our requests and we won't have to constantly realloc while
        // compressing. So in theory we always do one alloc + copy.
        //
        // We could alternatively create a new buffer every time, but then
        // we'll do some number of reallocs + copy to find the right final
        // buffer size.
        let mut compressed_buffer: Vec<u8> = Vec::new();

        // Consumer-side bearer-token adapter, if a provider is bound. It owns
        // the token subscription, the cached `Authorization` header, and token
        // usability; the loop below stays auth-agnostic -- it only asks whether
        // it may send and stamps the header the adapter hands back.
        let mut auth = self.token_provider.take().map(BearerAuth::new);
        // Constant for the whole run (the adapter is created once), so precompute
        // it for the auth-aware retry decision in `finalize_completed_export`.
        let auth_bound = auth.is_some();

        loop {
            // Admit pdata only when auth is ready (a usable token is cached, or no
            // provider is bound) and we are below the in-flight cap. While a bound
            // provider has no usable token we stop pulling pdata, so it
            // back-pressures upstream instead of being accepted and NACK'd. A token
            // is guaranteed to eventually arrive -- the extension's readiness probe
            // holds data-path startup until the first publish, and its watch stream
            // stays live while we hold the provider handle -- so waiting (not
            // dropping) is always correct here.
            let accepting_pdata = auth.as_ref().is_none_or(BearerAuth::is_ready)
                && inflight_exports.len() < max_in_flight;

            let msg = tokio::select! {
                biased;

                // Pick up token refreshes (initial + subsequent) even while pdata
                // intake is gated, so a pending token can arrive and unblock us.
                // The `async` block keeps this lazy: `select!` evaluates a branch
                // expression even when its `if` guard is false, and `auth` is
                // `None` when no provider is bound. The `None` arm is unreachable
                // while the guard holds; it pends rather than panics.
                () = async {
                    match auth.as_mut() {
                        Some(a) => a.poll_refresh().await,
                        None => std::future::pending().await,
                    }
                }, if auth.as_ref().is_some_and(BearerAuth::is_active) => {
                    // A refresh was drained (the adapter caches it and logs any
                    // anomaly); loop to re-evaluate intake readiness.
                    continue;
                }

                // Drain a finished export. Guarded because an empty in-flight set
                // is immediately ready (`FuturesUnordered::next` yields `None`),
                // which would otherwise busy-loop.
                completed = inflight_exports.next_completion(), if !inflight_exports.is_empty() => {
                    if let Some(completed) = completed {
                        finalize_completed_export(
                            completed,
                            &effect_handler,
                            &mut self.pdata_metrics,
                            auth_bound,
                        )
                        .await;
                    }
                    continue;
                }

                // Inbound message. Control always flows; pdata is admitted only
                // when `accepting_pdata` (and force-drained during shutdown).
                msg = msg_chan.recv_when(accepting_pdata) => {
                    let msg = msg?;
                    otel_debug!("otlp.exporter.http.receive");
                    msg
                }
            };

            match msg {
                Message::Control(NodeControlMsg::Shutdown { deadline, reason }) => {
                    otel_info!("otlp.exporter.http.shutdown", reason = reason);
                    while !inflight_exports.is_empty() {
                        if let Some(completed) = inflight_exports.next_completion().await {
                            finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.pdata_metrics,
                                auth_bound,
                            )
                            .await;
                        }
                    }
                    return Ok(TerminalState::new(
                        deadline,
                        self.pdata_metrics.terminal_snapshots(),
                    ));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => _ = metrics_reporter.report_measurement(&mut self.pdata_metrics),
                Message::PData(pdata) => {
                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();

                    // We normally only reach here with a usable token, since intake
                    // is gated on `accepting_pdata`. The exception is shutdown, which
                    // force-drains buffered pdata even while auth was pending: with no
                    // usable token we cannot send, so NACK it as retryable -- a token
                    // may yet arrive, so nothing is dropped.
                    if let Some(a) = auth.as_ref() {
                        if !a.is_ready() {
                            let mut nack = NackMsg::new(
                                a.not_ready_reason(),
                                OtapPdata::new(context, payload),
                            );
                            nack.permanent = false;
                            _ = effect_handler.notify_nack(nack).await;
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .messages
                                .inc();
                            continue;
                        }
                    }

                    // The cached bearer header, cloned per request, takes
                    // precedence over any statically configured `authorization`.
                    let auth_header = auth.as_ref().and_then(BearerAuth::header);

                    // For the OtapArrowRecords path we keep the uncompressed bytes in
                    // `proto_buffer` rather than materializing them into a `Bytes` up front: when
                    // compression is enabled we feed the slice directly into the encoder, avoiding
                    // an alloc+memcpy of the full uncompressed payload.
                    enum Uncompressed {
                        // Already a refcounted Bytes (OtlpBytes path).
                        Bytes(Bytes),
                        // Lives in `proto_buffer` (OtapArrowRecords path).
                        InProtoBuffer,
                    }

                    // proto encode the payload into the request body, while keeping a copy of the
                    // original payload if the context allows it to be returned.
                    let (uncompressed, saved_payload) = match payload {
                        OtapPayload::OtlpBytes(mut otlp_bytes) => {
                            if context.may_return_payload() {
                                // use cheap clone of bytes as the request body
                                let body = otlp_bytes.clone_bytes();
                                (Uncompressed::Bytes(body), otlp_bytes.into())
                            } else {
                                // take the bytes and replace with empty bytes in original payload
                                let body = otlp_bytes.replace_bytes(Bytes::new());
                                (Uncompressed::Bytes(body), otlp_bytes.into())
                            }
                        }
                        OtapPayload::OtapArrowRecords(mut otap_batch) => {
                            // encode the OTAP batch as protobuf request body
                            proto_buffer.clear();
                            let encode_result =
                                match signal_type {
                                    SignalType::Logs => logs_proto_encoder
                                        .encode(&mut otap_batch, &mut proto_buffer),
                                    SignalType::Metrics => metrics_proto_encoder
                                        .encode(&mut otap_batch, &mut proto_buffer),
                                    SignalType::Traces => traces_proto_encoder
                                        .encode(&mut otap_batch, &mut proto_buffer),
                                };

                            if !context.may_return_payload() {
                                // drop the original OTAP batch if the context indicates it
                                // does not wish it to be returned
                                _ = otap_batch.take_payload();
                            }

                            if let Err(e) = encode_result {
                                // encoding error, we must have received an invalid structured batch
                                let mut nack = NackMsg::new(
                                    e.to_string(),
                                    OtapPdata::new(context, otap_batch.into()),
                                );
                                nack.permanent = true;
                                _ = effect_handler.notify_nack(nack).await;
                                self.pdata_metrics
                                    .with(SignalOutcomeAttributes {
                                        signal: signal_type,
                                        outcome: Outcome::Failure,
                                    })
                                    .messages
                                    .inc();
                                continue;
                            }

                            (Uncompressed::InProtoBuffer, otap_batch.into())
                        }
                    };

                    let body = match compression {
                        Some(method) => {
                            let uncompressed_slice: &[u8] = match &uncompressed {
                                Uncompressed::Bytes(b) => b.as_ref(),
                                Uncompressed::InProtoBuffer => proto_buffer.as_ref(),
                            };
                            if let Err(e) =
                                method.encode(uncompressed_slice, &mut compressed_buffer)
                            {
                                let mut nack = NackMsg::new(
                                    e.to_string(),
                                    OtapPdata::new(context, saved_payload),
                                );
                                nack.permanent = true;
                                _ = effect_handler.notify_nack(nack).await;
                                self.pdata_metrics
                                    .with(SignalOutcomeAttributes {
                                        signal: signal_type,
                                        outcome: Outcome::Failure,
                                    })
                                    .messages
                                    .inc();
                                continue;
                            }
                            Bytes::copy_from_slice(&compressed_buffer)
                        }
                        None => match uncompressed {
                            Uncompressed::Bytes(b) => b,
                            Uncompressed::InProtoBuffer => {
                                Bytes::copy_from_slice(proto_buffer.as_ref())
                            }
                        },
                    };

                    let endpoint: Rc<String> = Rc::clone(match signal_type {
                        SignalType::Logs => &logs_endpoint,
                        SignalType::Metrics => &metrics_endpoint,
                        SignalType::Traces => &traces_endpoint,
                    });

                    let max_response_body_len = self.config.max_response_body_length;

                    let client = client_pool.get_client();
                    inflight_exports.push(async move {
                        let mut req = client.post(endpoint.as_str()).body(body);
                        if let Some(auth) = auth_header {
                            // A per-request header takes precedence over the
                            // client's default headers, so the refreshed bearer
                            // token overrides any statically configured
                            // `authorization` credential.
                            req = req.header(http::header::AUTHORIZATION, auth);
                        }
                        if let Some(method) = compression {
                            req = req.header(
                                http::header::CONTENT_ENCODING,
                                method.as_http_content_encoding(),
                            );
                        }
                        let result = req.send().await;

                        CompletedExport {
                            result: query_result_to_service_response(
                                &signal_type,
                                max_response_body_len,
                                result,
                            )
                            .await,
                            context,
                            saved_payload,
                            signal_type,
                        }
                    })
                }
                _ => {
                    // ignore unhandled messages
                }
            }
        }
    }
}

#[derive(Debug)]
struct ServiceResponse {
    partial_success: Option<PartialSuccess>,
}

#[derive(Debug)]
struct PartialSuccess {
    rejected: i64,
    error_message: String,
}

impl From<ExportLogsPartialSuccess> for PartialSuccess {
    fn from(value: ExportLogsPartialSuccess) -> Self {
        Self {
            rejected: value.rejected_log_records,
            error_message: value.error_message,
        }
    }
}

impl From<ExportMetricsPartialSuccess> for PartialSuccess {
    fn from(value: ExportMetricsPartialSuccess) -> Self {
        Self {
            rejected: value.rejected_data_points,
            error_message: value.error_message,
        }
    }
}

impl From<ExportTracePartialSuccess> for PartialSuccess {
    fn from(value: ExportTracePartialSuccess) -> Self {
        Self {
            rejected: value.rejected_spans,
            error_message: value.error_message,
        }
    }
}

impl From<ExportLogsServiceResponse> for ServiceResponse {
    fn from(value: ExportLogsServiceResponse) -> Self {
        Self {
            partial_success: value.partial_success.map(Into::into),
        }
    }
}

impl From<ExportMetricsServiceResponse> for ServiceResponse {
    fn from(value: ExportMetricsServiceResponse) -> Self {
        Self {
            partial_success: value.partial_success.map(Into::into),
        }
    }
}

impl From<ExportTraceServiceResponse> for ServiceResponse {
    fn from(value: ExportTraceServiceResponse) -> Self {
        Self {
            partial_success: value.partial_success.map(Into::into),
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum ServiceRequestError {
    #[error("An error occurred sending HTTP request: {err}{}", format_source(err))]
    RequestError {
        #[from]
        err: reqwest::Error,
    },

    #[error("An error occurred decoding response body: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("Response body size {body_size} exceeds maximum allowed size of {max_size} bytes")]
    BodyTooLarge { body_size: usize, max_size: usize },
}

impl ServiceRequestError {
    fn is_retryable(&self) -> bool {
        match self {
            Self::RequestError { err: req_err } => {
                match req_err.status() {
                    Some(status) => {
                        // we received a non-200 response. The OTLP HTTP spec defines certain
                        // status codes for which the client may retry the request
                        // https://opentelemetry.io/docs/specs/otlp/#retryable-response-codes
                        status == StatusCode::TOO_MANY_REQUESTS
                            || status == StatusCode::BAD_GATEWAY
                            || status == StatusCode::SERVICE_UNAVAILABLE
                            || status == StatusCode::GATEWAY_TIMEOUT
                    }
                    None => {
                        // we've encountered some other kind of error sending the request. For
                        // example, maybe there was connection refused, the server disconnected
                        // without sending a response, or there was non HTTP timeout.
                        //
                        // The OTLP spec isn't entirely clear on what to do here, but it does
                        // instruct to adhere to HTTP spec and explicitly states to retry on
                        // server disconnects
                        // https://opentelemetry.io/docs/specs/otlp/#all-other-responses
                        //
                        // we'll do something reasonable here and retry on these errors which
                        // may be transient, network related
                        req_err.is_connect() || req_err.is_timeout()
                    }
                }
            }

            Self::BodyTooLarge { .. } | ServiceRequestError::DecodeError(_) => {
                // these errors happen when we've received a 200 response, but for some reason
                // were unable to deserialize the response body.
                //
                // this indicates either a full success, or partial success. In either case, we
                // shouldn't retry. The spec explicitly states this for partial success
                // https://opentelemetry.io/docs/specs/otlp/#partial-success-1
                false
            }
        }
    }

    /// Whether this is an HTTP auth-failure response (401/403). When a bearer
    /// token provider is bound these are treated as retryable, because they
    /// usually mean the cached token lapsed or a refresh raced; the batch can
    /// succeed once a fresh token is in use.
    fn is_auth_failure(&self) -> bool {
        matches!(
            self,
            Self::RequestError { err }
                if matches!(
                    err.status(),
                    Some(StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
                )
        )
    }
}

fn format_source(e: &reqwest::Error) -> String {
    use std::error::Error;
    match e.source() {
        Some(src) => match src.source() {
            Some(inner_src) => {
                format!(": {src}: {inner_src}")
            }
            None => {
                format!(": {src}")
            }
        },
        None => String::new(),
    }
}

async fn query_result_to_service_response(
    signal_type: &SignalType,
    max_response_body_len: usize,
    result: Result<Response, reqwest::Error>,
) -> Result<ServiceResponse, ServiceRequestError> {
    let resp = result?.error_for_status()?;
    let mut body = collect_body(resp, max_response_body_len).await?;

    let service_resp = match signal_type {
        SignalType::Logs => ExportLogsServiceResponse::decode(&mut body).map(Into::into),
        SignalType::Metrics => ExportMetricsServiceResponse::decode(&mut body).map(Into::into),
        SignalType::Traces => ExportTraceServiceResponse::decode(&mut body).map(Into::into),
    };

    Ok(service_resp?)
}

async fn collect_body(response: Response, max_len: usize) -> Result<Bytes, ServiceRequestError> {
    let content_length = response.content_length().unwrap_or(0) as usize;
    if content_length > max_len {
        return Err(ServiceRequestError::BodyTooLarge {
            body_size: content_length,
            max_size: max_len,
        });
    }
    let mut buf = BytesMut::with_capacity(content_length);
    let mut stream = response.bytes_stream();

    let mut remaining = max_len;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;

        if chunk.len() > remaining {
            return Err(ServiceRequestError::BodyTooLarge {
                body_size: max_len - remaining + chunk.len(),
                max_size: max_len,
            });
        }

        remaining -= chunk.len();
        buf.extend_from_slice(&chunk);
    }

    Ok(buf.freeze())
}

async fn finalize_completed_export(
    completed: CompletedExport,
    effect_handler: &EffectHandler<OtapPdata>,
    pdata_metrics: &mut MeasurementMetricSet<ExporterPDataExportMetrics>,
    auth_bound: bool,
) {
    let CompletedExport {
        result,
        context,
        saved_payload,
        signal_type,
    } = completed;

    let pdata = OtapPdata::new(context, saved_payload);

    let err = match result {
        Ok(service_resp) => service_resp.partial_success.and_then(|partial_success| {
            // As per OTLP HTTP spec, the server may use partial success to convey information
            // even in the case where it fully accepts the request. In these cases, it MUST have
            // set the rejected_<signal> field to 0. We'll treat this case as a success
            if partial_success.rejected == 0 {
                otel_debug!(
                    "otlp.exporter.http.zero_partial_rejected",
                    details = partial_success.error_message
                );

                None
            } else {
                // In the case we received a partial_success, the spec states that the request
                // should not be retried.
                // https://opentelemetry.io/docs/specs/otlp/#partial-success-1
                let retryable = false;
                Some((
                    format!(
                        "{} ({} rejected)",
                        partial_success.error_message, partial_success.rejected
                    ),
                    retryable,
                ))
            }
        }),
        Err(e) => {
            // With a bearer token provider bound, a 401/403 usually means the
            // cached token lapsed or a refresh raced; retry rather than drop, so
            // the batch can succeed once a fresh token is in use.
            let retryable = e.is_retryable() || (auth_bound && e.is_auth_failure());
            Some((e.to_string(), retryable))
        }
    };

    let export_and_notify_success = match err {
        None => effect_handler.notify_ack(AckMsg::new(pdata)).await.is_ok(),
        Some((err_msg, retryable)) => {
            otel_warn!(
                "otlp.exporter.http.export_error",
                message = err_msg,
                retryable = retryable
            );
            let mut nack = NackMsg::new(&err_msg, pdata);
            nack.permanent = !retryable;
            _ = effect_handler.notify_nack(nack).await;
            false
        }
    };

    let outcome = if export_and_notify_success {
        Outcome::Success
    } else {
        Outcome::Failure
    };
    pdata_metrics
        .with(SignalOutcomeAttributes {
            signal: signal_type,
            outcome,
        })
        .messages
        .inc();
}

/// A simple pool of HTTP clients to allow for concurrent exports.
///
/// Then intention here is to force requests to be distributed across multiple TCP connections to
/// the OTLP server. In the case of our OTLP Receiver, there may be many instances each running on
/// different threads, but all listening on the same port (using SO_REUSEPORT). Having multiple
/// connections helps to balance requests across any receiver instances.
///
/// Note that internally, reqwest's Client already manages a pool of connections, so the number of
/// connections is not guaranteed to be equal to the number of clients.
struct HttpClientPool {
    next_client: usize,
    pool: Vec<Client>,
}

impl HttpClientPool {
    /// Builds the user-configured static request headers, marking every value
    /// sensitive so it is redacted in any `HeaderMap` `Debug` output and
    /// excluded from HTTP/2 HPACK indexing. This mirrors the OTLP/gRPC
    /// `GrpcClientSettings::build_static_metadata` path, which marks its
    /// metadata values sensitive for the same reasons.
    ///
    /// `reserve_extra` pre-sizes the returned map for additional headers the
    /// caller will insert afterwards (e.g. the protocol `Content-Type` /
    /// `Accept` headers), so construction stays single-allocation.
    ///
    /// Header names/values are validated up front by
    /// [`HttpClientSettings::validate`], so for a validated config the per-entry
    /// parse below cannot fail; the fallible path defends against a programmatic
    /// caller that bypassed validation.
    fn build_static_headers(
        client_settings: &HttpClientSettings,
        reserve_extra: usize,
    ) -> Result<HeaderMap, HttpClientError> {
        let mut headers = HeaderMap::with_capacity(client_settings.headers.len() + reserve_extra);
        for (name, value) in &client_settings.headers {
            let header_name = http::HeaderName::from_bytes(name.as_bytes()).map_err(|e| {
                HttpClientError::InvalidConfig(format!("invalid header name \"{name}\": {e}"))
            })?;
            let mut header_value = HeaderValue::from_str(value.expose_secret()).map_err(|e| {
                HttpClientError::InvalidConfig(format!("invalid value for header \"{name}\": {e}"))
            })?;
            // Mark the value sensitive so it is redacted in any `HeaderMap`
            // `Debug` output and excluded from HTTP/2 HPACK indexing.
            header_value.set_sensitive(true);
            _ = headers.insert(header_name, header_value);
        }
        Ok(headers)
    }

    async fn try_new(
        client_settings: &HttpClientSettings,
        pool_size: NonZeroUsize,
    ) -> Result<Self, HttpClientError> {
        // Build the user-configured static headers first (pre-sized for the two
        // protocol headers added below) so the protocol headers always win on
        // any (already validated against) collision.
        let mut default_headers = Self::build_static_headers(client_settings, 2)?;

        // TODO eventually this header value should be dynamic once we support JSON OTLP payloads
        _ = default_headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static(PROTOBUF_CONTENT_TYPE),
        );
        _ = default_headers.insert(
            http::header::ACCEPT,
            HeaderValue::from_static(PROTOBUF_CONTENT_TYPE),
        );

        let pool_size: usize = pool_size.into();
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            let client_builder = client_settings
                .client_builder()
                .await?
                .default_headers(default_headers.clone());
            pool.push(client_builder.build()?);
        }

        Ok(Self {
            pool,
            next_client: 0,
        })
    }

    fn get_client(&mut self) -> Client {
        let client = self.pool[self.next_client].clone(); // clones Arc
        self.next_client += 1;
        if self.next_client >= self.pool.len() {
            self.next_client = 0;
        }

        client
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    use arrow::array::Int32Array;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use http_body_util::Full;
    use hyper::Response;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use otap_df_config::PortName;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{PipelineCompletionMsg, runtime_ctrl_msg_channel};
    use otap_df_engine::shared::message::SharedSender;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::node::test_node;
    use otap_df_pdata::OtapArrowRecords;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::otap::Logs;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Metric, MetricsData, ResourceMetrics, ScopeMetrics,
    };
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, TracesData};
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use otap_df_pdata::testing::round_trip::otlp_to_otap;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;

    use parking_lot::lock_api::Mutex;
    use prost::Message;
    use tokio::runtime::Runtime;
    use tokio_util::sync::CancellationToken;
    use tokio_util::task::TaskTracker;

    use {
        otap_df_config::tls::{TlsClientConfig, TlsConfig, TlsServerConfig},
        otap_test_tls_certs::{ExtendedKeyUsage, generate_ca},
        tempfile::TempDir,
    };

    use super::*;

    use otap_df_otap::otap_grpc::common::AckRegistry;
    use otap_df_otap::otlp_http::client_settings::HttpClientSettings;
    use otap_df_otap::otlp_http::{HttpServerSettings, serve, tune_max_concurrent_requests};
    use otap_df_otap::otlp_metrics::OtlpReceiverMetrics;
    use otap_df_otap::testing::TestCallData;

    /// run test HTTP server serving OTLP HTTP API. Internally, this uses the OTLP HTTP server that
    /// is used in OTLP Receiver. This returns a cancellation token (to shutdown the server when
    /// the test is finished), and a receiver that will emit any pdata that the server produces.
    fn run_server(
        tokio_rt: &Runtime,
        pipeline_ctx: &PipelineContext,
        endpoint_addr: &str,
    ) -> (tokio::sync::mpsc::Receiver<OtapPdata>, CancellationToken) {
        let server_node_id = test_node("test-server");
        let port_name = PortName::from("server_out");
        let mut msg_senders = HashMap::new();
        let (pdata_tx, pdata_rx) = tokio::sync::mpsc::channel(10);
        _ = msg_senders.insert(port_name.clone(), SharedSender::mpsc(pdata_tx));

        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(10);
        let (_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(5);
        let server_effect_handler =
            otap_df_engine::shared::receiver::EffectHandler::<OtapPdata>::new(
                server_node_id,
                msg_senders,
                Some(port_name),
                runtime_ctrl_msg_tx,
                metrics_reporter,
            );

        let mut server_settings = HttpServerSettings {
            listening_addr: endpoint_addr.parse().unwrap(),
            ..Default::default()
        };
        tune_max_concurrent_requests(&mut server_settings, 1);

        let ack_registry = AckRegistry::new(None, None, None);
        let server_metrics = OtlpReceiverMetrics::register(pipeline_ctx);
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();

        _ = tokio_rt.spawn(async move {
            serve(
                server_effect_handler,
                server_settings,
                ack_registry,
                Arc::new(Mutex::new(server_metrics)),
                otap_df_engine::memory_limiter::SharedReceiverAdmissionState::default(),
                None,
                server_cancellation_token,
            )
            .await
        });

        // Wait for the server to be ready to accept connections. Without this,
        // the exporter may attempt to connect before the server has bound,
        // leading to a retryable NACK and the test hanging in validation.
        wait_for_port_ready(endpoint_addr);

        (pdata_rx, server_cancellation_token2)
    }

    /// run an http server that returns error for any request
    ///
    /// if `status_err` is Some, server will return this status code with empty body
    /// if `status_err` is false, server will return 200 status code with body that
    /// indicates only a partial success
    fn run_error_server(
        tokio_rt: &Runtime,
        endpoint_addr: &str,
        status_err: Option<u16>,
    ) -> CancellationToken {
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();
        let endpoint_addr = endpoint_addr.to_string();
        _ = tokio_rt.spawn(async move {
            serve_errors(endpoint_addr, server_cancellation_token, status_err).await
        });

        server_cancellation_token2
    }

    async fn serve_errors(
        endpoint_addr: String,
        shutdown_token: CancellationToken,
        status_err: Option<u16>,
    ) {
        let listener = tokio::net::TcpListener::bind(endpoint_addr).await.unwrap();
        let tracker = TaskTracker::new();
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => break,
                accept_result = listener.accept() => {
                    let (stream, peer_addr) = accept_result.unwrap();
                    let shutdown_token = shutdown_token.clone();
                    drop(tracker.spawn(async move {
                        let io = TokioIo::new(stream);
                        let conn = http1::Builder::new().serve_connection(io, service_fn(|req| async move {
                            if let Some(status) = status_err {

                                Ok::<_, hyper::Error>(Response::builder()
                                    .status(status)
                                    .body(Full::new(Bytes::from("".as_bytes().to_vec())))
                                    .unwrap())
                            } else {
                                let mut body = Vec::new();
                                let uri = req.uri();
                                match uri.path() {
                                    LOGS_PATH => {
                                        let service_resp = ExportLogsServiceResponse {
                                            partial_success: Some(ExportLogsPartialSuccess {
                                                rejected_log_records: 1,
                                                error_message: "partial success error".into(),
                                            }),
                                        };
                                        service_resp.encode(&mut body).unwrap();
                                    }
                                    METRICS_PATH => {
                                        let service_resp = ExportMetricsServiceResponse {
                                            partial_success: Some(ExportMetricsPartialSuccess {
                                                rejected_data_points: 1,
                                                error_message: "partial success error".into(),
                                            }),
                                        };
                                        service_resp.encode(&mut body).unwrap();
                                    }
                                    TRACES_PATH => {
                                        let service_resp = ExportTraceServiceResponse {
                                            partial_success: Some(ExportTracePartialSuccess {
                                                rejected_spans: 1,
                                                error_message: "partial success error".into(),
                                            }),
                                        };
                                        service_resp.encode(&mut body).unwrap();
                                    }
                                    _ => {
                                        panic!("unexpected path: {uri}");
                                    }
                                }
                                Ok(Response::builder()
                                    .status(200)
                                    .body(Full::new(Bytes::from(body)))
                                    .unwrap())
                            }
                        }));
                        let mut conn = std::pin::pin!(conn);

                        tokio::select!{
                            _ = shutdown_token.cancelled() => {
                                conn.as_mut().graceful_shutdown();
                                let _ = conn.await;
                            },
                            conn_result = &mut conn => {
                                if let Err(e) = conn_result {
                                    eprintln!("Error serving connection from {peer_addr}: {e}");
                                }
                            }
                         }
                    }));
                }
            }
        }

        let _ = tracker.close();
    }

    /// Polls a TCP port until it accepts connections, or panics after 5 seconds.
    fn wait_for_port_ready(addr: &str) {
        let addr: std::net::SocketAddr = addr.parse().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if Instant::now() >= deadline {
                panic!("Server did not become ready within 5 seconds on {addr}");
            }
            match std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(50)) {
                Ok(_) => return,
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    /// Runs a minimal HTTP server that records the headers of the first request
    /// it receives into `captured` and always replies `200 OK`. Used to assert
    /// what the exporter actually puts on the wire.
    fn run_header_capture_server(
        tokio_rt: &Runtime,
        endpoint_addr: &str,
        captured: Arc<parking_lot::Mutex<Option<HeaderMap>>>,
    ) -> CancellationToken {
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();
        let endpoint_addr = endpoint_addr.to_string();
        _ = tokio_rt.spawn(async move {
            let listener = tokio::net::TcpListener::bind(endpoint_addr).await.unwrap();
            let tracker = TaskTracker::new();
            loop {
                tokio::select! {
                    _ = server_cancellation_token.cancelled() => break,
                    accept_result = listener.accept() => {
                        let (stream, _peer_addr) = accept_result.unwrap();
                        let captured = captured.clone();
                        let shutdown_token = server_cancellation_token.clone();
                        drop(tracker.spawn(async move {
                            let io = TokioIo::new(stream);
                            let service = service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                                let captured = captured.clone();
                                async move {
                                    *captured.lock() = Some(req.headers().clone());
                                    Ok::<_, hyper::Error>(Response::builder()
                                        .status(200)
                                        .body(Full::new(Bytes::from(Vec::new())))
                                        .unwrap())
                                }
                            });
                            let conn = http1::Builder::new().serve_connection(io, service);
                            let mut conn = std::pin::pin!(conn);
                            tokio::select! {
                                _ = shutdown_token.cancelled() => {
                                    conn.as_mut().graceful_shutdown();
                                    let _ = conn.await;
                                },
                                conn_result = &mut conn => {
                                    let _ = conn_result;
                                }
                            }
                        }));
                    }
                }
            }
            let _ = tracker.close();
        });

        server_cancellation_token2
    }

    /// Runs a server that answers every request with a fixed HTTP status and an
    /// empty body. Used to drive the auth-failure (401/403) handling path.
    fn run_fixed_status_server(
        tokio_rt: &Runtime,
        endpoint_addr: &str,
        status: u16,
    ) -> CancellationToken {
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();
        let endpoint_addr = endpoint_addr.to_string();
        _ = tokio_rt.spawn(async move {
            let listener = tokio::net::TcpListener::bind(endpoint_addr).await.unwrap();
            let tracker = TaskTracker::new();
            loop {
                tokio::select! {
                    _ = server_cancellation_token.cancelled() => break,
                    accept_result = listener.accept() => {
                        let (stream, _peer_addr) = accept_result.unwrap();
                        let shutdown_token = server_cancellation_token.clone();
                        drop(tracker.spawn(async move {
                            let io = TokioIo::new(stream);
                            let service = service_fn(move |_req: hyper::Request<hyper::body::Incoming>| async move {
                                Ok::<_, hyper::Error>(
                                    Response::builder()
                                        .status(status)
                                        .body(Full::new(Bytes::from(Vec::new())))
                                        .unwrap(),
                                )
                            });
                            let conn = http1::Builder::new().serve_connection(io, service);
                            let mut conn = std::pin::pin!(conn);
                            tokio::select! {
                                _ = shutdown_token.cancelled() => {
                                    conn.as_mut().graceful_shutdown();
                                    let _ = conn.await;
                                },
                                conn_result = &mut conn => {
                                    let _ = conn_result;
                                }
                            }
                        }));
                    }
                }
            }
            let _ = tracker.close();
        });

        server_cancellation_token2
    }

    #[test]
    fn test_auth_failure_is_retryable_when_provider_bound() {
        // With a bearer token provider bound, a 401 from the backend (e.g. the
        // cached token lapsed) must be NACK'd as retryable, not permanently
        // dropped, so the batch can succeed once a fresh token is in use.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let cancel = run_fixed_status_server(&tokio_rt, &endpoint_addr, 401);
        wait_for_port_ready(&endpoint_addr);

        let config = default_test_config(endpoint);
        let test_runtime = TestRuntime::<OtapPdata>::new();
        // A valid (non-expiring) token so the request is actually sent and the
        // server's 401 drives the auth-failure path.
        let exporter =
            exporter_with_provider(&test_runtime, config, MockTokenProvider::new("token"));

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        let pdatas = subscribe_pdatas(
            vec![OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
            ))],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    let msg = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("expected a pipeline completion message");
                    match msg {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            assert!(
                                !nack.permanent,
                                "a 401 with a bound provider must be retryable, not dropped"
                            );
                        }
                        PipelineCompletionMsg::DeliverAck { .. } => {
                            panic!("a 401 response must not Ack")
                        }
                    }
                })
            });

        cancel.cancel();
    }

    #[test]
    fn test_configured_headers_sent_on_wire() {
        use std::collections::HashMap;

        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        let mut headers = HashMap::new();
        _ = headers.insert("authorization".to_string(), "Basic dXNlcjpwYXNz".into());
        _ = headers.insert("x-test".to_string(), "abc".into());
        let settings = HttpClientSettings {
            headers,
            ..Default::default()
        };

        tokio_rt.block_on(async {
            let mut pool = HttpClientPool::try_new(&settings, NonZeroUsize::new(1).unwrap())
                .await
                .expect("failed to build client pool");
            let client = pool.get_client();
            let resp = client
                .post(format!("http://{endpoint_addr}/v1/logs"))
                .body(Vec::new())
                .send()
                .await
                .expect("request failed");
            assert!(resp.status().is_success());
        });

        cancel.cancel();

        let headers = captured
            .lock()
            .clone()
            .expect("server did not capture any request headers");
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Basic dXNlcjpwYXNz"
        );
        assert_eq!(headers.get("x-test").unwrap().to_str().unwrap(), "abc");
        // Protocol headers are still applied and take precedence.
        assert_eq!(
            headers
                .get(http::header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            PROTOBUF_CONTENT_TYPE
        );
    }

    /// Test double for the `BearerTokenProvider` capability with configurable
    /// stream behavior. `tokens` are published on the stream in order; when
    /// `keep_open` is true the stream stays pending after draining them (never
    /// ends), and when false it ends once they are drained (simulating a
    /// provider that closes its stream).
    struct MockTokenProvider {
        tokens: Vec<String>,
        keep_open: bool,
        /// Expiry applied to every published token (`None` = non-expiring).
        expires_on: Option<Instant>,
    }

    impl MockTokenProvider {
        /// A provider that publishes a single non-expiring token and keeps its
        /// stream open.
        fn new(token: &str) -> Self {
            Self {
                tokens: vec![token.to_string()],
                keep_open: true,
                expires_on: None,
            }
        }
    }

    #[async_trait(?Send)]
    impl BearerTokenProvider for MockTokenProvider {
        async fn get_token(
            &self,
        ) -> Result<
            otap_df_engine::capability::auth::BearerToken,
            otap_df_engine::capability::CapabilityError,
        > {
            // Not exercised by the exporter (it consumes `token_stream`); return
            // the first configured token for completeness.
            Ok(otap_df_engine::capability::auth::BearerToken::with_expiry(
                self.tokens.first().cloned().unwrap_or_default(),
                None,
            ))
        }

        fn token_stream(
            &self,
        ) -> otap_df_engine::capability::auth::bearer_token_provider::TokenStream {
            let expires_on = self.expires_on;
            let published = futures::stream::iter(self.tokens.clone().into_iter().map(move |t| {
                otap_df_engine::capability::auth::BearerToken::with_expiry(t, expires_on)
            }));
            if self.keep_open {
                published.chain(futures::stream::pending()).boxed()
            } else {
                published.boxed()
            }
        }
    }

    /// Build an `OtlpHttpExporter` wrapped for the test runtime with a bound
    /// bearer token provider.
    fn exporter_with_provider(
        test_runtime: &TestRuntime<OtapPdata>,
        config: Config,
        provider: MockTokenProvider,
    ) -> ExporterWrapper<OtapPdata> {
        otap_df_otap::crypto::ensure_crypto_provider();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_HTTP_EXPORTER_URN));
        let telemetry_registry_handle = test_runtime.metrics_registry();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            "test_group".into(),
            "test_pipeline".into(),
            0,
            1,
            0,
        );
        ExporterWrapper::local(
            OtlpHttpExporter {
                config,
                pdata_metrics: ExporterPDataExportMetrics::register(&pipeline_ctx),
                token_provider: Some(Box::new(provider)),
            },
            node_id,
            node_config,
            test_runtime.config(),
        )
    }

    #[test]
    fn test_bearer_token_injected_on_wire() {
        // End-to-end proof that a bound `bearer_token_provider` injects a fresh
        // `authorization` bearer token on the outbound request, and that the
        // token takes precedence over a statically configured `authorization`
        // header.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        // Static authorization header the bearer token must override.
        let mut headers = HashMap::new();
        _ = headers.insert("authorization".to_string(), "Basic static".into());
        let config = Config {
            http: HttpClientSettings {
                headers,
                ..Default::default()
            },
            ..default_test_config(endpoint)
        };

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let exporter = exporter_with_provider(
            &test_runtime,
            config,
            MockTokenProvider::new("provider-token"),
        );

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        let pdatas = subscribe_pdatas(
            vec![OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
            ))],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                })
            });

        cancel.cancel();

        let headers = captured
            .lock()
            .clone()
            .expect("server did not capture any request headers");
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Bearer provider-token",
            "the bearer token must reach the HTTP server and override the static header"
        );
    }

    #[test]
    fn test_bearer_token_unavailable_nacks_and_sends_nothing() {
        // Provider is bound but never publishes a token: batches must be NACK'd
        // as retryable and no request may reach the server.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        let config = default_test_config(endpoint);
        let test_runtime = TestRuntime::<OtapPdata>::new();
        // Stream stays open but never yields a token.
        let exporter = exporter_with_provider(
            &test_runtime,
            config,
            MockTokenProvider {
                tokens: vec![],
                keep_open: true,
                expires_on: None,
            },
        );

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        let pdatas = subscribe_pdatas(
            vec![OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
            ))],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    // The single batch must be NACK'd (retryable), never Ack'd.
                    let msg = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("expected a pipeline completion message");
                    match msg {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            assert!(!nack.permanent, "unavailable-token NACK must be retryable");
                            assert!(
                                nack.reason.contains("bearer token unavailable"),
                                "unexpected NACK reason: {}",
                                nack.reason
                            );
                        }
                        PipelineCompletionMsg::DeliverAck { .. } => {
                            panic!("unexpected Ack: no request should have been sent")
                        }
                    }
                })
            });

        cancel.cancel();
        assert!(
            captured.lock().is_none(),
            "no request must reach the server when no token is available"
        );
    }

    #[test]
    fn test_expired_bearer_token_nacks_retryable_while_stream_open() {
        // The provider publishes a token already within the usability margin of
        // expiry and keeps its stream open. The exporter must refuse to send it (a
        // request could outlive the token), NACK retryably (a refresh may still
        // arrive), and send nothing to the server.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        let config = default_test_config(endpoint);
        let test_runtime = TestRuntime::<OtapPdata>::new();
        // Token already expired (expiry in the past, so within the usability margin).
        let exporter = exporter_with_provider(
            &test_runtime,
            config,
            MockTokenProvider {
                tokens: vec!["stale-token".to_string()],
                keep_open: true,
                expires_on: Some(Instant::now()),
            },
        );

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        let pdatas = subscribe_pdatas(
            vec![OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
            ))],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    let msg = pipeline_completion_rx
                        .recv()
                        .await
                        .expect("expected a pipeline completion message");
                    match msg {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            assert!(
                                !nack.permanent,
                                "expired-token NACK must be retryable while the stream is open"
                            );
                            assert!(
                                nack.reason.contains("expiry"),
                                "unexpected NACK reason: {}",
                                nack.reason
                            );
                        }
                        PipelineCompletionMsg::DeliverAck { .. } => {
                            panic!("unexpected Ack: an expired token must not be sent")
                        }
                    }
                })
            });

        cancel.cancel();
        assert!(
            captured.lock().is_none(),
            "no request must reach the server when the only token is expired"
        );
    }

    #[test]
    fn test_invalid_bearer_token_is_skipped_and_valid_used() {
        // A malformed token (header-invalid bytes) is dropped via the error arm
        // (incrementing `auth_token_errors`); the subsequent valid token is what
        // reaches the wire.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        let config = default_test_config(endpoint);
        let test_runtime = TestRuntime::<OtapPdata>::new();
        // First token is header-invalid (embedded newline), second is valid.
        let exporter = exporter_with_provider(
            &test_runtime,
            config,
            MockTokenProvider {
                tokens: vec!["bad\ntoken".to_string(), "good-token".to_string()],
                keep_open: true,
                expires_on: None,
            },
        );

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        let pdatas = subscribe_pdatas(
            vec![OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
            ))],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                })
            });

        cancel.cancel();
        let headers = captured
            .lock()
            .clone()
            .expect("server did not capture any request headers");
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Bearer good-token",
            "the malformed token must be skipped and the valid token used"
        );
    }

    #[test]
    fn test_last_token_reused_after_stream_closes() {
        // The provider publishes one token then closes its stream; subsequent
        // batches must keep using the cached token.
        otap_df_otap::crypto::ensure_crypto_provider();
        let tokio_rt = Runtime::new().unwrap();
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let captured: Arc<parking_lot::Mutex<Option<HeaderMap>>> =
            Arc::new(parking_lot::Mutex::new(None));
        let cancel = run_header_capture_server(&tokio_rt, &endpoint_addr, captured.clone());
        wait_for_port_ready(&endpoint_addr);

        let config = default_test_config(endpoint);
        let test_runtime = TestRuntime::<OtapPdata>::new();
        // One token, then the stream ends.
        let exporter = exporter_with_provider(
            &test_runtime,
            config,
            MockTokenProvider {
                tokens: vec!["provider-token".to_string()],
                keep_open: false,
                expires_on: None,
            },
        );

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        // Two batches: the first caches the token and observes the stream close;
        // the second must reuse the cached token.
        let pdatas = subscribe_pdatas(
            vec![
                OtapPdata::new_default(OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
                    Bytes::from(bytes.clone()),
                ))),
                OtapPdata::new_default(OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
                    Bytes::from(bytes),
                ))),
            ],
            false,
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                })
            });

        cancel.cancel();
        let headers = captured
            .lock()
            .clone()
            .expect("server did not capture any request headers");
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Bearer provider-token",
            "the cached token must keep being used after the provider closes its stream"
        );
    }

    #[test]
    fn build_static_headers_marks_values_sensitive() {
        // Symmetric with the OTLP/gRPC `build_static_metadata_builds_map` test:
        // every static header value the exporter puts on the wire must be marked
        // sensitive so it is excluded from HTTP/2 HPACK indexing and redacted in
        // any `HeaderMap` `Debug` output. (The wire-capture test above cannot
        // assert this: the sensitive flag is a client-side `HeaderValue`
        // property that is not transmitted to the server.)
        let mut headers = HashMap::new();
        _ = headers.insert("authorization".to_string(), "Basic abc123".into());
        _ = headers.insert("x-scope-orgid".to_string(), "tenant-1".into());
        let settings = HttpClientSettings {
            headers,
            ..Default::default()
        };

        let built =
            HttpClientPool::build_static_headers(&settings, 0).expect("should build headers");
        assert_eq!(built.len(), 2);
        assert_eq!(
            built.get("authorization").unwrap().to_str().unwrap(),
            "Basic abc123"
        );
        assert_eq!(
            built.get("x-scope-orgid").unwrap().to_str().unwrap(),
            "tenant-1"
        );
        assert!(
            built.get("authorization").unwrap().is_sensitive(),
            "static header values must be marked sensitive so they are excluded from HPACK \
             indexing and redacted in HeaderMap Debug output"
        );
        assert!(
            built.get("x-scope-orgid").unwrap().is_sensitive(),
            "every static header value must be marked sensitive, not just the first"
        );
    }

    /// run test HTTP server serving OTLP HTTP API. Internally, this uses the OTLP HTTP server that
    /// is used in OTLP Receiver. This returns a cancellation token (to shutdown the server when
    /// the test is finished), and a receiver that will emit any pdata that the server produces.
    fn run_tls_server(
        tokio_rt: &Runtime,
        pipeline_ctx: &PipelineContext,
        endpoint_addr: &str,
        tls_server_config: TlsServerConfig,
    ) -> (tokio::sync::mpsc::Receiver<OtapPdata>, CancellationToken) {
        let server_node_id = test_node("test-server");
        let port_name = PortName::from("server_out");
        let mut msg_senders = HashMap::new();
        let (pdata_tx, pdata_rx) = tokio::sync::mpsc::channel(10);
        _ = msg_senders.insert(port_name.clone(), SharedSender::mpsc(pdata_tx));

        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(10);
        let (_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(5);
        let server_effect_handler =
            otap_df_engine::shared::receiver::EffectHandler::<OtapPdata>::new(
                server_node_id,
                msg_senders,
                Some(port_name),
                runtime_ctrl_msg_tx,
                metrics_reporter,
            );

        let mut server_settings = HttpServerSettings {
            listening_addr: endpoint_addr.parse().unwrap(),
            tls: Some(tls_server_config),
            ..Default::default()
        };
        tune_max_concurrent_requests(&mut server_settings, 1);

        let ack_registry = AckRegistry::new(None, None, None);
        let server_metrics = OtlpReceiverMetrics::register(pipeline_ctx);
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();

        _ = tokio_rt.spawn(async move {
            let _ = serve(
                server_effect_handler,
                server_settings,
                ack_registry,
                Arc::new(Mutex::new(server_metrics)),
                otap_df_engine::memory_limiter::SharedReceiverAdmissionState::default(),
                None,
                server_cancellation_token,
            )
            .await;
        });

        // Wait for the server to be ready to accept connections. Without this,
        // the exporter may attempt to connect before the server has bound,
        // leading to a retryable NACK and the test hanging in validation.
        wait_for_port_ready(endpoint_addr);

        (pdata_rx, server_cancellation_token2)
    }

    fn setup_exporter(
        test_runtime: &TestRuntime<OtapPdata>,
        config: Config,
    ) -> (PipelineContext, ExporterWrapper<OtapPdata>) {
        otap_df_otap::crypto::ensure_crypto_provider();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_HTTP_EXPORTER_URN));
        let telemetry_registry_handle = test_runtime.metrics_registry();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let test_runtime_name = test_runtime.config().name.clone();
        let node_id = test_node(test_runtime_name.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            "test_group".into(),
            "test_pipeline".into(),
            0,
            1,
            0,
        );

        let exporter = ExporterWrapper::local(
            OtlpHttpExporter {
                config,
                pdata_metrics: ExporterPDataExportMetrics::register(&pipeline_ctx),
                token_provider: None,
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        (pipeline_ctx, exporter)
    }

    fn gen_batches_for_each_signal_type() -> (LogsData, MetricsData, TracesData) {
        let logs_batch = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord {
                        event_name: "Hello".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let metrics_batch = MetricsData::new(vec![ResourceMetrics {
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric::build().name("metric").finish()],
                ..Default::default()
            }],
            ..Default::default()
        }]);

        let traces_batch = TracesData::new(vec![ResourceSpans {
            scope_spans: vec![otap_df_pdata::proto::opentelemetry::trace::v1::ScopeSpans {
                spans: vec![otap_df_pdata::proto::opentelemetry::trace::v1::Span {
                    name: "span".into(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }]);

        (logs_batch, metrics_batch, traces_batch)
    }

    fn subscribe_pdatas(pdatas: Vec<OtapPdata>, return_payload: bool) -> Vec<OtapPdata> {
        let interests = if return_payload {
            Interests::ACKS | Interests::NACKS | Interests::RETURN_DATA
        } else {
            Interests::ACKS | Interests::NACKS
        };
        pdatas
            .into_iter()
            .map(|pdata| pdata.test_subscribe_to(interests, TestCallData::default().into(), 123))
            .collect()
    }

    fn default_test_config(endpoint: String) -> Config {
        Config {
            http: HttpClientSettings::default(),
            endpoint,
            client_pool_size: NonZeroUsize::try_from(2).unwrap(),
            max_response_body_length: 1024,
            max_in_flight: 10,
            traces_endpoint: None,
            metrics_endpoint: None,
            logs_endpoint: None,
        }
    }

    #[test]
    fn test_from_config_validates_endpoint() {
        let invalid_config = serde_json::json!({
            "endpoint": "not a valid url",
            "http": {},
            "client_pool_size": 5
        });

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let telemetry_registry_handle = test_runtime.metrics_registry();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            "test_group".into(),
            "test_pipeline".into(),
            0,
            1,
            0,
        );

        let result = OtlpHttpExporter::from_config(pipeline_ctx, &invalid_config, None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
        assert!(err.to_string().contains("invalid endpoint URL"))
    }

    #[test]
    fn test_from_config_validates_endpoint_overrides() {
        let test_cases = [
            (
                serde_json::json!({
                    "endpoint": "http://127.0.0.1",
                    "http": {},
                    "client_pool_size": 5,
                    "logs_endpoint": "invalid endpoint"
                }),
                "logs",
            ),
            (
                serde_json::json!({
                    "endpoint": "http://127.0.0.1",
                    "http": {},
                    "client_pool_size": 5,
                    "metrics_endpoint": "invalid endpoint"
                }),
                "metrics",
            ),
            (
                serde_json::json!({
                    "endpoint": "http://127.0.0.1",
                    "http": {},
                    "client_pool_size": 5,
                    "traces_endpoint": "invalid endpoint"
                }),
                "traces",
            ),
        ];
        for (invalid_config, signal_name) in test_cases {
            let test_runtime = TestRuntime::<OtapPdata>::new();
            let telemetry_registry_handle = test_runtime.metrics_registry();
            let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
            let pipeline_ctx = controller_ctx.pipeline_context_with(
                "test_group".into(),
                "test_pipeline".into(),
                0,
                1,
                0,
            );

            let result = OtlpHttpExporter::from_config(pipeline_ctx, &invalid_config, None);
            assert!(result.is_err());
            let err = result.err().unwrap();
            assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
            assert!(
                err.to_string()
                    .contains(&format!("invalid {signal_name} endpoint URL"))
            )
        }
    }

    #[test]
    fn test_exports_otlp_signals() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (pipeline_ctx, exporter) = setup_exporter(&test_runtime, config);
        let (mut pdata_rx, server_cancellation_token) =
            run_server(&tokio_rt, &pipeline_ctx, &endpoint_addr);

        let (logs_batch, metrics_batch, traces_batch) = gen_batches_for_each_signal_type();

        let mut pdatas = vec![];

        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
        )));

        let mut bytes = Vec::new();
        metrics_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)),
        )));

        let mut bytes = Vec::new();
        traces_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportTracesRequest(Bytes::from(bytes)),
        )));

        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    result.unwrap();

                    // ensure we got back all the signals we expected ...
                    let num_expected_pdatas = 3;
                    let mut pdatas_received = Vec::new();
                    while let Some(pdata) = pdata_rx.recv().await {
                        pdatas_received.push(pdata);
                        if pdatas_received.len() >= num_expected_pdatas {
                            break;
                        }
                    }

                    for mut pdata in pdatas_received {
                        match pdata.signal_type() {
                            SignalType::Logs => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = LogsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Logs(pdata_decoded)],
                                    &[OtlpProtoMessage::Logs(logs_batch.clone())],
                                );
                            }
                            SignalType::Metrics => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = MetricsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Metrics(pdata_decoded)],
                                    &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                                );
                            }
                            SignalType::Traces => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = TracesData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Traces(pdata_decoded)],
                                    &[OtlpProtoMessage::Traces(traces_batch.clone())],
                                );
                            }
                        }
                    }

                    let mut ack_count = 0;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                ack_count += 1;
                                if ack_count >= num_expected_pdatas {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverNack { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_pdatas);

                    server_cancellation_token.cancel();
                })
            })
    }

    fn run_error_status_code_test(status: u16, retryable: bool) {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);
        let server_cancellation_token = run_error_server(&tokio_rt, &endpoint_addr, Some(status));

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    server_cancellation_token.cancel();

                    let mut ack_count = 0;
                    let num_expected_nacks = 1;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                assert!(
                                    nack.reason.contains("HTTP status")
                                        && nack.reason.contains(&status.to_string()),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert_eq!(
                                    nack.permanent, !retryable,
                                    "invalid retryable nack decision for status {status}"
                                );

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    /// Scenario: The OTLP HTTP server returns retryable and permanent non-success statuses.
    /// Guarantees: Each consumed request yields one Nack and exactly one failed export outcome.
    #[test]
    fn test_handles_non_200_response_status() {
        let test_cases = [(500, false), (429, true), (503, true), (504, true)];

        for (status, retryable) in test_cases {
            run_error_status_code_test(status, retryable);
        }
    }

    /// Scenario: An OTLP HTTP export finishes with a terminal service-request error.
    /// Guarantees: Finalization records exactly one failure for the exported signal.
    #[test]
    fn failed_export_finalization_records_one_terminal_outcome() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = ExporterPDataExportMetrics::register(&pipeline_ctx);

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
        let effect_handler = EffectHandler::new(test_node("test-exporter"), metrics_reporter);
        let completed = CompletedExport {
            result: Err(ServiceRequestError::BodyTooLarge {
                body_size: 2,
                max_size: 1,
            }),
            context: Context::default(),
            saved_payload: OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
            signal_type: SignalType::Logs,
        };

        Runtime::new().unwrap().block_on(finalize_completed_export(
            completed,
            &effect_handler,
            &mut metrics,
            false,
        ));

        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Failure,
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .get(SignalOutcomeAttributes {
                    signal: SignalType::Logs,
                    outcome: Outcome::Success,
                })
                .messages
                .get(),
            0
        );
    }

    #[test]
    fn test_handles_connection_refused_errors() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        // Note - no server is running

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    let mut ack_count = 0;
                    let num_expected_nacks = 1;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                assert!(
                                    nack.reason.contains("client error (Connect)"),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert!(
                                    !nack.permanent,
                                    "expected connection error to be retryable nack"
                                );

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_handles_partial_success() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        let tokio_rt = Runtime::new().unwrap();
        let server_cancellation_token = run_error_server(&tokio_rt, &endpoint_addr, None);

        let (logs_batch, metrics_batch, traces_batch) = gen_batches_for_each_signal_type();

        let mut pdatas = vec![];
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
        )));

        let mut bytes = Vec::new();
        metrics_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportMetricsRequest(Bytes::from(bytes)),
        )));

        let mut bytes = Vec::new();
        traces_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportTracesRequest(Bytes::from(bytes)),
        )));

        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    let mut ack_count = 0;
                    let num_expected_nacks = 3;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                assert!(
                                    nack.reason.contains("partial success error (1 rejected)"),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert!(
                                    nack.permanent,
                                    "expected partial success to be permanent nack"
                                );

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);

                    server_cancellation_token.cancel();
                })
            })
    }

    #[test]
    fn test_handles_response_body_too_large() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = Config {
            // smaller than expected response body
            max_response_body_length: 2,
            ..default_test_config(endpoint)
        };

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        let tokio_rt = Runtime::new().unwrap();
        let server_cancellation_token = run_error_server(&tokio_rt, &endpoint_addr, None);

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let mut pdatas = vec![];
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
        )));

        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    let mut ack_count = 0;
                    let num_expected_nacks = 1;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                assert!(
                                    nack.reason.contains("exceeds maximum allowed size"),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert!(
                                    nack.permanent,
                                    "expected too large body to be permanent nack"
                                );

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);

                    server_cancellation_token.cancel();
                })
            })
    }

    #[test]
    fn test_handles_invalid_otap_payloads() {
        // Malformed structured OTAP payloads are rejected when inserted into `OtapArrowRecords`,
        // so the exporter never receives this invalid batch shape.
        let invalid_record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                "resource",
                DataType::Int32,
                false,
            )])),
            vec![Arc::new(Int32Array::from(vec![1, 2, 3]))],
        )
        .unwrap();
        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        let err = otap_batch
            .set(ArrowPayloadType::Logs, invalid_record_batch)
            .expect_err("invalid OTAP batch should be rejected before export");

        assert!(
            err.to_string().contains("Invalid schema for payload Logs")
                && err.to_string().contains("Column `resource`"),
            "unexpected validation error: {err}"
        );
    }

    #[test]
    fn test_nacks_for_otap_payloads_when_context_indicates_no_payload_return() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);
        let server_cancellation_token = run_error_server(&tokio_rt, &endpoint_addr, Some(500));

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];

        let pdatas = subscribe_pdatas(pdatas, true);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    server_cancellation_token.cancel();

                    let mut ack_count = 0;
                    let num_expected_nacks = 1;
                    let mut pipeline_completion_rx = ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                match nack.refused.payload() {
                                    OtapPayload::OtapArrowRecords(otap_batch) => {
                                        let logs_batch = otap_batch.get(ArrowPayloadType::Logs).unwrap();
                                        assert!(
                                            logs_batch.num_rows() > 0,
                                            "expected record batches to be returned in Nack, but it was empty"
                                        );
                                    }
                                    other_payload => {
                                        panic!(
                                            "received unexpected payload type in Nack: {:?}",
                                            other_payload
                                        );
                                    }
                                }

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_nacks_for_otlp_payloads_when_context_indicates_no_payload_return() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);
        let server_cancellation_token = run_error_server(&tokio_rt, &endpoint_addr, Some(500));

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let mut pdatas = vec![];
        let mut bytes = Vec::new();
        logs_batch.encode(&mut bytes).unwrap();
        pdatas.push(OtapPdata::new_default(OtapPayload::OtlpBytes(
            OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes)),
        )));

        let pdatas = subscribe_pdatas(pdatas, true);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    server_cancellation_token.cancel();

                    let mut ack_count = 0;
                    let num_expected_nacks = 1;
                    let mut pipeline_completion_rx = ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                ack_count += 1;

                                match nack.refused.payload() {
                                    OtapPayload::OtlpBytes(proto_bytes) => {
                                        assert!(
                                            !proto_bytes.as_bytes().is_empty(),
                                            "expected payload bytes to be returned in Nack, but it was empty"
                                        );
                                    }
                                    other_payload => {
                                        panic!(
                                            "received unexpected payload type in Nack: {:?}",
                                            other_payload
                                        );
                                    }
                                }

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_export_otap_signals() {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (pipeline_ctx, exporter) = setup_exporter(&test_runtime, config);
        let (mut pdata_rx, server_cancellation_token) =
            run_server(&tokio_rt, &pipeline_ctx, &endpoint_addr);

        let (logs_batch, metrics_batch, traces_batch) = gen_batches_for_each_signal_type();

        let pdatas = vec![
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Logs(logs_batch.clone()),
            ))),
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Metrics(metrics_batch.clone()),
            ))),
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Traces(traces_batch.clone()),
            ))),
        ];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    let num_expected_pdatas = 3;
                    let mut pdatas_received = Vec::new();
                    while let Some(pdata) = pdata_rx.recv().await {
                        pdatas_received.push(pdata);
                        if pdatas_received.len() >= num_expected_pdatas {
                            break;
                        }
                    }
                    server_cancellation_token.cancel();

                    for mut pdata in pdatas_received {
                        match pdata.signal_type() {
                            SignalType::Logs => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = LogsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Logs(pdata_decoded)],
                                    &[OtlpProtoMessage::Logs(logs_batch.clone())],
                                );
                            }
                            SignalType::Metrics => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = MetricsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Metrics(pdata_decoded)],
                                    &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                                );
                            }
                            SignalType::Traces => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into_with_default().unwrap();
                                let pdata_decoded = TracesData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Traces(pdata_decoded)],
                                    &[OtlpProtoMessage::Traces(traces_batch.clone())],
                                );
                            }
                        }
                    }

                    let mut ack_count = 0;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                ack_count += 1;
                                if ack_count >= num_expected_pdatas {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverNack { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_pdatas);
                })
            })
    }

    #[test]
    pub fn test_uses_endpoint_overrides_if_provided() {
        let logs_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let logs_endpoint_addr = format!("127.0.0.1:{}", logs_port);
        let logs_endpoint = format!("http://{logs_endpoint_addr}/v1/logs");

        let metrics_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let metrics_endpoint_addr = format!("127.0.0.1:{}", metrics_port);
        let metrics_endpoint = format!("http://{metrics_endpoint_addr}/v1/metrics");

        let traces_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let traces_endpoint_addr = format!("127.0.0.1:{}", traces_port);
        let traces_endpoint = format!("http://{traces_endpoint_addr}/v1/traces");

        let config = Config {
            logs_endpoint: Some(logs_endpoint),
            metrics_endpoint: Some(metrics_endpoint),
            traces_endpoint: Some(traces_endpoint),
            ..default_test_config("http://placeholder".to_string())
        };

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (pipeline_ctx, exporter) = setup_exporter(&test_runtime, config);

        let (mut logs_pdata_rx, logs_server_cancellation_token) =
            run_server(&tokio_rt, &pipeline_ctx, &logs_endpoint_addr);
        let (mut metrics_pdata_rx, metrics_server_cancellation_token) =
            run_server(&tokio_rt, &pipeline_ctx, &metrics_endpoint_addr);
        let (mut traces_pdata_rx, traces_server_cancellation_token) =
            run_server(&tokio_rt, &pipeline_ctx, &traces_endpoint_addr);

        let (logs_batch, metrics_batch, traces_batch) = gen_batches_for_each_signal_type();

        let pdatas = vec![
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Logs(logs_batch.clone()),
            ))),
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Metrics(metrics_batch.clone()),
            ))),
            OtapPdata::new_default(OtapPayload::OtapArrowRecords(otlp_to_otap(
                &OtlpProtoMessage::Traces(traces_batch.clone()),
            ))),
        ];

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    // ensure we got back all the signals we expected, from the correct servers.

                    let mut pdata = logs_pdata_rx.recv().await.unwrap();
                    let otlp_bytes: OtlpProtoBytes =
                        pdata.take_payload().try_into_with_default().unwrap();
                    let pdata_decoded = LogsData::decode(otlp_bytes.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Logs(pdata_decoded)],
                        &[OtlpProtoMessage::Logs(logs_batch.clone())],
                    );

                    let mut pdata = metrics_pdata_rx.recv().await.unwrap();
                    let otlp_bytes: OtlpProtoBytes =
                        pdata.take_payload().try_into_with_default().unwrap();
                    let pdata_decoded = MetricsData::decode(otlp_bytes.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Metrics(pdata_decoded)],
                        &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                    );

                    let mut pdata = traces_pdata_rx.recv().await.unwrap();
                    let otlp_bytes: OtlpProtoBytes =
                        pdata.take_payload().try_into_with_default().unwrap();
                    let pdata_decoded = TracesData::decode(otlp_bytes.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Traces(pdata_decoded)],
                        &[OtlpProtoMessage::Traces(traces_batch.clone())],
                    );

                    logs_server_cancellation_token.cancel();
                    metrics_server_cancellation_token.cancel();
                    traces_server_cancellation_token.cancel();
                })
            })
    }

    fn run_tls_success_test(
        client_tls_config: TlsClientConfig,
        server_tls_config: TlsServerConfig,
    ) {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("https://localhost:{port}");

        let config = Config {
            http: HttpClientSettings {
                tls: Some(client_tls_config),
                ..Default::default()
            },
            ..default_test_config(endpoint)
        };

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (pipeline_ctx, exporter) = setup_exporter(&test_runtime, config);

        let (mut pdata_rx, server_cancellation_token) =
            run_tls_server(&tokio_rt, &pipeline_ctx, &endpoint_addr, server_tls_config);

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();

                    let num_expected_pdatas = 1;
                    let mut pdatas_received = Vec::new();
                    let recv_deadline = tokio::time::timeout(Duration::from_secs(10), async {
                        while let Some(pdata) = pdata_rx.recv().await {
                            pdatas_received.push(pdata);
                            if pdatas_received.len() >= num_expected_pdatas {
                                break;
                            }
                        }
                    });
                    recv_deadline.await.expect(
                        "timed out waiting for server to receive pdata; \
                         server may not have been ready in time",
                    );

                    server_cancellation_token.cancel();

                    // assert the pdata sent was the pdata received
                    let pdata: OtlpProtoBytes = pdatas_received[0]
                        .take_payload()
                        .try_into_with_default()
                        .unwrap();
                    let pdata_decoded = LogsData::decode(pdata.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Logs(pdata_decoded)],
                        &[OtlpProtoMessage::Logs(logs_batch.clone())],
                    );

                    let mut ack_count = 0;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                ack_count += 1;
                                if ack_count >= num_expected_pdatas {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverNack { .. } => {
                                panic!("unexpected Nack message")
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_pdatas);
                })
            })
    }

    fn run_tls_failure_test(
        client_tls_config: TlsClientConfig,
        server_tls_config: TlsServerConfig,
        expected_err_content: &'static str,
        expect_permanent_nack: bool,
    ) {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("https://localhost:{port}");

        let config = Config {
            http: HttpClientSettings {
                tls: Some(client_tls_config),
                ..Default::default()
            },
            ..default_test_config(endpoint)
        };

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (pipeline_ctx, exporter) = setup_exporter(&test_runtime, config);

        let (_, server_cancellation_token) =
            run_tls_server(&tokio_rt, &pipeline_ctx, &endpoint_addr, server_tls_config);

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }

                    tokio::time::sleep(Duration::from_millis(10)).await;

                    ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // ensure exit success
                    result.unwrap();
                    server_cancellation_token.cancel();

                    let mut nack_count = 0;
                    let expected_nacks = 1;
                    let mut pipeline_completion_rx =
                        ctx.take_pipeline_completion_receiver().unwrap();
                    loop {
                        let msg = match pipeline_completion_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineCompletionMsg::DeliverNack { nack } => {
                                nack_count += 1;

                                assert!(
                                    nack.reason.contains(expected_err_content),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert_eq!(nack.permanent, expect_permanent_nack,);

                                if nack_count >= expected_nacks {
                                    break;
                                }
                            }
                            PipelineCompletionMsg::DeliverAck { .. } => {
                                panic!("unexpected Ack message")
                            }
                        }
                    }
                    assert_eq!(nack_count, expected_nacks);
                })
            })
    }

    #[test]
    fn test_from_config_validates_server_name_override_set() {
        let invalid_config = serde_json::json!({
            "endpoint": "https://localhost",
            "http": {
                "tls": {
                    "server_name_override": "localhost123"
                }
            },
            "client_pool_size": 5
        });

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let telemetry_registry_handle = test_runtime.metrics_registry();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            "test_group".into(),
            "test_pipeline".into(),
            0,
            1,
            0,
        );

        let result = OtlpHttpExporter::from_config(pipeline_ctx, &invalid_config, None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
        assert!(
            err.to_string()
                .contains("server_name_override is not supported")
        )
    }

    #[test]
    fn test_tls_server_only_ca_pem_from_str() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        run_tls_success_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: Some(ca.cert_pem.clone()),
                include_system_ca_certs_pool: None,
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
        );
    }

    #[test]
    fn test_tls_server_only_ca_pem_from_file() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        ca.write_cert_to_dir(path, "ca");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        run_tls_success_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: Some(path.join("ca.crt")),
                ca_pem: None,
                include_system_ca_certs_pool: None,
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
        );
    }

    #[test]
    fn test_tls_server_insecure_skip_verify_true() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        ca.write_cert_to_dir(path, "ca");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        run_tls_success_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: None,
                include_system_ca_certs_pool: None,
                server_name: None,
                insecure: None,
                insecure_skip_verify: Some(true),
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
        );
    }

    #[test]
    fn test_tls_server_failure_no_ca_configured() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        ca.write_cert_to_dir(path, "ca");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        run_tls_failure_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: None,
                include_system_ca_certs_pool: None,
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
            "client error (Connect): invalid peer certificate: UnknownIssuer",
            false,
        );
    }

    #[test]
    fn test_tls_server_failure_invalid_ca_configured() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        let ca2 = generate_ca("Test CA 2");

        run_tls_failure_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: Some(ca2.cert_pem.to_string()),
                include_system_ca_certs_pool: None,
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
            "client error (Connect): invalid peer certificate: UnknownIssuer",
            false,
        );
    }

    #[test]
    fn test_tls_server_failure_server_name_mismatch() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost123",
            Some("localhost123"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        server.write_to_dir(path, "server");

        run_tls_failure_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: None,
                    key_file: None,
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: Some(ca.cert_pem.to_string()),
                include_system_ca_certs_pool: None,
                server_name: None, // the test runner func will use hostname "localhost"
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: None,
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
            "client error (Connect): invalid peer certificate",
            false,
        );
    }

    #[test]
    fn test_tls_mtls_success_cert_pem() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        // Generate CA and certificates
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        let client = ca.issue_leaf("Test Client", None, Some(ExtendedKeyUsage::ClientAuth));

        server.write_to_dir(path, "server");
        client.write_to_dir(path, "client");

        run_tls_success_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: None,
                    cert_pem: Some(client.cert_pem.clone()),
                    key_file: None,
                    key_pem: Some(client.key_pem.clone()),
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: Some(ca.cert_pem.clone()),
                include_system_ca_certs_pool: Some(false),
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: Some(ca.cert_pem.to_string()),
                include_system_ca_certs_pool: Some(false),
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
        );
    }

    #[test]
    fn test_tls_mtls_success_cert_file() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        // Generate CA and certificates
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        let client = ca.issue_leaf("Test Client", None, Some(ExtendedKeyUsage::ClientAuth));

        server.write_to_dir(path, "server");
        client.write_to_dir(path, "client");
        ca.write_cert_to_dir(path, "ca");

        run_tls_success_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("client.crt")),
                    cert_pem: None,
                    key_file: Some(path.join("client.key")),
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: Some(path.join("ca.crt")),
                ca_pem: None,
                include_system_ca_certs_pool: Some(false),
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: None,
                client_ca_pem: Some(ca.cert_pem.to_string()),
                include_system_ca_certs_pool: None,
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
        );
    }

    #[test]
    fn test_tls_mtls_failure_wrong_client_cert() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        // Generate CA and certificates
        let ca = generate_ca("Test CA");
        let server = ca.issue_leaf(
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );
        let trusted_client =
            ca.issue_leaf("Trusted Client", None, Some(ExtendedKeyUsage::ClientAuth));

        // Wrong client cert from different CA
        let other_ca = generate_ca("Other CA");
        let wrong_client =
            other_ca.issue_leaf("Wrong Client", None, Some(ExtendedKeyUsage::ClientAuth));

        server.write_to_dir(path, "server");
        trusted_client.write_to_dir(path, "trusted_client");
        wrong_client.write_to_dir(path, "wrong_client");

        run_tls_failure_test(
            TlsClientConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("wrong_client.crt")),
                    cert_pem: None,
                    key_file: Some(path.join("wrong_client.key")),
                    key_pem: None,
                    reload_interval: None,
                },
                ca_file: None,
                ca_pem: Some(ca.cert_pem.clone()),
                include_system_ca_certs_pool: Some(false),
                server_name: None,
                insecure: None,
                insecure_skip_verify: None,
            },
            TlsServerConfig {
                config: TlsConfig {
                    cert_file: Some(path.join("server.crt")),
                    key_file: Some(path.join("server.key")),
                    cert_pem: None,
                    key_pem: None,
                    reload_interval: None,
                },
                client_ca_file: Some(path.join("trusted_client.crt")),
                client_ca_pem: None,
                include_system_ca_certs_pool: Some(false),
                watch_client_ca: false,
                handshake_timeout: Some(Duration::from_secs(10)),
            },
            "client error",
            true,
        );
    }

    #[test]
    fn test_start_returns_error_if_mtls_cert_without_key() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        let ca = generate_ca("Test CA");
        let client = ca.issue_leaf("Test Client", None, Some(ExtendedKeyUsage::ClientAuth));
        client.write_to_dir(path, "client");

        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint = format!("https://localhost:{port}");

        let config = Config {
            http: HttpClientSettings {
                tls: Some(TlsClientConfig {
                    config: TlsConfig {
                        cert_file: Some(path.join("client.crt")),
                        cert_pem: None,
                        key_file: None, // Missing key
                        key_pem: None,
                        reload_interval: None,
                    },
                    ca_file: None,
                    ca_pem: Some(ca.cert_pem.clone()),
                    include_system_ca_certs_pool: Some(false),
                    server_name: None,
                    insecure: None,
                    insecure_skip_verify: None,
                }),
                ..Default::default()
            },
            ..default_test_config(endpoint)
        };

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        test_runtime
            .set_exporter(exporter)
            .run_test(|_ctx| Box::pin(async move {}))
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    let err = result.unwrap_err();
                    let err_message = err.to_string();
                    assert!(
                        err_message.contains("both cert and key must be provided"),
                        "unexpected error: {}",
                        err_message
                    )
                })
            });
    }

    #[test]
    fn test_start_returns_error_if_mtls_key_without_cert() {
        otap_df_otap::crypto::ensure_crypto_provider();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let path = temp_dir.path();

        let ca = generate_ca("Test CA");
        let client = ca.issue_leaf("Test Client", None, Some(ExtendedKeyUsage::ClientAuth));
        client.write_to_dir(path, "client");

        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint = format!("https://localhost:{port}");

        let config = Config {
            http: HttpClientSettings {
                tls: Some(TlsClientConfig {
                    config: TlsConfig {
                        cert_file: None, // Missing cert
                        cert_pem: None,
                        key_file: Some(path.join("client.key")),
                        key_pem: None,
                        reload_interval: None,
                    },
                    ca_file: None,
                    ca_pem: Some(ca.cert_pem.clone()),
                    include_system_ca_certs_pool: Some(false),
                    server_name: None,
                    insecure: None,
                    insecure_skip_verify: None,
                }),
                ..Default::default()
            },
            ..default_test_config(endpoint)
        };

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        test_runtime
            .set_exporter(exporter)
            .run_test(|_ctx| Box::pin(async move {}))
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    let err = result.unwrap_err();
                    let err_message = err.to_string();
                    assert!(
                        err_message.contains("both cert and key must be provided"),
                        "unexpected error: {}",
                        err_message
                    )
                })
            });
    }

    /// Captured snapshot of a single HTTP request received by the compression test server.
    #[derive(Debug)]
    struct CapturedRequest {
        content_encoding: Option<String>,
        body: Vec<u8>,
    }

    fn decode_with(encoding: Option<&str>, body: &[u8]) -> Vec<u8> {
        use std::io::Read;
        match encoding {
            None => body.to_vec(),
            Some("gzip") => {
                let mut out = Vec::new();
                _ = flate2::read::GzDecoder::new(body)
                    .read_to_end(&mut out)
                    .unwrap();
                out
            }
            Some("deflate") => {
                // Match the OTLP/HTTP receiver: zlib-wrapped DEFLATE, not raw.
                let mut out = Vec::new();
                _ = flate2::read::ZlibDecoder::new(body)
                    .read_to_end(&mut out)
                    .unwrap();
                out
            }
            Some("zstd") => {
                let mut out = Vec::new();
                zstd::stream::copy_decode(body, &mut out).unwrap();
                out
            }
            Some(other) => panic!("unexpected content-encoding: {other}"),
        }
    }

    fn run_compression_round_trip(
        compression: Option<otap_df_otap::compression::CompressionMethod>,
        expected_encoding: Option<&'static str>,
    ) {
        let port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let endpoint_addr = format!("127.0.0.1:{port}");
        let endpoint = format!("http://{endpoint_addr}");

        let mut config = default_test_config(endpoint);
        config.http.compression = compression;

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        // Bind the capture server. We do not use the run_capture_server helper above
        // because we need to hand the precise bound address to the wait_for_port_ready
        // helper - simpler to inline.
        let (tx, mut captured_rx) = tokio::sync::mpsc::channel::<CapturedRequest>(8);
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();
        let bind_addr = endpoint_addr.clone();
        _ = tokio_rt.spawn(async move {
            use http_body_util::BodyExt;
            let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
            let tracker = TaskTracker::new();
            loop {
                tokio::select! {
                    _ = server_cancellation_token.cancelled() => break,
                    accept_result = listener.accept() => {
                        let (stream, _peer_addr) = accept_result.unwrap();
                        let shutdown_token = server_cancellation_token.clone();
                        let tx = tx.clone();
                        drop(tracker.spawn(async move {
                            let io = TokioIo::new(stream);
                            let conn = http1::Builder::new().serve_connection(io, service_fn(move |req: hyper::Request<hyper::body::Incoming>| {
                                let tx = tx.clone();
                                async move {
                                    let content_encoding = req
                                        .headers()
                                        .get(http::header::CONTENT_ENCODING)
                                        .and_then(|v| v.to_str().ok())
                                        .map(|s| s.to_string());
                                    let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
                                    _ = tx.send(CapturedRequest {
                                        content_encoding,
                                        body: body_bytes.to_vec(),
                                    }).await;

                                    let resp = ExportLogsServiceResponse::default();
                                    let mut body = Vec::new();
                                    resp.encode(&mut body).unwrap();
                                    Ok::<_, hyper::Error>(Response::builder()
                                        .status(200)
                                        .body(Full::new(Bytes::from(body)))
                                        .unwrap())
                                }
                            }));
                            let mut conn = std::pin::pin!(conn);
                            tokio::select! {
                                _ = shutdown_token.cancelled() => {
                                    conn.as_mut().graceful_shutdown();
                                    let _ = conn.await;
                                }
                                conn_result = &mut conn => {
                                    if let Err(e) = conn_result {
                                        eprintln!("capture server error: {e}");
                                    }
                                }
                            }
                        }));
                    }
                }
            }
            let _ = tracker.close();
        });
        wait_for_port_ready(&endpoint_addr);

        let (logs_batch, _, _) = gen_batches_for_each_signal_type();
        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otlp_to_otap(&OtlpProtoMessage::Logs(logs_batch.clone())),
        ))];
        let pdatas = subscribe_pdatas(pdatas, false);

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| {
                Box::pin(async move {
                    for pdata in pdatas {
                        ctx.send_pdata(pdata).await.unwrap();
                    }
                    ctx.send_shutdown(Instant::now() + Duration::from_millis(500), "test complete")
                        .await
                        .unwrap();
                })
            })
            .run_validation(|_ctx, result| {
                Box::pin(async move {
                    result.unwrap();
                    let captured = captured_rx.recv().await.expect("no request captured");
                    assert_eq!(
                        captured.content_encoding.as_deref(),
                        expected_encoding,
                        "unexpected content-encoding header"
                    );
                    let decoded = decode_with(captured.content_encoding.as_deref(), &captured.body);
                    let decoded_req = otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest::decode(decoded.as_slice())
                        .expect("body did not decode as ExportLogsServiceRequest");
                    let received_logs = LogsData {
                        resource_logs: decoded_req.resource_logs,
                    };
                    assert_equivalent(
                        &[OtlpProtoMessage::Logs(received_logs)],
                        &[OtlpProtoMessage::Logs(logs_batch.clone())],
                    );

                    server_cancellation_token2.cancel();
                })
            });
    }

    #[test]
    fn test_export_with_gzip_compression() {
        run_compression_round_trip(
            Some(otap_df_otap::compression::CompressionMethod::Gzip),
            Some("gzip"),
        );
    }

    #[test]
    fn test_export_with_zstd_compression() {
        run_compression_round_trip(
            Some(otap_df_otap::compression::CompressionMethod::Zstd),
            Some("zstd"),
        );
    }

    #[test]
    fn test_export_with_deflate_compression() {
        run_compression_round_trip(
            Some(otap_df_otap::compression::CompressionMethod::Deflate),
            Some("deflate"),
        );
    }

    #[test]
    fn test_export_without_compression_omits_content_encoding() {
        run_compression_round_trip(None, None);
    }
}
