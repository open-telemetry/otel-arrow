// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Inline runtime storage and representation-independent pdata capabilities.

use std::borrow::Cow;

use bytes::{Bytes, BytesMut};
use otel_arrow_dfe_config::{SignalFormat, SignalType};
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers, OtlpProtoBytes};
use prost::Message;

use crate::{
    CodecError, CodecRegistry, CodecService, EncodeOutput, EncodedPdata, EncodingPlan,
    InspectionPlan, PdataEncoding, PdataView, ResolvedCodec,
};

/// Concrete inline storage used during the transition from specialized OTLP bytes.
#[derive(Clone, Debug)]
pub enum PayloadStorage {
    /// Compatibility storage for serialized OTLP service requests.
    OtlpBytes(OtlpProtoBytes),
    /// Independently decodable bytes and their resolved codec identity.
    Encoded(EncodedPdata),
    /// Native mutable OTAP Arrow records.
    OtapArrowRecords(OtapArrowRecords),
}

impl PayloadStorage {
    fn signal_type(&self) -> SignalType {
        match self {
            Self::OtlpBytes(bytes) => bytes.signal_type(),
            Self::Encoded(encoded) => encoded.signal_type(),
            Self::OtapArrowRecords(records) => records.signal_type(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::OtlpBytes(bytes) => bytes.is_empty(),
            Self::Encoded(encoded) => encoded.bytes().is_empty(),
            Self::OtapArrowRecords(records) => records.is_empty(),
        }
    }

    fn retained_memory_bytes(&self) -> usize {
        match self {
            Self::OtlpBytes(bytes) => bytes.retained_memory_bytes(),
            Self::Encoded(encoded) => encoded.bytes().len(),
            Self::OtapArrowRecords(records) => records.retained_memory_bytes(),
        }
    }
}

/// Logical pdata representation identity, independent of compatibility storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdataFormat {
    /// Native mutable OTAP records.
    Otap,
    /// Independently encoded bytes understood by a registered codec.
    Encoded(ResolvedCodec),
}

impl PdataFormat {
    /// Native OTAP representation.
    pub const OTAP: Self = Self::Otap;

    /// Builds an encoded representation identity.
    #[must_use]
    pub const fn encoded(codec: ResolvedCodec) -> Self {
        Self::Encoded(codec)
    }

    /// Returns the encoded codec identity, if this is a byte representation.
    #[must_use]
    pub const fn codec(self) -> Option<ResolvedCodec> {
        match self {
            Self::Otap => None,
            Self::Encoded(codec) => Some(codec),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CachedMeasurement(usize);

impl CachedMeasurement {
    const UNKNOWN: usize = usize::MAX;

    const fn unknown() -> Self {
        Self(Self::UNKNOWN)
    }

    const fn get(self) -> Option<usize> {
        if self.0 == Self::UNKNOWN {
            None
        } else {
            Some(self.0)
        }
    }

    fn set(&mut self, value: usize) {
        debug_assert_ne!(value, Self::UNKNOWN, "measurement exceeds supported range");
        self.0 = value;
    }

    fn take(&mut self) -> Self {
        std::mem::replace(self, Self::unknown())
    }
}

/// Runtime pdata value with inline storage and representation-local caches.
#[derive(Clone, Debug)]
pub struct PdataPayload {
    storage: PayloadStorage,
    item_count: CachedMeasurement,
    size: CachedMeasurement,
}

/// Transitional name retained while nodes migrate to [`PdataPayload`].
pub type OtapPayload = PdataPayload;

/// Transitional storage name retained while callers migrate to capabilities.
pub type PayloadData = PayloadStorage;

/// A failed native conversion together with the exact recoverable input.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct PdataPayloadDecodeError(Box<PdataPayloadDecodeErrorInner>);

#[derive(Debug, thiserror::Error)]
#[error("{source}")]
struct PdataPayloadDecodeErrorInner {
    #[source]
    source: CodecError,
    payload: PdataPayload,
}

impl PdataPayloadDecodeError {
    /// Returns the codec error.
    #[must_use]
    pub const fn error(&self) -> &CodecError {
        &self.0.source
    }

    /// Returns the exact payload retained for retry or Nack.
    #[must_use]
    pub const fn payload(&self) -> &PdataPayload {
        &self.0.payload
    }

    /// Splits the codec error and recoverable payload.
    #[must_use]
    pub fn into_parts(self) -> (CodecError, PdataPayload) {
        let inner = *self.0;
        (inner.source, inner.payload)
    }
}

impl PdataPayload {
    fn from_storage(storage: PayloadStorage) -> Self {
        Self {
            storage,
            item_count: CachedMeasurement::unknown(),
            size: CachedMeasurement::unknown(),
        }
    }

    /// Constructs transitional OTLP storage through the validated registry.
    #[must_use]
    pub fn from_otlp(bytes: OtlpProtoBytes) -> Self {
        Self::from_storage(PayloadStorage::OtlpBytes(bytes))
    }

    /// Constructs native OTAP storage.
    #[must_use]
    pub fn from_otap(records: OtapArrowRecords) -> Self {
        Self::from_storage(PayloadStorage::OtapArrowRecords(records))
    }

    /// Wraps admitted encoded bytes without copying or decoding them.
    #[must_use]
    pub fn from_encoded(encoded: EncodedPdata) -> Self {
        Self::from_storage(PayloadStorage::Encoded(encoded))
    }

    /// Supplies a receiver-known item count without parsing encoded bytes.
    #[must_use]
    pub fn with_item_count(mut self, item_count: usize) -> Self {
        self.item_count.set(item_count);
        self
    }

    /// Logical representation identity.
    #[must_use]
    pub fn format(&self) -> PdataFormat {
        match &self.storage {
            PayloadStorage::OtlpBytes(bytes) => {
                PdataFormat::encoded(otlp_codec(bytes.signal_type()))
            }
            PayloadStorage::Encoded(encoded) => PdataFormat::encoded(encoded.codec()),
            PayloadStorage::OtapArrowRecords(_) => PdataFormat::OTAP,
        }
    }

    /// Encoding identity for byte representations.
    #[must_use]
    pub fn encoding(&self) -> Option<&PdataEncoding> {
        self.format().codec().map(ResolvedCodec::encoding)
    }

    /// Borrows existing encoded bytes without conversion.
    #[must_use]
    pub fn encoded_bytes(&self) -> Option<&Bytes> {
        match &self.storage {
            PayloadStorage::OtlpBytes(bytes) => Some(bytes.bytes()),
            PayloadStorage::Encoded(encoded) => Some(encoded.bytes()),
            PayloadStorage::OtapArrowRecords(_) => None,
        }
    }

    /// Consumes an encoded payload and returns its shared byte buffer.
    #[must_use]
    pub fn into_encoded_bytes(self) -> Option<Bytes> {
        match self.storage {
            PayloadStorage::OtlpBytes(bytes) => Some(bytes.into_bytes()),
            PayloadStorage::Encoded(encoded) => Some(encoded.into_bytes()),
            PayloadStorage::OtapArrowRecords(_) => None,
        }
    }

    /// Borrows native records only when already materialized.
    #[must_use]
    pub fn otap_ref(&self) -> Option<&OtapArrowRecords> {
        match &self.storage {
            PayloadStorage::OtapArrowRecords(records) => Some(records),
            _ => None,
        }
    }

    /// Borrows the transitional concrete storage.
    #[must_use]
    pub const fn storage(&self) -> &PayloadStorage {
        &self.storage
    }

    /// Consumes the payload and returns transitional concrete storage.
    #[must_use]
    pub fn into_storage(self) -> PayloadStorage {
        self.storage
    }

    /// Transitional alias for [`Self::storage`].
    #[must_use]
    pub const fn data(&self) -> &PayloadStorage {
        self.storage()
    }

    /// Transitional alias for [`Self::into_storage`].
    #[must_use]
    pub fn into_data(self) -> PayloadStorage {
        self.into_storage()
    }

    /// Returns the telemetry signal.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        self.storage.signal_type()
    }

    /// Transitional storage format used by legacy batching.
    #[must_use]
    pub const fn signal_format(&self) -> SignalFormat {
        match self.storage {
            PayloadStorage::OtlpBytes(_) => SignalFormat::OtlpBytes,
            PayloadStorage::Encoded(_) => SignalFormat::Encoded,
            PayloadStorage::OtapArrowRecords(_) => SignalFormat::OtapRecords,
        }
    }

    /// Returns true when the payload is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    /// Known item count, keeping unknown distinct from an empty batch.
    #[must_use]
    pub fn known_item_count(&self) -> Option<usize> {
        match &self.storage {
            PayloadStorage::OtapArrowRecords(records) => Some(records.num_items()),
            _ => self.item_count.get(),
        }
    }

    /// Returns the primary-signal item count.
    ///
    /// A codec counter may return `None` for bytes it cannot count. This method
    /// preserves the historical zero fallback, but does not cache that fallback
    /// as a known empty batch.
    pub fn num_items(&mut self) -> usize {
        if let Some(count) = self.known_item_count() {
            return count;
        }
        let count = match &self.storage {
            PayloadStorage::OtlpBytes(bytes) => bytes.num_items(),
            PayloadStorage::Encoded(encoded) => {
                let Some(count) = encoded
                    .codec()
                    .count_items(encoded.signal_type(), encoded.bytes())
                else {
                    return 0;
                };
                count
            }
            PayloadStorage::OtapArrowRecords(records) => records.num_items(),
        };
        self.item_count.set(count);
        count
    }

    /// Returns the logical size of the current representation.
    pub fn num_bytes(&mut self) -> Option<usize> {
        match &self.storage {
            PayloadStorage::OtlpBytes(bytes) => Some(bytes.bytes().len()),
            PayloadStorage::Encoded(encoded) => Some(encoded.bytes().len()),
            PayloadStorage::OtapArrowRecords(records) => {
                if let Some(size) = self.size.get() {
                    return Some(size);
                }
                let size = records.num_bytes()?;
                self.size.set(size);
                Some(size)
            }
        }
    }

    /// Returns the best available retained-memory estimate.
    #[must_use]
    pub fn retained_memory_bytes(&self) -> usize {
        self.storage.retained_memory_bytes()
    }

    /// Takes the payload while preserving its cache with the returned value.
    #[must_use]
    pub fn take_payload(&mut self) -> Self {
        let empty = match &mut self.storage {
            PayloadStorage::OtlpBytes(bytes) => PayloadStorage::OtlpBytes(bytes.take_payload()),
            PayloadStorage::Encoded(encoded) => {
                let empty = EncodedPdata::from_resolved(
                    encoded.codec(),
                    encoded.signal_type(),
                    Bytes::new(),
                );
                PayloadStorage::Encoded(std::mem::replace(encoded, empty))
            }
            PayloadStorage::OtapArrowRecords(records) => {
                PayloadStorage::OtapArrowRecords(records.take_payload())
            }
        };
        Self {
            storage: empty,
            item_count: self.item_count.take(),
            size: self.size.take(),
        }
    }

    /// Extracts native records or decodes through the supplied pipeline service.
    pub fn try_into_otap(
        self,
        codecs: &CodecService,
    ) -> Result<OtapArrowRecords, PdataPayloadDecodeError> {
        let Self {
            storage,
            item_count,
            size,
        } = self;
        match storage {
            PayloadStorage::OtapArrowRecords(records) => Ok(records),
            PayloadStorage::OtlpBytes(bytes) => {
                let codec = otlp_codec(bytes.signal_type());
                let payload = Self {
                    storage: PayloadStorage::OtlpBytes(bytes.clone()),
                    item_count,
                    size,
                };
                codecs
                    .decode_parts(codec, bytes.signal_type(), bytes.bytes())
                    .map_err(|source| decode_error(source, payload))
            }
            PayloadStorage::Encoded(encoded) => {
                let payload = Self {
                    storage: PayloadStorage::Encoded(encoded.clone()),
                    item_count,
                    size,
                };
                codecs
                    .decode(&encoded)
                    .map_err(|source| decode_error(source, payload))
            }
        }
    }

    /// Returns a representation-independent read-only view.
    pub fn view<'a>(
        &'a self,
        codecs: &CodecService,
        plan: &InspectionPlan,
    ) -> Result<PdataView<'a>, CodecError> {
        match &self.storage {
            PayloadStorage::OtlpBytes(bytes) => codecs.view_parts(
                otlp_codec(bytes.signal_type()),
                bytes.signal_type(),
                bytes.bytes(),
                plan,
            ),
            PayloadStorage::Encoded(encoded) => codecs.view(encoded, plan),
            PayloadStorage::OtapArrowRecords(records) => {
                Ok(PdataView::Native(Cow::Borrowed(records)))
            }
        }
    }

    /// Runs a synchronous consumer over output prepared for a resolved plan.
    pub fn with_encoded_output<R>(
        &mut self,
        codecs: &CodecService,
        plan: &EncodingPlan,
        consume: impl FnOnce(EncodeOutput<'_>) -> R,
    ) -> Result<R, CodecError> {
        plan.codec().require_encoder(self.signal_type())?;
        match &mut self.storage {
            PayloadStorage::OtlpBytes(bytes) if otlp_codec(bytes.signal_type()) == plan.codec() => {
                Ok(consume(EncodeOutput::bytes(bytes.clone_bytes())))
            }
            PayloadStorage::Encoded(encoded) if encoded.codec() == plan.codec() => {
                Ok(consume(EncodeOutput::bytes(encoded.bytes().clone())))
            }
            PayloadStorage::OtapArrowRecords(records) => {
                codecs.with_encoded_output(records, plan, consume)
            }
            PayloadStorage::OtlpBytes(bytes) => {
                let mut records = codecs.decode_parts(
                    otlp_codec(bytes.signal_type()),
                    bytes.signal_type(),
                    bytes.bytes(),
                )?;
                codecs.with_encoded_output(&mut records, plan, consume)
            }
            PayloadStorage::Encoded(encoded) => {
                let mut records = codecs.decode(encoded)?;
                codecs.with_encoded_output(&mut records, plan, consume)
            }
        }
    }

    /// Detaches owned bytes before an asynchronous send.
    pub fn encode_bytes(
        &mut self,
        codecs: &CodecService,
        plan: &EncodingPlan,
    ) -> Result<Bytes, CodecError> {
        self.with_encoded_output(codecs, plan, |output| output.into_bytes())
    }

    /// Returns an empty transitional OTLP payload for a signal.
    #[must_use]
    pub fn empty(signal: SignalType) -> Self {
        Self::from_otlp(OtlpProtoBytes::empty(signal))
    }

    /// Test-only cache inspection.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn test_has_cached_item_count(&self) -> bool {
        self.item_count.get().is_some()
    }

    /// Test-only cache inspection.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn test_has_cached_size(&self) -> bool {
        self.size.get().is_some()
    }
}

fn decode_error(source: CodecError, payload: PdataPayload) -> PdataPayloadDecodeError {
    PdataPayloadDecodeError(Box::new(PdataPayloadDecodeErrorInner { source, payload }))
}

fn otlp_codec(signal: SignalType) -> ResolvedCodec {
    let registry = CodecRegistry::global().expect("the pdata codec registry must be valid");
    let codec = registry
        .resolve(&PdataEncoding::OTLP)
        .expect("the built-in OTLP codec must be registered");
    codec
        .require_decoder(signal)
        .expect("the built-in OTLP codec must support every OTLP signal");
    codec
}

impl From<OtapArrowRecords> for PdataPayload {
    fn from(records: OtapArrowRecords) -> Self {
        Self::from_otap(records)
    }
}

#[cfg(any(test, feature = "testing"))]
#[derive(Debug, thiserror::Error)]
pub enum CompatibilityConversionError {
    /// Low-level OTLP-to-OTAP conversion failed.
    #[error(transparent)]
    Decode(#[from] otel_arrow_dfe_pdata::encode::Error),
    /// Low-level OTAP-to-OTLP conversion failed.
    #[error(transparent)]
    Encode(#[from] otel_arrow_dfe_pdata::error::Error),
    /// A generalized encoded payload could not be decoded.
    #[error(transparent)]
    Codec(#[from] CodecError),
}

#[cfg(any(test, feature = "testing"))]
impl otel_arrow_dfe_pdata::TryFromWithOptions<PdataPayload> for OtapArrowRecords {
    type Error = CompatibilityConversionError;

    fn try_from_with_options(
        value: PdataPayload,
        options: otel_arrow_dfe_config::ConversionOptions,
    ) -> Result<Self, Self::Error> {
        match value.into_storage() {
            PayloadStorage::OtapArrowRecords(records) => Ok(records),
            PayloadStorage::OtlpBytes(bytes) => {
                Ok(<Self as otel_arrow_dfe_pdata::TryFromWithOptions<
                    OtlpProtoBytes,
                >>::try_from_with_options(bytes, options)?)
            }
            PayloadStorage::Encoded(encoded) => {
                let service = CodecService::new().map_err(CodecError::from)?;
                Ok(service.decode(&encoded)?)
            }
        }
    }
}

#[cfg(any(test, feature = "testing"))]
impl otel_arrow_dfe_pdata::TryFromWithOptions<PdataPayload> for OtlpProtoBytes {
    type Error = CompatibilityConversionError;

    fn try_from_with_options(
        value: PdataPayload,
        options: otel_arrow_dfe_config::ConversionOptions,
    ) -> Result<Self, Self::Error> {
        match value.into_storage() {
            PayloadStorage::OtlpBytes(bytes) => Ok(bytes),
            PayloadStorage::Encoded(encoded) if encoded.encoding() == &PdataEncoding::OTLP => Ok(
                Self::new_from_bytes(encoded.signal_type(), encoded.into_bytes()),
            ),
            PayloadStorage::Encoded(encoded) => {
                let service = CodecService::new().map_err(CodecError::from)?;
                let records = service.decode(&encoded)?;
                Ok(<Self as otel_arrow_dfe_pdata::TryFromWithOptions<
                    OtapArrowRecords,
                >>::try_from_with_options(records, options)?)
            }
            PayloadStorage::OtapArrowRecords(records) => {
                Ok(<Self as otel_arrow_dfe_pdata::TryFromWithOptions<
                    OtapArrowRecords,
                >>::try_from_with_options(records, options)?)
            }
        }
    }
}

impl From<OtlpProtoBytes> for PdataPayload {
    fn from(bytes: OtlpProtoBytes) -> Self {
        Self::from_otlp(bytes)
    }
}

impl From<EncodedPdata> for PdataPayload {
    fn from(encoded: EncodedPdata) -> Self {
        Self::from_encoded(encoded)
    }
}

impl From<PayloadStorage> for PdataPayload {
    fn from(storage: PayloadStorage) -> Self {
        Self::from_storage(storage)
    }
}

impl TryFrom<OtlpProtoMessage> for PdataPayload {
    type Error = prost::EncodeError;

    fn try_from(message: OtlpProtoMessage) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::new();
        Ok(match message {
            OtlpProtoMessage::Logs(logs) => {
                logs.encode(&mut bytes)?;
                OtlpProtoBytes::ExportLogsRequest(bytes.freeze()).into()
            }
            OtlpProtoMessage::Metrics(metrics) => {
                metrics.encode(&mut bytes)?;
                OtlpProtoBytes::ExportMetricsRequest(bytes.freeze()).into()
            }
            OtlpProtoMessage::Traces(traces) => {
                traces.encode(&mut bytes)?;
                OtlpProtoBytes::ExportTracesRequest(bytes.freeze()).into()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use std::sync::Arc;

    use super::*;
    use crate::{CodecMetadata, CodecRegistration};
    use otel_arrow_dfe_pdata::otap::Logs;

    const UNCOUNTABLE_ENCODING: PdataEncoding = PdataEncoding::new("uncountable-test-v1");
    static UNCOUNTABLE_METADATA: CodecMetadata =
        CodecMetadata::new(UNCOUNTABLE_ENCODING, &[SignalType::Logs]);
    static UNCOUNTABLE_REGISTRATIONS: [CodecRegistration; 1] =
        [CodecRegistration::new(&UNCOUNTABLE_METADATA)
            .with_decoder(|_| Box::new(UnusedDecoder))
            .with_item_counter(|_, _| None)];

    struct UnusedDecoder;

    impl crate::PdataDecoder for UnusedDecoder {
        fn decode(
            &mut self,
            _signal: SignalType,
            _bytes: &Bytes,
        ) -> Result<OtapArrowRecords, CodecError> {
            unreachable!("item counting must not instantiate the decoder")
        }
    }

    /// Scenario: Transitional OTLP, generalized encoded, and native storage is built on 64 bit.
    /// Guarantees: Inline encoded storage keeps the runtime payload at 64 bytes.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn payload_layout_is_stable() {
        assert_eq!(size_of::<EncodedPdata>(), 48);
        assert_eq!(size_of::<PayloadStorage>(), 48);
        assert_eq!(size_of::<PdataPayload>(), 64);
    }

    /// Scenario: Native records are converted into the native capability.
    /// Guarantees: Conversion moves the records without creating a codec instance.
    #[test]
    fn native_conversion_does_not_create_a_codec() {
        let codecs = CodecService::new().unwrap();
        let payload = PdataPayload::from(OtapArrowRecords::Logs(Logs::default()));
        let records = payload.try_into_otap(&codecs).unwrap();
        assert!(matches!(records, OtapArrowRecords::Logs(_)));
        assert_eq!(codecs.test_instance_count().unwrap(), 0);
    }

    /// Scenario: A decoder's stateless counter cannot determine an encoded batch's item count.
    /// Guarantees: The zero compatibility fallback is not cached as a known empty batch.
    #[test]
    fn unavailable_encoded_count_remains_unknown() {
        let registry = Arc::new(
            CodecRegistry::validate(&UNCOUNTABLE_REGISTRATIONS).expect("valid test registry"),
        );
        let codec = registry
            .resolve_decoder(&UNCOUNTABLE_ENCODING, SignalType::Logs)
            .expect("test decoder");
        let mut payload = PdataPayload::from_encoded(
            codec
                .admit(SignalType::Logs, Bytes::from_static(b"opaque"))
                .expect("supported signal"),
        );

        assert_eq!(payload.num_items(), 0);
        assert_eq!(payload.known_item_count(), None);
    }
}
