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
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_config::{SignalFormat, SignalType};
use otap_df_engine::MessageSourceLocalEffectHandlerExtension;
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
use otap_df_pdata::{
    OtapArrowRecords, OtapPayload, OtapPayloadHelpers, OtlpProtoBytes, error::Error as PDataError,
    otap::batching::make_item_batches, otlp::batching::make_bytes_batches,
};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::num::NonZeroU64;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otel:batch:processor";

/// Default configuration item min-size (OTAP default)
pub const DEFAULT_OTAP_MIN_SIZE_ITEMS: usize = 8192;

/// Default configuration item min-size (OTLP default)
pub const DEFAULT_OTLP_MIN_SIZE_BYTES: usize = 262144;

/// Timeout in milliseconds for periodic flush
pub const DEFAULT_TIMEOUT_MS: u64 = 200;

/// Log messages
const LOG_MSG_BATCHING_FAILED_PREFIX: &str = "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// How to size a batch.
///
/// Note: these are not always supported. In the present code, the only
/// supported Sizer value is Items. We expect future support for bytes and
/// requests sizers.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sizer {
    /// Count requests. This metric counts one per OtapPdata message.
    Requests,
    /// Count items.  The number of log records, trace spans, or
    /// metric data points.
    Items,
    /// Count bytes.
    Bytes,
}

impl Sizer {
    /// Returns Sizer-specific size logic.
    fn batch_size<T: OtapPayloadHelpers>(&self, payload: &T) -> Result<usize, PDataError> {
        match self {
            Self::Requests => Ok(1),
            Self::Items => Ok(payload.num_items()),
            Self::Bytes => payload.num_bytes().ok_or_else(|| PDataError::Format {
                error: "bytes encoding not known".into(),
            }),
        }
    }
}

/// Min/max size for a specific format
#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    /// Flush current batch when this count is reached, as a
    /// minimum. Measures the quantity indicated by `sizer`. When
    /// pending data reaches this threshold a new batch will form and
    /// be sent.
    pub min_size: Option<NonZeroUsize>,

    /// Optionally limit batch sizes to an upper bound. Measured in
    /// the quantity indicated by `sizer`, as described for min_size.
    /// This limit is passed to the lower-level batching logic as a
    /// not-to-exceed maximum.
    pub max_size: Option<NonZeroUsize>,

    /// The sizer, "requests", "items", or "bytes".  See `Sizer`.
    pub sizer: Sizer,
}

/// Batching format option.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BatchingFormat {
    /// Force to OTAP (default).
    Otap,

    /// Force to OTLP.
    Otlp,

    /// Preserve format, support both independently.
    Preserve,
}

/// The common signature of the batching methods
trait Batcher<T: OtapPayloadHelpers> {
    fn make_batches(
        fmtcfg: &FormatConfig,
        signal: SignalType,
        records: Vec<T>,
    ) -> Result<Vec<T>, PDataError>;

    /// We are using an empty DelayData request as a one-shot
    /// timer. This returns the appropriate empty request.
    /// TODO: Add proper one-shot timer and cancellation, see #1472.
    fn wakeup(signal: SignalType) -> T;
}

/// Batch processor configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The OTAP configuration.
    #[serde(default = "default_otap")]
    pub otap: FormatConfig,

    /// The OTLP configuration.
    #[serde(default = "default_otlp")]
    pub otlp: FormatConfig,

    /// Flush non-empty batches on this interval, which may be 0 for
    /// immediate flush or None for no timeout.
    #[serde(with = "humantime_serde", default = "default_flush_timeout")]
    pub flush_timeout: Duration,

    /// Limits the number of pending requests for ack/nack tracking.
    #[serde(default = "default_inbound_request_limit")]
    pub inbound_request_limit: NonZeroUsize,

    /// Limits the number of outbound requests for ack/nack tracking.
    #[serde(default = "default_outbound_request_limit")]
    pub outbound_request_limit: NonZeroUsize,

    /// Batching format choice. Default is to preserve format, meaning
    /// to batch OTAP and OTLP separately.
    #[serde(default = "default_batching_format")]
    pub format: BatchingFormat,
}

const fn default_otap_min_size_items() -> Option<NonZeroUsize> {
    NonZeroUsize::new(DEFAULT_OTAP_MIN_SIZE_ITEMS)
}

const fn default_otap_max_size_items() -> Option<NonZeroUsize> {
    // No upper-bound
    None
}

const fn default_otap_sizer_items() -> Sizer {
    Sizer::Bytes
}

const fn default_otlp_min_size_bytes() -> Option<NonZeroUsize> {
    NonZeroUsize::new(DEFAULT_OTLP_MIN_SIZE_BYTES)
}

const fn default_otlp_max_size_bytes() -> Option<NonZeroUsize> {
    // No upper-bound
    None
}

const fn default_otlp_sizer_bytes() -> Sizer {
    Sizer::Bytes
}

const fn default_flush_timeout() -> Duration {
    Duration::from_millis(DEFAULT_TIMEOUT_MS)
}

const fn default_inbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(1024).expect("ok") // default_min_size/8
}

const fn default_outbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(512).expect("ok") // default_min_size/16
}

const fn default_batching_format() -> BatchingFormat {
    BatchingFormat::Preserve
}

const fn default_otap() -> FormatConfig {
    FormatConfig {
        min_size: default_otap_min_size_items(),
        max_size: default_otap_max_size_items(),
        sizer: default_otap_sizer_items(),
    }
}

const fn default_otlp() -> FormatConfig {
    FormatConfig {
        min_size: default_otlp_min_size_bytes(),
        max_size: default_otlp_max_size_bytes(),
        sizer: default_otlp_sizer_bytes(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            otap: default_otap(),
            otlp: default_otlp(),
            flush_timeout: default_flush_timeout(),
            inbound_request_limit: default_inbound_request_limit(),
            outbound_request_limit: default_outbound_request_limit(),
            format: default_batching_format(),
        }
    }
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let no_timeout = self.flush_timeout == Duration::ZERO;
        match self.format {
            BatchingFormat::Otap => {
                self.otap.validate(SignalFormat::OtapRecords, no_timeout)?;
            }
            BatchingFormat::Otlp => {
                self.otlp.validate(SignalFormat::OtlpBytes, no_timeout)?;
            }
            BatchingFormat::Preserve => {
                self.otap.validate(SignalFormat::OtapRecords, no_timeout)?;
                self.otlp.validate(SignalFormat::OtlpBytes, no_timeout)?;
            }
        };

        Ok(())
    }
}

impl FormatConfig {
    #[cfg(test)]
    const fn new_items(min_size: usize, max_size: usize) -> FormatConfig {
        FormatConfig {
            min_size: NonZeroUsize::new(min_size),
            max_size: NonZeroUsize::new(max_size),
            sizer: Sizer::Items,
        }
    }

    #[cfg(test)]
    const fn new_bytes(min_size: usize, max_size: usize) -> FormatConfig {
        FormatConfig {
            min_size: NonZeroUsize::new(min_size),
            max_size: NonZeroUsize::new(max_size),
            sizer: Sizer::Bytes,
        }
    }

    /// The lower of the two size limits.
    fn lower_limit(&self) -> usize {
        self.min_size.or(self.max_size).expect("valid").get()
    }

    /// Validate the config, given its format. `no_timeout` indicates the
    /// parent Config has flush_timeout == 0.
    pub fn validate(&self, format: SignalFormat, no_timeout: bool) -> Result<(), ConfigError> {
        // At least one size is set.
        if self.min_size.or(self.max_size).is_none() {
            return Err(ConfigError::InvalidUserConfig {
                error: "max_size or min_size must be set".into(),
            });
        }

        // Check for a supported sizer. Presently, OTAP supports only
        // items, OTLP supports only bytes.
        let (expect_sizer, with_msg) = match format {
            SignalFormat::OtapRecords => (Sizer::Items, "OTAP batch sizer: must be items"),
            SignalFormat::OtlpBytes => (Sizer::Bytes, "OTLP batch sizer: must be bytes"),
        };
        if self.sizer != expect_sizer {
            return Err(ConfigError::InvalidUserConfig {
                error: with_msg.into(),
            });
        }

        // If both sizes are set, check max_size is >= the min_size.
        if let (Some(max_size), Some(min_size)) = (self.max_size, self.min_size) {
            if max_size < min_size {
                return Err(ConfigError::InvalidUserConfig {
                    error: format!(
                        "max_size ({}) must be >= min_size ({}) or unset",
                        max_size, min_size,
                    ),
                });
            }
        }

        // no_timeout indicates there is not a time-based flush criteria, which
        // raises requirements:
        if no_timeout {
            // If min_size is set, we need a timeout to avoid
            // indefinite delay.
            if self.min_size.is_some() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "min_size set requires flush_timeout is set".into(),
                });
            }
            // If max_size is unset, we're doing nothing with a batch processor,
            // so this is considered an error.
            if self.max_size.is_none() {
                return Err(ConfigError::InvalidUserConfig {
                    error: "flush_timeout unset requires max_size is set".into(),
                });
            }
        }

        Ok(())
    }
}

/// Per-signal state
struct SignalBatches<T: OtapPayloadHelpers> {
    logs: SignalBuffer<T>,
    metrics: SignalBuffer<T>,
    traces: SignalBuffer<T>,
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

struct Inputs<T: OtapPayloadHelpers> {
    /// Input batches.
    pending: Vec<T>,

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
struct SignalBuffer<T: OtapPayloadHelpers> {
    /// Pending input state.
    inputs: Inputs<T>,

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
    otap_signals: Option<SignalBatches<OtapArrowRecords>>,
    otlp_signals: Option<SignalBatches<OtlpProtoBytes>>,
    metrics: MetricSet<BatchProcessorMetrics>,
}

struct BatchProcessorFormat<'a, T: OtapPayloadHelpers> {
    config: &'a Config,
    fmtcfg: &'a FormatConfig,
    signals: &'a mut SignalBatches<T>,
    metrics: &'a mut MetricSet<BatchProcessorMetrics>,
}

struct BatchProcessorSignal<'a, T: OtapPayloadHelpers>
where
    SignalBuffer<T>: Batcher<T>,
{
    signal: SignalType,
    config: &'a Config,
    fmtcfg: &'a FormatConfig,
    buffer: &'a mut SignalBuffer<T>,
    metrics: &'a mut MetricSet<BatchProcessorMetrics>,
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
    /// preserve the original NodeUserConfig (including outputs/default_output).
    pub fn build_from_json(
        cfg: &Value,
        metrics: MetricSet<BatchProcessorMetrics>,
    ) -> Result<Self, ConfigError> {
        let config: Config =
            serde_json::from_value(cfg.clone()).map_err(|e| ConfigError::InvalidUserConfig {
                error: format!("invalid OTAP batch processor config: {e}"),
            })?;

        // This checks that if both are present, max_size >= min_size, and
        // that at least one is present so that lower_limit is valid below.
        config.validate()?;

        let otap_signals: Option<SignalBatches<OtapArrowRecords>> = config
            .format
            .has_otap()
            .then(|| SignalBatches::new(&config));
        let otlp_signals: Option<SignalBatches<OtlpProtoBytes>> = config
            .format
            .has_otlp()
            .then(|| SignalBatches::new(&config));

        Ok(BatchProcessor {
            config,
            otap_signals,
            otlp_signals,
            metrics,
        })
    }

    /// Backward-compatible helper used by unit tests to construct a processor wrapper
    /// directly from JSON. Note: This creates a fresh NodeUserConfig without outputs,
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
            if let Some(mut otap_signals) = self.otap_format() {
                otap_signals
                    .for_signal(signal)
                    .flush_signal_impl(effect, Instant::now(), FlushReason::Shutdown)
                    .await?;
            }
            if let Some(mut otlp_signals) = self.otlp_format() {
                otlp_signals
                    .for_signal(signal)
                    .flush_signal_impl(effect, Instant::now(), FlushReason::Shutdown)
                    .await?;
            }
        }
        Ok(())
    }

    fn otap_format(&mut self) -> Option<BatchProcessorFormat<'_, OtapArrowRecords>> {
        self.otap_signals
            .as_mut()
            .map(|signals| BatchProcessorFormat {
                fmtcfg: &self.config.otap,
                config: &self.config,
                signals,
                metrics: &mut self.metrics,
            })
    }

    fn otlp_format(&mut self) -> Option<BatchProcessorFormat<'_, OtlpProtoBytes>> {
        self.otlp_signals
            .as_mut()
            .map(|signals| BatchProcessorFormat {
                fmtcfg: &self.config.otlp,
                config: &self.config,
                signals,
                metrics: &mut self.metrics,
            })
    }

    /// Process one incoming batch. Immediately acks empty requests.
    /// If this input causes pending data to exceed the lower bound, it will
    /// flush at least one output.
    async fn process_signal_impl(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        request: OtapPdata,
    ) -> Result<(), EngineError> {
        let items = request.num_items();

        if items == 0 {
            self.metrics.dropped_empty_records.inc();
            effect.notify_ack(AckMsg::new(request)).await?;
            return Ok(());
        }

        // Increment consumed_items for the appropriate signal
        let signal = request.signal_type();
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
        };

        let (ctx, payload) = request.into_parts();

        match payload {
            OtapPayload::OtapArrowRecords(otap) => {
                if self.otap_signals.is_some() {
                    self.otap_format()
                        .expect("some")
                        .for_signal(signal)
                        .accept_payload(effect, ctx, otap, items)
                        .await?
                } else {
                    self.otlp_format()
                        .expect("some")
                        .for_signal(signal)
                        .accept_payload(effect, ctx, otap.try_into()?, items)
                        .await?
                }
            }
            OtapPayload::OtlpBytes(otlp) => {
                if self.otlp_signals.is_some() {
                    self.otlp_format()
                        .expect("some")
                        .for_signal(signal)
                        .accept_payload(effect, ctx, otlp, items)
                        .await?
                } else {
                    self.otap_format()
                        .expect("some")
                        .for_signal(signal)
                        .accept_payload(effect, ctx, otlp.try_into()?, items)
                        .await?
                }
            }
        };
        Ok(())
    }
}

impl<'a, T: OtapPayloadHelpers> BatchProcessorFormat<'a, T>
where
    SignalBuffer<T>: Batcher<T>,
{
    const fn for_signal(&mut self, signal: SignalType) -> BatchProcessorSignal<'_, T> {
        BatchProcessorSignal {
            signal,
            config: self.config,
            fmtcfg: self.fmtcfg,
            buffer: match signal {
                SignalType::Logs => &mut self.signals.logs,
                SignalType::Traces => &mut self.signals.traces,
                SignalType::Metrics => &mut self.signals.metrics,
            },
            metrics: self.metrics,
        }
    }
}

impl Batcher<OtapArrowRecords> for SignalBuffer<OtapArrowRecords> {
    fn make_batches(
        fmtcfg: &FormatConfig,
        signal: SignalType,
        pending: Vec<OtapArrowRecords>,
    ) -> Result<Vec<OtapArrowRecords>, PDataError> {
        // OTAP only supports Sizer::Items (checked in validate)
        debug_assert_eq!(fmtcfg.sizer, Sizer::Items);
        make_item_batches(signal, nzu_to_nz64(fmtcfg.max_size), pending)
    }

    fn wakeup(signal: SignalType) -> OtapArrowRecords {
        match signal {
            SignalType::Logs => OtapArrowRecords::Logs(otap_df_pdata::otap::Logs::default()),
            SignalType::Metrics => {
                OtapArrowRecords::Metrics(otap_df_pdata::otap::Metrics::default())
            }
            SignalType::Traces => OtapArrowRecords::Traces(otap_df_pdata::otap::Traces::default()),
        }
    }
}

impl Batcher<OtlpProtoBytes> for SignalBuffer<OtlpProtoBytes> {
    fn make_batches(
        fmtcfg: &FormatConfig,
        signal: SignalType,
        pending: Vec<OtlpProtoBytes>,
    ) -> Result<Vec<OtlpProtoBytes>, PDataError> {
        // OTLP only supports Sizer::Bytes (checked in validate)
        debug_assert_eq!(fmtcfg.sizer, Sizer::Bytes);
        make_bytes_batches(signal, nzu_to_nz64(fmtcfg.max_size), pending)
    }

    fn wakeup(signal: SignalType) -> OtlpProtoBytes {
        match signal {
            SignalType::Logs => OtlpProtoBytes::ExportLogsRequest(Bytes::new()),
            SignalType::Metrics => OtlpProtoBytes::ExportMetricsRequest(Bytes::new()),
            SignalType::Traces => OtlpProtoBytes::ExportTracesRequest(Bytes::new()),
        }
    }
}

impl<'a, T: OtapPayloadHelpers> BatchProcessorSignal<'a, T>
where
    SignalBuffer<T>: Batcher<T>,
{
    async fn accept_payload(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        ctx: Context,
        payload: T,
        items: usize,
    ) -> Result<(), EngineError> {
        // If there are subscribers, calculate an inbound slot key.
        let inkey = ctx
            .has_subscribers()
            .then(|| {
                self.buffer
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
        let timeout = self.config.flush_timeout;
        let mut arrival: Option<Instant> = None;
        if timeout != Duration::ZERO && self.buffer.inputs.is_empty() {
            let now = Instant::now();
            arrival = Some(now);
            self.buffer
                .set_arrival(self.signal, now, timeout, effect)
                .await?;
        }

        self.buffer
            .inputs
            .accept(payload, BatchPortion::new(inkey, items));

        // Flush based on size when the batch reaches the lower limit.
        if timeout != Duration::ZERO && self.buffer.inputs.items < self.fmtcfg.lower_limit() {
            Ok(())
        } else {
            self.flush_signal_impl(
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
        effect: &mut local::EffectHandler<OtapPdata>,
        now: Instant,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        // If the input is empty.
        if self.buffer.inputs.is_empty() {
            return Ok(());
        }

        // If this is a timer-based flush and we were called too soon,
        // skip. this may happen if the batch for which the timer was set
        // flushes for size before the timer.
        if reason == FlushReason::Timer
            && self.config.flush_timeout != Duration::ZERO
            && now.duration_since(self.buffer.arrival.expect("timed")) < self.config.flush_timeout
        {
            return Ok(());
        }

        let mut inputs = self.buffer.inputs.drain();

        match reason {
            FlushReason::Size => self.metrics.flushes_size.inc(),
            FlushReason::Timer => self.metrics.flushes_timer.inc(),
            FlushReason::Shutdown => {}
        }

        let count = inputs.requests();
        let pending = inputs.take_pending();

        let mut output_batches =
            match SignalBuffer::<T>::make_batches(self.fmtcfg, self.signal, pending) {
                Ok(v) => v,
                Err(e) => {
                    self.metrics.batching_errors.add(count as u64);
                    log_batching_failed(effect, self.signal, &e).await;
                    let str = e.to_string();
                    let res = Err(str.clone());
                    // In this case, we are sending failure to all the pending inputs.
                    self.buffer
                        .handle_partial_responses(self.signal, effect, &res, inputs.context)
                        .await?;
                    return Err(EngineError::InternalError { message: str });
                }
            };

        // If size-triggered and we requested splitting (upper_limit is Some), re-buffer the last partial
        // output if it is smaller than the configured lower_limit. Timer/Shutdown flush everything.
        if self.config.flush_timeout != Duration::ZERO
            && reason == FlushReason::Size
            && self.fmtcfg.max_size.is_some()
            && output_batches.len() > 1
        {
            let num_output = output_batches.len();
            let sizer = self.fmtcfg.sizer;
            match sizer {
                Sizer::Items => {
                    // This property holds because item-based batches never batches
                    // short of min_size.
                    debug_assert!(
                        sizer
                            .batch_size(&output_batches[0])
                            .expect("first over lower_limit")
                            >= self.fmtcfg.lower_limit()
                    );
                }
                Sizer::Requests => unreachable!("requests sizer not implemented"),
                Sizer::Bytes => {
                    // All batches can be under or over. We know byte size.
                    debug_assert!(sizer.batch_size(&output_batches[num_output - 1]).is_ok());
                }
            };

            let last_batch_size = self
                .fmtcfg
                .sizer
                .batch_size(&output_batches[num_output - 1])?;

            if last_batch_size < self.fmtcfg.lower_limit() {
                self.buffer.take_remaining(&mut inputs, &mut output_batches);

                // We use the latest arrival time as the new arrival for timeout purposes.
                self.buffer
                    .set_arrival(self.signal, now, self.config.flush_timeout, effect)
                    .await?;
            }
        }

        let mut input_context = inputs.take_context();

        for records in output_batches {
            let items = records.num_items();
            let mut pdata = OtapPdata::new(Context::default(), records.into());

            // Increment produced_items for the appropriate signal
            match self.signal {
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
            if let Some(ctxs) = self.buffer.drain_context(items, &mut input_context) {
                let (outkey, _notused) =
                    self.buffer
                        .outbound
                        .allocate(|| (ctxs, ()))
                        .ok_or_else(|| EngineError::ProcessorError {
                            processor: effect.processor_id(),
                            kind: ProcessorErrorKind::Other,
                            error: "outbound slots not available".into(),
                            source_detail: "".into(),
                        })?;

                effect.subscribe_to(
                    Interests::NACKS | Interests::ACKS,
                    outkey.into(),
                    &mut pdata,
                );
            }
            effect.send_message_with_source_node(pdata).await?;
        }

        Ok(())
    }

    async fn handle(
        &mut self,
        signal: SignalType,
        calldata: CallData,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), String>,
    ) -> Result<(), EngineError> {
        let outkey: SlotKey = calldata.try_into()?;

        if let Some(parts) = self.buffer.outbound.take(outkey) {
            self.buffer
                .handle_partial_responses(signal, effect, res, parts)
                .await?;
        }

        Ok(())
    }
}

impl BatchProcessor {
    async fn handle_ack(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        ack: AckMsg<OtapPdata>,
    ) -> Result<(), EngineError> {
        self.handle_response(*ack.accepted, ack.calldata, effect, &Ok(()))
            .await
    }

    async fn handle_nack(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        nack: NackMsg<OtapPdata>,
    ) -> Result<(), EngineError> {
        let res = Err(nack.reason);
        self.handle_response(*nack.refused, nack.calldata, effect, &res)
            .await
    }

    async fn handle_response(
        &mut self,
        retdata: OtapPdata,
        calldata: CallData,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), String>,
    ) -> Result<(), EngineError> {
        if calldata.is_empty() {
            return Ok(());
        }

        let signal = retdata.signal_type();
        match retdata.signal_format() {
            SignalFormat::OtapRecords => {
                self.otap_format()
                    .expect("some")
                    .for_signal(signal)
                    .handle(signal, calldata, effect, res)
                    .await
            }
            SignalFormat::OtlpBytes => {
                self.otlp_format()
                    .expect("some")
                    .for_signal(signal)
                    .handle(signal, calldata, effect, res)
                    .await
            }
        }
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

                    match data.signal_format() {
                        SignalFormat::OtapRecords => {
                            self.otap_format()
                                .expect("some")
                                .for_signal(signal)
                                .flush_signal_impl(effect, when, FlushReason::Timer)
                                .await?
                        }
                        SignalFormat::OtlpBytes => {
                            self.otlp_format()
                                .expect("some")
                                .for_signal(signal)
                                .flush_signal_impl(effect, when, FlushReason::Timer)
                                .await?
                        }
                    };

                    Ok(())
                }
                NodeControlMsg::Ack(ack) => self.handle_ack(effect, ack).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(effect, nack).await,
                NodeControlMsg::TimerTick { .. } => unreachable!(),
            },
            Message::PData(request) => self.process_signal_impl(effect, request).await,
        }
    }
}

impl<T: OtapPayloadHelpers> Default for Inputs<T> {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            context: Vec::new(),
            items: 0,
        }
    }
}

impl BatchingFormat {
    const fn has_otlp(&self) -> bool {
        !matches!(self, Self::Otap)
    }

    const fn has_otap(&self) -> bool {
        !matches!(self, Self::Otlp)
    }
}

impl<T: OtapPayloadHelpers> SignalBatches<T>
where
    SignalBuffer<T>: Batcher<T>,
{
    fn new(config: &Config) -> Self {
        Self {
            logs: SignalBuffer::new(config),
            traces: SignalBuffer::new(config),
            metrics: SignalBuffer::new(config),
        }
    }
}

impl BatchPortion {
    const fn new(inkey: Option<SlotKey>, items: usize) -> Self {
        Self { inkey, items }
    }
}

impl MultiContext {
    const fn new(inputs: Vec<BatchPortion>) -> Self {
        Self { inputs, pos: 0 }
    }
}

impl<T: OtapPayloadHelpers> Inputs<T> {
    fn drain(&mut self) -> Self {
        Self {
            pending: self.pending.drain(..).collect(),
            context: self.context.drain(..).collect(),
            items: std::mem::take(&mut self.items),
        }
    }

    const fn is_empty(&self) -> bool {
        self.items == 0
    }

    const fn requests(&self) -> usize {
        self.pending.len()
    }

    fn accept(&mut self, batch: T, part: BatchPortion) {
        self.items += part.items;
        self.pending.push(batch);
        self.context.push(part);
    }

    fn take_pending(&mut self) -> Vec<T> {
        std::mem::take(&mut self.pending)
    }

    fn take_context(&mut self) -> MultiContext {
        MultiContext::new(std::mem::take(&mut self.context))
    }
}

impl<T: OtapPayloadHelpers> SignalBuffer<T>
where
    Inputs<T>: Default,
    Self: Batcher<T>,
{
    fn new(cfg: &Config) -> Self {
        Self {
            inputs: Inputs::default(),
            inbound: SlotState::new(cfg.inbound_request_limit.get()),
            outbound: SlotState::new(cfg.outbound_request_limit.get()),
            arrival: None,
        }
    }

    /// Takes the residual batch, used in case the final output is less than
    /// the lower bound. This removes the last output btach, the corresponding
    /// context, and places it back in the pending buffer as the first in line.
    fn take_remaining(&mut self, from_inputs: &mut Inputs<T>, output_batches: &mut Vec<T>) {
        // SAFETY: protected by output_batches.len() > 1.
        let remaining = output_batches.pop().expect("has last");
        let last_input = from_inputs.context.last().expect("has last");
        let last_items = remaining.num_items();
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

        effect
            .delay_data(
                now + timeout,
                Box::new(OtapPdata::new(
                    Context::default(),
                    Self::wakeup(signal).into(),
                )),
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
        wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otap_df_config::validation::validate_typed_config::<Config>,
    };

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::OtapPdata;
    use crate::testing::TestCallData;
    use otap_df_config::node::NodeUserConfig;
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
    use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_message_to_bytes, otlp_to_otap};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Helper to create test pipeline context
    fn create_test_pipeline_context() -> (
        otap_df_engine::context::PipelineContext,
        TelemetryRegistryHandle,
    ) {
        let telemetry_registry = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            PipelineGroupId::from("test_group".to_string()),
            PipelineId::from("test_pipeline".to_string()),
            0,
            1, // num_cores
            0,
        );
        (pipeline_ctx, telemetry_registry)
    }

    /// Helper to set up a test runtime with batch processor
    fn setup_test_runtime(
        cfg: Value,
    ) -> (
        TelemetryRegistryHandle,
        otap_df_telemetry::reporter::MetricsReporter,
        otap_df_engine::testing::processor::TestPhase<OtapPdata>,
    ) {
        let rt = TestRuntime::new();
        let telemetry_registry = rt.metrics_registry();
        let metrics_reporter = rt.metrics_reporter();

        // Create processor using TestRuntime's registry
        let controller = ControllerContext::new(telemetry_registry.clone());
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 1, 0);
        let node = test_node("batch-processor-test");
        let mut node_config = NodeUserConfig::new_processor_config(OTAP_BATCH_PROCESSOR_URN);
        node_config.config = cfg;
        let proc_config = ProcessorConfig::new("batch");
        let proc =
            create_otap_batch_processor(pipeline_ctx, node, Arc::new(node_config), &proc_config)
                .expect("create processor");

        let phase = rt.set_processor(proc);

        (telemetry_registry, metrics_reporter, phase)
    }

    /// Helper to verify consumed and produced item metrics
    fn verify_item_metrics(
        telemetry_registry: &TelemetryRegistryHandle,
        signal: SignalType,
        expected_items: usize,
    ) {
        let mut consumed_items = 0u64;
        let mut produced_items = 0u64;
        let mut consumed_batches = 0u64;
        let mut produced_batches = 0u64;

        telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "otap.processor.batch" {
                for (field, value) in iter {
                    match (signal, field.name) {
                        (SignalType::Logs, "consumed.items.logs") => {
                            consumed_items = value.to_u64_lossy()
                        }
                        (SignalType::Logs, "produced.items.logs") => {
                            produced_items = value.to_u64_lossy()
                        }
                        (SignalType::Traces, "consumed.items.traces") => {
                            consumed_items = value.to_u64_lossy()
                        }
                        (SignalType::Traces, "produced.items.traces") => {
                            produced_items = value.to_u64_lossy()
                        }
                        (SignalType::Logs, "consumed.batches.logs") => {
                            consumed_batches = value.to_u64_lossy()
                        }
                        (SignalType::Logs, "produced.batches.logs") => {
                            produced_batches = value.to_u64_lossy()
                        }
                        (SignalType::Traces, "consumed.batches.traces") => {
                            consumed_batches = value.to_u64_lossy()
                        }
                        (SignalType::Traces, "produced.batches.traces") => {
                            produced_batches = value.to_u64_lossy()
                        }
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

        // Prepare a NodeUserConfig with an output and a default_output
        let mut nuc = NodeUserConfig::with_user_config(
            OTAP_BATCH_PROCESSOR_URN.into(),
            serde_json::json!({
                "otap": {
                    "sizer": "items",
                    "min_size": 1000
                },
            }),
        );
        nuc.add_output("main_output");
        nuc.set_default_output("main_output");
        let nuc = Arc::new(nuc);

        // Create processor via factory and ensure the provided NodeUserConfig is preserved
        let proc_cfg = ProcessorConfig::new("batch");
        let node = test_node(proc_cfg.name.clone());
        let wrapper = create_otap_batch_processor(pipeline_ctx, node, nuc.clone(), &proc_cfg)
            .expect("factory should succeed");

        let uc = wrapper.user_config();
        assert!(uc.outputs.iter().any(|port| port.as_ref() == "main_output"));
        assert_eq!(uc.default_output.as_deref(), Some("main_output"));
    }

    #[test]
    fn test_default_config_ok() {
        let _cfg: Config = serde_json::from_value(json!({})).unwrap_or_default();
    }

    #[test]
    fn test_config_validation() {
        // Both sizes set, max >= batch: OK
        let cfg = Config {
            otap: FormatConfig::new_items(100, 200),
            otlp: FormatConfig::new_bytes(10000, 20000),
            flush_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // The OTLP configuration is invalid, but that's OK because
        // format is OTAP.
        let cfg = Config {
            otap: FormatConfig::new_items(100, 0),
            otlp: FormatConfig::new_items(0, 0),
            flush_timeout: Duration::from_millis(100),
            format: BatchingFormat::Otap,
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Only max size: OK (OTLP default)
        let cfg = Config {
            otap: FormatConfig::new_items(0, 200),
            flush_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // All formats
        let cfg = Config {
            otap: FormatConfig::new_items(0, 200),
            otlp: FormatConfig::new_bytes(20000, 0),
            flush_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Split-only: OK
        let cfg = Config {
            otlp: FormatConfig::new_bytes(0, 100),
            otap: FormatConfig::new_items(0, 0),
            flush_timeout: Duration::ZERO,
            format: BatchingFormat::Otlp,
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Format: preserve, 1 invalid
        let cfg = Config {
            otlp: FormatConfig::new_bytes(0, 0),
            format: BatchingFormat::Preserve,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // Both None: ERROR
        let cfg = Config {
            otap: FormatConfig::new_bytes(0, 0),
            format: BatchingFormat::Otap,
            flush_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // Max < batch: ERROR
        let cfg = Config {
            otap: FormatConfig::new_items(200, 100),
            flush_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // lower-bound without timeout: ERROR
        let cfg = Config {
            otap: FormatConfig::new_items(100, 200),
            flush_timeout: Duration::ZERO,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    /// Test event: either process input or deliver pending timers
    #[derive(Clone)]
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

    fn otap_pdata_to_message(data: &OtapPdata) -> OtlpProtoMessage {
        let rec: OtapArrowRecords = data.clone().payload().try_into().unwrap();
        otap_to_otlp(&rec)
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
        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(config);

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
                    .flat_map(|out| out.outputs.iter().map(otap_pdata_to_message))
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
                verify_item_metrics(&telemetry_registry, signal, input_item_count);
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
                "otap": {
                    "min_size": 4,
                    "max_size": 5,
                    "sizer": "items",
                },
                "flush_timeout": "1s"
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
                "otap": {
                    "min_size": 10,  // Higher than input (3 items), so won't trigger size flush
                    "max_size": 10,
                    "sizer": "items",
                },
                "flush_timeout": "50ms"
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
                "otap": {
                    "min_size": 3,
                    "max_size": 3,
                    "sizer": "items",
                },
                "flush_timeout": "1s"
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

        run_batch_processor_test(
            events.iter().cloned(),
            subscribe,
            json!({
                "otap": {
                    "min_size": 5,
                    "max_size": 10,
                    "sizer": "items",
                },
                "flush_timeout": "1s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Collect all outputs across events
                let all_outputs: Vec<_> = event_outputs.iter().flat_map(|e| e.messages()).collect();

                // With min_size=5, max_size=10, 4 inputs of 3 items each = 12 items
                // Should emit 2 batches of 6 items each (concatenation/rebuffering)
                assert_eq!(all_outputs.len(), 2, "should emit 2 batches at threshold");

                for batch in all_outputs.iter() {
                    assert_eq!(batch.num_items(), 6, "each batch should have 6 items");
                }
            },
        );

        run_batch_processor_test(
            events.iter().cloned(),
            subscribe,
            json!({
                "otlp": {
                    "min_size": 50,
                    "max_size": 100,
                    "sizer": "bytes",
                },
                "format": "otlp",
                "flush_timeout": "1s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |_| {},
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

    /// Generic test for split-only mode (no timeout, no min_size)
    fn test_split_only(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>, subscribe: bool) {
        let inputs: Vec<_> = inputs_otlp.collect();
        let events: Vec<TestEvent> = inputs.iter().map(|i| TestEvent::Input(i.clone())).collect();

        run_batch_processor_test(
            events.clone().into_iter(),
            subscribe,
            json!({
                "otap": {
                    "min_size": null,
                    "max_size": 2,
                    "sizer": "items",
                },
                "format": "otap",
                "flush_timeout": "0s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |event_outputs| {
                // Collect all outputs across events
                let all_outputs: Vec<_> = event_outputs.iter().flat_map(|e| e.messages()).collect();
                // With no min_size and no timeout, flushes immediately on every input
                // With max_size=2, each 3-item input splits to [2, 1]
                assert_eq!(all_outputs.len(), 4, "should emit 4 batches: 2, 1, 2, 1");

                let batch_sizes: Vec<_> = all_outputs.iter().map(|p| p.num_items()).collect();
                assert_eq!(
                    batch_sizes,
                    vec![2, 1, 2, 1],
                    "split pattern without rebuffering"
                );
            },
        );

        run_batch_processor_test(
            events.into_iter(),
            subscribe,
            json!({
                "otlp": {
                    "min_size": null,
                    "max_size": 20,
                    "sizer": "bytes",
                },
                "format": "otlp",
                "flush_timeout": "0s"
            }),
            None::<fn(usize, &OtapPdata) -> AckPolicy>,
            |_| {},
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
        // Use inputs with unique markers
        let num_inputs = 20;
        let inputs_otlp: Vec<OtlpProtoMessage> = (0..num_inputs).map(create_marked_input).collect();

        let mut events: Vec<TestEvent> = inputs_otlp
            .iter()
            .map(|i| TestEvent::Input(i.clone()))
            .collect();
        events.push(TestEvent::Elapsed);

        for cfg in [
            json!({
                "otap": {
                    "min_size": 4,
                    "max_size": 5,
                    "sizer": "items",
                },
                "flush_timeout": "1s"
            }),
            json!({
                "otlp": {
                    "min_size": 20,
                    "max_size": 30,
                    "sizer": "bytes",
                },
                "format": "otlp",
                "flush_timeout": "1s"
            }),
        ] {
            let extract_markers = extract_markers.clone();
            run_batch_processor_test(
                events.iter().cloned(),
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

    /// Test Preserve mode: this can't use the same test harness used above because it
    /// arranges mixed-format inputs.
    #[test]
    fn test_preserve_mode_mixed_formats() {
        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "format": "preserve",
            "otap": {
                "min_size": 100,  // Won't trigger size flush
                "max_size": 100,
                "sizer": "items",
            },
            "otlp": {
                "min_size": 10000,  // Won't trigger size flush
                "max_size": 20000,
                "sizer": "bytes",
            },
            "flush_timeout": "50ms"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_tx, mut pipeline_rx) = pipeline_ctrl_msg_channel(10);
                ctx.set_pipeline_ctrl_sender(pipeline_tx);

                // Create test data
                let mut datagen = DataGenerator::new(1);
                let logs1: OtlpProtoMessage = datagen.generate_logs().into();
                let logs2: OtlpProtoMessage = datagen.generate_logs().into();

                // Convert to OTAP format
                let otlp_message1 = otlp_message_to_bytes(&logs1);
                let otap_message2 = otlp_to_otap(&logs2);

                let mut outputs = Vec::new();
                let mut pending_delays: Vec<(Instant, Box<OtapPdata>)> = Vec::new();

                // Send both
                ctx.process(Message::PData(OtapPdata::new_default(otlp_message1.into())))
                    .await
                    .expect("process otap");

                ctx.process(Message::PData(OtapPdata::new_default(otap_message2.into())))
                    .await
                    .expect("process otlp");

                // Drain control channel for DelayData
                while let Ok(PipelineControlMsg::DelayData { when, data, .. }) =
                    pipeline_rx.try_recv()
                {
                    pending_delays.push((when, data));
                }

                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "no outputs before timeout"
                );

                // Trigger timeout
                for (when, data) in pending_delays {
                    ctx.process(Message::Control(NodeControlMsg::DelayedData { when, data }))
                        .await
                        .expect("process delayed");
                }

                // Drain outputs after timeout
                let mut drained_outputs = ctx.drain_pdata().await;
                outputs.append(&mut drained_outputs);
                assert!(!outputs.is_empty(), "should have outputs after timeout");

                // Verify both formats
                let mut has_otap = false;
                let mut has_otlp = false;
                for output in &outputs {
                    match output.signal_format() {
                        SignalFormat::OtapRecords => has_otap = true,
                        SignalFormat::OtlpBytes => has_otlp = true,
                    }
                }

                assert!(has_otap, "should have OTAP output");
                assert!(has_otlp, "should have OTLP output");

                // Test equivalence.
                let outputs: Vec<_> = outputs.iter().map(otap_pdata_to_message).collect();

                assert_equivalent(&[logs1, logs2], &outputs);

                // Collect telemetry for verify_item_metrics.
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                verify_item_metrics(&telemetry_registry, SignalType::Logs, 6);
            });
    }
}
