// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP Filter processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Collect telemetry like number of filtered data is removed datapoints

use self::config::Config;
use self::metrics::FilterPdataMetrics;
use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ProcessorErrorKind, format_error_sources};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_telemetry::metrics::MetricSet;
use serde_json::Value;
use std::sync::Arc;

mod config;
mod metrics;
/// The URN for the filter processor
pub const FILTER_PROCESSOR_URN: &str = "urn:otel:filter:processor";

/// processor that outputs all data received to stdout
pub struct FilterProcessor {
    config: Config,
    metrics: MetricSet<FilterPdataMetrics>,
}

/// Factory function to create a FilterProcessor.
///
/// See the module documentation for configuration examples
pub fn create_filter_processor(
    pipeline_ctx: PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    Ok(ProcessorWrapper::local(
        FilterProcessor::from_config(pipeline_ctx, &node_config.config)?,
        node,
        node_config,
        processor_config,
    ))
}

/// Register FilterProcessor as an OTAP processor factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static FILTER_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: FILTER_PROCESSOR_URN,
        create: |pipeline_ctx: PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_filter_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

impl FilterProcessor {
    /// Creates a new FilterProcessor
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, pipeline_ctx: PipelineContext) -> Self {
        let metrics = pipeline_ctx.register_metrics::<FilterPdataMetrics>();
        FilterProcessor { config, metrics }
    }

    /// Creates a new FilterProcessor from a configuration object
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, ConfigError> {
        let metrics = pipeline_ctx.register_metrics::<FilterPdataMetrics>();
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Ok(FilterProcessor { config, metrics })
    }
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for FilterProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect_handler: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match msg {
            Message::Control(control) => {
                if let NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } = control
                {
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Ok(())
            }
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                // convert to arrow records
                let (context, payload) = pdata.into_parts();

                let mut arrow_records: OtapArrowRecords = payload.try_into()?;
                arrow_records.decode_transport_optimized_ids()?;

                let filtered_arrow_records: OtapArrowRecords = match signal {
                    SignalType::Metrics => {
                        // ToDo: Add support for metrics
                        arrow_records
                    }
                    SignalType::Logs => {
                        // get logs
                        let (filtered_arrow_records, log_signals_consumed, log_signals_filtered) =
                            self.config
                                .log_filters()
                                .filter(arrow_records)
                                .map_err(|e| {
                                    let source_detail = format_error_sources(&e);
                                    Error::ProcessorError {
                                        processor: effect_handler.processor_id(),
                                        kind: ProcessorErrorKind::Other,
                                        error: format!("Filter error: {e}"),
                                        source_detail,
                                    }
                                })?;

                        // get logs after
                        self.metrics.log_signals_consumed.add(log_signals_consumed);
                        self.metrics.log_signals_filtered.add(log_signals_filtered);

                        filtered_arrow_records
                    }
                    SignalType::Traces => {
                        // get spans
                        let (filtered_arrow_records, span_signals_consumed, span_signals_filtered) =
                            self.config
                                .trace_filters()
                                .filter(arrow_records)
                                .map_err(|e| {
                                    let source_detail = format_error_sources(&e);
                                    Error::ProcessorError {
                                        processor: effect_handler.processor_id(),
                                        kind: ProcessorErrorKind::Other,
                                        error: format!("Filter error: {e}"),
                                        source_detail,
                                    }
                                })?;

                        self.metrics
                            .span_signals_consumed
                            .add(span_signals_consumed);
                        self.metrics
                            .span_signals_filtered
                            .add(span_signals_filtered);

                        filtered_arrow_records
                    }
                };
                effect_handler
                    .send_message_with_source_node(OtapPdata::new(
                        context,
                        filtered_arrow_records.into(),
                    ))
                    .await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::filter_processor::{FILTER_PROCESSOR_URN, FilterProcessor, config::Config};
    use crate::pdata::OtapPdata;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::processor::{TestContext, ValidateContext};
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::otap::filter::{
        AnyValue as AnyValueFilter, KeyValue as KeyValueFilter, MatchType,
        logs::{LogFilter, LogMatchProperties, LogSeverityNumberMatchProperties},
        traces::{TraceFilter, TraceMatchProperties},
    };
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span, Status, TracesData,
            span::{Event, Link, SpanKind},
            status::StatusCode,
        },
    };
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message as _;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    // build logs data for testing version 1
    fn build_logs_1() -> LogsData {
        LogsData::new(vec![
            // ResourceLogs for prod/checkout-service
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("version", AnyValue::new_string("2.0")),
                        KeyValue::new("service.name", AnyValue::new_string("checkout-service")),
                        KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                        KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                        KeyValue::new("deployment.environment", AnyValue::new_string("prod")),
                        KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                        KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                        KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                        KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                        KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                        KeyValue::new("process.pid", AnyValue::new_int(12345)),
                        KeyValue::new("team", AnyValue::new_string("payments")),
                        KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                    ])
                    .finish(),
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("library")
                        .version("scopev1")
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(2_200_000_000u64)
                            .severity_number(SeverityNumber::Warn)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("component", AnyValue::new_string("rest")),
                                KeyValue::new("http.method", AnyValue::new_string("POST")),
                                KeyValue::new("http.route", AnyValue::new_string("/api/checkout")),
                                KeyValue::new("http.status_code", AnyValue::new_int(200)),
                                KeyValue::new("retryable", AnyValue::new_bool(false)),
                                KeyValue::new("backoff_ms", AnyValue::new_int(0)),
                                KeyValue::new("throttle_factor", AnyValue::new_double(0.0)),
                            ])
                            .severity_text("WARN")
                            .body(AnyValue::new_string("checkout started"))
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(2_300_000_000u64)
                            .severity_number(SeverityNumber::Error)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                                KeyValue::new(
                                    "exception.message",
                                    AnyValue::new_string("connection reset by peer"),
                                ),
                                KeyValue::new(
                                    "exception.stacktrace",
                                    AnyValue::new_string("...stack..."),
                                ),
                                KeyValue::new(
                                    "peer.address",
                                    AnyValue::new_string("10.42.0.7:5432"),
                                ),
                                KeyValue::new("peer.port", AnyValue::new_int(5432)),
                                KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                                KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                                KeyValue::new("jitter", AnyValue::new_double(0.003)),
                            ])
                            .severity_text("ERROR")
                            .body(AnyValue::new_string("failed to write to socket"))
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(2_600_000_000u64)
                            .severity_number(SeverityNumber::Warn)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("component", AnyValue::new_string("db")),
                                KeyValue::new(
                                    "query",
                                    AnyValue::new_string(
                                        "UPDATE inventory SET count = count - 1 WHERE sku='ABC123'",
                                    ),
                                ),
                                KeyValue::new("rows_affected", AnyValue::new_int(1)),
                                KeyValue::new("success", AnyValue::new_bool(true)),
                            ])
                            .severity_text("WARN")
                            .body(AnyValue::new_string("inventory updated"))
                            .finish(),
                    ],
                )],
            ),
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("version", AnyValue::new_string("2.0")),
                        KeyValue::new("service.name", AnyValue::new_string("inventory-service")),
                        KeyValue::new("service.version", AnyValue::new_string("0.9.1")),
                        KeyValue::new("service.instance.number", AnyValue::new_int(7)),
                        KeyValue::new("deployment.environment", AnyValue::new_string("staging")),
                        KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                        KeyValue::new("host.cpu_cores", AnyValue::new_int(4)),
                        KeyValue::new("host.uptime_sec", AnyValue::new_int(12_345)),
                        KeyValue::new("sampling.rate", AnyValue::new_double(0.10)),
                        KeyValue::new("debug.enabled", AnyValue::new_bool(true)),
                        KeyValue::new("process.pid", AnyValue::new_int(22222)),
                        KeyValue::new("team", AnyValue::new_string("inventory")),
                    ])
                    .finish(),
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("library")
                        .version("scopev2")
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(3_000_000_000u64)
                            .severity_number(SeverityNumber::Warn)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("component", AnyValue::new_string("rest")),
                                KeyValue::new(
                                    "http.route",
                                    AnyValue::new_string("/api/internal/cache_warm"),
                                ),
                                KeyValue::new("http.status_code", AnyValue::new_int(200)),
                                KeyValue::new("success", AnyValue::new_bool(true)),
                            ])
                            .severity_text("WARN")
                            .body(AnyValue::new_string("warmup complete"))
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(3_100_000_000u64)
                            .severity_number(SeverityNumber::Info)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("event.domain", AnyValue::new_string("ops")),
                                KeyValue::new("message", AnyValue::new_string("heartbeat")),
                                KeyValue::new("uptime_sec", AnyValue::new_int(100)),
                            ])
                            .severity_text("INFO")
                            .body(AnyValue::new_string("heartbeat"))
                            .finish(),
                    ],
                )],
            ),
        ])
    }

    // build logs data for testing version 2
    fn build_logs_2() -> LogsData {
        LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("version", AnyValue::new_string("2.0")),
                    KeyValue::new(
                        "service.name",
                        AnyValue::new_string("payments-checkout-service"),
                    ),
                    KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                    KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                    // Use "production" to exercise regex ^prod(uction)?$
                    KeyValue::new("deployment.environment", AnyValue::new_string("production")),
                    KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                    KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                    KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                    KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                    KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                    KeyValue::new("process.pid", AnyValue::new_int(12345)),
                    KeyValue::new("team", AnyValue::new_string("payments")),
                    KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    // Log A: WARN with string body; attributes include int/float/bool
                    LogRecord::build()
                        .time_unix_nano(2_200_000_000u64)
                        .severity_number(SeverityNumber::Warn)
                        .body(AnyValue::new_string("event"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("feature_flag", AnyValue::new_bool(true)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("ok checkout #1"))
                        .finish(),
                    // Log B: ERROR with numeric (double) body; attributes include int/float/bool
                    LogRecord::build()
                        .time_unix_nano(2_300_000_000u64)
                        .severity_number(SeverityNumber::Error)
                        .body(AnyValue::new_string("event"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("svc")),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("attempt", AnyValue::new_int(3)),
                            KeyValue::new("error_rate", AnyValue::new_double(0.02)),
                        ])
                        .severity_text("ERROR")
                        .body(AnyValue::new_double(12.5))
                        .finish(),
                ],
            )],
        )])
    }

    fn build_traces() -> TracesData {
        // TracesData dataset (2 ResourceSpans, 4 Spans total). Links omit trace_id/span_id.
        // Mixed attribute types across resource, span, event, and link levels.
        TracesData::new(vec![
    // Resource 0: production payments/checkout
    ResourceSpans::new(
        Resource::build().attributes(vec![
            KeyValue::new("version", AnyValue::new_string("2.0")),
            KeyValue::new("service.name", AnyValue::new_string("payments-checkout-service")),
            KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
            KeyValue::new("service.instance.number", AnyValue::new_int(42)),
            KeyValue::new("deployment.environment", AnyValue::new_string("production")),
            KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
            KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
            KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
            KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
        ]).finish(),
        vec![
            ScopeSpans::new(
                InstrumentationScope::build().name("library").version("scopev1").finish(),
                vec![
                    // P1: checkout-warn
                    Span::build()
                        .name("checkout-warn")
                        .trace_state("state-p1")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("checkout-event")
                                .attributes(vec![
                                    KeyValue::new("message", AnyValue::new_string("ok checkout #1")),
                                    KeyValue::new("attempt", AnyValue::new_int(1)),
                                    KeyValue::new("success", AnyValue::new_bool(true)),
                                    KeyValue::new("duration_ms", AnyValue::new_double(12.5)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            // no trace_id/span_id
                            Link::build()
                                .trace_state("link-state-1")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.99)),
                                ])
                                .finish(),
                        ])
                        .finish(),

                    // P2: checkout-error
                    Span::build()
                        .name("checkout-error")
                        .trace_state("state-p2")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Error, "payment failed"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("svc")),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("attempt", AnyValue::new_int(3)),
                            KeyValue::new("error_rate", AnyValue::new_double(0.02)),
                            KeyValue::new("http.status_code", AnyValue::new_int(500)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("payment-event")
                                .attributes(vec![
                                    KeyValue::new("message", AnyValue::new_string("payment error: insufficient funds")),
                                    KeyValue::new("amount", AnyValue::new_double(149.99)),
                                    KeyValue::new("success", AnyValue::new_bool(false)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-2")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("external")),
                                    KeyValue::new("correlation", AnyValue::new_bool(false)),
                                    KeyValue::new("retries", AnyValue::new_int(1)),
                                    KeyValue::new("quality", AnyValue::new_double(0.75)),
                                ])
                                .finish(),
                        ])
                        .finish(),

                    // P3: db-update-warn
                    Span::build()
                        .name("db-update-warn")
                        .trace_state("state-p3")
                        .kind(SpanKind::Internal)
                        .status(Status::new(StatusCode::Ok, "db updated"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("db")),
                            KeyValue::new("rows_affected", AnyValue::new_int(1)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                            KeyValue::new("latency_ms", AnyValue::new_double(4.7)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("db-event")
                                .attributes(vec![
                                    KeyValue::new("query", AnyValue::new_string("UPDATE inventory SET count = count - 1 WHERE sku='ABC123'")),
                                    KeyValue::new("message", AnyValue::new_string("inventory updated")),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-3")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.95)),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ],
            ),
        ],
    ),

    // Resource 1: staging inventory (1 span)
    ResourceSpans::new(
        Resource::build().attributes(vec![
            KeyValue::new("version", AnyValue::new_string("2.0")),
            KeyValue::new("service.name", AnyValue::new_string("inventory-svc")),
            KeyValue::new("service.version", AnyValue::new_string("0.9.1")),
            KeyValue::new("service.instance.number", AnyValue::new_int(7)),
            KeyValue::new("deployment.environment", AnyValue::new_string("staging-eu")),
            KeyValue::new("cloud.region", AnyValue::new_string("europe-west1")),
            KeyValue::new("host.cpu_cores", AnyValue::new_int(4)),
            KeyValue::new("sampling.rate", AnyValue::new_double(0.10)),
            KeyValue::new("debug.enabled", AnyValue::new_bool(true)),
        ]).finish(),
        vec![
            ScopeSpans::new(
                InstrumentationScope::build().name("library").version("scopev2").finish(),
                vec![
                    // S1: cache-warm-warn
                    Span::build()
                        .name("cache-warm-warn")
                        .trace_state("state-s1")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Ok, "cache warm"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("cache")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("warmup-event")
                                .attributes(vec![
                                    KeyValue::new("message", AnyValue::new_string("warmup complete")),
                                    KeyValue::new("duration_ms", AnyValue::new_double(3.0)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-s1")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ],
            ),
        ],
    ),
])
    }

    /// Validation closure that checks the outputted data
    fn validation_procedure() -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut _ctx| Box::pin(async move {})
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario_logs(
        sent: LogsData,
        expected: LogsData,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |mut ctx| {
            Box::pin(async move {
                //convert logsdata to otappdata
                let mut bytes = vec![];
                sent.encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_logs_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes.into()).into());
                ctx.process(Message::PData(otlp_logs_bytes))
                    .await
                    .expect("failed to process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                let received_logs_data = &msgs[0];
                let (_, payload) = received_logs_data.clone().into_parts();
                let otlp_bytes: OtlpProtoBytes = payload
                    .try_into()
                    .expect("failed to convert to OtlpProtoBytes");
                let received_logs_data = match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => LogsData::decode(bytes.as_ref())
                        .expect("failed to decode logs into logsdata"),
                    _ => panic!("expected logs type"),
                };

                assert_eq!(received_logs_data, expected);
            })
        }
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario_traces(
        expected: TracesData,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |mut ctx| {
            Box::pin(async move {
                //convert tracesdata to otappdata
                let traces_data = build_traces();
                let mut bytes = vec![];
                traces_data
                    .encode(&mut bytes)
                    .expect("failed to encode trace data into bytes");
                let otlp_traces_bytes = OtapPdata::new_default(
                    OtlpProtoBytes::ExportTracesRequest(bytes.into()).into(),
                );
                ctx.process(Message::PData(otlp_traces_bytes))
                    .await
                    .expect("failed to process");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                let received_traces_data = &msgs[0];
                let (_, payload) = received_traces_data.clone().into_parts();
                let otlp_bytes: OtlpProtoBytes = payload
                    .try_into()
                    .expect("failed to convert to OtlpProtoBytes");
                let received_traces_data = match otlp_bytes {
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        TracesData::decode(bytes.as_ref())
                            .expect("failed to decode traces into tracesdata")
                    }
                    _ => panic!("expected traces type"),
                };

                assert_eq!(received_traces_data, expected);
            })
        }
    }

    #[test]
    fn test_filter_processor_logs_strict() {
        let test_runtime = TestRuntime::new();

        let include_props = LogMatchProperties::new(
            MatchType::Strict,
            vec![
                KeyValueFilter::new(
                    "deployment.environment".to_string(),
                    AnyValueFilter::String("prod".to_string()),
                ),
                KeyValueFilter::new(
                    "service.instance.number".to_string(),
                    AnyValueFilter::Int(42),
                ),
                KeyValueFilter::new("sampling.rate".to_string(), AnyValueFilter::Double(0.25)),
                KeyValueFilter::new("debug.enabled".to_string(), AnyValueFilter::Boolean(false)),
            ],
            vec![
                KeyValueFilter::new("peer.port".to_string(), AnyValueFilter::Int(5432)),
                KeyValueFilter::new("jitter".to_string(), AnyValueFilter::Double(0.003)),
            ],
            vec!["WARN".to_string(), "ERROR".to_string()], // test severity_texts
            Some(LogSeverityNumberMatchProperties::new(13, true)), // WARN and above
            vec![
                AnyValueFilter::String("checkout started".to_string()),
                AnyValueFilter::String("failed to write to socket".to_string()),
            ], // test bodies
        );

        let exclude_props = LogMatchProperties::new(
            MatchType::Strict,
            vec![KeyValueFilter::new(
                "deployment.environment".to_string(),
                AnyValueFilter::String("staging".to_string()),
            )],
            vec![
                KeyValueFilter::new(
                    "component".to_string(),
                    AnyValueFilter::String("db".to_string()),
                ),
                KeyValueFilter::new("retryable".to_string(), AnyValueFilter::Boolean(true)),
            ],
            vec!["WARN".to_string()], // exclude WARN by severity_texts
            None,
            vec![
                AnyValueFilter::String("checkout started".to_string()), // exclude by body
                AnyValueFilter::Int(10),
                AnyValueFilter::Double(12.0),
                AnyValueFilter::Boolean(true),
            ],
        );
        let log_filter = LogFilter::new(
            Some(include_props),
            Some(exclude_props),
            Vec::new(), // log_record ignored
        );

        let trace_filter = TraceFilter::new(None, None);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        let expected_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("version", AnyValue::new_string("2.0")),
                    KeyValue::new("service.name", AnyValue::new_string("checkout-service")),
                    KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                    KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                    KeyValue::new("deployment.environment", AnyValue::new_string("prod")),
                    KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                    KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                    KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                    KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                    KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                    KeyValue::new("process.pid", AnyValue::new_int(12345)),
                    KeyValue::new("team", AnyValue::new_string("payments")),
                    KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    LogRecord::build()
                        .time_unix_nano(2_300_000_000u64)
                        .severity_number(SeverityNumber::Error)
                        .body(AnyValue::new_string("event"))
                        .attributes(vec![
                            KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                            KeyValue::new(
                                "exception.message",
                                AnyValue::new_string("connection reset by peer"),
                            ),
                            KeyValue::new(
                                "exception.stacktrace",
                                AnyValue::new_string("...stack..."),
                            ),
                            KeyValue::new("peer.address", AnyValue::new_string("10.42.0.7:5432")),
                            KeyValue::new("peer.port", AnyValue::new_int(5432)),
                            KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                            KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                            KeyValue::new("jitter", AnyValue::new_double(0.003)),
                        ])
                        .severity_text("ERROR")
                        .body(AnyValue::new_string("failed to write to socket"))
                        .finish(),
                ],
            )],
        )]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_logs(build_logs_1(), expected_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_logs_regex() {
        let test_runtime = TestRuntime::new();

        let include_props = LogMatchProperties::new(
            MatchType::Regexp,
            vec![
                KeyValueFilter::new(
                    "deployment.environment".to_string(),
                    AnyValueFilter::String(r"^prod(uction)?$".to_string()),
                ),
                KeyValueFilter::new(
                    "service.name".to_string(),
                    AnyValueFilter::String(r".*checkout.*".to_string()),
                ),
            ],
            Vec::new(), // record_attributes
            vec![r"^(WARN|ERROR)$".to_string()],
            Some(LogSeverityNumberMatchProperties::new(13, false)), // WARN+
            vec![
                AnyValueFilter::String(r"^ok.*".to_string()), // matches "ok checkout #1"
                AnyValueFilter::Double(12.5),                 // matches numeric body 12.5 exactly
            ],
        );

        let exclude_props = LogMatchProperties::new(
            MatchType::Regexp,
            vec![KeyValueFilter::new(
                "deployment.environment".to_string(),
                AnyValueFilter::String(r"^stag.*".to_string()),
            )],
            vec![
                KeyValueFilter::new(
                    "component".to_string(),
                    AnyValueFilter::String(r"^(db|cache)$".to_string()),
                ),
                // Also test boolean attribute exclusion: drop if retryable == true
                KeyValueFilter::new("retryable".to_string(), AnyValueFilter::Boolean(true)),
                KeyValueFilter::new("error_rate".to_string(), AnyValueFilter::Double(0.02)),
            ],
            vec![r"^DEBUG$".to_string()],
            None,
            vec![
                AnyValueFilter::String(r"^DELETE .+".to_string()),
                AnyValueFilter::String(r"^heartbeat$".to_string()),
            ],
        );
        let log_filter = LogFilter::new(
            Some(include_props),
            Some(exclude_props),
            Vec::new(), // log_record ignored
        );

        let trace_filter = TraceFilter::new(None, None);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );
        let expected_logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("version", AnyValue::new_string("2.0")),
                    KeyValue::new(
                        "service.name",
                        AnyValue::new_string("payments-checkout-service"),
                    ),
                    KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                    KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                    KeyValue::new("deployment.environment", AnyValue::new_string("production")),
                    KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                    KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                    KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                    KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                    KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                    KeyValue::new("process.pid", AnyValue::new_int(12345)),
                    KeyValue::new("team", AnyValue::new_string("payments")),
                    KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    // Kept: WARN with string body
                    LogRecord::build()
                        .time_unix_nano(2_200_000_000u64)
                        .severity_number(SeverityNumber::Warn)
                        .body(AnyValue::new_string("event"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("feature_flag", AnyValue::new_bool(true)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("ok checkout #1"))
                        .finish(),
                ],
            )],
        )]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_logs(build_logs_2(), expected_logs_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_logs_strict_include_only() {
        let test_runtime = TestRuntime::new();

        let include_props = LogMatchProperties::new(
            MatchType::Strict,
            vec![
                KeyValueFilter::new(
                    "deployment.environment".to_string(),
                    AnyValueFilter::String("prod".to_string()),
                ),
                KeyValueFilter::new(
                    "service.instance.number".to_string(),
                    AnyValueFilter::Int(42),
                ),
                KeyValueFilter::new("sampling.rate".to_string(), AnyValueFilter::Double(0.25)),
                KeyValueFilter::new("debug.enabled".to_string(), AnyValueFilter::Boolean(false)),
            ],
            vec![
                KeyValueFilter::new("peer.port".to_string(), AnyValueFilter::Int(5432)),
                KeyValueFilter::new("jitter".to_string(), AnyValueFilter::Double(0.003)),
            ],
            vec!["WARN".to_string(), "ERROR".to_string()], // test severity_texts
            Some(LogSeverityNumberMatchProperties::new(13, true)), // WARN and above
            vec![
                AnyValueFilter::String("checkout started".to_string()),
                AnyValueFilter::String("failed to write to socket".to_string()),
            ], // test bodies
        );

        let log_filter = LogFilter::new(
            Some(include_props),
            None,
            Vec::new(), // log_record ignored
        );

        let trace_filter = TraceFilter::new(None, None);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );
        let expected_logs_data = LogsData::new(vec![
            // ResourceLogs for prod/checkout-service
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("version", AnyValue::new_string("2.0")),
                        KeyValue::new("service.name", AnyValue::new_string("checkout-service")),
                        KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                        KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                        KeyValue::new("deployment.environment", AnyValue::new_string("prod")),
                        KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                        KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                        KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                        KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                        KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                        KeyValue::new("process.pid", AnyValue::new_int(12345)),
                        KeyValue::new("team", AnyValue::new_string("payments")),
                        KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                    ])
                    .finish(),
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("library")
                        .version("scopev1")
                        .finish(),
                    vec![
                        // ERROR: passes include; not removed by exclude
                        LogRecord::build()
                            .time_unix_nano(2_300_000_000u64)
                            .severity_number(SeverityNumber::Error)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                                KeyValue::new(
                                    "exception.message",
                                    AnyValue::new_string("connection reset by peer"),
                                ),
                                KeyValue::new(
                                    "exception.stacktrace",
                                    AnyValue::new_string("...stack..."),
                                ),
                                KeyValue::new(
                                    "peer.address",
                                    AnyValue::new_string("10.42.0.7:5432"),
                                ),
                                KeyValue::new("peer.port", AnyValue::new_int(5432)),
                                KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                                KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                                KeyValue::new("jitter", AnyValue::new_double(0.003)),
                            ])
                            .severity_text("ERROR")
                            .body(AnyValue::new_string("failed to write to socket"))
                            .finish(),
                    ],
                )],
            ),
        ]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_logs(build_logs_1(), expected_logs_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_logs_strict_exclude_only() {
        let test_runtime = TestRuntime::new();

        let exclude_props = LogMatchProperties::new(
            MatchType::Strict,
            vec![KeyValueFilter::new(
                "deployment.environment".to_string(),
                AnyValueFilter::String("staging".to_string()),
            )],
            vec![
                KeyValueFilter::new(
                    "component".to_string(),
                    AnyValueFilter::String("db".to_string()),
                ),
                KeyValueFilter::new("retryable".to_string(), AnyValueFilter::Boolean(true)),
            ],
            vec!["INFO".to_string()],
            None,
            vec![
                AnyValueFilter::String("inventory updated".to_string()), // exclude by body
                AnyValueFilter::String("warmup complete".to_string()),
                AnyValueFilter::String("heartbeat".to_string()),
            ],
        );

        let log_filter = LogFilter::new(
            None,
            Some(exclude_props),
            Vec::new(), // log_record ignored
        );

        let trace_filter = TraceFilter::new(None, None);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );
        let expected_logs_data = LogsData::new(vec![
            // ResourceLogs for prod/checkout-service
            ResourceLogs::new(
                Resource::build()
                    .attributes(vec![
                        KeyValue::new("version", AnyValue::new_string("2.0")),
                        KeyValue::new("service.name", AnyValue::new_string("checkout-service")),
                        KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                        KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                        KeyValue::new("deployment.environment", AnyValue::new_string("prod")),
                        KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                        KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                        KeyValue::new("host.uptime_sec", AnyValue::new_int(86_400)),
                        KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                        KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                        KeyValue::new("process.pid", AnyValue::new_int(12345)),
                        KeyValue::new("team", AnyValue::new_string("payments")),
                        KeyValue::new("telemetry.sdk.language", AnyValue::new_string("rust")),
                    ])
                    .finish(),
                vec![ScopeLogs::new(
                    InstrumentationScope::build()
                        .name("library")
                        .version("scopev1")
                        .finish(),
                    vec![
                        LogRecord::build()
                            .time_unix_nano(2_200_000_000u64)
                            .severity_number(SeverityNumber::Warn)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("component", AnyValue::new_string("rest")),
                                KeyValue::new("http.method", AnyValue::new_string("POST")),
                                KeyValue::new("http.route", AnyValue::new_string("/api/checkout")),
                                KeyValue::new("http.status_code", AnyValue::new_int(200)),
                                KeyValue::new("retryable", AnyValue::new_bool(false)),
                                KeyValue::new("backoff_ms", AnyValue::new_int(0)),
                                KeyValue::new("throttle_factor", AnyValue::new_double(0.0)),
                            ])
                            .severity_text("WARN")
                            .body(AnyValue::new_string("checkout started"))
                            .finish(),
                        LogRecord::build()
                            .time_unix_nano(2_300_000_000u64)
                            .severity_number(SeverityNumber::Error)
                            .body(AnyValue::new_string("event"))
                            .attributes(vec![
                                KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                                KeyValue::new(
                                    "exception.message",
                                    AnyValue::new_string("connection reset by peer"),
                                ),
                                KeyValue::new(
                                    "exception.stacktrace",
                                    AnyValue::new_string("...stack..."),
                                ),
                                KeyValue::new(
                                    "peer.address",
                                    AnyValue::new_string("10.42.0.7:5432"),
                                ),
                                KeyValue::new("peer.port", AnyValue::new_int(5432)),
                                KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                                KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                                KeyValue::new("jitter", AnyValue::new_double(0.003)),
                            ])
                            .severity_text("ERROR")
                            .body(AnyValue::new_string("failed to write to socket"))
                            .finish(),
                    ],
                )],
            ),
        ]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_logs(build_logs_1(), expected_logs_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_traces_strict() {
        let test_runtime = TestRuntime::new();

        // Scenario 1: Strict include + exclude
        // Expected kept spans: ["checkout-warn", "checkout-error"]
        let strict_include = TraceMatchProperties::new(
            MatchType::Strict,
            vec![
                KeyValueFilter::new(
                    "deployment.environment".to_string(),
                    AnyValueFilter::String("production".to_string()),
                ),
                KeyValueFilter::new(
                    "service.name".to_string(),
                    AnyValueFilter::String("payments-checkout-service".to_string()),
                ),
            ],
            Vec::new(), // span_attributes
            vec![
                "checkout-warn".to_string(),
                "checkout-error".to_string(),
                "db-update-warn".to_string(),
            ],
            vec![
                "checkout-event".to_string(),
                "payment-event".to_string(),
                "db-event".to_string(),
            ],
            Vec::new(), // event_attributes
            Vec::new(), // link_attributes
        );

        let strict_exclude = TraceMatchProperties::new(
            MatchType::Strict,
            vec![KeyValueFilter::new(
                "deployment.environment".to_string(),
                AnyValueFilter::String("staging-eu".to_string()),
            )],
            vec![KeyValueFilter::new(
                "component".to_string(),
                AnyValueFilter::String("db".to_string()),
            )],
            vec!["checkout-error".to_string()], // span_names
            vec!["payment-event".to_string()],  // event_names
            vec![KeyValueFilter::new(
                "success".to_string(),
                AnyValueFilter::Boolean(false),
            )], // event_attributes
            vec![KeyValueFilter::new(
                "correlation".to_string(),
                AnyValueFilter::Boolean(false),
            )], // link_attributes
        );

        let trace_filter = TraceFilter::new(Some(strict_include), Some(strict_exclude));

        let log_filter = LogFilter::new(None, None, vec![]);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        // Expected TracesData after applying Scenario 1: Strict include + exclude
        // Kept spans: ["checkout-warn", "checkout-error"]
        let expected_data = TracesData::new(vec![ResourceSpans::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("version", AnyValue::new_string("2.0")),
                    KeyValue::new(
                        "service.name",
                        AnyValue::new_string("payments-checkout-service"),
                    ),
                    KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                    KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                    KeyValue::new("deployment.environment", AnyValue::new_string("production")),
                    KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                    KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                    KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                    KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                ])
                .finish(),
            vec![ScopeSpans::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    Span::build()
                        .trace_id(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        .span_id(vec![0, 0, 0, 0, 0, 0, 0, 0])
                        .name("checkout-warn")
                        .trace_state("state-p1")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("checkout-event")
                                .attributes(vec![
                                    KeyValue::new(
                                        "message",
                                        AnyValue::new_string("ok checkout #1"),
                                    ),
                                    KeyValue::new("attempt", AnyValue::new_int(1)),
                                    KeyValue::new("success", AnyValue::new_bool(true)),
                                    KeyValue::new("duration_ms", AnyValue::new_double(12.5)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-1")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.99)),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ],
            )],
        )]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_traces(expected_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_traces_include_only() {
        let test_runtime = TestRuntime::new();
        // Scenario 3: Include-only (Strict)
        // Expected kept spans: ["checkout-warn", "checkout-error", "db-update-warn"]
        let include_only = TraceMatchProperties::new(
            MatchType::Strict,
            vec![
                KeyValueFilter::new(
                    "deployment.environment".to_string(),
                    AnyValueFilter::String("production".to_string()),
                ),
                KeyValueFilter::new(
                    "service.name".to_string(),
                    AnyValueFilter::String("payments-checkout-service".to_string()),
                ),
            ],
            Vec::new(), // span_attributes
            Vec::new(), // span_names
            Vec::new(), // event_names
            Vec::new(), // event_attributes
            Vec::new(), // link_attributes
        );

        let trace_filter = TraceFilter::new(Some(include_only), None);
        let log_filter = LogFilter::new(None, None, vec![]);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        // Expected TracesData after applying Scenario 3: Include-only (Strict)
        // Kept spans: ["checkout-warn", "checkout-error", "db-update-warn"]
        let expected_data = TracesData::new(vec![
    ResourceSpans::new(
        Resource::build().attributes(vec![
            KeyValue::new("version", AnyValue::new_string("2.0")),
            KeyValue::new("service.name", AnyValue::new_string("payments-checkout-service")),
            KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
            KeyValue::new("service.instance.number", AnyValue::new_int(42)),
            KeyValue::new("deployment.environment", AnyValue::new_string("production")),
            KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
            KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
            KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
            KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
        ]).finish(),
        vec![
            ScopeSpans::new(
                InstrumentationScope::build().name("library").version("scopev1").finish(),
                vec![
                    // P1: checkout-warn
                    Span::build()
                                            .trace_id(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        .span_id(vec![0, 0, 0, 0, 0, 0, 0, 0])
                        .name("checkout-warn")
                        .trace_state("state-p1")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("checkout-event")
                                .attributes(vec![
                                    KeyValue::new("message", AnyValue::new_string("ok checkout #1")),
                                    KeyValue::new("attempt", AnyValue::new_int(1)),
                                    KeyValue::new("success", AnyValue::new_bool(true)),
                                    KeyValue::new("duration_ms", AnyValue::new_double(12.5)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-1")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.99)),
                                ])
                                .finish(),
                        ])
                        .finish(),

                    // P2: checkout-error
                    Span::build()
                                            .trace_id(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        .span_id(vec![0, 0, 0, 0, 0, 0, 0, 0])
                        .name("checkout-error")
                        .trace_state("state-p2")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Error, "payment failed"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("svc")),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("attempt", AnyValue::new_int(3)),
                            KeyValue::new("error_rate", AnyValue::new_double(0.02)),
                            KeyValue::new("http.status_code", AnyValue::new_int(500)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("payment-event")
                                .attributes(vec![
                                    KeyValue::new("message", AnyValue::new_string("payment error: insufficient funds")),
                                    KeyValue::new("amount", AnyValue::new_double(149.99)),
                                    KeyValue::new("success", AnyValue::new_bool(false)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-2")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("external")),
                                    KeyValue::new("correlation", AnyValue::new_bool(false)),
                                    KeyValue::new("retries", AnyValue::new_int(1)),
                                    KeyValue::new("quality", AnyValue::new_double(0.75)),
                                ])
                                .finish(),
                        ])
                        .finish(),

                    // P3: db-update-warn
                    Span::build()
                                            .trace_id(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        .span_id(vec![0, 0, 0, 0, 0, 0, 0, 0])
                        .name("db-update-warn")
                        .trace_state("state-p3")
                        .kind(SpanKind::Internal)
                        .status(Status::new(StatusCode::Ok, "db updated"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("db")),
                            KeyValue::new("rows_affected", AnyValue::new_int(1)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                            KeyValue::new("latency_ms", AnyValue::new_double(4.7)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("db-event")
                                .attributes(vec![
                                    KeyValue::new("query", AnyValue::new_string("UPDATE inventory SET count = count - 1 WHERE sku='ABC123'")),
                                    KeyValue::new("message", AnyValue::new_string("inventory updated")),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-3")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.95)),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ],
            ),
        ],
    ),
]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_traces(expected_data))
            .validate(validation_procedure());
    }

    #[test]
    fn test_filter_processor_traces_exclude_only() {
        let test_runtime = TestRuntime::new();

        let exclude_only = TraceMatchProperties::new(
            MatchType::Regexp,
            // Resource attributes (exclude staging)
            vec![KeyValueFilter::new(
                "deployment.environment".to_string(),
                AnyValueFilter::String(r"^stag.*".to_string()),
            )],
            // Span attributes (exclude db or cache components)
            vec![KeyValueFilter::new(
                "component".to_string(),
                AnyValueFilter::String(r"^(db|cache)$".to_string()),
            )],
            // Span names (exclude spans starting with "cache-" or "db-")
            vec![r"^(cache-.*|db-.*)$".to_string()],
            // Event names (exclude warmup-event and db-event)
            vec![r"^(warmup-event|db-event)$".to_string()],
            // Event attributes (exclude message == "warmup complete" or "inventory updated",
            // and any "query" that begins with "UPDATE inventory")
            vec![
                KeyValueFilter::new(
                    "message".to_string(),
                    AnyValueFilter::String(r"^(warmup complete|inventory updated)$".to_string()),
                ),
                KeyValueFilter::new(
                    "query".to_string(),
                    AnyValueFilter::String(r"^UPDATE inventory .*".to_string()),
                ),
            ],
            // Link attributes (exclude spans with link_tag == "external", which drops checkout-error)
            vec![KeyValueFilter::new(
                "link_tag".to_string(),
                AnyValueFilter::String(r"^external$".to_string()),
            )],
        );

        let trace_filter = TraceFilter::new(None, Some(exclude_only));

        let log_filter = LogFilter::new(None, None, vec![]);

        let config = Config::new(log_filter, trace_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        // Expected TracesData after applying the updated Exclude-only (Regex) filter
        // Kept span: "checkout-warn" only (production resource). All others excluded by the regex rules.
        let expected_data = TracesData::new(vec![ResourceSpans::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("version", AnyValue::new_string("2.0")),
                    KeyValue::new(
                        "service.name",
                        AnyValue::new_string("payments-checkout-service"),
                    ),
                    KeyValue::new("service.version", AnyValue::new_string("1.4.3")),
                    KeyValue::new("service.instance.number", AnyValue::new_int(42)),
                    KeyValue::new("deployment.environment", AnyValue::new_string("production")),
                    KeyValue::new("cloud.region", AnyValue::new_string("us-central1")),
                    KeyValue::new("host.cpu_cores", AnyValue::new_int(8)),
                    KeyValue::new("sampling.rate", AnyValue::new_double(0.25)),
                    KeyValue::new("debug.enabled", AnyValue::new_bool(false)),
                ])
                .finish(),
            vec![ScopeSpans::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    // Only the checkout-warn span remains
                    Span::build()
                        .trace_id(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        .span_id(vec![0, 0, 0, 0, 0, 0, 0, 0])
                        .name("checkout-warn")
                        .trace_state("state-p1")
                        .kind(SpanKind::Server)
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .attributes(vec![
                            KeyValue::new("component", AnyValue::new_string("rest")),
                            KeyValue::new("http.status_code", AnyValue::new_int(200)),
                            KeyValue::new("retryable", AnyValue::new_bool(false)),
                            KeyValue::new("load", AnyValue::new_double(0.73)),
                            KeyValue::new("items_count", AnyValue::new_int(2)),
                        ])
                        .events(vec![
                            Event::build()
                                .name("checkout-event")
                                .attributes(vec![
                                    KeyValue::new(
                                        "message",
                                        AnyValue::new_string("ok checkout #1"),
                                    ),
                                    KeyValue::new("attempt", AnyValue::new_int(1)),
                                    KeyValue::new("success", AnyValue::new_bool(true)),
                                    KeyValue::new("duration_ms", AnyValue::new_double(12.5)),
                                ])
                                .finish(),
                        ])
                        .links(vec![
                            Link::build()
                                .trace_state("link-state-1")
                                .attributes(vec![
                                    KeyValue::new("link_tag", AnyValue::new_string("internal")),
                                    KeyValue::new("correlation", AnyValue::new_bool(true)),
                                    KeyValue::new("retries", AnyValue::new_int(0)),
                                    KeyValue::new("quality", AnyValue::new_double(0.99)),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ],
            )],
        )]);
        test_runtime
            .set_processor(processor)
            .run_test(scenario_traces(expected_data))
            .validate(validation_procedure());
    }
}
