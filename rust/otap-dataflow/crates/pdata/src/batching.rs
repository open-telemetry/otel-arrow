// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Representation-independent batching plans and codec batching contracts.
//!
//! This layer owns conversion, sizing, and in-line merge/split operations.
//! Scheduling, bounded request tracking, and Ack/Nack routing belong to the node.

use std::num::{NonZeroU64, NonZeroUsize};

use bytes::Bytes;
use otel_arrow_dfe_config::{EncodeOptions, SignalType};
use serde::{Deserialize, Serialize};

use crate::OtapPayload;
pub use crate::PdataFormat;
use crate::codec::{
    self, CodecDirection, CodecExecutor, CodecState, EncodedPdata, EncodingPlan, PdataEncoding,
    ResolvedCodec,
};
use crate::error::Error;

/// Unit used for flush thresholds, splitting, and input ownership attribution.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BatchSizer {
    /// Original request count (reserved, not currently supported).
    Requests,
    /// Log records, spans, or metric data points.
    Items,
    /// Encoded bytes, including format wrappers.
    Bytes,
}

impl BatchSizer {
    /// Measures an input already prepared by its batching plan.
    pub fn batch_size(&self, payload: &OtapPayload) -> Result<usize, Error> {
        match self {
            Self::Items => payload.known_item_count().ok_or_else(|| {
                format_error("item count is unavailable; prepare the input before batching")
            }),
            Self::Bytes => payload
                .encoded_bytes()
                .map(Bytes::len)
                .ok_or_else(|| format_error("encoded byte size is unavailable")),
            Self::Requests => Err(format_error("request sizing is unsupported")),
        }
    }
}

/// Sizing and bounded split amplification policy shared by native batchers.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BatchProfile {
    /// Flush threshold in the selected sizing unit.
    pub min_size: Option<NonZeroUsize>,
    /// Optional best-effort maximum output size in that same unit.
    pub max_size: Option<NonZeroUsize>,
    /// Unit supported by the selected native batcher or OTAP fallback.
    pub sizer: BatchSizer,
    /// Per-entry split fragment budget; indivisible/over-budget entries remain whole.
    #[serde(default = "default_fragments")]
    pub max_split_fragments: Option<NonZeroUsize>,
    /// Maximum duplicated wrapper bytes per split entry.
    #[serde(default = "default_overhead")]
    pub max_split_overhead_bytes: Option<NonZeroUsize>,
    /// Per-flush additional split fan-out budget, excluding mandatory whole entries.
    #[serde(default = "default_fragments")]
    pub max_split_fragments_per_flush: Option<NonZeroUsize>,
}

const fn default_fragments() -> Option<NonZeroUsize> {
    NonZeroUsize::new(65536)
}
const fn default_overhead() -> Option<NonZeroUsize> {
    NonZeroUsize::new(8 * 1024 * 1024)
}

impl BatchProfile {
    /// Existing native OTAP defaults, also used by the item-based fallback.
    #[must_use]
    pub const fn otap() -> Self {
        Self {
            min_size: NonZeroUsize::new(8192),
            max_size: None,
            sizer: BatchSizer::Items,
            max_split_fragments: None,
            max_split_overhead_bytes: None,
            max_split_fragments_per_flush: None,
        }
    }

    /// Existing OTLP byte-batching defaults.
    #[must_use]
    pub const fn otlp() -> Self {
        Self {
            min_size: NonZeroUsize::new(262144),
            max_size: None,
            sizer: BatchSizer::Bytes,
            max_split_fragments: default_fragments(),
            max_split_overhead_bytes: default_overhead(),
            max_split_fragments_per_flush: default_fragments(),
        }
    }

    /// Checks representation-independent sizing constraints.
    pub fn validate(&self) -> Result<(), Error> {
        if self.min_size.or(self.max_size).is_none() {
            return Err(format_error("max_size or min_size must be set"));
        }
        if let (Some(max), Some(min)) = (self.max_size, self.min_size)
            && max < min
        {
            return Err(format_error("max_size must be >= min_size or unset"));
        }
        if self.sizer == BatchSizer::Requests {
            return Err(format_error("request sizing is unsupported"));
        }
        Ok(())
    }

    /// The first size threshold reached by pending inputs.
    #[must_use]
    pub fn lower_limit(&self) -> usize {
        self.min_size
            .or(self.max_size)
            .expect("validated batching profile")
            .get()
    }
}

/// A codec's optional native batching capabilities and default policy.
#[derive(Debug)]
pub struct BatchingSupport {
    /// Supported units; declaring a unit promises in-line merge and split support.
    pub sizers: &'static [BatchSizer],
    /// Default policy when the node has no explicit codec profile.
    pub default_profile: BatchProfile,
}

/// Codec-native outputs paired with input ownership weights, in input order.
/// Weights must sum to the measured input total, even if wrapper bytes duplicate.
pub struct CodecBatches {
    /// Encoded independent output batches and their input-unit ownership.
    pub batches: Vec<(Bytes, usize)>,
    /// Entries emitted whole because of split amplification limits.
    pub budget_fallbacks: u64,
}

impl PdataFormat {
    /// Independently encoded OTLP protobuf bytes.
    pub const OTLP: Self = Self::encoded(ResolvedCodec::OTLP);

    /// Resolves a canonical format name or the legacy OTLP alias.
    pub fn resolve(name: &str) -> Result<Self, Error> {
        match name {
            "otap" => Ok(Self::OTAP),
            "otlp" => Ok(Self::OTLP),
            name => codec::find(&PdataEncoding::from(name.to_owned())).map(Self::encoded),
        }
    }

    /// Stable name for configuration and diagnostics.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.codec()
            .map_or("otap", |codec| codec.metadata().encoding.as_str())
    }

    /// Supported signal types, known without examining any payload.
    #[must_use]
    pub fn signals(self) -> &'static [SignalType] {
        self.codec().map_or(
            &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
            |codec| codec.metadata().signals,
        )
    }

    /// Declared native profile, or the standard OTAP item fallback profile.
    #[must_use]
    pub fn default_profile(self) -> BatchProfile {
        self.codec()
            .and_then(|codec| codec.metadata().batching.as_ref())
            .map_or_else(BatchProfile::otap, |support| {
                support.default_profile.clone()
            })
    }

    /// Whether this format declares native batching rather than relying on OTAP.
    #[must_use]
    pub fn has_native_batching(self) -> bool {
        self.codec()
            .is_none_or(|codec| codec.metadata().batching.is_some())
    }

    /// Converts only when necessary and retains the original payload on failure.
    pub fn materialize(
        self,
        payload: &mut OtapPayload,
        context: &mut CodecState,
        encoding: Option<&EncodingPlan>,
    ) -> Result<(), Error> {
        if payload.format() == self {
            return Ok(());
        }
        match self.codec() {
            None => payload
                .materialize_otap(context)
                .map_err(|error| format_error(error.to_string())),
            Some(codec) => {
                let plan = encoding.ok_or_else(|| {
                    format_error(format!(
                        "missing startup encoding plan for {}",
                        codec.metadata().encoding
                    ))
                })?;
                payload.convert_encoding(context, plan)
            }
        }
    }
}

/// A resolved batching operation. Working and output formats may differ when
/// an explicitly requested encoding uses the item-based OTAP fallback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchPlan {
    working: PdataFormat,
    output: PdataFormat,
    encoding: Option<EncodingPlan>,
    profile: BatchProfile,
}

impl BatchPlan {
    /// Resolves native batching or the OTAP fallback before buffering input.
    pub fn new(format: PdataFormat, profile: BatchProfile, preserve: bool) -> Result<Self, Error> {
        profile.validate()?;
        let native = match format.codec() {
            None => profile.sizer == BatchSizer::Items,
            Some(codec) => {
                for signal in format.signals() {
                    codec.require(*signal, CodecDirection::Decode)?;
                }
                codec
                    .metadata()
                    .batching
                    .as_ref()
                    .is_some_and(|support| support.sizers.contains(&profile.sizer))
            }
        };
        if !native && profile.sizer != BatchSizer::Items {
            return Err(format_error(format!(
                "{} does not support {:?} batching; OTAP fallback requires items",
                format.name(),
                profile.sizer
            )));
        }
        if !preserve && let Some(codec) = format.codec() {
            for signal in format.signals() {
                codec.require(*signal, CodecDirection::Encode)?;
            }
        }
        let working = if native { format } else { PdataFormat::OTAP };
        let output = if preserve { working } else { format };
        let encoding = if preserve {
            None
        } else {
            format
                .codec()
                .map(|codec| EncodingPlan::new(codec, EncodeOptions::default()))
                .transpose()?
        };
        Ok(Self {
            working,
            output,
            encoding,
            profile,
        })
    }

    /// The validated profile used for thresholds and output retention.
    #[must_use]
    pub const fn profile(&self) -> &BatchProfile {
        &self.profile
    }

    /// Logical output format, also used to select a bounded buffer at startup.
    #[must_use]
    pub const fn output_format(&self) -> PdataFormat {
        self.output
    }

    /// Prepares one input before its measurement and delivery context are buffered.
    pub fn prepare(
        &self,
        payload: &mut OtapPayload,
        context: &mut CodecState,
    ) -> Result<(), Error> {
        if !self.output.signals().contains(&payload.signal_type()) {
            return Err(format_error(format!(
                "{} does not support {:?}",
                self.output.name(),
                payload.signal_type()
            )));
        }
        self.working
            .materialize(payload, context, self.encoding.as_ref())?;
        if self.profile.sizer == BatchSizer::Items && payload.known_item_count().is_none() {
            let codec = self
                .working
                .codec()
                .expect("native OTAP has known item counts");
            let bytes = payload
                .encoded_bytes()
                .expect("encoded working format")
                .clone();
            let count =
                context
                    .instance(codec)
                    .measure(payload.signal_type(), bytes, BatchSizer::Items)?;
            payload.set_item_count(count);
        }
        Ok(())
    }

    /// Prepares one input using pipeline-runtime codec state.
    pub fn prepare_with(
        &self,
        payload: &mut OtapPayload,
        executor: &CodecExecutor,
    ) -> Result<(), Error> {
        executor.execute(|context| self.prepare(payload, context))
    }

    /// Finishes a flushed output. Retained tails stay in the working format so
    /// their ownership units and measured sizes do not change between flushes.
    pub fn finish(&self, payload: &mut OtapPayload, context: &mut CodecState) -> Result<(), Error> {
        self.output
            .materialize(payload, context, self.encoding.as_ref())
    }

    /// Finishes a flushed output using pipeline-runtime codec state.
    pub fn finish_with(
        &self,
        payload: &mut OtapPayload,
        executor: &CodecExecutor,
    ) -> Result<(), Error> {
        executor.execute(|context| self.finish(payload, context))
    }

    /// Merges/splits prepared inputs while preserving their in-line ownership.
    pub fn batch(
        &self,
        signal: SignalType,
        inputs: Vec<OtapPayload>,
        context: &mut CodecState,
    ) -> Result<BatchingOutput, Error> {
        let total = inputs.iter().try_fold(0usize, |total, input| {
            if input.format() != self.working || input.signal_type() != signal {
                return Err(format_error("batch input was not prepared for this plan"));
            }
            total
                .checked_add(self.profile.sizer.batch_size(input)?)
                .ok_or_else(|| format_error("batch ownership overflow"))
        })?;
        let result = match self.working.codec() {
            None => {
                let records = inputs
                    .into_iter()
                    .map(|input| {
                        input
                            .try_into_otap(context)
                            .expect("checked native OTAP working format")
                    })
                    .collect();
                let limit = self
                    .profile
                    .max_size
                    .map(|size| NonZeroU64::new(size.get() as u64).expect("nonzero"));
                let batches = crate::otap::batching::make_item_batches(signal, limit, records)?;
                BatchingOutput {
                    batches: batches
                        .into_iter()
                        .map(|records| {
                            let weight = records.num_items();
                            (records.into(), weight)
                        })
                        .collect(),
                    budget_fallbacks: 0,
                }
            }
            Some(codec) => {
                let inputs = inputs
                    .into_iter()
                    .map(|input| input.into_encoded_bytes().expect("checked working format"))
                    .collect();
                let result = context
                    .instance(codec)
                    .batch(signal, &self.profile, inputs)?;
                BatchingOutput {
                    batches: result
                        .batches
                        .into_iter()
                        .map(|(bytes, weight)| {
                            let encoded = EncodedPdata::from_resolved(codec, signal, bytes);
                            let payload = OtapPayload::from_encoded(encoded);
                            let payload = if self.profile.sizer == BatchSizer::Items {
                                payload.with_item_count(weight)
                            } else {
                                payload
                            };
                            (payload, weight)
                        })
                        .collect(),
                    budget_fallbacks: result.budget_fallbacks,
                }
            }
        };
        let output_total = result
            .batches
            .iter()
            .try_fold(0usize, |total, (_, weight)| total.checked_add(*weight));
        if output_total != Some(total) || result.batches.iter().any(|(_, weight)| *weight == 0) {
            return Err(format_error(
                "batcher did not partition input ownership exactly",
            ));
        }
        Ok(result)
    }

    /// Batches prepared input using pipeline-runtime codec state.
    pub fn batch_with(
        &self,
        signal: SignalType,
        inputs: Vec<OtapPayload>,
        executor: &CodecExecutor,
    ) -> Result<BatchingOutput, Error> {
        executor.execute(|context| self.batch(signal, inputs, context))
    }
}

/// Re-batched working payloads and input ownership, not delivery contexts.
pub struct BatchingOutput {
    /// Outputs in input order, each paired with its input-unit ownership weight.
    pub batches: Vec<(OtapPayload, usize)>,
    /// Entries emitted whole after reaching split amplification budgets.
    pub budget_fallbacks: u64,
}

fn format_error(error: impl Into<String>) -> Error {
    Error::Format {
        error: error.into(),
    }
}
