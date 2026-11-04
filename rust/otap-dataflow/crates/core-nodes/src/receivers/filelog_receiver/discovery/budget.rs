// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use futures::task::AtomicWaker;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::task::Waker;

const OCCUPIED: u8 = 1;
const READER: u8 = 2;

/// Shared allowance for one conversion's two transient source descriptors.
///
/// Reservations cover the conversion peak until handles close or transfer into
/// a reserved resident slot. One read worker may register a pending request;
/// discovery cannot reacquire ahead of that request. Acquisition never waits.
#[derive(Debug, Default)]
pub struct ProbeBudget {
    state: AtomicU8,
    reader_wake: AtomicWaker,
}

impl ProbeBudget {
    /// Tries a discovery/admission reservation without queuing or waiting.
    pub fn try_reserve(&self) -> Option<ProbeReservation<'_>> {
        self.state
            .compare_exchange(0, OCCUPIED, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| ProbeReservation { budget: self })
    }

    /// Registers the single read worker before checking availability.
    ///
    /// Dropping the request cancels priority. The waker must wake the worker's
    /// bounded control wait; repeated polling is neither needed nor fairness.
    pub fn request_reader(self: &Arc<Self>, wake: Waker) -> Option<ReaderProbeRequest> {
        if self.state.fetch_or(READER, Ordering::AcqRel) & READER != 0 {
            return None;
        }
        // Register before the request's first availability check. A release
        // before registration cannot be lost: READER prevents discovery from
        // taking the free allowance, and try_reserve observes that release.
        self.reader_wake.register(&wake);
        Some(ReaderProbeRequest {
            budget: Arc::clone(self),
        })
    }
}

/// One pending read-worker request, cancelled on drop.
#[derive(Debug)]
pub struct ReaderProbeRequest {
    budget: Arc<ProbeBudget>,
}

impl ReaderProbeRequest {
    /// Claims the next released conversion allowance, without blocking.
    #[must_use]
    pub fn try_reserve(&self) -> Option<ProbeReservation<'_>> {
        self.budget
            .state
            .compare_exchange(
                READER,
                READER | OCCUPIED,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .ok()
            .map(|_| ProbeReservation {
                budget: &self.budget,
            })
    }
}

impl Drop for ReaderProbeRequest {
    fn drop(&mut self) {
        // Release the old wake handle before another request can register one.
        drop(self.budget.reader_wake.take());
        let _ = self.budget.state.fetch_and(!READER, Ordering::Release);
    }
}

/// RAII charge. Declare before handles so handles drop before the charge.
#[derive(Debug)]
pub struct ProbeReservation<'a> {
    budget: &'a ProbeBudget,
}

impl Drop for ProbeReservation<'_> {
    fn drop(&mut self) {
        if self.budget.state.fetch_and(!OCCUPIED, Ordering::Release) & READER != 0 {
            self.budget.reader_wake.wake();
        }
    }
}
