// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP exporter via HTTP
//!
//! This exporter sends telemetry data to an OTLP server using the HTTP Protocol.
//!
//! ToDo:
//! - support for TLS/mTLS
//! - support JSON encoding payloads (currently only proto is supported and it's not configurable)

use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::FutureExt;
use http::{HeaderMap, HeaderValue};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::Error as EngineError;
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
use tower::limit::{ConcurrencyLimit, ConcurrencyLimitLayer, GlobalConcurrencyLimitLayer};

use crate::OTAP_EXPORTER_FACTORIES;
use crate::metrics::ExporterPDataMetrics;
use crate::otlp_exporter::InFlightExports;
use crate::otlp_http::PROTOBUF_CONTENT_TYPE;
use crate::otlp_http::client_settings::HttpClientSettings;
use crate::otlp_http_exporter::config::Config;
use crate::pdata::{Context, OtapPdata};

mod config;

/// The URN for the OTLP HTTP exporter
pub const OTLP_HTTP_EXPORTER_URN: &str = "urn:otel:otlphttp:exporter";

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
        otel_info!(
            "otlp.exporter.http.start",
            http_endpoint = self.config.http.endpoint.as_str()
        );

        self.config.http.log_proxy_info();

        let telemetery_timer_cancel = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let max_in_flight = self.config.max_in_flight.max(1);
        // TODO no unwrap
        let mut client_pool =
            HttpClientPool::try_new(&self.config.http, self.config.client_pool_size).unwrap();

        let mut inflight_exports = InFlightExports::new();

        let mut logs_proto_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_proto_encoder = TracesProtoBytesEncoder::new();
        let mut proto_buffer = ProtoBuffer::with_capacity(8 * 1024);

        // TODO these could be consts
        let logs_endpoint = Rc::new(format!("{}/v1/logs", self.config.http.endpoint));
        let metrics_endpoint = Rc::new(format!("{}/v1/metrics", self.config.http.endpoint));
        let traces_endpoint = Rc::new(format!("{}/v1/traces", self.config.http.endpoint));

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
                    otel_info!("otlp.exporter.http.shutdown");
                    while !inflight_exports.is_empty() {
                        if let Some(completed) = inflight_exports.next_completion().await {
                            finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.pdata_metrics,
                            )
                            .await;
                        }
                        _ = telemetery_timer_cancel.cancel().await;
                        return Ok(TerminalState::new(deadline, [self.pdata_metrics]));
                    }
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    todo!("handle collect telemetry")
                }
                Message::PData(pdata) => {
                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();

                    // proto encode the payload into the request body, while keeping a copy of the
                    // original payload if the context allows it to be returned
                    let (body, saved_payload) = match payload {
                        OtapPayload::OtlpBytes(mut otlp_bytes) => {
                            if context.may_return_payload() {
                                // copy the bytes to the request body
                                let body = Bytes::copy_from_slice(otlp_bytes.as_bytes());
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
                            let body = if let Err(e) = encode_result {
                                todo!("handle encode error")
                            } else {
                                Bytes::copy_from_slice(proto_buffer.as_ref())
                            };

                            if !context.may_return_payload() {
                                // drop the original OTAP batch if the the context indicates it
                                // does not wish it to be returned
                                _ = otap_batch.take_payload();
                            }

                            (body, otap_batch.into())
                        }
                    };

                    let endpoint = Rc::clone(match signal_type {
                        SignalType::Logs => &logs_endpoint,
                        SignalType::Metrics => &metrics_endpoint,
                        SignalType::Traces => &traces_endpoint,
                    });

                    let client = client_pool.get_client();
                    inflight_exports.push(async move {
                        let result = client.post(endpoint.as_str()).body(body).send().await;

                        CompletedExport {
                            result: query_result_to_service_response(&signal_type, result).await,
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

// TODO comments
#[derive(thiserror::Error, Debug)]
enum ServiceRequestError {
    #[error("An error occurred sending HTTP request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("An error occurred decoding response body: {0}")]
    DecodeError(#[from] prost::DecodeError),
}

async fn query_result_to_service_response(
    signal_type: &SignalType,
    result: Result<Response, reqwest::Error>,
) -> Result<ServiceResponse, ServiceRequestError> {
    let mut bytes = result?.bytes().await?;

    let service_resp = match signal_type {
        SignalType::Logs => ExportLogsServiceResponse::decode(&mut bytes).map(Into::into),
        SignalType::Metrics => ExportMetricsServiceResponse::decode(&mut bytes).map(Into::into),
        SignalType::Traces => ExportTraceServiceResponse::decode(&mut bytes).map(Into::into),
    };

    Ok(service_resp?)
}

async fn finalize_completed_export(
    completed: CompletedExport,
    effect_handler: &EffectHandler<OtapPdata>,
    pdata_metrics: &mut MetricSet<ExporterPDataMetrics>,
) {
    println!("ici 1: {:?}", completed);

    let CompletedExport {
        result,
        context,
        saved_payload,
        signal_type,
    } = completed;

    let pdata = OtapPdata::new(context, saved_payload);

    let err_msg = match result {
        Ok(service_resp) => service_resp.partial_success.map(|partial_success| {
            // TODO - should use count rejected to populate an error here if error_message
            // is empty string? spec says it SHOULD be populated, but I guess that doesn't
            // mean it always will be:
            // https://opentelemetry.io/docs/specs/otlp/#partial-success-1
            partial_success.error_message
        }),
        Err(e) => Some(e.to_string()),
    };

    // TODO - should we peek and log the errors here?

    let export_and_notify_success = match err_msg {
        // TODO should we peek and log error if we couldn't Ack/NAck here?
        None => effect_handler.notify_ack(AckMsg::new(pdata)).await.is_ok(),
        Some(err_msg) => {
            _ = effect_handler
                .notify_nack(NackMsg::new(&err_msg, pdata))
                .await;
            false
        }
    };

    if export_and_notify_success {
        pdata_metrics.add_exported(signal_type, 1)
    } else {
        pdata_metrics.add_failed(signal_type, 1)
    }
}

// TODO comments on this
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

        // TODO - this value should be dynamic once we support JSON OTLP payloads
        _ = default_headers.insert(
            http::header::CONTENT_TYPE,
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
    use std::net::SocketAddr;
    use std::time::{Duration, Instant};

    use otap_df_config::PortName;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::shared::message::SharedSender;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::node::test_node;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Metric, MetricsData, ResourceMetrics, ScopeMetrics,
    };
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, TracesData};
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use parking_lot::lock_api::Mutex;
    use portpicker::pick_unused_port;
    use prost::Message;
    use tokio::runtime::Runtime;
    use tokio_util::sync::CancellationToken;

    use super::*;

    use crate::otap_grpc::common::AckRegistry;
    use crate::otlp_http::client_settings::HttpClientSettings;
    use crate::otlp_http::{HttpServerSettings, serve, tune_max_concurrent_requests};
    use crate::otlp_receiver::OtlpReceiverMetrics;

    #[test]
    fn test_exports_otlp_signals() {
        let port = pick_unused_port().unwrap();
        let endpoint_addr = format!("127.0.0.1:{}", port);
        let endpoint = format!("http://{endpoint_addr}");

        // TODO - can we make this easier to init?
        // TODO - just a bunch of random values selected?
        let config = Config {
            http: HttpClientSettings {
                endpoint: endpoint.clone(),
                compression: None,
                concurrency_limit: 1,
                connect_timeout: Duration::from_millis(100),
                tcp_nodelay: false,
                tcp_keepalive: None,
                tcp_keepalive_interval: None,
                timeout: None,
                proxy: None,
            },
            client_pool_size: NonZeroUsize::try_from(2).unwrap(),
            max_in_flight: 10,
        };

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_HTTP_EXPORTER_URN));
        // let telemetry_registry_handle = TelemetryRegistryHandle::new();
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

        let mut exporter = ExporterWrapper::local(
            OtlpHttpExporter {
                config: config,
                pdata_metrics: pipeline_ctx.register_metrics::<ExporterPDataMetrics>(),
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        // server setup stuff (TODO move to func?)
        let server_node_id = test_node(format!("{test_runtime_name}-server"));
        let port_name = PortName::from("server_out");
        let mut msg_senders = HashMap::new();
        let (pdata_tx, mut pdata_rx) = tokio::sync::mpsc::channel(10);
        _ = msg_senders.insert(port_name.clone(), SharedSender::mpsc(pdata_tx));

        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);
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

        // TODO need a way to detect if server is ready?
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
                    loop {
                        match pdata_rx.recv().await {
                            Some(pdata) => {
                                pdatas_received.push(pdata);
                                if pdatas_received.len() >= num_expected_pdatas {
                                    server_cancellation_token2.cancel();
                                }
                            }
                            None => break,
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

                    // validate we received three Acks
                    let mut ack_count = 0;
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();
                    loop {
                        let msg = match pipeline_ctrl_rx.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break, // channel closed, no more messages will be received
                        };

                        match msg {
                            PipelineControlMsg::DeliverAck { node_id, .. } => {
                                assert_eq!(node_id, node_id);
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

                    // validate we updated our metrics
                    // TODO - this doesn't seem to be working be
                    // telemetry_registry_handle.visit_current_metrics(|metric_desc, _attrs, iter| {
                    //     for (field, value) in iter {
                    //         println!("field = {:?}, value = {:?}", field, value);
                    //     }
                    // });
                })
            })
    }
}
