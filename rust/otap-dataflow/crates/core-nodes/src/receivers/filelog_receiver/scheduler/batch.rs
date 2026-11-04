// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small Task 16 boundary; no batch builder or delivery state machine.

use super::{CommittedFrontierGuard, Frame, ReaderKey};

/// Owned framed output. Source ranges and boundary/fragment evidence stay intact.
#[derive(Debug)]
pub(super) struct Output {
    pub key: ReaderKey,
    pub epoch: u64,
    pub frame: Frame,
    pub guard: CommittedFrontierGuard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Flow {
    Continue,
    Stop,
}

/// Failure explicitly returns ownership. The scheduler retains this output and
/// stops; composition must dispose it with the reader's unresolved work.
pub(super) struct Rejected {
    pub output: Output,
}

/// Task 16 review boundary. Preflight includes retained batch/carry-over state
/// and all known record/byte/progress-mark limits. It runs before *every* framing
/// step, including empty-input replay, and before every source operation.
pub(super) trait BatchSink {
    fn preflight(&mut self, key: ReaderKey, epoch: u64) -> Flow;
    /// Both success values consume ownership. Stop retains at most one carry-over
    /// and must prevent subsequent admission until composition resolves it.
    // Return the owned frame inline: failure must not allocate to preserve it.
    #[allow(clippy::result_large_err)]
    fn offer(&mut self, output: Output) -> Result<Flow, Rejected>;
    fn unresolved(&self, key: ReaderKey, epoch: u64) -> bool;
}
