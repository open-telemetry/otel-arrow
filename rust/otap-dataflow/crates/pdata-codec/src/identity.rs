// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::{Borrow, Cow};
use std::fmt;

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;

use crate::{CodecError, CodecRegistry, ResolvedCodec};

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
