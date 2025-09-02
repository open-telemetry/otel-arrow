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
// For optional conversion during flush/partitioning
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::otap::batching::make_output_batches;

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otap:processor:batch";

/// Default configuration values (parity-aligned as we confirm Go defaults)
pub const DEFAULT_SEND_BATCH_SIZE: usize = 8192;
/// Default upper bound on batch size used to chunk oversized inputs (in number of items).
/// See Config::send_batch_max_size for normalization semantics.
pub const DEFAULT_SEND_BATCH_MAX_SIZE: usize = 8192;
/// Timeout in milliseconds for periodic flush
pub const DEFAULT_TIMEOUT_MS: u64 = 200;

/// Semantic constants (avoid magic numbers)
/// Minimum allowed send_batch_size
pub const MIN_SEND_BATCH_SIZE: usize = 1;
/// Sentinel meaning: follow send_batch_size for max size (Go parity)
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
const LOG_MSG_BATCHING_FAILED_PREFIX: &str =
    "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// Signal label constants (avoid repeated string literals)
const SIG_LOGS: &str = "logs";
const SIG_METRICS: &str = "metrics";
const SIG_TRACES: &str = "traces";

/// Configuration for the OTAP batch processor (parity with Go batchprocessor)
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Flush current batch when this count is reached.
    #[serde(default = "default_send_batch_size")]
    pub send_batch_size: usize,
    /// Hard cap for splitting very large inputs.
    /// Go behavior: 0 (or missing) => use send_batch_size; we use 0 as default and normalize later
    #[serde(default = "default_send_batch_max_size")]
    pub send_batch_max_size: usize,
    /// Flush non-empty batches on this interval (milliseconds).
    #[serde(default = "default_timeout_ms")]
    pub timeout: u64,
    /// Optional metadata partitioning keys (resource/scope/attribute names)
    #[serde(default)]
    pub metadata_keys: Vec<String>,
    /// Optional limit on the number of distinct metadata-based groups this processor will track.
    ///
    /// Note: This is currently a no-op because grouping by metadata_keys has not yet been
    /// implemented. When grouping lands, this will cap the number of concurrent groups and the
    /// overflow strategy will be documented.
    #[serde(default)]
    pub metadata_cardinality_limit: Option<usize>,
}

fn default_send_batch_size() -> usize {
    DEFAULT_SEND_BATCH_SIZE
}

fn default_send_batch_max_size() -> usize {
    FOLLOW_SEND_BATCH_SIZE_SENTINEL // normalized in from_config
}

fn default_timeout_ms() -> u64 {
    DEFAULT_TIMEOUT_MS
}

impl Default for Config {
    fn default() -> Self {
        Self {
            send_batch_size: default_send_batch_size(),
            send_batch_max_size: default_send_batch_max_size(),
            timeout: default_timeout_ms(),
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
    // For inputs that cannot be converted to OTAP (currently dropped with a log)
    passthrough: Vec<OtapPdata>,
    // Running row counts per signal for size triggers
    rows_logs: usize,
    rows_metrics: usize,
    rows_traces: usize,
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
        // Allow Go-style timeout strings; parse to milliseconds before deserialization.
        let mut cfg_norm = cfg.clone();
        if let Some(s) = cfg.get("timeout").and_then(|v| v.as_str()) {
            if let Some(ms) = parse_duration_ms(s) {
                if let Some(obj) = cfg_norm.as_object_mut() {
                    let _ = obj.insert("timeout".to_string(), Value::from(ms));
                }
            }
        }

        let mut config: Config =
            serde_json::from_value(cfg_norm).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        // Basic validation/normalization
        if config.send_batch_size == 0 {
            config.send_batch_size = MIN_SEND_BATCH_SIZE;
        }
        // Normalize 0 -> send_batch_size
        let effective_sbs = config.send_batch_size;
        let max = if config.send_batch_max_size == FOLLOW_SEND_BATCH_SIZE_SENTINEL {
            effective_sbs
        } else {
            config.send_batch_max_size
        };
        config.send_batch_max_size = max;

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
            passthrough: Vec::new(),
            rows_logs: 0,
            rows_metrics: 0,
            rows_traces: 0,
        };
        Ok(ProcessorWrapper::local(proc, node, user_config, proc_cfg))
    }

    /// Returns true if any per-signal buffered rows reach or exceed the emission threshold.
    /// Uses actual row counts from OTAP records.
    fn size_triggers_emission(&self) -> bool {
        let threshold = self.config.send_batch_size;
        self.rows_logs >= threshold
            || self.rows_metrics >= threshold
            || self.rows_traces >= threshold
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
        if self.current_logs.is_empty()
            && !self
                .passthrough
                .iter()
                .any(|p| p.signal_type() == SignalType::Logs)
        {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_logs);
        if !input.is_empty() {
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                // Bypass upstream splitter for degenerate max; just forward each record
                for records in input {
                    let pdata: OtapPdata = records.into();
                    effect.send_message(pdata).await?;
                }
                // Always reset counter in the degenerate path
                self.rows_logs = 0;
            } else {
                // Avoid upstream splitter where unnecessary or unsafe (see requested_split_max)
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
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
                        let pdata: OtapPdata = records.into();
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_logs = 0;
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                log_batching_failed(effect, SIG_LOGS, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is (avoids upstream edge-cases)
                        for records in input {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_logs = 0;
                }
            }
        }
        // Also flush any passthrough items for this signal type
        if !self.passthrough.is_empty() {
            let mut remaining = Vec::with_capacity(self.passthrough.len());
            for pdata in self.passthrough.drain(..) {
                if pdata.signal_type() == SignalType::Logs {
                    effect.send_message(pdata).await?;
                } else {
                    remaining.push(pdata);
                }
            }
            self.passthrough = remaining;
        }
        Ok(())
    }

    async fn flush_metrics(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        if self.current_metrics.is_empty()
            && !self
                .passthrough
                .iter()
                .any(|p| p.signal_type() == SignalType::Metrics)
        {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_metrics);
        if !input.is_empty() {
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                for records in input {
                    let pdata: OtapPdata = records.into();
                    effect.send_message(pdata).await?;
                }
                self.rows_metrics = 0;
            } else {
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
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
                        let pdata: OtapPdata = records.into();
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_metrics = 0;
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                log_batching_failed(effect, SIG_METRICS, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is
                        for records in input {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_metrics = 0;
                }
            }
        }
        if !self.passthrough.is_empty() {
            let mut remaining = Vec::with_capacity(self.passthrough.len());
            for pdata in self.passthrough.drain(..) {
                if pdata.signal_type() == SignalType::Metrics {
                    effect.send_message(pdata).await?;
                } else {
                    remaining.push(pdata);
                }
            }
            self.passthrough = remaining;
        }
        Ok(())
    }

    async fn flush_traces(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        if self.current_traces.is_empty()
            && !self
                .passthrough
                .iter()
                .any(|p| p.signal_type() == SignalType::Traces)
        {
            return Ok(());
        }
        let input = std::mem::take(&mut self.current_traces);
        if !input.is_empty() {
            let max_val = self.config.send_batch_max_size;
            if max_val <= MIN_SEND_BATCH_SIZE {
                for records in input {
                    let pdata: OtapPdata = records.into();
                    effect.send_message(pdata).await?;
                }
                self.rows_traces = 0;
            } else {
                let max = requested_split_max(&input, max_val);
                if let Some(max_nz) = max {
                    let mut output_batches = match make_output_batches(Some(max_nz), input) {
                        Ok(v) => v,
                        Err(e) => {
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
                        let pdata: OtapPdata = records.into();
                        effect.send_message(pdata).await?;
                    }

                    if !rebuffered {
                        self.rows_traces = 0;
                    }
                } else {
                    // No split requested (safe path)
                    if input.len() > 1 {
                        // Coalesce upstream only when there are multiple records to merge
                        let output_batches = match make_output_batches(None, input) {
                            Ok(v) => v,
                            Err(e) => {
                                log_batching_failed(effect, SIG_TRACES, &e).await;
                                Vec::new()
                            }
                        };
                        for records in output_batches {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    } else {
                        // Single record: forward as-is
                        for records in input {
                            let pdata: OtapPdata = records.into();
                            effect.send_message(pdata).await?;
                        }
                    }
                    self.rows_traces = 0;
                }
            }
        }
        if !self.passthrough.is_empty() {
            let mut remaining = Vec::with_capacity(self.passthrough.len());
            for pdata in self.passthrough.drain(..) {
                if pdata.signal_type() == SignalType::Traces {
                    effect.send_message(pdata).await?;
                } else {
                    remaining.push(pdata);
                }
            }
            self.passthrough = remaining;
        }
        Ok(())
    }
}

/// Factory function to create an OTAP batch processor
pub fn create_otap_batch_processor(
    _pipeline_ctx: otap_df_engine::context::PipelineContext,
    node: NodeId,
    config: &Value,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    OtapBatchProcessor::from_config(node, config, processor_config)
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
                        // Follow OTLP semantics: only flush on timer if a size threshold has been reached
                        if self.size_triggers_emission() {
                            self.flush_current(effect, FlushReason::Timer).await
                        } else {
                            Ok(())
                        }
                    }
                    otap_df_engine::control::NodeControlMsg::Config { .. } => Ok(()),
                    otap_df_engine::control::NodeControlMsg::Shutdown { .. } => {
                        // Flush and shutdown
                        self.flush_current(effect, FlushReason::Shutdown).await?;
                        effect.info(LOG_MSG_SHUTTING_DOWN).await;
                        Ok(())
                    }
                    otap_df_engine::control::NodeControlMsg::Ack { .. }
                    | otap_df_engine::control::NodeControlMsg::Nack { .. }
                    | otap_df_engine::control::NodeControlMsg::CollectTelemetry { .. } => Ok(()),
                }
            }
            Message::PData(data) => {
                let max = self.config.send_batch_max_size;
                match OtapArrowRecords::try_from(data.clone()) {
                    Ok(rec) => {
                        let rows = rec.batch_length();
                        // Per-signal policy: drop zero-row; pre-flush if exceeding max; append rows; flush on max or send_batch_size.
                        match &rec {
OtapArrowRecords::Logs(_) => {
                                if rows == 0 {
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_logs + rows > max
                                    {
                                        self.flush_logs(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_logs += rows;
                                    self.current_logs.push(rec);
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_logs >= max)
                                        || self.rows_logs >= self.config.send_batch_size
                                    {
                                        self.flush_logs(effect, FlushReason::Size).await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
OtapArrowRecords::Metrics(_) => {
                                if rows == 0 {
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_metrics + rows > max
                                    {
                                        self.flush_metrics(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_metrics += rows;
                                    self.current_metrics.push(rec);
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_metrics >= max)
                                        || self.rows_metrics >= self.config.send_batch_size
                                    {
                                        self.flush_metrics(effect, FlushReason::Size).await
                                    } else {
                                        Ok(())
                                    }
                                }
                            }
OtapArrowRecords::Traces(_) => {
                                if rows == 0 {
                                    effect.info(LOG_MSG_DROP_EMPTY).await;
                                    Ok(())
                                } else {
                                    if max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_traces + rows > max
                                    {
                                        self.flush_traces(effect, FlushReason::Size).await?;
                                    }
                                    self.rows_traces += rows;
                                    self.current_traces.push(rec);
                                    if (max > FOLLOW_SEND_BATCH_SIZE_SENTINEL
                                        && self.rows_traces >= max)
                                        || self.rows_traces >= self.config.send_batch_size
                                    {
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
                        match data.signal_type() {
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
                 cfg: &Value,
                 proc_cfg: &ProcessorConfig| {
            create_otap_batch_processor(pipeline_ctx, node, cfg, proc_cfg)
        },
    };

/// Parses duration strings using Go-like syntax (e.g., "200ms", "2s", "1m", "1m30s").
/// Returns milliseconds. Bare numbers are NOT accepted here to mirror Go's time.ParseDuration.
fn parse_duration_ms(s: &str) -> Option<u64> {
    // humantime supports inputs like "200ms", "2s", "1m", "1h", but not scientific exponents
    humantime::parse_duration(s)
        .ok()
        .map(|d| d.as_millis() as u64)
}

#[cfg(test)]
mod test_helpers {
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::common::v1::InstrumentationScope;

    // Test helper constants to avoid magic strings in scope names
    const TEST_SCOPE_NAME: &str = "lib";
    use otel_arrow_rust::proto::opentelemetry::metrics::v1::{
        Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::status::StatusCode;
    use otel_arrow_rust::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, Status, TracesData,
    };
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
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

    pub(super) fn traces_record_with_n_spans(n: usize) -> OtapArrowRecords {
        let spans: Vec<Span> = (0..n)
            .map(|i| {
                Span::build(vec![i as u8; 16], vec![i as u8; 8], format!("span{i}"), 1u64)
                    .status(Status::new("ok", StatusCode::Ok))
                    .finish()
            })
            .collect();
        let traces = TracesData::new(vec![
            ResourceSpans::build(Resource::default())
                .scope_spans(vec![
                    ScopeSpans::build(InstrumentationScope::new(TEST_SCOPE_NAME))
                        .spans(spans)
                        .finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_spans_otap_batch(&traces).expect("encode traces")
    }

    pub(super) fn logs_record_with_n_entries(n: usize) -> OtapArrowRecords {
        let logs: Vec<LogRecord> = (0..n)
            .map(|i| {
                LogRecord::build(i as u64, SeverityNumber::Info, format!("log{i}"))
                    .finish()
            })
            .collect();
        let logs_data = LogsData::new(vec![
            ResourceLogs::build(Resource::default())
                .scope_logs(vec![
ScopeLogs::build(InstrumentationScope::new(TEST_SCOPE_NAME)).log_records(logs).finish(),
                ])
                .finish(),
        ]);
        crate::encoder::encode_logs_otap_batch(&logs_data).expect("encode logs")
    }
}

#[cfg(test)]
mod tests {
use super::test_helpers::{one_trace_record, logs_record_with_n_entries};
    use super::*;
    use otap_df_engine::testing::test_node;
    use serde_json::json;
    use otel_arrow_rust::otap::OtapArrowRecords;

    // Test constants to avoid magic numbers/strings
    const TEST_SHUTDOWN_DEADLINE_MS: u64 = 50;
    const TEST_SHUTDOWN_REASON: &str = "test";

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_factory_creation() {
        let cfg = json!({"send_batch_size": 1000, "timeout": 100});
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
            "timeout": 250,
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
            "timeout": 250,
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
            "send_batch_size": 10, // keep large so count threshold doesn't trigger
            "send_batch_max_size": 2,
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max1");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        // run scenario
        let validation = phase.run_test(|mut ctx| async move {
            let pdata1: OtapPdata = one_trace_record().into();
            ctx.process(Message::PData(pdata1))
                .await
                .expect("process 1");
            let pdata2: OtapPdata = one_trace_record().into();
            ctx.process(Message::PData(pdata2))
                .await
                .expect("process 2");
            let emitted = ctx.drain_pdata().await;
            assert_eq!(emitted.len(), 1, "flush expected once at max boundary");

            let pdata3: OtapPdata = one_trace_record().into();
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
    fn test_immediate_flush_on_max_reached() {
        use crate::pdata::OtapPdata;
        use otap_df_engine::message::Message;
        use otap_df_engine::testing::processor::TestRuntime;

        let cfg = json!({
            "send_batch_size": 10,
            "send_batch_max_size": 1, // reaching max on first push triggers immediate flush-after-push
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_max2");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata: OtapPdata = one_trace_record().into();
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
    fn test_max_defaults_to_size_when_zero_or_missing() {
        let cfg = json!({
            "send_batch_size": 7,
            "send_batch_max_size": 0,
            "timeout": "200ms"
        });
        let proc_cfg = ProcessorConfig::new("norm-max");
        let node = test_node(proc_cfg.name.clone());
        let res = OtapBatchProcessor::from_config(node.clone(), &cfg, &proc_cfg);
        assert!(res.is_ok());

        // Missing max -> also defaults to size
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
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_non_convertible");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            // Metrics OTLP bytes are not yet supported for conversion -> should be dropped
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportMetricsRequest(vec![1, 2, 3]));
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
            "timeout": 10
        });
        let processor_config = ProcessorConfig::new("otap_batch_test_drop_on_shutdown");
        let test_rt = TestRuntime::new();
        let node = test_node(processor_config.name.clone());
        let proc = OtapBatchProcessor::from_config(node, &cfg, &processor_config)
            .expect("proc from config");

        let phase = test_rt.set_processor(proc);

        let validation = phase.run_test(|mut ctx| async move {
            let pdata = OtapPdata::from(OtlpProtoBytes::ExportMetricsRequest(vec![9, 9, 9]));
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

        // Configure: small max to force splitting, large send_batch_size to avoid count-based flushes
        let cfg = json!({
            "send_batch_size": 1000,
            "send_batch_max_size": 3,
            "timeout": 10
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
            let pdata: OtapPdata = rec.into();
            ctx.process(Message::PData(pdata)).await.expect("process");

            // With the guard, a single oversize record should NOT be split; we forward as-is.
            let emitted = ctx.drain_pdata().await;
            assert_eq!(
                emitted.len(),
                1,
                "single oversize record should be forwarded unsplit due to guard"
            );
            let first = emitted.into_iter().next().unwrap();
            let first_rec: OtapArrowRecords = first.try_into().unwrap();
            assert_eq!(first_rec.batch_length(), 5, "guard prevents splitting a single record");

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
            assert_eq!(emitted.len(), 0, "no remainder expected on shutdown with guard");
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
    const TEST_BATCHING_SMOKE_EXPECTED_LOGS_PANIC_NOTE: &str = "Upstream otel-arrow-rust batching/grouping occasionally panics (empty-group/unreachable in groups.rs) for this scenario. Re-enable after upstream fix.";

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
