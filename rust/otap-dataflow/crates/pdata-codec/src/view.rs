// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::PdataEncoding;

/// Borrowed view of an accepted encoded representation.
#[derive(Clone, Copy, Debug)]
pub struct EncodedView<'a> {
    encoding: &'a PdataEncoding,
    signal: SignalType,
    bytes: &'a [u8],
}

impl<'a> EncodedView<'a> {
    pub(crate) fn new(encoding: &'a PdataEncoding, signal: SignalType, bytes: &'a [u8]) -> Self {
        Self {
            encoding,
            signal,
            bytes,
        }
    }

    /// Encoded representation identity.
    #[must_use]
    pub const fn encoding(&self) -> &PdataEncoding {
        self.encoding
    }

    /// Signal carried outside the bytes.
    #[must_use]
    pub const fn signal_type(&self) -> SignalType {
        self.signal
    }

    /// Complete independently decodable batch bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        self.bytes
    }
}

/// Representation-neutral read-only pdata view.
pub enum PdataView<'a> {
    /// Encoded bytes explicitly accepted by the consumer's view plan.
    Encoded(EncodedView<'a>),
    /// Native records, borrowed when already native or owned after fallback decode.
    Native(Cow<'a, OtapArrowRecords>),
}

impl PdataView<'_> {
    /// Signal exposed by this view.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        match self {
            Self::Encoded(view) => view.signal_type(),
            Self::Native(records) => records.signal_type(),
        }
    }
}
