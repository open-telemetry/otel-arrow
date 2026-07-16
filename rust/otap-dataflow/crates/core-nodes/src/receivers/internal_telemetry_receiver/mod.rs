// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal telemetry receiver.
//!
//! This receiver consumes internal logs from the logging channel and drains
//! internal metrics from the telemetry registry. It emits both signals as
//! OTLP export requests into the observability pipeline.
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

use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::local::receiver as local;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_telemetry::event::{LogEvent, ObservedEvent};
use otap_df_telemetry::metrics::MetricSetSnapshot;
use otap_df_telemetry::metrics::otlp::{
    MetricView, MetricViewSelector, MetricViewStream, MetricsOtlpEncoder,
};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

/// The URN for the internal telemetry receiver.
pub use otap_df_telemetry::INTERNAL_TELEMETRY_RECEIVER_URN;

/// Configuration for the internal telemetry receiver.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration for registry-backed internal metrics.
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// Registry-backed internal metrics configuration.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    /// How frequently accumulated registry metrics are emitted.
    ///
    /// When omitted, the engine telemetry reporting interval is used.
    #[serde(default, with = "humantime_serde::option")]
    pub interval: Option<Duration>,

    /// Views applied while projecting metric-set fields to OTLP metrics.
    #[serde(default)]
    pub views: Vec<ViewConfig>,
}

/// A supported metric view transformation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ViewConfig {
    /// Selects metric-set fields to transform.
    pub selector: ViewSelector,

    /// Overrides properties of each selected OTLP metric stream.
    pub stream: ViewStream,
}

/// Exact-match selector for a metric view.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ViewSelector {
    /// Metric-set (instrumentation scope) name to match.
    pub scope_name: Option<String>,

    /// Scalar metric-set entity attributes that must all match exactly.
    #[serde(default)]
    pub scope_attributes: HashMap<String, ConfigAttributeValue>,

    /// Metric field (instrument) name to match.
    pub instrument_name: Option<String>,
}

/// Supported output stream overrides for a metric view.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ViewStream {
    /// Replacement metric name.
    pub name: Option<String>,

    /// Replacement metric description.
    pub description: Option<String>,
}

impl MetricsConfig {
    const fn is_empty(&self) -> bool {
        self.interval.is_none() && self.views.is_empty()
    }
}

impl From<ViewConfig> for MetricView {
    fn from(view: ViewConfig) -> Self {
        Self {
            selector: MetricViewSelector {
                scope_name: view.selector.scope_name,
                scope_attributes: view.selector.scope_attributes,
                instrument_name: view.selector.instrument_name,
            },
            stream: MetricViewStream {
                name: view.stream.name,
                description: view.stream.description,
            },
        }
    }
}

/// A receiver that emits internal logs and metrics as OTLP data.
pub struct InternalTelemetryReceiver {
    config: Config,
    /// Internal telemetry settings obtained from the pipeline context during construction.
    /// Contains the logs receiver channel, pre-encoded resource bytes, and registry handle.
    internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
}

/// Declares the internal telemetry receiver as a local receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static INTERNAL_TELEMETRY_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: INTERNAL_TELEMETRY_RECEIVER_URN,
    create: |mut pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        // Get internal telemetry settings from the pipeline context
        let internal_telemetry = pipeline.take_internal_telemetry().ok_or_else(|| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: "InternalTelemetryReceiver requires internal telemetry settings in pipeline context".to_owned(),
            }
        })?;

        let config = InternalTelemetryReceiver::parse_config(&node_config.config)?;
        config.validate_metrics_enabled(internal_telemetry.metrics_interval.is_some())?;

        Ok(ReceiverWrapper::local(
            InternalTelemetryReceiver::new_with_telemetry(config, internal_telemetry),
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: InternalTelemetryReceiver::validate_config,
};

impl InternalTelemetryReceiver {
    /// Create a new receiver with the given configuration and internal telemetry settings.
    #[must_use]
    pub const fn new_with_telemetry(
        config: Config,
        internal_telemetry: otap_df_telemetry::InternalTelemetrySettings,
    ) -> Self {
        Self {
            config,
            internal_telemetry,
        }
    }

    /// Parse configuration from a JSON value.
    pub fn parse_config(config: &Value) -> Result<Config, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        config.validate()?;
        Ok(config)
    }

    fn validate_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
        Self::parse_config(config).map(drop)
    }
}

impl Config {
    fn validate(&self) -> Result<(), otap_df_config::error::Error> {
        if self
            .metrics
            .interval
            .is_some_and(|interval| interval.is_zero())
        {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "internal telemetry receiver metrics interval must be greater than zero"
                    .to_owned(),
            });
        }
        if let Some((key, _)) = self.metrics.views.iter().find_map(|view| {
            view.selector
                .scope_attributes
                .iter()
                .find(|(_, value)| matches!(value, ConfigAttributeValue::Array(_)))
        }) {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver metric view scope attribute '{key}' must be a scalar value"
                ),
            });
        }
        Ok(())
    }

    fn validate_metrics_enabled(
        &self,
        metrics_enabled: bool,
    ) -> Result<(), otap_df_config::error::Error> {
        if !metrics_enabled && !self.metrics.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "internal telemetry receiver metrics configuration requires engine internal metrics to use the ITS provider".to_owned(),
            });
        }
        Ok(())
    }

    fn metrics_interval(&self, engine_interval: Option<Duration>) -> Option<Duration> {
        engine_interval.map(|interval| self.metrics.interval.unwrap_or(interval))
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
        let logs_receiver = internal.logs_receiver;
        let resource_bytes = internal.resource_bytes;
        let log_tap = internal.log_tap;
        let registry = internal.registry;
        let mut scope_cache = ScopeToBytesMap::new(registry.clone());
        let metrics_interval = self.config.metrics_interval(internal.metrics_interval);
        let views = self
            .config
            .metrics
            .views
            .into_iter()
            .map(MetricView::from)
            .collect();
        let metrics_encoder = metrics_interval
            .map(|_| MetricsOtlpEncoder::new_with_views(&resource_bytes, views))
            .transpose()
            .map_err(|error| Error::PdataConversionError {
                error: error.to_string(),
            })?;
        let mut metrics_interval = metrics_interval.map(|period| {
            let mut interval = interval_at(Instant::now() + period, period);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
            interval
        });
        let mut logs_channel_open = true;
        let mut pending_metric_export = None;

        loop {
            tokio::select! {
                biased;

                // Handle control messages with priority
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            // Cancel an interval export that may be waiting on a
                            // full downstream channel. Dropping its transaction
                            // restores the drained values before the bounded
                            // terminal attempt below.
                            drop(pending_metric_export.take());
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    if let Some(log_tap) = log_tap.as_ref() {
                                        log_tap.record(log_event.clone());
                                    }
                                    Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                                }
                            }
                            Self::send_metric_batch_until(
                                &effect_handler,
                                &registry,
                                metrics_encoder.as_ref(),
                                deadline,
                            ).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            drop(pending_metric_export.take());
                            // Drain any remaining logs from channel before shutdown
                            while let Ok(event) = logs_receiver.try_recv() {
                                if let ObservedEvent::Log(log_event) = event {
                                    if let Some(log_tap) = log_tap.as_ref() {
                                        log_tap.record(log_event.clone());
                                    }
                                    Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                                }
                            }
                            Self::send_metric_batch_until(
                                &effect_handler,
                                &registry,
                                metrics_encoder.as_ref(),
                                deadline,
                            ).await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                            // No metrics to report for now
                        }
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                             // Ignore other control messages
                        }
                    }
                }

                result = async {
                    pending_metric_export
                        .as_mut()
                        .expect("metric export branch requires an in-flight export")
                        .await
                }, if pending_metric_export.is_some() => {
                    pending_metric_export = None;
                    result?;
                }

                // Drain and emit registry metrics at the configured cold-path interval.
                _ = Self::next_metrics_tick(&mut metrics_interval), if pending_metric_export.is_none() => {
                    pending_metric_export = Some(Box::pin(Self::send_metric_batch(
                        &effect_handler,
                        &registry,
                        metrics_encoder.as_ref(),
                    )));
                }

                // Receive logs from the channel
                result = logs_receiver.recv_async(), if logs_channel_open => {
                    match result {
                        Ok(ObservedEvent::Log(log_event)) => {
                            if let Some(log_tap) = log_tap.as_ref() {
                                log_tap.record(log_event.clone());
                            }
                            Self::send_log_event(&effect_handler, log_event, &resource_bytes, &mut scope_cache).await?;
                        }
                        Ok(ObservedEvent::Engine(_)) => {
                            // Engine events are not yet processed
                        }
                        Err(_) => {
                            logs_channel_open = false;
                            if metrics_encoder.is_none() {
                                return Ok(TerminalState::default());
                            }
                        }
                    }
                }
            }
        }
    }
}

impl InternalTelemetryReceiver {
    async fn next_metrics_tick(interval: &mut Option<Interval>) {
        match interval {
            Some(interval) => {
                let _ = interval.tick().await;
            }
            None => std::future::pending().await,
        }
    }

    /// Attempts one final metric export within the pipeline shutdown deadline.
    ///
    /// Timing out cancels [`Self::send_metric_batch`]; its uncommitted export
    /// transaction is then dropped and restores the drained registry values.
    async fn send_metric_batch_until(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        tokio::time::timeout_at(
            Instant::from_std(deadline),
            Self::send_metric_batch(effect_handler, registry, encoder),
        )
        .await
        .map_err(|_| Error::InternalError {
            message: "timed out while flushing internal metrics during shutdown".to_owned(),
        })?
    }

    /// Flushes pending snapshots and transactionally sends one direct OTLP request.
    ///
    /// The registry lock is released before encoding and downstream delivery.
    /// The batch is committed only after the downstream channel accepts it;
    /// encoding errors, send errors, and cancellation roll it back on drop.
    async fn send_metric_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
    ) -> Result<(), Error> {
        let Some(encoder) = encoder else {
            return Ok(());
        };

        registry
            .flush_pending_metrics()
            .await
            .map_err(|error| Error::InternalError {
                message: format!("failed to flush internal metrics collector: {error}"),
            })?;
        let export = registry.begin_metric_export_batch();
        let Some(metrics) =
            encoder
                .encode(export.batch())
                .map_err(|error| Error::PdataConversionError {
                    error: error.to_string(),
                })?
        else {
            let _ = export.commit();
            return Ok(());
        };

        effect_handler
            .send_message(OtapPdata::new(Context::default(), metrics.into()))
            .await?;
        let _ = export.commit();
        Ok(())
    }

    /// Send a log event as OTLP logs with scope attributes from entity context.
    async fn send_log_event(
        effect_handler: &local::EffectHandler<OtapPdata>,
        log_event: LogEvent,
        resource_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
    ) -> Result<(), Error> {
        let mut buf = ProtoBuffer::with_capacity(512);

        encode_export_logs_request(&mut buf, &log_event, resource_bytes, scope_cache);

        let pdata = OtapPdata::new(
            Context::default(),
            OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into(),
        );
        effect_handler.send_message(pdata).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::observed_state::SendPolicy;
    use otap_df_config::pipeline::telemetry::{
        TelemetryConfig,
        metrics::{MetricsConfig as SdkMetricsConfig, MetricsProvider},
    };
    use otap_df_config::settings::telemetry::logs::{LoggingProviders, LogsConfig, ProviderMode};
    use otap_df_engine::control::{NodeControlMsg, runtime_ctrl_msg_channel};
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::receiver::Receiver as _;
    use otap_df_engine::message::{Receiver as EngineReceiver, Sender as EngineSender};
    use otap_df_engine::testing::{create_not_send_channel, setup_test_runtime, test_node};
    use otap_df_pdata::OtapPayload;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otap_df_telemetry::instrument::Counter;
    use otap_df_telemetry::reporter::MetricsReporter;
    use otap_df_telemetry::testing::EmptyAttributes;
    use otap_df_telemetry::{InternalTelemetrySystem, LogContext};
    use otap_df_telemetry_macros::metric_set;
    use prost::Message as _;
    use std::collections::HashMap;
    use std::time::{Duration, Instant as StdInstant};
    use tokio_util::sync::CancellationToken;

    #[metric_set(name = "receiver.internal_telemetry.test")]
    #[derive(Debug, Default)]
    struct TestMetrics {
        /// Number of test events emitted.
        #[metric(unit = "{event}")]
        emitted: Counter<u64>,
    }

    fn decode_metric_value(pdata: OtapPdata) -> i64 {
        let OtapPayload::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) = pdata.payload()
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

        assert_eq!(config.metrics.interval, Some(Duration::from_secs(60)));
        assert_eq!(
            config.metrics.views,
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
    }

    #[test]
    fn validates_metrics_configuration() {
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
    }

    #[test]
    fn resolves_receiver_interval_only_when_its_metrics_are_enabled() {
        let inherited = Config::default();
        assert_eq!(
            inherited.metrics_interval(Some(Duration::from_secs(60))),
            Some(Duration::from_secs(60))
        );

        let configured = Config {
            metrics: MetricsConfig {
                interval: Some(Duration::from_secs(5)),
                views: Vec::new(),
            },
        };
        assert_eq!(
            configured.metrics_interval(Some(Duration::from_secs(60))),
            Some(Duration::from_secs(5))
        );
        assert_eq!(configured.metrics_interval(None), None);
        configured
            .validate_metrics_enabled(true)
            .expect("configured metrics are valid when ITS metrics are enabled");
        assert!(
            configured.validate_metrics_enabled(false).is_err(),
            "configured metrics must be rejected when ITS metrics are disabled"
        );
        Config::default()
            .validate_metrics_enabled(false)
            .expect("an empty metrics config is valid when metrics are disabled");
    }

    #[test]
    fn failed_downstream_send_restores_drained_metric_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set: otap_df_telemetry::metrics::MetricSet<TestMetrics> =
                registry.register_metric_set(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                &[otap_df_telemetry::metrics::MetricValue::U64(9)],
            );
            let encoder = MetricsOtlpEncoder::new(&ResourceLogs::default().encode_to_vec())
                .expect("valid empty OTLP resource");

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

            let _error = InternalTelemetryReceiver::send_metric_batch(
                &effect_handler,
                &registry,
                Some(&encoder),
            )
            .await
            .expect_err("closed downstream must fail delivery");

            let retry = registry.drain_metric_export_batch();
            assert_eq!(retry.metric_sets.len(), 1);
            assert_eq!(
                retry.metric_sets[0].values,
                vec![otap_df_telemetry::metrics::MetricValue::U64(9)]
            );
        }));
    }

    #[test]
    fn shutdown_interrupts_periodic_metric_export_blocked_by_downstream_backpressure() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let registry = TelemetryRegistryHandle::new();
            let metric_set: otap_df_telemetry::metrics::MetricSet<TestMetrics> =
                registry.register_metric_set(EmptyAttributes());
            registry.accumulate_metric_set_snapshot(
                metric_set.metric_set_key(),
                &[otap_df_telemetry::metrics::MetricValue::U64(9)],
            );

            let (logs_sender, logs_receiver) = flume::bounded(1);
            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {
                    metrics: MetricsConfig {
                        interval: Some(Duration::from_millis(10)),
                        views: Vec::new(),
                    },
                },
                otap_df_telemetry::InternalTelemetrySettings {
                    logs_receiver,
                    resource_bytes: ResourceLogs::default().encode_to_vec().into(),
                    registry: registry.clone(),
                    metrics_interval: Some(Duration::from_millis(10)),
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
                vec![otap_df_telemetry::metrics::MetricValue::U64(9)]
            );

            drop(output_rx);
            drop(logs_sender);
        }));
    }

    #[test]
    fn metric_set_flows_through_collector_across_intervals_and_shutdown() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let engine_reporting_interval = Duration::from_secs(60);
            let receiver_interval = Duration::from_millis(25);
            let registry = TelemetryRegistryHandle::new();
            let config = TelemetryConfig {
                reporting_interval: engine_reporting_interval,
                metrics: SdkMetricsConfig {
                    provider: MetricsProvider::Its,
                    ..SdkMetricsConfig::default()
                },
                logs: LogsConfig {
                    providers: LoggingProviders {
                        global: ProviderMode::Noop,
                        engine: ProviderMode::Noop,
                        internal: ProviderMode::Noop,
                        admin: ProviderMode::Noop,
                    },
                    ..LogsConfig::default()
                },
                ..TelemetryConfig::default()
            };
            let telemetry = InternalTelemetrySystem::new(
                &config,
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
                    metrics: MetricsConfig {
                        interval: Some(receiver_interval),
                        views: Vec::new(),
                    },
                },
                telemetry
                    .internal_telemetry_settings()
                    .expect("ITS metrics should configure the receiver"),
            );

            let (output_tx, output_rx) = create_not_send_channel(4);
            let mut outputs = HashMap::new();
            let _ = outputs.insert("".into(), EngineSender::Local(LocalSender::mpsc(output_tx)));
            let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(4);
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

            // Queue the final snapshot immediately before shutdown. The
            // receiver's FIFO barrier guarantees that it reaches the registry
            // before the final drain, regardless of collector scheduling.
            metric_set.emitted.add(5);
            reporter
                .report(&mut metric_set)
                .expect("final metric snapshot should be queued");
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_secs(1),
                    reason: "test shutdown".to_owned(),
                })
                .expect("shutdown control should be sent");

            let shutdown_output = tokio::time::timeout(Duration::from_secs(1), output_rx.recv())
                .await
                .expect("timed out waiting for final metrics drain")
                .expect("receiver output channel should remain open");
            assert_eq!(decode_metric_value(shutdown_output), 5);

            let receiver_result = receiver_task.await.expect("receiver task should join");
            assert!(receiver_result.is_ok(), "receiver should stop cleanly");
            assert!(
                output_rx.try_recv().is_err(),
                "the shutdown snapshot must be emitted exactly once"
            );

            collector_cancel.cancel();
            collector_task
                .await
                .expect("collector task should join")
                .expect("collector should stop cleanly");
            telemetry
                .shutdown_otel()
                .expect("ITS telemetry shutdown should succeed");
        }));
    }
}
