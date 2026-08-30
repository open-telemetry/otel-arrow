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

/// Converts complete encoded batches into native OTAP Arrow records.
///
/// Each input must be independently decodable; codecs must not rely on state
/// from an earlier batch. The returned records must carry the same signal as
/// the `signal` argument. The runtime checks this invariant and wraps failures
/// with the codec identity and operation for diagnostics.
///
/// Implementations may keep reusable scratch storage in `self`. Instances are
/// created lazily and reused only within one pipeline runtime. They must be
/// `Send` so the surrounding runtime service remains usable by both local and
/// shared engine variants, although calls do not cross an async suspension.
pub trait PdataDecoder: Send {
    /// Decodes one complete batch while the caller retains the original bytes.
    ///
    /// Return [`CodecError`] for malformed input, unsupported content, limits,
    /// or any representation-specific failure. Do not panic on untrusted bytes.
    fn decode(&mut self, signal: SignalType, bytes: &Bytes)
    -> Result<OtapArrowRecords, CodecError>;
}
