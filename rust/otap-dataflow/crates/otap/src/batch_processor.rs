// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTAP batch processor.  Batches OtapPdata by item count or timer,
//! uses the lower-level otap_df_pdata::otap::groups module for
//! merging and splitting batches.
//!
//! Configuration is modelled on the (original) OpenTelemetry batch
//! processor, not the relatively-new exporterhelper batcher, which
//! supports batching by a sizer "bytes", "requests", or "items".
//!
//! There are two limits, a lower bound and an upper bound. Both are
//! optional, the user can set one or the other or both. This
//! component uses only the lower of the two numbers, along with the
//! timeout.  Whether it reaches the lower bound or the timeout first,
//! it will flush pending data. If the upper bound is set, merging and
//! splitting will take place, otherwise only merging is performed.
//!
//! When this component flushes because it has reached a limit, and
//! splitting is configured, it means there can be residual data left
//! after flushing. Retained data is always "first in line" for
//! considering in the next flush event. Note that the lower-level
//! function in otap_df_pdata::otap::groups is required to support
//! "in-line" batching (see that component for the definition).
//!
//! This component should be installed before any retry processor
//! (i.e., only retry after batching). This component does not support
//! Interests::RETURN_DATA because (a) more memory required, (b) forces
//! whole-request retry (instead of partial).

use crate::OTAP_PROCESSOR_FACTORIES;
use crate::accessory::slots::{Key as SlotKey, State as SlotState};
use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::Bytes;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::{
    ConsumerEffectHandlerExtension, Interests, ProducerEffectHandlerExtension,
    config::ProcessorConfig,
    control::{AckMsg, CallData, NackMsg, NodeControlMsg},
    error::{Error as EngineError, ProcessorErrorKind},
    local::processor as local,
    message::Message,
    node::NodeId,
    processor::ProcessorWrapper,
};
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
const LOG_MSG_DROP_CONVERSION_FAILED: &str =
    "OTAP batch processor: dropping message: OTAP conversion failed";
const LOG_MSG_BATCHING_FAILED_PREFIX: &str = "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// Batch processor configuration.
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
struct SignalBatches {
    logs: SignalBuffer,
    metrics: SignalBuffer,
    traces: SignalBuffer,
}

/// Per-input wait context, including the arriving request's context.
struct BatchContext {
    /// Original request context.
    ctx: Context,
    /// Number of outbounds
    outbound: usize,
}

/// Portion of input wait context
struct BatchPortion {
    /// The number of these matches Signalbuffer.inbound[inkey]
    inkey: Option<SlotKey>,
    /// Number of items
    items: usize,
}

#[derive(Default)]
struct Inputs {
    /// Input batches.
    pending: Vec<OtapArrowRecords>,

    /// Waiter context
    context: Vec<BatchPortion>,

    /// A count defined by num_items(), number of spans, log records, or metric data points.
    items: usize,
}

struct MultiContext {
    inputs: Vec<BatchPortion>,
    pos: usize,
}

/// Per-signal buffer state
struct SignalBuffer {
    /// Pending input state.
    inputs: Inputs,

    /// Map of inbound requests.  This contains a limited number of pending request
    /// contexts with details for the impl to notify after an outcome is available.
    inbound: SlotState<BatchContext>,

    /// Map of outbound requests.  This contains the assignments from each input
    /// batch, each a corresponding (maybe partial) inbound context.
    outbound: SlotState<Vec<BatchPortion>>,

    /// Arrival time of the oldest data. This is reset whenever the number in the
    /// pending Inputs becomes non-empty.
    arrival: Option<Instant>,
}

/// Local (!Send) batch processor
pub struct BatchProcessor {
    config: Config,
    signals: SignalBatches,
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

        let signals = SignalBatches::new(&config);
        let lower_limit = config
            .send_batch_size
            .or(config.send_batch_max_size)
            .expect("valid");

        Ok(BatchProcessor {
            config,
            signals,
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

    /// Process one incoming batch. Immediately acks empty requests.
    /// If this input causes pending data to exceed the lower bound, it will
    /// flush at least one output.
    async fn process_signal_impl(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        ctx: Context,
        rec: OtapArrowRecords,
    ) -> Result<(), EngineError> {
        let items = rec.num_items();

        if items == 0 {
            self.metrics.dropped_empty_records.inc();
            effect
                .notify_ack(AckMsg::new(OtapPdata::new(ctx, rec.into())))
                .await?;
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

        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };

        // If there are subscribers, calculate an inbound slot key.
        let inkey = ctx
            .has_subscribers()
            .then(|| {
                buffer
                    .inbound
                    .allocate(|| {
                        (
                            BatchContext { ctx, outbound: 0 },
                            (), // not used
                        )
                    })
                    .ok_or_else(|| EngineError::ProcessorError {
                        processor: effect.processor_id(),
                        kind: ProcessorErrorKind::Other,
                        error: "inbound slots not available".into(),
                        source_detail: "".into(),
                    })
            })
            .transpose()?
            .map(|(bc, _)| bc);

        // Set the arrival time when the current input is empty.
        let timeout = self.config.timeout;
        let mut arrival: Option<Instant> = None;
        if timeout != Duration::ZERO && buffer.inputs.is_empty() {
            let now = Instant::now();
            arrival = Some(now);
            buffer.set_arrival(signal, now, timeout, effect).await?;
        }

        buffer.inputs.accept(rec, BatchPortion::new(inkey, items));

        // Flush based on size when the batch reaches the lower limit.
        if self.config.timeout != Duration::ZERO && buffer.inputs.items < self.lower_limit.get() {
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

    /// Flushes all of the input, merging and splitting as necessary to
    /// respect the optional limit. Reason-specific behavior:
    ///
    /// - If Timer and it is arriving early, ignore. This happens because
    ///   timers are not canceled.
    /// - If Size and the final output is smaller than the lower bound, it
    ///   will be retained as first-in-line.
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
        if buffer.inputs.is_empty() {
            return Ok(());
        }
        // If this is a timer-based flush and we were called too soon,
        // skip. this may happen if the batch for which the timer was set
        // flushes for size before the timer.
        if reason == FlushReason::Timer
            && self.config.timeout != Duration::ZERO
            && now.duration_since(buffer.arrival.expect("timed")) < self.config.timeout
        {
            return Ok(());
        }

        let mut inputs = buffer.inputs.drain();

        match reason {
            FlushReason::Size => self.metrics.flushes_size.inc(),
            FlushReason::Timer => self.metrics.flushes_timer.inc(),
            FlushReason::Shutdown => {}
        }

        let count = inputs.requests();
        let upper_limit = self.config.send_batch_max_size;
        let pending = inputs.take_pending();
        let mut output_batches =
            match make_output_batches(signal, nzu_to_nz64(upper_limit), pending) {
                Ok(v) => v,
                Err(e) => {
                    self.metrics.batching_errors.add(count as u64);
                    log_batching_failed(effect, signal, &e).await;
                    let str = e.to_string();
                    let res = Err(str.clone());
                    // In this case, we are sending failure to all the pending inputs.
                    buffer
                        .handle_partial_responses(signal, effect, &res, inputs.context)
                        .await?;
                    return Err(EngineError::InternalError { message: str });
                }
            };

        // If size-triggered and we requested splitting (upper_limit is Some), re-buffer the last partial
        // output if it is smaller than the configured lower_limit. Timer/Shutdown flush everything.
        if self.config.timeout != Duration::ZERO
            && reason == FlushReason::Size
            && upper_limit.is_some()
            && output_batches.len() > 1
        {
            debug_assert!(output_batches[0].num_items() >= self.lower_limit.get());

            if let Some(last_items) = output_batches.last().map(|last| last.num_items()) {
                if last_items < self.lower_limit.get() {
                    buffer.take_remaining(&mut inputs, &mut output_batches, last_items);

                    // We use the latest arrival time as the new arrival for timeout purposes.
                    buffer
                        .set_arrival(signal, now, self.config.timeout, effect)
                        .await?;
                }
            }
        }

        let mut input_context = inputs.take_context();

        for records in output_batches {
            let items = records.num_items();
            let mut pdata = OtapPdata::new(Context::default(), records.into());

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

            // If any items require notification, get an outbound slot and subscribe.
            if let Some(ctxs) = buffer.drain_context(items, &mut input_context) {
                let (outkey, _notused) =
                    buffer.outbound.allocate(|| (ctxs, ())).ok_or_else(|| {
                        EngineError::ProcessorError {
                            processor: effect.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error: "outbound slots not available".into(),
                            source_detail: "".into(),
                        }
                    })?;

                effect.subscribe_to(
                    Interests::NACKS | Interests::ACKS,
                    outkey.into(),
                    &mut pdata,
                );
            }
            effect.send_message(pdata).await?;
        }

        Ok(())
    }

    async fn handle_ack(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        ack: AckMsg<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.handle_response(ack.accepted.signal_type(), ack.calldata, effect, &Ok(()))
            .await
    }

    async fn handle_nack(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        nack: NackMsg<OtapPdata>,
    ) -> Result<(), EngineError> {
        let res = Err(nack.reason);
        self.handle_response(nack.refused.signal_type(), nack.calldata, effect, &res)
            .await
    }

    async fn handle_response(
        &mut self,
        signal: SignalType,
        calldata: CallData,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), String>,
    ) -> Result<(), EngineError> {
        if calldata.is_empty() {
            return Ok(());
        }
        let buffer = match signal {
            SignalType::Logs => &mut self.signals.logs,
            SignalType::Metrics => &mut self.signals.metrics,
            SignalType::Traces => &mut self.signals.traces,
        };
        let outkey: SlotKey = calldata.try_into()?;

        if let Some(parts) = buffer.outbound.take(outkey) {
            buffer
                .handle_partial_responses(signal, effect, res, parts)
                .await?;
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
                    Ok(())
                }
                NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                } => metrics_reporter.report(&mut self.metrics).map_err(|e| {
                    EngineError::InternalError {
                        message: e.to_string(),
                    }
                }),
                NodeControlMsg::DelayedData { data, when } => {
                    let signal = data.signal_type();
                    self.flush_signal_impl(signal, effect, when, FlushReason::Timer)
                        .await?;
                    Ok(())
                }
                NodeControlMsg::Ack(ack) => self.handle_ack(effect, ack).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(effect, nack).await,
                NodeControlMsg::TimerTick { .. } => unreachable!(),
            },
            Message::PData(mut request) => {
                let signal_type = request.signal_type();
                let payload = request.take_payload();

                match OtapArrowRecords::try_from(payload) {
                    Ok(rec) => {
                        let (ctx, _) = request.into_parts();
                        self.process_signal_impl(signal_type, effect, ctx, rec)
                            .await
                    }
                    Err(e) => {
                        // Conversion failed, drop the data. Count, Nack, and Log.
                        self.metrics.dropped_conversion.inc();
                        effect
                            .notify_nack(NackMsg::new(e.to_string(), request))
                            .await?;
                        effect.info(LOG_MSG_DROP_CONVERSION_FAILED).await;
                        Ok(())
                    }
                }
            }
        }
    }
}

impl SignalBatches {
    fn new(config: &Config) -> Self {
        Self {
            logs: SignalBuffer::new(config),
            traces: SignalBuffer::new(config),
            metrics: SignalBuffer::new(config),
        }
    }
}

impl BatchPortion {
    fn new(inkey: Option<SlotKey>, items: usize) -> Self {
        Self { inkey, items }
    }
}

impl MultiContext {
    fn new(inputs: Vec<BatchPortion>) -> Self {
        Self { inputs, pos: 0 }
    }
}

impl Inputs {
    fn drain(&mut self) -> Self {
        Self {
            pending: self.pending.drain(..).collect(),
            context: self.context.drain(..).collect(),
            items: std::mem::take(&mut self.items),
        }
    }

    fn is_empty(&self) -> bool {
        self.items == 0
    }

    fn requests(&self) -> usize {
        self.pending.len()
    }

    fn accept(&mut self, batch: OtapArrowRecords, part: BatchPortion) {
        self.items += part.items;
        self.pending.push(batch);
        self.context.push(part);
    }

    fn take_pending(&mut self) -> Vec<OtapArrowRecords> {
        std::mem::take(&mut self.pending)
    }

    fn take_context(&mut self) -> MultiContext {
        MultiContext::new(std::mem::take(&mut self.context))
    }
}

impl SignalBuffer {
    fn new(_config: &Config) -> Self {
        // TODO configure limits
        Self {
            inputs: Inputs::default(),
            inbound: SlotState::new(1000),
            outbound: SlotState::new(1000),
            arrival: None,
        }
    }

    /// Takes the residual batch, used in case the final output is less than
    /// the lower bound. This removes the last output btach, the corresponding
    /// context, and places it back in the pending buffer as the first in line.
    fn take_remaining(
        &mut self,
        from_inputs: &mut Inputs,
        output_batches: &mut Vec<OtapArrowRecords>,
        last_items: usize,
    ) {
        // SAFETY: protected by output_batches.len() > 1.
        let remaining = output_batches.pop().expect("has last");
        let last_input = from_inputs.context.last().expect("has last");
        let new_part = BatchPortion::new(last_input.inkey, last_items);

        from_inputs.items -= last_items;

        self.inputs.accept(remaining, new_part);
    }

    /// Using a multi-context corresponding with the input pending
    /// data, and considering an output item count for a single output
    /// batch, this determines the set of (maybe partial) pending
    /// batches that correspond. When merging only (not splitting),
    /// this will return the entire set of pending contexts; when
    /// splitting, this will return all except the portion that was
    /// retained as first-in-line.
    fn drain_context(
        &mut self,
        mut items: usize,
        contexts: &mut MultiContext,
    ) -> Option<Vec<BatchPortion>> {
        let mut out = Vec::new();

        while items > 0 && contexts.pos < contexts.inputs.len() {
            let bp = contexts.inputs.get_mut(contexts.pos).expect("valid");

            let take = bp.items.min(items);
            bp.items -= take;
            items -= take;

            if bp.items == 0 {
                contexts.pos += 1;
            }

            if let Some(inkey) = bp.inkey
                && let Some(batch) = self.inbound.get_mut(inkey)
            {
                batch.outbound += 1;

                out.push(BatchPortion::new(Some(inkey), take));
            }
        }

        (!out.is_empty()).then_some(out)
    }

    /// Handles a response, returning an Ack or Nack conditionally when
    /// the outcome is known. This implementation will return as soon as the
    /// first error is received, or it will return when every outbound context
    /// succeeds.
    async fn handle_partial_responses(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), String>,
        parts: Vec<BatchPortion>,
    ) -> Result<(), EngineError> {
        for part in parts {
            if let Some(inkey) = part.inkey {
                let removed = self.inbound.mutate(inkey, |batch| {
                    if res.is_err() {
                        false
                    } else if batch.outbound > 1 {
                        batch.outbound -= 1;
                        true
                    } else {
                        false
                    }
                });
                if let Some(mut batch) = removed {
                    let rdata =
                        OtapPdata::new(std::mem::take(&mut batch.ctx), OtapPayload::empty(signal));
                    if let Err(err) = res {
                        effect.notify_nack(NackMsg::new(err, rdata)).await?;
                    } else {
                        effect.notify_ack(AckMsg::new(rdata)).await?;
                    }
                }
            }
        }

        Ok(())
    }

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
    use crate::testing::TestCallData;
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
    use otap_df_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otap_df_pdata::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, TracesData,
    };
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

    /// Test event: either process input or deliver pending timers
    enum TestEvent {
        Input(OtlpProtoMessage),
        Elapsed, // Signal to deliver all pending DelayedData messages
    }

    /// Policy for acking or nacking an output
    enum AckPolicy {
        Ack,
        Nack(&'static str),
    }

    /// Outputs collected at a specific event position
    #[derive(Clone)]
    struct EventOutputs {
        /// Outputs that arrived during this event
        outputs: Vec<OtapPdata>,
    }

    impl EventOutputs {
        fn messages(&self) -> Vec<OtlpProtoMessage> {
            (0..self.outputs.len()).map(|i| self.message(i)).collect()
        }

        fn message(&self, i: usize) -> OtlpProtoMessage {
            self.outputs
                .get(i)
                .map(|d| {
                    let payload: OtlpProtoBytes = d.clone().payload().try_into().expect("ok");
                    payload.try_into().expect("ok")
                })
                .expect("ok")
        }
    }

    fn run_batch_processor_test<F, P>(
        events: impl Iterator<Item = TestEvent>,
        subscribe: bool,
        config: Value,
        nack_policy: Option<P>,
        verify_outputs: F,
    ) where
        F: FnOnce(&[EventOutputs]) + Send + 'static,
        P: Fn(usize, &OtapPdata) -> AckPolicy + Send + 'static,
    {
        let (metrics_registry, metrics_reporter, phase) = setup_test_runtime(config);

        // Collect events and extract inputs for reference
        let events: Vec<TestEvent> = events.collect();
        let inputs_otlp: Vec<OtlpProtoMessage> = events
            .iter()
            .filter_map(|e| match e {
                TestEvent::Input(msg) => Some(msg.clone()),
                TestEvent::Elapsed => None,
            })
            .collect();

        let signal = inputs_otlp[0].signal_type();
        let input_item_count: usize = inputs_otlp.iter().map(|m| m.num_items()).sum();
        let num_inputs = inputs_otlp.len();
        let total_events = events.len();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                // Track outputs by event position
                let mut event_outputs: Vec<EventOutputs> = vec![
                    EventOutputs {
                        outputs: Vec::new()
                    };
                    total_events
                ];
                let mut received_acks: Vec<TestCallData> = Vec::new();
                let mut received_nacks: Vec<TestCallData> = Vec::new();

                // Track latest DelayedData message
                let mut pending_delay: Option<(Instant, Box<OtapPdata>)> = None;
                let mut input_idx = 0;
                let mut total_outputs = 0;

                // Process each event in sequence
                for (event_idx, event) in events.into_iter().enumerate() {
                    // Determine if this is an elapsed event
                    let is_elapsed = matches!(event, TestEvent::Elapsed);

                    // Process the event
                    match event {
                        TestEvent::Input(input_otlp) => {
                            // Convert and send input
                            let rec = match &input_otlp {
                                OtlpProtoMessage::Traces(t) => {
                                    encode_spans_otap_batch(t).expect("encode traces")
                                }
                                OtlpProtoMessage::Logs(l) => {
                                    encode_logs_otap_batch(l).expect("encode logs")
                                }
                                OtlpProtoMessage::Metrics(_) => unimplemented!("metrics"),
                            };

                            let pdata = OtapPdata::new_default(rec.into());
                            let pdata = if subscribe {
                                pdata.test_subscribe_to(
                                    Interests::ACKS | Interests::NACKS,
                                    TestCallData::new_with(input_idx, 0).into(),
                                    1,
                                )
                            } else {
                                pdata
                            };

                            ctx.process(Message::PData(pdata))
                                .await
                                .expect("process input");

                            input_idx += 1;
                        }
                        TestEvent::Elapsed => {
                            // Elapsed event - no input to process
                        }
                    }

                    // If this is an Elapsed event, deliver the pending DelayedData if present
                    if is_elapsed {
                        if let Some((when, data)) = pending_delay.take() {
                            // Note we deliver "when" exactly as the DelayData requested,
                            // which is a future timestamp; however it's the deadline requested,
                            // and since "when" passes through, the comparison is succesful using
                            // the expected instant.
                            let delayed_msg =
                                Message::Control(NodeControlMsg::DelayedData { when, data });
                            ctx.process(delayed_msg).await.expect("process delayed");
                        }
                    }

                    // Drain outputs, ack/nack them, and drain control channel until empty
                    loop {
                        let mut looped = 0;

                        // Drain outputs produced by this event
                        for new_output in ctx.drain_pdata().await {
                            event_outputs[event_idx].outputs.push(new_output.clone());
                            total_outputs += 1;
                            looped += 1;

                            // Apply ack/nack policy
                            if new_output.has_subscribers() {
                                let policy = nack_policy
                                    .as_ref()
                                    .map(|p| p(total_outputs - 1, &new_output))
                                    .unwrap_or(AckPolicy::Ack);

                                match policy {
                                    AckPolicy::Ack => {
                                        ctx.process(Message::Control(NodeControlMsg::Ack(
                                            Context::next_ack(AckMsg::new(new_output))
                                                .expect("has subs")
                                                .1,
                                        )))
                                        .await
                                        .expect("process ack");
                                    }
                                    AckPolicy::Nack(reason) => {
                                        ctx.process(Message::Control(NodeControlMsg::Nack(
                                            Context::next_nack(NackMsg::new(reason, new_output))
                                                .expect("has subs")
                                                .1,
                                        )))
                                        .await
                                        .expect("process nack");
                                    }
                                }
                            }
                        }

                        // Drain control channel for DelayData requests and acks/nacks
                        loop {
                            match pipeline_rx.try_recv() {
                                Ok(PipelineControlMsg::DelayData { when, data, .. }) => {
                                    looped += 1;
                                    pending_delay = Some((when, data));
                                }
                                Ok(PipelineControlMsg::DeliverAck { ack, .. }) => {
                                    looped += 1;
                                    let calldata: TestCallData =
                                        ack.calldata.try_into().expect("calldata");
                                    received_acks.push(calldata);
                                }
                                Ok(PipelineControlMsg::DeliverNack { nack, .. }) => {
                                    looped += 1;
                                    let calldata: TestCallData =
                                        nack.calldata.try_into().expect("calldata");
                                    received_nacks.push(calldata);
                                }
                                Ok(_) => {
                                    panic!("unexpected case");
                                }
                                Err(_) => {
                                    break;
                                }
                            }
                        }

                        // If no outputs and no control messages, we're done with this event
                        if looped == 0 {
                            break;
                        }
                    }
                }
                // TODO: tests shutdown in this framework (add TestEvent::Shutdown).

                // Verify subscribe mode
                if subscribe {
                    if nack_policy.is_none() {
                        assert_eq!(received_acks.len(), num_inputs);
                    } else {
                        assert_eq!(received_acks.len() + received_nacks.len(), num_inputs);
                    }
                }

                // Collect telemetry
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");

                // Verify all outputs are equivalent to inputs
                let outputs: Vec<OtlpProtoMessage> = event_outputs
                    .iter()
                    .flat_map(|out| {
                        out.outputs.iter().map(|p| {
                            let rec: OtapArrowRecords = p.clone().payload().try_into().unwrap();
                            otap_to_otlp(&rec)
                        })
                    })
                    .collect();

                let output_item_count: usize = outputs.iter().map(|m| m.num_items()).sum();

                // First item counts, then data values.
                assert_eq!(output_item_count, input_item_count);
                assert_equivalent(&inputs_otlp, &outputs);

                // Test-specific validation.
                (verify_outputs)(&event_outputs);
            })
            .validate(move |_| async move {
                // TODO: Not clear why, but this sleep is necessary (probably flaky)
                // for the NodeControlMsg::CollectTelemetry sent above to take effect.
                tokio::time::sleep(Duration::from_millis(50)).await;
                verify_item_metrics(&metrics_registry, signal, input_item_count);
            });
    }

    /// Generic test helper for size-based flush
    fn test_size_flush(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>, subscribe: bool) {
        let inputs: Vec<_> = inputs_otlp.collect();
        let mut events: Vec<TestEvent> =
            inputs.iter().map(|i| TestEvent::Input(i.clone())).collect();
        events.push(TestEvent::Elapsed);

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            json!({
                "send_batch_size": 4,
                "send_batch_max_size": 5,
                "timeout": "1s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Find first non-empty event (should have size-triggered output)
                let first_output_event = event_outputs
                    .iter()
                    .find(|e| !e.outputs.is_empty())
                    .expect("should have at least one output event");

                // Verify first batch had at least threshold items (size-triggered)
                let first_batch_rec = first_output_event.message(0);
                assert!(
                    first_batch_rec.num_items() >= 4,
                    "first batch has at least threshold items (size-triggered)"
                );
            },
        );
    }

    #[test]
    fn test_size_flush_traces_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_traces().into()), false);
    }

    #[test]
    fn test_size_flush_traces_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_traces().into()), true);
    }

    #[test]
    fn test_size_flush_logs_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_logs().into()), false);
    }

    #[test]
    fn test_size_flush_logs_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_size_flush((0..2).map(|_| datagen.generate_logs().into()), true);
    }

    /// Generic test for timer flush
    fn test_timer_flush(input_otlp: OtlpProtoMessage, subscribe: bool) {
        let events = vec![TestEvent::Input(input_otlp), TestEvent::Elapsed];

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            json!({
                "send_batch_size": 10,  // Higher than input (3 items), so won't trigger size flush
                "send_batch_max_size": 10,
                "timeout": "50ms"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Event 0 (Input): Should have NO outputs (size threshold not reached)
                assert!(
                    event_outputs[0].outputs.is_empty(),
                    "input event should not trigger size flush"
                );

                // Event 1 (Elapsed): Should have timer-triggered output
                assert_eq!(
                    event_outputs[1].outputs.len(),
                    1,
                    "elapsed event should trigger timer flush"
                );

                let output_rec = event_outputs[1].message(0);
                assert_eq!(output_rec.num_items(), 3, "should flush all 3 items");
            },
        );
    }

    #[test]
    fn test_timer_flush_traces_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_traces().into(), false);
    }

    #[test]
    fn test_timer_flush_traces_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_traces().into(), true);
    }

    #[test]
    fn test_timer_flush_logs_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_logs().into(), false);
    }

    #[test]
    fn test_timer_flush_logs_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_timer_flush(datagen.generate_logs().into(), true);
    }

    /// Generic test for splitting oversize batches
    fn test_split_oversize(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>, subscribe: bool) {
        let inputs: Vec<_> = inputs_otlp.collect();
        let events: Vec<TestEvent> = inputs.iter().map(|i| TestEvent::Input(i.clone())).collect();

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            json!({
                "send_batch_size": 3,
                "send_batch_max_size": 3,
                "timeout": "1s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Collect all outputs across events
                let all_outputs: Vec<_> = event_outputs.iter().flat_map(|e| e.messages()).collect();

                // Should emit 3 batches of 3 items each (splits at max size)
                assert_eq!(all_outputs.len(), 3, "should emit 3 full batches");

                for batch in all_outputs {
                    assert_eq!(batch.num_items(), 3, "each batch should have 3 items");
                }
            },
        );
    }

    #[test]
    fn test_split_oversize_traces_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_traces().into()), false);
    }

    #[test]
    fn test_split_oversize_traces_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_traces().into()), true);
    }

    #[test]
    fn test_split_oversize_logs_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_logs().into()), false);
    }

    #[test]
    fn test_split_oversize_logs_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_oversize((0..3).map(|_| datagen.generate_logs().into()), true);
    }

    /// Generic test for concatenation and rebuffering
    fn test_concatenate(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>, subscribe: bool) {
        let inputs: Vec<_> = inputs_otlp.collect();
        let mut events: Vec<TestEvent> =
            inputs.iter().map(|i| TestEvent::Input(i.clone())).collect();
        events.push(TestEvent::Elapsed);

        let cfg = json!({
            "send_batch_size": 5,
            "send_batch_max_size": 10,
            "timeout": "1s"
        });

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            cfg,
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Collect all outputs across events
                let all_outputs: Vec<_> = event_outputs.iter().flat_map(|e| e.messages()).collect();

                // With send_batch_size=5, send_batch_max_size=10, 4 inputs of 3 items each = 12 items
                // Should emit 2 batches of 6 items each (concatenation/rebuffering)
                assert_eq!(all_outputs.len(), 2, "should emit 2 batches at threshold");

                for batch in all_outputs.iter() {
                    assert_eq!(batch.num_items(), 6, "each batch should have 6 items");
                }
            },
        );
    }

    #[test]
    fn test_concatenate_traces_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_traces().into()), false);
    }

    #[test]
    fn test_concatenate_traces_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_traces().into()), true);
    }

    #[test]
    fn test_concatenate_logs_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_logs().into()), false);
    }

    #[test]
    fn test_concatenate_logs_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_concatenate((0..4).map(|_| datagen.generate_logs().into()), true);
    }

    /// Generic test for split-only mode (no timeout, no send_batch_size)
    fn test_split_only(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>, subscribe: bool) {
        let inputs: Vec<_> = inputs_otlp.collect();
        let events: Vec<TestEvent> = inputs.iter().map(|i| TestEvent::Input(i.clone())).collect();

        let cfg = json!({
            "send_batch_size": null,
            "send_batch_max_size": 2,
            "timeout": "0s",
        });

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            cfg,
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Collect all outputs across events
                let all_outputs: Vec<_> = event_outputs.iter().flat_map(|e| e.messages()).collect();

                // With no send_batch_size and no timeout, flushes immediately on every input
                // With send_batch_max_size=2, each 3-item input splits to [2, 1]
                assert_eq!(all_outputs.len(), 4, "should emit 4 batches: 2, 1, 2, 1");

                let batch_sizes: Vec<_> = all_outputs.iter().map(|p| p.num_items()).collect();
                assert_eq!(
                    batch_sizes,
                    vec![2, 1, 2, 1],
                    "split pattern without rebuffering"
                );
            },
        );
    }

    #[test]
    fn test_split_only_traces_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_traces().into()), false);
    }

    #[test]
    fn test_split_only_traces_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_traces().into()), true);
    }

    #[test]
    fn test_split_only_logs_no_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_logs().into()), false);
    }

    #[test]
    fn test_split_only_logs_with_ack() {
        let mut datagen = DataGenerator::new(1);
        test_split_only((0..2).map(|_| datagen.generate_logs().into()), true);
    }

    /// Test nack delivery
    fn test_split_with_nack_ordering(
        create_marked_input: impl Fn(usize) -> OtlpProtoMessage + Send + 'static,
        extract_markers: impl Fn(&OtlpProtoMessage) -> Vec<u64> + Send + Clone + 'static,
        nack_position: usize,
    ) {
        let cfg = json!({
            "send_batch_size": 4,
            "send_batch_max_size": 5,
            "timeout": "1s",
        });

        // Use inputs with unique markers
        let num_inputs = 20;
        let inputs_otlp: Vec<OtlpProtoMessage> = (0..num_inputs).map(create_marked_input).collect();

        let mut events: Vec<TestEvent> = inputs_otlp
            .iter()
            .map(|i| TestEvent::Input(i.clone()))
            .collect();
        events.push(TestEvent::Elapsed);

        run_batch_processor_test(
            events.into_iter(),
            true,
            cfg,
            Some(move |idx: usize, _output: &OtapPdata| -> AckPolicy {
                if idx == nack_position {
                    AckPolicy::Nack("test nack")
                } else {
                    AckPolicy::Ack
                }
            }),
            move |event_outputs| {
                let mut max_marker: Option<u64> = None;

                for output_msg in event_outputs {
                    let markers: Vec<_> = output_msg
                        .messages()
                        .iter()
                        .flat_map(&extract_markers)
                        .collect();

                    if markers.is_empty() {
                        continue;
                    }

                    let batch_min = *markers.iter().min().unwrap();
                    let batch_max = *markers.iter().max().unwrap();

                    // Verify in-line property: current batch's minimum must be > previous max
                    if let Some(prev_max) = max_marker {
                        assert!(batch_min > prev_max);
                    }

                    max_marker = Some(batch_max);
                }
            },
        );
    }

    /// Create traces with unique timestamps as markers
    fn create_marked_traces(base_id: usize) -> OtlpProtoMessage {
        let base_time = 1_000_000_000 + (base_id * 1000) as u64;
        let spans: Vec<Span> = (0..3)
            .map(|i| Span {
                name: format!("span_{}", base_id * 3 + i),
                start_time_unix_nano: base_time + i as u64,
                end_time_unix_nano: base_time + i as u64 + 100,
                ..Default::default()
            })
            .collect();

        OtlpProtoMessage::Traces(TracesData {
            // Note we have resource=None
            resource_spans: vec![ResourceSpans {
                scope_spans: vec![ScopeSpans {
                    // Note we have scope=Default
                    scope: Some(InstrumentationScope::default()),
                    spans,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
    }

    /// Extract timestamps from traces as markers
    fn extract_trace_markers(msg: &OtlpProtoMessage) -> Vec<u64> {
        if let OtlpProtoMessage::Traces(traces) = msg {
            traces
                .resource_spans
                .iter()
                .flat_map(|rs| &rs.scope_spans)
                .flat_map(|ss| &ss.spans)
                .map(|span| span.start_time_unix_nano)
                .collect()
        } else {
            vec![]
        }
    }

    /// Create logs with unique timestamps as markers
    fn create_marked_logs(base_id: usize) -> OtlpProtoMessage {
        let base_time = 1_000_000_000 + (base_id * 1000) as u64;
        let log_records: Vec<LogRecord> = (0..3)
            .map(|i| LogRecord {
                time_unix_nano: base_time + i,
                ..Default::default()
            })
            .collect();

        OtlpProtoMessage::Logs(LogsData {
            // Note we have resource=None
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    // Note we have scope=None
                    log_records,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
    }

    /// Extract timestamps from logs as markers
    fn extract_log_markers(msg: &OtlpProtoMessage) -> Vec<u64> {
        if let OtlpProtoMessage::Logs(logs) = msg {
            logs.resource_logs
                .iter()
                .flat_map(|rl| &rl.scope_logs)
                .flat_map(|sl| &sl.log_records)
                .map(|lr| lr.time_unix_nano)
                .collect()
        } else {
            vec![]
        }
    }

    #[test]
    fn test_traces_nack_ordering_position_0() {
        test_split_with_nack_ordering(create_marked_traces, extract_trace_markers, 0);
    }

    #[test]
    fn test_traces_nack_ordering_position_1() {
        test_split_with_nack_ordering(create_marked_traces, extract_trace_markers, 1);
    }

    #[test]
    fn test_traces_nack_ordering_position_2() {
        test_split_with_nack_ordering(create_marked_traces, extract_trace_markers, 2);
    }

    #[test]
    fn test_traces_nack_ordering_position_3() {
        test_split_with_nack_ordering(create_marked_traces, extract_trace_markers, 3);
    }

    #[test]
    fn test_logs_nack_ordering_position_0() {
        test_split_with_nack_ordering(create_marked_logs, extract_log_markers, 0);
    }

    #[test]
    fn test_logs_nack_ordering_position_1() {
        test_split_with_nack_ordering(create_marked_logs, extract_log_markers, 1);
    }

    #[test]
    fn test_logs_nack_ordering_position_2() {
        test_split_with_nack_ordering(create_marked_logs, extract_log_markers, 2);
    }

    #[test]
    fn test_logs_nack_ordering_position_3() {
        test_split_with_nack_ordering(create_marked_logs, extract_log_markers, 3);
    }
}
