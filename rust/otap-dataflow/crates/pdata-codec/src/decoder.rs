// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoding extension contract for encoded pdata representations.
//!
//! A decoder instance belongs to one pipeline runtime and may reuse mutable
//! scratch state across calls. Calls are synchronous and must not retain the
//! input [`Bytes`]. The caller keeps the original encoded batch, which allows
//! it to recover the exact message when decoding fails.

use bytes::Bytes;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::OtapArrowRecords;

use crate::CodecError;

/// Amount of format validation requested when decoding encoded pdata.
///
/// `BestEffort` permits codecs to use a faster parser that may not discover
/// malformed content outside the fields it visits. `Strict` requires the
/// complete encoded batch to be validated before decoded records are returned.
/// A codec may use its strict implementation for both modes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum DecodeValidation {
    /// Prefer decoding performance over validation of unvisited content.
    #[default]
    BestEffort,
    /// Reject malformed content anywhere in the encoded batch.
    Strict,
}

/// Immutable policy supplied when a pipeline-local decoder is created.
///
/// The runtime resolves this policy once for the pipeline and passes it to a
/// codec's decoder factory. Implementations should select a concrete decoder
/// strategy at that boundary instead of branching while visiting each field.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DecodePolicy {
    validation: DecodeValidation,
}

impl DecodePolicy {
    /// Creates a decode policy with the requested validation level.
    #[must_use]
    pub const fn new(validation: DecodeValidation) -> Self {
        Self { validation }
    }

    /// Returns the requested validation level.
    #[must_use]
    pub const fn validation(self) -> DecodeValidation {
        self.validation
    }
}

/// Converts complete encoded batches into native OTAP Arrow records.
///
/// Each input must be independently decodable; codecs must not rely on state
/// from an earlier batch. The returned records must carry the same signal as
/// the `signal` argument. The runtime checks this invariant and ensures every
/// implementation failure is reported with the resolved codec identity and
/// operation. Implementations may add the same context themselves; the runtime
/// avoids wrapping it twice.
///
/// Implementations may keep reusable scratch storage in `self`. Instances are
/// created lazily and reused only within one pipeline runtime. They must be
/// `Send` so the surrounding runtime service remains usable by both local and
/// shared engine variants, although calls do not cross an async suspension.
/// Decoder factories receive the pipeline's [`DecodePolicy`] once when the
/// instance is created.
pub trait PdataDecoder: Send {
    /// Decodes one complete batch while the caller retains the original bytes.
    ///
    /// Return [`CodecError`] for malformed input, unsupported content, limits,
    /// or any representation-specific failure. Do not panic on untrusted bytes.
    fn decode(&mut self, signal: SignalType, bytes: &Bytes)
    -> Result<OtapArrowRecords, CodecError>;
}
