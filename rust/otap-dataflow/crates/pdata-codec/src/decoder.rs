// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoding extension contract for encoded pdata representations.
//!
//! A decoder instance belongs to one pipeline runtime and may reuse mutable
//! scratch state across calls. [`PdataDecoder::decode`] is synchronous and must
//! not retain the input [`Bytes`]. [`PdataDecoder::start_decode`] may instead
//! return an independently owned [`PdataDecodeJob`] for bounded incremental work.
//! Jobs may share the input bytes, but cannot borrow mutable decoder state.
//! The caller retains the original message for exact failure recovery.

use std::time::Duration;

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
/// shared engine variants. Mutable access to the instance never crosses an
/// async suspension; an incremental job must own all of its progress.
/// Decoder factories receive the pipeline's [`DecodePolicy`] once when the
/// instance is created.
/// Decoding may optionally start an owned incremental job. Such jobs must not
/// borrow mutable decoder state: the service releases that instance before the
/// first suspension, and other nodes may use it while the job is pending.
pub trait PdataDecoder: Send {
    /// Starts a decode without allocating a future for ordinary synchronous codecs.
    ///
    /// The default completes immediately. Incremental implementations return an
    /// independently owned job containing all progress and temporary buffers.
    fn start_decode(
        &mut self,
        signal: SignalType,
        bytes: &Bytes,
    ) -> Result<DecodeTask, CodecError> {
        self.decode(signal, bytes).map(DecodeTask::Ready)
    }

    /// Decodes one complete batch while the caller retains the original bytes.
    ///
    /// Return [`CodecError`] for malformed input, unsupported content, limits,
    /// or any representation-specific failure. Do not panic on untrusted bytes.
    fn decode(&mut self, signal: SignalType, bytes: &Bytes)
    -> Result<OtapArrowRecords, CodecError>;
}

/// A completed decode or an independently owned resumable conversion.
///
/// Only incremental conversions allocate a job. Native moves and ordinary
/// synchronous decoders retain their existing ownership and storage model.
pub enum DecodeTask {
    /// A synchronous decoder has already returned all records.
    Ready(OtapArrowRecords),
    /// The caller drives bounded work and yields between pending steps.
    Incremental(Box<dyn PdataDecodeJob>),
}

/// Work limits for one uninterrupted incremental step.
///
/// These are cooperative budgets, not hard real-time guarantees. One record or
/// a finalization operation may exceed them and must be measured separately.
#[derive(Clone, Copy, Debug)]
pub struct DecodeBudget {
    max_records: usize,
    max_bytes: usize,
    max_duration: Duration,
}

impl Default for DecodeBudget {
    fn default() -> Self {
        Self {
            max_records: 128,
            max_bytes: 64 * 1024,
            max_duration: Duration::from_millis(1),
        }
    }
}

impl DecodeBudget {
    /// Maximum records processed before offering another scheduling opportunity.
    #[must_use]
    pub const fn max_records(self) -> usize {
        self.max_records
    }

    /// Maximum encoded message bytes processed before offering another opportunity.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }

    /// Target uninterrupted time, checked periodically rather than per record.
    #[must_use]
    pub const fn max_duration(self) -> Duration {
        self.max_duration
    }
}

/// One bounded, independently owned batch conversion.
///
/// Returning `None` requests a scheduling opportunity. Return records only after
/// the complete batch is validated; partial output must never escape. Dropping
/// a job releases its partial builder without acknowledging the original input.
/// Jobs cannot borrow the decoder or the service's runtime lock. Every pending
/// step must make bounded CPU progress. This is not an external-I/O readiness
/// interface: returning pending while waiting for I/O would cause a busy loop.
///
/// Jobs must be `Send` because shared-engine processors and exporters retain
/// them across suspension in `Send` caller futures. This excludes local-only
/// state such as [`std::rc::Rc`], even when used by a local pipeline. The bound
/// does not offload decoding: each step runs synchronously in its calling task.
pub trait PdataDecodeJob: Send {
    /// Performs one bounded portion of work, or finalizes the single output batch.
    fn step(&mut self, budget: DecodeBudget) -> Result<Option<OtapArrowRecords>, CodecError>;
}
