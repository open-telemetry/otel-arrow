// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor.
//! Batches OtapPdata by count or timer; uses upstream OTAP batching for merge/split.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error as EngineError, ProcessorErrorKind};
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::otap::batching::make_output_batches;
use otap_df_pdata::{OtapPayload, OtlpProtoBytes};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::num::NonZeroU64;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otel:batch:processor";

/// Default configuration values (parity-aligned as we confirm Go defaults)
pub const DEFAULT_SEND_BATCH_SIZE: usize = 8192;

/// Timeout in milliseconds for periodic flush
pub const DEFAULT_TIMEOUT_MS: u64 = 200;

/// Log messages
const LOG_MSG_SHUTTING_DOWN: &str = "OTAP batch processor shutting down";
const LOG_MSG_DROP_CONVERSION_FAILED: &str =
    "OTAP batch processor: dropping message: OTAP conversion failed";
const LOG_MSG_BATCHING_FAILED_PREFIX: &str = "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// Configuration for the OTAP batch processor (parity with Go batchprocessor)
///
/// TODO these are currently modeled on the legacy batch processor. we want
/// to emulate the new exporterhelper batch sender.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Flush current batch when this count is reached, as a
    /// minimum. Measures the number of items (logs: records, traces:
    /// spans, metrics: data points). When pending data reaches this
    /// threshold a new batch will form and be sent.
    #[serde(default = "default_send_batch_size")]
    pub send_batch_size: Option<NonZeroUsize>,

    /// Optionally limit batch sizes to an upper bound. Measured in
    /// items, as described for send_batch_size.
    #[serde(default = "default_send_batch_max_size")]
    pub send_batch_max_size: Option<NonZeroUsize>,

    /// Flush non-empty batches on this interval, which may be 0 for
    /// immediate flush or None for no timeout.
    #[serde(with = "humantime_serde", default = "default_timeout_duration")]
    pub timeout: Duration,
}

const fn default_send_batch_size() -> Option<NonZeroUsize> {
    NonZeroUsize::new(DEFAULT_SEND_BATCH_SIZE)
}

const fn default_send_batch_max_size() -> Option<NonZeroUsize> {
    // No upper-bound
    None
}

const fn default_timeout_duration() -> Duration {
    Duration::from_millis(DEFAULT_TIMEOUT_MS)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            send_batch_size: default_send_batch_size(),
            send_batch_max_size: default_send_batch_max_size(),
            timeout: default_timeout_duration(),
        }
    }
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // At least one size is set.
        if self.send_batch_size.or(self.send_batch_max_size).is_none() {
            return Err(ConfigError::InvalidUserConfig {
                error: "send_batch_max_size or send_batch_size must be set".into(),
            });
        }

        // If both sizes are set, check max_size is >= the batch_size.
        if let (Some(max_size), Some(batch_size)) = (self.send_batch_max_size, self.send_batch_size)
        {
            if max_size < batch_size {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "send_batch_max_size ({}) must be >= send_batch_size ({}) or unset",
                        max_size, batch_size,
                    ),
                });
            }
        }

        // Zero-timeout is a valid split-only configuration.
        if self.timeout == Duration::ZERO {
            if let Some(batch_size) = self.send_batch_size {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!("send_batch_size ({}) requires a timeout", batch_size),
                });
            }
            if self.send_batch_max_size.is_none() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "send_batch_max_size required for split-only (no timeout) configuration"
                        .into(),
                });
            }
        }

        Ok(())
    }
}

/// Per-signal state
#[derive(Default)]
struct BatchSignals {
    logs: SignalBuffer,
    metrics: SignalBuffer,
    traces: SignalBuffer,
}

/// Per-signal buffer state
#[derive(Default)]
struct SignalBuffer {
    /// Pending input for the next call to otap_df_pdata::otap::make_output_batches.
    pending: Vec<OtapArrowRecords>,

    /// A count defined by batch_length(), number of spans, log records, or metric data points.
    items: usize,

    /// Arrival time of the oldest data.
    arrival: Option<Instant>,
}

/// Local (!Send) batch processor
pub struct BatchProcessor {
    config: Config,
    signals: BatchSignals,
    lower_limit: NonZeroUsize,
    metrics: MetricSet<BatchProcessorMetrics>,
}

/// There are three reasons to flush.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FlushReason {
    Size,
    Timer,
    Shutdown,
}

/// Minimal, essential metrics for the batch processor.
#[metric_set(name = "otap.processor.batch")]
#[derive(Debug, Default, Clone)]
pub struct BatchProcessorMetrics {
    /// Total items consumed for logs signal
    #[metric(unit = "{item}")]
    consumed_items_logs: Counter<u64>,
    /// Total items consumed for metrics signal
    #[metric(unit = "{item}")]
    consumed_items_metrics: Counter<u64>,
    /// Total items consumed for traces signal
    #[metric(unit = "{item}")]
    consumed_items_traces: Counter<u64>,

    /// Total batches consumed for logs signal
    #[metric(unit = "{item}")]
    consumed_batches_logs: Counter<u64>,
    /// Total batches consumed for metrics signal
    #[metric(unit = "{item}")]
    consumed_batches_metrics: Counter<u64>,
    /// Total batches consumed for traces signal
    #[metric(unit = "{item}")]
    consumed_batches_traces: Counter<u64>,

    /// Total items produced for logs signal
    #[metric(unit = "{item}")]
    produced_items_logs: Counter<u64>,
    /// Total items produced for metrics signal
    #[metric(unit = "{item}")]
    produced_items_metrics: Counter<u64>,
    /// Total items produced for traces signal
    #[metric(unit = "{item}")]
    produced_items_traces: Counter<u64>,

    /// Total batches produced for logs signal
    #[metric(unit = "{item}")]
    produced_batches_logs: Counter<u64>,
    /// Total batches produced for metrics signal
    #[metric(unit = "{item}")]
    produced_batches_metrics: Counter<u64>,
    /// Total batches produced for traces signal
    #[metric(unit = "{item}")]
    produced_batches_traces: Counter<u64>,

    /// Number of flushes triggered by size threshold (all signals)
    #[metric(unit = "{flush}")]
    flushes_size: Counter<u64>,
    /// Number of flushes triggered by timer (all signals)
    #[metric(unit = "{flush}")]
    flushes_timer: Counter<u64>,

    /// Number of messages dropped due to conversion failures
    #[metric(unit = "{msg}")]
    dropped_conversion: Counter<u64>,
    /// Number of batches for which errors encountered
    #[metric(unit = "{error}")]
    batching_errors: Counter<u64>,
    /// Number of empty records dropped
    #[metric(unit = "{msg}")]
    dropped_empty_records: Counter<u64>,
}

fn nzu_to_nz64(nz: Option<NonZeroUsize>) -> Option<NonZeroU64> {
    nz.map(|nz| NonZeroU64::new(nz.get() as u64).expect("nonzero"))
}

async fn log_batching_failed(
    effect: &mut local::EffectHandler<OtapPdata>,
    signal: SignalType,
    err: &impl std::fmt::Display,
) {
    effect
        .info(&format!(
            "{LOG_MSG_BATCHING_FAILED_PREFIX} {signal:?}: {err}{LOG_MSG_BATCHING_FAILED_SUFFIX}"
        ))
        .await;
}

impl BatchProcessor {
    /// Parse JSON config and build the processor instance with the provided metrics set.
    /// This function does not wrap the processor into a ProcessorWrapper so callers can
    /// preserve the original NodeUserConfig (including out_ports/default_out_port).
    pub fn build_from_json(
        cfg: &Value,
        metrics: MetricSet<BatchProcessorMetrics>,
    ) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(cfg.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        // This checks that if both are present, max_size >= batch_size, and
        // that at least one is present so that lower_limit is valid below.
        config.validate()?;

        let lower_limit = config
            .send_batch_size
            .or(config.send_batch_max_size)
            .expect("valid");

        Ok(BatchProcessor {
            config,
            signals: BatchSignals::default(),
            lower_limit,
            metrics,
        })
    }

    /// Backward-compatible helper used by unit tests to construct a processor wrapper
    /// directly from JSON. Note: This creates a fresh NodeUserConfig without out_ports,
    /// which is fine for unit tests that do not rely on engine wiring.
    pub fn from_config(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
        metrics: MetricSet<BatchProcessorMetrics>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let proc = Self::build_from_json(cfg, metrics)?;
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            OTAP_BATCH_PROCESSOR_URN,
        ));
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Flush all per-signal buffers (logs, metrics, traces).
    async fn flush_shutdown(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        for signal in [SignalType::Logs, SignalType::Traces, SignalType::Metrics] {
            self.flush_signal_impl(signal, effect, Instant::now(), FlushReason::Shutdown)
                .await?;
        }
        Ok(())
    }

    /// Helper to process incoming signal data with generic batching logic
    async fn process_signal_impl(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        rec: OtapArrowRecords,
    ) -> Result<(), EngineError> {
        let items = rec.batch_length();

        if items == 0 {
            self.metrics.dropped_empty_records.inc();
            return Ok(());
        }

        // Increment consumed_items for the appropriate signal
        match signal {
            SignalType::Logs => {
                self.metrics.consumed_items_logs.add(items as u64);
                self.metrics.consumed_batches_logs.add(1);
            }
            SignalType::Metrics => {
                self.metrics.consumed_items_metrics.add(items as u64);
                self.metrics.consumed_batches_metrics.add(1);
            }
            SignalType::Traces => {
                self.metrics.consumed_items_traces.add(items as u64);
                self.metrics.consumed_batches_traces.add(1);
            }
        }

        let timeout = self.config.timeout;

        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };

        let mut arrival: Option<Instant> = None;
        if buffer.items == 0 {
            let now = Instant::now();
            arrival = Some(now);
            if timeout != Duration::ZERO {
                buffer.set_arrival(signal, now, timeout, effect).await?;
            }
        }
        buffer.items += items;
        buffer.pending.push(rec);

        // Flush based on size when the batch reaches the lower limit.
        if self.config.timeout != Duration::ZERO && buffer.items < self.lower_limit.get() {
            Ok(())
        } else {
            self.flush_signal_impl(
                signal,
                effect,
                arrival.unwrap_or_else(Instant::now),
                FlushReason::Size,
            )
            .await
        }
    }

    /// Generic flush implementation for any signal type
    async fn flush_signal_impl(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        now: Instant,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };

        // If the input is empty.
        if buffer.pending.is_empty() {
            return Ok(());
        }
        // If this is a timer-based flush and we were called too soon,
        // skip. this may happen if the batch for which the timer was set
        // flushes for size before the timer.
        if reason == FlushReason::Timer
            && self.config.timeout != Duration::ZERO
            && now.duration_since(buffer.arrival.expect("timer")) < self.config.timeout
        {
            return Ok(());
        }

        let input = buffer.pending.drain(..).collect::<Vec<_>>();

        buffer.items = 0;

        match reason {
            FlushReason::Size => self.metrics.flushes_size.inc(),
            FlushReason::Timer => self.metrics.flushes_timer.inc(),
            FlushReason::Shutdown => {}
        }

        let count = input.len();
        let upper_limit = self.config.send_batch_max_size;
        let mut output_batches = match make_output_batches(signal, nzu_to_nz64(upper_limit), input)
        {
            Ok(v) => v,
            Err(e) => {
                self.metrics.batching_errors.add(count as u64);
                log_batching_failed(effect, signal, &e).await;
                return Err(EngineError::InternalError {
                    message: e.to_string(),
                });
            }
        };

        // If size-triggered and we requested splitting (upper_limit is Some), re-buffer the last partial
        // output if it is smaller than the configured lower_limit. Timer/Shutdown flush everything.
        if self.config.timeout != Duration::ZERO
            && reason == FlushReason::Size
            && upper_limit.is_some()
            && !output_batches.is_empty()
        {
            if let Some(last_items) = output_batches.last().map(|last| last.batch_length()) {
                if last_items < self.lower_limit.get() {
                    let remainder = output_batches.pop().expect("last exists");

                    // We use the latest arrival time as the new arrival for timeout purposes.
                    buffer.items = last_items;
                    buffer.pending.push(remainder);
                    buffer
                        .set_arrival(signal, now, self.config.timeout, effect)
                        .await?;
                }
            }
        }

        for records in output_batches {
            let items = records.batch_length();
            let pdata = OtapPdata::new_todo_context(records.into());

            // Increment produced_items for the appropriate signal
            match signal {
                SignalType::Logs => {
                    self.metrics.produced_items_logs.add(items as u64);
                    self.metrics.produced_batches_logs.add(1);
                }
                SignalType::Metrics => {
                    self.metrics.produced_items_metrics.add(items as u64);
                    self.metrics.produced_batches_metrics.add(1);
                }
                SignalType::Traces => {
                    self.metrics.produced_items_traces.add(items as u64);
                    self.metrics.produced_batches_traces.add(1);
                }
            }

            effect.send_message(pdata).await?;
        }

        Ok(())
    }
}

/// Factory function to create a batch processor.
pub fn create_otap_batch_processor(
    pipeline_ctx: otap_df_engine::context::PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let metrics = pipeline_ctx.register_metrics::<BatchProcessorMetrics>();
    let proc = BatchProcessor::build_from_json(&node_config.config, metrics)?;
    Ok(ProcessorWrapper::local(
        proc,
        node,
        node_config,
        processor_config,
    ))
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for BatchProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(ctrl) => match ctrl {
                NodeControlMsg::Config { .. } => Ok(()),
                NodeControlMsg::Shutdown { .. } => {
                    self.flush_shutdown(effect).await?;
                    effect.info(LOG_MSG_SHUTTING_DOWN).await;
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => metrics_reporter.report(&mut self.metrics).map_err(|e| {
                    EngineError::InternalError {
                        message: e.to_string(),
                    }
                }),
                NodeControlMsg::DelayedData { data, .. } => {
                    let signal = data.signal_type();
                    self.flush_signal_impl(signal, effect, Instant::now(), FlushReason::Timer)
                        .await?;
                    Ok(())
                }
                NodeControlMsg::TimerTick { .. }
                | NodeControlMsg::Ack { .. }
                | NodeControlMsg::Nack { .. } => unreachable!(),
            },
            Message::PData(request) => {
                let signal_type = request.signal_type();

                // TODO(#498): Use the context
                let (_ctx, data) = request.into_parts();

                match OtapArrowRecords::try_from(data) {
                    Ok(rec) => self.process_signal_impl(signal_type, effect, rec).await,
                    Err(_) => {
                        // Conversion failed: log and drop (TODO: Nack)
                        self.metrics.dropped_conversion.inc();
                        effect.info(LOG_MSG_DROP_CONVERSION_FAILED).await;
                        Ok(())
                    }
                }
            }
        }
    }
}

impl SignalBuffer {
    /// Called when a signal buffer changes state from zero to non-zero
    /// items.  Schedules a wakeup for the moment when the pending buffer
    /// should flush, where `now` is the current timestamp.
    async fn set_arrival(
        &mut self,
        signal: SignalType,
        now: Instant,
        timeout: Duration,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.arrival = Some(now);

        // TODO: We are using an empty DelayData request, which is
        // relatively bogus way to set a timer. We should revisit how
        // to set one-shot timers. Note! The processor trait does not
        // have a start() method, so it's relatively not easy to set a
        // periodic timer in this component. Note! This means we might
        // have a bunch of timers pending, when we want a timer we can
        // cancel.
        let wakeup: OtapPayload = match signal {
            SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(Bytes::new()),
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(Bytes::new()),
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(Bytes::new()),
        }
        .into();

        effect
            .delay_data(
                now + timeout,
                Box::new(OtapPdata::new(Context::default(), wakeup)),
            )
            .await
            .map_err(|_| EngineError::ProcessorError {
                processor: effect.processor_id(),
                kind: ProcessorErrorKind::Other,
                error: "could not set one-shot timer".into(),
                source_detail: "".into(),
            })
    }
}

/// Register factory for OTAP batch processor
#[allow(unsafe_code)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static OTAP_BATCH_PROCESSOR_FACTORY: otap_df_engine::ProcessorFactory<OtapPdata> =
    otap_df_engine::ProcessorFactory {
        name: OTAP_BATCH_PROCESSOR_URN,
        create: |pipeline_ctx: otap_df_engine::context::PipelineContext,
                 node: NodeId,
                 node_config: Arc<NodeUserConfig>,
                 proc_cfg: &ProcessorConfig| {
            create_otap_batch_processor(pipeline_ctx, node, node_config, proc_cfg)
        },
    };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_config::node::{DispatchStrategy, HyperEdgeConfig, NodeKind};
    use otap_df_config::{PipelineGroupId, PipelineId};
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::message::Message;
    use otap_df_engine::node::Node;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::encode::{encode_logs_otap_batch, encode_spans_otap_batch};
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::OtlpProtoMessage;
    use otap_df_pdata::testing::equiv::assert_equivalent;
    use otap_df_pdata::testing::fixtures::DataGenerator;
    use otap_df_pdata::testing::round_trip::otap_to_otlp;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Helper to create test pipeline context
    fn create_test_pipeline_context() -> (
        otap_df_engine::context::PipelineContext,
        MetricsRegistryHandle,
    ) {
        let metrics_registry = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            PipelineGroupId::from("test_group".to_string()),
            PipelineId::from("test_pipeline".to_string()),
            0,
            0,
        );
        (pipeline_ctx, metrics_registry)
    }

    /// Helper to set up a test runtime with batch processor
    fn setup_test_runtime(
        cfg: Value,
    ) -> (
        MetricsRegistryHandle,
        otap_df_telemetry::reporter::MetricsReporter,
        otap_df_engine::testing::processor::TestPhase<OtapPdata>,
    ) {
        let rt = TestRuntime::new();
        let metrics_registry = rt.metrics_registry();
        let metrics_reporter = rt.metrics_reporter();

        // Create processor using TestRuntime's registry
        let controller = ControllerContext::new(metrics_registry.clone());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
        let node = test_node("batch-processor-test");
        let mut node_config = NodeUserConfig::new_processor_config(OTAP_BATCH_PROCESSOR_URN);
        node_config.config = cfg;
        let proc_config = ProcessorConfig::new("batch");
        let proc =
            create_otap_batch_processor(pipeline_ctx, node, Arc::new(node_config), &proc_config)
                .expect("create processor");

        let phase = rt.set_processor(proc);

        (metrics_registry, metrics_reporter, phase)
    }

    /// Helper to verify consumed and produced item metrics
    fn verify_item_metrics(
        metrics_registry: &MetricsRegistryHandle,
        signal: SignalType,
        expected_items: usize,
    ) {
        let mut consumed_items = 0u64;
        let mut produced_items = 0u64;
        let mut consumed_batches = 0u64;
        let mut produced_batches = 0u64;

        metrics_registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "otap.processor.batch" {
                for (field, value) in iter {
                    match (signal, field.name) {
                        (SignalType::Logs, "consumed.items.logs") => consumed_items = value,
                        (SignalType::Logs, "produced.items.logs") => produced_items = value,
                        (SignalType::Traces, "consumed.items.traces") => consumed_items = value,
                        (SignalType::Traces, "produced.items.traces") => produced_items = value,
                        (SignalType::Logs, "consumed.batches.logs") => consumed_batches = value,
                        (SignalType::Logs, "produced.batches.logs") => produced_batches = value,
                        (SignalType::Traces, "consumed.batches.traces") => consumed_batches = value,
                        (SignalType::Traces, "produced.batches.traces") => produced_batches = value,
                        _ => {}
                    }
                }
            }
        });

        assert_eq!(
            consumed_items as usize, expected_items,
            "consumed_items metric must match"
        );
        assert_eq!(
            produced_items as usize, expected_items,
            "produced_items metric must match"
        );
        assert!(produced_batches != 0, "produced_batches != 0");
        assert!(consumed_batches != 0, "consumed_batches != 0");
    }

    #[test]
    fn test_factory_config_ports() {
        // Build a pipeline context to register metrics
        let (pipeline_ctx, _registry) = create_test_pipeline_context();

        // Prepare a NodeUserConfig with an out_port and a default_out_port
        let mut nuc = NodeUserConfig::with_user_config(
            NodeKind::Processor,
            OTAP_BATCH_PROCESSOR_URN.into(),
            serde_json::json!({
                "batch_send_size": "1000",
            }),
        );
        let mut dests: HashSet<otap_df_config::NodeId> = HashSet::new();
        let _ = dests.insert("exporter".into());
        let edge = HyperEdgeConfig {
            destinations: dests,
            dispatch_strategy: DispatchStrategy::RoundRobin,
        };
        let _ = nuc.add_out_port("out_port".into(), edge);
        nuc.set_default_out_port("out_port");
        let nuc = Arc::new(nuc);

        // Create processor via factory and ensure the provided NodeUserConfig is preserved
        let proc_cfg = ProcessorConfig::new("batch");
        let node = test_node(proc_cfg.name.clone());
        let wrapper = create_otap_batch_processor(pipeline_ctx, node, nuc.clone(), &proc_cfg)
            .expect("factory should succeed");

        let uc = wrapper.user_config();
        assert!(uc.out_ports.contains_key("out_port"));
        assert_eq!(uc.default_out_port.as_deref(), Some("out_port"));
        let edge = &uc.out_ports["out_port"];
        assert!(edge.destinations.contains("exporter"));
    }

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_config_validation() {
        // Both sizes set, max >= batch: OK
        let cfg = Config {
            send_batch_size: NonZeroUsize::new(100),
            send_batch_max_size: NonZeroUsize::new(200),
            timeout: Duration::from_millis(100),
        };
        assert!(cfg.validate().is_ok());

        // Only batch size: OK
        let cfg = Config {
            send_batch_size: NonZeroUsize::new(100),
            send_batch_max_size: None,
            timeout: Duration::from_millis(100),
        };
        assert!(cfg.validate().is_ok());

        // Only max size: OK
        let cfg = Config {
            send_batch_size: None,
            send_batch_max_size: NonZeroUsize::new(200),
            timeout: Duration::from_millis(100),
        };
        assert!(cfg.validate().is_ok());

        // Split-only: OK
        let cfg = Config {
            send_batch_size: None,
            send_batch_max_size: NonZeroUsize::new(100),
            timeout: Duration::ZERO,
        };
        assert!(cfg.validate().is_ok());

        // Both None: ERROR
        let cfg = Config {
            send_batch_size: None,
            send_batch_max_size: None,
            timeout: Duration::from_millis(100),
        };
        assert!(cfg.validate().is_err());

        // Max < batch: ERROR
        let cfg = Config {
            send_batch_size: NonZeroUsize::new(200),
            send_batch_max_size: NonZeroUsize::new(100),
            timeout: Duration::from_millis(100),
        };
        assert!(cfg.validate().is_err());

        // lower-bound without timeout: ERROR
        let cfg = Config {
            send_batch_size: NonZeroUsize::new(200),
            send_batch_max_size: NonZeroUsize::new(100),
            timeout: Duration::ZERO,
        };
        assert!(cfg.validate().is_err());
    }

    /// Generic test helper for size-based flush
    fn test_size_flush(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
        let cfg = json!({
            "send_batch_size": 4,
            "send_batch_max_size": 5,
            "timeout": "1s"
        });

        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(cfg);

        let inputs_otlp: Vec<_> = inputs_otlp.collect();
        let signal = inputs_otlp[0].signal_type();
        let input_item_count: usize = inputs_otlp.iter().map(|m| m.batch_length()).sum();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                // Spawn controller task to handle timer events
                let controller_task = tokio::spawn(async move {
                    while let Ok(msg) = pipeline_rx.recv().await {
                        if let PipelineControlMsg::DelayData { .. } = msg {}
                    }
                });

                for input_otlp in &inputs_otlp {
                    let otap_records = match input_otlp {
                        OtlpProtoMessage::Traces(t) => encode_spans_otap_batch(t).expect("encode"),
                        OtlpProtoMessage::Logs(l) => encode_logs_otap_batch(l).expect("encode"),
                        OtlpProtoMessage::Metrics(_) => panic!("metrics not supported"),
                    };
                    let pdata = OtapPdata::new_default(otap_records.into());
                    ctx.process(Message::PData(pdata)).await.expect("process");
                }

                // Should flush once when threshold (3) is reached
                let mut emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 1, "expected one batch after size threshold");

                let first_batch_rec: OtapArrowRecords =
                    emitted.remove(0).payload().try_into().unwrap();
                assert!(
                    first_batch_rec.batch_length() >= 3,
                    "first batch has at least threshold items"
                );

                // Shutdown should flush remaining items
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(100),
                    reason: "test".into(),
                }))
                .await
                .expect("shutdown");

                let remaining_emitted = ctx.drain_pdata().await;
                assert!(
                    !remaining_emitted.is_empty(),
                    "should flush remaining items"
                );

                // Verify equivalence across all outputs
                let all_outputs: Vec<OtlpProtoMessage> = std::iter::once(first_batch_rec)
                    .chain(remaining_emitted.into_iter().map(|p| {
                        let rec: OtapArrowRecords = p.payload().try_into().unwrap();
                        rec
                    }))
                    .map(|r| otap_to_otlp(&r))
                    .collect();
                assert_equivalent(&inputs_otlp, &all_outputs);

                // Trigger telemetry collection
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");

                controller_task.abort();
            })
            .validate(move |_| async move {
                // Allow metrics to be reported
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify metrics
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    #[test]
    fn test_size_flush_traces() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_traces().into()));
    }

    #[test]
    fn test_size_flush_logs() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_logs().into()));
    }

    /// Generic test helper for timer flush
    fn test_timer_flush(input_otlp: OtlpProtoMessage) {
        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 10,
            "timeout": "50ms"
        });

        let signal = input_otlp.signal_type();
        let input_item_count = input_otlp.batch_length();

        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(cfg);

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                let otap_records = match &input_otlp {
                    OtlpProtoMessage::Traces(t) => encode_spans_otap_batch(t).expect("encode"),
                    OtlpProtoMessage::Logs(l) => encode_logs_otap_batch(l).expect("encode"),
                    OtlpProtoMessage::Metrics(_) => panic!("metrics not supported"),
                };
                let pdata = OtapPdata::new_default(otap_records.into());

                ctx.process(Message::PData(pdata)).await.expect("process");

                // Should not flush immediately (below threshold of 10)
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 0, "no immediate flush below threshold");

                // Simulate pipeline controller
                let delayed_msg =
                    tokio::time::timeout(Duration::from_millis(200), pipeline_rx.recv())
                        .await
                        .expect("timeout")
                        .expect("channel closed");

                if let PipelineControlMsg::DelayData { when, data, .. } = delayed_msg {
                    ctx.sleep(when.duration_since(Instant::now())).await;
                    ctx.process(Message::Control(NodeControlMsg::DelayedData { when, data }))
                        .await
                        .expect("process delayed");

                    let emitted = ctx.drain_pdata().await;
                    assert_eq!(emitted.len(), 1, "timer should flush");

                    let output_rec: OtapArrowRecords =
                        emitted[0].clone().payload().try_into().unwrap();
                    let output_otlp = otap_to_otlp(&output_rec);

                    use std::slice::from_ref;
                    assert_equivalent(from_ref(&input_otlp), from_ref(&output_otlp));

                    // Trigger telemetry collection
                    ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                        metrics_reporter,
                    }))
                    .await
                    .expect("collect telemetry");
                } else {
                    panic!("Expected DelayData");
                }
            })
            .validate(move |_| async move {
                // Allow metrics to be reported
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify metrics
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    #[test]
    fn test_timer_flush_traces() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_traces().into());
    }

    #[test]
    fn test_timer_flush_logs() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_logs().into());
    }

    /// Generic test helper for splitting oversize batches
    fn test_split_oversize(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
        let cfg = json!({
            "send_batch_size": 3,
            "send_batch_max_size": 3,
            "timeout": "1s"
        });

        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(cfg);

        let inputs_otlp: Vec<_> = inputs_otlp.collect();
        let signal = inputs_otlp[0].signal_type();
        let input_item_count: usize = inputs_otlp.iter().map(|m| m.batch_length()).sum();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                // Spawn controller task to handle DelayData messages
                let controller_task = tokio::spawn(async move {
                    while let Ok(msg) = pipeline_rx.recv().await {
                        if let PipelineControlMsg::DelayData { .. } = msg {}
                    }
                });

                for input in &inputs_otlp {
                    let otap_records = match input {
                        OtlpProtoMessage::Traces(t) => encode_spans_otap_batch(t).expect("encode"),
                        OtlpProtoMessage::Logs(l) => encode_logs_otap_batch(l).expect("encode"),
                        OtlpProtoMessage::Metrics(_) => panic!("metrics not supported"),
                    };
                    let pdata = OtapPdata::new_default(otap_records.into());
                    ctx.process(Message::PData(pdata)).await.expect("process");
                }

                // Should emit 3 batches of 3 items each
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 3, "should emit 3 full batches");

                for batch in &emitted {
                    let rec: OtapArrowRecords = batch.clone().payload().try_into().unwrap();
                    assert_eq!(rec.batch_length(), 3, "each batch should have 3 items");
                }

                // Shutdown should have nothing remaining
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(100),
                    reason: "test".into(),
                }))
                .await
                .expect("shutdown");

                let final_emitted = ctx.drain_pdata().await;
                assert_eq!(final_emitted.len(), 0, "no remaining data");

                // Verify equivalence
                let all_outputs: Vec<OtlpProtoMessage> = emitted
                    .into_iter()
                    .map(|p| {
                        let rec: OtapArrowRecords = p.payload().try_into().unwrap();
                        otap_to_otlp(&rec)
                    })
                    .collect();
                assert_equivalent(&inputs_otlp, &all_outputs);

                // Trigger telemetry collection
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");

                controller_task.abort();
            })
            .validate(move |_| async move {
                // Allow metrics to be reported
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify metrics
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    #[test]
    fn test_split_oversize_traces() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_traces().into()));
    }

    #[test]
    fn test_split_oversize_logs() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_logs().into()));
    }

    /// Generic test helper for concatenation and rebuffering
    fn test_concatenate(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
        let cfg = json!({
            "send_batch_size": 5,
            "send_batch_max_size": 10,
            "timeout": "1s"
        });

        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(cfg);

        let inputs_otlp: Vec<_> = inputs_otlp.collect();
        let signal = inputs_otlp[0].signal_type();
        let input_item_count: usize = inputs_otlp.iter().map(|m| m.batch_length()).sum();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                let controller_task = tokio::spawn(async move {
                    while let Ok(msg) = pipeline_rx.recv().await {
                        if let PipelineControlMsg::DelayData { .. } = msg {}
                    }
                });

                for input_otlp in &inputs_otlp {
                    let otap_records = match input_otlp {
                        OtlpProtoMessage::Traces(t) => encode_spans_otap_batch(t).expect("encode"),
                        OtlpProtoMessage::Logs(l) => encode_logs_otap_batch(l).expect("encode"),
                        OtlpProtoMessage::Metrics(_) => panic!("metrics not supported"),
                    };
                    let pdata = OtapPdata::new_default(otap_records.into());
                    ctx.process(Message::PData(pdata)).await.expect("process");
                }

                // Should flush at 6 and 12 items, i.e., twice.
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 2, "should flush two batches at threshold");

                let records: Vec<_> = emitted
                    .into_iter()
                    .map(|d| {
                        let rec: OtapArrowRecords = d.clone().payload().try_into().unwrap();
                        assert_eq!(rec.batch_length(), 6);
                        rec
                    })
                    .collect();

                // Shutdown before drain
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(100),
                    reason: "test".into(),
                }))
                .await
                .expect("shutdown");

                let final_emitted = ctx.drain_pdata().await;
                assert_eq!(final_emitted.len(), 0, "shutdown flushes nothing");

                // Verify equivalence
                let all_outputs: Vec<OtlpProtoMessage> =
                    records.into_iter().map(|r| otap_to_otlp(&r)).collect();
                assert_equivalent(&inputs_otlp, &all_outputs);

                // Trigger telemetry collection
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");

                controller_task.abort();
            })
            .validate(move |_| async move {
                // Allow metrics to be reported
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify metrics
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    #[test]
    fn test_concatenate_traces() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_traces().into()));
    }

    #[test]
    fn test_concatenate_logs() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_logs().into()));
    }

    /// Generic test helper for split-only mode (no timeout, no send_batch_size)
    fn test_split_only(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
        let cfg = json!({
            "send_batch_size": null,
            "send_batch_max_size": 2,
            "timeout": "0s",
        });

        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(cfg);

        let inputs_otlp: Vec<_> = inputs_otlp.collect();
        let signal = inputs_otlp[0].signal_type();
        let input_item_count: usize = inputs_otlp.iter().map(|m| m.batch_length()).sum();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                let controller_task = tokio::spawn(async move {
                    while let Ok(msg) = pipeline_rx.recv().await {
                        if let PipelineControlMsg::DelayData { .. } = msg {}
                    }
                });

                for input_otlp in &inputs_otlp {
                    let otap_records = match input_otlp {
                        OtlpProtoMessage::Traces(t) => encode_spans_otap_batch(t).expect("encode"),
                        OtlpProtoMessage::Logs(l) => encode_logs_otap_batch(l).expect("encode"),
                        OtlpProtoMessage::Metrics(_) => panic!("metrics not supported"),
                    };
                    let pdata = OtapPdata::new_default(otap_records.into());
                    ctx.process(Message::PData(pdata)).await.expect("process");
                }

                // With no send_batch_size and no timeout, flushes immediately on every input
                // With send_batch_max_size=2, each 3-item input splits to [2, 1]
                let emitted = ctx.drain_pdata().await;
                assert_eq!(emitted.len(), 4, "should emit 4 batches: 2, 1, 2, 1");

                let batch_sizes: Vec<usize> = emitted
                    .iter()
                    .map(|p| {
                        let rec: OtapArrowRecords = p.clone().payload().try_into().unwrap();
                        rec.batch_length()
                    })
                    .collect();
                assert_eq!(
                    batch_sizes,
                    vec![2, 1, 2, 1],
                    "split pattern without rebuffering"
                );

                // Shutdown should have nothing remaining (all flushed)
                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(100),
                    reason: "test".into(),
                }))
                .await
                .expect("shutdown");

                let final_emitted = ctx.drain_pdata().await;
                assert_eq!(final_emitted.len(), 0, "no remaining data");

                // Verify equivalence
                let all_outputs: Vec<OtlpProtoMessage> = emitted
                    .into_iter()
                    .map(|p| {
                        let rec: OtapArrowRecords = p.payload().try_into().unwrap();
                        otap_to_otlp(&rec)
                    })
                    .collect();
                assert_equivalent(&inputs_otlp, &all_outputs);

                // Trigger telemetry collection
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");

                controller_task.abort();
            })
            .validate(move |_| async move {
                // Allow metrics to be reported
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify metrics
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    #[test]
    fn test_split_only_traces() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_traces().into()));
    }

    #[test]
    fn test_split_only_logs() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_logs().into()));
    }
}
