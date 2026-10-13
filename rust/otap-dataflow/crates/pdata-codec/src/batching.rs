// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Representation-independent batching plans and codec batching contracts.
//!
//! This layer owns conversion, sizing, and in-line merge/split operations.
//! Scheduling, bounded request tracking, and Ack/Nack routing belong to the node.

use std::num::{NonZeroU64, NonZeroUsize};

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use serde::{Deserialize, Serialize};

use crate::{
    CodecError, CodecRegistry, CodecService, EncodePolicy, EncodedPdata, EncodingPlan, OtapPayload,
    PdataEncoding, PdataFormat,
};

/// Reusable encoded batching implementation owned by one pipeline runtime.
///
/// Inputs are complete, independently decodable batches with the same encoding
/// and signal. Each output must also decode independently: concatenating framed
/// batches is valid only when the encoding defines that operation. Preserve item
/// order within a call; the processor's in-range passthrough can overtake buffered
/// inputs, so this is not a pipeline-wide ordering guarantee.
///
/// Honor the sizing and split-amplification budgets in [`BatchProfile`]. Sizes
/// are measured before transport compression. Indivisible entries and exhausted
/// budgets may produce outputs above the best-effort maximum. Return their count
/// in [`CodecBatches::budget_fallbacks`]. A failure must leave the batcher usable
/// for the next call. Delivery contexts and Ack/Nack routing remain in the node.
pub trait PdataBatcher: Send {
    /// Re-batches complete encoded inputs in order.
    fn batch(
        &mut self,
        signal: SignalType,
        profile: &BatchProfile,
        inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, CodecError>;
}

/// Unit used for flush thresholds, splitting, and input ownership attribution.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BatchSizer {
    /// Original request count (reserved, not currently supported).
    Requests,
    /// Log records, spans, or metric data points.
    Items,
    /// Complete encoded request bytes, including format wrappers and intrinsic
    /// compression, but excluding HTTP/gRPC transport compression and framing.
    Bytes,
}

impl BatchSizer {
    /// Measures an input already prepared by its batching plan.
    pub fn batch_size(&self, payload: &OtapPayload) -> Result<usize, CodecError> {
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
    /// Lower bound for flushing and passthrough in the selected sizing unit.
    /// Defaults to `max_size` when unset; zero allows any input up to `max_size`.
    pub min_size: Option<usize>,
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
            min_size: Some(8192),
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
            min_size: Some(262144),
            max_size: None,
            sizer: BatchSizer::Bytes,
            max_split_fragments: default_fragments(),
            max_split_overhead_bytes: default_overhead(),
            max_split_fragments_per_flush: default_fragments(),
        }
    }

    /// Checks representation-independent sizing constraints.
    pub fn validate(&self) -> Result<(), CodecError> {
        if self.min_size.is_none() && self.max_size.is_none() {
            return Err(format_error("max_size or min_size must be set"));
        }
        if let (Some(max), Some(min)) = (self.max_size, self.min_size)
            && max.get() < min
        {
            return Err(format_error("max_size must be >= min_size or unset"));
        }
        if self.sizer == BatchSizer::Requests {
            return Err(format_error("request sizing is unsupported"));
        }
        Ok(())
    }

    /// The first size threshold reached by pending inputs.
    ///
    /// # Panics
    /// Panics if neither size bound is set. Call [`Self::validate`] first;
    /// [`BatchPlan::new`] validates before exposing a profile to a processor.
    #[must_use]
    pub fn lower_limit(&self) -> usize {
        self.min_size
            .or(self.max_size.map(NonZeroUsize::get))
            .expect("validated batching profile has at least one size bound")
    }

    /// Whether an input is already within the inclusive accepted size range.
    #[must_use]
    pub fn in_passthrough_range(&self, size: usize) -> bool {
        size >= self.lower_limit() && self.max_size.is_none_or(|max| size <= max.get())
    }

    /// Whether every input is forwarded without batching or splitting.
    #[must_use]
    pub const fn forwards_everything(&self) -> bool {
        matches!(self.min_size, Some(0)) && self.max_size.is_none()
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
    /// Resolves a canonical format name or the legacy OTLP alias.
    pub fn resolve(registry: &CodecRegistry, name: &str) -> Result<Self, CodecError> {
        match name {
            "otap" => Ok(Self::OTAP),
            "otlp" => registry
                .resolve(&PdataEncoding::OTLP)
                .map(Self::encoded)
                .map_err(Into::into),
            name => registry
                .resolve(&PdataEncoding::from(name.to_owned()))
                .map(Self::encoded)
                .map_err(Into::into),
        }
    }

    /// Stable name for configuration and diagnostics.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.codec()
            .map_or("otap", |codec| codec.encoding().as_str())
    }

    /// Supported signal types, known without examining any payload.
    #[must_use]
    pub fn signals(self) -> &'static [SignalType] {
        self.codec().map_or(
            &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
            |codec| codec.metadata().signals(),
        )
    }

    /// Declared native profile, or the standard OTAP item fallback profile.
    #[must_use]
    pub fn default_profile(self) -> BatchProfile {
        self.codec()
            .and_then(|codec| codec.batching_support())
            .map_or_else(BatchProfile::otap, |support| {
                support.default_profile.clone()
            })
    }

    /// Whether this format declares native batching rather than relying on OTAP.
    #[must_use]
    pub fn has_native_batching(self) -> bool {
        self.codec().is_none_or(|codec| codec.can_batch())
    }

    /// Converts only when necessary and retains the original payload on failure.
    pub fn materialize(
        self,
        payload: &mut OtapPayload,
        service: &CodecService,
        encoding: Option<&EncodingPlan>,
    ) -> Result<(), CodecError> {
        if payload.format() == self {
            return Ok(());
        }
        match self.codec() {
            None => {
                let empty = OtapPayload::empty(payload.signal_type());
                let original = std::mem::replace(payload, empty);
                match original.try_into_otap(service) {
                    Ok(records) => {
                        *payload = records.into();
                        Ok(())
                    }
                    Err(error) => {
                        let (source, original) = error.into_parts();
                        *payload = original;
                        Err(source)
                    }
                }
            }
            Some(codec) => {
                let plan = encoding.ok_or_else(|| {
                    format_error(format!(
                        "missing startup encoding plan for {}",
                        codec.encoding()
                    ))
                })?;
                if plan.codec() != codec {
                    return Err(format_error(
                        "output encoding plan does not match requested format",
                    ));
                }
                let signal = payload.signal_type();
                let bytes = payload.encode_bytes(service, plan)?;
                // The encoding plan validates the output capability and signal;
                // output-only codecs need not support receiver admission.
                let encoded = EncodedPdata::from_resolved(codec, signal, bytes);
                *payload = OtapPayload::from_encoded(encoded);
                Ok(())
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
    pub fn new(
        format: PdataFormat,
        profile: BatchProfile,
        preserve: bool,
    ) -> Result<Self, CodecError> {
        profile.validate()?;
        let native = match format.codec() {
            None => profile.sizer == BatchSizer::Items,
            Some(codec) => codec
                .batching_support()
                .is_some_and(|support| support.sizers.contains(&profile.sizer)),
        };
        if !native && profile.sizer != BatchSizer::Items {
            return Err(format_error(format!(
                "{} does not support {:?} batching; OTAP fallback requires items",
                format.name(),
                profile.sizer
            )));
        }
        // Only fallback from an encoded input requires its decoder. Explicit
        // output selection may target an encode-only codec through native OTAP.
        if preserve
            && !native
            && let Some(codec) = format.codec()
        {
            for signal in format.signals() {
                codec.require_decoder(*signal)?;
            }
        }
        if !preserve && let Some(codec) = format.codec() {
            for signal in format.signals() {
                codec.require_encoder(*signal)?;
            }
        }
        let working = if native { format } else { PdataFormat::OTAP };
        let output = if preserve { working } else { format };
        let encoding = if preserve {
            None
        } else {
            format
                .codec()
                .map(|codec| EncodingPlan::new(codec, EncodePolicy::default()))
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
        service: &CodecService,
    ) -> Result<(), CodecError> {
        if !self.output.signals().contains(&payload.signal_type()) {
            return Err(format_error(format!(
                "{} does not support {:?}",
                self.output.name(),
                payload.signal_type()
            )));
        }
        self.working
            .materialize(payload, service, self.encoding.as_ref())?;
        if self.profile.sizer == BatchSizer::Items && payload.known_item_count().is_none() {
            let view = payload
                .encoded_view()
                .ok_or_else(|| format_error("item count is unavailable for native batching"))?;
            let codec = self
                .working
                .codec()
                .ok_or_else(|| format_error("item count is unavailable for native batching"))?;
            let count = codec
                .count_items(view.signal_type(), view.bytes())
                .ok_or_else(|| format_error("item count is unavailable for encoded batching"))?;
            let replacement = payload.take_payload().with_item_count(count);
            *payload = replacement;
        }
        Ok(())
    }

    /// Finishes a flushed output. Retained tails stay in the working format so
    /// their ownership units and measured sizes do not change between flushes.
    pub fn finish(
        &self,
        payload: &mut OtapPayload,
        service: &CodecService,
    ) -> Result<(), CodecError> {
        self.output
            .materialize(payload, service, self.encoding.as_ref())
    }

    /// Merges/splits prepared inputs while preserving their in-line ownership.
    pub fn batch(
        &self,
        signal: SignalType,
        inputs: Vec<OtapPayload>,
        service: &CodecService,
    ) -> Result<BatchingOutput, CodecError> {
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
                            .try_into_otap(service)
                            .map_err(|error| error.into_parts().0)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let limit = self.profile.max_size.map(|size| {
                    // SAFETY: supported targets have at most 64-bit usize,
                    // so widening preserves a nonzero value.
                    NonZeroU64::new(size.get() as u64).expect("nonzero size")
                });
                let batches =
                    otel_arrow_dfe_pdata::otap::batching::make_item_batches(signal, limit, records)
                        .map_err(|error| format_error(error.to_string()))?;
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
                    .map(|input| {
                        input
                            .into_encoded_bytes()
                            .ok_or_else(|| format_error("batch input is not encoded"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let result = service.batch(codec, signal, &self.profile, inputs)?;
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
}

/// Re-batched working payloads and input ownership, not delivery contexts.
pub struct BatchingOutput {
    /// Outputs in input order, each paired with its input-unit ownership weight.
    pub batches: Vec<(OtapPayload, usize)>,
    /// Entries emitted whole after reaching split amplification budgets.
    pub budget_fallbacks: u64,
}

fn format_error(error: impl Into<String>) -> CodecError {
    CodecError::invalid_batch(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{DECODE_ONLY_ENCODING, ENCODE_ONLY_ENCODING};
    use crate::{CodecServiceBuilder, DecodePolicy, DecodeValidation};
    use otel_arrow_dfe_pdata::OtapPayloadHelpers;
    use otel_arrow_dfe_pdata::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };
    use prost::Message;

    fn strict_service() -> CodecService {
        CodecServiceBuilder::from_global_registry()
            .unwrap()
            .with_decode_policy(DecodePolicy::new(DecodeValidation::Strict))
            .build()
    }

    /// Scenario: Repeated OTLP byte batching processes multiple logs, metrics, and traces requests.
    /// Guarantees: Each output independently decodes, counts and ownership are conserved,
    /// and native batching reuses one batcher without instantiating a decoder.
    #[test]
    fn native_batches_decode_independently_and_reuse_state() {
        let service = strict_service();
        let decoder = strict_service();
        let codec = service.registry().resolve(&PdataEncoding::OTLP).unwrap();
        let cases = [
            (
                SignalType::Logs,
                logs_with_full_resource_and_scope().encode_to_vec(),
                4,
            ),
            (
                SignalType::Metrics,
                metrics_sum_with_full_resource_and_scope().encode_to_vec(),
                2,
            ),
            (
                SignalType::Traces,
                traces_with_full_resource_and_scope().encode_to_vec(),
                2,
            ),
        ];
        for (signal, bytes, items) in cases {
            let bytes = Bytes::from(bytes);
            let plan = BatchPlan::new(
                PdataFormat::encoded(codec),
                BatchProfile {
                    min_size: Some(0),
                    max_size: NonZeroUsize::new(bytes.len() / 2),
                    ..BatchProfile::otlp()
                },
                true,
            )
            .unwrap();
            for _ in 0..2 {
                let mut input =
                    OtapPayload::from_encoded(codec.admit(signal, bytes.clone()).unwrap());
                plan.prepare(&mut input, &service).unwrap();
                assert_eq!(input.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
                let result = plan
                    .batch(signal, vec![input.clone(), input], &service)
                    .unwrap();
                assert!(result.batches.len() > 1);
                assert_eq!(
                    result
                        .batches
                        .iter()
                        .map(|(_, weight)| *weight)
                        .sum::<usize>(),
                    2 * bytes.len()
                );
                let mut decoded_items = 0;
                for (output, _) in result.batches {
                    assert_eq!(output.format(), PdataFormat::encoded(codec));
                    let records = output.try_into_otap(&decoder).unwrap();
                    assert_eq!(records.signal_type(), signal);
                    decoded_items += records.num_items();
                }
                assert_eq!(decoded_items, 2 * items);
                assert_eq!(service.test_instance_count().unwrap(), 1);
            }
        }
    }

    /// Scenario: A decode-only input needs OTAP fallback, or native records target an encode-only codec.
    /// Guarantees: Plans require only the capabilities used and reject incompatible output plans.
    #[test]
    fn fallback_and_output_only_plans_use_directional_capabilities() {
        let service = strict_service();
        let input_codec = service.registry().resolve(&DECODE_ONLY_ENCODING).unwrap();
        let output_codec = service.registry().resolve(&ENCODE_ONLY_ENCODING).unwrap();
        let fallback = BatchPlan::new(
            PdataFormat::encoded(input_codec),
            BatchProfile::otap(),
            true,
        )
        .unwrap();
        assert_eq!(fallback.output_format(), PdataFormat::OTAP);
        assert!(
            BatchPlan::new(
                PdataFormat::encoded(input_codec),
                BatchProfile::otap(),
                false
            )
            .is_err()
        );
        let mut input = OtapPayload::from_encoded(
            input_codec
                .admit(
                    SignalType::Logs,
                    logs_with_full_resource_and_scope().encode_to_vec().into(),
                )
                .unwrap(),
        );
        fallback.prepare(&mut input, &service).unwrap();
        assert!(input.otap_ref().is_some());
        let output = PdataFormat::encoded(output_codec);
        let wrong = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::OTLP,
            EncodePolicy::default(),
        )
        .unwrap();
        assert!(
            output
                .materialize(&mut input, &service, Some(&wrong))
                .is_err()
        );
        assert!(input.otap_ref().is_some());
        let plan = BatchPlan::new(output, BatchProfile::otap(), false).unwrap();
        plan.prepare(&mut input, &service).unwrap();
        plan.finish(&mut input, &service).unwrap();
        assert_eq!(input.format(), output);
        assert_eq!(
            otel_arrow_dfe_pdata::count_otlp_items(
                SignalType::Logs,
                input.encoded_bytes().unwrap()
            ),
            4
        );
    }

    /// Scenario: Strict fallback encounters malformed encoded input, then a valid batch.
    /// Guarantees: Failure preserves the exact bytes, identity, signal and cached count;
    /// the same decoder remains usable for subsequent input.
    #[test]
    fn fallback_failure_preserves_payload_and_recovers() {
        let service = strict_service();
        let codec = service.registry().resolve(&DECODE_ONLY_ENCODING).unwrap();
        let plan = BatchPlan::new(PdataFormat::encoded(codec), BatchProfile::otap(), true).unwrap();
        let bytes = Bytes::from_static(&[0x0a, 0x01, 0xff]);
        let mut input =
            OtapPayload::from_encoded(codec.admit(SignalType::Logs, bytes.clone()).unwrap())
                .with_item_count(7);
        for _ in 0..2 {
            assert!(plan.prepare(&mut input, &service).is_err());
            assert_eq!(input.format(), PdataFormat::encoded(codec));
            assert_eq!(input.signal_type(), SignalType::Logs);
            assert_eq!(input.known_item_count(), Some(7));
            assert_eq!(input.encoded_bytes().unwrap(), &bytes);
            assert_eq!(input.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
        }
        let mut valid = OtapPayload::from_encoded(
            codec
                .admit(
                    SignalType::Logs,
                    logs_with_full_resource_and_scope().encode_to_vec().into(),
                )
                .unwrap(),
        );
        plan.prepare(&mut valid, &service).unwrap();
        assert_eq!(valid.known_item_count(), Some(4));
        assert_eq!(service.test_instance_count().unwrap(), 1);
    }
}
