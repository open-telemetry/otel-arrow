// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Link-time codec discovery and immutable registry validation.
//!
//! Codec extension crates submit [`CodecRegistration`] values through
//! [`crate::register_pdata_codec!`]. A registration contains immutable metadata
//! and factories only; mutable decoder and encoder instances belong to the
//! pipeline-local runtime described by [`crate::CodecService`]. Registrations
//! may be one-sided, so a format can support decoding without encoding or the
//! reverse.
//!
//! [`CodecRegistry`] validates the complete linked set before a pipeline uses
//! it. Encoding names identify byte-compatible representations, and duplicate
//! or invalid names are rejected independently of linker order. Once validated,
//! [`ResolvedCodec`] is a cheap handle used for admission, capability checks,
//! stateless item counting, and lazy factory access.

use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;

use crate::{
    CodecError, CodecOperation, EncodePolicy, EncodedPdata, PdataDecoder, PdataEncoder,
    PdataEncoding, RegistryError,
};

/// Optional allocation-free item scan that needs no mutable codec instance.
pub type ItemCounter = fn(SignalType, &[u8]) -> Option<usize>;

/// Creates independent decoder state for one pipeline runtime.
pub type DecoderFactory = fn() -> Box<dyn PdataDecoder>;

/// Creates encoder state configured once for one output plan.
pub type EncoderFactory = fn(EncodePolicy) -> Result<Box<dyn PdataEncoder>, CodecError>;

/// Immutable metadata for one encoded representation.
#[derive(Debug)]
pub struct CodecMetadata {
    /// Globally stable representation name.
    encoding: PdataEncoding,
    /// Signals understood by this codec.
    signals: &'static [SignalType],
    /// Informational version. Incompatible versions require different identities.
    format_version: Option<&'static str>,
    /// Compression intrinsic to the representation, if any.
    compression: Option<&'static str>,
}

impl CodecMetadata {
    /// Declares an encoded representation and the signals it supports.
    #[must_use]
    pub const fn new(encoding: PdataEncoding, signals: &'static [SignalType]) -> Self {
        Self {
            encoding,
            signals,
            format_version: None,
            compression: None,
        }
    }

    /// Adds an informational version for diagnostics and discovery.
    ///
    /// A version that changes byte compatibility belongs in [`PdataEncoding`]
    /// instead so incompatible formats cannot resolve to the same codec.
    #[must_use]
    pub const fn with_format_version(mut self, format_version: &'static str) -> Self {
        self.format_version = Some(format_version);
        self
    }

    /// Declares compression intrinsic to the encoded representation.
    ///
    /// Transport compression, such as HTTP gzip, does not belong here.
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

    /// Returns the telemetry signals accepted by this representation.
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
}

/// Link-time codec registration containing only immutable factories.
#[derive(Debug)]
pub struct CodecRegistration {
    /// Representation identity and supported signals.
    metadata: &'static CodecMetadata,
    /// Decoder factory, absent for encode-only formats.
    decoder: Option<DecoderFactory>,
    /// Encoder factory, absent for decode-only formats.
    encoder: Option<EncoderFactory>,
    /// Optional stateless item scan used by flow metrics.
    count_items: Option<ItemCounter>,
}

impl CodecRegistration {
    /// Starts a registration with no decode or encode capabilities.
    ///
    /// Add at least one direction before submitting the registration to the
    /// registry. Registry validation rejects registrations with no factories.
    #[must_use]
    pub const fn new(metadata: &'static CodecMetadata) -> Self {
        Self {
            metadata,
            decoder: None,
            encoder: None,
            count_items: None,
        }
    }

    /// Adds a factory for pipeline-local decoder state.
    #[must_use]
    pub const fn with_decoder(mut self, decoder: DecoderFactory) -> Self {
        self.decoder = Some(decoder);
        self
    }

    /// Adds a factory for pipeline-local encoder state.
    #[must_use]
    pub const fn with_encoder(mut self, encoder: EncoderFactory) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// Adds an allocation-free item counter that needs no codec instance.
    #[must_use]
    pub const fn with_item_counter(mut self, count_items: ItemCounter) -> Self {
        self.count_items = Some(count_items);
        self
    }
}

/// Trusted codec extensions compiled into this binary.
#[allow(unsafe_code)]
#[linkme::distributed_slice]
pub static PDATA_CODEC_FACTORIES: [CodecRegistration];

/// One immutable registration from a validated registry.
#[derive(Clone, Copy, Debug)]
pub struct ResolvedCodec(&'static CodecRegistration);

impl PartialEq for ResolvedCodec {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for ResolvedCodec {}

impl Hash for ResolvedCodec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::from_ref(self.0).hash(state);
    }
}

impl ResolvedCodec {
    /// Immutable metadata and canonical encoding name.
    #[must_use]
    pub const fn metadata(self) -> &'static CodecMetadata {
        self.0.metadata
    }

    /// Canonical encoding identity.
    #[must_use]
    pub const fn encoding(self) -> &'static PdataEncoding {
        &self.0.metadata.encoding
    }

    /// Whether a decoder factory is registered.
    #[must_use]
    pub const fn can_decode(self) -> bool {
        self.0.decoder.is_some()
    }

    /// Whether an encoder factory is registered.
    #[must_use]
    pub const fn can_encode(self) -> bool {
        self.0.encoder.is_some()
    }

    /// Validates decoder support for a signal.
    pub fn require_decoder(self, signal: SignalType) -> Result<(), CodecError> {
        self.require(signal, CodecOperation::Decode, self.can_decode())
    }

    /// Validates encoder support for a signal.
    pub fn require_encoder(self, signal: SignalType) -> Result<(), CodecError> {
        self.require(signal, CodecOperation::Encode, self.can_encode())
    }

    fn require(
        self,
        signal: SignalType,
        operation: CodecOperation,
        factory_present: bool,
    ) -> Result<(), CodecError> {
        if factory_present && self.metadata().signals().contains(&signal) {
            Ok(())
        } else {
            Err(CodecError::Unsupported {
                encoding: self.encoding().clone(),
                operation,
                signal,
            })
        }
    }

    /// Counts items without instantiating mutable codec state when supported.
    #[must_use]
    pub fn count_items(self, signal: SignalType, bytes: &[u8]) -> Option<usize> {
        self.0
            .count_items
            .and_then(|counter| counter(signal, bytes))
    }

    /// Admits supported input without decoding or creating mutable codec state.
    pub fn admit(self, signal: SignalType, bytes: Bytes) -> Result<EncodedPdata, CodecError> {
        self.require_decoder(signal)?;
        Ok(EncodedPdata::from_resolved(self, signal, bytes))
    }

    pub(crate) fn create_decoder(self) -> Result<Box<dyn PdataDecoder>, CodecError> {
        self.0
            .decoder
            .map(|factory| factory())
            .ok_or_else(|| CodecError::Unsupported {
                encoding: self.encoding().clone(),
                operation: CodecOperation::Decode,
                signal: self.metadata().signals()[0],
            })
    }

    pub(crate) fn create_encoder(
        self,
        policy: EncodePolicy,
    ) -> Result<Box<dyn PdataEncoder>, CodecError> {
        self.0.encoder.ok_or_else(|| CodecError::Unsupported {
            encoding: self.encoding().clone(),
            operation: CodecOperation::Encode,
            signal: self.metadata().signals()[0],
        })?(policy)
    }
}

/// Validated immutable view of all codec extensions linked into a binary.
#[derive(Debug)]
pub struct CodecRegistry {
    codecs: Box<[ResolvedCodec]>,
}

static GLOBAL_REGISTRY: LazyLock<Result<Arc<CodecRegistry>, RegistryError>> =
    LazyLock::new(|| CodecRegistry::validate(&PDATA_CODEC_FACTORIES).map(Arc::new));

impl CodecRegistry {
    /// Returns the process registry after validating all linked registrations.
    pub fn global() -> Result<Arc<Self>, RegistryError> {
        GLOBAL_REGISTRY
            .as_ref()
            .map(Arc::clone)
            .map_err(Clone::clone)
    }

    /// Validates an explicit registration set.
    pub fn validate(factories: &'static [CodecRegistration]) -> Result<Self, RegistryError> {
        for (index, registration) in factories.iter().enumerate() {
            validate_registration(registration)?;
            if factories[..index]
                .iter()
                .any(|other| other.metadata.encoding == registration.metadata.encoding)
            {
                return Err(RegistryError::Duplicate {
                    encoding: registration.metadata.encoding.clone(),
                });
            }
        }
        Ok(Self {
            codecs: factories.iter().map(ResolvedCodec).collect(),
        })
    }

    /// Resolves a unique registered identity.
    pub fn resolve(&self, encoding: &PdataEncoding) -> Result<ResolvedCodec, RegistryError> {
        self.codecs
            .iter()
            .copied()
            .find(|codec| codec.encoding() == encoding)
            .ok_or_else(|| RegistryError::NotFound {
                encoding: encoding.clone(),
            })
    }

    /// Resolves and validates decoder support.
    pub fn resolve_decoder(
        &self,
        encoding: &PdataEncoding,
        signal: SignalType,
    ) -> Result<ResolvedCodec, CodecError> {
        let codec = self.resolve(encoding)?;
        codec.require_decoder(signal)?;
        Ok(codec)
    }

    /// Resolves and validates encoder support.
    pub fn resolve_encoder(
        &self,
        encoding: &PdataEncoding,
        signal: SignalType,
    ) -> Result<ResolvedCodec, CodecError> {
        let codec = self.resolve(encoding)?;
        codec.require_encoder(signal)?;
        Ok(codec)
    }

    /// Iterates all validated codec identities without creating mutable state.
    #[must_use]
    pub fn codecs(&self) -> impl ExactSizeIterator<Item = ResolvedCodec> + '_ {
        self.codecs.iter().copied()
    }
}

fn validate_registration(registration: &CodecRegistration) -> Result<(), RegistryError> {
    let metadata = registration.metadata;
    let encoding = &metadata.encoding;
    let name = encoding.as_str();
    if ["otap", "otlp", "preserve"].contains(&name) {
        return Err(RegistryError::InvalidIdentity {
            encoding: encoding.clone(),
            reason: "reserved format name",
        });
    }
    if name.is_empty()
        || !name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-:".contains(&byte)
        })
    {
        return Err(RegistryError::InvalidIdentity {
            encoding: encoding.clone(),
            reason: "identity must use lowercase ASCII letters, digits, '.', '_', '-', or ':'",
        });
    }
    if metadata.signals.is_empty() {
        return Err(RegistryError::EmptySignals {
            encoding: encoding.clone(),
        });
    }
    if registration.decoder.is_none() && registration.encoder.is_none() {
        return Err(RegistryError::EmptyCapabilities {
            encoding: encoding.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static EMPTY_METADATA: CodecMetadata =
        CodecMetadata::new(PdataEncoding::new("empty-v1"), &[SignalType::Logs])
            .with_format_version("1");
    static EMPTY_REGISTRATIONS: [CodecRegistration; 1] = [CodecRegistration::new(&EMPTY_METADATA)];

    /// Scenario: A linked extension declares no decoder or encoder factory.
    /// Guarantees: Production registry construction rejects empty capabilities.
    #[test]
    fn rejects_empty_capabilities() {
        assert!(matches!(
            CodecRegistry::validate(&EMPTY_REGISTRATIONS),
            Err(RegistryError::EmptyCapabilities { .. })
        ));
    }
}
