// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Monotonic clock used by admission buckets.
//!
//! # Why not `engine::clock`
//!
//! The engine's [`clock`](crate::clock) helpers are deliberately **not** used
//! here. `nanos_since_birth()` reads a thread-local epoch and honours a
//! thread-local test override, so two worker threads charging the same shared
//! bucket can produce timestamps that are not comparable -- and a GCRA bucket
//! whose timestamps are not comparable across its writers is not a rate limiter,
//! it is a race. An admission bucket therefore owns one epoch, captured when the
//! bucket is built, and every writer measures against that same epoch.
//!
//! # Determinism in tests
//!
//! Every time-dependent behaviour of the bucket (refill, burst window, bounded
//! debt, recovery) is exercised against [`ManualClock`], which is advanced
//! explicitly. No admission test sleeps: sleeping makes the assertions
//! timing-dependent precisely where the behaviour matters most.

use std::fmt::Debug;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// A monotonic, shared-epoch nanosecond clock.
///
/// "Shared epoch" is the load-bearing property: all readers of one clock
/// instance must measure from the same origin, so timestamps taken on different
/// threads are directly comparable.
pub trait MonotonicClock: Debug + Send + Sync {
    /// Nanoseconds elapsed since this clock's epoch.
    ///
    /// Must never go backwards for a given clock instance.
    fn now_nanos(&self) -> u64;
}

/// Production clock: one [`Instant`] epoch captured at construction.
///
/// `Instant` is monotonic on every supported platform, and holding the epoch by
/// value (rather than behind an `Arc`) keeps a reading to an `Instant::elapsed`
/// call with no pointer chase.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    epoch: Instant,
}

impl SystemClock {
    /// Captures the epoch for a new clock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock for SystemClock {
    #[inline]
    fn now_nanos(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
}

/// Deterministic clock for tests: time advances only when told to.
///
/// Shared between the test body and every thread charging the bucket, so
/// concurrency tests can pin time while many writers race on the same atomic.
#[derive(Debug, Default)]
#[cfg(test)]
pub struct ManualClock {
    nanos: AtomicU64,
}

#[cfg(test)]
impl ManualClock {
    /// Creates a clock reading `nanos`.
    #[must_use]
    pub fn new(nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(nanos),
        }
    }

    /// Moves the clock forward by `nanos`.
    pub fn advance(&self, nanos: u64) {
        let _ = self.nanos.fetch_add(nanos, Ordering::Relaxed);
    }
}

#[cfg(test)]
impl MonotonicClock for ManualClock {
    #[inline]
    fn now_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

/// The clock handle a bucket stores.
///
/// The production variant stores its epoch inline, avoiding a vtable call and
/// pointer chase. Tests add a shared manual variant so time-dependent behavior
/// remains deterministic without widening the production surface.
#[derive(Debug, Clone)]
pub enum AdmissionClock {
    /// Production: inline `Instant` epoch, no indirection.
    System(SystemClock),
    /// Tests: explicitly advanced, shared across threads.
    #[cfg(test)]
    Manual(Arc<ManualClock>),
}

impl AdmissionClock {
    /// Creates the production clock.
    #[must_use]
    pub fn system() -> Self {
        Self::System(SystemClock::new())
    }

    /// Reads the current time in nanoseconds since this clock's epoch.
    #[inline]
    #[must_use]
    pub fn now_nanos(&self) -> u64 {
        match self {
            Self::System(clock) => clock.now_nanos(),
            #[cfg(test)]
            Self::Manual(clock) => clock.now_nanos(),
        }
    }
}

impl Default for AdmissionClock {
    fn default() -> Self {
        Self::system()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a bucket test advances time explicitly instead of sleeping.
    /// Guarantees: the manual clock reports exactly the accumulated advance, so
    /// time-dependent admission behaviour is reproducible without wall-clock waits.
    #[test]
    fn manual_clock_advances_only_when_told() {
        let clock = ManualClock::new(0);

        assert_eq!(clock.now_nanos(), 0);
        clock.advance(1_500);
        assert_eq!(clock.now_nanos(), 1_500);
        clock.advance(500);
        assert_eq!(clock.now_nanos(), 2_000);
    }

    /// Scenario: several threads charge one bucket that shares a manual clock.
    /// Guarantees: every thread observes the same epoch and the same reading, which
    /// is the property thread-local engine clocks cannot provide.
    #[test]
    fn manual_clock_readings_are_shared_across_threads() {
        let clock = Arc::new(ManualClock::new(42));
        let observed: Vec<u64> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let clock = Arc::clone(&clock);
                    scope.spawn(move || clock.now_nanos())
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("clock reader thread"))
                .collect()
        });

        assert_eq!(observed, vec![42; 4]);
    }

    /// Scenario: the production clock is read twice in sequence.
    /// Guarantees: readings never move backwards, which the GCRA bucket relies on to
    /// keep its theoretical-arrival timestamp monotonic.
    #[test]
    fn system_clock_is_monotonic() {
        let clock = AdmissionClock::system();

        let first = clock.now_nanos();
        let second = clock.now_nanos();

        assert!(second >= first, "{second} < {first}");
    }
}
