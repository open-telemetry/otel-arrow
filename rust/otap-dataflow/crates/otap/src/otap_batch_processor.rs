// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor.
//! Batches OtapPdata by count or timer; uses upstream OTAP batching for merge/split.

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ProcessorConfig;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_engine::message::Message;
use otap_df_engine::node::NodeId;
use otap_df_engine::processor::ProcessorWrapper;
use serde::Deserialize;
use serde_json::Value;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::Duration;
// Telemetry metrics
pub mod metrics;
use crate::otap_batch_processor::metrics::OtapBatchProcessorMetrics;
use otap_df_telemetry::metrics::MetricSet;
// For optional conversion during flush/partitioning
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::otap::batching::make_output_batches;

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otap:processor:batch";

/// Default configuration values (parity-aligned as we confirm Go defaults)
pub const DEFAULT_SEND_BATCH_SIZE: usize = 8192;
/// Default upper bound on batch size used to chunk oversized inputs (in number of items).
/// See Config::send_batch_max_size for normalization semantics.
pub const DEFAULT_SEND_BATCH_MAX_SIZE: usize = 0; // 0 means unlimited (OTLP parity)
/// Timeout in milliseconds for periodic flush
pub const DEFAULT_TIMEOUT_MS: u64 = 200;

/// Semantic constants (avoid magic numbers)
/// Minimum allowed send_batch_size
pub const MIN_SEND_BATCH_SIZE: usize = 1;
/// Sentinel meaning: no upper limit for max size (Go parity)
pub const FOLLOW_SEND_BATCH_SIZE_SENTINEL: usize = 0;
/// Minimum allowed metadata cardinality limit when specified
pub const MIN_METADATA_CARDINALITY_LIMIT: usize = 1;
/// Minimum increment to apply when counting items for size calculations
pub const MIN_ITEM_INCREMENT: usize = 1;

/// Log messages
const LOG_MSG_SHUTTING_DOWN: &str = "OTAP batch processor shutting down";
const LOG_MSG_DROP_EMPTY: &str = "OTAP batch processor: dropping empty OTAP record (0 rows)";
const LOG_MSG_DROP_CONVERSION_FAILED: &str =
    "OTAP batch processor: dropping message: OTAP conversion failed";
const LOG_MSG_BATCHING_FAILED_PREFIX: &str = "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// Signal label constants (avoid repeated string literals)
const SIG_LOGS: &str = "logs";
const SIG_METRICS: &str = "metrics";
const SIG_TRACES: &str = "traces";

/// Configuration for the OTAP batch processor (parity with Go batchprocessor)
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Flush current batch when this count is reached.
    /// When set to 0, the batch size is ignored and data will be sent immediately
    /// subject only to send_batch_max_size (OTLP parity).
    #[serde(default = "default_send_batch_size_opt")]
    pub send_batch_size: Option<usize>,
    /// Hard cap for splitting very large inputs.
    /// Go behavior: 0 means no upper limit; missing defaults to 0 (unlimited).
    #[serde(default = "default_send_batch_max_size")]
    pub send_batch_max_size: usize,
    /// Flush non-empty batches on this interval.
    #[serde(with = "humantime_serde", default = "default_timeout_duration_opt")]
    pub timeout: Option<Duration>,
    /// Optional metadata partitioning keys (resource/scope/attribute names). Not yet supported.
    /// ToDo: Support metadata-aware batching.
    #[serde(default)]
    pub metadata_keys: Vec<String>,
    /// Optional limit on the number of distinct metadata-based groups this processor will track.
    ///
    /// Note: This is currently a no-op because grouping by metadata_keys has not yet been
    /// implemented. When grouping lands, this will cap the number of concurrent groups and the
    /// overflow strategy will be documented. Not yet supported.
    /// ToDo: Support metadata-aware batching
    #[serde(default)]
    pub metadata_cardinality_limit: Option<usize>,
}

fn default_send_batch_size_opt() -> Option<usize> {
    Some(DEFAULT_SEND_BATCH_SIZE)
}

fn default_send_batch_max_size() -> usize {
    FOLLOW_SEND_BATCH_SIZE_SENTINEL // normalized in from_config
}

fn default_timeout_duration_opt() -> Option<Duration> {
    Some(Duration::from_millis(DEFAULT_TIMEOUT_MS))
}

impl Default for Config {
    fn default() -> Self {
        Self {
            send_batch_size: default_send_batch_size_opt(),
            send_batch_max_size: default_send_batch_max_size(),
            timeout: default_timeout_duration_opt(),
            metadata_keys: Vec::new(),
            metadata_cardinality_limit: None,
        }
    }
}

/// Local (!Send) OTAP batch processor
pub struct OtapBatchProcessor {
    config: Config,
    // Per-signal buffers of convertible OTAP records
    current_logs: Vec<OtapArrowRecords>,
    current_metrics: Vec<OtapArrowRecords>,
    current_traces: Vec<OtapArrowRecords>,
    // Running row counts per signal for size triggers
    rows_logs: usize,
    rows_metrics: usize,
    rows_traces: usize,
    // Tracks whether a size/max threshold was crossed since the last input for each signal.
    // Used to decide whether a timer tick should flush that signal (OTLP parity semantics).
    dirty_logs: bool,
    dirty_metrics: bool,
    dirty_traces: bool,
    // Internal telemetry (optional)
    metrics: Option<MetricSet<OtapBatchProcessorMetrics>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FlushReason {
    Size,
    Timer,
    Shutdown,
}

/// Decide whether to request splitting from upstream batching.
/// Returns Some(max) only when a split is needed and safe, otherwise None.
fn requested_split_max(input: &[OtapArrowRecords], max_val: usize) -> Option<NonZeroU64> {
    // Workaround: avoid splitting a single record until upstream bug is fixed.
    // Also avoid requesting split when total rows fit within max.
    let total_rows: usize = input.iter().map(|r| r.batch_length()).sum();
    if total_rows <= max_val || input.len() <= 1 {
        None
    } else {
        NonZeroU64::new(max_val as u64)
    }
}

async fn log_batching_failed(
    effect: &mut local::EffectHandler<OtapPdata>,
    signal: &str,
    err: &impl std::fmt::Display,
) {
    effect
        .info(&format!(
            "{LOG_MSG_BATCHING_FAILED_PREFIX} {signal}: {err}{LOG_MSG_BATCHING_FAILED_SUFFIX}"
        ))
        .await;
}

impl OtapBatchProcessor {
    /// Construct a processor wrapper from a JSON configuration object and processor runtime config.
    /// The JSON should mirror the Go collector batchprocessor shape. Missing fields fall back to
    /// crate defaults. Invalid numeric values (e.g., zero) are normalized to minimal valid values.
    pub fn from_config(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut config: Config =
            serde_json::from_value(cfg.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        // Basic validation/normalization
        // Allow send_batch_size = 0 to mean: ignore size threshold (send immediately),
        // subject only to send_batch_max_size when non-zero (Go parity).
        // Keep send_batch_max_size = 0 as unlimited (no upper bound).
        // Validate: when both are non-zero, max must be >= size.
        if config.send_batch_max_size != 0 {
            if let Some(s) = config.send_batch_size {
                if s != 0 && config.send_batch_max_size < s {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "invalid OTAP batch processor config: send_batch_max_size ({}) must be >= send_batch_size ({}) or 0 (unlimited)",
                            config.send_batch_max_size, s
                        ),
                    });
                }
            }
        }

        if let Some(limit) = config.metadata_cardinality_limit {
            if limit < MIN_METADATA_CARDINALITY_LIMIT {
                config.metadata_cardinality_limit = Some(MIN_METADATA_CARDINALITY_LIMIT);
            }
        }
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            OTAP_BATCH_PROCESSOR_URN,
        ));
        let proc = OtapBatchProcessor {
            config,
            current_logs: Vec::new(),
            current_metrics: Vec::new(),
            current_traces: Vec::new(),
            rows_logs: 0,
            rows_metrics: 0,
            rows_traces: 0,
            dirty_logs: false,
            dirty_metrics: false,
            dirty_traces: false,
            metrics: None,
        };
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Construct a processor wrapper from a JSON configuration object and processor runtime config,
    /// with optional metrics for internal telemetry. Functional behavior is unchanged.
    pub fn from_config_with_metrics(
        node: NodeId,
        cfg: &Value,
        proc_cfg: &ProcessorConfig,
        metrics: Option<MetricSet<OtapBatchProcessorMetrics>>,
    ) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
        let mut config: Config =
            serde_json::from_value(cfg.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        if config.send_batch_max_size != 0 {
            if let Some(s) = config.send_batch_size {
                if s != 0 && config.send_batch_max_size < s {
                    return Err(ConfigError::InvalidUserConfig {
                        error: format!(
                            "invalid OTAP batch processor config: send_batch_max_size ({}) must be >= send_batch_size ({}) or 0 (unlimited)",
                            config.send_batch_max_size, s
                        ),
                    });
                }
            }
        }

        if let Some(limit) = config.metadata_cardinality_limit {
            if limit < MIN_METADATA_CARDINALITY_LIMIT {
                config.metadata_cardinality_limit = Some(MIN_METADATA_CARDINALITY_LIMIT);
            }
        }
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            OTAP_BATCH_PROCESSOR_URN,
        ));
        let proc = OtapBatchProcessor {
            config,
            current_logs: Vec::new(),
            current_metrics: Vec::new(),
            current_traces: Vec::new(),
            rows_logs: 0,
            rows_metrics: 0,
            rows_traces: 0,
            dirty_logs: false,
            dirty_metrics: false,
            dirty_traces: false,
            metrics,
        };
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Flush all per-signal buffers (logs, metrics, traces). Does nothing if all are empty.
    async fn flush_current(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        self.flush_logs(effect, reason).await?;
        self.flush_metrics(effect, reason).await?;
        self.flush_traces(effect, reason).await
    }

    async fn flush_logs(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        // Only return early if there is nothing buffered for this signal at all
        if self.current_logs.is_empty() {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_logs);
        if !input.is_empty() {
            if let Some(metrics) = &mut self.metrics {
                match reason {
                    FlushReason::Size => metrics.flushes_size.inc(),
                    FlushReason::Timer => metrics.flushes_timer.inc(),
                    FlushReason::Shutdown => metrics.flushes_shutdown.inc(),
                }
            }
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                // Bypass upstream splitter for degenerate max; just forward each record
                for records in input {
                    let pdata = OtapPdata::new_todo_context(records.into());
                    effect.send_message(pdata).await?;
                }
                // Always reset counter in the degenerate path
                self.rows_logs = 0;
                if reason == FlushReason::Size {
                    // Fully drained; no remainder -> not eligible for timer-based flush
                    self.dirty_logs = false;
                    if let Some(metrics) = &mut self.metrics {
                        metrics.dirty_cleared_logs.inc();
                    }
                }
            } else {
                // Avoid upstream splitter where unnecessary or unsafe (see requested_split_max)
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    if let Some(metrics) = &mut self.metrics {
                        metrics.split_requests.inc();
                    }
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.batching_errors.inc();
                            }
                            log_batching_failed(effect, SIG_LOGS, &e).await;
                            Vec::new()
                        }
                    };

                    // If size-triggered and we requested splitting (max is Some), re-buffer the last partial
                    // output if it is smaller than the configured max. Timer/Shutdown flush everything.
                    let mut rebuffered = false;
                    if reason == FlushReason::Size && !output_batches.is_empty() {
                        if let Some(last) = output_batches.last() {
                            let last_rows = last.batch_length();
                            if last_rows < max_val {
                                let remainder = output_batches.pop().expect("last exists");
                                self.rows_logs = last_rows;
                                self.current_logs.push(remainder);
                                rebuffered = true;
                            }
                        }
                    }

                    for records in output_batches {
                        let pdata = OtapPdata::new_todo_context(records.into());
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_logs = 0;
                    }
                    if reason == FlushReason::Size {
                        // Set timer eligibility based on whether a remainder was kept
                        self.dirty_logs = rebuffered;
                        if !rebuffered {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_logs.inc();
                            }
                        }
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                if let Some(metrics) = &mut self.metrics {
                                    metrics.batching_errors.inc();
                                }
                                log_batching_failed(effect, SIG_LOGS, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is (avoids upstream edge-cases)
                        for records in input {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_logs = 0;
                    if reason == FlushReason::Size {
                        // Fully drained in no-split path
                        self.dirty_logs = false;
                    }
                }
            }
        }
        Ok(())
    }

    async fn flush_metrics(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        if self.current_metrics.is_empty() {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_metrics);
        if !input.is_empty() {
            if let Some(metrics) = &mut self.metrics {
                match reason {
                    FlushReason::Size => metrics.flushes_size.inc(),
                    FlushReason::Timer => metrics.flushes_timer.inc(),
                    FlushReason::Shutdown => metrics.flushes_shutdown.inc(),
                }
            }
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                for records in input {
                    let pdata = OtapPdata::new_todo_context(records.into());
                    effect.send_message(pdata).await?;
                }
                self.rows_metrics = 0;
                if reason == FlushReason::Size {
                    self.dirty_metrics = false;
                }
            } else {
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    if let Some(metrics) = &mut self.metrics {
                        metrics.split_requests.inc();
                    }
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.batching_errors.inc();
                            }
                            log_batching_failed(effect, SIG_METRICS, &e).await;
                            Vec::new()
                        }
                    };

                    let mut rebuffered = false;
                    if reason == FlushReason::Size && !output_batches.is_empty() {
                        if let Some(last) = output_batches.last() {
                            let last_rows = last.batch_length();
                            if last_rows < max_val {
                                let remainder = output_batches.pop().expect("last exists");
                                self.rows_metrics = last_rows;
                                self.current_metrics.push(remainder);
                                rebuffered = true;
                            }
                        }
                    }

                    for records in output_batches {
                        let pdata = OtapPdata::new_todo_context(records.into());
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_metrics = 0;
                    }
                    if reason == FlushReason::Size {
                        self.dirty_metrics = rebuffered;
                        if !rebuffered {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_metrics.inc();
                            }
                        }
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                if let Some(metrics) = &mut self.metrics {
                                    metrics.batching_errors.inc();
                                }
                                log_batching_failed(effect, SIG_METRICS, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is
                        for records in input {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_metrics = 0;
                    if reason == FlushReason::Size {
                        self.dirty_metrics = false;
                        if let Some(metrics) = &mut self.metrics {
                            metrics.dirty_cleared_metrics.inc();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn flush_traces(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        if self.current_traces.is_empty() {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_traces);
        if !input.is_empty() {
            if let Some(metrics) = &mut self.metrics {
                match reason {
                    FlushReason::Size => metrics.flushes_size.inc(),
                    FlushReason::Timer => metrics.flushes_timer.inc(),
                    FlushReason::Shutdown => metrics.flushes_shutdown.inc(),
                }
            }
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                for records in input {
                    let pdata = OtapPdata::new_todo_context(records.into());
                    effect.send_message(pdata).await?;
                }
                self.rows_traces = 0;
                if reason == FlushReason::Size {
                    self.dirty_traces = false;
                }
            } else {
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    if let Some(metrics) = &mut self.metrics {
                        metrics.split_requests.inc();
                    }
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.batching_errors.inc();
                            }
                            log_batching_failed(effect, SIG_TRACES, &e).await;
                            Vec::new()
                        }
                    };

                    let mut rebuffered = false;
                    if reason == FlushReason::Size && !output_batches.is_empty() {
                        if let Some(last) = output_batches.last() {
                            let last_rows = last.batch_length();
                            if last_rows < max_val {
                                let remainder = output_batches.pop().expect("last exists");
                                self.rows_traces = last_rows;
                                self.current_traces.push(remainder);
                                rebuffered = true;
                            }
                        }
                    }

                    for records in output_batches {
                        let pdata = OtapPdata::new_todo_context(records.into());
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_traces = 0;
                    }
                    if reason == FlushReason::Size {
                        self.dirty_traces = rebuffered;
                        if !rebuffered {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_traces.inc();
                            }
                        }
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                if let Some(metrics) = &mut self.metrics {
                                    metrics.batching_errors.inc();
                                }
                                log_batching_failed(effect, SIG_TRACES, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is
                        for records in input {
                            let pdata = OtapPdata::new_todo_context(records.into());
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_traces = 0;
                    if reason == FlushReason::Size {
                        self.dirty_traces = false;
                        if let Some(metrics) = &mut self.metrics {
                            metrics.dirty_cleared_traces.inc();
                        }
                    }
                }
            }
        }
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
    OtapBatchProcessor::from_config_with_metrics(
        node,
        &node_config.config,
        processor_config,
        Some(metrics),
    )
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
                    otap_df_engine::control::NodeControlMsg::TimerTick { .. } => {
                        // Flush on timer only when thresholds were crossed and buffers are non-empty (unchanged behavior)
                        if self.dirty_logs && !self.current_logs.is_empty() {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.timer_flush_performed_logs.inc();
                            }
                            self.flush_logs(effect, FlushReason::Timer).await?;
                            self.dirty_logs = false;
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_logs.inc();
                            }
                        } else if let Some(metrics) = &mut self.metrics {
                            metrics.timer_flush_skipped_logs.inc();
                        }
                        if self.dirty_metrics && !self.current_metrics.is_empty() {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.timer_flush_performed_metrics.inc();
                            }
                            self.flush_metrics(effect, FlushReason::Timer).await?;
                            self.dirty_metrics = false;
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_metrics.inc();
                            }
                        } else if let Some(metrics) = &mut self.metrics {
                            metrics.timer_flush_skipped_metrics.inc();
                        }
                        if self.dirty_traces && !self.current_traces.is_empty() {
                            if let Some(metrics) = &mut self.metrics {
                                metrics.timer_flush_performed_traces.inc();
                            }
                            self.flush_traces(effect, FlushReason::Timer).await?;
                            self.dirty_traces = false;
                            if let Some(metrics) = &mut self.metrics {
                                metrics.dirty_cleared_traces.inc();
                            }
                        } else if let Some(metrics) = &mut self.metrics {
                            metrics.timer_flush_skipped_traces.inc();
                        }
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::Config { .. } => Ok(()),
                    otap_df_engine::control::NodeControlMsg::Shutdown { .. } => {
                        // Flush and shutdown
                        self.flush_current(effect, FlushReason::Shutdown).await?;
                        effect.info(LOG_MSG_SHUTTING_DOWN).await;
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    } => {
                        if let Some(metrics) = &mut self.metrics {
                            let _ = metrics_reporter.report(metrics);
                        }
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::Ack { .. }
                    | otap_df_engine::control::NodeControlMsg::Nack { .. } => Ok(()),
                }
            }
            Message::PData(request) => {
                let max = self.config.send_batch_max_size;
                let signal_type = request.signal_type();

                // Note: Context is dropped.
                let (_ctx, data) = request.into_parts();

                match OtapArrowRecords::try_from(data) {
                    Ok(rec) => {
                        let rows = rec.batch_length();
                        // Per-signal policy: drop zero-row; pre-flush if exceeding max; append rows; flush on max or send_batch_size.
                        match &rec {
                            OtapArrowRecords::Logs(_) => {
                                if rows == 0 {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.dropped_empty_records.inc();
                                    }
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.consumed_items_logs.add(rows as u64);
                                    }
                                    // Pre-append: flush if the incoming record would exceed max.
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_logs + rows > max
                                    {
                                        // Would exceed max: mark as threshold-crossed and flush current
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_logs.inc();
                                        }
                                        self.dirty_logs = true;
                                        self.flush_logs(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_logs += rows;
                                    self.current_logs.push(rec);
                                    // Post-append: flush when we hit the boundary exactly (>= max),
                                    // and also honor send_batch_size as a trigger (0 => immediate).
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_logs >= max)
                                        || matches!(self.config.send_batch_size, Some(s) if self.rows_logs >= s)
                                        || matches!(self.config.send_batch_size, Some(0))
                                    {
                                        // Threshold crossed: mark dirty and flush by size
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_logs.inc();
                                        }
                                        self.dirty_logs = true;
                                        self.flush_logs(effect, FlushReason::Size).await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
                            OtapArrowRecords::Metrics(_) => {
                                if rows == 0 {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.dropped_empty_records.inc();
                                    }
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.consumed_items_metrics.add(rows as u64);
                                    }
                                    // Pre-append: flush if the incoming record would exceed max.
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_metrics + rows > max
                                    {
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_metrics.inc();
                                        }
                                        self.dirty_metrics = true;
                                        self.flush_metrics(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_metrics += rows;
                                    self.current_metrics.push(rec);
                                    // Post-append: flush on boundary equality (>= max) and
                                    // honor send_batch_size as a trigger (0 => immediate).
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_metrics >= max)
                                        || matches!(self.config.send_batch_size, Some(s) if self.rows_metrics >= s)
                                        || matches!(self.config.send_batch_size, Some(0))
                                    {
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_metrics.inc();
                                        }
                                        self.dirty_metrics = true;
                                        self.flush_metrics(effect, FlushReason::Size).await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
                            OtapArrowRecords::Traces(_) => {
                                if rows == 0 {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.dropped_empty_records.inc();
                                    }
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if let Some(metrics) = &mut self.metrics {
                                        metrics.consumed_items_traces.add(rows as u64);
                                    }
                                    // Pre-append: flush if the incoming record would exceed max.
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_traces + rows > max
                                    {
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_traces.inc();
                                        }
                                        self.dirty_traces = true;
                                        self.flush_traces(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_traces += rows;
                                    self.current_traces.push(rec);
                                    // Post-append: flush on boundary equality (>= max) and
                                    // honor send_batch_size as a trigger (0 => immediate).
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_traces >= max)
                                        || matches!(self.config.send_batch_size, Some(s) if self.rows_traces >= s)
                                        || matches!(self.config.send_batch_size, Some(0))
                                    {
                                        if let Some(metrics) = &mut self.metrics {
                                            metrics.dirty_set_traces.inc();
                                        }
                                        self.dirty_traces = true;
                                        self.flush_traces(effect, FlushReason::Size).await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Conversion failed: log and drop (TODO: Nack)
                        if let Some(metrics) = &mut self.metrics {
                            metrics.dropped_conversion.inc();
                        }
                        match signal_type {
                            SignalType::Logs => {
                                effect.info(LOG_MSG_DROP_CONVERSION_FAILED).await;
                                Ok(())
                            }
                            SignalType::Traces => {
                                effect.info(LOG_MSG_DROP_CONVERSION_FAILED).await;
                                Ok(())
                            }
                            SignalType::Metrics => {
                                effect.info(LOG_MSG_DROP_CONVERSION_FAILED).await;
                                Ok(())
                            }
                        }
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
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::common::v1::InstrumentationScope;

    // Test helper constants to avoid magic strings in scope names
    const TEST_SCOPE_NAME: &str = "lib";
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::status::StatusCode;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData,
    };

    pub(super) fn one_trace_record() -> OtapArrowRecords {
        let traces = TracesData::new(vec![
            ResourceSpans::build(Resource::default())
                .scope_spans(vec![
                    ScopeSpans::build(InstrumentationScope::new(TEST_SCOPE_NAME))
                        .spans(vec![
                            Span::build(vec![0; 16], vec![1; 8], "span", 1u64)
                                .status(Status::new("ok", StatusCode::Ok))
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_spans_otap_batch(&traces).expect("encode traces")
    }

    pub(super) fn one_metric_record() -> OtapArrowRecords {
        // Minimal metrics: one Gauge with one NumberDataPoint
        let md = MetricsData::new(vec![
            ResourceMetrics::build(Resource::default())
                .scope_metrics(vec![
                    ScopeMetrics::build(InstrumentationScope::new(TEST_SCOPE_NAME))
                        .metrics(vec![
                            Metric::build_gauge(
                                "g",
                                Gauge::new(vec![NumberDataPoint::build_double(0u64, 1.0).finish()]),
                            )
                            .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_metrics_otap_batch(&md).expect("encode metrics")
    }

    pub(super) fn logs_record_with_n_entries(n: usize) -> OtapArrowRecords {
        let logs: Vec<LogRecord> = (0..n)
            .map(|i| LogRecord::build(i as u64, SeverityNumber::Info, format!("log{i}")).finish())
            .collect();
        let logs_data = LogsData::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new(TEST_SCOPE_NAME))
                        .log_records(logs)
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_logs_otap_batch(&logs_data).expect("encode logs")
    }
}

#[cfg(test)]
mod tests {
    // Helper to read dotted metric field names via snake_case identifiers in tests.
    // See crates/telemetry-macros/README.md ("Define a metric set"): if a metric field name is not
    // overridden, the field identifier is converted by replacing '_' with '.'.
    // Example: consumed_items_traces => consumed.items.traces
    fn get_metric(map: &std::collections::HashMap<&'static str, u64>, snake_case: &str) -> u64 {
        let dotted = snake_case.replace('_', ".");
        map.get(dotted.as_str())
            .copied()
            .or_else(|| map.get(snake_case).copied())
            .unwrap_or(0)
    }
    use super::test_helpers::{logs_record_with_n_entries, one_trace_record};
    use super::*;
    use crate::otap_batch_processor::metrics::OtapBatchProcessorMetrics;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::processor::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::MetricsSystem;
    use otel_arrow_rust::otap::OtapArrowRecords;
    use serde_json::json;

    // Test constants to avoid magic numbers/strings
    const TEST_SHUTDOWN_DEADLINE_MS: u64 = 50;
    const TEST_SHUTDOWN_REASON: &str = "test";

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_internal_telemetry_collects_and_reports() {
        use serde_json::json;
        use std::time::Duration;

        // Metrics system: provides registry + reporter + collection loop
        let ms = MetricsSystem::default();
        let registry = ms.registry();
        let reporter = ms.reporter();

        // Create a MetricSet for the batch processor using a PipelineContext bound to the registry
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            otap_df_config::PipelineGroupId::from("test-group".to_string()),
            otap_df_config::PipelineId::from("test-pipeline".to_string()),
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
        let proc = OtapBatchProcessor::from_config_with_metrics(
            node,
            &cfg,
            &processor_config,
            Some(metrics_set),
        )
        .expect("proc from config with metrics");

        // Start test runtime and concurrently run metrics collection loop
        let test_rt = TestRuntime::new();
        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Spawn metrics collection loop (detached)
            let _bg = tokio::spawn(ms.run_collection_loop());

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
            use std::collections::HashMap;
            use tokio::time::{Duration, sleep};

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
                eprintln!(
                    "[diag] no consumed_items_traces yet. sets observed: {:?}",
                    sets
                );
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
                "expected either logs rows or dropped-empty; got logs_rows={}, dropped_empty={}",
                logs_rows,
                dropped_empty
            );
        });
    }

    #[test]
    fn test_factory_creation() {
        let cfg = json!({"send_batch_size": 1000, "timeout": "100ms"});
        let processor_config = ProcessorConfig::new("otap_batch_test");
        let node = test_node(processor_config.name.clone());
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_metadata_and_max() {
        let cfg = json!({
            "send_batch_size": 3,
            "send_batch_max_size": 5,
            "timeout": "250ms",
            "metadata_keys": ["service.name", "telemetry.sdk.name"]
        });
        let processor_config = ProcessorConfig::new("otap_batch_test2");
        let node = test_node(processor_config.name.clone());
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
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
        let result = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_with_cardinality_limit() {
        let cfg = json!({
            "send_batch_size": 4,
            "send_batch_max_size": 10,
            "timeout": "250ms",
            "metadata_keys": ["service.name"],
            "metadata_cardinality_limit": 100
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_card");
        let node = test_node(processor_config.name.clone());
        let res = OtapBatchProcessor::from_config(node, &cfg, &processor_config);
        assert!(res.is_ok());
        // Ensure deserialization keeps the value
        let mut parsed: Config = serde_json::from_value(cfg).unwrap();
        assert_eq!(parsed.metadata_cardinality_limit, Some(100));
        // Normalize zero to one
        parsed.metadata_cardinality_limit = Some(0);
        // Simulate normalization by re-running from_config path
        let cfg2 = serde_json::json!({
            "metadata_cardinality_limit": 0
        });
        let proc_cfg = ProcessorConfig::new("norm");
        let node = test_node(proc_cfg.name.clone());
        let wrapper_res = OtapBatchProcessor::from_config(node, &cfg2, &proc_cfg);
        assert!(wrapper_res.is_ok());
    }

    #[test]
    fn test_flush_before_append_when_exceeding_max() {
        use crate::pdata::OtapPdata;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 2, // match max to isolate max-boundary behavior
            "send_batch_max_size": 2,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max1");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

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

            use otap_df_engine::control::NodeControlMsg;
            use std::time::Duration;
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS),
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
        use crate::pdata::OtapPdata;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 1000, // large so count threshold won't trigger
            "send_batch_max_size": 1000,
            "timeout": "200ms" // timer value is not used directly in test; we send TimerTick manually
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_timer_flush");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Process a single small record (below thresholds)
            let pdata = OtapPdata::new_default(one_trace_record().into());
            ctx.process(Message::PData(pdata)).await.expect("process 1");

            // No flush before timer
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "no flush expected before timer");

            // Send a timer tick -> should NOT flush because thresholds were not crossed
            use otap_df_engine::control::NodeControlMsg;
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
        use crate::pdata::OtapPdata;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 1,
            "send_batch_max_size": 1, // reaching max on first push triggers immediate flush-after-push
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max2");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

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

            use otap_df_engine::control::NodeControlMsg;
            use std::time::Duration;
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS),
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
    fn test_max_zero_means_unlimited_and_missing_ok() {
        let cfg = json!({
            "send_batch_size": 7,
            "send_batch_max_size": 0,
            "timeout": "200ms"
        });
        let proc_cfg = ProcessorConfig::new("norm-max");
        let node = test_node(proc_cfg.name.clone());
        let res = OtapBatchProcessor::from_config(node.clone(), &cfg, &proc_cfg);
        assert!(res.is_ok());

        // Missing max -> defaults to unlimited (0)
        let cfg2 = json!({
            "send_batch_size": 9,
            "timeout": "200ms"
        });
        let res2 = OtapBatchProcessor::from_config(node, &cfg2, &proc_cfg);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_drop_non_convertible_metrics_bytes() {
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 1,
            "send_batch_max_size": 10,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_non_convertible");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

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
        use crate::pdata::{OtapPdata, OtlpProtoBytes};
        use otap_df_engine::control::NodeControlMsg;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;
        use std::time::Duration;

        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 10,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_on_shutdown");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata =
                OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(vec![9, 9, 9]).into());
            ctx.process(Message::PData(pdata)).await.expect("process");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 0, "no flush before shutdown");

            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS),
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
    fn test_no_split_for_single_oversize_record_guard() {
        use crate::pdata::OtapPdata;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        // Configure: oversize single record relative to limits; guard prevents splitting
        let cfg = json!({
            "send_batch_size": 3,
            "send_batch_max_size": 3,
            "timeout": "10ms"
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_no_split_guard");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Build a single logs record with 5 rows
            let rec = logs_record_with_n_entries(5);
            assert_eq!(rec.batch_length(), 5, "test precondition: 5 rows");

            // Process it
            let pdata = OtapPdata::new_default(rec.into());
            ctx.process(Message::PData(pdata)).await.expect("process");

            // With the guard, a single oversize record should NOT be split; we forward as-is.
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "single oversize record should be forwarded unsplit due to guard"
            );
            let first = emitted.into_iter().next().unwrap().payload();
            let first_rec: OtapArrowRecords = first.try_into().unwrap();
            assert_eq!(
                first_rec.batch_length(),
                5,
                "guard prevents splitting a single record"
            );

            // Shutdown should not produce any additional items (nothing buffered)
            use otap_df_engine::control::NodeControlMsg;
            use std::time::Duration;
            ctx.process(Message::Control(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(TEST_SHUTDOWN_DEADLINE_MS),
                reason: TEST_SHUTDOWN_REASON.into(),
            }))
            .await
            .expect("shutdown");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                0,
                "no remainder expected on shutdown with guard"
            );
        });

        validation.validate(|_vctx| async move {});
    }
}

#[cfg(test)]
mod batching_smoke_tests {
    use super::test_helpers::{one_metric_record, one_trace_record};
    use super::*;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

    // Test constants for batching smoke tests

    #[test]
    #[ignore = "upstream-batching-bug"]
    // NOTE: Validates cross-signal partitioning and expected row totals.
    fn test_make_output_batches_partitions_and_splits() {
        // Build mixed input: 3 traces (1 row each), 2 metrics (1 dp each), interleaved
        let input = vec![
            one_trace_record(),
            one_metric_record(),
            one_trace_record(),
            one_metric_record(),
            one_trace_record(),
        ];

        // Request no split to validate partitioning only
        let outputs = make_output_batches(None, input).expect("batching ok");

        // Expect 2 outputs: one metrics (2 rows), one traces (3 rows)
        let mut metrics_batches = 0usize;
        let mut traces_batches = 0usize;
        let mut total_metrics_rows = 0usize;
        let mut total_traces_rows = 0usize;

        for out in &outputs {
            match out {
                OtapArrowRecords::Metrics(_) => {
                    metrics_batches += 1;
                    let rb = out
                        .get(ArrowPayloadType::UnivariateMetrics)
                        .expect("metrics rb");
                    assert!(rb.num_rows() > 0, "metrics batch must be non-empty");
                    total_metrics_rows += rb.num_rows();
                }
                OtapArrowRecords::Traces(_) => {
                    traces_batches += 1;
                    let rb = out.get(ArrowPayloadType::Spans).expect("spans rb");
                    assert!(rb.num_rows() > 0, "traces batch must be non-empty");
                    total_traces_rows += rb.num_rows();
                }
                OtapArrowRecords::Logs(_) => {
                    panic!("unexpected logs batch in outputs");
                }
            }
        }

        assert_eq!(metrics_batches, 1, "expected one metrics output");
        assert_eq!(traces_batches, 1, "expected one traces output");
        assert_eq!(total_metrics_rows, 2, "expected two metric rows total");
        assert_eq!(total_traces_rows, 3, "expected three trace rows total");
    }
}

#[cfg(test)]
mod timer_flush_behavior_tests {
    use super::*;
    use otap_df_engine::message::Message;
    use otap_df_engine::testing::processor::TestRuntime;
    use serde_json::json;

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
        let node = otap_df_engine::testing::test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // First push: 1 row -> below thresholds, nothing emitted
            let pdata1 = OtapPdata::new_default(test_helpers::one_trace_record().into());
            ctx.process(Message::PData(pdata1)).await.expect("p1");
            assert!(ctx.drain_pdata().await.is_empty());

            // Second push: another 1 row -> rows=2 triggers size flush, fully drained (no remainder)
            let pdata2 = OtapPdata::new_default(test_helpers::one_trace_record().into());
            ctx.process(Message::PData(pdata2)).await.expect("p2");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "size flush should emit one batch and drain"
            );

            // Third push: 1 row, below thresholds -> timer should NOT flush (dirty cleared on size flush)
            let pdata3 = OtapPdata::new_default(test_helpers::one_trace_record().into());
            ctx.process(Message::PData(pdata3)).await.expect("p3");
            assert!(ctx.drain_pdata().await.is_empty());

            use otap_df_engine::control::NodeControlMsg;
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
}
