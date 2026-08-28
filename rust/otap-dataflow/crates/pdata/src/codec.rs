// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pluggable codecs between independent encoded batches and native OTAP.
//!
//! Registrations are immutable and linked into the binary. Mutable codec state
//! is created lazily by a pipeline-owned [`CodecState`]. This module deliberately
//! does not participate in payload storage yet; the legacy OTLP representation
//! remains the runtime default while codec behavior is characterized in isolation.

use std::borrow::{Borrow, Cow};
use std::fmt;

use bytes::Bytes;
use otel_arrow_dfe_config::{EncodeOptions, SignalType};

use crate::error::Error;
use crate::otap::OtapArrowRecords;
use crate::otlp::logs::LogsProtoBytesEncoder;
use crate::otlp::metrics::MetricsProtoBytesEncoder;
use crate::otlp::traces::TracesProtoBytesEncoder;
use crate::otlp::{BoundedBuf, ProtoBuffer, ProtoBytesEncoder};
use crate::views::otlp::bytes::logs::RawLogsData;
use crate::views::otlp::bytes::metrics::RawMetricsData;
use crate::views::otlp::bytes::traces::RawTraceData;
use crate::{OtapPayloadHelpers, OtlpProtoBytes, TryIntoWithOptions};

/// Stable identity of an independently decodable byte representation.
///
/// Incompatible format versions and intrinsic compression use different names.
/// HTTP and gRPC compression are transport properties and are not part of this
/// identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PdataEncoding(Cow<'static, str>);

impl PdataEncoding {
    /// OTLP protobuf service-request bytes without transport compression.
    pub const OTLP: Self = Self::new("otlp-bytes");

    /// Declares a compile-time encoding identity.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    /// Returns the stable configuration and diagnostic name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for PdataEncoding {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for PdataEncoding {
    fn from(name: String) -> Self {
        Self(Cow::Owned(name))
    }
}

impl fmt::Display for PdataEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Immutable capabilities advertised by a codec extension.
#[derive(Debug)]
pub struct PdataCodecMetadata {
    /// Globally stable representation name.
    encoding: PdataEncoding,
    /// Signals understood by this codec.
    signals: &'static [SignalType],
    /// Informational version. Incompatible versions require different identities.
    format_version: Option<&'static str>,
    /// Compression intrinsic to the representation, if any.
    compression: Option<&'static str>,
    /// Whether encoded bytes can be converted to native OTAP.
    can_decode: bool,
    /// Whether native OTAP can be converted to this representation.
    can_encode: bool,
}

impl PdataCodecMetadata {
    /// Declares the identity, signals, and directions supported by a codec.
    #[must_use]
    pub const fn new(
        encoding: PdataEncoding,
        signals: &'static [SignalType],
        can_decode: bool,
        can_encode: bool,
    ) -> Self {
        Self {
            encoding,
            signals,
            format_version: None,
            compression: None,
            can_decode,
            can_encode,
        }
    }

    /// Adds an informational format version.
    #[must_use]
    pub const fn with_format_version(mut self, format_version: &'static str) -> Self {
        self.format_version = Some(format_version);
        self
    }

    /// Declares compression intrinsic to the encoded representation.
    #[must_use]
    pub const fn with_compression(mut self, compression: &'static str) -> Self {
        self.compression = Some(compression);
        self
    }

    /// Returns the stable representation identity.
    #[must_use]
    pub const fn encoding(&self) -> &PdataEncoding {
        &self.encoding
    }

    /// Returns the supported telemetry signals.
    #[must_use]
    pub const fn signals(&self) -> &'static [SignalType] {
        self.signals
    }

    /// Returns the informational format version, when supplied.
    #[must_use]
    pub const fn format_version(&self) -> Option<&'static str> {
        self.format_version
    }

    /// Returns compression intrinsic to the representation, when supplied.
    #[must_use]
    pub const fn compression(&self) -> Option<&'static str> {
        self.compression
    }

    /// Whether encoded bytes can be converted to native OTAP.
    #[must_use]
    pub const fn can_decode(&self) -> bool {
        self.can_decode
    }

    /// Whether native OTAP can be converted to this representation.
    #[must_use]
    pub const fn can_encode(&self) -> bool {
        self.can_encode
    }
}

/// A read-only view returned directly by a codec.
///
/// Views may borrow the input bytes, but never mutable codec state. The default
/// implementation owns decoded OTAP records.
pub enum CodecView<'a> {
    /// Borrowed OTLP protobuf service-request bytes.
    OtlpBytes {
        /// Signal carried outside the bytes.
        signal: SignalType,
        /// Uncompressed service-request bytes.
        bytes: &'a [u8],
    },
    /// Native OTAP records, either borrowed or decoded for this view.
    OtapArrowRecords(Cow<'a, OtapArrowRecords>),
}

impl CodecView<'_> {
    /// Signal exposed by this view.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::OtlpBytes { signal, .. } => *signal,
            Self::OtapArrowRecords(records) => records.signal_type(),
        }
    }
}

/// Reusable synchronous codec implementation owned by one pipeline runtime.
///
/// Every encoded input and output is a complete independently decodable batch.
/// Stream-relative dictionary state is not an encoded pdata representation.
pub trait PdataCodec: Send {
    /// Decodes one complete batch while the caller retains the original bytes.
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, crate::encode::Error>;

    /// Encodes native records into an independently owned complete batch.
    fn encode(&mut self, records: OtapArrowRecords, options: EncodeOptions)
    -> Result<Bytes, Error>;

    /// Prepares output that may borrow reusable encoder storage.
    ///
    /// The default implementation delegates to [`Self::encode`]. Codecs with a
    /// scratch buffer can return it through [`EncodeOutput::buffer`].
    fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<EncodeOutput<'a>, Error> {
        self.encode(records.clone(), options)
            .map(EncodeOutput::bytes)
    }

    /// Returns a representation-specific borrowed view when available.
    fn view<'a>(
        &mut self,
        signal: SignalType,
        bytes: &'a Bytes,
    ) -> Result<CodecView<'a>, crate::encode::Error> {
        self.decode(signal, bytes)
            .map(|records| CodecView::OtapArrowRecords(Cow::Owned(records)))
    }
}

/// Optional allocation-free item scan that needs no mutable codec instance.
pub type ItemCounter = fn(SignalType, &[u8]) -> Option<usize>;

/// Link-time codec registration. Only immutable factories are shared globally.
#[derive(Debug)]
pub struct PdataCodecRegistration {
    /// Representation identity and capabilities.
    metadata: &'static PdataCodecMetadata,
    /// Creates independent mutable state for one pipeline runtime.
    create: fn() -> Box<dyn PdataCodec>,
    /// Optional stateless item scan used by flow metrics.
    count_items: Option<ItemCounter>,
}

impl PdataCodecRegistration {
    /// Declares a codec factory for immutable metadata.
    #[must_use]
    pub const fn new(
        metadata: &'static PdataCodecMetadata,
        create: fn() -> Box<dyn PdataCodec>,
    ) -> Self {
        Self {
            metadata,
            create,
            count_items: None,
        }
    }

    /// Adds an allocation-free item counter that does not require codec state.
    #[must_use]
    pub const fn with_item_counter(mut self, count_items: ItemCounter) -> Self {
        self.count_items = Some(count_items);
        self
    }
}

/// Trusted codec extensions compiled into this binary.
#[allow(unsafe_code)]
#[linkme::distributed_slice]
pub static PDATA_CODEC_FACTORIES: [PdataCodecRegistration];

/// Registers a pdata codec in the link-time registry.
///
/// The registration expression must be const evaluable. The macro deliberately
/// hides the distributed-slice implementation so extension crates do not need
/// to depend on or use `linkme` directly.
#[macro_export]
macro_rules! register_pdata_codec {
    ($name:ident, $registration:expr $(,)?) => {
        #[allow(unsafe_code)]
        #[$crate::codec::__private::distributed_slice($crate::codec::PDATA_CODEC_FACTORIES)]
        static $name: $crate::codec::PdataCodecRegistration = $registration;
    };
}

/// Implementation details used by exported macros.
#[doc(hidden)]
pub mod __private {
    pub use linkme::distributed_slice;
}

/// An immutable codec resolved from the link-time registry.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedCodec(&'static PdataCodecRegistration);

impl PartialEq for ResolvedCodec {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for ResolvedCodec {}

impl ResolvedCodec {
    /// Built-in OTLP, available without a registry scan.
    pub const OTLP: Self = Self(&OTLP_CODEC);

    /// Immutable capabilities and canonical encoding name.
    #[must_use]
    pub const fn metadata(self) -> &'static PdataCodecMetadata {
        self.0.metadata
    }

    /// Checks signal and direction without another registry lookup.
    pub fn require(self, signal: SignalType, direction: CodecDirection) -> Result<(), Error> {
        let metadata = self.metadata();
        if !metadata.signals.contains(&signal) {
            return Err(codec_error(
                &metadata.encoding,
                format!("unsupported signal {signal:?}"),
            ));
        }
        match direction {
            CodecDirection::Decode if !metadata.can_decode => {
                Err(codec_error(&metadata.encoding, "decoder unavailable"))
            }
            CodecDirection::Encode if !metadata.can_encode => {
                Err(codec_error(&metadata.encoding, "encoder unavailable"))
            }
            _ => Ok(()),
        }
    }

    /// Counts items without instantiating mutable codec state when supported.
    #[must_use]
    pub fn count_items(self, signal: SignalType, bytes: &[u8]) -> Option<usize> {
        self.0
            .count_items
            .and_then(|counter| counter(signal, bytes))
    }
}

/// Startup-resolved output representation and output-specific options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodingPlan {
    codec: ResolvedCodec,
    options: EncodeOptions,
}

impl EncodingPlan {
    /// Default OTLP protobuf output.
    pub const OTLP: Self = Self {
        codec: ResolvedCodec::OTLP,
        options: EncodeOptions {
            otlp_size_limit: None,
        },
    };

    /// Builds a plan from an already resolved codec.
    pub fn new(codec: ResolvedCodec, options: EncodeOptions) -> Result<Self, Error> {
        if !codec.metadata().can_encode {
            return Err(codec_error(
                &codec.metadata().encoding,
                "encoder unavailable",
            ));
        }
        Ok(Self { codec, options })
    }

    /// Resolves an output name once while constructing a node.
    pub fn resolve(encoding: &PdataEncoding, options: EncodeOptions) -> Result<Self, Error> {
        Self::new(find(encoding)?, options)
    }

    /// Resolved output codec.
    #[must_use]
    pub const fn codec(self) -> ResolvedCodec {
        self.codec
    }

    /// Output-specific options.
    #[must_use]
    pub const fn options(self) -> EncodeOptions {
        self.options
    }
}

/// Mutable codec instances cached by one pipeline runtime.
#[derive(Default)]
pub struct CodecState {
    codecs: Vec<(ResolvedCodec, Box<dyn PdataCodec>)>,
}

impl CodecState {
    fn instance(&mut self, codec: ResolvedCodec) -> &mut dyn PdataCodec {
        let index = match self.codecs.iter().position(|(key, _)| *key == codec) {
            Some(index) => index,
            None => {
                let index = self.codecs.len();
                self.codecs.push((codec, (codec.0.create)()));
                index
            }
        };
        self.codecs[index].1.as_mut()
    }

    /// Decodes with reusable runtime state and verifies signal preservation.
    pub fn decode(
        &mut self,
        codec: ResolvedCodec,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, crate::encode::Error> {
        codec.require(signal, CodecDirection::Decode)?;
        let records = self
            .instance(codec)
            .decode(signal, bytes)
            .map_err(|error| codec_error(&codec.metadata().encoding, error.to_string()))?;
        if records.signal_type() != signal {
            return Err(codec_error(
                &codec.metadata().encoding,
                "decoder changed the signal type",
            )
            .into());
        }
        Ok(records)
    }

    /// Creates a view through the same reusable codec instance.
    pub fn view<'a>(
        &mut self,
        codec: ResolvedCodec,
        signal: SignalType,
        bytes: &'a Bytes,
    ) -> Result<CodecView<'a>, crate::encode::Error> {
        codec.require(signal, CodecDirection::Decode)?;
        let view = self
            .instance(codec)
            .view(signal, bytes)
            .map_err(|error| codec_error(&codec.metadata().encoding, error.to_string()))?;
        if view.signal_type() != signal {
            return Err(
                codec_error(&codec.metadata().encoding, "view changed the signal type").into(),
            );
        }
        Ok(view)
    }

    /// Encodes with a startup-resolved output plan.
    pub fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
        plan: &EncodingPlan,
    ) -> Result<EncodeOutput<'a>, Error> {
        plan.codec
            .require(records.signal_type(), CodecDirection::Encode)?;
        self.instance(plan.codec)
            .prepare_encode(records, plan.options)
    }

    /// Number of mutable codec instances created so far.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn test_instance_count(&self) -> usize {
        self.codecs.len()
    }
}

enum OutputStorage<'a> {
    Bytes(Bytes),
    Buffer(BufferOutput<'a>),
}

struct BufferOutput<'a> {
    buffer: &'a mut ProtoBuffer,
    max_retained_capacity: usize,
}

impl Drop for BufferOutput<'_> {
    fn drop(&mut self) {
        self.buffer.retain_capacity(self.max_retained_capacity);
    }
}

/// Prepared encoded output backed by owned bytes or reusable scratch storage.
pub struct EncodeOutput<'a>(OutputStorage<'a>);

impl<'a> EncodeOutput<'a> {
    /// Wraps independently owned bytes.
    #[must_use]
    pub fn bytes(bytes: Bytes) -> Self {
        Self(OutputStorage::Bytes(bytes))
    }

    /// Borrows reusable codec storage for synchronous consumption.
    #[must_use]
    pub fn buffer(buffer: &'a mut ProtoBuffer, max_retained_capacity: usize) -> Self {
        Self(OutputStorage::Buffer(BufferOutput {
            buffer,
            max_retained_capacity,
        }))
    }

    /// Detaches bytes for an asynchronous send without copying their contents.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        match self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => {
                let (bytes, capacity) = output.buffer.take_into_bytes();
                output
                    .buffer
                    .ensure_capacity(capacity.min(output.max_retained_capacity));
                bytes
            }
        }
    }

    /// Keeps scratch attached and copies only when output uses that scratch.
    #[must_use]
    pub fn copy_into_bytes(self) -> Bytes {
        match self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => Bytes::copy_from_slice(output.buffer.as_ref()),
        }
    }
}

impl AsRef<[u8]> for EncodeOutput<'_> {
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            OutputStorage::Bytes(bytes) => bytes,
            OutputStorage::Buffer(output) => output.buffer.as_ref(),
        }
    }
}

/// The operation required by a codec consumer.
#[derive(Clone, Copy, Debug)]
pub enum CodecDirection {
    /// Encoded bytes to native OTAP.
    Decode,
    /// Native OTAP to encoded bytes.
    Encode,
}

/// Finds a unique codec without creating mutable state.
pub fn find(encoding: &PdataEncoding) -> Result<ResolvedCodec, Error> {
    let mut matches = PDATA_CODEC_FACTORIES
        .iter()
        .filter(|registration| &registration.metadata.encoding == encoding);
    let registration = matches
        .next()
        .ok_or_else(|| codec_error(encoding, "no codec registered"))?;
    if matches.next().is_some() {
        return Err(codec_error(encoding, "duplicate encoding registration"));
    }
    Ok(ResolvedCodec(registration))
}

/// Resolves a codec and validates its signal and direction.
pub fn resolve(
    encoding: &PdataEncoding,
    signal: SignalType,
    direction: CodecDirection,
) -> Result<ResolvedCodec, Error> {
    let codec = find(encoding)?;
    codec.require(signal, direction)?;
    Ok(codec)
}

/// Iterates all compiled-in codec identities without creating mutable state.
pub fn registered_codecs() -> impl Iterator<Item = ResolvedCodec> {
    PDATA_CODEC_FACTORIES.iter().map(ResolvedCodec)
}

/// Validates names, capabilities, and uniqueness of linked registrations.
pub fn validate_registrations() -> Result<(), Error> {
    validate_factories(&PDATA_CODEC_FACTORIES)
}

fn validate_factories(factories: &[PdataCodecRegistration]) -> Result<(), Error> {
    for (index, registration) in factories.iter().enumerate() {
        let metadata = registration.metadata;
        let name = metadata.encoding.as_str();
        if ["otap", "otlp", "preserve"].contains(&name) {
            return Err(codec_error(&metadata.encoding, "reserved format name"));
        }
        if name.is_empty()
            || !name.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-:".contains(&byte)
            })
        {
            return Err(codec_error(
                &metadata.encoding,
                "identity must use lowercase ASCII letters, digits, '.', '_', '-', or ':'",
            ));
        }
        if metadata.signals.is_empty() || (!metadata.can_decode && !metadata.can_encode) {
            return Err(codec_error(
                &metadata.encoding,
                "must advertise a signal and an encoder or decoder",
            ));
        }
        if factories[..index]
            .iter()
            .any(|other| other.metadata.encoding == metadata.encoding)
        {
            return Err(codec_error(
                &metadata.encoding,
                "duplicate encoding registration",
            ));
        }
    }
    Ok(())
}

fn codec_error(encoding: &PdataEncoding, reason: impl Into<String>) -> Error {
    Error::PdataCodec {
        encoding: encoding.clone(),
        reason: reason.into(),
    }
}

const OTLP_INITIAL_BUFFER_CAPACITY: usize = 8 * 1024;
const OTLP_MAX_RETAINED_BUFFER_CAPACITY: usize = 256 * 1024;

struct OtlpSignalEncoder<E> {
    encoder: E,
    buffer: ProtoBuffer,
}

impl<E: Default> Default for OtlpSignalEncoder<E> {
    fn default() -> Self {
        Self {
            encoder: E::default(),
            buffer: ProtoBuffer::with_capacity(OTLP_INITIAL_BUFFER_CAPACITY),
        }
    }
}

#[derive(Default)]
struct OtlpEncoderState {
    logs: Option<Box<OtlpSignalEncoder<LogsProtoBytesEncoder>>>,
    metrics: Option<Box<OtlpSignalEncoder<MetricsProtoBytesEncoder>>>,
    traces: Option<Box<OtlpSignalEncoder<TracesProtoBytesEncoder>>>,
}

/// Built-in OTLP protobuf codec.
#[derive(Default)]
pub struct OtlpCodec {
    encoder: Option<Box<OtlpEncoderState>>,
}

impl OtlpCodec {
    fn output_limit(options: EncodeOptions) -> usize {
        options
            .otlp_size_limit
            .map_or(crate::otlp::common::MAX_OTLP_SIZE_LIMIT, |limit| {
                limit.get().min(crate::otlp::common::MAX_OTLP_SIZE_LIMIT)
            })
    }

    #[inline(never)]
    fn prepare_logs<'a>(
        state: &'a mut OtlpEncoderState,
        records: &mut OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<EncodeOutput<'a>, Error> {
        let state = state
            .logs
            .get_or_insert_with(|| Box::new(OtlpSignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, options)
    }

    #[inline(never)]
    fn prepare_metrics<'a>(
        state: &'a mut OtlpEncoderState,
        records: &mut OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<EncodeOutput<'a>, Error> {
        let state = state
            .metrics
            .get_or_insert_with(|| Box::new(OtlpSignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, options)
    }

    #[inline(never)]
    fn prepare_traces<'a>(
        state: &'a mut OtlpEncoderState,
        records: &mut OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<EncodeOutput<'a>, Error> {
        let state = state
            .traces
            .get_or_insert_with(|| Box::new(OtlpSignalEncoder::default()));
        prepare_signal(&mut state.encoder, &mut state.buffer, records, options)
    }
}

fn prepare_signal<'a, E: ProtoBytesEncoder>(
    encoder: &mut E,
    buffer: &'a mut ProtoBuffer,
    records: &mut OtapArrowRecords,
    options: EncodeOptions,
) -> Result<EncodeOutput<'a>, Error> {
    buffer.clear();
    buffer.set_limit(OtlpCodec::output_limit(options));
    if let Err(error) = encoder.encode(records, buffer) {
        buffer.retain_capacity(OTLP_MAX_RETAINED_BUFFER_CAPACITY);
        return Err(error);
    }
    Ok(EncodeOutput::buffer(
        buffer,
        OTLP_MAX_RETAINED_BUFFER_CAPACITY,
    ))
}

impl PdataCodec for OtlpCodec {
    fn decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<OtapArrowRecords, crate::encode::Error> {
        match signal {
            SignalType::Logs => {
                let _ = RawLogsData::try_new(bytes)?;
            }
            SignalType::Metrics => {
                let _ = RawMetricsData::try_new(bytes)?;
            }
            SignalType::Traces => {
                let _ = RawTraceData::try_new(bytes)?;
            }
        }
        OtlpProtoBytes::new_from_bytes(signal, bytes.clone()).try_into_with_default()
    }

    fn encode(
        &mut self,
        mut records: OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<Bytes, Error> {
        Ok(self.prepare_encode(&mut records, options)?.into_bytes())
    }

    fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
        options: EncodeOptions,
    ) -> Result<EncodeOutput<'a>, Error> {
        let state = self
            .encoder
            .get_or_insert_with(|| Box::new(OtlpEncoderState::default()));
        match records.signal_type() {
            SignalType::Logs => Self::prepare_logs(state, records, options),
            SignalType::Metrics => Self::prepare_metrics(state, records, options),
            SignalType::Traces => Self::prepare_traces(state, records, options),
        }
    }

    fn view<'a>(
        &mut self,
        signal: SignalType,
        bytes: &'a Bytes,
    ) -> Result<CodecView<'a>, crate::encode::Error> {
        Ok(CodecView::OtlpBytes { signal, bytes })
    }
}

/// Metadata for the built-in OTLP codec.
pub static OTLP_METADATA: PdataCodecMetadata = PdataCodecMetadata::new(
    PdataEncoding::OTLP,
    &[SignalType::Logs, SignalType::Metrics, SignalType::Traces],
    true,
    true,
);

crate::register_pdata_codec!(
    OTLP_CODEC,
    PdataCodecRegistration::new(&OTLP_METADATA, || Box::new(OtlpCodec::default()))
        .with_item_counter(|signal, bytes| {
            Some(crate::payload::count_otlp_items(signal, bytes))
        }),
);

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use prost::Message;

    use super::*;
    use crate::testing::codec_conformance::{DecodeConformanceCase, assert_decode_conformance};
    use crate::testing::fixtures::{
        logs_with_full_resource_and_scope, metrics_sum_with_full_resource_and_scope,
        traces_with_full_resource_and_scope,
    };

    fn logs_bytes() -> Bytes {
        logs_with_full_resource_and_scope().encode_to_vec().into()
    }

    /// Scenario: the built-in registry is resolved without constructing mutable state.
    /// Guarantees: OTLP metadata and the stateless item counter are available eagerly.
    #[test]
    fn resolves_otlp_without_codec_instance() {
        validate_registrations().unwrap();
        let codec = resolve(
            &PdataEncoding::OTLP,
            SignalType::Logs,
            CodecDirection::Decode,
        )
        .unwrap();
        let bytes = logs_bytes();
        assert_eq!(codec, ResolvedCodec::OTLP);
        assert_eq!(codec.count_items(SignalType::Logs, &bytes), Some(4));
        assert_eq!(CodecState::default().test_instance_count(), 0);
    }

    /// Scenario: OTLP bytes are viewed and then decoded through reusable codec state.
    /// Guarantees: the view borrows the original buffer and decoding preserves signal and items.
    #[test]
    fn otlp_view_and_decode_preserve_input() {
        let bytes = logs_bytes();
        let pointer = bytes.as_ptr();
        let mut state = CodecState::default();
        let view = state
            .view(ResolvedCodec::OTLP, SignalType::Logs, &bytes)
            .unwrap();
        match view {
            CodecView::OtlpBytes { signal, bytes } => {
                assert_eq!(signal, SignalType::Logs);
                assert_eq!(bytes.as_ptr(), pointer);
            }
            CodecView::OtapArrowRecords(_) => panic!("OTLP should expose a borrowed byte view"),
        }
        let records = state
            .decode(ResolvedCodec::OTLP, SignalType::Logs, &bytes)
            .unwrap();
        assert_eq!(records.signal_type(), SignalType::Logs);
        assert_eq!(records.num_items(), 4);
        assert_eq!(state.test_instance_count(), 1);
    }

    /// Scenario: native logs are encoded repeatedly through one startup-resolved plan.
    /// Guarantees: output remains independently decodable and the codec instance is reused.
    #[test]
    fn otlp_prepared_output_reuses_codec_state() {
        let bytes = logs_bytes();
        let mut decode_state = CodecState::default();
        let original = decode_state
            .decode(ResolvedCodec::OTLP, SignalType::Logs, &bytes)
            .unwrap();
        let mut encode_state = CodecState::default();
        for _ in 0..2 {
            let mut records = original.clone();
            let encoded = encode_state
                .prepare_encode(&mut records, &EncodingPlan::OTLP)
                .unwrap()
                .into_bytes();
            let decoded = decode_state
                .decode(ResolvedCodec::OTLP, SignalType::Logs, &encoded)
                .unwrap();
            assert_eq!(decoded.num_items(), 4);
        }
        assert_eq!(encode_state.test_instance_count(), 1);
    }

    /// Scenario: an output plan sets an OTLP limit smaller than the encoded batch.
    /// Guarantees: the limit reaches the reusable encoder and later encodes recover.
    #[test]
    fn otlp_encoder_recovers_after_limit_error() {
        let bytes = logs_bytes();
        let mut state = CodecState::default();
        let records = state
            .decode(ResolvedCodec::OTLP, SignalType::Logs, &bytes)
            .unwrap();
        let limited = EncodingPlan::new(
            ResolvedCodec::OTLP,
            EncodeOptions {
                otlp_size_limit: NonZeroUsize::new(1),
            },
        )
        .unwrap();
        assert!(
            state
                .prepare_encode(&mut records.clone(), &limited)
                .is_err()
        );
        let encoded = state
            .prepare_encode(&mut records.clone(), &EncodingPlan::OTLP)
            .unwrap()
            .into_bytes();
        assert!(!encoded.is_empty());
    }

    /// Scenario: OTLP logs, metrics, and traces run through the shared codec contract.
    /// Guarantees: every signal preserves items, rejects malformed input, and round-trips independently.
    #[test]
    fn otlp_codec_conforms_for_all_signals() {
        let cases = [
            (
                SignalType::Logs,
                Bytes::from(logs_with_full_resource_and_scope().encode_to_vec()),
                4,
            ),
            (
                SignalType::Metrics,
                Bytes::from(metrics_sum_with_full_resource_and_scope().encode_to_vec()),
                2,
            ),
            (
                SignalType::Traces,
                Bytes::from(traces_with_full_resource_and_scope().encode_to_vec()),
                2,
            ),
        ];
        for (signal, valid, expected_items) in cases {
            assert_decode_conformance(DecodeConformanceCase {
                codec: ResolvedCodec::OTLP,
                signal,
                valid,
                malformed: Some(Bytes::from_static(&[0x0a, 0x05, 0x01])),
                expected_items,
            });
        }
    }
}
