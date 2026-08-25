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

mod extended_logs;

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = INTERNAL_TELEMETRY_RECEIVER_URN,
    target = "otel.receiver.internal_telemetry",
);

use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
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
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::otlp::ProtoBuffer;
use otel_arrow_dfe_telemetry::event::{LogEvent, ObservedEvent};
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use otel_arrow_dfe_telemetry::metrics::otlp::{
    MetricView, MetricViewSelector, MetricViewStream, MetricsOtlpEncoder,
};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::self_tracing::{ScopeToBytesMap, encode_export_logs_request_batch};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use self::extended_logs::SymbolCache;

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

const MAX_LOG_BATCH_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_LOG_BATCH_MIN_BYTES: usize = 64 * 1024;
const DEFAULT_LOG_BATCH_DURATION: Duration = Duration::from_millis(200);

const fn default_log_batch_min_size() -> NonZeroUsize {
    NonZeroUsize::new(DEFAULT_LOG_BATCH_MIN_BYTES).expect("default batch size is nonzero")
}

const fn default_log_batch_max_size() -> NonZeroUsize {
    NonZeroUsize::new(MAX_LOG_BATCH_BYTES).expect("maximum batch size is nonzero")
}

const fn default_log_batch_duration() -> Duration {
    DEFAULT_LOG_BATCH_DURATION
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

    /// Configuration for internal log output.
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

/// Physical representation emitted for internal logs.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogsRepresentation {
    /// Existing direct OTLP protobuf encoding.
    #[default]
    Otlp,
    /// Canonical OTAP log tables with compact stacktrace extension tables.
    ArrowExtended,
}

/// Configuration for internal log output.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    /// Physical representation emitted by the receiver.
    #[serde(default)]
    pub representation: LogsRepresentation,

    /// Retained bytes that trigger a log batch flush.
    #[serde(default = "default_log_batch_min_size")]
    pub min_size: NonZeroUsize,

    /// Maximum retained bytes in a log batch.
    #[serde(default = "default_log_batch_max_size")]
    pub max_size: NonZeroUsize,

    /// Maximum time the oldest log can wait in a partial batch.
    #[serde(default = "default_log_batch_duration", with = "humantime_serde")]
    pub max_batch_duration: Duration,
}

impl Default for LogsConfig {
    fn default() -> Self {
        Self {
            representation: LogsRepresentation::default(),
            min_size: default_log_batch_min_size(),
            max_size: default_log_batch_max_size(),
            max_batch_duration: default_log_batch_duration(),
        }
    }
}

#[derive(Default)]
struct LogBatch {
    events: Vec<LogEvent>,
    retained_bytes: usize,
}

impl LogBatch {
    fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    fn clear(&mut self) {
        self.events.clear();
        self.retained_bytes = 0;
    }
}

fn retained_log_bytes(event: &LogEvent) -> usize {
    let context_bytes = if event.record.context.spilled() {
        event.record.context.capacity() * size_of::<otel_arrow_dfe_telemetry::registry::EntityKey>()
    } else {
        0
    };
    size_of::<LogEvent>()
        .saturating_add(event.record.body_attrs_bytes.len())
        .saturating_add(context_bytes)
        .saturating_add(
            event
                .record
                .stacktrace
                .as_ref()
                .map_or(0, |stacktrace| stacktrace.retained_bytes()),
        )
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
        if self
            .metrics
            .interval
            .is_some_and(|interval| interval.is_zero())
        {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
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
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver metric view scope attribute '{key}' must be a scalar value"
                ),
            });
        }
        if self.logs.min_size > self.logs.max_size {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs min_size ({}) must not exceed max_size ({})",
                    self.logs.min_size, self.logs.max_size
                ),
            });
        }
        if self.logs.max_size.get() > MAX_LOG_BATCH_BYTES {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: format!(
                    "internal telemetry receiver logs max_size must not exceed {MAX_LOG_BATCH_BYTES} bytes"
                ),
            });
        }
        if self.logs.max_batch_duration.is_zero() {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error:
                    "internal telemetry receiver logs max_batch_duration must be greater than zero"
                        .to_owned(),
            });
        }
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
        let mut scope_cache = ScopeToBytesMap::new(internal.registry.clone());
        let mut symbol_cache = SymbolCache::default();
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
        let mut metrics_interval = interval_at(Instant::now() + metrics_interval, metrics_interval);
        metrics_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut logs_channel_open = logs_enabled;
        let mut pending_metric_export = None;
        let mut log_batch = LogBatch::default();
        let mut log_batch_deadline = None;

        loop {
            if log_batch_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                Self::flush_log_batch(
                    &effect_handler,
                    &mut log_batch,
                    &internal.resource_field_bytes,
                    &mut scope_cache,
                    &mut symbol_cache,
                    logs_config.representation,
                )
                .await?;
                log_batch_deadline = None;
            }

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
                            Self::flush_terminal_telemetry(
                                &effect_handler,
                                &internal,
                                logs_enabled,
                                &mut log_batch,
                                &mut scope_cache,
                                &mut symbol_cache,
                                logs_config,
                                metrics_encoder.as_ref(),
                                deadline,
                            ).await?;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new::<[MetricSetSnapshot; 0]>(deadline, []));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            drop(pending_metric_export.take());
                            Self::flush_terminal_telemetry(
                                &effect_handler,
                                &internal,
                                logs_enabled,
                                &mut log_batch,
                                &mut scope_cache,
                                &mut symbol_cache,
                                logs_config,
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

                // Drain registry metrics at the configured cold-path interval,
                // emitting them only when the metrics signal is enabled.
                _ = metrics_interval.tick(), if pending_metric_export.is_none() => {
                    pending_metric_export = Some(Box::pin(Self::process_metric_batch(
                        &effect_handler,
                        &internal.registry,
                        metrics_encoder.as_ref(),
                    )));
                }

                // Receive logs from the channel
                result = internal.logs_receiver.recv_async(), if logs_channel_open => {
                    match result {
                        Ok(ObservedEvent::Log(log_event)) => {
                            let started_new_batch = Self::buffer_log_event(
                                &effect_handler,
                                &mut log_batch,
                                log_event,
                                &internal.resource_field_bytes,
                                &mut scope_cache,
                                &mut symbol_cache,
                                internal.log_tap.as_ref(),
                                logs_config,
                            ).await?;
                            if log_batch.is_empty() {
                                log_batch_deadline = None;
                            } else if started_new_batch {
                                log_batch_deadline =
                                    Some(Instant::now() + logs_config.max_batch_duration);
                            }
                        }
                        Ok(ObservedEvent::Engine(_)) => {
                            // Engine events are not yet processed
                        }
                        Err(_) => {
                            Self::flush_log_batch(
                                &effect_handler,
                                &mut log_batch,
                                &internal.resource_field_bytes,
                                &mut scope_cache,
                                &mut symbol_cache,
                                logs_config.representation,
                            ).await?;
                            log_batch_deadline = None;
                            logs_channel_open = false;
                        }
                    }
                }

                _ = tokio::time::sleep_until(
                    log_batch_deadline.unwrap_or_else(Instant::now)
                ), if log_batch_deadline.is_some() => {}
            }
        }
    }
}

impl InternalTelemetryReceiver {
    /// Drains queued logs and performs the final bounded metric-registry drain.
    async fn flush_terminal_telemetry(
        effect_handler: &local::EffectHandler<OtapPdata>,
        internal: &otel_arrow_dfe_telemetry::InternalTelemetrySettings,
        logs_enabled: bool,
        log_batch: &mut LogBatch,
        scope_cache: &mut ScopeToBytesMap,
        symbol_cache: &mut SymbolCache,
        logs_config: LogsConfig,
        encoder: Option<&MetricsOtlpEncoder>,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        if logs_enabled {
            tokio::time::timeout_at(Instant::from_std(deadline), async {
                while let Ok(event) = internal.logs_receiver.try_recv() {
                    if let ObservedEvent::Log(log_event) = event {
                        let _ = Self::buffer_log_event(
                            effect_handler,
                            log_batch,
                            log_event,
                            &internal.resource_field_bytes,
                            scope_cache,
                            symbol_cache,
                            internal.log_tap.as_ref(),
                            logs_config,
                        )
                        .await?;
                    }
                }
                Self::flush_log_batch(
                    effect_handler,
                    log_batch,
                    &internal.resource_field_bytes,
                    scope_cache,
                    symbol_cache,
                    logs_config.representation,
                )
                .await
            })
            .await
            .map_err(|_| Error::InternalError {
                message: "timed out while flushing internal logs during shutdown".to_owned(),
            })??;
        }

        Self::process_metric_batch_until(effect_handler, &internal.registry, encoder, deadline)
            .await
    }

    /// Attempts one final metric drain within the pipeline shutdown deadline.
    ///
    /// Timing out cancels [`Self::process_metric_batch`]; its uncommitted export
    /// transaction is then dropped and restores the drained registry values.
    async fn process_metric_batch_until(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
        deadline: std::time::Instant,
    ) -> Result<(), Error> {
        tokio::time::timeout_at(
            Instant::from_std(deadline),
            Self::process_metric_batch(effect_handler, registry, encoder),
        )
        .await
        .map_err(|_| Error::InternalError {
            message: "timed out while flushing internal metrics during shutdown".to_owned(),
        })?
    }

    /// Flushes pending snapshots and consumes one registry export window.
    ///
    /// When an encoder is provided, the batch is converted to OTLP and committed
    /// only after downstream delivery. Without an encoder, the export-only
    /// accumulator is committed immediately without conversion or emission. The
    /// independent admin accumulator is unaffected in both cases.
    async fn process_metric_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        registry: &TelemetryRegistryHandle,
        encoder: Option<&MetricsOtlpEncoder>,
    ) -> Result<(), Error> {
        registry
            .flush_pending_metrics()
            .await
            .map_err(|error| Error::InternalError {
                message: format!("failed to flush internal metrics collector: {error}"),
            })?;
        let export = registry.begin_metric_export_batch();
        let Some(encoder) = encoder else {
            let _ = export.commit();
            return Ok(());
        };
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

    /// Add a log to the bounded receiver-side batch, flushing as required.
    ///
    /// Returns true when the event starts a partial batch whose deadline must
    /// be scheduled by the caller.
    async fn buffer_log_event(
        effect_handler: &local::EffectHandler<OtapPdata>,
        batch: &mut LogBatch,
        log_event: LogEvent,
        resource_field_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
        symbol_cache: &mut SymbolCache,
        log_tap: Option<&otel_arrow_dfe_telemetry::log_tap::InternalLogTapHandle>,
        config: LogsConfig,
    ) -> Result<bool, Error> {
        if let Some(log_tap) = log_tap {
            log_tap.record(log_event.clone());
        }

        let event_bytes = retained_log_bytes(&log_event);
        let mut started_new_batch = batch.is_empty();
        if !batch.is_empty()
            && batch.retained_bytes.saturating_add(event_bytes) > config.max_size.get()
        {
            Self::flush_log_batch(
                effect_handler,
                batch,
                resource_field_bytes,
                scope_cache,
                symbol_cache,
                config.representation,
            )
            .await?;
            started_new_batch = true;
        }

        batch.retained_bytes = batch.retained_bytes.saturating_add(event_bytes);
        batch.events.push(log_event);
        if batch.retained_bytes >= config.min_size.get() {
            Self::flush_log_batch(
                effect_handler,
                batch,
                resource_field_bytes,
                scope_cache,
                symbol_cache,
                config.representation,
            )
            .await?;
            return Ok(false);
        }

        Ok(started_new_batch)
    }

    /// Encode and send all currently buffered logs as one pdata message.
    async fn flush_log_batch(
        effect_handler: &local::EffectHandler<OtapPdata>,
        batch: &mut LogBatch,
        resource_field_bytes: &Bytes,
        scope_cache: &mut ScopeToBytesMap,
        symbol_cache: &mut SymbolCache,
        representation: LogsRepresentation,
    ) -> Result<(), Error> {
        if batch.is_empty() {
            return Ok(());
        }

        let payload = match representation {
            LogsRepresentation::Otlp => {
                let capacity = batch.events.iter().fold(
                    resource_field_bytes.len().saturating_add(512),
                    |capacity, event| capacity.saturating_add(event.record.body_attrs_bytes.len()),
                );
                let mut buf = ProtoBuffer::with_capacity(capacity);
                let _ = encode_export_logs_request_batch(
                    &mut buf,
                    &batch.events,
                    resource_field_bytes,
                    scope_cache,
                );
                OtlpProtoBytes::ExportLogsRequest(buf.into_bytes()).into()
            }
            LogsRepresentation::ArrowExtended => extended_logs::encode(
                &batch.events,
                resource_field_bytes,
                scope_cache,
                symbol_cache,
            )
            .map_err(|error| Error::PdataConversionError { error })?
            .into(),
        };
        let pdata = OtapPdata::new(Context::default(), payload);
        effect_handler.send_message(pdata).await?;
        batch.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::observed_state::SendPolicy;
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
    use otel_arrow_dfe_pdata::PayloadData;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otel_arrow_dfe_telemetry::instrument::Counter;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
    use otel_arrow_dfe_telemetry::{
        __log_record_impl, InternalTelemetrySettings, InternalTelemetrySystem, Level, LogContext,
    };
    use otel_arrow_dfe_telemetry_macros::metric_set;
    use prost::Message as _;
    use std::collections::HashMap;
    use std::time::{Duration, Instant as StdInstant, SystemTime};
    use tokio_util::sync::CancellationToken;

    #[metric_set(name = "receiver.internal_telemetry.test")]
    #[derive(Debug, Default)]
    struct TestMetrics {
        /// Number of test events emitted.
        #[metric(unit = "{event}")]
        emitted: Counter<u64>,
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

    fn test_log_event() -> LogEvent {
        LogEvent {
            time: SystemTime::UNIX_EPOCH,
            record: __log_record_impl!(Level::INFO, "test.itr.batching")
                .into_record(LogContext::new()),
        }
    }

    fn decode_log_count(pdata: OtapPdata) -> usize {
        let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
            pdata.payload().into_data()
        else {
            panic!("internal telemetry receiver emitted a non-logs payload")
        };
        let request = ExportLogsServiceRequest::decode(bytes).expect("valid OTLP logs request");
        request
            .resource_logs
            .iter()
            .flat_map(|resource| &resource.scope_logs)
            .map(|scope| scope.log_records.len())
            .sum()
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
                serde_json::json!({ "min_size": 1024, "max_size": 512 }),
                "must not exceed max_size",
            ),
            (
                serde_json::json!({ "max_size": MAX_LOG_BATCH_BYTES + 1 }),
                "must not exceed 2097152 bytes",
            ),
            (
                serde_json::json!({ "max_batch_duration": "0s" }),
                "must be greater than zero",
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

    /// Scenario: two logs remain below the byte threshold until the batch deadline.
    /// Guarantees: the timer emits one scope-grouped OTLP request before shutdown.
    #[test]
    fn timer_flushes_partial_log_batch() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let (logs_sender, logs_receiver) = flume::bounded(4);
            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {
                    signals: vec![InternalTelemetrySignal::Logs],
                    metrics: MetricsConfig::default(),
                    logs: LogsConfig {
                        min_size: default_log_batch_max_size(),
                        max_batch_duration: Duration::from_millis(20),
                        ..LogsConfig::default()
                    },
                },
                InternalTelemetrySettings {
                    logs_receiver,
                    resource_field_bytes: Bytes::new(),
                    registry: TelemetryRegistryHandle::new(),
                    default_metric_drain_interval: Duration::from_secs(60),
                    log_tap: None,
                },
            );

            let (output_tx, output_rx) = create_not_send_channel(2);
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
            assert_eq!(decode_log_count(output), 2);

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

    /// Scenario: shutdown finds a partial log batch while downstream is blocked.
    /// Guarantees: the terminal log flush fails at the shutdown deadline instead of hanging.
    #[test]
    fn shutdown_bounds_log_flush_blocked_by_downstream_backpressure() {
        let (runtime, local_tasks) = setup_test_runtime();
        runtime.block_on(local_tasks.run_until(async move {
            let (logs_sender, logs_receiver) = flume::bounded(1);
            let receiver = InternalTelemetryReceiver::new_with_telemetry(
                Config {
                    signals: vec![InternalTelemetrySignal::Logs],
                    ..Config::default()
                },
                InternalTelemetrySettings {
                    logs_receiver,
                    resource_field_bytes: Bytes::new(),
                    registry: TelemetryRegistryHandle::new(),
                    default_metric_drain_interval: Duration::from_secs(60),
                    log_tap: None,
                },
            );

            let (output_tx, _output_rx) = create_not_send_channel(1);
            output_tx
                .send(OtapPdata::new(
                    Context::default(),
                    OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into(),
                ))
                .expect("downstream blocker should enqueue");
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

            logs_sender
                .send(ObservedEvent::Log(test_log_event()))
                .expect("log should enqueue");
            ctrl_tx
                .send(NodeControlMsg::Shutdown {
                    deadline: StdInstant::now() + Duration::from_millis(20),
                    reason: "test deadline".to_owned(),
                })
                .expect("shutdown should enqueue");

            let result = tokio::time::timeout(Duration::from_millis(500), receiver_task)
                .await
                .expect("shutdown deadline must bound the blocked log flush")
                .expect("receiver task should join");
            let error = match result {
                Ok(_) => panic!("blocked terminal log flush should fail"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("timed out while flushing internal logs"),
                "unexpected error: {error}"
            );
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

            let _error = InternalTelemetryReceiver::process_metric_batch(
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

            InternalTelemetryReceiver::process_metric_batch(&effect_handler, &registry, None)
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
