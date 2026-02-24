// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP exporter via HTTP
//!
//! This exporter sends telemetry data to an OTLP server using the HTTP Protocol.
//!
//! ToDo:
//! - TLS/mTLS
//! - Proxy settings
//! - Compression (payloads and accepting compressed responses)
//! - JSON encoding payloads (currently only proto is supported and it's not configurable)
//! - Allow endpoint overrides for each signal type (similar to Go collector implementation)
//! - Unit test metrics reporting

use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::{FutureExt, StreamExt};
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
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::wiring_contract::WiringContract;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
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
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_debug, otel_info};
use prost::Message as _;
use reqwest::{Client, Response};

use crate::OTAP_EXPORTER_FACTORIES;
use crate::metrics::ExporterPDataMetrics;
use crate::otlp_exporter::InFlightExports;
use crate::otlp_http::client_settings::HttpClientSettings;
use crate::otlp_http::{LOGS_PATH, METRICS_PATH, PROTOBUF_CONTENT_TYPE, TRACES_PATH};
use crate::otlp_http_exporter::config::Config;
use crate::pdata::{Context, OtapPdata};

mod config;

/// The URN for the OTLP HTTP exporter
pub const OTLP_HTTP_EXPORTER_URN: &str = "urn:otel:otlp_http:exporter";

/// Exporter that sends OTLP data via HTTP
pub struct OtlpHttpExporter {
    config: Config,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
}

/// Declare the OTLP HTTP Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTLP_HTTP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTLP_HTTP_EXPORTER_URN,
    create: factory_create,
    wiring_contract: WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

fn factory_create(
    pipeline: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    exporter_config: &ExporterConfig,
) -> Result<ExporterWrapper<OtapPdata>, ConfigError> {
    Ok(ExporterWrapper::local(
        OtlpHttpExporter::from_config(pipeline, &node_config.config)?,
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
    ) -> Result<Self, ConfigError> {
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();

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

        Ok(Self {
            config,
            pdata_metrics,
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
        mut msg_chan: MessageChannel<OtapPdata>,
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

        let telemetry_timer_cancel = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let max_in_flight = self.config.max_in_flight.max(1);
        let mut client_pool =
            HttpClientPool::try_new(&self.config.http, self.config.client_pool_size).map_err(
                |e| EngineError::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    kind: ExporterErrorKind::Configuration,
                    error: "unable to initialize HTTP client pool".into(),
                    source_detail: e.to_string(),
                },
            )?;

        let mut inflight_exports = InFlightExports::new();

        let mut logs_proto_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_proto_encoder = TracesProtoBytesEncoder::new();
        let mut proto_buffer = ProtoBuffer::with_capacity(8 * 1024);

        loop {
            // Opportunistically drain completions before we park on a recv.
            while let Some(completed) = inflight_exports.next_completion().now_or_never().flatten()
            {
                finalize_completed_export(completed, &effect_handler, &mut self.pdata_metrics)
                    .await;
            }

            // Backpressure guard: when full and a message is parked, only drain completions.
            if inflight_exports.len() >= max_in_flight {
                if let Some(completed) = inflight_exports.next_completion().await {
                    finalize_completed_export(completed, &effect_handler, &mut self.pdata_metrics)
                        .await;
                }
                continue;
            }

            // Prefer completions if any are ready, otherwise biased select between completion and recv.
            let msg = if inflight_exports.is_empty() {
                let msg = msg_chan.recv().await?;
                otel_debug!("otlp.exporter.http.receive");
                msg
            } else {
                let completion_fut = inflight_exports.next_completion().fuse();
                let recv_fut = msg_chan.recv().fuse();
                futures::pin_mut!(completion_fut, recv_fut);

                futures::select_biased! {
                    completed = completion_fut => {
                        if let Some(completed) = completed {
                            finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.pdata_metrics,
                            )
                            .await;
                        }
                        continue;
                    }
                    msg = recv_fut => {
                        let msg = msg?;
                        otel_debug!("otlp.exporter.http.receive");
                        msg
                    },
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
                            )
                            .await;
                        }
                    }
                    _ = telemetry_timer_cancel.cancel().await;
                    return Ok(TerminalState::new(deadline, [self.pdata_metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => _ = metrics_reporter.report(&mut self.pdata_metrics),
                Message::PData(pdata) => {
                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();

                    // proto encode the payload into the request body, while keeping a copy of the
                    // original payload if the context allows it to be returned
                    let (body, saved_payload) = match payload {
                        OtapPayload::OtlpBytes(mut otlp_bytes) => {
                            if context.may_return_payload() {
                                // use cheap clone of bytes as the request body
                                let body = otlp_bytes.clone_bytes();
                                (body, otlp_bytes.into())
                            } else {
                                // take the bytes and replace with empty bytes in original payload
                                let body = otlp_bytes.replace_bytes(Bytes::new());
                                (body, otlp_bytes.into())
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
                                // drop the original OTAP batch if the the context indicates it
                                // does not wish it to be returned
                                _ = otap_batch.take_payload();
                            }

                            let body = if let Err(e) = encode_result {
                                // encoding error, we must have received an invalid structured batch
                                let mut nack = NackMsg::new(
                                    e.to_string(),
                                    OtapPdata::new(context, otap_batch.into()),
                                );
                                nack.permanent = true;
                                _ = effect_handler.notify_nack(nack).await;
                                self.pdata_metrics.add_failed(signal_type, 1);
                                continue;
                            } else {
                                Bytes::copy_from_slice(proto_buffer.as_ref())
                            };

                            (body, otap_batch.into())
                        }
                    };

                    let endpoint = Rc::clone(match signal_type {
                        SignalType::Logs => &logs_endpoint,
                        SignalType::Metrics => &metrics_endpoint,
                        SignalType::Traces => &traces_endpoint,
                    });

                    let max_response_body_len = self.config.max_response_body_length;

                    let client = client_pool.get_client();
                    inflight_exports.push(async move {
                        let result = client.post(endpoint.as_str()).body(body).send().await;

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
}

fn format_source(e: &reqwest::Error) -> String {
    use std::error::Error;
    match e.source() {
        Some(src) => format!(": {src}"),
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
    pdata_metrics: &mut MetricSet<ExporterPDataMetrics>,
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
        Err(e) => Some((e.to_string(), e.is_retryable())),
    };

    let export_and_notify_success = match err {
        None => effect_handler.notify_ack(AckMsg::new(pdata)).await.is_ok(),
        Some((err_msg, retryable)) => {
            let mut nack = NackMsg::new(&err_msg, pdata);
            nack.permanent = !retryable;
            _ = effect_handler.notify_nack(nack).await;
            false
        }
    };

    if export_and_notify_success {
        pdata_metrics.add_exported(signal_type, 1)
    } else {
        pdata_metrics.add_failed(signal_type, 1)
    }
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
    fn try_new(
        client_settings: &HttpClientSettings,
        pool_size: NonZeroUsize,
    ) -> Result<Self, reqwest::Error> {
        let mut default_headers = HeaderMap::new();

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

    use arrow::array::{Int32Array, RecordBatch};
    use arrow::datatypes::{DataType, Field, Schema};
    use http_body_util::Full;
    use hyper::Response;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;
    use otap_df_config::PortName;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::shared::message::SharedSender;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::node::test_node;
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
    use otap_df_pdata::{OtapArrowRecords, OtlpProtoBytes};
    use otap_df_telemetry::reporter::MetricsReporter;
    use parking_lot::lock_api::Mutex;
    use portpicker::pick_unused_port;
    use prost::Message;
    use tokio::runtime::Runtime;
    use tokio_util::sync::CancellationToken;
    use tokio_util::task::TaskTracker;

    use super::*;

    use crate::otap_grpc::common::AckRegistry;
    use crate::otlp_http::client_settings::HttpClientSettings;
    use crate::otlp_http::{HttpServerSettings, serve, tune_max_concurrent_requests};
    use crate::otlp_receiver::OtlpReceiverMetrics;
    use crate::testing::TestCallData;

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

        let (pipeline_ctrl_msg_tx, _pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);
        let (_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(5);
        let server_effect_handler =
            otap_df_engine::shared::receiver::EffectHandler::<OtapPdata>::new(
                server_node_id,
                msg_senders,
                Some(port_name),
                pipeline_ctrl_msg_tx,
                metrics_reporter,
            );

        let mut server_settings = HttpServerSettings {
            listening_addr: endpoint_addr.parse().unwrap(),
            ..Default::default()
        };
        tune_max_concurrent_requests(&mut server_settings, 1);

        let ack_registry = AckRegistry::new(None, None, None);
        let server_metrics = pipeline_ctx.register_metrics::<OtlpReceiverMetrics>();
        let server_cancellation_token = CancellationToken::new();
        let server_cancellation_token2 = server_cancellation_token.clone();

        _ = tokio_rt.spawn(async move {
            serve(
                server_effect_handler,
                server_settings,
                ack_registry,
                Arc::new(Mutex::new(server_metrics)),
                None,
                server_cancellation_token,
            )
            .await
        });

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

    fn setup_exporter(
        test_runtime: &TestRuntime<OtapPdata>,
        config: Config,
    ) -> (PipelineContext, ExporterWrapper<OtapPdata>) {
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
                pdata_metrics: pipeline_ctx.register_metrics::<ExporterPDataMetrics>(),
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

        let result = OtlpHttpExporter::from_config(pipeline_ctx, &invalid_config);
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

            let result = OtlpHttpExporter::from_config(pipeline_ctx, &invalid_config);
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
        let port = pick_unused_port().unwrap();
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
                    // ensure exit success
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
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = LogsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Logs(pdata_decoded)],
                                    &[OtlpProtoMessage::Logs(logs_batch.clone())],
                                );
                            }
                            SignalType::Metrics => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = MetricsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Metrics(pdata_decoded)],
                                    &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                                );
                            }
                            SignalType::Traces => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = TracesData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Traces(pdata_decoded)],
                                    &[OtlpProtoMessage::Traces(traces_batch.clone())],
                                );
                            }
                        }
                    }

                    let mut ack_count = 0;
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverAck { .. } => {
                                ack_count += 1;
                                if ack_count >= num_expected_pdatas {
                                    break;
                                }
                            }
                            PipelineControlMsg::DeliverNack { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_pdatas);

                    server_cancellation_token.cancel();
                })
            })
    }

    fn run_error_status_code_test(status: u16, retryable: bool) {
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_handles_non_200_response_status() {
        let test_cases = [(500, false), (429, true), (503, true), (504, true)];

        for (status, retryable) in test_cases {
            run_error_status_code_test(status, retryable);
        }
    }

    #[test]
    fn test_handles_connection_refused_errors() {
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_handles_partial_success() {
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
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
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
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
        let port = pick_unused_port().unwrap();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        let config = default_test_config(endpoint);

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let (_, exporter) = setup_exporter(&test_runtime, config);

        // this is something we won't be able to serialize into a valid OTLP payload.
        // it would expect "resource" to be a struct column
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
        otap_batch.set(ArrowPayloadType::Logs, invalid_record_batch);

        let pdatas = vec![OtapPdata::new_default(OtapPayload::OtapArrowRecords(
            otap_batch,
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
                                ack_count += 1;

                                assert!(
                                    nack.reason.contains("Column `resource` data type mismatch"),
                                    "unexpected error message in Nack: {}",
                                    nack.reason
                                );

                                assert!(
                                    nack.permanent,
                                    "expected malformed OTAP batch to be permanent nack"
                                );

                                if ack_count >= num_expected_nacks {
                                    break;
                                }
                            }
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_nacks_for_otap_payloads_when_context_indicates_no_payload_return() {
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_nacks_for_otlp_payloads_when_context_indicates_no_payload_return() {
        let port = pick_unused_port().unwrap();
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
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverNack { nack, .. } => {
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
                            PipelineControlMsg::DeliverAck { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_nacks);
                })
            })
    }

    #[test]
    fn test_export_otap_signals() {
        let port = pick_unused_port().unwrap();
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
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = LogsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Logs(pdata_decoded)],
                                    &[OtlpProtoMessage::Logs(logs_batch.clone())],
                                );
                            }
                            SignalType::Metrics => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = MetricsData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Metrics(pdata_decoded)],
                                    &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                                );
                            }
                            SignalType::Traces => {
                                let pdata: OtlpProtoBytes =
                                    pdata.take_payload().try_into().unwrap();
                                let pdata_decoded = TracesData::decode(pdata.as_bytes()).unwrap();
                                assert_equivalent(
                                    &[OtlpProtoMessage::Traces(pdata_decoded)],
                                    &[OtlpProtoMessage::Traces(traces_batch.clone())],
                                );
                            }
                        }
                    }

                    let mut ack_count = 0;
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverAck { .. } => {
                                ack_count += 1;
                                if ack_count >= num_expected_pdatas {
                                    break;
                                }
                            }
                            PipelineControlMsg::DeliverNack { .. } => {
                                panic!("unexpected Nack message")
                            }
                            _ => {
                                // ignore other control messages
                            }
                        }
                    }
                    assert_eq!(ack_count, num_expected_pdatas);
                })
            })
    }

    #[test]
    pub fn test_uses_endpoint_overrides_if_provided() {
        let logs_port = pick_unused_port().unwrap();
        let logs_endpoint_addr = format!("127.0.0.1:{}", logs_port);
        let logs_endpoint = format!("http://{logs_endpoint_addr}/v1/logs");

        let metrics_port = pick_unused_port().unwrap();
        let metrics_endpoint_addr = format!("127.0.0.1:{}", metrics_port);
        let metrics_endpoint = format!("http://{metrics_endpoint_addr}/v1/metrics");

        let traces_port = pick_unused_port().unwrap();
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
                    let otlp_bytes: OtlpProtoBytes = pdata.take_payload().try_into().unwrap();
                    let pdata_decoded = LogsData::decode(otlp_bytes.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Logs(pdata_decoded)],
                        &[OtlpProtoMessage::Logs(logs_batch.clone())],
                    );

                    let mut pdata = metrics_pdata_rx.recv().await.unwrap();
                    let otlp_bytes: OtlpProtoBytes = pdata.take_payload().try_into().unwrap();
                    let pdata_decoded = MetricsData::decode(otlp_bytes.as_bytes()).unwrap();
                    assert_equivalent(
                        &[OtlpProtoMessage::Metrics(pdata_decoded)],
                        &[OtlpProtoMessage::Metrics(metrics_batch.clone())],
                    );

                    let mut pdata = traces_pdata_rx.recv().await.unwrap();
                    let otlp_bytes: OtlpProtoBytes = pdata.take_payload().try_into().unwrap();
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
}
