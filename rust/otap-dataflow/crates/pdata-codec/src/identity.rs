// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::{Borrow, Cow};
use std::fmt;

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;

use crate::{CodecError, CodecRegistry, ResolvedCodec};

/// A complete encoded batch constructed by a codec, with an established item count.
///
/// Codec extensions implement this on an opaque value returned by their own
/// builder. Keep its fields private: receivers should append messages through
/// that builder, not supply arbitrary bytes and a claimed count. Use
/// [`ResolvedCodec::admit`] instead for bytes received from an external source.
///
/// # Contract
///
/// The bytes must be independently decodable using [`Self::ENCODING`] for the
/// reported signal. The count must equal the codec's primary-signal item count,
/// including its documented treatment of invalid records, and be less than
/// `usize::MAX`. Counts are trusted just like a registered codec counter; this
/// trait is an extension contract, not a security boundary against faulty codecs.
///
/// [`ResolvedCodec::admit_built`] checks identity, signal support, and count range,
/// then moves the bytes and caches the count without scanning or decoding them.
/// The trait is statically dispatched and adds no payload allocation or storage.
/// A known count is not a decode-validation certificate: later materialization
/// still uses the selected decoder and may reject malformed record contents.
pub trait EncodedPdataBatch: Sized {
    /// Byte-compatible encoding emitted by this codec-owned builder.
    const ENCODING: PdataEncoding;

    /// Primary signal represented by the constructed batch.
    fn signal_type(&self) -> SignalType;

    /// Moves the finished bytes and their established primary-signal item count.
    fn into_parts(self) -> (Bytes, usize);
}

/// Stable identity of an independently decodable byte representation.
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Inline envelope for an independently encoded telemetry batch.
#[derive(Clone, Debug)]
pub struct EncodedPdata {
    codec: ResolvedCodec,
    signal: SignalType,
    bytes: Bytes,
}

impl EncodedPdata {
    /// Resolves and admits bytes without parsing or copying their contents.
    pub fn new(
        registry: &CodecRegistry,
        encoding: &PdataEncoding,
        signal: SignalType,
        bytes: Bytes,
    ) -> Result<Self, CodecError> {
        registry
            .resolve_decoder(encoding, signal)?
            .admit(signal, bytes)
    }

    pub(crate) const fn from_resolved(
        codec: ResolvedCodec,
        signal: SignalType,
        bytes: Bytes,
    ) -> Self {
        Self {
            codec,
            signal,
            bytes,
        }
    }

    /// Stable representation identity.
    #[must_use]
    pub fn encoding(&self) -> &PdataEncoding {
        self.codec.encoding()
    }

    /// Resolved codec identity.
    #[must_use]
    pub const fn codec(&self) -> ResolvedCodec {
        self.codec
    }

    /// Signal carried outside the encoded bytes.
    #[must_use]
    pub const fn signal_type(&self) -> SignalType {
        self.signal
    }

    /// Borrows the original shared buffer.
    #[must_use]
    pub const fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    /// Takes ownership of the shared buffer without copying it.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}
