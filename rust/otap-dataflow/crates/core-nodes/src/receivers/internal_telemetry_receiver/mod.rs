// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and drains
//! internal metrics from the telemetry registry. The receiver's `signals`
//! configuration independently controls which signals are emitted as OTLP
//! export requests into the observability pipeline. When metrics are disabled,
//! their export accumulator is still drained without OTLP conversion so that
//! retired metric sets can be released.
//!
//! Registry-backed metrics can use a receiver-local export interval and a
//! subset of OpenTelemetry metric views:
//!
//! ```yaml
//! config:
//!   metrics:
//!     interval: 60s
//!     views:
//!       - selector:
//!           scope_name: pipeline
//!           scope_attributes:
//!             pipeline.group.id: pipeline-group-a
//!           instrument_name: uptime
//!         stream:
//!           name: process_uptime
//!           description: Uptime of the pipeline process.
//! ```

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = INTERNAL_TELEMETRY_RECEIVER_URN,
    target = "otel.receiver.internal_telemetry",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::ReceiverFactory;
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::local::receiver as local;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use otel_arrow_dfe_telemetry::metrics::otlp::{MetricView, MetricsOtlpEncoder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

mod logs;
mod metrics;
use logs::LogExportState;
pub use logs::{LogFormatConfig, LogsConfig};
use metrics::MetricExportState;
pub use metrics::{MetricsConfig, ViewConfig, ViewSelector, ViewStream};

/// The URN for the internal telemetry receiver.
pub use otel_arrow_dfe_config::engine::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Signal type emitted by the internal telemetry receiver.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum InternalTelemetrySignal {
    /// Internal log records.
    Logs,
    /// Registry-backed internal metrics.
    Metrics,
}

impl InternalTelemetrySignal {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Logs => "logs",
            Self::Metrics => "metrics",
        }
    }
}

fn default_signals() -> Vec<InternalTelemetrySignal> {
    vec![
        InternalTelemetrySignal::Logs,
        InternalTelemetrySignal::Metrics,
    ]
}

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Non-empty set of signals emitted by this receiver.
    #[serde(default = "default_signals")]
    pub signals: Vec<InternalTelemetrySignal>,

    /// Configuration for registry-backed internal metrics.
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Configuration for receiver-side internal log batching.
    #[serde(default)]
    pub logs: LogsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            signals: default_signals(),
            metrics: MetricsConfig::default(),
            logs: LogsConfig::default(),
        }
    }
}

/// A receiver that emits internal logs and metrics as OTLP data.
pub struct InternalTelemetryReceiver {
    config: Config,
    /// Internal telemetry settings obtained from the pipeline context during construction.
    /// Contains the logs receiver channel, pre-encoded resource bytes, and registry handle.
    internal_telemetry: otel_arrow_dfe_telemetry::InternalTelemetrySettings,
}

/// Declares the internal telemetry receiver as a local receiver factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create:
        |mut pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            // Get internal telemetry settings from the pipeline context
            let internal_telemetry = pipeline.take_internal_telemetry().ok_or_else(|| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: "InternalTelemetryReceiver requires internal telemetry settings in pipeline context".to_owned(),
            }
        })?;

            let config = InternalTelemetryReceiver::parse_config(&node_config.config)?;

            Ok(ReceiverWrapper::local(
                InternalTelemetryReceiver::new_with_telemetry(config, internal_telemetry),
                node,
                node_config,
                receiver_config,
            ))
        },
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: InternalTelemetryReceiver::validate_config,
};

impl InternalTelemetryReceiver {
    /// Create a new receiver with the given configuration and internal telemetry settings.
    #[must_use]
    pub const fn new_with_telemetry(
        config: Config,
        internal_telemetry: otel_arrow_dfe_telemetry::InternalTelemetrySettings,
    ) -> Self {
        Self {
            config,
            internal_telemetry,
        }
    }

    /// Parse configuration from a JSON value.
    pub fn parse_config(config: &Value) -> Result<Config, otel_arrow_dfe_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        config.validate()?;
        Ok(config)
    }

    fn validate_config(config: &Value) -> Result<(), otel_arrow_dfe_config::error::Error> {
        Self::parse_config(config).map(drop)
    }
}

impl Config {
    fn validate(&self) -> Result<(), otel_arrow_dfe_config::error::Error> {
        if self.signals.is_empty() {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: "internal telemetry receiver signals must not be empty".to_owned(),
            });
        }
        let mut unique_signals = std::collections::HashSet::new();
        if let Some(signal) = self
            .signals
            .iter()
            .find(|signal| !unique_signals.insert(**signal))
        {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver signal '{}' is configured more than once",
                    signal.as_str()
                ),
            });
        }
        self.logs.validate()?;
        self.metrics.validate()?;
        Ok(())
    }

    fn logs_enabled(&self) -> bool {
        self.signals.contains(&InternalTelemetrySignal::Logs)
    }

    fn metrics_enabled(&self) -> bool {
        self.signals.contains(&InternalTelemetrySignal::Metrics)
    }

    fn metric_drain_interval(&self, engine_interval: Duration) -> Duration {
        if self.metrics_enabled() {
            self.metrics.interval.unwrap_or(engine_interval)
        } else {
            engine_interval
        }
    }
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for InternalTelemetryReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let internal = self.internal_telemetry.clone();
        let logs_config = self.config.logs;
        let logs_enabled = self.config.logs_enabled();
        let metrics_enabled = self.config.metrics_enabled();
        let metrics_interval = self
            .config
            .metric_drain_interval(internal.default_metric_drain_interval);
        let metrics_encoder = if metrics_enabled {
            let views = self
                .config
                .metrics
                .views
                .into_iter()
                .map(MetricView::from)
                .collect();
            Some(MetricsOtlpEncoder::new_with_views(
                &internal.resource_field_bytes,
                views,
            ))
        } else {
            None
        };
        let mut logs = LogExportState::new(logs_config, logs_enabled, internal.registry.clone());
        let mut metrics =
            MetricExportState::new(metrics_interval, internal.registry.clone(), metrics_encoder);

        loop {
            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    let (deadline, notify_drained) = match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => (deadline, true),
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => (deadline, false),
                        Ok(_) => continue,
                        Err(error) => return Err(Error::ChannelRecvError(error)),
                    };

                    metrics.cancel_pending();
                    logs.complete_pending_until(deadline).await?;
                    logs.flush_until(&effect_handler, &internal, deadline).await?;
                    metrics.flush_until(&effect_handler, deadline).await?;
                    if notify_drained {
                        effect_handler.notify_receiver_drained().await?;
                    }
                    return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                }

                result = metrics.run_once(&effect_handler) => {
                    result?;
                }

                result = logs.run_once(&effect_handler, &internal) => {
                    result?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use logs::{LOG_BATCH_MAX_BYTES, default_log_batch_otlp, estimate_log_bytes};
    use metrics::MetricExporter;
    use otel_arrow_dfe_channel::mpsc;
    use otel_arrow_dfe_config::observed_state::SendPolicy;
    use otel_arrow_dfe_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
    use otel_arrow_dfe_config::pipeline::telemetry::TelemetryConfig;
    use otel_arrow_dfe_config::settings::telemetry::logs::{
        LoggingProviders, LogsConfig as TelemetryLogsConfig, ProviderMode,
    };
    use otel_arrow_dfe_engine::control::{
        NodeControlMsg, RuntimeControlMsg, runtime_ctrl_msg_channel,
    };
    use otel_arrow_dfe_engine::local::message::{LocalReceiver, LocalSender};
    use otel_arrow_dfe_engine::local::receiver::Receiver as _;
    use otel_arrow_dfe_engine::message::{Receiver as EngineReceiver, Sender as EngineSender};
    use otel_arrow_dfe_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otel_arrow_dfe_otap::pdata::Context;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, PayloadData, Sizer};
    use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent};
    use otel_arrow_dfe_telemetry::instrument::Counter;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
    use otel_arrow_dfe_telemetry::{
        __log_record_impl, InternalTelemetrySettings, InternalTelemetrySystem, Level, LogContext,
    };
    use otel_arrow_dfe_telemetry_macros::metric_set;
    use prost::Message as _;
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use std::time::{Duration, Instant as StdInstant, SystemTime};
    use tokio_util::sync::CancellationToken;

    #[metric_set(name = "receiver.internal_telemetry.test")]
    #[derive(Debug, Default)]
    struct TestMetrics {
        /// Number of test events emitted.
        #[metric(unit = "{event}")]
        emitted: Counter<u64>,
    }

    fn test_log_event() -> LogEvent {
        LogEvent {
            time: SystemTime::UNIX_EPOCH,
            record: __log_record_impl!(Level::INFO, "receiver.batch.test")
                .into_record(LogContext::new()),
        }
    }

    fn test_logs_receiver(
        logs: LogsConfig,
        logs_receiver: flume::Receiver<ObservedEvent>,
    ) -> InternalTelemetryReceiver {
        InternalTelemetryReceiver::new_with_telemetry(
            Config {
                signals: vec![InternalTelemetrySignal::Logs],
                metrics: MetricsConfig::default(),
                logs,
            },
            InternalTelemetrySettings {
                logs_receiver,
                resource_field_bytes: Bytes::new(),
                registry: TelemetryRegistryHandle::new(),
                default_metric_drain_interval: Duration::from_secs(60),
                log_tap: None,
            },
        )
    }

    fn start_test_receiver(
        receiver: InternalTelemetryReceiver,
        output_tx: mpsc::Sender<OtapPdata>,
    ) -> (
        mpsc::Sender<NodeControlMsg<OtapPdata>>,
        tokio::task::JoinHandle<Result<TerminalState, Error>>,
    ) {
        let mut outputs = HashMap::new();
        let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
        let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(2);
        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(2);
        let effect_handler = local::EffectHandler::new(
            test_node("internal_telemetry_receiver"),
            outputs,
            None,
            runtime_ctrl_tx,
            metrics_reporter,
        );
        let (ctrl_tx, ctrl_rx) = create_not_send_channel::<NodeControlMsg<OtapPdata>>(2);
        let ctrl_channel =
            local::ControlChannel::new(EngineReceiver::Local(LocalReceiver::mpsc(ctrl_rx)));
        let receiver_task = tokio::task::spawn_local(async move {
            Box::new(receiver).start(ctrl_channel, effect_handler).await
        });
        (ctrl_tx, receiver_task)
    }

    fn decode_logs(pdata: OtapPdata) -> ExportLogsServiceRequest {
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
            pdata.payload().into_data()
        else {
            panic!("internal telemetry receiver emitted a non-logs payload")
        };
        ExportLogsServiceRequest::decode(bytes).expect("valid OTLP logs request")
    }

    fn decode_metric_value(pdata: OtapPdata) -> i64 {
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) =
            pdata.payload().into_data()
        else {
            panic!("internal telemetry receiver emitted a non-metrics payload")
        };
        let request =
            ExportMetricsServiceRequest::decode(bytes).expect("valid OTLP metrics request");
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        let [scope_metrics] = resource_metrics.scope_metrics.as_slice() else {
            panic!("expected one scope metrics message")
        };
        assert_eq!(
            scope_metrics.scope.as_ref().expect("scope").name,
            "receiver.internal_telemetry.test"
        );
        let [metric] = scope_metrics.metrics.as_slice() else {
            panic!("expected one metric")
        };
        assert_eq!(metric.name, "emitted");
        let Some(metric::Data::Sum(sum)) = metric.data.as_ref() else {
            panic!("expected a sum metric")
        };
        let [point] = sum.data_points.as_slice() else {
            panic!("expected one metric data point")
        };
        let Some(number_data_point::Value::AsInt(value)) = point.value else {
            panic!("expected an integer metric data point")
        };
        value
    }

    /// Scenario: metric settings include an interval, views, and scalar selectors.
    /// Guarantees: supported settings parse and omitted signals enable both signals.
    #[test]
    fn parses_supported_metrics_configuration() {
        let config = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "metrics": {
                "interval": "60s",
                "views": [{
                    "selector": {
                        "scope_name": "engine",
                        "scope_attributes": {
                            "service.instance.id": "pipeline-group-a",
                            "worker.id": 3,
                            "worker.ready": true,
                            "worker.load": 0.5
                        },
                        "instrument_name": "memory.rss"
                    },
                    "stream": {
                        "name": "process_memory_usage",
                        "description": "Total physical memory used by the process."
                    }
                }]
            }
        }))
        .expect("supported metrics config should parse");

        assert_eq!(config.signals, default_signals());
        let metrics = config.metrics;
        assert_eq!(metrics.interval, Some(Duration::from_secs(60)));
        assert_eq!(
            metrics.views,
            vec![ViewConfig {
                selector: ViewSelector {
                    scope_name: Some("engine".to_owned()),
                    scope_attributes: HashMap::from([
                        (
                            "service.instance.id".to_owned(),
                            ConfigAttributeValue::String("pipeline-group-a".to_owned()),
                        ),
                        ("worker.id".to_owned(), ConfigAttributeValue::I64(3)),
                        ("worker.ready".to_owned(), ConfigAttributeValue::Bool(true)),
                        ("worker.load".to_owned(), ConfigAttributeValue::F64(0.5)),
                    ]),
                    instrument_name: Some("memory.rss".to_owned()),
                },
                stream: ViewStream {
                    name: Some("process_memory_usage".to_owned()),
                    description: Some("Total physical memory used by the process.".to_owned()),
                },
            }]
        );

        let null_metrics = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "metrics": null
        }))
        .expect_err("the metrics configuration block cannot be null");
        assert!(null_metrics.to_string().contains("invalid type: null"));
    }

    /// Scenario: receiver settings contain invalid fields, selectors, intervals, or signals.
    /// Guarantees: malformed settings are rejected while each non-empty signal subset is valid.
    #[test]
    fn validates_receiver_configuration() {
        let zero_interval = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "metrics": { "interval": "0s" }
        }))
        .expect_err("zero interval must be rejected");
        assert!(
            zero_interval.to_string().contains("greater than zero"),
            "unexpected error: {zero_interval}"
        );

        for config in [
            serde_json::json!({ "unexpected": true }),
            serde_json::json!({ "metrics": { "unexpected": true } }),
            serde_json::json!({
                "metrics": {
                    "views": [{
                        "selector": {},
                        "stream": { "unit": "By" }
                    }]
                }
            }),
        ] {
            let _ = InternalTelemetryReceiver::parse_config(&config)
                .expect_err("unknown fields must be rejected");
        }

        let array_selector = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "metrics": {
                "views": [{
                    "selector": { "scope_attributes": { "worker.tags": ["a", "b"] } },
                    "stream": {}
                }]
            }
        }))
        .expect_err("array scope attribute selectors must be rejected");
        assert!(
            array_selector
                .to_string()
                .contains("must be a scalar value"),
            "unexpected error: {array_selector}"
        );

        let logs_only = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "signals": ["logs"]
        }))
        .expect("logs-only signal selection should be valid");
        assert!(logs_only.logs_enabled());
        assert!(!logs_only.metrics_enabled());

        for (logs, expected) in [
            (
                serde_json::json!({
                    "otlp": { "min_size": 1024, "max_size": 512, "sizer": "bytes" }
                }),
                "must be >= min_size",
            ),
            (
                serde_json::json!({
                    "otlp": {
                        "min_size": 1024,
                        "max_size": LOG_BATCH_MAX_BYTES.get() + 1,
                        "sizer": "bytes"
                    }
                }),
                "size limits must not exceed 2097152 bytes",
            ),
            (
                serde_json::json!({
                    "otlp": { "min_size": 1024, "max_size": null, "sizer": "items" }
                }),
                "OTLP sizer must be bytes",
            ),
            (
                serde_json::json!({
                    "otlp": { "min_size": 1024, "max_size": null, "sizer": "bytes" },
                    "max_batch_duration": "0s"
                }),
                "min_size set requires max_batch_duration is set",
            ),
        ] {
            let error = InternalTelemetryReceiver::parse_config(&serde_json::json!({
                "logs": logs
            }))
            .expect_err("invalid log batching settings must be rejected");
            assert!(
                error.to_string().contains(expected),
                "unexpected error: {error}"
            );
        }
        let _ = InternalTelemetryReceiver::parse_config(&serde_json::json!({
            "logs": {
                "otlp": { "min_size": null, "max_size": 2097152, "sizer": "bytes" },
                "max_batch_duration": "0s"
            }
        }))
        .expect("inactive OTAP defaults do not constrain OTLP immediate flushing");

        for (signals, expected) in [
            (serde_json::json!([]), "must not be empty"),
            (
                serde_json::json!(["metrics", "metrics"]),
                "configured more than once",
            ),
        ] {
            let error = InternalTelemetryReceiver::parse_config(&serde_json::json!({
                "signals": signals
            }))
            .expect_err("invalid signal selection must be rejected");
            assert!(
                error.to_string().contains(expected),
                "unexpected error: {error}"
            );
        }
    }

    /// Scenario: two sub-threshold logs wait in a partial receiver batch.
    /// Guarantees: the latency deadline emits one request containing both records.
    #[test]
    fn timer_flushes_partial_log_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let (logs_sender, logs_receiver) = flume::bounded(4);
            let receiver = test_logs_receiver(
                LogsConfig {
                    otlp: LogFormatConfig {
                        min_size: Some(LOG_BATCH_MAX_BYTES),
                        ..default_log_batch_otlp()
                    },
                    max_batch_duration: Some(Duration::from_millis(20)),
                    ..LogsConfig::default()
                },
                logs_receiver,
            );

            let (output_tx, output_rx) = create_not_send_channel(2);
            let (ctrl_tx, receiver_task) = start_test_receiver(receiver, output_tx);

            logs_sender
                .send(ObservedEvent::Log(test_log_event()))
                .expect("first log should enqueue");
            logs_sender
                .send(ObservedEvent::Log(test_log_event()))
                .expect("second log should enqueue");

            let output = tokio::time::timeout(Duration::from_millis(150), output_rx.recv())
                .await
                .expect("timer should flush before timeout")
                .expect("output channel should remain open");
            let request = decode_logs(output);
            assert_eq!(request.resource_logs[0].scope_logs.len(), 1);
            assert_eq!(request.resource_logs[0].scope_logs[0].log_records.len(), 2);

            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test complete".to_owned(),
                })
                .expect("shutdown should enqueue");
            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should shut down cleanly");
        }));
    }

    /// Scenario: the internal log channel closes with one sub-threshold log buffered.
    /// Guarantees: channel closure flushes the partial batch without waiting for its timer.
    #[test]
    fn channel_closure_flushes_partial_log_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let (logs_sender, logs_receiver) = flume::bounded(1);
            let receiver = test_logs_receiver(
                LogsConfig {
                    otlp: LogFormatConfig {
                        min_size: Some(LOG_BATCH_MAX_BYTES),
                        ..default_log_batch_otlp()
                    },
                    max_batch_duration: Some(Duration::from_secs(60)),
                    ..LogsConfig::default()
                },
                logs_receiver,
            );
            let (output_tx, output_rx) = create_not_send_channel(1);
            let (ctrl_tx, receiver_task) = start_test_receiver(receiver, output_tx);

            logs_sender
                .send(ObservedEvent::Log(test_log_event()))
                .expect("log should enqueue");
            drop(logs_sender);

            let output = tokio::time::timeout(Duration::from_millis(150), output_rx.recv())
                .await
                .expect("channel closure should flush before the batch timer")
                .expect("output channel should remain open");
            let request = decode_logs(output);
            assert_eq!(request.resource_logs[0].scope_logs[0].log_records.len(), 1);

            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test complete".to_owned(),
                })
                .expect("shutdown should enqueue");
            assert!(
                receiver_task
                    .await
                    .expect("receiver task should join")
                    .is_ok(),
                "receiver should shut down cleanly"
            );
        }));
    }

    /// Scenario: shutdown begins as a blocked split export becomes deliverable.
    /// Guarantees: the in-flight and waiting batches are each delivered exactly once in order.
    #[test]
    fn max_size_splits_partial_log_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let event = test_log_event();
            let second_event = LogEvent {
                time: SystemTime::UNIX_EPOCH,
                record: __log_record_impl!(Level::INFO, "receiver.batch.second")
                    .into_record(LogContext::new()),
            };
            let split_size = estimate_log_bytes(&event)
                .saturating_mul(2)
                .saturating_sub(1);
            let split_size = NonZeroUsize::new(split_size).expect("test event has estimated bytes");
            let (logs_sender, logs_receiver) = flume::bounded(2);
            let receiver = test_logs_receiver(
                LogsConfig {
                    otlp: LogFormatConfig {
                        min_size: Some(split_size),
                        max_size: Some(split_size),
                        sizer: Sizer::Bytes,
                    },
                    max_batch_duration: Some(Duration::from_secs(1)),
                    ..LogsConfig::default()
                },
                logs_receiver,
            );

            let (output_tx, output_rx) = create_not_send_channel(1);
            output_tx
                .send(OtapPdata::new(
                    Context::default(),
                    OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                ))
                .expect("downstream blocker should enqueue");
            let (ctrl_tx, receiver_task) = start_test_receiver(receiver, output_tx);

            logs_sender
                .send(ObservedEvent::Log(event))
                .expect("first log should enqueue");
            logs_sender
                .send(ObservedEvent::Log(second_event))
                .expect("second log should enqueue");
            tokio::time::sleep(Duration::from_millis(20)).await;
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "flush split batches".to_owned(),
                })
                .expect("shutdown should enqueue");
            tokio::time::sleep(Duration::from_millis(20)).await;
            let _blocker = output_rx
                .recv()
                .await
                .expect("downstream blocker should remain queued");
            for expected_name in ["receiver.batch.test", "receiver.batch.second"] {
                let output = tokio::time::timeout(Duration::from_millis(200), output_rx.recv())
                    .await
                    .expect("terminal flush should not combine split batches")
                    .expect("output channel should remain open");
                let request = decode_logs(output);
                assert_eq!(
                    request.resource_logs[0].scope_logs[0].log_records.len(),
                    1,
                    "terminal request should contain one record"
                );
                assert_eq!(
                    request.resource_logs[0].scope_logs[0].log_records[0].event_name,
                    expected_name
                );
            }
            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should shut down cleanly");
            assert!(
                output_rx.recv().await.is_err(),
                "shutdown must not retry the completed in-flight batch"
            );
        }));
    }

    /// Scenario: a threshold log export is blocked when shutdown begins.
    /// Guarantees: shutdown waits only until its deadline and never retries ambiguous ownership.
    #[test]
    fn shutdown_interrupts_blocked_log_batch_export() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let (logs_sender, logs_receiver) = flume::bounded(1);
            let receiver = test_logs_receiver(
                LogsConfig {
                    otlp: LogFormatConfig {
                        min_size: Some(NonZeroUsize::MIN),
                        ..default_log_batch_otlp()
                    },
                    ..LogsConfig::default()
                },
                logs_receiver,
            );

            let (output_tx, output_rx) = create_not_send_channel(1);
            output_tx
                .send(OtapPdata::new(
                    Context::default(),
                    OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                ))
                .expect("downstream blocker should enqueue");
            let (ctrl_tx, receiver_task) = start_test_receiver(receiver, output_tx);

            logs_sender
                .send(ObservedEvent::Log(test_log_event()))
                .expect("log should enqueue");
            tokio::time::sleep(Duration::from_millis(20)).await;
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_millis(20),
                    reason: "test deadline".to_owned(),
                })
                .expect("shutdown should enqueue");

            let result = tokio::time::timeout(Duration::from_millis(500), receiver_task)
                .await
                .expect("shutdown must interrupt the blocked log export")
                .expect("receiver task should join");
            let error = match result {
                Ok(_) => panic!("blocked terminal log flush should fail"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("timed out while completing an in-flight internal log export"),
                "unexpected error: {error}"
            );
            drop(output_rx);
        }));
    }

    /// Scenario: metric emission has and does not have a receiver-local interval override.
    /// Guarantees: the receiver uses the override when present and the engine interval otherwise.
    #[test]
    fn resolves_receiver_metrics_interval_override() {
        let defaults = Config::default();
        assert_eq!(
            defaults.metric_drain_interval(Duration::from_secs(60)),
            Duration::from_secs(60)
        );

        let configured = Config {
            signals: default_signals(),
            metrics: MetricsConfig {
                interval: Some(Duration::from_secs(5)),
                views: Vec::new(),
            },
            logs: LogsConfig::default(),
        };
        assert_eq!(
            configured.metric_drain_interval(Duration::from_secs(60)),
            Duration::from_secs(5)
        );

        let logs_only = Config {
            signals: vec![InternalTelemetrySignal::Logs],
            metrics: MetricsConfig {
                interval: Some(Duration::from_secs(5)),
                views: Vec::new(),
            },
            logs: LogsConfig::default(),
        };
        assert_eq!(
            logs_only.metric_drain_interval(Duration::from_secs(60)),
            Duration::from_secs(60),
            "a disabled signal must not let its emission settings control cleanup"
        );
    }

    /// Scenario: downstream delivery fails after a metric export transaction begins.
    /// Guarantees: the drained metric values are restored for a later delivery attempt.
    #[test]
    fn failed_downstream_send_restores_drained_metric_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set: otel_arrow_dfe_telemetry::metrics::MetricSet<TestMetrics> =
                registry.register_metric_set(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                0,
                &[otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)],
            );
            let encoder = MetricsOtlpEncoder::new(&ResourceLogs::default().encode_to_vec());

            let (output_tx, output_rx) = create_not_send_channel(1);
            drop(output_rx);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let _error = MetricExporter::process_batch(&effect_handler, &registry, Some(&encoder))
                .await
                .expect_err("closed downstream must fail delivery");

            let retry = registry.drain_metric_export_batch();
            assert_eq!(retry.metric_sets.len(), 1);
            assert_eq!(
                retry.metric_sets[0].values,
                vec![otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)]
            );
        }));
    }

    /// Scenario: the receiver consumes registry metrics while metric emission is disabled.
    /// Guarantees: the export accumulator is committed without downstream output or admin loss.
    #[test]
    fn disabled_metrics_are_drained_without_emission() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set: otel_arrow_dfe_telemetry::metrics::MetricSet<TestMetrics> =
                registry.register_metric_set(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                0,
                &[otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)],
            );

            let (output_tx, output_rx) = create_not_send_channel(1);
            drop(output_rx);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            MetricExporter::process_batch(&effect_handler, &registry, None)
                .await
                .expect("disabled metrics must not access the closed downstream");
            assert!(
                registry.drain_metric_export_batch().is_empty(),
                "disabled metric output must still commit the export accumulator"
            );

            let mut admin_values = Vec::new();
            registry.visit_admin_metrics_and_reset(|_, _, metrics| {
                admin_values.extend(metrics.map(|(_, value)| value.clone()));
            });
            assert_eq!(
                admin_values,
                vec![otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)],
                "discarding an ITS export must not consume the admin accumulator"
            );
        }));
    }

    /// Scenario: a periodic metric export is blocked when shutdown begins.
    /// Guarantees: shutdown cancels the blocked attempt and bounds the terminal retry.
    #[test]
    fn shutdown_interrupts_periodic_metric_export_blocked_by_downstream_backpressure() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set: otel_arrow_dfe_telemetry::metrics::MetricSet<TestMetrics> =
                registry.register_metric_set(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                0,
                &[otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)],
            );

            let (logs_sender, logs_receiver) = flume::bounded(1);
            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {
                    signals: vec![InternalTelemetrySignal::Metrics],
                    metrics: MetricsConfig {
                        interval: Some(Duration::from_millis(10)),
                        views: Vec::new(),
                    },
                    logs: LogsConfig::default(),
                },
                InternalTelemetrySettings {
                    logs_receiver,
                    resource_field_bytes: ResourceLogs::default().encode_to_vec().into(),
                    registry: registry.clone(),
                    default_metric_drain_interval: Duration::from_millis(10),
                    log_tap: None,
                },
            );

            let (output_tx, output_rx) = create_not_send_channel(1);
            output_tx
                .send(OtapPdata::new(
                    Context::default(),
                    OtlpProtoBytes::ExportMetricsRequest(Bytes::new()).into(),
                ))
                .expect("downstream blocker should enqueue");
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let (ctrl_tx, ctrl_rx) = create_not_send_channel::<NodeControlMsg<OtapPdata>>(1);
            let ctrl_channel =
                local::ControlChannel::new(EngineReceiver::Local(LocalReceiver::mpsc(ctrl_rx)));
            let receiver_task = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_channel, effect_handler).await
            });

            tokio::time::sleep(Duration::from_millis(30)).await;
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_millis(100),
                    reason: "test shutdown".to_owned(),
                })
                .expect("shutdown control should enqueue");

            let result = tokio::time::timeout(Duration::from_millis(500), receiver_task)
                .await
                .expect("shutdown must interrupt the blocked periodic export")
                .expect("receiver task should join");
            assert!(result.is_err(), "the bounded final export should time out");

            let retry = registry.drain_metric_export_batch();
            assert_eq!(retry.metric_sets.len(), 1);
            assert_eq!(
                retry.metric_sets[0].values,
                vec![otel_arrow_dfe_telemetry::metrics::MetricValue::U64(9)]
            );

            drop(output_rx);
            drop(logs_sender);
        }));
    }

    /// Scenario: a queued terminal log cannot enter a full downstream channel during shutdown.
    /// Guarantees: terminal log draining stops at the shutdown deadline instead of blocking.
    #[test]
    fn shutdown_bounds_terminal_log_drain_blocked_by_downstream_backpressure() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let (logs_sender, logs_receiver) = flume::bounded(1);
            let record = otel_arrow_dfe_telemetry::__log_record_impl!(
                Level::INFO,
                "internal_telemetry.test.terminal_log",
                message = "queued terminal log"
            )
            .into_record(LogContext::new());
            logs_sender
                .send(ObservedEvent::Log(LogEvent {
                    time: SystemTime::now(),
                    record,
                }))
                .expect("terminal log should enqueue");
            let internal = InternalTelemetrySettings {
                logs_receiver,
                resource_field_bytes: ResourceLogs::default().encode_to_vec().into(),
                registry: registry.clone(),
                default_metric_drain_interval: Duration::from_secs(60),
                log_tap: None,
            };
            let mut logs = LogExportState::new(LogsConfig::default(), true, registry);

            let (output_tx, _output_rx) = create_not_send_channel(1);
            output_tx
                .send(OtapPdata::new(
                    Context::default(),
                    OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                ))
                .expect("downstream blocker should enqueue");
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let deadline = StdInstant::now() + Duration::from_millis(50);
            let result = tokio::time::timeout(
                Duration::from_secs(1),
                logs.flush_until(&effect_handler, &internal, deadline),
            )
            .await
            .expect("terminal log drain must not outlive the shutdown deadline");
            let error = result.expect_err("the bounded terminal log drain should time out");
            assert!(
                error
                    .to_string()
                    .contains("remaining terminal telemetry was not flushed"),
                "unexpected timeout error: {error}"
            );
        }));
    }

    /// Scenario: terminal log draining starts after the deadline with an empty log queue.
    /// Guarantees: an empty queue does not produce a false internal-log timeout error.
    #[test]
    fn expired_deadline_with_empty_log_queue_does_not_report_log_timeout() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let (_logs_sender, logs_receiver) = flume::bounded(1);
            let internal = InternalTelemetrySettings {
                logs_receiver,
                resource_field_bytes: ResourceLogs::default().encode_to_vec().into(),
                registry: registry.clone(),
                default_metric_drain_interval: Duration::from_secs(60),
                log_tap: None,
            };
            let mut logs = LogExportState::new(LogsConfig::default(), true, registry);

            let (output_tx, _output_rx) = create_not_send_channel(1);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let result = logs
                .flush_until(&effect_handler, &internal, StdInstant::now())
                .await;
            if let Err(error) = result {
                assert!(
                    !error.to_string().contains("flushing internal logs"),
                    "empty log queue must not report a log timeout: {error}"
                );
            }
        }));
    }

    /// Scenario: real metric sets cross collection intervals and receiver ingress drain.
    /// Guarantees: terminal metrics flow once and the receiver reports that ingress is drained.
    #[test]
    fn metric_set_flows_through_collector_across_intervals_and_drain_ingress() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let engine_reporting_interval = Duration::from_secs(60);
            let receiver_interval = Duration::from_millis(25);
            let registry = TelemetryRegistryHandle::new();
            let config = TelemetryConfig {
                reporting_interval: engine_reporting_interval,
                logs: TelemetryLogsConfig {
                    providers: LoggingProviders {
                        global: ProviderMode::Noop,
                        engine: ProviderMode::Noop,
                        internal: ProviderMode::Noop,
                        admin: ProviderMode::Noop,
                    },
                    ..TelemetryLogsConfig::default()
                },
                ..TelemetryConfig::default()
            };
            let telemetry = InternalTelemetrySystem::new(
                &config,
                engine_reporting_interval,
                registry.clone(),
                None,
                SendPolicy::default(),
                LogContext::new,
                None,
            )
            .expect("ITS telemetry system should start");
            let mut metric_set = registry.register_metric_set::<TestMetrics>(EmptyAttributes());
            let mut reporter = telemetry.reporter();
            let collector = telemetry.collector();

            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {
                    signals: vec![InternalTelemetrySignal::Metrics],
                    metrics: MetricsConfig {
                        interval: Some(receiver_interval),
                        views: Vec::new(),
                    },
                    logs: LogsConfig::default(),
                },
                telemetry.internal_telemetry_settings(),
            );

            let (output_tx, output_rx) = create_not_send_channel(4);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, mut runtime_ctrl_rx) = runtime_ctrl_msg_channel(4);
            let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(4);
            let effect_handler = local::EffectHandler::new(
                test_node("internal_telemetry_receiver"),
                outputs,
                None,
                runtime_ctrl_tx,
                metrics_reporter,
            );

            let (ctrl_tx, ctrl_rx) = create_not_send_channel::<NodeControlMsg<OtapPdata>>(4);
            let ctrl_channel =
                local::ControlChannel::new(EngineReceiver::Local(LocalReceiver::mpsc(ctrl_rx)));
            let receiver_task = tokio::task::spawn_local(async move {
                Box::new(receiver).start(ctrl_channel, effect_handler).await
            });

            let collector_cancel = CancellationToken::new();
            let collector_task =
                tokio::task::spawn_local(collector.clone().run(collector_cancel.clone()));

            // Let both tasks initialize before advancing the receiver's interval.
            tokio::task::yield_now().await;

            // First collection window: mutate a real metric set and flush it through
            // the production reporter channel and collector.
            metric_set.emitted.add(3);
            reporter
                .report(&mut metric_set)
                .expect("first metric snapshot should be queued");
            assert_eq!(metric_set.emitted.get(), 0, "reporting clears hot values");
            let first_output = tokio::time::timeout(Duration::from_secs(2), output_rx.recv())
                .await
                .expect("timed out waiting for first periodic metrics")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(first_output), 3);
            assert!(
                output_rx.try_recv().is_err(),
                "the first snapshot must be emitted only once"
            );

            // Second collection window must contain only its own delta.
            metric_set.emitted.add(4);
            reporter
                .report(&mut metric_set)
                .expect("second metric snapshot should be queued");
            let second_output = tokio::time::timeout(Duration::from_secs(2), output_rx.recv())
                .await
                .expect("timed out waiting for second periodic metrics")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(second_output), 4);
            assert!(
                output_rx.try_recv().is_err(),
                "the second snapshot must be emitted only once"
            );

            // An interval without a reported snapshot emits no empty request.
            assert!(
                tokio::time::timeout(receiver_interval * 3, output_rx.recv())
                    .await
                    .is_err(),
                "an empty registry interval must not emit pdata"
            );

            // Queue the final snapshot immediately before ingress drain. The
            // receiver's FIFO barrier guarantees that it reaches the registry
            // before the final drain, regardless of collector scheduling.
            metric_set.emitted.add(5);
            reporter
                .report(&mut metric_set)
                .expect("final metric snapshot should be queued");
            ctrl_tx
                .send(NodeControlMsg::DrainIngress {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test ingress drain".to_owned(),
                })
                .expect("ingress-drain control should be sent");

            let final_output = tokio::time::timeout(Duration::from_secs(1), output_rx.recv())
                .await
                .expect("timed out waiting for final metrics drain")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(final_output), 5);

            let drained = tokio::time::timeout(Duration::from_secs(1), runtime_ctrl_rx.recv())
                .await
                .expect("timed out waiting for receiver-drained notification")
                .expect("runtime control channel should remain open");
            assert!(
                matches!(drained, RuntimeControlMsg::ReceiverDrained { .. }),
                "unexpected runtime control message: {drained:?}"
            );

            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should stop cleanly");
            assert!(
                output_rx.try_recv().is_err(),
                "the terminal snapshot must be emitted exactly once"
            );

            collector_cancel.cancel();
            collector_task
                .await
                .expect("collector task should join")
                .expect("collector should stop cleanly");
            drop(telemetry);
        }));
    }
}
