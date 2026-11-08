// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use futures::task::AtomicWaker;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::task::{Wake, Waker};
use std::time::{Duration, Instant};

const DISCOVERY_HANDOFF_WAIT: Duration = Duration::from_millis(10);

struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

thread_local! {
    // Initialized on the worker that actually waits, not on the constructing thread.
    static DISCOVERY_WAKE: Waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
}

const OCCUPIED: u8 = 1;
const READER: u8 = 2;
const DISCOVERY: u8 = 4;
const READER_TURN: u8 = 8;

/// One conversion's two transient descriptors, shared by both workers.
/// Registered demand alternates at release. Unregistered admission checks are
/// opportunistic and cannot steal either worker's handoff. No I/O uses a lock.
#[derive(Debug, Default)]
pub struct ProbeBudget {
    state: AtomicU8,
    reader_wake: AtomicWaker,
    discovery_wake: AtomicWaker,
}

impl ProbeBudget {
    /// Nonblocking admission/observation check. Busy does not register demand.
    pub fn try_reserve(&self) -> Option<ProbeReservation<'_>> {
        self.try_unregistered().ok()
    }

    fn try_unregistered(&self) -> Result<ProbeReservation<'_>, u8> {
        self.state
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |s| {
                (s & (OCCUPIED | READER | DISCOVERY) == 0).then_some(s | OCCUPIED)
            })
            .map(|_| ProbeReservation {
                budget: self,
                owner: 0,
                owns_request: false,
            })
    }

    /// Registers the single read worker before its first availability check.
    pub fn request_reader(self: &Arc<Self>, wake: Waker) -> Option<ReaderProbeRequest> {
        if !self.register(READER, &wake) {
            return None;
        }
        Some(ReaderProbeRequest {
            budget: Arc::clone(self),
        })
    }

    fn register(&self, owner: u8, wake: &Waker) -> bool {
        if self.state.fetch_or(owner, Ordering::AcqRel) & owner != 0 {
            return false;
        }
        self.waker(owner).register(wake);
        // The caller checks availability after registration. Concurrent release
        // either wakes it or leaves a reserved turn visible to that check.
        true
    }

    fn waker(&self, owner: u8) -> &AtomicWaker {
        if owner == READER {
            &self.reader_wake
        } else {
            &self.discovery_wake
        }
    }

    fn acquire(&self, owner: u8) -> Option<ProbeReservation<'_>> {
        self.state
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |s| {
                let other = if owner == READER { DISCOVERY } else { READER };
                let preferred = if s & READER_TURN != 0 {
                    READER
                } else {
                    DISCOVERY
                };
                (s & OCCUPIED == 0 && s & owner != 0 && (s & other == 0 || preferred == owner))
                    .then_some(s | OCCUPIED)
            })
            .ok()
            .map(|_| ProbeReservation {
                budget: self,
                owner,
                owns_request: false,
            })
    }

    fn cancel(&self, owner: u8) {
        drop(self.waker(owner).take());
        let state = self.state.fetch_and(!owner, Ordering::AcqRel) & !owner;
        if state & OCCUPIED == 0 {
            self.wake_waiter(state, owner);
        }
    }

    fn wake_waiter(&self, state: u8, released_by: u8) {
        let preferred = if state & READER_TURN != 0 {
            READER
        } else {
            DISCOVERY
        };
        let other = if preferred == READER {
            DISCOVERY
        } else {
            READER
        };
        let next = if state & preferred != 0 {
            preferred
        } else {
            other
        };
        if next != released_by && state & next != 0 {
            self.waker(next).wake();
        }
    }

    /// Discovery-worker acquisition. A registered reader receives its bounded
    /// turn, then discovery owns the next release before another reader can win.
    /// Waits at most 10 ms for notification, then reports Busy so discovery can
    /// return incomplete coverage. Unregistered external pressure reports Busy
    /// immediately. Permit fairness within the wait is independent of time.
    pub(super) fn reserve_discovery(
        &self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Option<ProbeReservation<'_>> {
        let started = Instant::now();
        self.reserve_discovery_with_wait(cancelled, || {
            let remaining = DISCOVERY_HANDOFF_WAIT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return false;
            }
            std::thread::park_timeout(remaining);
            true
        })
    }

    fn reserve_discovery_with_wait(
        &self,
        cancelled: &mut impl FnMut() -> bool,
        mut wait: impl FnMut() -> bool,
    ) -> Option<ProbeReservation<'_>> {
        if cancelled() {
            return None;
        }
        let contended = match self.try_unregistered() {
            Ok(permit) => return Some(permit),
            Err(observed) => observed,
        };
        // Use the failed acquisition's state. A later load could miss a reader
        // releasing and immediately registering another request.
        if contended & READER == 0 {
            return None;
        }
        let wake = DISCOVERY_WAKE.with(Clone::clone);
        if !self.register(DISCOVERY, &wake) {
            return None;
        }
        // Guard cancellation/unwind; transfer its registration to the permit.
        struct Demand<'a>(&'a ProbeBudget, bool);
        impl Drop for Demand<'_> {
            fn drop(&mut self) {
                if self.1 {
                    self.0.cancel(DISCOVERY);
                }
            }
        }
        let mut demand = Demand(self, true);
        loop {
            if cancelled() {
                return None;
            }
            // AtomicWaker consumes a wake; refresh before checking availability.
            self.discovery_wake.register(&wake);
            if let Some(mut permit) = self.acquire(DISCOVERY) {
                demand.1 = false;
                permit.owns_request = true;
                return Some(permit);
            }
            if !wait() {
                return None;
            }
        }
    }
}

/// Single read-worker request; dropping it relinquishes pending priority.
#[derive(Debug)]
pub struct ReaderProbeRequest {
    budget: Arc<ProbeBudget>,
}
impl ReaderProbeRequest {
    /// Refresh the read worker's notification target before checking availability.
    pub fn register_waker(&self, wake: &Waker) {
        self.budget.reader_wake.register(wake);
    }

    /// Claims this worker's turn without blocking.
    #[must_use]
    pub fn try_reserve(&self) -> Option<ProbeReservation<'_>> {
        self.budget.acquire(READER)
    }
}
impl Drop for ReaderProbeRequest {
    fn drop(&mut self) {
        self.budget.cancel(READER);
    }
}

/// Conversion charge; close or transfer handles before dropping it.
#[derive(Debug)]
pub struct ProbeReservation<'a> {
    budget: &'a ProbeBudget,
    owner: u8,
    owns_request: bool,
}
impl Drop for ProbeReservation<'_> {
    fn drop(&mut self) {
        if self.owns_request {
            drop(self.budget.waker(self.owner).take());
        }
        let old = self
            .budget
            .state
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |s| {
                let s = if self.owns_request {
                    s & !self.owner
                } else {
                    s
                };
                Some(if self.owner == READER {
                    s & !(OCCUPIED | READER_TURN)
                } else {
                    (s & !OCCUPIED) | READER_TURN
                })
            })
            .expect("unconditional release");
        let state = if self.owns_request {
            old & !self.owner
        } else {
            old
        };
        let state = if self.owner == READER {
            state & !READER_TURN
        } else {
            state | READER_TURN
        };
        self.budget.wake_waiter(state, self.owner);
    }
}

#[cfg(test)]
impl ProbeBudget {
    pub(super) fn discovery_waiting(&self) -> bool {
        self.state.load(Ordering::Acquire) & DISCOVERY != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A reader keeps its permit while discovery receives spurious wakes until expiry.
    /// Guarantees: The bounded wait returns Busy and removes discovery demand without stealing the reader's turn.
    #[test]
    fn expired_handoff_clears_discovery_demand() {
        let budget = Arc::new(ProbeBudget::default());
        let reader = budget.request_reader(Waker::noop().clone()).unwrap();
        let held = reader.try_reserve().unwrap();
        let mut waits = 0;
        let result = budget.reserve_discovery_with_wait(&mut || false, || {
            assert!(budget.discovery_waiting());
            waits += 1;
            waits < 3
        });
        assert!(result.is_none());
        assert_eq!(waits, 3);
        assert!(!budget.discovery_waiting());
        drop(held);
        assert!(reader.try_reserve().is_some());
        drop(reader);
        assert!(budget.try_reserve().is_some());
    }

    /// Scenario: A reader releases the probe during discovery's notification wait.
    /// Guarantees: Discovery acquires its handoff before expiry and unregisters on permit release.
    #[test]
    fn release_during_handoff_wait_acquires_permit() {
        let budget = Arc::new(ProbeBudget::default());
        let reader = budget.request_reader(Waker::noop().clone()).unwrap();
        let mut held = Some(reader.try_reserve().unwrap());
        let permit = budget
            .reserve_discovery_with_wait(&mut || false, || {
                drop(held.take().expect("only one notification wait"));
                true
            })
            .unwrap();
        assert!(reader.try_reserve().is_none());
        drop(permit);
        assert!(!budget.discovery_waiting());
        assert!(reader.try_reserve().is_some());
    }
}
