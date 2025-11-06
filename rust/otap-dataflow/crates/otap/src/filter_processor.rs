// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP Filter processor node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//! ToDo: Collect telemetry like number of filtered data is removed datapoints

use self::config::Config;
use crate::{OTAP_PROCESSOR_FACTORIES, pdata::OtapPdata};
use async_trait::async_trait;
use linkme::distributed_slice;

use otap_df_config::error::Error as ConfigError;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::error::{Error, ProcessorErrorKind, format_error_sources};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::otap::OtapArrowRecords;
use serde_json::Value;
use std::sync::Arc;

mod config;
/// The URN for the filter processor
pub const FILTER_PROCESSOR_URN: &str = "urn:otel:filter:processor";

/// processor that outputs all data received to stdout
pub struct FilterProcessor {
    config: Config,
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
    };

impl FilterProcessor {
    /// Creates a new FilterProcessor
    #[must_use]
    #[allow(dead_code)]
    pub fn new(config: Config, _pipeline_ctx: PipelineContext) -> Self {
        FilterProcessor { config }
    }

    /// Creates a new FilterProcessor from a configuration object
    pub fn from_config(
        _pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(config.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: e.to_string(),
            })?;
        Ok(FilterProcessor { config })
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
            Message::Control(_control) => {
                // ToDo: add internal telemetry that will be sent out here
                Ok(())
            }
            Message::PData(pdata) => {
                let signal = pdata.signal_type();
                // convert to arrow records
                let (context, payload) = pdata.into_parts();

                let arrow_records: OtapArrowRecords = payload.try_into()?;

                let filtered_arrow_records: OtapArrowRecords = match signal {
                    SignalType::Metrics => {
                        // ToDo: Add support for metrics
                        arrow_records
                    }
                    SignalType::Logs => {
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
                            })?
                    }
                    SignalType::Traces => {
                        // ToDo: Add support for traces
                        arrow_records
                    }
                };
                effect_handler
                    .send_message(OtapPdata::new(context, filtered_arrow_records.into()))
                    .await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::filter_processor::{FILTER_PROCESSOR_URN, FilterProcessor, config::Config};
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::message::Message;
    use otap_df_engine::processor::ProcessorWrapper;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::processor::{TestContext, ValidateContext};
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::otap::filter::{
        AnyValue as AnyValueFilter, KeyValue as KeyValueFilter, MatchType,
        logs::{LogFilter, LogMatchProperties, LogSeverityNumberMatchProperties},
    };
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, InstrumentationScope, KeyValue},
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber},
        resource::v1::Resource,
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message as _;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    /// Validation closure that checks the outputted data
    fn validation_procedure() -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut _ctx| Box::pin(async move {})
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario_strict() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>>
    {
        move |mut ctx| {
            Box::pin(async move {
                // send log message
                let logs_data = LogsData::new(vec![
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
        vec![
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("library")
                    .version("scopev1")
                    .finish(),
                vec![
                    // WARN: passes include; will be removed by exclude via severity_texts/body
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

                    // ERROR: passes include; not removed by exclude
                    LogRecord::build()
                        .time_unix_nano(2_300_000_000u64)
                        .severity_number(SeverityNumber::Error)
                        .body(AnyValue::new_string("event"))
                .attributes(vec![
                    KeyValue::new("exception.type", AnyValue::new_string("io::Error")),
                    KeyValue::new("exception.message", AnyValue::new_string("connection reset by peer")),
                    KeyValue::new("exception.stacktrace", AnyValue::new_string("...stack...")),
                    KeyValue::new("peer.address", AnyValue::new_string("10.42.0.7:5432")),
                    KeyValue::new("peer.port", AnyValue::new_int(5432)),
                    KeyValue::new("peer.tls", AnyValue::new_bool(true)),
                    KeyValue::new("bytes_sent", AnyValue::new_int(0)),
                            KeyValue::new("jitter", AnyValue::new_double(0.003)),
                        ])
                        .severity_text("ERROR")
                        .body(AnyValue::new_string("failed to write to socket"))
                        .finish(),

                    // WARN: component=db; fails exclude (component=db)
                    LogRecord::build()
                        .time_unix_nano(2_600_000_000u64)
                        .severity_number(SeverityNumber::Warn)
                        .body(AnyValue::new_string("event"))
                .attributes(vec![
                    KeyValue::new("component", AnyValue::new_string("db")),
                    KeyValue::new("query", AnyValue::new_string("UPDATE inventory SET count = count - 1 WHERE sku='ABC123'")),
                    KeyValue::new("rows_affected", AnyValue::new_int(1)),
                            KeyValue::new("success", AnyValue::new_bool(true)),
                        ])
                        .severity_text("WARN")
                        .body(AnyValue::new_string("inventory updated"))
                        .finish(),
                ],
            ),
        ],
    ),

    // ResourceLogs for staging/inventory-service (excluded by both filters)
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
        vec![
            ScopeLogs::new(
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
                    KeyValue::new("http.route", AnyValue::new_string("/api/internal/cache_warm")),
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
            ),
        ],
    ),
]);

                //convert logsdata to otappdata
                let mut bytes = vec![];
                logs_data
                    .encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_logs_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
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
                    OtlpProtoBytes::ExportLogsRequest(bytes) => LogsData::decode(bytes.as_slice())
                        .expect("failed to decode logs into logsdata"),
                    _ => panic!("expected logs type"),
                };

                let expected_logs_data = LogsData::new(vec![ResourceLogs::new(
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
                                    KeyValue::new(
                                        "exception.type",
                                        AnyValue::new_string("io::Error"),
                                    ),
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
                )]);
                assert_eq!(received_logs_data, expected_logs_data);
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
            include_props,
            exclude_props,
            Vec::new(), // log_record ignored
        );

        let config = Config::new(log_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario_strict())
            .validate(validation_procedure());
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario_regex() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>>
    {
        move |mut ctx| {
            Box::pin(async move {
                // send log message
                let logs_data = LogsData::new(vec![ResourceLogs::new(
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
                            KeyValue::new(
                                "deployment.environment",
                                AnyValue::new_string("production"),
                            ),
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
                )]);

                //convert logsdata to otappdata
                let mut bytes = vec![];
                logs_data
                    .encode(&mut bytes)
                    .expect("failed to encode log data into bytes");
                let otlp_logs_bytes =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(bytes).into());
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
                    OtlpProtoBytes::ExportLogsRequest(bytes) => LogsData::decode(bytes.as_slice())
                        .expect("failed to decode logs into logsdata"),
                    _ => panic!("expected logs type"),
                };

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
                            KeyValue::new(
                                "deployment.environment",
                                AnyValue::new_string("production"),
                            ),
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
                assert_eq!(received_logs_data, expected_logs_data);
            })
        }
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
            include_props,
            exclude_props,
            Vec::new(), // log_record ignored
        );

        let config = Config::new(log_filter);
        let user_config = Arc::new(NodeUserConfig::new_processor_config(FILTER_PROCESSOR_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let processor = ProcessorWrapper::local(
            FilterProcessor::new(config, pipeline_ctx),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario_regex())
            .validate(validation_procedure());
    }
}
