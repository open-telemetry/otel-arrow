// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batch-sink contract for reader scheduling.

use super::{CommittedFrontierGuard, Frame, ReaderKey};

/// Owned framed output. Source ranges and boundary/fragment evidence stay intact.
#[derive(Debug)]
pub(crate) struct Output {
    pub file_id: super::cp::FileId,
    pub key: ReaderKey,
    pub epoch: u32,
    pub frame: Frame,
    pub guard: CommittedFrontierGuard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Flow {
    Continue,
    Stop,
}

/// Failure explicitly returns ownership. The scheduler retains this output and
/// stops; composition must dispose it with the reader's unresolved work.
pub(crate) struct Rejected {
    pub output: Output,
}

/// Preflight includes retained batch/carry-over state
/// and all known record/byte/progress-mark limits. It runs before *every* framing
/// step, including empty-input replay, and before every source operation.
/// A preflight Stop pauses all readers, including readers already represented in
/// the batch. Composition must seal/resolve the batch or service its flush
/// deadline, then call `resume_sink` once the blocking condition clears.
///
/// Before the first turn, composition registers bounded context for each reader
/// key: file ID, epoch, initial offset and framing resume from its identity seed,
/// plus configured file metadata. The sink validates output against that context.
/// Registration and metadata updates belong to the concrete sink adapter.
///
/// Composition must service the concrete sink's flush deadline even when no
/// reader is ready or source work is paused. Reader deadlines alone are not a
/// batch timer; deadline polling must not depend on another source event.
/// The batch-builder integration must also flush when descriptor eviction finds
/// no eligible victim, then resolve progress before retrying protected readers.
pub(crate) trait BatchSink {
    fn preflight(&mut self, key: ReaderKey, epoch: u32) -> Flow;
    /// Both success values consume ownership. Stop retains at most one carry-over
    /// and blocks further source/framing work. Acceptance freezes file metadata
    /// and observed time, including for carry-over; later updates cannot rewrite
    /// accepted output. After authorized batch progress application, admit any
    /// carry-over into the next batch before composition calls `resume_sink`.
    // Return the owned frame inline: failure must not allocate to preserve it.
    #[allow(clippy::result_large_err)]
    fn offer(&mut self, output: Output) -> Result<Flow, Rejected>;
    /// True while this reader/epoch owns progress in an open batch, retained
    /// batch, or carry-over. Sealing, sending, or Ack alone cannot clear the
    /// eviction veto: retained progress must be applied by its authority first.
    /// Carry-over keeps the veto after the preceding batch is released.
    fn unresolved(&self, key: ReaderKey, epoch: u32) -> bool;
}
