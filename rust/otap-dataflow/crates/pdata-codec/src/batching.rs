// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Representation-independent batching plans and codec batching contracts.
//!
//! This layer owns conversion, sizing, and in-line merge/split operations.
//! Scheduling, bounded request tracking, and Ack/Nack routing belong to the node.

use std::num::NonZeroU64;

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use serde::{Deserialize, Serialize};

use crate::{
    CodecError, CodecRegistry, CodecService, EncodePolicy, EncodedPdata, EncodingPlan,
    PdataEncoding, PdataFormat, PdataPayload,
};

/// Reusable encoded batching implementation owned by one pipeline runtime.
///
/// Inputs are complete, independently decodable batches with the same encoding
/// and signal. Each output must also decode independently: concatenating framed
/// batches is valid only when the encoding defines that operation. Preserve item
/// order within a call; the processor's in-range passthrough can overtake buffered
/// inputs, so this is not a pipeline-wide ordering guarantee.
///
/// Honor the sizing and split-amplification budgets in [`BatchSettings`]. Sizes
/// are measured before transport compression. Indivisible entries and exhausted
/// budgets may produce outputs above the best-effort maximum. Return their count
/// in [`CodecBatches::budget_fallbacks`]. A failure must leave the batcher usable
/// for the next call. Delivery contexts and Ack/Nack routing remain in the node.
///
/// Report the inclusive original input range for each output and explicitly list
/// inputs consumed without output, as specified by [`CodecBatches`]. Capture
/// provenance while batching; never reconstruct it from output measurements.
pub trait PdataBatcher: Send {
    /// Re-batches complete encoded inputs in order.
    fn batch(
        &mut self,
        signal: SignalType,
        settings: &BatchSettings,
        inputs: Vec<Bytes>,
    ) -> Result<CodecBatches, CodecError>;
}

/// Measurement unit for flush thresholds and splitting, never completion ownership.
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
    pub fn batch_size(&self, payload: &PdataPayload) -> Result<usize, CodecError> {
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

/// Sizing thresholds and bounded split amplification shared by native batchers.
///
/// These settings control batching behavior, not representation identity (see
/// [`PdataFormat`]). Limits use u64 independently of the target pointer width;
/// batchers must cap or reject upper bounds that exceed their internal size types.
/// Narrowing must never wrap a large limit into a smaller value.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BatchSettings {
    /// Lower bound for flushing and passthrough in the selected sizing unit.
    /// Defaults to `max_size` when unset; zero allows any input up to `max_size`.
    pub min_size: Option<u64>,
    /// Optional best-effort maximum output size in that same unit.
    pub max_size: Option<NonZeroU64>,
    /// Unit supported by the selected native batcher or OTAP fallback.
    pub sizer: BatchSizer,
    /// Per-entry split fragment budget; indivisible/over-budget entries remain whole.
    #[serde(default = "default_fragments")]
    pub max_split_fragments: Option<NonZeroU64>,
    /// Maximum duplicated wrapper bytes per split entry.
    #[serde(default = "default_overhead")]
    pub max_split_overhead_bytes: Option<NonZeroU64>,
    /// Per-flush additional split fan-out budget, excluding mandatory whole entries.
    #[serde(default = "default_fragments")]
    pub max_split_fragments_per_flush: Option<NonZeroU64>,
}

const fn default_fragments() -> Option<NonZeroU64> {
    NonZeroU64::new(65536)
}
const fn default_overhead() -> Option<NonZeroU64> {
    NonZeroU64::new(8 * 1024 * 1024)
}

impl BatchSettings {
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
    /// [`BatchPlan::new`] validates before exposing settings to a processor.
    #[must_use]
    pub fn lower_limit(&self) -> u64 {
        self.min_size
            .or(self.max_size.map(NonZeroU64::get))
            .expect("validated batching settings have at least one size bound")
    }

    /// Whether an input is already within the inclusive accepted size range.
    #[must_use]
    pub fn in_passthrough_range(&self, size: usize) -> bool {
        // Measurements fit in u64 on supported targets; widen instead of narrowing limits.
        let size = size as u64;
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
    /// Declaring [`BatchSizer::Items`] requires a registered item counter,
    /// even when another unit is the default or the codec has no decoder.
    pub sizers: &'static [BatchSizer],
    /// Default policy when the node has no explicit codec settings.
    pub default_settings: BatchSettings,
}

/// Independently decodable outputs with inclusive input ranges and explicit
/// successful completions without output. See [`otel_arrow_dfe_pdata::batching::BatchingResult`].
pub type CodecBatches = otel_arrow_dfe_pdata::batching::BatchingResult<Bytes>;

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

    /// Declared native settings, or the standard OTAP item fallback settings.
    #[must_use]
    pub fn default_settings(self) -> BatchSettings {
        self.codec()
            .and_then(|codec| codec.batching_support())
            .map_or_else(BatchSettings::otap, |support| {
                support.default_settings.clone()
            })
    }

    /// Whether this format declares native batching rather than relying on OTAP.
    #[must_use]
    pub fn has_native_batching(self) -> bool {
        self.codec().is_none_or(|codec| codec.can_batch())
    }

    /// Replaces `payload` in place with this destination representation.
    ///
    /// `payload` is both input and output; `Ok(())` means it now has `self`'s
    /// format. Matching formats are untouched. Native conversion moves records
    /// or decodes bytes, while encoded conversion uses the startup encoding plan.
    /// Failure leaves the original payload, bytes and caches available for recovery.
    /// This is an internal implementation detail of `BatchPlan::prepare/finish`.
    fn materialize(
        self,
        payload: &mut PdataPayload,
        service: &CodecService,
        encoding: Option<&EncodingPlan>,
    ) -> Result<(), CodecError> {
        if payload.format() == self {
            return Ok(());
        }
        match self.codec() {
            None => {
                let empty = PdataPayload::empty(payload.signal_type());
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
                *payload = PdataPayload::from_encoded(encoded);
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
    settings: BatchSettings,
}

impl BatchPlan {
    /// Resolves native batching or the OTAP fallback before buffering input.
    pub fn new(
        format: PdataFormat,
        settings: BatchSettings,
        preserve: bool,
    ) -> Result<Self, CodecError> {
        settings.validate()?;
        let native = match format.codec() {
            None => settings.sizer == BatchSizer::Items,
            Some(codec) => codec
                .batching_support()
                .is_some_and(|support| support.sizers.contains(&settings.sizer)),
        };
        if !native && settings.sizer != BatchSizer::Items {
            return Err(format_error(format!(
                "{} does not support {:?} batching; OTAP fallback requires items",
                format.name(),
                settings.sizer
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
            settings,
        })
    }

    /// The validated settings used for thresholds and output retention.
    #[must_use]
    pub const fn settings(&self) -> &BatchSettings {
        &self.settings
    }

    /// Logical output format, also used to select a bounded buffer at startup.
    #[must_use]
    pub const fn output_format(&self) -> PdataFormat {
        self.output
    }

    /// Prepares one input before its measurement and delivery context are buffered.
    pub fn prepare(
        &self,
        payload: &mut PdataPayload,
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
        if self.settings.sizer == BatchSizer::Items && payload.known_item_count().is_none() {
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

    /// Converts a flushed output to its requested representation in place.
    /// Retained tails stay in the working format; provenance does not depend on size.
    pub fn finish(
        &self,
        payload: &mut PdataPayload,
        service: &CodecService,
    ) -> Result<(), CodecError> {
        self.output
            .materialize(payload, service, self.encoding.as_ref())
    }

    /// Merges/splits prepared inputs and validates their explicit input provenance.
    pub fn batch(
        &self,
        signal: SignalType,
        inputs: Vec<PdataPayload>,
        service: &CodecService,
    ) -> Result<BatchingOutput, CodecError> {
        let input_count = inputs.len();
        if inputs
            .iter()
            .any(|input| input.format() != self.working || input.signal_type() != signal)
        {
            return Err(format_error("batch input was not prepared for this plan"));
        }
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
                let result = otel_arrow_dfe_pdata::otap::batching::make_item_batches_with_ranges(
                    signal,
                    self.settings.max_size,
                    records,
                )
                .map_err(|error| format_error(error.to_string()))?;
                BatchingOutput {
                    batches: result
                        .batches
                        .into_iter()
                        .map(|(records, inputs)| (records.into(), inputs))
                        .collect(),
                    completed_inputs: result.completed_inputs,
                    budget_fallbacks: result.budget_fallbacks,
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
                let result = service.batch(codec, signal, &self.settings, inputs)?;
                BatchingOutput {
                    batches: result
                        .batches
                        .into_iter()
                        .map(|(bytes, inputs)| {
                            let count = if self.settings.sizer == BatchSizer::Items {
                                Some(codec.count_items(signal, &bytes).ok_or_else(|| {
                                    format_error(
                                        "native item batching requires an exact output item count",
                                    )
                                })?)
                            } else {
                                None
                            };
                            let encoded = EncodedPdata::from_resolved(codec, signal, bytes);
                            let mut payload = PdataPayload::from_encoded(encoded);
                            if let Some(count) = count {
                                payload = payload.with_item_count(count);
                            }
                            Ok((payload, inputs))
                        })
                        .collect::<Result<Vec<_>, CodecError>>()?,
                    completed_inputs: result.completed_inputs,
                    budget_fallbacks: result.budget_fallbacks,
                }
            }
        };
        validate_input_ranges(&result, input_count)?;
        Ok(result)
    }
}

/// Re-batched working payloads and their input provenance, never delivery contexts.
pub type BatchingOutput = otel_arrow_dfe_pdata::batching::BatchingResult<PdataPayload>;

/// Validate coverage without allocating a per-input bitmap. Output ranges may
/// share only a boundary input; completed indices must be disjoint and sorted.
fn validate_input_ranges<T>(
    result: &otel_arrow_dfe_pdata::batching::BatchingResult<T>,
    input_count: usize,
) -> Result<(), CodecError> {
    let mut outputs = result.batches.iter().map(|(_, range)| range).peekable();
    let mut completed = result.completed_inputs.iter().copied().peekable();
    let mut covered = 0usize;
    let mut previous_output = false;
    while outputs.peek().is_some() || completed.peek().is_some() {
        let is_output = match (outputs.peek(), completed.peek()) {
            (Some(range), Some(index)) => *range.start() <= *index,
            (Some(_), None) => true,
            _ => false,
        };
        let (start, end) = if is_output {
            // SAFETY: is_output is true only when the output iterator is nonempty.
            let range = outputs.next().expect("peeked output");
            if range.is_empty() {
                return Err(format_error("batcher returned an empty input range"));
            }
            (*range.start(), *range.end())
        } else {
            // SAFETY: the loop guarantees a remaining completed input here.
            let index = completed.next().expect("peeked completed input");
            (index, index)
        };
        let shared_boundary = is_output && previous_output && covered.checked_sub(1) == Some(start);
        if start > end || end >= input_count || (start != covered && !shared_boundary) {
            return Err(format_error(
                "batcher returned invalid or incomplete input ranges",
            ));
        }
        covered = end + 1;
        previous_output = is_output;
    }
    if covered != input_count {
        return Err(format_error("batcher did not account for every input"));
    }
    Ok(())
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
    /// Guarantees: Each output independently decodes, item counts and input provenance are preserved,
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
                BatchSettings {
                    min_size: Some(0),
                    max_size: NonZeroU64::new((bytes.len() / 2) as u64),
                    ..BatchSettings::otlp()
                },
                true,
            )
            .unwrap();
            for _ in 0..2 {
                let mut input =
                    PdataPayload::from_encoded(codec.admit(signal, bytes.clone()).unwrap());
                plan.prepare(&mut input, &service).unwrap();
                assert_eq!(input.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
                let result = plan
                    .batch(signal, vec![input.clone(), input], &service)
                    .unwrap();
                assert!(result.batches.len() > 1);
                assert_eq!(*result.batches.first().unwrap().1.start(), 0);
                assert_eq!(*result.batches.last().unwrap().1.end(), 1);
                assert!(result.completed_inputs.is_empty());
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

    /// Scenario: Native item batching prepares an uncounted encoded input and splits off a short tail.
    /// Guarantees: Inputs and every output cache item counts distinct from byte lengths,
    /// retain their encoding, and reuse batching state without creating a decoder.
    #[test]
    fn native_item_batching_caches_counts_without_decoding() {
        use crate::{CodecBatcherRegistration, CodecMetadata, CodecRegistration};
        use std::sync::Arc;

        // Each independently decodable item occupies two bytes.
        struct PairBatcher;
        impl PdataBatcher for PairBatcher {
            fn batch(
                &mut self,
                _: SignalType,
                settings: &BatchSettings,
                inputs: Vec<Bytes>,
            ) -> Result<CodecBatches, CodecError> {
                assert_eq!(settings.max_size, NonZeroU64::new(2));
                Ok(CodecBatches {
                    batches: inputs
                        .into_iter()
                        .enumerate()
                        .flat_map(|(index, bytes)| {
                            (0..bytes.len()).step_by(4).map(move |start| {
                                (
                                    bytes.slice(start..(start + 4).min(bytes.len())),
                                    index..=index,
                                )
                            })
                        })
                        .collect(),
                    completed_inputs: Vec::new(),
                    budget_fallbacks: 0,
                })
            }
        }
        static METADATA: CodecMetadata =
            CodecMetadata::new(PdataEncoding::new("test-pairs"), &[SignalType::Logs]);
        static SUPPORT: BatchingSupport = BatchingSupport {
            sizers: &[BatchSizer::Items],
            default_settings: BatchSettings::otap(),
        };
        static REGISTRATIONS: [CodecRegistration; 1] = [CodecRegistration::new(&METADATA)
            .with_decoder(|_| panic!("native item batching must not create a decoder"))
            .with_item_counter(|_, bytes| Some(bytes.len() / 2))
            .with_batcher(CodecBatcherRegistration::new(&SUPPORT, || {
                Box::new(PairBatcher)
            }))];
        let service = CodecServiceBuilder::from_registry(Arc::new(
            CodecRegistry::validate(&REGISTRATIONS).unwrap(),
        ))
        .build();
        let codec = service.registry().resolve(METADATA.encoding()).unwrap();
        let plan = BatchPlan::new(
            PdataFormat::encoded(codec),
            BatchSettings {
                min_size: Some(2),
                max_size: NonZeroU64::new(2),
                ..BatchSettings::otap()
            },
            true,
        )
        .unwrap();
        let bytes = Bytes::from_static(b"aabbcc");
        for instance_count in [0, 1] {
            let mut input =
                PdataPayload::from_encoded(codec.admit(SignalType::Logs, bytes.clone()).unwrap());
            assert_eq!(input.known_item_count(), None);
            plan.prepare(&mut input, &service).unwrap();
            assert_eq!(input.known_item_count(), Some(3));
            assert_eq!(input.format(), PdataFormat::encoded(codec));
            assert_eq!(input.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
            assert_eq!(service.test_instance_count().unwrap(), instance_count);
            let result = plan.batch(SignalType::Logs, vec![input], &service).unwrap();
            assert_eq!(result.batches.len(), 2);
            for ((output, owners), (expected, count)) in result
                .batches
                .iter()
                .zip([(&b"aabb"[..], 2), (&b"cc"[..], 1)])
            {
                assert_eq!(output.format(), PdataFormat::encoded(codec));
                assert_eq!(output.signal_type(), SignalType::Logs);
                assert_eq!(output.encoded_bytes().unwrap().as_ref(), expected);
                assert_eq!(output.known_item_count(), Some(count));
                assert_eq!(plan.settings().sizer.batch_size(output).unwrap(), count);
                assert_eq!(*owners, 0..=0);
            }
            assert!(result.completed_inputs.is_empty());
            assert_eq!(service.test_instance_count().unwrap(), 1);
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
            BatchSettings::otap(),
            true,
        )
        .unwrap();
        assert_eq!(fallback.output_format(), PdataFormat::OTAP);
        assert!(
            BatchPlan::new(
                PdataFormat::encoded(input_codec),
                BatchSettings::otap(),
                false
            )
            .is_err()
        );
        let mut input = PdataPayload::from_encoded(
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
        let plan = BatchPlan::new(output, BatchSettings::otap(), false).unwrap();
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
        let plan =
            BatchPlan::new(PdataFormat::encoded(codec), BatchSettings::otap(), true).unwrap();
        let bytes = Bytes::from_static(&[0x0a, 0x01, 0xff]);
        let mut input =
            PdataPayload::from_encoded(codec.admit(SignalType::Logs, bytes.clone()).unwrap())
                .with_item_count(7);
        for _ in 0..2 {
            assert!(plan.prepare(&mut input, &service).is_err());
            assert_eq!(input.format(), PdataFormat::encoded(codec));
            assert_eq!(input.signal_type(), SignalType::Logs);
            assert_eq!(input.known_item_count(), Some(7));
            assert_eq!(input.encoded_bytes().unwrap(), &bytes);
            assert_eq!(input.encoded_bytes().unwrap().as_ptr(), bytes.as_ptr());
        }
        let mut valid = PdataPayload::from_encoded(
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
    /// Scenario: Batchers report merges, splits, empty completions, or malformed input ranges.
    /// Guarantees: All inputs are accounted for exactly by ordered provenance, with only
    /// split boundary overlap allowed; invalid indices never reach the processor.
    #[test]
    #[allow(
        clippy::single_range_in_vec_init,
        reason = "Each element describes one output's inclusive input range, not its individual indices."
    )]
    fn input_ranges_validate_coverage_without_sizes() {
        let check = |ranges: Vec<std::ops::RangeInclusive<usize>>, completed_inputs, count| {
            validate_input_ranges(
                &otel_arrow_dfe_pdata::batching::BatchingResult {
                    batches: ranges.into_iter().map(|range| ((), range)).collect(),
                    completed_inputs,
                    budget_fallbacks: 0,
                },
                count,
            )
        };
        for (ranges, completed, count) in [
            (vec![], vec![], 0),
            (vec![0..=0, 0..=1, 1..=2], vec![], 3),
            (vec![1..=2, 2..=2], vec![0, 3], 4),
            (vec![], vec![0, 1], 2),
        ] {
            assert!(check(ranges, completed, count).is_ok());
        }
        // A consumed inclusive iterator still exposes its last endpoints, but
        // indexes an empty slice. It must not bypass completion tracking.
        let mut exhausted = 0..=0;
        assert_eq!(exhausted.next(), Some(0));
        assert!(check(vec![exhausted], vec![], 1).is_err());
        for (ranges, completed, count) in [
            (vec![0..=0], vec![], 2),
            (vec![0..=2], vec![], 2),
            (vec![1..=1], vec![], 2),
            (vec![0..=1, 0..=1], vec![], 2),
            (vec![0..=0], vec![0], 1),
            (vec![], vec![1, 0], 2),
            (vec![], vec![0, 0], 1),
            (vec![usize::MAX..=usize::MAX], vec![], 1),
            (vec![std::ops::RangeInclusive::new(1, 0)], vec![], 2),
        ] {
            assert!(check(ranges, completed, count).is_err());
        }
    }

    /// Scenario: OTAP concatenation removes dangling metric data points from two inputs.
    /// Guarantees: Reduced item counts preserve both input owners; wholly redacted inputs
    /// explicitly complete without output instead of failing count conservation.
    #[test]
    fn native_ranges_survive_integrity_redaction() {
        use otel_arrow_dfe_pdata::OtapArrowRecords;
        use otel_arrow_dfe_pdata::arrow::array::{
            ArrayRef, RecordBatch, StringArray, UInt8Array, UInt16Array, UInt32Array,
        };
        use otel_arrow_dfe_pdata::otap::{Metrics, OtapBatchStore};
        use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
        use std::sync::Arc;
        let service = strict_service();
        let plan = BatchPlan::new(PdataFormat::OTAP, BatchSettings::otap(), true).unwrap();
        for surviving in [true, false] {
            let make = || {
                let mut metrics = Metrics::default();
                metrics
                    .set(
                        ArrowPayloadType::UnivariateMetrics,
                        RecordBatch::try_from_iter([
                            ("id", Arc::new(UInt16Array::from(vec![0])) as ArrayRef),
                            (
                                "metric_type",
                                Arc::new(UInt8Array::from(vec![
                                    otel_arrow_dfe_pdata::otlp::metrics::MetricType::Gauge as u8,
                                ])) as ArrayRef,
                            ),
                            (
                                "name",
                                Arc::new(StringArray::from(vec!["test.gauge"])) as ArrayRef,
                            ),
                        ])
                        .unwrap(),
                    )
                    .unwrap();
                metrics
                    .set(
                        ArrowPayloadType::NumberDataPoints,
                        RecordBatch::try_from_iter([
                            ("id", Arc::new(UInt32Array::from(vec![0, 1])) as ArrayRef),
                            (
                                "parent_id",
                                Arc::new(UInt16Array::from(if surviving {
                                    vec![0, 7]
                                } else {
                                    vec![7, 7]
                                })) as ArrayRef,
                            ),
                        ])
                        .unwrap(),
                    )
                    .unwrap();
                PdataPayload::from(OtapArrowRecords::Metrics(metrics))
            };
            let result = plan
                .batch(SignalType::Metrics, vec![make(), make()], &service)
                .unwrap();
            if surviving {
                assert_eq!(result.batches.len(), 1);
                assert_eq!(result.batches[0].1, 0..=1);
                assert_eq!(result.batches[0].0.known_item_count(), Some(2));
                assert!(result.completed_inputs.is_empty());
            } else {
                assert!(result.batches.is_empty());
                assert_eq!(result.completed_inputs, [0, 1]);
            }
        }
    }
}
