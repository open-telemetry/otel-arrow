// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Internal clock utilities for engine control-plane timing.
//!
//! The engine needs monotonic time for shutdown deadlines, periodic timers,
//! processor-local resumes, and return-path duration metrics. Production code
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
//! `Instant::now()` / `tokio::time::sleep_until(...)`. Timeouts and intervals
//! are built on `sleep_until`, so they follow the same clock. The simulated
//! clock's waiter bookkeeping and locking are only used when tests explicitly
//! install a `SimClock`, so the expected production impact is negligible.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
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

/// Returns the current instant of the clock that drives sleeps.
///
/// System sleeps are tokio timers, so this reads tokio's clock. Without
/// tokio's `test-util` feature that is `Instant::now()`; with it, this follows
/// a paused tokio clock, so relative deadlines line up with
/// `tokio::time::advance` in tests that pause time.
fn timer_now() -> Instant {
    match current_clock() {
        ClockHandle::System => tokio::time::Instant::now().into_std(),
        #[cfg(any(test, feature = "test-utils"))]
        ClockHandle::Sim(clock) => clock.now(),
    }
}

/// Longest delay a relative deadline is set to, roughly 30 years. Matches
/// tokio's cap for `tokio::time::sleep`.
const MAX_DELAY: Duration = Duration::from_secs(86400 * 365 * 30);

/// Returns the deadline `duration` from now on the clock that drives sleeps.
///
/// `duration` is capped at [`MAX_DELAY`], so a huge duration (such as
/// `Duration::MAX`) gives a far-future deadline instead of overflowing
/// `Instant`.
fn deadline_after(duration: Duration) -> Instant {
    timer_now() + duration.min(MAX_DELAY)
}

/// Creates a sleep future for the active clock after a duration.
#[must_use]
pub fn sleep(duration: Duration) -> Sleep {
    sleep_until(deadline_after(duration))
}

/// Error returned by [`timeout_at`] and [`timeout`] when the deadline passes
/// before the wrapped future completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elapsed;

/// Runs `future` until it completes or the active clock reaches `deadline`.
///
/// The wrapped future is polled before the deadline, so a future that is ready
/// at the deadline still wins (same as `tokio::time::timeout_at`).
pub async fn timeout_at<F: Future>(deadline: Instant, future: F) -> Result<F::Output, Elapsed> {
    let mut future = std::pin::pin!(future);
    let mut sleep = sleep_until(deadline);
    std::future::poll_fn(|cx| {
        let had_budget_before = tokio::task::coop::has_budget_remaining();
        if let Poll::Ready(output) = future.as_mut().poll(cx) {
            return Poll::Ready(Ok(output));
        }
        poll_deadline(&mut sleep, had_budget_before, cx).map(|()| Err(Elapsed))
    })
    .await
}

/// Polls a timeout's deadline after its wrapped future returned `Pending`.
///
/// Tokio sleeps (and the `SimClock` waiters) spend tokio's cooperative
/// budget. If the wrapped future used up a budget it had before its poll, the
/// deadline is polled without the budget, as `tokio::time::timeout` does, so
/// a future that exhausts the budget on every poll can't block expiry.
/// Outside a tokio runtime the budget is always available and this is a plain
/// poll.
fn poll_deadline(sleep: &mut Sleep, had_budget_before: bool, cx: &mut Context<'_>) -> Poll<()> {
    if had_budget_before && !tokio::task::coop::has_budget_remaining() {
        Pin::new(&mut tokio::task::coop::unconstrained(sleep.as_mut())).poll(cx)
    } else {
        sleep.as_mut().poll(cx)
    }
}

/// Runs `future` until it completes or `duration` passes on the active clock.
///
/// The deadline is taken when this function is called, not when the returned
/// future is first polled.
pub fn timeout<F: Future>(
    duration: Duration,
    future: F,
) -> impl Future<Output = Result<F::Output, Elapsed>> {
    timeout_at(deadline_after(duration), future)
}

/// Lateness past which [`Interval`] skips missed ticks instead of firing the
/// next one on schedule. Matches tokio's interval.
const INTERVAL_SKIP_THRESHOLD: Duration = Duration::from_millis(5);

/// Periodic ticker on the active clock.
///
/// Missed ticks are skipped, like `tokio::time::MissedTickBehavior::Skip`: a
/// tick that fires late is followed by the next tick still aligned to the
/// original schedule.
pub struct Interval {
    next: Instant,
    period: Duration,
    sleep: Option<Sleep>,
}

/// Creates an [`Interval`] whose first tick completes one `period` from now.
///
/// # Panics
///
/// Panics if `period` is zero.
#[must_use]
pub fn interval(period: Duration) -> Interval {
    interval_at(deadline_after(period), period)
}

/// Creates an [`Interval`] whose first tick completes at `start`.
///
/// # Panics
///
/// Panics if `period` is zero.
#[must_use]
pub fn interval_at(start: Instant, period: Duration) -> Interval {
    assert!(period > Duration::ZERO, "`period` must be non-zero");
    Interval {
        next: start,
        period,
        sleep: None,
    }
}

impl Interval {
    /// Completes at the next tick and returns the instant it was scheduled for.
    ///
    /// Cancel safe: dropping the returned future before it completes keeps the
    /// pending tick for the next call.
    pub async fn tick(&mut self) -> Instant {
        let Self {
            next,
            period,
            sleep,
        } = self;
        sleep.get_or_insert_with(|| sleep_until(*next)).await;
        *sleep = None;

        let scheduled = *next;
        let now = timer_now();
        *next = if now > scheduled + INTERVAL_SKIP_THRESHOLD {
            let behind = (now - scheduled).as_nanos() % period.as_nanos();
            now + *period - Duration::from_nanos(behind as u64)
        } else {
            scheduled + *period
        };
        scheduled
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;

    /// Scenario: a future wrapped in `timeout_at` is still pending when the
    /// installed `SimClock` reaches the deadline, while a second wrapped future
    /// is already ready at its deadline.
    /// Guarantees: the timeout resolves to `Err(Elapsed)` only once simulated
    /// time reaches the deadline, and a ready future wins over an elapsed
    /// deadline.
    #[test]
    fn timeout_at_follows_sim_clock() {
        let clock = SimClock::new();
        let _guard = clock.install();
        let deadline = clock.now() + Duration::from_millis(10);

        let mut pending = Box::pin(timeout_at(deadline, std::future::pending::<()>()));
        assert!(pending.as_mut().now_or_never().is_none());
        clock.advance(Duration::from_millis(9));
        assert!(pending.as_mut().now_or_never().is_none());
        clock.advance(Duration::from_millis(1));
        assert_eq!(pending.now_or_never(), Some(Err(Elapsed)));

        let ready = timeout_at(deadline, std::future::ready(7));
        assert_eq!(ready.now_or_never(), Some(Ok(7)));
    }

    /// Scenario: `timeout` is created, simulated time moves past its duration,
    /// and only then is the returned future polled.
    /// Guarantees: the deadline is taken when `timeout` is called, so the first
    /// poll already reports `Err(Elapsed)`.
    #[test]
    fn timeout_deadline_starts_at_call() {
        let clock = SimClock::new();
        let _guard = clock.install();

        let fut = timeout(Duration::from_millis(10), std::future::pending::<()>());
        clock.advance(Duration::from_millis(10));
        assert_eq!(fut.now_or_never(), Some(Err(Elapsed)));
    }

    /// Scenario: `timeout`, `sleep` and `interval` are given `Duration::MAX`
    /// on the installed `SimClock`, then a day of simulated time passes.
    /// Guarantees: the deadline is capped far in the future instead of
    /// overflowing `Instant` (as tokio does), so nothing panics, a ready
    /// future still wins, and the timers stay pending.
    #[test]
    fn huge_durations_cap_deadline_on_sim_clock() {
        let clock = SimClock::new();
        let _guard = clock.install();

        let ready = timeout(Duration::MAX, std::future::ready(7));
        assert_eq!(ready.now_or_never(), Some(Ok(7)));

        let mut pending = Box::pin(timeout(Duration::MAX, std::future::pending::<()>()));
        let mut sleep = sleep(Duration::MAX);
        let mut interval = interval(Duration::MAX);
        clock.advance(Duration::from_secs(86_400));
        assert!(pending.as_mut().now_or_never().is_none());
        assert!(sleep.as_mut().now_or_never().is_none());
        assert!(interval.tick().now_or_never().is_none());
    }

    /// Scenario: `timeout`, `sleep` and `interval` are given `Duration::MAX`
    /// on the system clock, then tokio's paused clock is advanced a day.
    /// Guarantees: same as on the `SimClock`: no overflow panic, a ready
    /// future still wins, and the tokio timers accept the capped deadline and
    /// stay pending.
    #[tokio::test(start_paused = true)]
    async fn huge_durations_cap_deadline_on_tokio_clock() {
        let ready = timeout(Duration::MAX, std::future::ready(7));
        assert_eq!(ready.await, Ok(7));

        let mut pending = Box::pin(timeout(Duration::MAX, std::future::pending::<()>()));
        let mut sleep = sleep(Duration::MAX);
        let mut interval = interval(Duration::MAX);
        tokio::time::advance(Duration::from_secs(86_400)).await;
        assert!(pending.as_mut().now_or_never().is_none());
        assert!(sleep.as_mut().now_or_never().is_none());
        assert!(interval.tick().now_or_never().is_none());
    }

    /// Scenario: with tokio's clock paused and real time moving on, `timeout`,
    /// `sleep` and `interval` are created and tokio time is advanced by hand.
    /// Guarantees: relative deadlines on the system clock are measured on
    /// tokio's clock, so each fires exactly when tokio time reaches it, as
    /// with `tokio::time::timeout`, `tokio::time::sleep` and
    /// `tokio::time::interval`.
    #[tokio::test(start_paused = true)]
    async fn relative_deadlines_follow_paused_tokio_clock() {
        let mut timeout = Box::pin(timeout(
            Duration::from_secs(3),
            std::future::pending::<()>(),
        ));
        let mut sleep = sleep(Duration::from_secs(3));
        let mut interval = interval(Duration::from_secs(3));
        let start = tokio::time::Instant::now().into_std();

        // Real time drifts away from the paused tokio clock.
        std::thread::sleep(Duration::from_millis(20));

        tokio::time::advance(Duration::from_millis(2_999)).await;
        assert!(timeout.as_mut().now_or_never().is_none());
        assert!(sleep.as_mut().now_or_never().is_none());
        assert!(interval.tick().now_or_never().is_none());

        tokio::time::advance(Duration::from_millis(1)).await;
        assert_eq!(timeout.now_or_never(), Some(Err(Elapsed)));
        assert_eq!(sleep.now_or_never(), Some(()));
        assert_eq!(
            interval.tick().now_or_never(),
            Some(start + Duration::from_secs(3))
        );
    }

    /// Scenario: `timeout` wraps a future that spends the task's whole tokio
    /// cooperative budget on every poll and never completes.
    /// Guarantees: the timeout still expires, because the deadline is polled
    /// without the budget once the wrapped future has used it up (as
    /// `tokio::time::timeout` does).
    #[tokio::test]
    async fn timeout_expires_when_wrapped_future_exhausts_coop_budget() {
        let budget_hog = async {
            loop {
                tokio::task::coop::consume_budget().await;
            }
        };
        let guarded = tokio::time::timeout(
            Duration::from_secs(5),
            timeout(Duration::from_millis(10), budget_hog),
        )
        .await;
        assert_eq!(guarded, Ok(Err(Elapsed)), "clock::timeout never expired");
    }

    /// Scenario: an `Interval` with a 10ms period ticks on schedule, then is
    /// polled 25ms late; pending tick futures are dropped and polled again.
    /// Guarantees: each tick returns its scheduled instant; a late tick skips
    /// the missed ticks and stays aligned to the original schedule (like
    /// tokio's `MissedTickBehavior::Skip`); a dropped pending tick keeps its
    /// sleep, so repeated polls register one clock waiter, not one per poll.
    #[test]
    fn interval_skips_missed_ticks_on_sim_clock() {
        let clock = SimClock::new();
        let _guard = clock.install();
        let start = clock.now() + Duration::from_millis(10);
        let period = Duration::from_millis(10);
        let mut interval = interval_at(start, period);

        assert!(interval.tick().now_or_never().is_none());
        assert!(interval.tick().now_or_never().is_none());
        let waiters = clock
            .inner
            .state
            .lock()
            .expect("sim clock poisoned")
            .waiters
            .len();
        assert_eq!(waiters, 1);
        clock.advance_to(start);
        assert_eq!(interval.tick().now_or_never(), Some(start));

        // Late by 25ms: the tick scheduled at start+10 fires, the ones at
        // start+20 and start+30 are skipped, and the next lands on start+40.
        clock.advance_to(start + Duration::from_millis(35));
        assert_eq!(
            interval.tick().now_or_never(),
            Some(start + Duration::from_millis(10))
        );
        assert!(interval.tick().now_or_never().is_none());
        clock.advance_to(start + Duration::from_millis(40));
        assert_eq!(
            interval.tick().now_or_never(),
            Some(start + Duration::from_millis(40))
        );

        // A tick within the skip threshold keeps the plain schedule.
        clock.advance_to(start + Duration::from_millis(52));
        assert_eq!(
            interval.tick().now_or_never(),
            Some(start + Duration::from_millis(50))
        );
        clock.advance_to(start + Duration::from_millis(60));
        assert_eq!(
            interval.tick().now_or_never(),
            Some(start + Duration::from_millis(60))
        );
    }
}
