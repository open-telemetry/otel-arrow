// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal clock utilities for engine control-plane timing.
//!
//! The engine needs monotonic time for shutdown deadlines, periodic timers,
//! delayed-data wakeups, and return-path duration metrics. Production code
//! should continue to use the system clock, but deterministic simulation tests
//! need to control time explicitly so they can replay the same interleavings
//! without depending on wall-clock progress.
//!
//! This module provides that seam without making the engine's public types
//! generic over a clock trait. The default behavior uses the system clock. In
//! tests, a `SimClock` can be installed as a thread-local override so the same
//! production control-plane code runs against deterministic time.
//!
//! The production path is intentionally small: clock reads go through a
//! thread-local lookup and a small enum branch, then immediately use
//! `Instant::now()` / `tokio::time::sleep_until(...)`. The simulated clock's
//! waiter bookkeeping and locking are only used when tests explicitly install a
//! `SimClock`, so the expected production impact is negligible.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Boxed sleep future used by the engine control plane.
pub type Sleep = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

thread_local! {
    static SYSTEM_BIRTH: Cell<Instant> = Cell::new(Instant::now());
    static CLOCK_OVERRIDE: RefCell<Option<ClockHandle>> = const { RefCell::new(None) };
}

#[derive(Clone)]
enum ClockHandle {
    System,
    #[cfg(any(test, feature = "test-utils"))]
    Sim(std::sync::Arc<SimClockInner>),
}

/// Selects the active clock for the current thread.
///
/// Production code normally sees `ClockHandle::System`. Tests install a
/// thread-local override so the same control-plane code can run against a
/// deterministic clock without changing public engine APIs.
fn current_clock() -> ClockHandle {
    CLOCK_OVERRIDE.with(|clock| clock.borrow().clone().unwrap_or(ClockHandle::System))
}

/// Returns the current monotonic instant from the active clock.
#[must_use]
pub fn now() -> Instant {
    match current_clock() {
        ClockHandle::System => Instant::now(),
        #[cfg(any(test, feature = "test-utils"))]
        ClockHandle::Sim(clock) => clock.now(),
    }
}

/// Returns a monotonic timestamp in nanoseconds since the active clock epoch.
///
/// `0` is reserved elsewhere in the engine as the sentinel for "timestamp not
/// set", so this function always returns a strictly positive value. The
/// `saturating_add(1)` is therefore intentional: it shifts the first valid
/// timestamp away from `0` while preserving monotonic ordering.
#[must_use]
pub fn nanos_since_birth() -> u64 {
    match current_clock() {
        ClockHandle::System => SYSTEM_BIRTH.with(|birth| {
            Instant::now()
                .duration_since(birth.get())
                .as_nanos()
                .saturating_add(1) as u64
        }),
        #[cfg(any(test, feature = "test-utils"))]
        ClockHandle::Sim(clock) => clock.nanos_since_birth(),
    }
}

/// Creates a sleep future for the active clock.
#[must_use]
pub fn sleep_until(deadline: Instant) -> Sleep {
    match current_clock() {
        ClockHandle::System => Box::pin(tokio::time::sleep_until(tokio::time::Instant::from_std(
            deadline,
        ))),
        #[cfg(any(test, feature = "test-utils"))]
        ClockHandle::Sim(clock) => clock.sleep_until(deadline),
    }
}

/// Creates a sleep future for the active clock after a duration.
#[must_use]
pub fn sleep(duration: Duration) -> Sleep {
    sleep_until(now() + duration)
}

#[cfg(any(test, feature = "test-utils"))]
/// Deterministic monotonic clock used by tests and DST scenarios.
///
/// A `SimClock` is installed on the current thread with [`SimClock::install`],
/// after which the engine's clock helpers use it instead of the system clock.
/// Tests can then advance time explicitly and wake due sleepers without waiting
/// for wall-clock time to pass.
#[derive(Clone, Debug)]
pub struct SimClock {
    inner: std::sync::Arc<SimClockInner>,
}

#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug)]
struct SimClockInner {
    state: std::sync::Mutex<SimClockState>,
}

#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug)]
struct SimClockState {
    birth: Instant,
    now: Instant,
    waiters: Vec<Waiter>,
}

#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug)]
struct Waiter {
    deadline: Instant,
    tx: tokio::sync::oneshot::Sender<()>,
}

#[cfg(any(test, feature = "test-utils"))]
impl SimClock {
    /// Creates a new simulated monotonic clock.
    #[must_use]
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            inner: std::sync::Arc::new(SimClockInner {
                state: std::sync::Mutex::new(SimClockState {
                    birth: now,
                    now,
                    waiters: Vec::new(),
                }),
            }),
        }
    }

    /// Returns the current simulated instant.
    #[must_use]
    pub fn now(&self) -> Instant {
        self.inner.now()
    }

    /// Advances simulated time by the provided duration.
    pub fn advance(&self, duration: Duration) {
        self.inner.advance(self.now() + duration);
    }

    /// Advances simulated time to the provided deadline if it is in the future.
    pub fn advance_to(&self, deadline: Instant) {
        self.inner.advance(deadline);
    }

    /// Installs this clock on the current thread until the returned guard is dropped.
    ///
    /// The override is thread-local on purpose: DST scenarios run the relevant
    /// runtime components on a single-threaded runtime and can therefore
    /// control time without affecting unrelated tests.
    #[must_use]
    pub fn install(&self) -> ClockOverrideGuard {
        let previous =
            CLOCK_OVERRIDE.with(|clock| clock.replace(Some(ClockHandle::Sim(self.inner.clone()))));
        ClockOverrideGuard { previous }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for SimClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl SimClockInner {
    fn now(&self) -> Instant {
        self.state.lock().expect("sim clock poisoned").now
    }

    fn nanos_since_birth(&self) -> u64 {
        let state = self.state.lock().expect("sim clock poisoned");
        state
            .now
            .duration_since(state.birth)
            .as_nanos()
            .saturating_add(1) as u64
    }

    fn sleep_until(self: std::sync::Arc<Self>, deadline: Instant) -> Sleep {
        let maybe_rx = {
            let mut state = self.state.lock().expect("sim clock poisoned");
            if deadline <= state.now {
                None
            } else {
                let (tx, rx) = tokio::sync::oneshot::channel();
                state.waiters.push(Waiter { deadline, tx });
                Some(rx)
            }
        };

        match maybe_rx {
            None => Box::pin(std::future::ready(())),
            Some(rx) => Box::pin(async move {
                let _ = rx.await;
            }),
        }
    }

    fn advance(&self, deadline: Instant) {
        let due = {
            let mut state = self.state.lock().expect("sim clock poisoned");
            if deadline <= state.now {
                return;
            }

            state.now = deadline;
            let now = state.now;
            let mut pending = Vec::new();
            let mut due = Vec::new();
            for waiter in state.waiters.drain(..) {
                if waiter.deadline <= now {
                    due.push(waiter.tx);
                } else {
                    pending.push(waiter);
                }
            }
            state.waiters = pending;
            due
        };

        for waiter in due {
            let _ = waiter.send(());
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub struct ClockOverrideGuard {
    previous: Option<ClockHandle>,
}

#[cfg(any(test, feature = "test-utils"))]
impl Drop for ClockOverrideGuard {
    fn drop(&mut self) {
        let previous = self.previous.take();
        CLOCK_OVERRIDE.with(|clock| {
            let _ = clock.replace(previous);
        });
    }
}
