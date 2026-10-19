// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Representation-aware batch processor. Batches OtapPdata by a codec-supported
//! size or timer. BatchPlan owns conversion and merge/split operations; this
//! node owns scheduling and delivery completion through explicit input ranges.
//!
//! # Configuration
//!
//! Configuration is modelled on the (original) OpenTelemetry batch
//! processor, not the relatively-new exporterhelper batcher, which
//! supports batching by a sizer "bytes", "requests", or "items".
//!
//! There are two limits, a lower bound (`min_size`) and an upper bound
//! (`max_size`); at least one must be set, and an unset `min_size`
//! defaults to `max_size`. Together they define the range of acceptable
//! batch sizes `[min_size, max_size]` (unbounded above when `max_size` is
//! unset). An arriving input already within that range is forwarded as-is
//! with its original context. All other inputs are buffered and then re-batched
//! and flushed when either pending data reaches the lower bound required to
//! make at least one batch or the max_batch_duration is reached.
//!
//! Because in-range inputs bypass the buffer, output order is not
//! preserved relative to data that is still buffered.
//!
//! # Flush behavior
//!
//! When this component flushes because it has reached a limit then there
//! can be residual data left after the flush. This retained data is always
//! "first in line" for considering in the next flush event and is guaranteed
//! to be flushed in the next batch. The implication of this is that a batch
//! can sit in the batch_processor for up to `2 * max_batch_duration`.
//!
//! # Pipeline Placement
//!
//! This component should be installed before any retry processor
//! (i.e., only retry after batching). This component does not support
//! Interests::RETURN_DATA because (a) more memory required, (b) forces
//! whole-request retry (instead of partial).

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = OTAP_BATCH_PROCESSOR_URN,
    target = "otel.processor.batch",
);

use async_trait::async_trait;
#[cfg(test)]
use bytes::Bytes;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::MessageSourceLocalEffectHandlerExtension;
use otel_arrow_dfe_engine::{
    ConsumerEffectHandlerExtension, Interests, LocalWakeupRequirements,
    ProcessorRuntimeRequirements, ProducerEffectHandlerExtension,
    config::ProcessorConfig,
    control::{AckMsg, CallData, NackMsg, NodeControlMsg, WakeupSlot},
    error::{Error as EngineError, ProcessorErrorKind},
    local::processor as local,
    message::Message,
    node::NodeId,
    processor::{FlowMetricHook, ProcessorWrapper},
};
use otel_arrow_dfe_otap::OTAP_PROCESSOR_FACTORIES;
use otel_arrow_dfe_otap::accessory::slots::{Key as SlotKey, State as SlotState};
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata, PdataEffectHandlerExtension, PeerAddrMerger};
#[cfg(test)]
use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
pub use otel_arrow_dfe_pdata_codec::BatchSizer as Sizer;
use otel_arrow_dfe_pdata_codec::{
    BatchPlan, BatchSettings, BatchingOutput, CodecError, CodecRegistry, OtapPayload,
    PdataEncoding, PdataFormat,
};
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry_macros::metric_set;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::num::{NonZeroU64, NonZeroUsize};
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// URN for the OTAP batch processor
pub const OTAP_BATCH_PROCESSOR_URN: &str = "urn:otel:processor:batch";

/// Default configuration item min-size (OTAP default)
pub const DEFAULT_OTAP_MIN_SIZE_ITEMS: u64 = 8192;

/// Default configuration item min-size (OTLP default)
pub const DEFAULT_OTLP_MIN_SIZE_BYTES: u64 = 262144;

/// Default max batch duration in milliseconds
pub const DEFAULT_MAX_BATCH_DURATION_MS: u64 = 200;

/// Log messages
const LOG_MSG_BATCHING_FAILED_PREFIX: &str = "OTAP batch processor: low-level batching failed for";
const LOG_MSG_BATCHING_FAILED_SUFFIX: &str = "; dropping";

/// Event emitted at build time when a format is configured to forward every
/// input without batching (`min_size: 0` and no `max_size`).
const LOG_EVENT_FORWARDS_EVERYTHING: &str = "batch.config.forwards_everything";

// Each finite buffer owns three signal slots, independent of its codec.
const fn wakeup_slot(buffer: usize, signal: SignalType) -> WakeupSlot {
    let offset = match signal {
        SignalType::Logs => 0,
        SignalType::Metrics => 1,
        SignalType::Traces => 2,
    };
    WakeupSlot(buffer as u128 * 3 + offset)
}

#[cfg(test)]
fn signal_from_wakeup_slot(slot: WakeupSlot) -> Option<(usize, SignalType)> {
    let buffer = usize::try_from(slot.0 / 3).ok()?;
    let signal = match slot.0 % 3 {
        0 => SignalType::Logs,
        1 => SignalType::Metrics,
        _ => SignalType::Traces,
    };
    Some((buffer, signal))
}

/// Batching settings for one representation, identified by its containing config key.
/// The format selects the codec; these settings control sizing and split budgets.
#[derive(Debug, Clone, Deserialize)]
pub struct FormatConfig {
    /// Lower bound of the acceptable batch size. Measured in the quantity
    /// indicated by `sizer`. When pending data reaches this threshold a
    /// new batch will form and be sent. When unset, defaults to `max_size`.
    /// Zero is allowed and means any input up to `max_size` is forwarded as-is.
    pub min_size: Option<u64>,

    /// Optionally limit batch sizes to an upper bound. Measured in
    /// the quantity indicated by `sizer`, as described for min_size.
    /// This is a best-effort split target. Indivisible entries or exhausted
    /// split budgets may require an encoded output larger than this value.
    pub max_size: Option<NonZeroU64>,

    /// The sizer, "requests", "items", or "bytes".  See `Sizer`.
    pub sizer: Sizer,

    /// Maximum number of fragments a single oversize resource entry may be
    /// split into when byte-splitting (OTLP bytes only). If splitting an entry
    /// would exceed this budget, the entry is emitted whole (best-effort,
    /// possibly exceeding `max_size`) and counted in the
    /// `split_budget_fallbacks` metric. `None` means unbounded. This bounds
    /// worst-case fan-out (and memory) when `max_size` is small relative to a
    /// single indivisible input.
    #[serde(default = "default_max_split_fragments")]
    pub max_split_fragments: Option<NonZeroU64>,

    /// Maximum number of duplicated wrapper bytes a single oversize resource
    /// entry may amplify into when byte-splitting (OTLP bytes only). Each
    /// fragment re-encodes the resource/scope headers, so a large header split
    /// across many records can amplify output far beyond the input even when
    /// the fragment count stays under `max_split_fragments`. This is measured
    /// from the actual greedy packing as the entry is split (not projected up
    /// front); if it exceeds this budget the entry is emitted whole
    /// (best-effort, possibly exceeding `max_size`) and counted in the
    /// `split_budget_fallbacks` metric. `None` means unbounded.
    #[serde(default = "default_max_split_overhead_bytes")]
    pub max_split_overhead_bytes: Option<NonZeroU64>,

    /// Maximum number of output batches a single flush may build from byte
    /// splitting (OTLP bytes only) before further oversize resource entries are
    /// emitted whole instead of split. All output batches are held in memory
    /// before any are sent, so a flush containing many large entries -- each
    /// within its own per-entry budgets -- could otherwise amplify into an
    /// unbounded output allocation. This caps the *additional split fan-out*
    /// independently of any downstream Ack/Nack slot accounting; it does not cap
    /// the mandatory floor of one whole output batch per remaining top-level
    /// entry. Over-limit entries are counted in the `split_budget_fallbacks`
    /// metric. `None` means unbounded.
    #[serde(default = "default_max_split_fragments_per_flush")]
    pub max_split_fragments_per_flush: Option<NonZeroU64>,
}

/// Batching format option.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchingFormat {
    /// Force to OTAP.
    Otap,

    /// Force to OTLP.
    Otlp,

    /// Preserve when native batching is supported; otherwise emit OTAP.
    Preserve,

    /// Force output to a registered codec.
    Codec(String),
}

impl<'de> Deserialize<'de> for BatchingFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match String::deserialize(deserializer)?.as_str() {
            "otap" => Self::Otap,
            "otlp" | "otlp-bytes" => Self::Otlp,
            "preserve" => Self::Preserve,
            name => Self::Codec(name.to_owned()),
        })
    }
}

impl Serialize for BatchingFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::Otap => "otap",
            Self::Otlp => "otlp",
            Self::Preserve => "preserve",
            Self::Codec(name) => name,
        })
    }
}

/// Batch processor configuration, with compatible built-in settings aliases.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "ConfigInput")]
pub struct Config {
    /// Native OTAP settings (legacy alias for codecs.otap).
    pub otap: FormatConfig,
    /// OTLP byte settings (legacy alias for codecs.otlp-bytes).
    pub otlp: FormatConfig,
    /// Explicit settings indexed by canonical codec name.
    pub codecs: BTreeMap<String, FormatConfig>,
    /// Maximum age of pending input.
    pub max_batch_duration: Duration,
    /// Per-buffer inbound completion tracking limit.
    pub inbound_request_limit: NonZeroUsize,
    /// Per-buffer outbound completion tracking limit.
    pub outbound_request_limit: NonZeroUsize,
    /// Desired output representation, or native batching with OTAP fallback.
    pub format: BatchingFormat,
}

#[derive(Deserialize)]
struct ConfigInput {
    otap: Option<FormatConfig>,
    otlp: Option<FormatConfig>,
    #[serde(default)]
    codecs: BTreeMap<String, FormatConfig>,
    #[serde(with = "humantime_serde", default = "default_max_batch_duration")]
    max_batch_duration: Duration,
    #[serde(default = "default_inbound_request_limit")]
    inbound_request_limit: NonZeroUsize,
    #[serde(default = "default_outbound_request_limit")]
    outbound_request_limit: NonZeroUsize,
    #[serde(default = "default_batching_format")]
    format: BatchingFormat,
}

impl TryFrom<ConfigInput> for Config {
    type Error = String;
    fn try_from(input: ConfigInput) -> Result<Self, Self::Error> {
        let registry = CodecRegistry::global().map_err(|error| error.to_string())?;
        let otlp = registry
            .resolve(&PdataEncoding::OTLP)
            .map(PdataFormat::encoded)
            .map_err(|error| error.to_string())?;
        let mut codecs = BTreeMap::new();
        for (name, settings) in input.codecs {
            let format =
                PdataFormat::resolve(&registry, &name).map_err(|error| error.to_string())?;
            let canonical = format.name().to_owned();
            if (format == PdataFormat::OTAP && input.otap.is_some())
                || (format == otlp && input.otlp.is_some())
                || codecs.insert(canonical.clone(), settings).is_some()
            {
                return Err(format!("duplicate batching settings for {canonical}"));
            }
        }
        Ok(Self {
            otap: input.otap.unwrap_or_else(default_otap),
            otlp: input.otlp.unwrap_or_else(default_otlp),
            codecs,
            max_batch_duration: input.max_batch_duration,
            inbound_request_limit: input.inbound_request_limit,
            outbound_request_limit: input.outbound_request_limit,
            format: input.format,
        })
    }
}

const fn default_otap_min_size_items() -> Option<u64> {
    Some(DEFAULT_OTAP_MIN_SIZE_ITEMS)
}

const fn default_otap_max_size_items() -> Option<NonZeroU64> {
    // No upper-bound
    None
}

const fn default_otap_sizer_items() -> Sizer {
    Sizer::Items
}

const fn default_otlp_min_size_bytes() -> Option<u64> {
    Some(DEFAULT_OTLP_MIN_SIZE_BYTES)
}

const fn default_otlp_max_size_bytes() -> Option<NonZeroU64> {
    // No upper-bound
    None
}

const fn default_otlp_sizer_bytes() -> Sizer {
    Sizer::Bytes
}

const fn default_max_batch_duration() -> Duration {
    Duration::from_millis(DEFAULT_MAX_BATCH_DURATION_MS)
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

/// Default cap on the number of fragments a single oversize resource entry may
/// split into. Bounds worst-case fan-out when `max_size` is tiny relative to an
/// indivisible input; entries projected to exceed this are emitted whole.
///
/// This is a pathological-fan-out backstop, not a tuning knob: it is meant to
/// almost never fire under normal load and is deliberately set well above the
/// fan-out any legitimate large resource entry produces. Chosen as a round
/// power of two (2^16) rather than a hard performance limit.
const fn default_max_split_fragments() -> Option<NonZeroU64> {
    NonZeroU64::new(65_536)
}

/// Default cap on the duplicated wrapper bytes a single oversize resource entry
/// may amplify into when splitting. Bounds worst-case output-byte amplification
/// from re-encoding large resource/scope headers across many fragments; entries
/// whose amplification (measured during greedy packing) exceeds this are emitted
/// whole. Default: 8 MiB.
const fn default_max_split_overhead_bytes() -> Option<NonZeroU64> {
    NonZeroU64::new(8 * 1024 * 1024)
}

/// Default cap on the number of output batches a single flush may build from
/// byte splitting before further oversize entries are emitted whole. Bounds the
/// in-memory output allocation for a flush with many large entries, independent
/// of Ack/Nack slot accounting; over-limit entries are emitted whole. Chosen as
/// a round power of two (2^16) backstop, not a hard performance limit.
const fn default_max_split_fragments_per_flush() -> Option<NonZeroU64> {
    NonZeroU64::new(65_536)
}

const fn default_otap() -> FormatConfig {
    FormatConfig {
        min_size: default_otap_min_size_items(),
        max_size: default_otap_max_size_items(),
        sizer: default_otap_sizer_items(),
        max_split_fragments: None,
        max_split_overhead_bytes: None,
        max_split_fragments_per_flush: None,
    }
}

const fn default_otlp() -> FormatConfig {
    FormatConfig {
        min_size: default_otlp_min_size_bytes(),
        max_size: default_otlp_max_size_bytes(),
        sizer: default_otlp_sizer_bytes(),
        max_split_fragments: default_max_split_fragments(),
        max_split_overhead_bytes: default_max_split_overhead_bytes(),
        max_split_fragments_per_flush: default_max_split_fragments_per_flush(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            otap: default_otap(),
            otlp: default_otlp(),
            max_batch_duration: default_max_batch_duration(),
            inbound_request_limit: default_inbound_request_limit(),
            outbound_request_limit: default_outbound_request_limit(),
            format: default_batching_format(),
            codecs: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Validates all active batching plans and their capabilities.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.plans().map(|_| ())
    }

    fn settings(&self, format: PdataFormat) -> BatchSettings {
        if let Some(settings) = self.codecs.get(format.name()) {
            return settings.into();
        }
        if format == PdataFormat::OTAP {
            return (&self.otap).into();
        }
        if format
            .codec()
            .is_some_and(|codec| codec.encoding() == &PdataEncoding::OTLP)
        {
            return (&self.otlp).into();
        }
        let mut default = format.default_settings();
        // A codec without native support uses the configured OTAP fallback settings.
        if !format.has_native_batching() {
            self.codecs.get("otap").unwrap_or(&self.otap).into()
        } else {
            // Adding a compiled-in codec must not invalidate existing immediate
            // flush configurations. Implicit settings use the native threshold
            // as a split limit; explicit settings still require valid split settings.
            if self.max_batch_duration == Duration::ZERO
                && default.min_size.is_some_and(|min| min > 0)
            {
                default.max_size = default
                    .max_size
                    .or(default.min_size.and_then(NonZeroU64::new));
                default.min_size = None;
            }
            default
        }
    }

    fn plans(&self) -> Result<Vec<(PdataFormat, BatchPlan)>, ConfigError> {
        let registry = CodecRegistry::global().map_err(config_error)?;
        let otlp = registry
            .resolve(&PdataEncoding::OTLP)
            .map(PdataFormat::encoded)
            .map_err(config_error)?;
        let formats = if self.format == BatchingFormat::Preserve {
            let mut formats = vec![PdataFormat::OTAP, otlp];
            let mut extra: Vec<_> = registry
                .codecs()
                .filter(|codec| codec.can_decode())
                .map(PdataFormat::encoded)
                .filter(|format| *format != otlp)
                .collect();
            extra.sort_by_key(|format| format.name());
            formats.extend(extra);
            formats
        } else {
            vec![match &self.format {
                BatchingFormat::Otap => PdataFormat::OTAP,
                BatchingFormat::Otlp => otlp,
                BatchingFormat::Codec(name) => {
                    PdataFormat::resolve(&registry, name).map_err(config_error)?
                }
                BatchingFormat::Preserve => unreachable!(),
            }]
        };
        formats
            .into_iter()
            .map(|format| {
                let settings = self.settings(format);
                if self.max_batch_duration == Duration::ZERO
                    && settings.min_size.is_some_and(|min| min > 0)
                {
                    return Err(config_error(
                        "min_size > 0 requires max_batch_duration is set",
                    ));
                }
                BatchPlan::new(format, settings, self.format == BatchingFormat::Preserve)
                    .map(|plan| (format, plan))
                    .map_err(config_error)
            })
            .collect()
    }
}

fn config_error(error: impl std::fmt::Display) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.to_string(),
    }
}

impl From<&FormatConfig> for BatchSettings {
    fn from(settings: &FormatConfig) -> Self {
        Self {
            min_size: settings.min_size,
            max_size: settings.max_size,
            sizer: settings.sizer,
            max_split_fragments: settings.max_split_fragments,
            max_split_overhead_bytes: settings.max_split_overhead_bytes,
            max_split_fragments_per_flush: settings.max_split_fragments_per_flush,
        }
    }
}

impl FormatConfig {
    #[cfg(test)]
    const fn new_items(min_size: Option<u64>, max_size: u64) -> FormatConfig {
        FormatConfig {
            min_size,
            max_size: NonZeroU64::new(max_size),
            sizer: Sizer::Items,
            max_split_fragments: None,
            max_split_overhead_bytes: None,
            max_split_fragments_per_flush: None,
        }
    }

    #[cfg(test)]
    const fn new_bytes(min_size: Option<u64>, max_size: u64) -> FormatConfig {
        FormatConfig {
            min_size,
            max_size: NonZeroU64::new(max_size),
            sizer: Sizer::Bytes,
            max_split_fragments: default_max_split_fragments(),
            max_split_overhead_bytes: default_max_split_overhead_bytes(),
            max_split_fragments_per_flush: default_max_split_fragments_per_flush(),
        }
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
    /// Pending work plus emitted outputs that must finish before Ack.
    outbound: usize,
}

/// Delivery owner of one buffered input, independent of its measured size.
#[derive(Clone, Copy)]
struct BatchPortion {
    /// Slot identifying the original input whose completion must wait for this work.
    inkey: Option<SlotKey>,
    /// Peer address observed by the receiving socket for the input that this
    /// portion came from. Retained on every portion (whether or not the input
    /// requested ack/nack) so that a merged output batch's `peer_addr` can be
    /// computed even when the contributing inputs had no subscribers.
    peer_addr: Option<SocketAddr>,
}

#[derive(Default)]
struct Inputs {
    /// Input batches.
    pending: Vec<OtapPayload>,

    /// Waiter context
    context: Vec<BatchPortion>,

    /// Sum of exact input measurements used only to trigger flushing.
    size: usize,
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

    /// Whether the local scheduler currently has a live wakeup for this signal.
    wakeup_armed: bool,
    slot: WakeupSlot,
}

/// Local (!Send) batch processor
pub struct BatchProcessor {
    config: Config,
    formats: Vec<Option<FormatState>>,
    routes: Vec<(PdataFormat, usize)>,
    metrics: MetricSet<BatchProcessorMetrics>,
}

struct FormatState {
    plan: BatchPlan,
    signals: SignalBatches,
}

struct BatchProcessorFormat<'a> {
    config: &'a Config,
    plan: &'a BatchPlan,
    signals: &'a mut SignalBatches,
    metrics: &'a mut MetricSet<BatchProcessorMetrics>,
}

struct BatchProcessorSignal<'a> {
    signal: SignalType,
    config: &'a Config,
    fmtcfg: &'a BatchSettings,
    plan: &'a BatchPlan,
    buffer: &'a mut SignalBuffer,
    metrics: &'a mut MetricSet<BatchProcessorMetrics>,
}

/// Failure classification retained while unwinding all contributing input owners.
/// Codec conversion failures are permanent; downstream Nacks keep their classification.
enum BatchFailure {
    Retryable(String),
    Permanent(String),
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
    /// Number of flushes triggered by size threshold (all signals)
    #[metric(unit = "{flush}")]
    flushes_size: Counter<u64>,
    /// Number of flushes triggered by timer (all signals)
    #[metric(unit = "{flush}")]
    flushes_timer: Counter<u64>,
    /// Number of inputs forwarded as-is, without re-batching or completion
    /// tracking, because their size was already within the configured
    /// [min_size, max_size] range on arrival.
    #[metric(unit = "{batch}")]
    passthrough_batches: Counter<u64>,

    /// Number of input requests pending at flush time
    #[metric(unit = "{request}")]
    flush_pending_requests: Mmsc,
    /// Number of bytes pending at flush time when byte size is known
    #[metric(unit = "By")]
    flush_pending_bytes: Mmsc,
    /// Time from first pending input arrival to actual flush start
    #[metric(unit = "ns")]
    flush_age_duration: Mmsc,
    /// Delay between scheduled timer wakeup and actual timer flush start
    #[metric(unit = "ns")]
    flush_timer_lateness_duration: Mmsc,
    /// Number of output batches successfully handed downstream by each flush
    #[metric(unit = "{batch}")]
    flush_output_batches: Mmsc,
    /// Final-representation bytes successfully handed downstream by each flush,
    /// when all output sizes are known; excludes transport compression and framing
    #[metric(unit = "By")]
    flush_output_bytes: Mmsc,

    /// Number of messages dropped due to conversion failures
    #[metric(unit = "{msg}")]
    dropped_conversion: Counter<u64>,
    /// Number of batches for which errors encountered
    #[metric(unit = "{error}")]
    batching_errors: Counter<u64>,
    /// Number of requests nacked due to inbound slot exhaustion
    #[metric(unit = "{msg}")]
    nacked_inbound_slots: Counter<u64>,
    /// Number of requests nacked due to inbound slot exhaustion
    #[metric(unit = "{msg}")]
    nacked_outbound_slots: Counter<u64>,

    /// Number of oversize resource entries emitted whole (best-effort) because
    /// splitting them would have exceeded the configured fragment budget.
    #[metric(unit = "{entry}")]
    split_budget_fallbacks: Counter<u64>,
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
    fn local_wakeup_requirements(&self) -> LocalWakeupRequirements {
        LocalWakeupRequirements::new(self.formats.iter().flatten().count() * 3)
    }

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

        let plans = config.plans()?;
        let mut formats: Vec<Option<FormatState>> = vec![None, None];
        let mut routes = Vec::new();
        for (source, plan) in plans {
            if plan.settings().forwards_everything() {
                otel_warn!(
                    LOG_EVENT_FORWARDS_EVERYTHING,
                    format = source.name(),
                    message = "min_size is 0 and max_size is unset: every input is \
                        forwarded as-is and no batching is performed"
                );
            }
            let existing = formats
                .iter()
                .position(|state| state.as_ref().is_some_and(|state| state.plan == plan));
            let index = existing.unwrap_or_else(|| {
                if source == PdataFormat::OTAP {
                    0
                } else if source
                    .codec()
                    .is_some_and(|codec| codec.encoding() == &PdataEncoding::OTLP)
                {
                    1
                } else {
                    formats.len()
                }
            });
            if index == formats.len() {
                formats.push(None);
            }
            if formats[index].is_none() {
                formats[index] = Some(FormatState {
                    plan,
                    signals: SignalBatches::new(&config, index),
                });
            }
            routes.push((source, index));
        }
        Ok(Self {
            config,
            formats,
            routes,
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

    /// Flushes each registered buffer at shutdown.
    async fn flush_shutdown(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        for index in 0..self.formats.len() {
            for signal in [SignalType::Logs, SignalType::Traces, SignalType::Metrics] {
                if let Some(mut format) = self.format(index) {
                    format
                        .for_signal(signal)
                        .flush_signal_impl(effect, Instant::now(), FlushReason::Shutdown)
                        .await?;
                }
            }
        }
        Ok(())
    }

    fn format(&mut self, index: usize) -> Option<BatchProcessorFormat<'_>> {
        self.formats
            .get_mut(index)?
            .as_mut()
            .map(|state| BatchProcessorFormat {
                config: &self.config,
                plan: &state.plan,
                signals: &mut state.signals,
                metrics: &mut self.metrics,
            })
    }

    /// The pdata plan owns preparation; the node owns scheduling and delivery.
    async fn process_signal_impl(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        request: OtapPdata,
    ) -> Result<(), EngineError> {
        let signal = request.signal_type();
        let (ctx, mut payload) = request.into_parts();
        let index = if self.config.format == BatchingFormat::Preserve {
            self.routes
                .iter()
                .find(|(source, _)| *source == payload.format())
                .map(|(_, index)| *index)
        } else {
            self.routes.first().map(|(_, index)| *index)
        };
        let Some(index) = index else {
            effect
                .notify_nack(NackMsg::new_permanent(
                    "no batching plan for admitted codec",
                    OtapPdata::new(ctx, payload),
                ))
                .await?;
            return Ok(());
        };
        // SAFETY: routes only reference initialized format states created in
        // from_config, and neither collection is modified after construction.
        let mut format = self.format(index).expect("configured route");
        if let Err(error) = effect.prepare_batch(format.plan, &mut payload).await {
            format.metrics.dropped_conversion.inc();
            effect
                .notify_nack(NackMsg::new_permanent(
                    error.to_string(),
                    OtapPdata::new(ctx, payload),
                ))
                .await?;
            return Ok(());
        }
        format
            .for_signal(signal)
            .accept_payload(effect, ctx, payload)
            .await
    }
}

impl<'a> BatchProcessorFormat<'a> {
    const fn for_signal(&mut self, signal: SignalType) -> BatchProcessorSignal<'_> {
        BatchProcessorSignal {
            signal,
            config: self.config,
            fmtcfg: self.plan.settings(),
            plan: self.plan,
            buffer: match signal {
                SignalType::Logs => &mut self.signals.logs,
                SignalType::Traces => &mut self.signals.traces,
                SignalType::Metrics => &mut self.signals.metrics,
            },
            metrics: self.metrics,
        }
    }
}

impl<'a> BatchProcessorSignal<'a> {
    async fn accept_payload(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        ctx: Context,
        payload: OtapPayload,
    ) -> Result<(), EngineError> {
        let size = self.fmtcfg.sizer.batch_size(&payload)?;
        if size == 0 {
            // Note: we do not check for empty envelopes, e.g., logs
            // requests with only a resource and no log records. We do
            // not count these.
            let mut pdata = OtapPdata::new(ctx, payload);
            pdata.complete_processor_without_output(effect);
            effect.notify_ack(AckMsg::new(pdata)).await?;
            return Ok(());
        }

        // Pass-through: an input that is already an acceptable batch is
        // forwarded as-is with its original context.
        if self.fmtcfg.in_passthrough_range(size) {
            return self.forward_as_is(effect, ctx, payload).await;
        }

        // Capture the receiver-observed peer address before `ctx` may be
        // moved into BatchContext below. The portion retains it so the
        // flush path can merge peer_addr across contributing inputs even
        // when none of them subscribed to ack/nack.
        let peer_addr = ctx.peer_addr();

        // Retain contexts needed for Ack/Nack routing or metrics unwinding.
        let inkey = if ctx.needs_completion_tracking() {
            let slot = self
                .buffer
                .inbound
                .allocate_with_data(BatchContext { ctx, outbound: 1 });

            match slot {
                Err(bctx) => {
                    self.metrics.nacked_inbound_slots.inc();
                    let refused = OtapPdata::new(bctx.ctx, payload);
                    // Note: Failure to Ack/Nack is an engine-level error.
                    effect
                        .notify_nack(NackMsg::new("inbound routes exhausted", refused))
                        .await?;
                    // Note: Inbound slot exhaustion leads to dropping
                    // the request. This is by choice, and we expect
                    // the caller to retry.
                    return Ok(());
                }
                Ok(bc) => Some(bc),
            }
        } else {
            None
        };

        // Record the arrival time when the current input is empty. Defer
        // scheduling the wakeup until after measuring the accepted payload, so
        // one-request size flushes do not pay set/cancel timer overhead.
        let timeout = self.config.max_batch_duration;
        let arrival = if timeout != Duration::ZERO && self.buffer.inputs.is_empty() {
            Some(Instant::now())
        } else {
            None
        };

        self.buffer
            .inputs
            .accept(payload, BatchPortion::new(inkey, peer_addr), size);

        let pending_size = self.buffer.inputs.size_by(self.fmtcfg.sizer)?;

        // Flush based on size when the batch reaches the lower limit.
        if timeout != Duration::ZERO && (pending_size as u64) < self.fmtcfg.lower_limit() {
            if let Some(now) = arrival {
                self.buffer
                    .set_arrival(self.signal, now, timeout, effect)
                    .await?;
            }
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

    /// Sends a whole input in the requested output format with its original context.
    /// Matching formats keep their original storage; OTAP fallback plans finish encoding.
    /// Downstream Ack/Nack route directly to the input's subscribers, so no
    /// inbound or outbound slot is used.
    async fn forward_as_is(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        ctx: Context,
        mut payload: OtapPayload,
    ) -> Result<(), EngineError> {
        if let Err(error) = effect.finish_batch(self.plan, &mut payload).await {
            self.metrics.dropped_conversion.inc();
            effect
                .notify_nack(NackMsg::new_permanent(
                    error.to_string(),
                    OtapPdata::new(ctx, payload),
                ))
                .await?;
            return Ok(());
        }
        self.metrics.passthrough_batches.inc();
        let pdata = OtapPdata::new(ctx, payload);
        effect.send_message_with_source_node(pdata).await?;
        Ok(())
    }

    /// Flushes all of the input, merging and splitting as necessary to
    /// respect the optional limit. Reason-specific behavior:
    ///
    /// - If Timer and it is arriving early, ignore. This happens because
    ///   timers are not canceled.
    /// - If Size, a nonempty final output below the lower bound can be retained
    ///   as first-in-line when it belongs only to the last buffered input.
    ///   Multi-input tails are sent now to preserve unambiguous ownership.
    async fn flush_signal_impl(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        now: Instant,
        reason: FlushReason,
    ) -> Result<(), EngineError> {
        // Timer wakeups are already popped from the local scheduler before
        // delivery. Only size/shutdown flushes need to cancel an armed timer.
        if reason == FlushReason::Timer {
            self.buffer.wakeup_armed = false;
        }

        // If the input is empty.
        if self.buffer.inputs.is_empty() {
            return Ok(());
        }

        if reason != FlushReason::Timer && self.buffer.wakeup_armed {
            let _ = effect.cancel_wakeup(self.buffer.slot);
            self.buffer.wakeup_armed = false;
        }

        // If this is a timer-based flush and we were called too soon,
        // skip. this may happen if the batch for which the timer was set
        // flushes for size before the timer.
        if reason == FlushReason::Timer
            && self.config.max_batch_duration != Duration::ZERO
            && self
                .buffer
                .arrival
                .is_some_and(|arrival| now.duration_since(arrival) < self.config.max_batch_duration)
        {
            return Ok(());
        }

        let flush_started = Instant::now();
        self.metrics
            .flush_pending_requests
            .record(self.buffer.inputs.requests() as f64);
        if let Some(bytes) = self.buffer.inputs.known_bytes() {
            self.metrics.flush_pending_bytes.record(bytes as f64);
        }
        if let Some(arrival) = self.buffer.arrival {
            self.metrics
                .flush_age_duration
                .record(flush_started.duration_since(arrival).as_nanos() as f64);
        }
        if reason == FlushReason::Timer {
            self.metrics
                .flush_timer_lateness_duration
                .record(flush_started.saturating_duration_since(now).as_nanos() as f64);
        }

        let mut inputs = self.buffer.inputs.drain();

        match reason {
            FlushReason::Size => self.metrics.flushes_size.inc(),
            FlushReason::Timer => self.metrics.flushes_timer.inc(),
            FlushReason::Shutdown => {}
        }

        let count = inputs.requests();

        let pending = inputs.take_pending();

        let BatchingOutput {
            batches: mut output_batches,
            completed_inputs: _,
            budget_fallbacks,
        } = match effect.batch(self.plan, self.signal, pending).await {
            Ok(v) => v,
            Err(e) => {
                self.metrics.batching_errors.add(count as u64);
                log_batching_failed(effect, self.signal, &e).await;
                let res = Err(BatchFailure::Retryable(e.to_string()));
                // In this case, we are sending failure to all the pending inputs.
                self.buffer
                    .handle_partial_responses(self.signal, effect, &res, inputs.context)
                    .await?;
                return Ok(());
            }
        };

        if budget_fallbacks > 0 {
            self.metrics.split_budget_fallbacks.add(budget_fallbacks);
        }

        // Establish every output reference before releasing any pending input
        // or awaiting an operation that can complete/fail. Ranges, not sizes,
        // determine which original requests must wait for each output.
        for (_, range) in &output_batches {
            for part in &inputs.context[range.clone()] {
                if let Some(key) = part.inkey
                    && let Some(input) = self.buffer.inbound.get_mut(key)
                {
                    input.outbound += 1;
                }
            }
        }

        // Retain only a remainder with one original delivery owner. A range
        // spanning inputs is sent now through normal multi-owner tracking;
        // retaining and re-splitting it would require finer provenance. Require
        // the last input so older residuals cannot repeatedly reset their timer.
        let mut retained_partial = false;
        if self.config.max_batch_duration != Duration::ZERO
            && reason == FlushReason::Size
            && self.fmtcfg.max_size.is_some()
            && output_batches.len() > 1
        {
            // SAFETY: the length check above guarantees a last output.
            let (last_payload, range) = output_batches.last().expect("last output");
            if range.start() == range.end() && *range.end() == inputs.context.len() - 1 {
                let size = self.fmtcfg.sizer.batch_size(last_payload)?;
                if size > 0 && (size as u64) < self.fmtcfg.lower_limit() {
                    self.buffer
                        .take_remaining(size, &inputs, &mut output_batches);
                    self.buffer
                        .set_arrival(self.signal, now, self.config.max_batch_duration, effect)
                        .await?;
                    retained_partial = true;
                }
            }
        }

        // Each buffered input owns one pending reference. Outputs (including
        // a retained remainder) now hold their own references. Inputs explicitly
        // completed without output have no other reference and Ack here.
        self.buffer
            .handle_partial_responses(self.signal, effect, &Ok(()), inputs.context.iter().copied())
            .await?;

        let mut emitted_batches = 0usize;
        let mut emitted_bytes = Some(0usize);

        // Record one observation per flush, including any successful outputs
        // before an encoding, completion, or downstream handoff failure.
        let result: Result<(), EngineError> = async {
            for (mut records, range) in output_batches {
                let (routed_ctxs, merged_peer) = self.buffer.output_context(range, &inputs.context);
                if let Err(error) = effect.finish_batch(self.plan, &mut records).await {
                    self.metrics.dropped_conversion.inc();
                    if let Some(parts) = routed_ctxs {
                        self.buffer
                            .handle_partial_responses(
                                self.signal,
                                effect,
                                &Err(BatchFailure::Permanent(error.to_string())),
                                parts,
                            )
                            .await?;
                    }
                    continue;
                }
                let bytes = records.measured_bytes();
                let mut pdata = OtapPdata::new(Context::default(), records);

                // Forward the receiver-observed peer address only when every
                // input merged into this output batch came from the same peer
                // (see Context::merge_peer_addr). Mixed-peer batches leave
                // peer_addr as None to avoid misattribution.
                if let Some(addr) = merged_peer {
                    pdata.set_peer_addr(addr);
                }
                if let Some(ctxs) = routed_ctxs {
                    match self.buffer.outbound.allocate_with_data(ctxs) {
                        Err(ctxs) => {
                            for bp in ctxs {
                                if let Some(inkey) = bp.inkey
                                    && let Some(batch) = self.buffer.inbound.take(inkey)
                                {
                                    self.metrics.nacked_outbound_slots.inc();
                                    // Note: Failure to Ack/Nack is an engine-level error.
                                    effect
                                        .notify_nack(NackMsg::new(
                                            "outbound routes exhausted",
                                            OtapPdata::new(
                                                batch.ctx,
                                                OtapPayload::empty(self.signal),
                                            ),
                                        ))
                                        .await?;
                                    // Note: failure to get an outbound slot does not
                                    // stop the outbound request, since it can contain data
                                    // that did not request Ack/Nack.
                                }
                            }
                        }
                        Ok(outkey) => {
                            let mut calldata: CallData = outkey.into();
                            calldata.push((self.buffer.slot.0 as u64).into());
                            effect.subscribe_to(
                                Interests::NACKS | Interests::ACKS,
                                calldata,
                                &mut pdata,
                            );
                        }
                    };
                }

                effect.send_message_with_source_node(pdata).await?;
                emitted_batches += 1;
                emitted_bytes = emitted_bytes
                    .zip(bytes)
                    .and_then(|(total, bytes)| total.checked_add(bytes));
            }
            Ok(())
        }
        .await;

        self.metrics
            .flush_output_batches
            .record(emitted_batches as f64);
        if let Some(bytes) = emitted_bytes {
            self.metrics.flush_output_bytes.record(bytes as f64);
        }
        result?;

        if !retained_partial {
            self.buffer.arrival = None;
            self.buffer.wakeup_armed = false;
        }

        Ok(())
    }

    async fn handle(
        &mut self,
        signal: SignalType,
        calldata: CallData,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), BatchFailure>,
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
        self.handle_response(*ack.accepted, ack.unwind.route.calldata, effect, &Ok(()))
            .await
    }

    async fn handle_nack(
        &mut self,
        effect: &mut local::EffectHandler<OtapPdata>,
        nack: NackMsg<OtapPdata>,
    ) -> Result<(), EngineError> {
        let res = Err(if nack.permanent {
            BatchFailure::Permanent(nack.reason)
        } else {
            BatchFailure::Retryable(nack.reason)
        });
        self.handle_response(*nack.refused, nack.unwind.route.calldata, effect, &res)
            .await
    }

    async fn handle_response(
        &mut self,
        retdata: OtapPdata,
        calldata: CallData,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), BatchFailure>,
    ) -> Result<(), EngineError> {
        if calldata.is_empty() {
            return Ok(());
        }

        let mut calldata = calldata;
        if calldata.len() != 2 {
            return Err(EngineError::InternalError {
                message: "invalid batch completion token".into(),
            });
        }
        // SAFETY: the length check above guarantees two elements, and calldata
        // has not been mutated since, so pop() must return Some.
        let slot = u64::from(calldata.pop().expect("checked token"));
        let index = (slot / 3) as usize;
        let signal = match slot % 3 {
            0 => SignalType::Logs,
            1 => SignalType::Metrics,
            _ => SignalType::Traces,
        };
        let _ = retdata;
        let mut format = self
            .format(index)
            .ok_or_else(|| EngineError::InternalError {
                message: "unknown batch completion buffer".into(),
            })?;
        format
            .for_signal(signal)
            .handle(signal, calldata, effect, res)
            .await
    }
}

/// Factory function to create a batch processor.
pub fn create_otap_batch_processor(
    pipeline_ctx: otel_arrow_dfe_engine::context::PipelineContext,
    node: NodeId,
    node_config: Arc<NodeUserConfig>,
    processor_config: &ProcessorConfig,
) -> Result<ProcessorWrapper<OtapPdata>, ConfigError> {
    let metrics = BatchProcessorMetrics::register(&pipeline_ctx);
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
    fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        ProcessorRuntimeRequirements {
            local_wakeups: Some(self.local_wakeup_requirements()),
            ..ProcessorRuntimeRequirements::none()
        }
    }

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
                } => {
                    effect
                        .report_local_scheduler_metrics(&mut metrics_reporter)
                        .map_err(|e| EngineError::InternalError {
                            message: e.to_string(),
                        })?;
                    metrics_reporter.report(&mut self.metrics).map_err(|e| {
                        EngineError::InternalError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(())
                }
                NodeControlMsg::Wakeup { slot, when, .. } => {
                    let Ok(index) = usize::try_from(slot.0 / 3) else {
                        return Ok(());
                    };
                    let signal = match slot.0 % 3 {
                        0 => SignalType::Logs,
                        1 => SignalType::Metrics,
                        _ => SignalType::Traces,
                    };
                    if let Some(mut format) = self.format(index) {
                        format
                            .for_signal(signal)
                            .flush_signal_impl(effect, when, FlushReason::Timer)
                            .await?;
                    }

                    Ok(())
                }
                NodeControlMsg::ResumeData { .. } => Ok(()),
                NodeControlMsg::Ack(ack) => self.handle_ack(effect, ack).await,
                NodeControlMsg::Nack(nack) => self.handle_nack(effect, nack).await,
                NodeControlMsg::DrainIngress { .. } => Ok(()),
                NodeControlMsg::TimerTick { .. } => unreachable!(),
                NodeControlMsg::MemoryPressureChanged { .. } => Ok(()),
            },
            Message::PData(request) => self.process_signal_impl(effect, request).await,
        }
    }
}

impl SignalBatches {
    fn new(config: &Config, index: usize) -> Self {
        Self {
            logs: SignalBuffer::new(config, wakeup_slot(index, SignalType::Logs)),
            traces: SignalBuffer::new(config, wakeup_slot(index, SignalType::Traces)),
            metrics: SignalBuffer::new(config, wakeup_slot(index, SignalType::Metrics)),
        }
    }
}

impl BatchPortion {
    const fn new(inkey: Option<SlotKey>, peer_addr: Option<SocketAddr>) -> Self {
        Self { inkey, peer_addr }
    }
}

impl Inputs {
    fn drain(&mut self) -> Self {
        Self {
            pending: std::mem::take(&mut self.pending),
            context: std::mem::take(&mut self.context),
            size: std::mem::take(&mut self.size),
        }
    }

    const fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    const fn requests(&self) -> usize {
        self.pending.len()
    }

    fn known_bytes(&self) -> Option<usize> {
        known_total_bytes(&self.pending)
    }

    fn size_by(&self, sizer: Sizer) -> Result<usize, CodecError> {
        match sizer {
            Sizer::Requests => Ok(self.requests()),
            // These measurements trigger flushing; they never assign ownership.
            Sizer::Items | Sizer::Bytes => Ok(self.size),
        }
    }

    fn accept(&mut self, batch: OtapPayload, part: BatchPortion, size: usize) {
        self.size += size;
        self.pending.push(batch);
        self.context.push(part);
    }

    fn take_pending(&mut self) -> Vec<OtapPayload> {
        std::mem::take(&mut self.pending)
    }
}

fn known_total_bytes(payloads: &[OtapPayload]) -> Option<usize> {
    payloads.iter().try_fold(0usize, |total, payload| {
        payload.measured_bytes().map(|bytes| total + bytes)
    })
}

impl SignalBuffer {
    fn new(cfg: &Config, slot: WakeupSlot) -> Self {
        Self {
            inputs: Inputs::default(),
            inbound: SlotState::new(cfg.inbound_request_limit.get()),
            outbound: SlotState::new(cfg.outbound_request_limit.get()),
            arrival: None,
            wakeup_armed: false,
            slot,
        }
    }

    /// Moves a single-owner remainder back into pending storage. Its output
    /// reference becomes the pending reference for the next flush.
    fn take_remaining(
        &mut self,
        size: usize,
        from_inputs: &Inputs,
        output_batches: &mut Vec<(OtapPayload, RangeInclusive<usize>)>,
    ) {
        // SAFETY: the caller checked that the last output exists and references
        // exactly one input; BatchPlan validated that this index is in bounds.
        let (remaining, range) = output_batches.pop().expect("checked remainder");
        let part = from_inputs.context[*range.start()];
        self.inputs.accept(remaining, part, size);
    }

    /// Resolves a validated input range into delivery owners and a merged peer.
    /// Reference counts were established for all outputs before any completion.
    fn output_context(
        &mut self,
        range: RangeInclusive<usize>,
        contexts: &[BatchPortion],
    ) -> (Option<Vec<BatchPortion>>, Option<SocketAddr>) {
        let mut out = Vec::new();
        let mut peer_merger = PeerAddrMerger::new();
        for part in &contexts[range] {
            peer_merger.push(part.peer_addr);
            if let Some(key) = part.inkey
                && self.inbound.get_mut(key).is_some()
            {
                out.push(*part);
            }
        }
        ((!out.is_empty()).then_some(out), peer_merger.finish())
    }

    /// Handles a response, returning an Ack or Nack conditionally when
    /// the outcome is known. This implementation will return as soon as the
    /// first error is received, or it will return when every outbound context
    /// succeeds.
    async fn handle_partial_responses(
        &mut self,
        signal: SignalType,
        effect: &mut local::EffectHandler<OtapPdata>,
        res: &Result<(), BatchFailure>,
        parts: impl IntoIterator<Item = BatchPortion>,
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
                        let nack = match err {
                            BatchFailure::Retryable(reason) => NackMsg::new(reason, rdata),
                            BatchFailure::Permanent(reason) => {
                                NackMsg::new_permanent(reason, rdata)
                            }
                        };
                        effect.notify_nack(nack).await?;
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
        _signal: SignalType,
        now: Instant,
        timeout: Duration,
        effect: &mut local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        effect
            .set_wakeup(self.slot, now + timeout)
            .map(|_| {
                self.arrival = Some(now);
                self.wakeup_armed = true;
            })
            .map_err(|_| EngineError::ProcessorError {
                processor: effect.processor_id(),
                kind: ProcessorErrorKind::Other,
                error: "could not set wakeup".into(),
                source_detail: "".into(),
            })
    }
}

/// Register factory for OTAP batch processor
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Processor)]
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static OTAP_BATCH_PROCESSOR_FACTORY: otel_arrow_dfe_engine::ProcessorFactory<OtapPdata> =
    otel_arrow_dfe_engine::ProcessorFactory {
        name: OTAP_BATCH_PROCESSOR_URN,
        create:
            |pipeline_ctx: otel_arrow_dfe_engine::context::PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             proc_cfg: &ProcessorConfig,
             _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
                create_otap_batch_processor(pipeline_ctx, node, node_config, proc_cfg)
            },
        context_declarations: None,
        wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<Config>,
    };

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_config::{PipelineGroupId, PipelineId};
    use otel_arrow_dfe_engine::config::ProcessorConfig;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::control::{
        NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
        runtime_ctrl_msg_channel,
    };
    use otel_arrow_dfe_engine::message::Message;
    use otel_arrow_dfe_engine::node::Node;
    use otel_arrow_dfe_engine::testing::liveness::next_completion;
    use otel_arrow_dfe_engine::testing::processor::TestRuntime;
    use otel_arrow_dfe_engine::testing::test_node;
    use otel_arrow_dfe_otap::pdata::OtapPdata;
    use otel_arrow_dfe_otap::testing::TestCallData;
    use otel_arrow_dfe_otap::testing::{next_ack, next_nack};
    use otel_arrow_dfe_pdata::encode::{encode_logs_otap_batch, encode_spans_otap_batch};
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::InstrumentationScope;
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, LogsData, ResourceLogs, ScopeLogs,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span, TracesData,
    };
    use otel_arrow_dfe_pdata::testing::equiv::assert_equivalent;
    use otel_arrow_dfe_pdata::testing::fixtures::DataGenerator;
    use otel_arrow_dfe_pdata::testing::round_trip::{
        otap_to_otlp, otlp_message_to_bytes, otlp_to_otap,
    };
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    const FAILING_OUTPUT_ENCODING: PdataEncoding = PdataEncoding::new("test-failing-output");
    static FAILING_OUTPUT_METADATA: otel_arrow_dfe_pdata_codec::CodecMetadata =
        otel_arrow_dfe_pdata_codec::CodecMetadata::new(
            FAILING_OUTPUT_ENCODING,
            &[SignalType::Logs],
        );

    struct FailingOutputEncoder;

    impl otel_arrow_dfe_pdata_codec::PdataEncoder for FailingOutputEncoder {
        fn encode(
            &mut self,
            records: otel_arrow_dfe_pdata::OtapArrowRecords,
        ) -> Result<Bytes, CodecError> {
            // A one-record output succeeds so tests can exercise a flush with
            // both rejected and successfully encoded batches.
            if records.num_items() == 1 {
                return Ok(Bytes::from_static(b"encoded log"));
            }
            Err(CodecError::operation(
                &FAILING_OUTPUT_ENCODING,
                otel_arrow_dfe_pdata_codec::CodecOperation::Encode,
                std::io::Error::other("unencodable test payload"),
            ))
        }
    }

    otel_arrow_dfe_pdata_codec::register_pdata_codec!(
        FAILING_OUTPUT_CODEC,
        otel_arrow_dfe_pdata_codec::CodecRegistration::new(&FAILING_OUTPUT_METADATA)
            .with_encoder(|_| Ok(Box::new(FailingOutputEncoder))),
    );

    const PAGE_ENCODING: PdataEncoding = PdataEncoding::new("test-six-byte-pages");
    static PAGE_METADATA: otel_arrow_dfe_pdata_codec::CodecMetadata =
        otel_arrow_dfe_pdata_codec::CodecMetadata::new(PAGE_ENCODING, &[SignalType::Logs]);
    static PAGE_SUPPORT: otel_arrow_dfe_pdata_codec::BatchingSupport =
        otel_arrow_dfe_pdata_codec::BatchingSupport {
            sizers: &[Sizer::Bytes],
            default_settings: BatchSettings::otlp(),
        };
    struct PageCodec;
    impl otel_arrow_dfe_pdata_codec::PdataDecoder for PageCodec {
        fn decode(
            &mut self,
            _: SignalType,
            _: &Bytes,
        ) -> Result<otel_arrow_dfe_pdata::OtapArrowRecords, CodecError> {
            panic!("encoded batching must not decode pages")
        }
    }
    impl otel_arrow_dfe_pdata_codec::PdataBatcher for PageCodec {
        fn batch(
            &mut self,
            _: SignalType,
            settings: &BatchSettings,
            inputs: Vec<Bytes>,
        ) -> Result<otel_arrow_dfe_pdata_codec::CodecBatches, CodecError> {
            let rows: Vec<_> = inputs
                .iter()
                .enumerate()
                .flat_map(|(index, bytes)| {
                    bytes
                        .iter()
                        .filter(|byte| **byte != 0)
                        .map(move |byte| (*byte, index))
                })
                .collect();
            let size = settings.max_size.map_or(6, |size| {
                usize::try_from(size.get()).unwrap_or(usize::MAX).min(6)
            });
            let batches: Vec<_> = rows
                .chunks(size)
                .map(|rows| {
                    (
                        Bytes::from(rows.iter().map(|(byte, _)| *byte).collect::<Vec<_>>()),
                        rows.first().unwrap().1..=rows.last().unwrap().1,
                    )
                })
                .collect();
            let completed_inputs = (0..inputs.len())
                .filter(|index| !batches.iter().any(|(_, range)| range.contains(index)))
                .collect();
            Ok(otel_arrow_dfe_pdata_codec::CodecBatches {
                batches,
                completed_inputs,
                budget_fallbacks: 0,
            })
        }
    }
    otel_arrow_dfe_pdata_codec::register_pdata_codec!(
        PAGE_CODEC,
        otel_arrow_dfe_pdata_codec::CodecRegistration::new(&PAGE_METADATA)
            .with_decoder(|_| Box::new(PageCodec))
            .with_item_counter(|_, bytes| Some(bytes.len()))
            .with_batcher(otel_arrow_dfe_pdata_codec::CodecBatcherRegistration::new(
                &PAGE_SUPPORT,
                || Box::new(PageCodec)
            )),
    );

    /// Scenario: A native codec packs inputs [9, 2] into [6, 5], with the tail spanning both owners.
    /// Guarantees: Both outputs are sent, each input waits for all its outputs in either
    /// response order, and a tail Nack reaches both owners exactly once.
    #[test]
    fn codec_ranges_route_multi_owner_tail() {
        for reverse in [false, true] {
            for nack_tail in [false, true] {
                let (_, _, phase) = setup_test_runtime(json!({
                    "format": "preserve", "max_batch_duration": "1s",
                    "codecs": {"test-six-byte-pages": {"sizer": "bytes", "min_size": 10, "max_size": 10}}
                }));
                phase
                    .run_test(move |mut ctx| async move {
                        let (tx, mut rx) = pipeline_completion_msg_channel(16);
                        ctx.set_pipeline_completion_sender(tx);
                        let mut done = Completions::new(2);
                        let codec = CodecRegistry::global()
                            .unwrap()
                            .resolve(&PAGE_ENCODING)
                            .unwrap();
                        for (i, bytes) in
                            [Bytes::from_static(b"123456789"), Bytes::from_static(b"ab")]
                                .into_iter()
                                .enumerate()
                        {
                            let input = OtapPdata::new_default(
                                codec.admit(SignalType::Logs, bytes).unwrap().into(),
                            )
                            .test_subscribe_to(
                                Interests::ACKS | Interests::NACKS,
                                TestCallData::new_with(i as u64, 0).into(),
                                1,
                            );
                            ctx.process(Message::PData(input)).await.unwrap();
                        }
                        let mut outputs = ctx.drain_pdata().await;
                        assert_eq!(outputs.len(), 2, "a multi-owner tail must not be retained");
                        assert_eq!(
                            outputs[0].payload_ref().encoded_bytes().unwrap().as_ref(),
                            b"123456"
                        );
                        assert_eq!(
                            outputs[1].payload_ref().encoded_bytes().unwrap().as_ref(),
                            b"789ab"
                        );
                        let tail = outputs.pop().unwrap();
                        let first = outputs.pop().unwrap();
                        let policy = if nack_tail {
                            AckPolicy::Nack("tail failed")
                        } else {
                            AckPolicy::Ack
                        };
                        if reverse {
                            respond(&mut ctx, tail, &policy).await;
                            done.drain(&mut rx);
                            assert_eq!(done.acks, if nack_tail { vec![0, 0] } else { vec![0, 1] });
                            respond(&mut ctx, first, &AckPolicy::Ack).await;
                        } else {
                            respond(&mut ctx, first, &AckPolicy::Ack).await;
                            done.drain(&mut rx);
                            assert_eq!(done.acks, vec![0, 0]);
                            respond(&mut ctx, tail, &policy).await;
                        }
                        done.drain(&mut rx);
                        assert_eq!(done.acks, if nack_tail { vec![0, 0] } else { vec![1, 1] });
                        assert_eq!(done.nacks, if nack_tail { vec![1, 1] } else { vec![0, 0] });
                        flush_timer(&mut ctx).await;
                        assert!(ctx.drain_pdata().await.is_empty());
                    })
                    .validate(|_| async {});
            }
        }
    }

    /// Scenario: A codec consumes nonempty encoded inputs but produces no surviving rows.
    /// Guarantees: Explicit no-output completions Ack each input once without sending data.
    #[test]
    fn codec_ranges_complete_inputs_without_output() {
        let (_, _, phase) = setup_test_runtime(json!({
            "format": "preserve", "max_batch_duration": "1s",
            "codecs": {"test-six-byte-pages": {"sizer": "bytes", "min_size": 10, "max_size": 10}}
        }));
        phase
            .run_test(|mut ctx| async move {
                let (tx, mut rx) = pipeline_completion_msg_channel(16);
                ctx.set_pipeline_completion_sender(tx);
                let mut done = Completions::new(2);
                let codec = CodecRegistry::global()
                    .unwrap()
                    .resolve(&PAGE_ENCODING)
                    .unwrap();
                for i in 0..2 {
                    let input = OtapPdata::new_default(
                        codec
                            .admit(SignalType::Logs, Bytes::from_static(&[0; 6]))
                            .unwrap()
                            .into(),
                    )
                    .test_subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        TestCallData::new_with(i as u64, 0).into(),
                        1,
                    );
                    ctx.process(Message::PData(input)).await.unwrap();
                }
                assert!(ctx.drain_pdata().await.is_empty());
                done.drain(&mut rx);
                assert_eq!(done.acks, [1, 1]);
                assert_eq!(done.nacks, [0, 0]);
            })
            .validate(|_| async {});
    }

    fn otlp_format() -> PdataFormat {
        let registry = CodecRegistry::global().unwrap();
        PdataFormat::encoded(registry.resolve(&PdataEncoding::OTLP).unwrap())
    }

    /// Helper to create test pipeline context
    fn create_test_pipeline_context() -> (
        otel_arrow_dfe_engine::context::PipelineContext,
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
        otel_arrow_dfe_telemetry::reporter::MetricsReporter,
        otel_arrow_dfe_engine::testing::processor::TestPhase<OtapPdata>,
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

    fn mmsc_metric_count(
        telemetry_registry: &TelemetryRegistryHandle,
        set_name: &str,
        metric_name: &str,
    ) -> u64 {
        mmsc_metric_summary(telemetry_registry, set_name, metric_name).0
    }

    fn mmsc_metric_summary(
        telemetry_registry: &TelemetryRegistryHandle,
        set_name: &str,
        metric_name: &str,
    ) -> (u64, f64) {
        let mut summary = (0, 0.0);
        telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == set_name {
                for (field, metric_value) in iter {
                    if field.name == metric_name
                        && let otel_arrow_dfe_telemetry::metrics::MetricValue::Distribution(
                            distribution,
                        ) = metric_value
                    {
                        let (count, sum, _, _) = distribution.summary();
                        summary = (count, sum);
                    }
                }
            }
        });
        summary
    }

    async fn assert_flush_output_metrics(
        registry: &TelemetryRegistryHandle,
        batches: usize,
        bytes: usize,
    ) {
        otel_arrow_dfe_engine::testing::liveness::completes_within(
            Duration::from_secs(5),
            "flush output metrics",
            async {
                loop {
                    let count = mmsc_metric_summary(
                        registry,
                        "otap.processor.batch",
                        "flush.output.batches",
                    );
                    let size =
                        mmsc_metric_summary(registry, "otap.processor.batch", "flush.output.bytes");
                    if count.0 != 0 && size.0 != 0 {
                        assert_eq!(
                            count,
                            (1, batches as f64),
                            "one batch-count observation per flush"
                        );
                        assert_eq!(
                            size,
                            (1, bytes as f64),
                            "one final-byte-size observation per flush"
                        );
                        return;
                    }
                    tokio::task::yield_now().await;
                }
            },
        )
        .await;
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

    /// Scenario: The batch processor is created with its default configuration.
    /// Guarantees: Default codec settings and duration limits are accepted.
    #[test]
    fn test_default_config_ok() {
        let cfg: Config = serde_json::from_value(json!({})).unwrap();
        cfg.validate().unwrap();
    }

    /// Scenario: Configuration supplies the full u64 range for batching thresholds and split budgets.
    /// Guarantees: Startup retains every limit without narrowing; small inputs cannot satisfy a huge minimum.
    #[test]
    fn batching_settings_preserve_u64_limits() {
        let cfg: Config = serde_json::from_value(json!({
            "format": "otlp",
            "otlp": {
                "sizer": "bytes",
                "min_size": u64::MAX,
                "max_size": u64::MAX,
                "max_split_fragments": u64::MAX,
                "max_split_overhead_bytes": u64::MAX,
                "max_split_fragments_per_flush": u64::MAX
            }
        }))
        .unwrap();
        let plans = cfg.plans().unwrap();
        let settings = plans[0].1.settings();
        assert_eq!(
            settings,
            &BatchSettings {
                min_size: Some(u64::MAX),
                max_size: NonZeroU64::new(u64::MAX),
                sizer: Sizer::Bytes,
                max_split_fragments: NonZeroU64::new(u64::MAX),
                max_split_overhead_bytes: NonZeroU64::new(u64::MAX),
                max_split_fragments_per_flush: NonZeroU64::new(u64::MAX),
            }
        );
        assert_eq!(settings.lower_limit(), u64::MAX);
        assert!(!settings.in_passthrough_range(2));
    }

    /// Scenario: users configure built-in settings through legacy aliases or
    /// canonical codec names, or request ambiguous/unsupported configurations.
    /// Guarantees: compatible YAML resolves identically and invalid capabilities
    /// are rejected before the processor buffers any requests.
    #[test]
    fn codec_settings_preserve_configuration_compatibility() {
        let legacy: Config = serde_json::from_value(json!({
            "format": "otlp", "otlp": { "min_size": 32, "max_size": 64, "sizer": "bytes" }
        }))
        .unwrap();
        let canonical: Config = serde_json::from_value(json!({
            "format": "otlp-bytes", "codecs": { "otlp-bytes": { "min_size": 32, "max_size": 64, "sizer": "bytes" } }
        })).unwrap();
        assert_eq!(legacy.plans().unwrap(), canonical.plans().unwrap());
        let immediate: Config = serde_json::from_value(json!({
            "max_batch_duration": "0s",
            "otap": { "max_size": 100, "sizer": "items" },
            "otlp": { "max_size": 10000, "sizer": "bytes" }
        }))
        .unwrap();
        immediate.validate().unwrap();
        assert!(
            serde_json::from_value::<Config>(json!({
                "otlp": { "min_size": 32, "sizer": "bytes" },
                "codecs": { "otlp-bytes": { "min_size": 32, "sizer": "bytes" } }
            }))
            .unwrap_err()
            .to_string()
            .contains("duplicate")
        );
        assert!(
            serde_json::from_value::<Config>(json!({
                "codecs": { "not-registered": { "min_size": 1, "sizer": "items" } }
            }))
            .is_err()
        );
        let decode_only: Config = serde_json::from_value(json!({
            "format": "test-otlp-decode-only"
        }))
        .unwrap();
        assert!(
            decode_only
                .validate()
                .unwrap_err()
                .to_string()
                .contains("cannot encode")
        );
        let unsupported: Config = serde_json::from_value(json!({
            "codecs": { "test-otlp-codec": { "min_size": 1, "sizer": "bytes" } }
        }))
        .unwrap();
        assert!(
            unsupported
                .validate()
                .unwrap_err()
                .to_string()
                .contains("fallback requires items")
        );
    }

    /// Scenario: preserve mode receives a native-batching extension and a
    /// decoder-only extension, and downstream changes their output representation.
    /// Guarantees: native output retains its codec, fallback emits OTAP, and both
    /// Ack and Nack reach the correct input through explicit completion tokens,
    /// preserving both retryable and permanent downstream failures.
    #[test]
    fn codec_batches_route_completions_after_downstream_conversion() {
        use otel_arrow_dfe_pdata_codec::{CodecService, EncodedPdata, PdataEncoding};
        let (_, _, phase) = setup_test_runtime(json!({
            "format": "preserve",
            "otap": { "min_size": 1000000, "sizer": "items" },
            "codecs": { "test-native-otlp": { "min_size": 1000000, "sizer": "bytes" } },
            "max_batch_duration": "1s"
        }));
        phase
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(completion_tx);
                let mut generator = DataGenerator::new(1);
                for (index, name) in [
                    "test-native-otlp",
                    "test-otlp-decode-only",
                    "test-otlp-decode-only",
                ]
                .into_iter()
                .enumerate()
                {
                    let logs: OtlpProtoMessage = generator.generate_logs().into();
                    let bytes = otlp_message_to_bytes(&logs).clone_bytes();
                    let registry = CodecRegistry::global().unwrap();
                    let encoded = EncodedPdata::new(
                        &registry,
                        &PdataEncoding::from(name.to_owned()),
                        SignalType::Logs,
                        bytes,
                    )
                    .unwrap();
                    let input = OtapPdata::new_default(encoded.into()).test_subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        TestCallData::new_with(index as u64, 0).into(),
                        23,
                    );
                    ctx.process(Message::PData(input)).await.unwrap();
                    assert!(ctx.drain_pdata().await.is_empty());
                    assert!(ctx.fire_wakeup().await.unwrap());
                    let mut outputs = ctx.drain_pdata().await;
                    assert_eq!(outputs.len(), 1);
                    let output = outputs.pop().unwrap();
                    assert_eq!(
                        output.payload_ref().format().name(),
                        if index == 0 { name } else { "otap" }
                    );
                    assert_equivalent(&[logs], &[otap_pdata_to_message(&output)]);
                    // Both replies use a representation different from the emitted one.
                    let returned = if index == 0 {
                        let service = CodecService::new().unwrap();
                        let (context, payload) = output.into_parts();
                        OtapPdata::new(context, payload.try_into_otap(&service).unwrap().into())
                    } else {
                        let service = CodecService::new().unwrap();
                        let plan = otel_arrow_dfe_pdata_codec::EncodingPlan::resolve(
                            service.registry(),
                            &PdataEncoding::OTLP,
                            Default::default(),
                        )
                        .unwrap();
                        let (context, mut payload) = output.into_parts();
                        let signal = payload.signal_type();
                        let bytes = payload.encode_bytes(&service, &plan).unwrap();
                        OtapPdata::new(context, plan.codec().admit(signal, bytes).unwrap().into())
                    };
                    if index == 0 {
                        let (_, ack) = next_ack(AckMsg::new(returned)).unwrap();
                        ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
                            .await
                            .unwrap();
                    } else {
                        let nack = if index == 2 {
                            NackMsg::new_permanent("test rejection", returned)
                        } else {
                            NackMsg::new("test rejection", returned)
                        };
                        let (_, nack) = next_nack(nack).unwrap();
                        ctx.process(Message::Control(NodeControlMsg::Nack(nack)))
                            .await
                            .unwrap();
                    }
                    let completion = next_completion(
                        &mut completion_rx,
                        Duration::from_secs(1),
                        "codec batch completion",
                    )
                    .await;
                    let (node, calldata) = match completion {
                        PipelineCompletionMsg::DeliverAck { ack } if index == 0 => {
                            let (node, ack) = next_ack(ack).unwrap();
                            (node, ack.unwind.route.calldata)
                        }
                        PipelineCompletionMsg::DeliverNack { nack } if index != 0 => {
                            assert_eq!(nack.permanent, index == 2);
                            assert_eq!(nack.reason, "test rejection");
                            let (node, nack) = next_nack(nack).unwrap();
                            (node, nack.unwind.route.calldata)
                        }
                        other => panic!("unexpected completion: {other:?}"),
                    };
                    assert_eq!(node, 23);
                    assert_eq!(
                        TestCallData::try_from(calldata).unwrap(),
                        TestCallData::new_with(index as u64, 0)
                    );
                }
            })
            .validate(|_| async {});
    }

    #[test]
    fn test_config_validation() {
        // Both sizes set, max >= batch: OK
        let cfg = Config {
            otap: FormatConfig::new_items(Some(100), 200),
            otlp: FormatConfig::new_bytes(Some(10000), 20000),
            max_batch_duration: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // The OTLP configuration is invalid, but that's OK because
        // format is OTAP.
        let cfg = Config {
            otap: FormatConfig::new_items(Some(100), 0),
            otlp: FormatConfig::new_items(None, 0),
            max_batch_duration: Duration::from_millis(100),
            format: BatchingFormat::Otap,
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Only max size: OK (OTLP default)
        let cfg = Config {
            otap: FormatConfig::new_items(None, 200),
            max_batch_duration: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // All formats
        let cfg = Config {
            otap: FormatConfig::new_items(None, 200),
            otlp: FormatConfig::new_bytes(Some(20000), 0),
            max_batch_duration: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Split-only: OK
        let cfg = Config {
            otlp: FormatConfig::new_bytes(None, 100),
            otap: FormatConfig::new_items(None, 0),
            max_batch_duration: Duration::ZERO,
            format: BatchingFormat::Otlp,
            ..Default::default()
        };
        assert!(cfg.validate().is_ok());

        // Format: preserve, 1 invalid
        let cfg = Config {
            otlp: FormatConfig::new_bytes(None, 0),
            format: BatchingFormat::Preserve,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // Both None: ERROR
        let cfg = Config {
            otap: FormatConfig::new_bytes(None, 0),
            format: BatchingFormat::Otap,
            max_batch_duration: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // Max < batch: ERROR
        let cfg = Config {
            otap: FormatConfig::new_items(Some(200), 100),
            max_batch_duration: Duration::from_millis(100),
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // lower-bound without timeout: ERROR
        let cfg = Config {
            otap: FormatConfig::new_items(Some(100), 200),
            max_batch_duration: Duration::ZERO,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());
    }

    /// Scenario: validation of configs using an explicit `min_size: 0`, with
    /// and without `max_size` and `max_batch_duration`.
    /// Guarantees: `min_size: 0` is accepted with or without a timeout and
    /// with or without `max_size`; it satisfies `max_size >= min_size`; a
    /// non-zero `min_size` without a timeout is still rejected.
    #[test]
    fn test_config_validation_min_size_zero() {
        for timeout in [Duration::ZERO, Duration::from_millis(100)] {
            // min_size: 0 with max_size: OK.
            let cfg = Config {
                otap: FormatConfig::new_items(Some(0), 200),
                max_batch_duration: timeout,
                format: BatchingFormat::Otap,
                ..Default::default()
            };
            assert!(cfg.validate().is_ok(), "min 0 + max, timeout {timeout:?}");
            assert!(!BatchSettings::from(&cfg.otap).forwards_everything());

            // min_size: 0 without max_size: OK (warned at build time).
            let cfg = Config {
                otap: FormatConfig::new_items(Some(0), 0),
                max_batch_duration: timeout,
                format: BatchingFormat::Otap,
                ..Default::default()
            };
            assert!(cfg.validate().is_ok(), "min 0 only, timeout {timeout:?}");
            assert!(BatchSettings::from(&cfg.otap).forwards_everything());
        }

        // Non-zero min_size without timeout is still an error.
        let cfg = Config {
            otap: FormatConfig::new_items(Some(1), 200),
            max_batch_duration: Duration::ZERO,
            format: BatchingFormat::Otap,
            ..Default::default()
        };
        assert!(cfg.validate().is_err());

        // min_size: 0 parses from JSON.
        let cfg: Config = serde_json::from_value(json!({
            "format": "otap",
            "otap": { "min_size": 0, "max_size": 10, "sizer": "items" },
            "max_batch_duration": "0s"
        }))
        .expect("parse");
        assert_eq!(cfg.otap.min_size, Some(0));
        assert!(cfg.validate().is_ok());
    }

    /// Scenario: `lower_limit` and `in_passthrough_range` across min/max
    /// combinations, including `min_size` unset and `min_size: 0`.
    /// Guarantees: an unset `min_size` defaults to `max_size`; the
    /// pass-through range is `[lower_limit, max_size]` inclusive, and has no
    /// upper bound when `max_size` is unset.
    #[test]
    fn test_passthrough_range() {
        let both = BatchSettings::from(&FormatConfig::new_items(Some(5), 10));
        assert_eq!(both.lower_limit(), 5);
        assert!(!both.in_passthrough_range(4));
        assert!(!both.in_passthrough_range(11));
        assert!(both.in_passthrough_range(5));
        assert!(both.in_passthrough_range(10));

        let max_only = BatchSettings::from(&FormatConfig::new_items(None, 10));
        assert_eq!(max_only.lower_limit(), 10);
        assert!(!max_only.in_passthrough_range(9));
        assert!(!max_only.in_passthrough_range(11));
        assert!(max_only.in_passthrough_range(10));

        let min_only = BatchSettings::from(&FormatConfig::new_items(Some(5), 0));
        assert_eq!(min_only.lower_limit(), 5);
        assert!(!min_only.in_passthrough_range(4));
        assert!(min_only.in_passthrough_range(usize::MAX));

        let zero = BatchSettings::from(&FormatConfig::new_items(Some(0), 10));
        assert_eq!(zero.lower_limit(), 0);
        assert!(zero.in_passthrough_range(1));
        assert!(zero.in_passthrough_range(10));
        assert!(!zero.in_passthrough_range(11));
    }

    /// Test event: either process input or deliver pending timers
    #[derive(Clone)]
    enum TestEvent {
        Input(OtlpProtoMessage),
        Elapsed, // Signal to deliver due wakeups
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
                    let payload: OtlpProtoBytes =
                        d.clone().payload().try_into_with_default().expect("ok");
                    payload.try_into().expect("ok")
                })
                .expect("ok")
        }
    }

    fn otap_pdata_to_message(data: &OtapPdata) -> OtlpProtoMessage {
        let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
        let rec = data.payload_ref().clone().try_into_otap(&service).unwrap();
        otap_to_otlp(&rec)
    }

    const fn all_wakeup_slots() -> [WakeupSlot; 6] {
        [
            wakeup_slot(0, SignalType::Logs),
            wakeup_slot(0, SignalType::Metrics),
            wakeup_slot(0, SignalType::Traces),
            wakeup_slot(1, SignalType::Logs),
            wakeup_slot(1, SignalType::Metrics),
            wakeup_slot(1, SignalType::Traces),
        ]
    }

    /// Node index of the batch processor built by [`setup_test_runtime`].
    /// Test inputs subscribe with other node ids, so a response whose next
    /// subscriber is this index belongs to the batch processor.
    fn batch_node_index() -> usize {
        test_node("batch-processor-test").index
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
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(config);

        // Collect events and extract inputs for reference
        let events: Vec<TestEvent> = events.collect();
        let inputs_otlp: Vec<OtlpProtoMessage> = events
            .iter()
            .filter_map(|e| match e {
                TestEvent::Input(msg) => Some(msg.clone()),
                TestEvent::Elapsed => None,
            })
            .collect();

        let input_item_count: usize = inputs_otlp.iter().map(|m| m.num_items()).sum();
        let num_inputs = inputs_otlp.len();
        let total_events = events.len();

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                // Track outputs by event position
                let mut event_outputs: Vec<EventOutputs> = vec![
                    EventOutputs {
                        outputs: Vec::new()
                    };
                    total_events
                ];
                let mut received_acks: Vec<TestCallData> = Vec::new();
                let mut received_nacks: Vec<TestCallData> = Vec::new();

                let mut input_idx = 0;
                let mut total_outputs = 0;

                // Process each event in sequence
                for (event_idx, event) in events.into_iter().enumerate() {
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
                            let when = Instant::now() + Duration::from_secs(1);
                            for slot in all_wakeup_slots() {
                                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                                    slot,
                                    when,
                                    revision: 0,
                                }))
                                .await
                                .expect("process wakeup");
                            }
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
                            if new_output.has_ack_or_nack_interests() {
                                let policy = nack_policy
                                    .as_ref()
                                    .map(|p| p(total_outputs - 1, &new_output))
                                    .unwrap_or(AckPolicy::Ack);

                                // Route like the engine: a response goes to
                                // the batch processor only when it is the next
                                // subscriber. A passed-through output carries
                                // the input's own context, so its response
                                // goes directly upstream.
                                match policy {
                                    AckPolicy::Ack => {
                                        let (node_id, ack) =
                                            next_ack(AckMsg::new(new_output)).expect("has subs");
                                        if node_id == batch_node_index() {
                                            ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
                                                .await
                                                .expect("process ack");
                                        } else {
                                            let calldata: TestCallData =
                                                ack.unwind.route.calldata.try_into().expect("cd");
                                            received_acks.push(calldata);
                                            looped += 1;
                                        }
                                    }
                                    AckPolicy::Nack(reason) => {
                                        let (node_id, nack) =
                                            next_nack(NackMsg::new(reason, new_output))
                                                .expect("has subs");
                                        if node_id == batch_node_index() {
                                            ctx.process(Message::Control(NodeControlMsg::Nack(
                                                nack,
                                            )))
                                            .await
                                            .expect("process nack");
                                        } else {
                                            let calldata: TestCallData =
                                                nack.unwind.route.calldata.try_into().expect("cd");
                                            received_nacks.push(calldata);
                                            looped += 1;
                                        }
                                    }
                                }
                            }
                        }

                        loop {
                            match pipeline_completion_rx.try_recv() {
                                Ok(PipelineCompletionMsg::DeliverAck { ack }) => {
                                    looped += 1;
                                    if let Some((_node_id, ack)) = next_ack(ack) {
                                        let calldata: TestCallData =
                                            ack.unwind.route.calldata.try_into().expect("calldata");
                                        received_acks.push(calldata);
                                    }
                                }
                                Ok(PipelineCompletionMsg::DeliverNack { nack }) => {
                                    looped += 1;
                                    if let Some((_node_id, nack)) = next_nack(nack) {
                                        let calldata: TestCallData = nack
                                            .unwind
                                            .route
                                            .calldata
                                            .try_into()
                                            .expect("calldata");
                                        received_nacks.push(calldata);
                                    }
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
                "max_batch_duration": "1s"
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
                "max_batch_duration": "50ms"
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

    /// Scenario: the batch processor derives wakeup slots from the supported
    /// `(format, signal)` pairs used by its internal timers.
    /// Guarantees: every supported pair round-trips through the encoder/decoder,
    /// and each pair maps to a distinct wakeup slot.
    #[test]
    fn test_wakeup_slot_round_trip_and_uniqueness() {
        let slots = [
            (0, SignalType::Logs),
            (0, SignalType::Metrics),
            (0, SignalType::Traces),
            (1, SignalType::Logs),
            (1, SignalType::Metrics),
            (1, SignalType::Traces),
        ];

        for (expected_format, expected_signal) in slots {
            let slot = wakeup_slot(expected_format, expected_signal);
            assert_eq!(
                signal_from_wakeup_slot(slot),
                Some((expected_format, expected_signal))
            );
        }

        let mut unique = std::collections::HashSet::new();
        for (format, signal) in slots {
            assert!(
                unique.insert(wakeup_slot(format, signal)),
                "slot mapping should be unique for each format/signal pair"
            );
        }
        assert_eq!(unique.len(), 6);
    }

    // The processor replaces wakeups per slot. This test proves that an early
    // wakeup is ignored and that the current wakeup still flushes the buffered
    // input later.
    /// Scenario: A stale timer wakeup arrives before the current batch deadline.
    /// Guarantees: The early wakeup leaves data buffered and the current wakeup flushes it once.
    #[test]
    fn test_timer_flush_ignores_stale_wakeup() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 5,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "50ms"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let mut datagen = DataGenerator::new(1);
                let first = datagen.generate_logs();
                let second = datagen.generate_logs();
                let third = datagen.generate_logs();

                let rec = encode_logs_otap_batch(&first).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process first input");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "first input should remain buffered"
                );

                // The second input takes the buffer over the min size, so the processor flushes
                // before the original timer fires.
                let rec = encode_logs_otap_batch(&second).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process second input");
                let first_flush = ctx.drain_pdata().await;
                assert_eq!(first_flush.len(), 1, "size flush should emit one batch");

                let rec = encode_logs_otap_batch(&third).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process third input");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "new post-flush batch should remain buffered"
                );

                let stale_when = Instant::now();
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: wakeup_slot(0, SignalType::Logs),
                    when: stale_when,
                    revision: 0,
                }))
                .await
                .expect("process stale wakeup");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "stale wakeup should be ignored"
                );

                let current_when = Instant::now() + Duration::from_secs(1);
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: wakeup_slot(0, SignalType::Logs),
                    when: current_when,
                    revision: 1,
                }))
                .await
                .expect("process current wakeup");
                let final_flush = ctx.drain_pdata().await;
                assert_eq!(
                    final_flush.len(),
                    1,
                    "current wakeup should flush buffered input"
                );

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 3 inputs consumed; 2 batches produced (one size flush after
                // the second input, one timer flush after the third).
            });
    }

    /// A size flush that empties the buffer must clear timer state. Otherwise
    /// later one-request size flushes try to cancel a wakeup that is no longer
    /// live, which inflates scheduler cancel-miss telemetry.
    #[test]
    fn test_size_flush_clears_timer_state_after_full_drain() {
        fn logs_with_count(base_id: usize, count: usize) -> LogsData {
            let base_time = 1_000_000_000 + (base_id * 1000) as u64;
            LogsData {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: (0..count)
                            .map(|i| LogRecord {
                                time_unix_nano: base_time + i as u64,
                                ..Default::default()
                            })
                            .collect(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }
        }

        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 5,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                for (index, count) in [3, 3, 6, 6].into_iter().enumerate() {
                    let rec = encode_logs_otap_batch(&logs_with_count(index, count))
                        .expect("encode logs");
                    ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                        .await
                        .expect("process input");

                    let outputs = ctx.drain_pdata().await;
                    match index {
                        0 => assert!(outputs.is_empty(), "first input should arm the timer"),
                        _ => assert_eq!(outputs.len(), 1, "input {index} should size-flush"),
                    }
                }

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 4 inputs consumed; 3 size-flushed batches produced (the
                // first input only arms the timer; each subsequent input
                // size-flushes one batch).
                assert_eq!(
                    mmsc_metric_count(
                        &telemetry_registry,
                        "otap.processor.batch",
                        "flush.age.duration"
                    ),
                    1,
                    "only the flush of the timer-armed partial batch should use timer age"
                );
            });
    }

    /// Scenario: buffered input has armed a real batch wakeup, but the processor
    /// first receives a foreign wakeup slot that does not decode to any local
    /// `(format, signal)` timer.
    /// Guarantees: the foreign wakeup is ignored without flushing or corrupting
    /// state, and the real/current wakeup still flushes the buffered input later.
    #[test]
    fn test_unknown_wakeup_slot_is_ignored_without_side_effects() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 5,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "50ms"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let mut datagen = DataGenerator::new(1);
                let input = datagen.generate_logs();

                let rec = encode_logs_otap_batch(&input).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec.into())))
                    .await
                    .expect("process input");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "input should remain buffered until the real wakeup"
                );

                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: WakeupSlot(99),
                    when: Instant::now(),
                    revision: 0,
                }))
                .await
                .expect("process unknown wakeup");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "foreign wakeup should be ignored"
                );

                let current_when = Instant::now() + Duration::from_secs(1);
                ctx.process(Message::Control(NodeControlMsg::Wakeup {
                    slot: wakeup_slot(0, SignalType::Logs),
                    when: current_when,
                    revision: 1,
                }))
                .await
                .expect("process current wakeup");
                let flushed = ctx.drain_pdata().await;
                assert_eq!(flushed.len(), 1, "real wakeup should flush buffered input");

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 1 input consumed; 1 batch produced by the real wakeup.
            });
    }

    /// Scenario: the batch processor runs in forced OTAP mode, has live
    /// outbound completion state in its OTAP batch bookkeeping, and then
    /// receives a downstream Ack whose returned payload format is OTLP bytes.
    /// Guarantees: response handling falls back to the active OTAP batch state,
    /// releases the outbound slot, and delivers the upstream Ack without
    /// panicking on the returned payload format.
    #[test]
    fn test_ack_response_format_falls_back_to_active_batch_state() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "format": "otap",
            "otap": {
                "min_size": 4,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);

                // An unsubscribed 3-item input is buffered (below min_size) so
                // the subscribed input below is merged with it rather than
                // passed through, exercising outbound completion tracking.
                let first: OtlpProtoMessage = datagen.generate_logs().into();
                ctx.process(Message::PData(OtapPdata::new_default(
                    otlp_message_to_bytes(&first).into(),
                )))
                .await
                .expect("process first input");

                let input: OtlpProtoMessage = datagen.generate_logs().into();
                let input_bytes = otlp_message_to_bytes(&input);

                let pdata = OtapPdata::new_default(input_bytes.clone().into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    23,
                );

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");

                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1, "size flush should emit one batch");

                let output = outputs.remove(0);
                let (output_ctx, _output_payload) = output.into_parts();
                let returned = OtapPdata::new(output_ctx, input_bytes.into());

                let (_, ack) =
                    next_ack(AckMsg::new(returned)).expect("expected outbound ack subscriber");
                ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
                    .await
                    .expect("process ack");

                match next_completion(
                    &mut pipeline_completion_rx,
                    Duration::from_secs(1),
                    "batch processor upstream completion after format fallback ack",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        let (node_id, ack) = next_ack(ack).expect("expected ack subscriber");
                        assert_eq!(node_id, 23);
                        let calldata: TestCallData =
                            ack.unwind.route.calldata.try_into().expect("calldata");
                        assert_eq!(TestCallData::default(), calldata);
                    }
                    other => panic!("expected upstream ack after format fallback, got {other:?}"),
                }
            })
            .validate(|_| async move {});
    }

    // A partial batch that never reached the size threshold must still flush on
    // Shutdown, and its downstream Ack must release the upstream completion state
    // rather than leaving correlated requests stuck.
    #[test]
    fn test_shutdown_flushes_buffered_input_and_releases_completion() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 10,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "10s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                let input = datagen.generate_logs();
                let rec = encode_logs_otap_batch(&input).expect("encode logs");
                let pdata = OtapPdata::new_default(rec.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    17,
                );

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");
                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "input should remain buffered before shutdown"
                );

                ctx.process(Message::Control(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "test shutdown".into(),
                }))
                .await
                .expect("process shutdown");

                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1, "shutdown should flush buffered batch");
                let output = outputs.remove(0);

                let (_, ack) = next_ack(AckMsg::new(output)).expect("expected ack subscriber");
                ctx.process(Message::Control(NodeControlMsg::Ack(ack)))
                    .await
                    .expect("process ack");

                match next_completion(
                    &mut pipeline_completion_rx,
                    Duration::from_secs(1),
                    "batch processor upstream completion after shutdown flush",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        let (node_id, ack) = next_ack(ack).expect("expected ack subscriber");
                        assert_eq!(node_id, 17);
                        let calldata: TestCallData =
                            ack.unwind.route.calldata.try_into().expect("calldata");
                        assert_eq!(TestCallData::default(), calldata);
                    }
                    other => panic!("expected upstream ack after shutdown flush, got {other:?}"),
                }

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 1 input consumed; 1 batch produced by the shutdown flush.
            });
    }

    // Subscribed requests consume bounded inbound slots until downstream outcomes
    // release them. Hitting the slot cap should return an explicit Nack instead
    // of stalling the event loop or surfacing as an engine error.
    #[test]
    fn test_inbound_slot_exhaustion_surfaces_error() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 10,
                "max_size": 10,
                "sizer": "items",
            },
            "max_batch_duration": "10s",
            "inbound_request_limit": 1
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(10);
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_runtime_ctrl_sender(runtime_ctrl_tx);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                for request_index in 0..2 {
                    let input = datagen.generate_logs();
                    let rec = encode_logs_otap_batch(&input).expect("encode logs");
                    let pdata = OtapPdata::new_default(rec.into()).test_subscribe_to(
                        Interests::ACKS | Interests::NACKS,
                        TestCallData::new_with(request_index, 0).into(),
                        11,
                    );

                    let result = ctx.process(Message::PData(pdata)).await;
                    if request_index == 0 {
                        result.expect("first subscribed request should be accepted");
                    } else {
                        result.expect("second request should be nacked, not fail the processor");
                    }
                }

                match next_completion(
                    &mut pipeline_completion_rx,
                    Duration::from_millis(50),
                    "expected completion nack",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        assert!(
                            nack.reason.contains("inbound routes exhausted"),
                            "unexpected nack reason: {}",
                            nack.reason
                        );
                    }
                    PipelineCompletionMsg::DeliverAck { .. } => {
                        panic!("expected nack when inbound slots are exhausted")
                    }
                }
            })
            .validate(|_| async move {});
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
                "max_batch_duration": "1s"
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
                "max_batch_duration": "1s"
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
                "max_batch_duration": "1s"
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
                "max_batch_duration": "0s"
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
                "max_batch_duration": "0s"
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
                "max_batch_duration": "1s"
            }),
            json!({
                "otlp": {
                    "min_size": 20,
                    "max_size": 30,
                    "sizer": "bytes",
                },
                "format": "otlp",
                "max_batch_duration": "1s"
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

    /// OTLP min_size is byte-based. The size flush decision must compare the
    /// configured threshold against pending bytes, not against log-record count.
    /// Scenario: Encoded OTLP input reaches the configured byte threshold.
    /// Guarantees: The processor flushes OTLP-encoded output without waiting for its timer.
    #[test]
    fn test_otlp_byte_min_size_triggers_size_flush() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "format": "otlp",
            "otlp": {
                "min_size": 4,
                "sizer": "bytes",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let mut datagen = DataGenerator::new(1);
                let logs: OtlpProtoMessage = datagen.generate_logs().into();
                let input_bytes = otlp_message_to_bytes(&logs);
                assert!(
                    input_bytes.num_bytes() >= 4,
                    "generated OTLP request should exceed the byte threshold"
                );

                ctx.process(Message::PData(OtapPdata::new_default(input_bytes.into())))
                    .await
                    .expect("process otlp bytes");

                let outputs = ctx.drain_pdata().await;
                assert_eq!(
                    outputs.len(),
                    1,
                    "byte min_size should trigger an immediate size flush"
                );
                assert_eq!(outputs[0].payload_ref().format(), otlp_format());

                let output_messages: Vec<_> = outputs.iter().map(otap_pdata_to_message).collect();
                assert_equivalent(&[logs], &output_messages);

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 1 input consumed; 1 batch produced by the byte-size flush.
            });
    }

    /// Scenario: a subscribed zero-byte OTLP request reaches the batch processor.
    /// Guarantees: it is acknowledged without output or entering the batch buffer.
    #[test]
    fn test_otlp_zero_byte_request_acked_immediately() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "format": "otlp",
            "otlp": {
                "min_size": 100,
                "sizer": "bytes",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(1);
                ctx.set_pipeline_completion_sender(completion_tx);
                let empty = OtlpProtoBytes::ExportLogsRequest(Bytes::new());
                let pdata = OtapPdata::new_default(empty.into()).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::new_with(0, 0).into(),
                    11,
                );
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process empty otlp");

                assert!(ctx.drain_pdata().await.is_empty(), "no batch should flush");
                match next_completion(
                    &mut completion_rx,
                    Duration::from_secs(1),
                    "zero-byte input should be acknowledged",
                )
                .await
                {
                    PipelineCompletionMsg::DeliverAck { mut ack } => {
                        assert_eq!(ack.accepted.num_items(), 0);
                    }
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        panic!("zero-byte input was unexpectedly nacked: {}", nack.reason);
                    }
                }

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
            });
    }

    /// An OTLP request that has non-zero bytes but zero log records is not
    /// treated as empty under the bytes sizer; it flows through the batcher
    /// like any other input.
    #[test]
    fn test_otlp_empty_container_passes_through_batcher() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "format": "otlp",
            "otlp": {
                "min_size": 1,
                "sizer": "bytes",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let logs = LogsData {
                    resource_logs: vec![ResourceLogs::default()],
                };
                let input_bytes = otlp_message_to_bytes(&OtlpProtoMessage::Logs(logs));
                assert!(input_bytes.num_bytes() > 0);

                ctx.process(Message::PData(OtapPdata::new_default(input_bytes.into())))
                    .await
                    .expect("process empty-container otlp");

                let outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1, "empty container should size-flush");

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
            });
    }

    /// Scenario: preserve-mode batching receives interleaved OTLP and native OTAP batches.
    /// Guarantees: each built-in representation is batched independently and keeps its output format.
    #[test]
    fn test_preserve_mode_mixed_formats() {
        let (_telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
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
            "max_batch_duration": "50ms"
        }));

        phase
            .run_test(move |mut ctx| async move {
                // Create test data
                let mut datagen = DataGenerator::new(1);
                let logs1: OtlpProtoMessage = datagen.generate_logs().into();
                let logs2: OtlpProtoMessage = datagen.generate_logs().into();

                // Convert to OTAP format
                let otlp_message1 = otlp_message_to_bytes(&logs1);
                let otap_message2 = otlp_to_otap(&logs2);

                let mut outputs = Vec::new();
                // Send both
                ctx.process(Message::PData(OtapPdata::new_default(otlp_message1.into())))
                    .await
                    .expect("process otap");

                ctx.process(Message::PData(OtapPdata::new_default(otap_message2.into())))
                    .await
                    .expect("process otlp");

                assert!(
                    ctx.drain_pdata().await.is_empty(),
                    "no outputs before timeout"
                );

                // Trigger timeout for both active batching slots.
                let when = Instant::now() + Duration::from_secs(1);
                for slot in [
                    wakeup_slot(1, SignalType::Logs),
                    wakeup_slot(0, SignalType::Logs),
                ] {
                    ctx.process(Message::Control(NodeControlMsg::Wakeup {
                        slot,
                        when,
                        revision: 0,
                    }))
                    .await
                    .expect("process wakeup");
                }

                // Drain outputs after timeout
                let mut drained_outputs = ctx.drain_pdata().await;
                outputs.append(&mut drained_outputs);
                assert!(!outputs.is_empty(), "should have outputs after timeout");

                // Verify both formats
                let mut has_otap = false;
                let mut has_otlp = false;
                for output in &outputs {
                    if output.payload_ref().format() == PdataFormat::OTAP {
                        has_otap = true;
                    } else {
                        assert_eq!(output.payload_ref().format(), otlp_format());
                        has_otlp = true;
                    }
                }

                assert!(has_otap, "should have OTAP output");
                assert!(has_otlp, "should have OTLP output");

                // Test equivalence.
                let outputs: Vec<_> = outputs.iter().map(otap_pdata_to_message).collect();

                assert_equivalent(&[logs1, logs2], &outputs);
                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                // 2 inputs consumed (one OTLP, one OTAP); 2 batches produced
                // (one per format, both flushed by their respective wakeups).
            });
    }

    /// When inbound_request_limit is 1 and two subscribed requests
    /// arrive before the batch flushes, the second request is Nacked.
    #[test]
    fn test_inbound_slot_exhaustion_nacks() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 1000,
                "sizer": "items",
            },
            "max_batch_duration": "10s",
            "inbound_request_limit": 1,
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                let logs1 = datagen.generate_logs();
                let logs2 = datagen.generate_logs();

                let rec1 = encode_logs_otap_batch(&logs1).expect("encode");
                let rec2 = encode_logs_otap_batch(&logs2).expect("encode");

                // Both inputs subscribe for ack/nack.
                let pdata1 = OtapPdata::new_default(rec1.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::new_with(0, 0).into(),
                    1,
                );
                let pdata2 = OtapPdata::new_default(rec2.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::new_with(1, 0).into(),
                    1,
                );

                // First request succeeds.
                ctx.process(Message::PData(pdata1))
                    .await
                    .expect("first input succeeds");

                // Second request is not an engine error.
                ctx.process(Message::PData(pdata2))
                    .await
                    .expect("second input succeeds (nacked, not engine error)");

                let mut nack_count = 0;
                while let Ok(msg) = pipeline_completion_rx.try_recv() {
                    if let PipelineCompletionMsg::DeliverNack { nack } = msg {
                        assert!(nack.reason.contains("inbound routes exhausted"));
                        nack_count += 1;
                    }
                }
                assert_eq!(nack_count, 1, "exactly one request should be nacked");
            })
            .validate(|_| async {});
    }

    /// When outbound_request_limit is 1 and a flush produces multiple
    /// outbound batches, failure to allocate the outbound slot is Nacked.
    #[test]
    fn test_outbound_slot_exhaustion_nacks() {
        // max_size=1 forces each item into its own outbound batch,
        // so a single input with multiple items needs multiple outbound slots.
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 1,
                "max_size": 1,
                "sizer": "items",
            },
            "max_batch_duration": "1s",
            "outbound_request_limit": 1,
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(10);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let mut datagen = DataGenerator::new(1);
                let logs1 = datagen.generate_logs();

                let rec1 = encode_logs_otap_batch(&logs1).expect("encode");

                // Subscribe for ack/nack.
                let pdata1 = OtapPdata::new_default(rec1.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::new_with(0, 0).into(),
                    1,
                );

                // Process triggers flush.
                ctx.process(Message::PData(pdata1))
                    .await
                    .expect("process succeeds (nack, not engine error)");

                let mut nack_count = 0;
                while let Ok(msg) = pipeline_completion_rx.try_recv() {
                    if let PipelineCompletionMsg::DeliverNack { nack } = msg {
                        assert!(nack.reason.contains("outbound routes exhausted"));
                        nack_count += 1;
                    }
                }

                assert_eq!(nack_count, 1);
            })
            .validate(|_| async {});
    }

    /// Reads the current value of a `U64` counter metric from the registry.
    fn counter_metric_value(
        telemetry_registry: &TelemetryRegistryHandle,
        set_name: &str,
        metric_name: &str,
    ) -> u64 {
        let mut value = 0u64;
        telemetry_registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == set_name {
                for (field, metric_value) in iter {
                    if field.name == metric_name
                        && let otel_arrow_dfe_telemetry::metrics::MetricValue::U64(v) = metric_value
                    {
                        value = *v;
                    }
                }
            }
        });
        value
    }

    /// Builds an OTLP logs request wrapping a single `ResourceLogs` / single
    /// `ScopeLogs` with `count` sizable records, so its encoded form far
    /// exceeds a small `max_size` and can only be split *within* the resource
    /// entry (the case the byte splitter's sub-resource splitting handles).
    fn single_resource_logs_bytes(count: usize) -> OtlpProtoBytes {
        let records: Vec<LogRecord> = (0..count)
            .map(|i| LogRecord {
                severity_text: format!("record-{i:02}-{}", "x".repeat(40)),
                ..Default::default()
            })
            .collect();
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: records,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        otlp_message_to_bytes(&OtlpProtoMessage::Logs(logs))
    }

    /// Builds an OTLP logs request wrapping a single `ResourceLogs` / single
    /// `ScopeLogs` whose `InstrumentationScope` name is `header_len` bytes, with
    /// `count` tiny records. The scope *header* is large relative to the records,
    /// so splitting at record granularity re-encodes that big header per record
    /// -- amplifying the output bytes even though the fragment count stays low.
    /// This isolates the byte-amplification (overhead) budget from the fragment
    /// budget.
    fn single_resource_logs_big_scope_header(count: usize, header_len: usize) -> OtlpProtoBytes {
        let records: Vec<LogRecord> = (0..count)
            .map(|_| LogRecord {
                severity_text: "x".to_string(),
                ..Default::default()
            })
            .collect();
        let logs = LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "h".repeat(header_len),
                        ..Default::default()
                    }),
                    log_records: records,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        otlp_message_to_bytes(&OtlpProtoMessage::Logs(logs))
    }

    /// Scenario: a subscribed oversize single-resource logs input is flushed
    /// immediately (max_batch_duration = 0), splitting into multiple fragments;
    /// every fragment except the last is Acked and the last fragment is Nacked.
    ///
    /// Guarantees: Ack/Nack ownership follows the original input ranges even
    /// when header duplication inflates the fragments' encoded size, so the
    /// final fragment is subscribed and
    /// attributed back to the input. The input therefore receives exactly one
    /// Nack and is never Acked -- no fragment escapes tracking.
    #[test]
    fn test_split_fragment_nack_propagates_to_input() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otlp": {
                "min_size": null,
                "max_size": 100,
                "sizer": "bytes",
            },
            "format": "otlp",
            "max_batch_duration": "0s",
        }));

        phase
            .run_test(move |mut ctx| async move {
                let (pipeline_completion_tx, mut pipeline_completion_rx) =
                    pipeline_completion_msg_channel(16);
                ctx.set_pipeline_completion_sender(pipeline_completion_tx);

                let bytes = single_resource_logs_bytes(8);
                let pdata = OtapPdata::new_default(bytes.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::new_with(0, 0).into(),
                    1,
                );

                // Immediate flush (max_batch_duration = 0) splits the oversize
                // single resource into multiple fragments in one shot.
                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");

                let outputs = ctx.drain_pdata().await;
                assert!(
                    outputs.len() > 1,
                    "oversize single resource must split into multiple fragments, got {}",
                    outputs.len()
                );

                // Ack every fragment except the last, which is Nacked.
                let last = outputs.len() - 1;
                for (i, out) in outputs.into_iter().enumerate() {
                    assert!(
                        out.has_ack_or_nack_interests(),
                        "every fragment must be subscribed for ack/nack"
                    );
                    if i == last {
                        ctx.process(Message::Control(NodeControlMsg::Nack(
                            next_nack(NackMsg::new("downstream failed", out))
                                .expect("has subs")
                                .1,
                        )))
                        .await
                        .expect("process nack");
                    } else {
                        ctx.process(Message::Control(NodeControlMsg::Ack(
                            next_ack(AckMsg::new(out)).expect("has subs").1,
                        )))
                        .await
                        .expect("process ack");
                    }
                }

                // The input must be Nacked exactly once and never Acked.
                let mut acks = 0;
                let mut nacks = 0;
                while let Ok(msg) = pipeline_completion_rx.try_recv() {
                    match msg {
                        PipelineCompletionMsg::DeliverAck { .. } => acks += 1,
                        PipelineCompletionMsg::DeliverNack { .. } => nacks += 1,
                    }
                }
                assert_eq!(acks, 0, "input must not be acked when a fragment nacks");
                assert_eq!(nacks, 1, "input must be nacked exactly once");
            })
            .validate(|_| async {});
    }

    /// Scenario: a single oversize `ResourceLogs` would split into more
    /// fragments than the configured `max_split_fragments` budget allows.
    ///
    /// Guarantees: the entry is emitted whole (best-effort, exceeding
    /// `max_size`) rather than fanning out unboundedly, exactly one output is
    /// produced, and the `split.budget.fallbacks` counter records the fallback.
    #[test]
    fn test_split_fragment_budget_fallback_emits_whole() {
        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otlp": {
                "min_size": null,
                "max_size": 100,
                "sizer": "bytes",
                "max_split_fragments": 2,
            },
            "format": "otlp",
            "max_batch_duration": "0s",
        }));

        phase
            .run_test(move |mut ctx| async move {
                // 8 records project to 8 fragments, exceeding the budget of 2.
                let bytes = single_resource_logs_bytes(8);
                let input_bytes = bytes.num_bytes();
                let pdata = OtapPdata::new_default(bytes.into());

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");

                let outputs = ctx.drain_pdata().await;
                assert_eq!(
                    outputs.len(),
                    1,
                    "entry over the fragment budget must be emitted whole, not split"
                );
                let out_bytes = outputs[0]
                    .clone()
                    .payload()
                    .num_bytes()
                    .expect("otlp bytes size known");
                assert_eq!(
                    out_bytes, input_bytes,
                    "the whole entry is emitted unchanged (and exceeds max_size)"
                );
                assert!(out_bytes > 100, "the whole entry exceeds max_size");

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert_eq!(
                    counter_metric_value(
                        &telemetry_registry,
                        "otap.processor.batch",
                        "split.budget.fallbacks"
                    ),
                    1,
                    "exactly one budget fallback should be recorded"
                );
            });
    }

    /// Scenario: a single oversize `ResourceLogs` has only a few records (well
    /// under `max_split_fragments`) but a very large scope header, so splitting
    /// at record granularity would re-encode that header per fragment and
    /// amplify the output bytes past `max_split_overhead_bytes`.
    ///
    /// Guarantees: the byte-amplification budget (independent of the fragment
    /// count) triggers the fallback -- the entry is emitted whole, exactly one
    /// output is produced, and `split.budget.fallbacks` records it.
    #[test]
    fn test_split_overhead_budget_fallback_emits_whole() {
        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otlp": {
                "min_size": null,
                "max_size": 100,
                "sizer": "bytes",
                // Fragment budget is generous; only the overhead budget bites.
                "max_split_fragments": 1000,
                "max_split_overhead_bytes": 1000,
            },
            "format": "otlp",
            "max_batch_duration": "0s",
        }));

        phase
            .run_test(move |mut ctx| async move {
                // 6 records * ~4 KiB scope header re-encoded per record projects
                // to ~24 KiB of duplicated wrapper bytes, over the 1000-byte
                // overhead budget, while only 6 fragments stay under the
                // fragment budget of 1000.
                let bytes = single_resource_logs_big_scope_header(6, 4096);
                let input_bytes = bytes.num_bytes();
                let pdata = OtapPdata::new_default(bytes.into());

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");

                let outputs = ctx.drain_pdata().await;
                assert_eq!(
                    outputs.len(),
                    1,
                    "entry over the overhead budget must be emitted whole, not split"
                );
                let out_bytes = outputs[0]
                    .clone()
                    .payload()
                    .num_bytes()
                    .expect("otlp bytes size known");
                assert_eq!(
                    out_bytes, input_bytes,
                    "the whole entry is emitted unchanged (and exceeds max_size)"
                );
                assert!(out_bytes > 100, "the whole entry exceeds max_size");

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert_eq!(
                    counter_metric_value(
                        &telemetry_registry,
                        "otap.processor.batch",
                        "split.budget.fallbacks"
                    ),
                    1,
                    "exactly one budget fallback should be recorded"
                );
            });
    }

    /// Scenario: an OTLP config omits `max_split_fragments`,
    /// `max_split_overhead_bytes`, and `max_split_fragments_per_flush`.
    ///
    /// Guarantees: serde applies the documented defaults (64k fragments, 8 MiB
    /// overhead, 64k fragments per flush) rather than leaving the fields `None`
    /// (unbounded), so an omitted config is bounded, not unlimited.
    #[test]
    fn test_split_budget_defaults_applied_when_omitted() {
        let config: Config = serde_json::from_value(json!({
            "otlp": {
                "min_size": null,
                "max_size": 1048576,
                "sizer": "bytes",
            },
            "format": "otlp",
            "max_batch_duration": "0s",
        }))
        .expect("config parses");

        assert_eq!(
            config.otlp.max_split_fragments,
            NonZeroU64::new(65_536),
            "omitted max_split_fragments must default to 64k, not None (unbounded)"
        );
        assert_eq!(
            config.otlp.max_split_overhead_bytes,
            NonZeroU64::new(8 * 1024 * 1024),
            "omitted max_split_overhead_bytes must default to 8 MiB, not None (unbounded)"
        );
        assert_eq!(
            config.otlp.max_split_fragments_per_flush,
            NonZeroU64::new(65_536),
            "omitted max_split_fragments_per_flush must default to 64k, not None (unbounded)"
        );
    }

    /// Scenario: a single oversize `ResourceLogs` has a large scope header and
    /// many tiny records that greedily pack several-per-fragment, so the *real*
    /// split produces only a handful of fragments -- but a per-record projection
    /// of the duplicated header would far exceed `max_split_overhead_bytes`.
    ///
    /// Guarantees: the overhead budget is measured from the actual greedy packing
    /// rather than a per-record worst case, so the entry is still split (more than
    /// one output, each within `max_size`) and no `split.budget.fallbacks` is
    /// recorded -- avoiding a false collapse back into one oversize batch that
    /// would recreate the original oversize-request bug.
    #[test]
    fn test_split_overhead_uses_real_packing_not_per_record() {
        let (telemetry_registry, metrics_reporter, phase) = setup_test_runtime(json!({
            "otlp": {
                "min_size": null,
                "max_size": 2000,
                "sizer": "bytes",
                "max_split_fragments": 100000,
                // A per-record projection (records * ~500-byte header) would be
                // hundreds of KiB; the real packing duplicates the header only a
                // handful of times, well under this budget.
                "max_split_overhead_bytes": 100000,
            },
            "format": "otlp",
            "max_batch_duration": "0s",
        }));

        phase
            .run_test(move |mut ctx| async move {
                // ~1200 tiny records under a ~500-byte scope header; at
                // max_size = 2000 many records pack into each fragment, so the
                // real fragment count (and duplicated header bytes) is small.
                let bytes = single_resource_logs_big_scope_header(1200, 500);
                let pdata = OtapPdata::new_default(bytes.into());

                ctx.process(Message::PData(pdata))
                    .await
                    .expect("process input");

                let outputs = ctx.drain_pdata().await;
                assert!(
                    outputs.len() > 1,
                    "records that pack within budget must still be split, got {}",
                    outputs.len()
                );
                for out in &outputs {
                    let out_bytes = out
                        .clone()
                        .payload()
                        .num_bytes()
                        .expect("otlp bytes size known");
                    assert!(
                        out_bytes <= 2000,
                        "each greedy fragment must stay within max_size, got {out_bytes}"
                    );
                }

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert_eq!(
                    counter_metric_value(
                        &telemetry_registry,
                        "otap.processor.batch",
                        "split.budget.fallbacks"
                    ),
                    0,
                    "the real packing stays under the overhead budget, so no fallback"
                );
            });
    }

    /// Scenario: every input merged into one output batch carries the same
    /// `peer_addr`. Guarantee: the flushed batch's `peer_addr` equals that
    /// common address (single-peer batches keep working as if no merge
    /// happened).
    #[test]
    fn test_flush_peer_addr_preserved_when_single_peer() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer: SocketAddr = "10.0.0.1:5000".parse().unwrap();
                let mut datagen = DataGenerator::new(1);

                // Send enough inputs to size-trigger one output batch.
                for _ in 0..2 {
                    let rec =
                        encode_logs_otap_batch(&datagen.generate_logs()).expect("encode logs");
                    let pdata = OtapPdata::new_default(rec.into()).with_peer_addr(peer);
                    ctx.process(Message::PData(pdata))
                        .await
                        .expect("process input");
                }

                let outputs = ctx.drain_pdata().await;
                assert!(!outputs.is_empty(), "size flush should produce a batch");
                for out in outputs {
                    assert_eq!(
                        out.peer_addr(),
                        Some(peer),
                        "single-peer batch must preserve peer_addr"
                    );
                }
            })
            .validate(|_| async {});
    }

    /// Scenario: inputs merged into one output batch came from distinct
    /// peers. Guarantee: the flushed batch's `peer_addr` is `None` (no
    /// silent misattribution to one arbitrary contributor).
    #[test]
    fn test_flush_peer_addr_dropped_when_mixed_peers() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer_a: SocketAddr = "10.0.0.1:5000".parse().unwrap();
                let peer_b: SocketAddr = "10.0.0.2:5000".parse().unwrap();
                let mut datagen = DataGenerator::new(1);

                for peer in [peer_a, peer_b] {
                    let rec =
                        encode_logs_otap_batch(&datagen.generate_logs()).expect("encode logs");
                    let pdata = OtapPdata::new_default(rec.into()).with_peer_addr(peer);
                    ctx.process(Message::PData(pdata))
                        .await
                        .expect("process input");
                }

                let outputs = ctx.drain_pdata().await;
                assert!(!outputs.is_empty(), "size flush should produce a batch");
                for out in outputs {
                    assert_eq!(
                        out.peer_addr(),
                        None,
                        "mixed-peer batch must not be attributed to one peer"
                    );
                }
            })
            .validate(|_| async {});
    }

    /// Scenario: at least one input has no peer_addr (e.g. came from a
    /// sourceless receiver). Guarantee: the flushed batch's `peer_addr`
    /// is `None`.
    #[test]
    fn test_flush_peer_addr_dropped_when_any_input_peerless() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "1s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer: SocketAddr = "10.0.0.1:5000".parse().unwrap();
                let mut datagen = DataGenerator::new(1);

                // First input has a peer, second does not.
                let rec1 = encode_logs_otap_batch(&datagen.generate_logs()).expect("encode logs");
                ctx.process(Message::PData(
                    OtapPdata::new_default(rec1.into()).with_peer_addr(peer),
                ))
                .await
                .expect("process input");

                let rec2 = encode_logs_otap_batch(&datagen.generate_logs()).expect("encode logs");
                ctx.process(Message::PData(OtapPdata::new_default(rec2.into())))
                    .await
                    .expect("process input");

                let outputs = ctx.drain_pdata().await;
                assert!(!outputs.is_empty(), "size flush should produce a batch");
                for out in outputs {
                    assert_eq!(
                        out.peer_addr(),
                        None,
                        "any peerless contributor must drop peer_addr on the merged batch"
                    );
                }
            })
            .validate(|_| async {});
    }

    /// Build a `LogsData` containing exactly `count` log records, with
    /// unique time_unix_nano values derived from `base_id` for ordering
    /// assertions. Used by the retained-partial tests below.
    fn logs_with_n_records(base_id: usize, count: usize) -> LogsData {
        let base_time = 1_000_000_000 + (base_id * 1000) as u64;
        LogsData {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: (0..count)
                        .map(|i| LogRecord {
                            time_unix_nano: base_time + i as u64,
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// take_remaining retained-partial path, single contributing input.
    ///
    /// Scenario: one 8-item input from peer A -> `make_batches` splits into
    /// `[5, 3]`. The 3-item tail is below `min_size=4` and is re-buffered
    /// via `take_remaining`. A follow-up 2-item input from peer A then
    /// triggers a 5-item size flush whose contents come from the retained
    /// portion (peer A) + the new input (peer A).
    ///
    /// Guarantee: the second flush's batch carries `peer_addr = A` -- the
    /// retained portion correctly remembered its single contributor's peer.
    #[test]
    fn test_take_remaining_preserves_peer_addr_single_input() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "10s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer: SocketAddr = "10.0.0.1:5000".parse().unwrap();

                // 8-item input from peer A -> batches [5, 3]; the 3-item
                // tail is retained because 3 < min_size=4.
                let big = encode_logs_otap_batch(&logs_with_n_records(0, 8)).expect("encode logs");
                ctx.process(Message::PData(
                    OtapPdata::new_default(big.into()).with_peer_addr(peer),
                ))
                .await
                .expect("process big input");

                let first = ctx.drain_pdata().await;
                assert_eq!(first.len(), 1, "first flush should emit one batch");
                assert_eq!(
                    first[0].peer_addr(),
                    Some(peer),
                    "single-peer first batch must preserve peer_addr"
                );

                // 2-item follow-up from peer A: retained(3, A) + new(2, A) = 5 -> flush.
                let extra =
                    encode_logs_otap_batch(&logs_with_n_records(1, 2)).expect("encode logs");
                ctx.process(Message::PData(
                    OtapPdata::new_default(extra.into()).with_peer_addr(peer),
                ))
                .await
                .expect("process follow-up input");

                let second = ctx.drain_pdata().await;
                assert_eq!(second.len(), 1, "second flush should emit one batch");
                assert_eq!(
                    second[0].peer_addr(),
                    Some(peer),
                    "single-input retained partial must preserve peer_addr on the next merge"
                );
            })
            .validate(|_| async {});
    }

    /// take_remaining retained-partial path, followed by a merge of several
    /// inputs that all share one peer.
    ///
    /// Scenario: inputs of sizes [3, 3] from peer A size-flush at 6 items
    /// -> `make_batches` splits into `[5, 1]` and input #2's final item is
    /// retained. A 2-item input from peer A is buffered (3 < min_size), and
    /// a follow-up 2-item input from peer A triggers a second size flush of
    /// the retained item plus both new inputs.
    ///
    /// Guarantee: the second flush's batch carries `peer_addr = A`, even
    /// though it merges the retained portion with more than one input --
    /// because all contributors agreed.
    #[test]
    fn test_take_remaining_preserves_peer_addr_multi_input_same_peer() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "10s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer: SocketAddr = "10.0.0.1:5000".parse().unwrap();

                for (idx, count) in [(0, 3usize), (1, 3), (2, 2)] {
                    let rec = encode_logs_otap_batch(&logs_with_n_records(idx, count))
                        .expect("encode logs");
                    ctx.process(Message::PData(
                        OtapPdata::new_default(rec.into()).with_peer_addr(peer),
                    ))
                    .await
                    .expect("process input");
                }

                let first = ctx.drain_pdata().await;
                assert_eq!(first.len(), 1, "first flush should emit one batch");
                assert_eq!(first[0].peer_addr(), Some(peer));

                let extra =
                    encode_logs_otap_batch(&logs_with_n_records(3, 2)).expect("encode logs");
                ctx.process(Message::PData(
                    OtapPdata::new_default(extra.into()).with_peer_addr(peer),
                ))
                .await
                .expect("process follow-up input");

                let second = ctx.drain_pdata().await;
                assert_eq!(second.len(), 1, "second flush should emit one batch");
                assert_eq!(
                    second[0].peer_addr(),
                    Some(peer),
                    "multi-input retained partial with one shared peer must preserve it"
                );
            })
            .validate(|_| async {});
    }

    /// take_remaining retained-partial path, followed by a merge of inputs
    /// from distinct peers.
    ///
    /// Scenario: inputs of sizes [3, 3] from peer A size-flush at 6 items
    /// -> `make_batches` splits into `[5, 1]` and input #2's final item
    /// (peer A) is retained. A 2-item input from peer B is buffered, and a
    /// follow-up 2-item input from peer A triggers a second size flush that
    /// merges the retained item (A), the peer B input, and the new input (A).
    ///
    /// Guarantee: the second flush's batch carries `peer_addr = None` --
    /// the retained portion merged with a different peer is not
    /// misattributed to one arbitrary contributor.
    #[test]
    fn test_take_remaining_drops_peer_addr_multi_input_different_peers() {
        let (_telemetry_registry, _metrics_reporter, phase) = setup_test_runtime(json!({
            "otap": {
                "min_size": 4,
                "max_size": 5,
                "sizer": "items",
            },
            "max_batch_duration": "10s"
        }));

        phase
            .run_test(move |mut ctx| async move {
                let peer_a: SocketAddr = "10.0.0.1:5000".parse().unwrap();
                let peer_b: SocketAddr = "10.0.0.2:5000".parse().unwrap();

                for (idx, count, peer) in [(0usize, 3usize, peer_a), (1, 3, peer_a), (2, 2, peer_b)]
                {
                    let rec = encode_logs_otap_batch(&logs_with_n_records(idx, count))
                        .expect("encode logs");
                    ctx.process(Message::PData(
                        OtapPdata::new_default(rec.into()).with_peer_addr(peer),
                    ))
                    .await
                    .expect("process input");
                }

                let _first = ctx.drain_pdata().await;

                let extra =
                    encode_logs_otap_batch(&logs_with_n_records(3, 2)).expect("encode logs");
                ctx.process(Message::PData(
                    OtapPdata::new_default(extra.into()).with_peer_addr(peer_a),
                ))
                .await
                .expect("process follow-up input");

                let second = ctx.drain_pdata().await;
                assert_eq!(second.len(), 1, "second flush should emit one batch");
                assert_eq!(
                    second[0].peer_addr(),
                    None,
                    "retained partial spanning distinct peers must not be attributed to one"
                );
            })
            .validate(|_| async {});
    }

    // ---------------------------------------------------------------------
    // Ack/Nack tracking of a retained residual. The same scenario is run for
    // OTAP (items sizer) and OTLP (bytes sizer) inputs.
    // ---------------------------------------------------------------------

    /// Input format fed to [`check_residual_holds_input`].
    #[derive(Clone, Copy)]
    enum ResidualFormat {
        Otap,
        Otlp,
        EncodedOtlp,
    }

    /// Logs with `units` resource entries holding one log record each. Every
    /// entry encodes to the same OTLP size (`time_unix_nano` is fixed64), so
    /// one entry is one sizer unit for both formats: 1 item for OTAP and
    /// [`residual_unit_size`] bytes for OTLP. The OTLP batcher packs whole
    /// resource entries, so both formats produce the same batch layout.
    fn residual_logs(base_id: usize, units: usize) -> LogsData {
        LogsData {
            resource_logs: (0..units)
                .map(|i| ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            time_unix_nano: 1_000_000_000 + (base_id * 1000 + i) as u64,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                })
                .collect(),
        }
    }

    /// Size of one [`residual_logs`] unit in the format's sizer unit.
    fn residual_unit_size(fmt: ResidualFormat) -> usize {
        match fmt {
            ResidualFormat::Otap => 1,
            ResidualFormat::Otlp | ResidualFormat::EncodedOtlp => {
                otlp_message_to_bytes(&OtlpProtoMessage::Logs(residual_logs(0, 1))).num_bytes()
            }
        }
    }

    /// Batch processor config with `min_size = max_size = units` sizer units.
    fn residual_config(fmt: ResidualFormat, units: usize) -> Value {
        let size = units * residual_unit_size(fmt);
        match fmt {
            ResidualFormat::Otap => json!({
                "format": "otap",
                "otap": { "min_size": size, "max_size": size, "sizer": "items" },
                "max_batch_duration": "1s"
            }),
            ResidualFormat::Otlp | ResidualFormat::EncodedOtlp => json!({
                "format": "otlp",
                "otlp": { "min_size": size, "max_size": size, "sizer": "bytes" },
                "max_batch_duration": "1s"
            }),
        }
    }

    /// Encode `logs` in `fmt` and subscribe it with calldata index `idx`.
    fn residual_pdata(fmt: ResidualFormat, logs: &LogsData, idx: usize) -> OtapPdata {
        let pdata = match fmt {
            ResidualFormat::Otap => {
                OtapPdata::new_default(encode_logs_otap_batch(logs).expect("encode").into())
            }
            ResidualFormat::Otlp => OtapPdata::new_default(
                otlp_message_to_bytes(&OtlpProtoMessage::Logs(logs.clone())).into(),
            ),
            ResidualFormat::EncodedOtlp => {
                let service = otel_arrow_dfe_pdata_codec::CodecService::new().unwrap();
                let codec = service.registry().resolve(&PdataEncoding::OTLP).unwrap();
                let bytes =
                    otlp_message_to_bytes(&OtlpProtoMessage::Logs(logs.clone())).clone_bytes();
                OtapPdata::new_default(codec.admit(SignalType::Logs, bytes).unwrap().into())
            }
        };
        pdata.test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::new_with(idx as u64, 0).into(),
            1,
        )
    }

    /// Ack/Nack counts delivered upstream, indexed by the input's calldata
    /// index (see [`residual_pdata`]).
    #[derive(Debug, Default)]
    struct Completions {
        acks: Vec<usize>,
        nacks: Vec<usize>,
    }

    impl Completions {
        fn new(n: usize) -> Self {
            Self {
                acks: vec![0; n],
                nacks: vec![0; n],
            }
        }

        fn index(&self, cd: &TestCallData) -> usize {
            (0..self.acks.len())
                .find(|&i| *cd == TestCallData::new_with(i as u64, 0))
                .expect("known calldata")
        }

        fn drain(
            &mut self,
            rx: &mut otel_arrow_dfe_engine::control::PipelineCompletionMsgReceiver<OtapPdata>,
        ) {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    PipelineCompletionMsg::DeliverAck { ack } => {
                        if let Some((_, ack)) = next_ack(ack) {
                            let cd = ack.unwind.route.calldata.try_into().expect("calldata");
                            let i = self.index(&cd);
                            self.acks[i] += 1;
                        }
                    }
                    PipelineCompletionMsg::DeliverNack { nack } => {
                        if let Some((_, nack)) = next_nack(nack) {
                            let cd = nack.unwind.route.calldata.try_into().expect("calldata");
                            let i = self.index(&cd);
                            self.nacks[i] += 1;
                        }
                    }
                }
            }
        }
    }

    /// Deliver an Ack or Nack for `out` back to the processor.
    async fn respond(
        ctx: &mut otel_arrow_dfe_engine::testing::processor::TestContext<OtapPdata>,
        out: OtapPdata,
        policy: &AckPolicy,
    ) {
        let msg = match policy {
            AckPolicy::Ack => NodeControlMsg::Ack(next_ack(AckMsg::new(out)).expect("subs").1),
            AckPolicy::Nack(reason) => {
                NodeControlMsg::Nack(next_nack(NackMsg::new(*reason, out)).expect("subs").1)
            }
        };
        ctx.process(Message::Control(msg))
            .await
            .expect("process response");
    }

    /// Deliver an expired wakeup for every slot, forcing a timer flush.
    async fn flush_timer(
        ctx: &mut otel_arrow_dfe_engine::testing::processor::TestContext<OtapPdata>,
    ) {
        for slot in all_wakeup_slots() {
            ctx.process(Message::Control(NodeControlMsg::Wakeup {
                slot,
                when: Instant::now() + Duration::from_secs(10),
                revision: 0,
            }))
            .await
            .expect("process wakeup");
        }
    }

    /// Shared core for the residual Ack/Nack tests.
    ///
    /// With min_size = max_size = 4 units, input A (3 units) is buffered and
    /// input B (6 units) triggers a size flush of 9 units: outputs [4, 4] are
    /// sent and B's final unit is retained as a residual. Both outputs are
    /// Acked; B must not complete while its residual is buffered. A timer
    /// flush then sends the residual, which must still be tracked, and
    /// `final_response` is applied to it; B must complete accordingly.
    fn check_residual_holds_input(fmt: ResidualFormat, final_response: AckPolicy) {
        let (_registry, _reporter, phase) = setup_test_runtime(residual_config(fmt, 4));
        phase
            .run_test(move |mut ctx| async move {
                let (tx, mut rx) = pipeline_completion_msg_channel(16);
                ctx.set_pipeline_completion_sender(tx);
                let mut done = Completions::new(2);

                ctx.process(Message::PData(residual_pdata(fmt, &residual_logs(0, 3), 0)))
                    .await
                    .expect("process A");
                ctx.process(Message::PData(residual_pdata(fmt, &residual_logs(1, 6), 1)))
                    .await
                    .expect("process B");

                let outs = ctx.drain_pdata().await;
                assert_eq!(outs.len(), 2, "size flush sends [4, 4] and retains 1");
                for out in outs {
                    respond(&mut ctx, out, &AckPolicy::Ack).await;
                }
                done.drain(&mut rx);
                assert_eq!(done.acks, vec![1, 0], "B must wait for its residual");
                assert_eq!(done.nacks, vec![0, 0]);

                flush_timer(&mut ctx).await;
                let outs = ctx.drain_pdata().await;
                assert_eq!(outs.len(), 1, "timer flush sends the residual");
                let residual = outs.into_iter().next().expect("one output");
                assert!(
                    residual.has_ack_or_nack_interests(),
                    "residual output must be tracked back to B"
                );
                let expect_nack = matches!(final_response, AckPolicy::Nack(_));
                respond(&mut ctx, residual, &final_response).await;
                done.drain(&mut rx);

                if expect_nack {
                    assert_eq!(done.acks, vec![1, 0], "B must never be acked");
                    assert_eq!(done.nacks, vec![0, 1], "B nacked exactly once");
                } else {
                    assert_eq!(done.acks, vec![1, 1], "B acked exactly once");
                    assert_eq!(done.nacks, vec![0, 0]);
                }
            })
            .validate(|_| async {});
    }

    /// Scenario: an OTAP size flush sends two full outputs and retains the
    /// final item of input B as a residual; both outputs are Acked, then the
    /// residual is timer-flushed and Acked.
    /// Guarantees: B is not Acked while part of its data is still buffered,
    /// and is Acked exactly once after its residual output is Acked.
    #[test]
    fn test_otap_residual_holds_input_until_flushed() {
        check_residual_holds_input(ResidualFormat::Otap, AckPolicy::Ack);
    }

    /// Scenario: as above for OTAP, but the output carrying B's residual is
    /// Nacked downstream.
    /// Guarantees: the residual output is subscribed on B's behalf, so the
    /// Nack reaches B exactly once and B is never Acked.
    #[test]
    fn test_otap_residual_nack_propagates_to_input() {
        check_residual_holds_input(ResidualFormat::Otap, AckPolicy::Nack("downstream failed"));
    }

    /// Scenario: an OTLP bytes size flush sends two full outputs and retains
    /// input B's final resource entry as a residual; both outputs are Acked,
    /// then the residual is timer-flushed and Acked.
    /// Guarantees: B is not Acked while part of its data is still buffered,
    /// and is Acked exactly once after its residual output is Acked.
    #[test]
    fn test_otlp_residual_holds_input_until_flushed() {
        check_residual_holds_input(ResidualFormat::Otlp, AckPolicy::Ack);
    }

    /// Scenario: as above for OTLP, but the output carrying B's residual is
    /// Nacked downstream.
    /// Guarantees: the residual output is subscribed on B's behalf, so the
    /// Nack reaches B exactly once and B is never Acked.
    #[test]
    fn test_otlp_residual_nack_propagates_to_input() {
        check_residual_holds_input(ResidualFormat::Otlp, AckPolicy::Nack("downstream failed"));
    }

    /// Scenario: Codec-native encoded OTLP batching retains a residual after size-triggered output.
    /// Guarantees: The input is acknowledged only after its buffered residual is flushed and acknowledged.
    #[test]
    fn test_encoded_otlp_residual_holds_input_until_flushed() {
        check_residual_holds_input(ResidualFormat::EncodedOtlp, AckPolicy::Ack);
    }

    /// Scenario: A downstream consumer Nacks a residual retained by codec-native OTLP batching.
    /// Guarantees: The residual keeps its original input subscription and propagates the downstream Nack.
    #[test]
    fn test_encoded_otlp_residual_nack_propagates_to_input() {
        check_residual_holds_input(
            ResidualFormat::EncodedOtlp,
            AckPolicy::Nack("downstream failed"),
        );
    }

    // ---------------------------------------------------------------------
    // Pass-through of inputs already within [min_size, max_size]. Uses the
    // residual helpers above: one `residual_logs` unit is one sizer unit in
    // both formats.
    // ---------------------------------------------------------------------

    /// Batch processor config for `fmt` with the given min/max in units and
    /// a 1s timeout. `None` leaves the field unset.
    fn passthrough_config(
        fmt: ResidualFormat,
        min_units: Option<usize>,
        max_units: Option<usize>,
        timeout: &str,
    ) -> Value {
        let unit = residual_unit_size(fmt);
        let min = min_units.map(|u| u * unit);
        let max = max_units.map(|u| u * unit);
        match fmt {
            ResidualFormat::Otap => json!({
                "format": "otap",
                "otap": { "min_size": min, "max_size": max, "sizer": "items" },
                "max_batch_duration": timeout
            }),
            ResidualFormat::Otlp | ResidualFormat::EncodedOtlp => json!({
                "format": "otlp",
                "otlp": { "min_size": min, "max_size": max, "sizer": "bytes" },
                "max_batch_duration": timeout
            }),
        }
    }

    /// Size of an output in `fmt` units.
    fn output_units(fmt: ResidualFormat, out: &OtapPdata) -> usize {
        let mut out = out.clone();
        match fmt {
            ResidualFormat::Otap => out.num_items(),
            ResidualFormat::Otlp | ResidualFormat::EncodedOtlp => {
                // SAFETY: callers use the matching OTLP-output test configuration;
                // both legacy and generalized encoded OTLP report Some(buffer.len()).
                // A panic here indicates a fixture or output-format regression.
                out.num_bytes().expect("otlp size") / residual_unit_size(fmt)
            }
        }
    }

    /// Deliver `policy` for an output that was passed through: its next
    /// subscriber is the original input (node 1), not the batch processor,
    /// so the response is routed straight upstream without calling the
    /// processor. Returns the upstream calldata index.
    fn respond_passthrough(out: OtapPdata, policy: &AckPolicy) -> (bool, TestCallData) {
        match policy {
            AckPolicy::Ack => {
                let (node, ack) = next_ack(AckMsg::new(out)).expect("subs");
                assert_eq!(node, 1, "passed-through output routes to the input");
                (true, ack.unwind.route.calldata.try_into().expect("cd"))
            }
            AckPolicy::Nack(reason) => {
                let (node, nack) = next_nack(NackMsg::new(*reason, out)).expect("subs");
                assert_eq!(node, 1, "passed-through output routes to the input");
                (false, nack.unwind.route.calldata.try_into().expect("cd"))
            }
        }
    }

    /// A shared helper that exercises the below scenario for multiple formats
    /// and with either an ack or nack.
    ///
    /// Scenario: min=4/max=8; a 2-item input is buffered, then a
    /// 5-item input arrives and either a downstream Ack or downstream nack
    /// are delivered.
    /// Guarantees: the in-range input is forwarded immediately and unchanged
    /// with its original context, so its Ack/nack routes straight upstream with
    /// no batch-processor tracking. The buffered input is untouched and later
    /// timer-flushed through the tracked path; `passthrough.batches` counts
    /// only the in-range input.
    fn test_in_range_input_passes_through(fmt: ResidualFormat, response: AckPolicy) {
        let (registry, reporter, phase) =
            setup_test_runtime(passthrough_config(fmt, Some(4), Some(8), "1s"));
        phase
            .run_test(move |mut ctx| async move {
                let (tx, mut rx) = pipeline_completion_msg_channel(16);
                ctx.set_pipeline_completion_sender(tx);
                let mut done = Completions::new(2);

                ctx.process(Message::PData(residual_pdata(fmt, &residual_logs(0, 2), 0)))
                    .await
                    .expect("process A");
                assert!(ctx.drain_pdata().await.is_empty(), "A is buffered");

                let input = residual_pdata(fmt, &residual_logs(1, 5), 1);
                let input_format = input.payload_ref().format();
                let input_buffer = input
                    .payload_ref()
                    .encoded_bytes()
                    .map(|bytes| (bytes.as_ptr() as usize, bytes.len()));
                ctx.process(Message::PData(input)).await.expect("process B");
                let outs = ctx.drain_pdata().await;
                assert_eq!(outs.len(), 1, "B is forwarded immediately, A stays");
                let b = outs.into_iter().next().expect("B");
                assert_eq!(output_units(fmt, &b), 5, "B is forwarded unchanged");
                assert_eq!(b.payload_ref().format(), input_format);
                assert_eq!(
                    b.payload_ref()
                        .encoded_bytes()
                        .map(|bytes| (bytes.as_ptr() as usize, bytes.len())),
                    input_buffer,
                    "encoded passthrough retains the original buffer"
                );

                let is_ack = matches!(response, AckPolicy::Ack);
                let (acked, cd) = respond_passthrough(b, &response);
                assert_eq!(acked, is_ack);
                assert_eq!(cd, TestCallData::new_with(1, 0), "B's own calldata");

                // The processor saw nothing from B's completion.
                done.drain(&mut rx);
                assert_eq!(done.acks, vec![0, 0]);
                assert_eq!(done.nacks, vec![0, 0]);

                flush_timer(&mut ctx).await;
                let outs = ctx.drain_pdata().await;
                assert_eq!(outs.len(), 1, "timer flushes buffered A");
                let a = outs.into_iter().next().expect("A");
                assert_eq!(output_units(fmt, &a), 2);
                // A was buffered, so it is a regular tracked batch: its Ack
                // goes through the batch processor, which then Acks A.
                respond(&mut ctx, a, &AckPolicy::Ack).await;
                done.drain(&mut rx);
                assert_eq!(done.acks, vec![1, 0], "A acked via the processor");
                assert_eq!(done.nacks, vec![0, 0]);

                ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: reporter,
                }))
                .await
                .expect("collect telemetry");
            })
            .validate(move |_| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert_eq!(
                    counter_metric_value(&registry, "otap.processor.batch", "passthrough.batches"),
                    1,
                    "only B (in range on arrival) is passed through"
                );
            });
    }

    /// Scenario: See [`test_in_range_input_passes_through`] for OTAP + Ack
    /// Guarantees: See [`test_in_range_input_passes_through`] for OTAP + Ack
    #[test]
    fn test_otap_in_range_input_passes_through() {
        test_in_range_input_passes_through(ResidualFormat::Otap, AckPolicy::Ack);
    }

    /// Scenario: See [`test_in_range_input_passes_through`] for OTAP + Nack
    /// Guarantees: See [`test_in_range_input_passes_through`] for OTAP + Nack
    #[test]
    fn test_otap_in_range_input_nack_routes_upstream() {
        test_in_range_input_passes_through(ResidualFormat::Otap, AckPolicy::Nack("failed"));
    }

    /// Scenario: See [`test_in_range_input_passes_through`] for OTLP + Ack
    /// Guarantees: See [`test_in_range_input_passes_through`] for OTLP + Ack
    #[test]
    fn test_otlp_in_range_input_passes_through() {
        for fmt in [ResidualFormat::Otlp, ResidualFormat::EncodedOtlp] {
            test_in_range_input_passes_through(fmt, AckPolicy::Ack);
        }
    }

    /// Scenario: See [`test_in_range_input_passes_through`] for OTLP + Nack
    /// Guarantees: See [`test_in_range_input_passes_through`] for OTLP + Nack
    #[test]
    fn test_otlp_in_range_input_nack_routes_upstream() {
        for fmt in [ResidualFormat::Otlp, ResidualFormat::EncodedOtlp] {
            test_in_range_input_passes_through(fmt, AckPolicy::Nack("failed"));
        }
    }

    /// Scenario: An encoder rejects two inputs, either separately in passthrough or merged on a timer flush.
    /// Guarantees: Each original owner receives exactly one permanent Nack with its own route;
    /// buffering cannot turn the same conversion failure into a retryable rejection,
    /// and a flush with no emitted batches records zero output bytes and batches.
    #[test]
    fn output_conversion_failures_permanently_nack_every_input() {
        for min_size in [0, 8] {
            let (registry, reporter, phase) = setup_test_runtime(json!({
                "format": "test-failing-output",
                "codecs": { "test-failing-output": { "min_size": min_size, "sizer": "items" } },
                "max_batch_duration": "1s"
            }));
            phase
                .run_test(move |mut ctx| async move {
                    let (completion_tx, mut completion_rx) = pipeline_completion_msg_channel(10);
                    ctx.set_pipeline_completion_sender(completion_tx);
                    for index in 0..2 {
                        let input =
                            residual_pdata(ResidualFormat::Otap, &residual_logs(index, 2), index);
                        ctx.process(Message::PData(input)).await.unwrap();
                    }
                    if min_size > 0 {
                        assert!(ctx.fire_wakeup().await.unwrap());
                    }
                    assert!(ctx.drain_pdata().await.is_empty());
                    for index in 0..2 {
                        let completion = next_completion(
                            &mut completion_rx,
                            Duration::from_secs(1),
                            "output conversion rejection",
                        )
                        .await;
                        let PipelineCompletionMsg::DeliverNack { nack } = completion else {
                            panic!("expected permanent Nack, got {completion:?}");
                        };
                        assert!(nack.permanent, "min_size={min_size}");
                        assert!(nack.reason.contains("unencodable test payload"));
                        let (node, nack) = next_nack(nack).unwrap();
                        assert_eq!(node, 1);
                        assert_eq!(
                            TestCallData::try_from(nack.unwind.route.calldata).unwrap(),
                            TestCallData::new_with(index, 0),
                        );
                    }
                    assert!(completion_rx.try_recv().is_err());
                    if min_size > 0 {
                        ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter,
                        }))
                        .await
                        .unwrap();
                        assert_flush_output_metrics(&registry, 0, 0).await;
                    }
                })
                .validate(|_| async {});
        }
    }

    /// Scenario: One native input is split into native, OTLP, extension, or partially rejected outputs.
    /// Guarantees: Each flush records one observation containing only successfully emitted batches
    /// and their final-representation bytes, excluding intermediate Arrow sizes and rejected outputs.
    #[test]
    fn flush_output_metrics_measure_final_successful_outputs() {
        for format in ["otap", "otlp", "test-otlp-codec", "test-failing-output"] {
            let (registry, reporter, phase) = setup_test_runtime(json!({
                "format": format,
                "codecs": { (format): { "min_size": 0, "max_size": 4, "sizer": "items" } },
                "max_batch_duration": "0s"
            }));
            phase
                .run_test(move |mut ctx| async move {
                    let records = encode_logs_otap_batch(&residual_logs(0, 9)).unwrap();
                    ctx.process(Message::PData(OtapPdata::new_default(records.into())))
                        .await
                        .unwrap();
                    let outputs = ctx.drain_pdata().await;
                    // The failing encoder rejects the two four-record batches,
                    // but still emits the final one-record batch.
                    let batches = if format == "test-failing-output" {
                        1
                    } else {
                        3
                    };
                    assert_eq!(outputs.len(), batches, "{format}");
                    let bytes = outputs
                        .iter()
                        .map(|output| {
                            let payload = output.payload_ref();
                            if format == "otap" {
                                payload.measured_bytes().unwrap()
                            } else {
                                payload.encoded_bytes().unwrap().len()
                            }
                        })
                        .sum();
                    ctx.process(Message::Control(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: reporter,
                    }))
                    .await
                    .unwrap();
                    assert_flush_output_metrics(&registry, batches, bytes).await;
                })
                .validate(|_| async {});
        }
    }

    /// Scenario: An in-range native batch targets a codec that batches through OTAP.
    /// Guarantees: Passthrough applies the requested output encoding and keeps the input's Ack route.
    #[test]
    fn test_fallback_passthrough_finishes_output_encoding() {
        let (_, _, phase) = setup_test_runtime(json!({
            "format": "test-otlp-codec",
            "codecs": { "test-otlp-codec": { "min_size": 0, "sizer": "items" } },
            "max_batch_duration": "0s"
        }));
        phase
            .run_test(|mut ctx| async move {
                let logs = residual_logs(0, 2);
                let input = residual_pdata(ResidualFormat::Otap, &logs, 0);
                ctx.process(Message::PData(input)).await.unwrap();
                let mut outputs = ctx.drain_pdata().await;
                assert_eq!(outputs.len(), 1);
                let output = outputs.pop().unwrap();
                assert_eq!(output.payload_ref().format().name(), "test-otlp-codec");
                assert_equivalent(
                    &[OtlpProtoMessage::Logs(logs)],
                    &[otap_pdata_to_message(&output)],
                );
                let (acked, calldata) = respond_passthrough(output, &AckPolicy::Ack);
                assert!(acked);
                assert_eq!(calldata, TestCallData::new_with(0, 0));
            })
            .validate(|_| async {});
    }

    /// Drives `inputs` (sizes in units) through a processor configured with
    /// `config`, without any timer event, and returns the output sizes.
    fn outputs_for_inputs(fmt: ResidualFormat, config: Value, inputs: &[usize]) -> Vec<usize> {
        let (_registry, _reporter, phase) = setup_test_runtime(config);
        let inputs = inputs.to_vec();
        let (tx, rx) = std::sync::mpsc::channel();
        phase
            .run_test(move |mut ctx| async move {
                let mut sizes = Vec::new();
                for (i, units) in inputs.into_iter().enumerate() {
                    ctx.process(Message::PData(residual_pdata(
                        fmt,
                        &residual_logs(i, units),
                        i,
                    )))
                    .await
                    .expect("process");
                    for out in ctx.drain_pdata().await {
                        sizes.push(output_units(fmt, &out));
                    }
                }
                tx.send(sizes).expect("send");
            })
            .validate(|_| async {});
        rx.recv().expect("sizes")
    }

    /// Scenario: `max_size` only (min unset) with a timeout; inputs of 3
    /// units (below max) and then 4 units (equal to max) with max=4.
    /// Guarantees: an unset min_size defaults to max_size, so the input
    /// below max is still buffered (no behavior change for max-only
    /// configs), while the input equal to max is passed through as-is.
    #[test]
    fn test_max_only_buffers_below_max_passes_equal() {
        for fmt in [
            ResidualFormat::Otap,
            ResidualFormat::Otlp,
            ResidualFormat::EncodedOtlp,
        ] {
            let cfg = passthrough_config(fmt, None, Some(4), "1s");
            assert_eq!(
                outputs_for_inputs(fmt, cfg, &[3, 4]),
                vec![4],
                "3 is buffered; 4 passes through"
            );
        }
    }

    /// Scenario: `min_size` only (no max) with a timeout; a 2-unit input
    /// followed by a 10-unit input with min=4.
    /// Guarantees: an input at or above min_size is passed through even
    /// though smaller data is buffered, and the buffered data is not merged
    /// into it.
    #[test]
    fn test_min_only_passes_through_large_input() {
        for fmt in [
            ResidualFormat::Otap,
            ResidualFormat::Otlp,
            ResidualFormat::EncodedOtlp,
        ] {
            let cfg = passthrough_config(fmt, Some(4), None, "1s");
            assert_eq!(outputs_for_inputs(fmt, cfg, &[2, 10]), vec![10]);
        }
    }

    /// Scenario: `min_size: 0` with max=4 and no timeout; inputs of 1, 3, 4
    /// and 9 units.
    /// Guarantees: every input up to max_size is forwarded as-is without a
    /// timer, and an input above max_size is still split exactly as before
    /// (9 -> [4, 4, 1]).
    #[test]
    fn test_min_zero_passes_small_and_splits_oversize() {
        for fmt in [
            ResidualFormat::Otap,
            ResidualFormat::Otlp,
            ResidualFormat::EncodedOtlp,
        ] {
            let cfg = passthrough_config(fmt, Some(0), Some(4), "0s");
            assert_eq!(
                outputs_for_inputs(fmt, cfg, &[1, 3, 4, 9]),
                vec![1, 3, 4, 4, 4, 1]
            );
        }
    }

    /// Scenario: `min_size: 0` and no `max_size` for OTAP (valid, with a
    /// build-time warning); inputs of 1 and 50 items arrive.
    /// Guarantees: the processor builds successfully and forwards every
    /// input unchanged and in order.
    #[test]
    fn test_min_zero_no_max_forwards_everything() {
        let fmt = ResidualFormat::Otap;
        let cfg = json!({
            "format": "otap",
            "otap": { "min_size": 0, "max_size": null, "sizer": "items" },
            "max_batch_duration": "0s"
        });
        assert_eq!(outputs_for_inputs(fmt, cfg, &[1, 50]), vec![1, 50]);
    }
}
