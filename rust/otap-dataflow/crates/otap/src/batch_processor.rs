// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor.
//! Batches OtapPdata by count or timer; uses upstream OTAP batching for merge/split.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::otap::batching::make_output_batches;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::num::NonZeroU64;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otap:processor:batch";

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
    /// Flush current batch when this count is reached (min).
    #[serde(default = "default_send_batch_size")]
    pub send_batch_size: Option<NonZeroUsize>,
    /// Flush current batch when this count is reached (max).
    #[serde(default = "default_send_batch_max_size")]
    pub send_batch_max_size: Option<NonZeroUsize>,
    /// Flush non-empty batches on this interval, which may be 0 for
    /// immediate flush or None for no timeout.
    #[serde(with = "humantime_serde", default = "default_timeout_duration")]
    pub timeout: Option<Duration>,
}

fn default_send_batch_size() -> Option<NonZeroUsize> {
    NonZeroUsize::new(DEFAULT_SEND_BATCH_SIZE)
}

fn default_send_batch_max_size() -> Option<NonZeroUsize> {
    None
}

fn default_timeout_duration() -> Option<Duration> {
    Some(Duration::from_millis(DEFAULT_TIMEOUT_MS))
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
        if self.send_batch_size.or(self.send_batch_max_size).is_none() {
            return Err(ConfigError::InvalidUserConfig {
                error: "send_batch_max_size or send_batch_size must be set".into(),
            });
        }

        if let Some(max_size) = self.send_batch_max_size {
            if let Some(batch_size) = self.send_batch_size {
                if max_size < batch_size {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "send_batch_max_size ({}) must be >= send_batch_size ({}) or unset",
                            max_size, batch_size,
                        ),
                    });
                }
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
    pending: Vec<OtapArrowRecords>,

    /// A count defined by batch_length(), number of spans, log records, or metric data points.
    items: usize,
}

/// Local (!Send) OTAP batch processor
pub struct OtapBatchProcessor {
    config: Config,
    signals: BatchSignals,
    lower_limit: NonZeroUsize,
    metrics: MetricSet<OtapBatchProcessorMetrics>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FlushReason {
    Size,
    Timer,
    Shutdown,
}

/// Minimal, essential metrics for the OTAP batch processor, plus observability for
/// timer flush decisions.
#[metric_set(name = "otap.processor.batch")]
#[derive(Debug, Default, Clone)]
pub struct OtapBatchProcessorMetrics {
    /// Total items consumed for logs signal
    #[metric(unit = "{item}")]
    consumed_items_logs: Counter<u64>,
    /// Total items consumed for metrics signal
    #[metric(unit = "{item}")]
    consumed_items_metrics: Counter<u64>,
    /// Total items consumed for traces signal
    #[metric(unit = "{item}")]
    consumed_items_traces: Counter<u64>,

    /// Number of flushes triggered by size threshold (all signals)
    #[metric(unit = "{flush}")]
    flushes_size: Counter<u64>,
    /// Number of flushes triggered by timer (all signals)
    #[metric(unit = "{flush}")]
    flushes_timer: Counter<u64>,
    /// Number of flushes triggered by shutdown (all signals)
    #[metric(unit = "{flush}")]
    flushes_shutdown: Counter<u64>,

    /// Number of messages dropped due to conversion failures
    #[metric(unit = "{msg}")]
    dropped_conversion: Counter<u64>,
    /// Number of batching errors encountered
    #[metric(unit = "{error}")]
    batching_errors: Counter<u64>,
    /// Number of empty records dropped
    #[metric(unit = "{msg}")]
    dropped_empty_records: Counter<u64>,

    /// Number of split requests issued to upstream batching
    #[metric(unit = "{event}")]
    split_requests: Counter<u64>,

    /// Timer-triggered flushes that were performed for logs
    #[metric(unit = "{flush}")]
    timer_flush_performed_logs: Counter<u64>,
    /// Timer-triggered flushes that were performed for metrics
    #[metric(unit = "{flush}")]
    timer_flush_performed_metrics: Counter<u64>,
    /// Timer-triggered flushes that were performed for traces
    #[metric(unit = "{flush}")]
    timer_flush_performed_traces: Counter<u64>,

    /// Timer-triggered flushes that were skipped for logs (below threshold)
    #[metric(unit = "{event}")]
    timer_flush_skipped_logs: Counter<u64>,
    /// Timer-triggered flushes that were skipped for metrics (below threshold)
    #[metric(unit = "{event}")]
    timer_flush_skipped_metrics: Counter<u64>,
    /// Timer-triggered flushes that were skipped for traces (below threshold)
    #[metric(unit = "{event}")]
    timer_flush_skipped_traces: Counter<u64>,
}

fn nzu_to_nz64(nz: NonZeroUsize) -> NonZeroU64 {
    NonZeroU64::new(nz.get() as u64).expect("nonzero")
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

impl OtapBatchProcessor {
    /// Parse JSON config and build the processor instance with the provided metrics set.
    /// This function does not wrap the processor into a ProcessorWrapper so callers can
    /// preserve the original NodeUserConfig (including out_ports/default_out_port).
    pub fn build_from_json(
        cfg: &Value,
        metrics: MetricSet<OtapBatchProcessorMetrics>,
    ) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(cfg.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        // This checks that if both are present, max_size >= batch_size, and
        // that at least one is present so that lower_limit is valid below.
        config.validate()?;

        let lower_limit = config
            .send_batch_max_size
            .or(config.send_batch_size)
            .expect("valid");

        Ok(OtapBatchProcessor {
            config,
            signals: BatchSignals::default(),
            lower_limit,
            metrics: metrics,
        })
    }

    /// Backward-compatible helper used by unit tests to construct a processor wrapper
    /// directly from JSON. Note: This creates a fresh NodeUserConfig without out_ports,
    /// which is fine for unit tests that do not rely on engine wiring.
    pub fn from_config(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
        metrics: MetricSet<OtapBatchProcessorMetrics>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let proc = Self::build_from_json(cfg, metrics)?;
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            OTAP_BATCH_PROCESSOR_URN,
        ));
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Flush all per-signal buffers (logs, metrics, traces). Does nothing if all are empty.
    async fn flush_current(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        self.flush_signal_impl(SignalType::Logs, effect, reason)
            .await?;
        self.flush_signal_impl(SignalType::Metrics, effect, reason)
            .await?;
        self.flush_signal_impl(SignalType::Traces, effect, reason)
            .await
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
            SignalType::Logs => self.metrics.consumed_items_logs.add(items as u64),
            SignalType::Metrics => self.metrics.consumed_items_metrics.add(items as u64),
            SignalType::Traces => self.metrics.consumed_items_traces.add(items as u64),
        }

        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };

        eprintln!(
            "[process_signal_impl] {:?}: BEFORE adding - items={}, pending_batches={}",
            signal,
            buffer.items,
            buffer.pending.len()
        );

        buffer.items += items;
        buffer.pending.push(rec);

        eprintln!(
            "[process_signal_impl] {:?}: AFTER adding {} items - total={}, pending_batches={}, lower_limit={}",
            signal,
            items,
            buffer.items,
            buffer.pending.len(),
            self.lower_limit
        );

        let should_flush = buffer.items >= self.lower_limit.get();
        eprintln!(
            "[process_signal_impl] {:?}: threshold check: {} >= {} = {}",
            signal, buffer.items, self.lower_limit, should_flush
        );

        if should_flush {
            eprintln!(
                "[process_signal_impl] {:?}: triggering SIZE flush (items={} >= lower_limit={})",
                signal, buffer.items, self.lower_limit
            );
            self.flush_signal_impl(signal, effect, FlushReason::Size)
                .await
        } else {
            eprintln!(
                "[process_signal_impl] {:?}: buffering (items={} < lower_limit={})",
                signal, buffer.items, self.lower_limit
            );
            Ok(())
        }
    }

    /// Generic flush implementation for any signal type
    async fn flush_signal_impl(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };

        eprintln!(
            "\n=== [flush_signal_impl] {:?}: reason={:?}, items={}, pending_batches={} ===",
            signal,
            reason,
            buffer.items,
            buffer.pending.len()
        );
        eprintln!(
            "    Config: send_batch_size={:?}, send_batch_max_size={:?}, lower_limit={}",
            self.config.send_batch_size, self.config.send_batch_max_size, self.lower_limit
        );

        let initial_items = buffer.items;
        buffer.items = 0;

        let input = std::mem::take(&mut buffer.pending);
        if input.is_empty() {
            eprintln!("    → Empty buffer, nothing to flush");
            return Ok(());
        }

        let total_input_items: usize = input.iter().map(|r| r.batch_length()).sum();
        eprintln!(
            "    Input: {} record batches totaling {} items",
            input.len(),
            total_input_items
        );
        for (idx, rec) in input.iter().enumerate() {
            eprintln!("      Record[{}]: batch_length={}", idx, rec.batch_length());
        }

        match reason {
            FlushReason::Size => self.metrics.flushes_size.inc(),
            FlushReason::Timer => self.metrics.flushes_timer.inc(),
            FlushReason::Shutdown => self.metrics.flushes_shutdown.inc(),
        }

        if let Some(upper_limit) = self.config.send_batch_max_size {
            self.metrics.split_requests.inc();
            eprintln!(
                "    → Path: WITH upper_limit (send_batch_max_size={})",
                upper_limit
            );
            eprintln!("    Calling make_output_batches...");

            let mut output_batches =
                match make_output_batches(signal, input, Some(nzu_to_nz64(upper_limit))) {
                    Ok(v) => v,
                    Err(e) => {
                        self.metrics.batching_errors.inc();
                        eprintln!("    ✗ make_output_batches FAILED: {}", e);
                        log_batching_failed(effect, signal, &e).await;
                        return Err(EngineError::InternalError {
                            message: e.to_string(),
                        });
                    }
                };

            let total_output_items: usize = output_batches.iter().map(|b| b.batch_length()).sum();
            eprintln!(
                "    ✓ make_output_batches returned {} batches totaling {} items",
                output_batches.len(),
                total_output_items
            );
            for (idx, batch) in output_batches.iter().enumerate() {
                eprintln!(
                    "      Output[{}]: batch_length={}",
                    idx,
                    batch.batch_length()
                );
            }

            // If size-triggered and we requested splitting (max is Some), re-buffer the last partial
            // output if it is smaller than the configured lower_limit. Timer/Shutdown flush everything.
            if reason == FlushReason::Size && !output_batches.is_empty() {
                if let Some(last_items) = output_batches.last().map(|last| last.batch_length()) {
                    let threshold = self.lower_limit.get();
                    eprintln!(
                        "    Rebuffer check (reason=Size): last_batch_items={} vs lower_limit={}",
                        last_items, threshold
                    );
                    if last_items < threshold {
                        eprintln!(
                            "      → REBUFFERING last batch ({} items < {} lower_limit)",
                            last_items, threshold
                        );
                        let remainder = output_batches.pop().expect("last exists");
                        buffer.items = last_items;
                        buffer.pending.push(remainder);
                        eprintln!(
                            "      → Buffer state after rebuffer: items={}, pending_batches={}",
                            buffer.items,
                            buffer.pending.len()
                        );
                    } else {
                        eprintln!(
                            "      → NOT rebuffering ({} items >= {} lower_limit)",
                            last_items, threshold
                        );
                    }
                } else {
                    eprintln!("    No last batch to check for rebuffering");
                }
            } else if reason != FlushReason::Size {
                eprintln!(
                    "    Skipping rebuffer check (reason={:?}, flushing everything)",
                    reason
                );
            }

            eprintln!("    → Emitting {} batches", output_batches.len());
            for (idx, records) in output_batches.into_iter().enumerate() {
                let items = records.batch_length();
                eprintln!("      Emitting batch[{}] with {} items", idx, items);
                let pdata = OtapPdata::new_todo_context(records.into());
                effect.send_message(pdata).await?;
            }
        } else {
            eprintln!("    → Path: WITHOUT upper_limit (send_batch_max_size=None)");
            // No split requested (safe path)
            if input.len() > 1 {
                eprintln!("    → Coalescing {} input batches", input.len());
                // Coalesce upstream only when there are multiple records to merge
                let output_batches = match make_output_batches(signal, input, None) {
                    Ok(v) => {
                        eprintln!("      ✓ Coalesced into {} output batches", v.len());
                        v
                    }
                    Err(e) => {
                        self.metrics.batching_errors.inc();
                        eprintln!("      ✗ Coalescing FAILED: {}", e);
                        log_batching_failed(effect, signal, &e).await;
                        Vec::new()
                    }
                };
                eprintln!("    → Emitting {} coalesced batches", output_batches.len());
                for (idx, records) in output_batches.into_iter().enumerate() {
                    let items = records.batch_length();
                    eprintln!("      Emitting batch[{}] with {} items", idx, items);
                    let pdata = OtapPdata::new_todo_context(records.into());
                    effect.send_message(pdata).await?;
                }
            } else {
                // Single record: forward as-is (avoids upstream edge-cases)
                eprintln!("    → Forwarding single input batch as-is (no coalescing)");
                for (idx, records) in input.into_iter().enumerate() {
                    let items = records.batch_length();
                    eprintln!("      Emitting batch[{}] with {} items", idx, items);
                    let pdata = OtapPdata::new_todo_context(records.into());
                    effect.send_message(pdata).await?;
                }
            }
        }

        eprintln!(
            "=== [flush_signal_impl] {:?}: COMPLETE - buffer state: items={}, pending_batches={} ===\n",
            signal,
            buffer.items,
            buffer.pending.len()
        );
        Ok(())
    }
}

/// Factory function to create an OTAP batch processor
pub fn create_otap_batch_processor(
    pipeline_ctx: otap_df_engine::context::PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let metrics = pipeline_ctx.register_metrics::<OtapBatchProcessorMetrics>();
    let proc = OtapBatchProcessor::build_from_json(&node_config.config, metrics)?;
    // IMPORTANT: preserve the original node_config so engine wiring (out_ports/default_out_port)
    // remains intact.
    Ok(ProcessorWrapper::local(
        proc,
        node,
        node_config,
        processor_config,
    ))
}

#[async_trait(?Send)]
impl local::Processor<OtapPdata> for OtapBatchProcessor {
    async fn process(
        &mut self,
        msg: Message<OtapPdata>,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match msg {
            Message::Control(ctrl) => {
                match ctrl {
                    NodeControlMsg::TimerTick { .. } => {
                        // Flush on timer only when thresholds were crossed and buffers are non-empty
                        if !self.signals.logs.pending.is_empty() {
                            if self.signals.logs.items >= self.lower_limit.get() {
                                self.metrics.timer_flush_performed_logs.inc();
                                self.flush_signal_impl(
                                    SignalType::Logs,
                                    effect,
                                    FlushReason::Timer,
                                )
                                .await?;
                            } else {
                                self.metrics.timer_flush_skipped_logs.inc();
                            }
                        }
                        if !self.signals.metrics.pending.is_empty() {
                            if self.signals.metrics.items >= self.lower_limit.get() {
                                self.metrics.timer_flush_performed_metrics.inc();
                                self.flush_signal_impl(
                                    SignalType::Metrics,
                                    effect,
                                    FlushReason::Timer,
                                )
                                .await?;
                            } else {
                                self.metrics.timer_flush_skipped_metrics.inc();
                            }
                        }
                        if !self.signals.traces.pending.is_empty() {
                            if self.signals.traces.items >= self.lower_limit.get() {
                                self.metrics.timer_flush_performed_traces.inc();
                                self.flush_signal_impl(
                                    SignalType::Traces,
                                    effect,
                                    FlushReason::Timer,
                                )
                                .await?;
                            } else {
                                self.metrics.timer_flush_skipped_traces.inc();
                            }
                        }
                        Ok(())
                    }
                    NodeControlMsg::Config { .. } => Ok(()),
                    NodeControlMsg::Shutdown { .. } => {
                        // Flush and shutdown
                        self.flush_current(effect, FlushReason::Shutdown).await?;
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
                    NodeControlMsg::DelayedData { .. } => {
                        unreachable!("unused");
                    }
                    NodeControlMsg::Ack { .. } | NodeControlMsg::Nack { .. } => Ok(()),
                }
            }
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
mod test_helpers {
    use super::*;
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::common::v1::AnyValue;
    use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::status::StatusCode;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData,
    };

    // Test helper constants to avoid magic strings in scope names
    pub(super) fn one_trace_record() -> OtapArrowRecords {
        let traces = TracesData::new(vec![ResourceSpans::new(
            Resource::default(),
            vec![ScopeSpans::new(
                InstrumentationScope::build().name("lib").finish(),
                vec![
                    Span::build()
                        .trace_id(vec![0; 16])
                        .span_id(vec![1; 8])
                        .name("span")
                        .start_time_unix_nano(1u64)
                        .status(Status::new(StatusCode::Ok, "ok"))
                        .finish(),
                ],
            )],
        )]);
        crate::encoder::encode_spans_otap_batch(&traces).expect("encode traces")
    }

    pub(super) fn one_metric_record() -> OtapArrowRecords {
        // Minimal metrics: one Gauge with one NumberDataPoint
        let md = MetricsData::new(vec![ResourceMetrics::new(
            Resource::default(),
            vec![ScopeMetrics::new(
                InstrumentationScope::build().name("lib").finish(),
                vec![
                    Metric::build()
                        .name("g")
                        .data_gauge(Gauge::new(vec![
                            NumberDataPoint::build()
                                .time_unix_nano(2u64)
                                .value_double(1.0)
                                .finish(),
                        ]))
                        .finish(),
                ],
            )],
        )]);
        crate::encoder::encode_metrics_otap_batch(&md).expect("encode metrics")
    }

    pub(super) fn logs_record_with_n_entries(n: usize) -> OtapArrowRecords {
        let logs: Vec<LogRecord> = (0..n)
            .map(|i| {
                LogRecord::build()
                    .time_unix_nano(i as u64)
                    .severity_number(SeverityNumber::Info)
                    .body(AnyValue::new_string(format!("log{i}")))
                    .finish()
            })
            .collect();
        let logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::build().name("lib").finish(),
                logs,
            )],
        )]);
        crate::encoder::encode_logs_otap_batch(&logs_data).expect("encode logs")
    }

    /// Construct a processor wrapper from a JSON configuration object and processor runtime config.
    /// The JSON should mirror the Go collector batchprocessor shape. Missing fields fall back to
    /// crate defaults. Invalid numeric values (e.g., zero) are normalized to minimal valid values.
    #[cfg(test)]
    pub fn from_config(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let handle = otap_df_telemetry::registry::MetricsRegistryHandle::default();
        let metrics: MetricSet<OtapBatchProcessorMetrics> =
            handle.register(otap_df_telemetry::testing::EmptyAttributes());
        OtapBatchProcessor::from_config(node, cfg, proc_cfg, metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::test_helpers::{
        from_config, logs_record_with_n_entries, one_metric_record, one_trace_record,
    };
    use super::*;
    use crate::pdata::{OtapPdata, OtlpProtoBytes};
    use otap_df_config::PipelineGroupId;
    use otap_df_config::PipelineId;
    use otap_df_config::node::{DispatchStrategy, HyperEdgeConfig, NodeKind, NodeUserConfig};
    use otap_df_config::pipeline::PipelineConfig;
    use otap_df_engine::config::ProcessorConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::message::Message;
    use otap_df_engine::node::Node; // bring trait in scope for user_config()
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use serde_json::json;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[test]
    fn test_factory_preserves_node_user_config_ports() {
        // Build a pipeline context to register metrics
        let registry = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            PipelineGroupId::from("g".to_string()),
            PipelineId::from("p".to_string()),
            0,
            0,
        );

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
    // See crates/telemetry-macros/README.md ("Define a metric set"): if a metric field name is not
    // overridden, the field identifier is converted by replacing '_' with '.'.
    // Example: consumed_items_traces => consumed.items.traces
    fn get_metric(map: &HashMap<&'static str, u64>, snake_case: &str) -> u64 {
        let dotted = snake_case.replace('_', ".");
        map.get(dotted.as_str())
            .copied()
            .or_else(|| map.get(snake_case).copied())
            .unwrap_or(0)
    }

    // Test constants to avoid magic numbers/strings
    const TEST_SHUTDOWN_DEADLINE_MS: u64 = 50;
    const TEST_SHUTDOWN_REASON: &str = "test";

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_internal_telemetry_collects_and_reports() {
        let test_rt = TestRuntime::new();
        let registry = test_rt.metrics_registry();
        let reporter = test_rt.metrics_reporter();

        // Create a MetricSet for the batch processor using a PipelineContext bound to the registry
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            PipelineGroupId::from("test-group".to_string()),
            PipelineId::from("test-pipeline".to_string()),
            0,
            0,
        );
        let metrics_set = pipeline_ctx.register_metrics::<OtapBatchProcessorMetrics>();

        // Build processor with metrics injected (no behavior change)
        let cfg = json!({
            "send_batch_size": 1000,
            "send_batch_max_size": 1000,
            "timeout": "200ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_telemetry_test");
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config, metrics_set)
            .expect("proc from config with metrics");

        // Start test runtime and concurrently run metrics collection loop
        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // 1) Process a logs record. Current encoder path yields 0 rows for logs in this scenario,
            // so the processor treats it as empty and increments dropped_empty_records.
            // TODO(telemetry-logs-rows): Once otel-arrow-rust encodes non-empty logs batches (or
            // OtapArrowRecords::batch_length handles logs), switch assertions to consumed_items_logs.
            let pdata_logs: OtapPdata =
                OtapPdata::new_default(logs_record_with_n_entries(3).into());
            ctx.process(Message::PData(pdata_logs))
                .await
                .expect("process logs");

            // 2) Process a non-empty traces record to exercise positive row counting
            let pdata_traces: OtapPdata = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata_traces))
                .await
                .expect("process traces");
            // Sanity: confirm the trace record itself has rows so telemetry should account for it.
            {
                let rec = one_trace_record();
                assert!(
                    rec.batch_length() >= 1,
                    "trace record must have >=1 rows for telemetry accounting"
                );
            }

            // 3) Send a timer tick -> should not flush (skipped), but increments timer_flush_skipped_*
            ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                .await
                .expect("timer tick");

            // 4) Trigger telemetry collection (report + reset)
            ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }))
            .await
            .expect("collect telemetry");

            // Let collector accumulate snapshot and flush again to minimize timing races.
            ctx.sleep(Duration::from_millis(10)).await;
            ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }))
            .await
            .expect("collect telemetry (2)");
            ctx.sleep(Duration::from_millis(10)).await;
            // 5) One more collection to ensure snapshots hit the collector
            ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            }))
            .await
            .expect("collect telemetry (3)");
            ctx.sleep(Duration::from_millis(30)).await;
        });

        // Validate aggregated metrics are present and sensible
        validation.validate(|_vctx| async move {

            // Poll the registry until the collector has accumulated the snapshot
            let mut map: HashMap<&'static str, u64> = HashMap::new();
            for _ in 0..100 {
                map.clear();
                registry.visit_current_metrics(|desc, _attrs, iter| {
                    if desc.name == "otap.processor.batch" {
                        for (field, val) in iter {
                            let _ = map.insert(field.name, val);
                        }
                    }
                });
                // Wait until traces are observed (positive path), logs may still be seen as empty.
                if get_metric(&map, "consumed_items_traces") >= 1 {
                    break;
                }
                sleep(Duration::from_millis(10)).await;
            }

            // Diagnostic: collect all metric sets and fields found (non-zero only)
            let mut sets: std::collections::BTreeMap<&'static str, Vec<(&'static str, u64)>> =
                Default::default();
            registry.visit_current_metrics(|desc, _attrs, iter| {
                let entry = sets.entry(desc.name).or_default();
                for (field, val) in iter {
                    entry.push((field.name, val));
                }
            });

            // Positive path: traces row counter observed
            let traces_rows = get_metric(&map, "consumed_items_traces");
            if traces_rows < 1 {
                eprintln!("[diag] no consumed_items_traces yet. sets observed: {sets:?}");
            }
            assert!(traces_rows >= 1);

            // Logs path (current): likely dropped as empty; timer flush should be skipped
            assert_eq!(get_metric(&map, "flushes_timer"), 0);
            assert!(get_metric(&map, "timer_flush_skipped_logs") >= 1);
            // Logs may either be processed (rows counted) or dropped as empty depending on encoder/test data.
            let logs_rows = get_metric(&map, "consumed_items_logs");
            let dropped_empty = get_metric(&map, "dropped_empty_records");
            assert!(
                logs_rows >= 1 || dropped_empty >= 1,
                "expected either logs rows or dropped-empty; got logs_rows={logs_rows}, dropped_empty={dropped_empty}",
            );
        });
    }

    #[test]
    fn test_factory_creation() {
        let cfg = json!({"send_batch_size": 1000, "timeout": "100ms"});
        let processor_config = ProcessorConfig::new("otap_batch_test");
        let node = test_node(processor_config.name.clone());
        let result = from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeout_go_style_string() {
        let cfg = json!({
            "send_batch_size": 3,
            "timeout": "200ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test3");
        let node = test_node(processor_config.name.clone());
        let result = from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_flush_before_append_when_exceeding_max() {
        let cfg = json!({
            "send_batch_size": 2, // match max to isolate max-boundary behavior
            "send_batch_max_size": 2,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max1");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        // run scenario
        let validation = phase.run_test(|mut ctx| async move {
            let pdata1 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata1))
                .await
                .expect("process 1");
            let pdata2 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata2))
                .await
                .expect("process 2");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 1, "flush expected once at max boundary");

            let pdata3 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata3))
                .await
                .expect("process 3");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                0,
                "no flush expected after third until shutdown"
            );

            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Instant::now().add(Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS)),
                reason: TEST_SHUTDOWN_REASON.into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "remaining buffered item should flush on shutdown"
            );
        });

        // no additional validation phase assertions needed
        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_timer_does_not_flush_if_below_threshold() {
        let cfg = json!({
            "send_batch_size": 1000, // large so count threshold won't trigger
            "send_batch_max_size": 1000,
            "timeout": "200ms" // timer value is not used directly in test; we send TimerTick manually
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_timer_flush");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Process a single small record (below thresholds)
            let pdata = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata)).await.expect("process 1");

            // No flush before timer
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "no flush expected before timer");

            // Send a timer tick -> should NOT flush because thresholds were not crossed
            ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                .await
                .expect("timer tick");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                0,
                "no flush expected on timer when thresholds not crossed"
            );
        });

        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_immediate_flush_on_max_reached() {
        let cfg = json!({
            "send_batch_size": 1,
            "send_batch_max_size": 1, // reaching max on first push triggers immediate flush-after-push
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max2");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata)).await.expect("process 1");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "single item should flush immediately when max=1"
            );

            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Instant::now().add(Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS)),
                reason: TEST_SHUTDOWN_REASON.into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert!(
                emitted.is_empty(),
                "no additional items expected on shutdown"
            );
        });

        validation.validate(|_vctx| async move {});
    }
    #[test]
    fn test_max_unset_ok() {
        // Null means unlimited
        let cfg = json!({
            "send_batch_size": 7,
            "send_batch_max_size": Value::Null,
            "timeout": "200ms"
        });
        let proc_cfg = ProcessorConfig::new("norm-max");
        let node = test_node(proc_cfg.name.clone());
        let res = from_config(node.clone(), &cfg, &proc_cfg);
        assert!(res.is_ok());

        // Missing max means unlimited
        let cfg = json!({
            "send_batch_size": 9,
            "timeout": "200ms"
        });
        let res = from_config(node, &cfg, &proc_cfg);
        assert!(res.is_ok());
    }

    #[test]
    fn test_drop_non_convertible_metrics_bytes() {
        let cfg = json!({
            "send_batch_size": 1,
            "send_batch_max_size": 10,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_non_convertible");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Metrics OTLP bytes are not yet supported for conversion -> should be dropped
            let pdata =
                OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(vec![1, 2, 3]).into());
            ctx.process(Message::PData(pdata)).await.expect("process 1");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                0,
                "non-convertible inputs should not be forwarded"
            );
        });
        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_non_convertible_metrics_bytes_dropped_on_shutdown() {
        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 10,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_on_shutdown");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata =
                OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(vec![9, 9, 9]).into());
            ctx.process(Message::PData(pdata)).await.expect("process");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "no flush before shutdown");

            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Instant::now().add(Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS)),
                reason: TEST_SHUTDOWN_REASON.into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "non-convertible inputs should be dropped");
        });
        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_split_for_single_oversize_record() {
        // Configure: oversize single record relative to limits; batcher should split it
        let cfg = json!({
            "send_batch_size": 3,
            "send_batch_max_size": 3,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_split_oversize");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Build a single logs record with 5 rows
            let rec = logs_record_with_n_entries(5);
            assert_eq!(rec.batch_length(), 5, "test precondition: 5 rows");

            // Process it
            let pdata = OtapPdata::new_default(rec.into());
            ctx.process(Message::PData(pdata)).await.expect("process");

            // Batcher should split: 5 rows with max_size=3 → first batch has 3 rows, remainder has 2
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "first split batch (3 rows) should be emitted immediately"
            );
            let first = emitted.into_iter().next().unwrap().payload();
            let first_rec: OtapArrowRecords = first.try_into().unwrap();
            assert_eq!(
                first_rec.batch_length(),
                3,
                "first batch should have max_size rows"
            );

            // Shutdown should flush the remainder (2 rows below threshold)
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Instant::now().add(Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS)),
                reason: TEST_SHUTDOWN_REASON.into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "remainder (2 rows) should be flushed on shutdown"
            );
            let remainder = emitted.into_iter().next().unwrap().payload();
            let remainder_rec: OtapArrowRecords = remainder.try_into().unwrap();
            assert_eq!(
                remainder_rec.batch_length(),
                2,
                "remainder should have 2 rows"
            );
        });

        validation.validate(|_vctx| async move {});
    }

    // TODO: Add a positive-path test that simulates a size-triggered split leaving
    // a small remainder in the buffer, and assert that a subsequent TimerTick
    // flushes that remainder (dirty flag remains set). Crafting this requires
    // multiple buffered records so requested_split_max returns Some(max) and the
    // upstream splitter produces >1 outputs. We can enable such a test after
    // stabilizing upstream batching behavior or by constructing a scenario with
    // multiple small OtapArrowRecords that collectively exceed send_batch_max_size.

    #[test]
    fn test_timer_tick_does_not_flush_after_size_flush_when_no_remainder() {
        // Configure size == max so size-triggered flush fully drains (no split/remainder).
        let cfg = json!({
            "send_batch_size": 2,
            "send_batch_max_size": 2,
            "timeout": "50ms"
        });
        let processor_config = ProcessorConfig::new("otap_timer_no_remainder");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // First push: 1 row -> below thresholds, nothing emitted
            let pdata1 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata1)).await.expect("p1");
            assert!(ctx.drain_pdata().await.is_empty());

            // Second push: another 1 row -> rows=2 triggers size flush, fully drained (no remainder)
            let pdata2 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata2)).await.expect("p2");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "size flush should emit one batch and drain"
            );

            // Third push: 1 row, below thresholds -> timer should NOT flush (dirty cleared on size flush)
            let pdata3 = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata3)).await.expect("p3");
            assert!(ctx.drain_pdata().await.is_empty());

            ctx.process(Message::Control(NodeControlMsg::TimerTick {}))
                .await
                .expect("timer");
            let emitted2 = ctx.drain_pdata().await;
            assert_eq!(
                emitted2.len(),
                0,
                "timer should not flush when no remainder and no new threshold crossing"
            );
        });

        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_metrics_batching_with_split() {
        // Test that metrics can be properly batched and split
        let cfg = json!({
            "send_batch_size": 2,
            "send_batch_max_size": 2,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_metrics_test");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = from_config(node, &cfg, &processor_config).expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Process two metric records to trigger flush
            let pdata1 = OtapPdata::new_default(one_metric_record().into());
            ctx.process(Message::PData(pdata1))
                .await
                .expect("process metric 1");

            let pdata2 = OtapPdata::new_default(one_metric_record().into());
            ctx.process(Message::PData(pdata2))
                .await
                .expect("process metric 2");

            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 1, "metrics should flush at threshold");

            // Verify the batched output is metrics
            let first = emitted.into_iter().next().unwrap().payload();
            let first_rec: OtapArrowRecords = first.try_into().unwrap();
            assert!(matches!(first_rec, OtapArrowRecords::Metrics(_)));
            assert_eq!(2, first_rec.batch_length());
        });

        validation.validate(|_vctx| async move {});
    }

    #[test]
    fn test_batch_with_out_port() {
        let id = PipelineId::from("batch-with-out-port".to_string());
        let group_id = PipelineGroupId::from("batch".to_string());
        let pipeline = PipelineConfig::from_yaml(
            group_id.clone(),
            id.clone(),
            r#"settings:
  default_pipeline_ctrl_msg_channel_size: 100
  default_node_ctrl_msg_channel_size: 100
  default_pdata_channel_size: 100

nodes:
  receiver:
    kind: receiver
    plugin_urn: "urn:otel:otap:fake_data_generator:receiver"
    out_ports:
      out_port:
        destinations:
          - proc
        dispatch_strategy: round_robin
    config:
      traffic_config:
        max_batch_size: 1000
        signals_per_second: 1000
        log_weight: 100
      registry_path: https://github.com/open-telemetry/semantic-conventions.git[model]

  proc:
    kind: processor
    plugin_urn: "urn:otap:processor:batch"
    out_ports:
      out_port:
        destinations:
          - exporter
        dispatch_strategy: round_robin
    config: {}

  exporter:
    kind: exporter
    plugin_urn: "urn:otel:otap:perf:exporter"
    config:
      frequency: 1000
      cpu_usage: false
      mem_usage: false
      disk_usage: false
      io_usage: false
"#,
        )
        .unwrap();

        let metrics = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics);
        let pipeline_ctx = controller_ctx.pipeline_context_with(group_id, id, 0, 0);

        let runtime = super::super::OTAP_PIPELINE_FACTORY
            .build(pipeline_ctx, pipeline)
            .unwrap();

        assert!(runtime.node_count() > 0, "pipeline should contain nodes");

        // Verify presence of batch processor
        let mut found = false;
        for i in 0..runtime.node_count() {
            if let Some(node) = runtime.get_node(i) {
                let uc = node.user_config();
                if uc.kind == NodeKind::Processor
                    && uc.plugin_urn.as_ref() == "urn:otap:processor:batch"
                {
                    found = true;
                    break;
                }
            }
        }
        assert!(
            found,
            "expected a batch processor node (urn:otap:processor:batch)"
        );
    }
}
