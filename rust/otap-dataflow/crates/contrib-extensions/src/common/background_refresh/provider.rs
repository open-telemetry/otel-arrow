// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The generic background provider extension: `Arc<Inner>` state, and the
//! background refresh loop driven by the active `Extension::start()` task.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::{
    CapabilityError, CapabilityErrorSource, ExtensionCapability,
};
use otel_arrow_dfe_engine::control::ExtensionControlMsg;
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_engine::extension::EffectHandler;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use rand::RngExt;
use tokio::sync::watch::Receiver;
use tokio::sync::{MutexGuard, watch};

use super::metrics::{BackgroundProviderMetrics, BackgroundProviderMetricsTracker};

/// Floor between successful refreshes; avoids busy-looping on near-expired
/// values.
const MIN_REFRESH_INTERVAL_SECS: u64 = 10;
/// Base reschedule delay after a failed acquisition. Consecutive failures grow
/// this exponentially (with jitter) up to `MAX_REFRESH_RETRY_SECS`.
const REFRESH_RETRY_SECS: u64 = 10;
/// Upper bound on the retry backoff after repeated failures.
const MAX_REFRESH_RETRY_SECS: u64 = 300;
/// Maximum random jitter subtracted from a scheduled (successful) refresh
/// instant. Spreads the otherwise-aligned refresh ticks of many per-core
/// extensions so they do not hit the source on the same second. Only
/// ever moves the refresh earlier, never past the expiry safety buffer.
const REFRESH_JITTER_SECS: u64 = 60;
/// Next-refresh delay used for non-expiring values (~1 year). The loop is still
/// woken by control messages in the meantime.
const NON_EXPIRING_REFRESH_SECS: u64 = 365 * 24 * 60 * 60;

/// The provider-specific half of a background extension: how one value is
/// acquired, and how an acquisition failure is logged.
#[async_trait]
pub trait BackgroundProviderSource<T>: Send + Sync + 'static {
    /// Error returned by a failed acquisition.
    type Error: std::error::Error + Send + Sync + 'static;

    /// How close to expiration a value stops being usable.
    fn usable_margin() -> Duration;

    /// The monotonic instant at which the value expires, if known.
    fn expires_on(value: &T) -> Option<Instant>;

    /// Acquires a single value (no retries). Retry and scheduling policy belong
    /// to the refresh loop, not to the source.
    async fn fetch(&self) -> Result<T, Self::Error>;

    /// Emits the source's "refresh failed" internal log event. Owned by the
    /// source because `otel_warn!` requires a literal event name.
    fn log_refresh_failure(&self, error: &Self::Error);
}

/// Shared, clonable background provider extension.
///
/// Every clone (consumers + the background refresh task) observes the same
/// `Inner` state via `Arc`, so they share one value cache and refresh loop.
pub struct BackgroundProviderExtension<
    S: BackgroundProviderSource<T>,
    M: BackgroundProviderMetrics,
    T,
    C: ExtensionCapability,
> {
    inner: Arc<Inner<S, M, T, C>>,
}

// Manual `Clone`: deriving would add `S: Clone, M: Clone` bounds, but the state
// is shared through the `Arc` and is never cloned itself.
impl<S: BackgroundProviderSource<T>, M: BackgroundProviderMetrics, T, C: ExtensionCapability> Clone
    for BackgroundProviderExtension<S, M, T, C>
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Failure bookkeeping shared by the background refresh loop and the slow-path
/// `get_value`.
///
/// Keeping the streak counter here (rather than local to the refresh loop) is
/// what lets the negative cache widen in step with the loop's retry backoff:
/// otherwise a sustained outage would leave the loop retrying every 5 minutes
/// while cache-miss callers kept probing the source every 10 seconds.
#[derive(Default)]
struct FailureState {
    /// Instant of the most recent failed acquisition.
    last_failure: Option<Instant>,
    /// Number of consecutive failed acquisitions. Reset on any success.
    consecutive_failures: u32,
}

/// Shared state behind [`BackgroundProviderExtension`].
struct Inner<
    S: BackgroundProviderSource<T>,
    M: BackgroundProviderMetrics,
    T,
    C: ExtensionCapability,
> {
    /// Provider-specific value source.
    source: S,
    /// Refresh this far ahead of a value's expiry.
    expiry_buffer: Duration,
    /// Value cache + pub/sub for `value_stream()`.
    tx: watch::Sender<Option<T>>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<C>,
    /// Coalesces concurrent slow-path fetches onto one in-flight request.
    fetch_lock: tokio::sync::Mutex<()>,
    /// Negative cache + retry-backoff state. Used to throttle slow-path retries
    /// so a failing source is not stampeded.
    failures: Mutex<FailureState>,
    /// Metric tracker. Its critical sections are short and never span an
    /// `.await`, so a `std` `Mutex` is appropriate.
    metrics: Mutex<BackgroundProviderMetricsTracker<M>>,
}

impl<S: BackgroundProviderSource<T>, M: BackgroundProviderMetrics, T: Clone, C: ExtensionCapability>
    BackgroundProviderExtension<S, M, T, C>
{
    /// Builds a new extension instance.
    #[must_use]
    pub fn new(
        name: &str,
        source: S,
        expiry_buffer: Duration,
        tx: watch::Sender<Option<T>>,
        metrics: BackgroundProviderMetricsTracker<M>,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                source,
                expiry_buffer,
                tx,
                cap_err: CapabilityErrorSource::new(name.to_owned().into()),
                fetch_lock: tokio::sync::Mutex::new(()),
                failures: Mutex::new(FailureState::default()),
                metrics: Mutex::new(metrics),
            }),
        }
    }

    /// Acquire a fetch lock.
    pub async fn acquire_fetch_lock(&self) -> MutexGuard<'_, ()> {
        self.inner.fetch_lock.lock().await
    }

    /// Returns the cached value if it is present and still comfortably before
    /// its expiry (outside the usability safety margin).
    #[must_use]
    pub fn current_fresh_value(&self) -> Option<T> {
        // The value lives inside the watch channel behind a temporary read
        // guard; clone it out so we can return an owned value (and release the
        // guard, which would otherwise block the writer). `T` clones
        // are cheap: a refcount bump on the shared secret.
        let value = self.inner.tx.borrow().clone()?;
        match S::expires_on(&value) {
            Some(expires_on) => {
                if Instant::now() + S::usable_margin() < expires_on {
                    Some(value)
                } else {
                    None
                }
            }
            None => Some(value),
        }
    }

    /// Returns true if the most recent acquisition failed and the backoff for
    /// the current failure streak has not yet elapsed. Used as a negative cache
    /// to throttle slow-path retries.
    #[must_use]
    pub fn recently_failed(&self) -> bool {
        // Open the shared box holding the failure state. If the lock is somehow
        // poisoned, treat it as "no recent failure" and allow a retry rather
        // than failing here.
        let guard = match self.inner.failures.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        // If a failure timestamp is recorded, we are throttling only while it
        // is still within the cooldown window; otherwise (no failure recorded)
        // we are not throttling.
        match guard.last_failure {
            Some(failed_at) => {
                let window = negative_cache_window_secs(guard.consecutive_failures);
                failed_at.elapsed() < Duration::from_secs(window)
            }
            None => false,
        }
    }

    /// Number of consecutive failed acquisitions recorded so far.
    #[must_use]
    pub fn consecutive_failures(&self) -> u32 {
        // A poisoned lock degrades to "no failures", which only costs us a
        // shorter backoff; it must not take the refresh loop down.
        self.inner
            .failures
            .lock()
            .map(|f| f.consecutive_failures)
            .unwrap_or(0)
    }

    /// Subscribe to value publishing.
    #[must_use]
    pub fn subscribe(&self) -> Receiver<Option<T>> {
        self.inner.tx.subscribe()
    }

    /// Acquires a value and publishes it to consumers.
    pub async fn refresh_once(&self) -> Result<T, S::Error> {
        let start = Instant::now();
        match self.inner.source.fetch().await {
            Ok(value) => {
                let latency_ms = start.elapsed().as_secs_f64() * 1_000.0;
                // Publish the value to consumers and update the cache. Using
                // `send_replace` (rather than `send`) ensures the cache is
                // updated even when no receivers are currently subscribed.
                let _ = self.inner.tx.send_replace(Some(value.clone()));
                // Record success + publish under a single metrics lock.
                if let Ok(mut metrics) = self.inner.metrics.lock() {
                    metrics.record_success(latency_ms);
                    metrics.record_publish();
                }
                // Clear the negative cache: acquisitions are healthy again.
                if let Ok(mut failures) = self.inner.failures.lock() {
                    *failures = FailureState::default();
                }
                Ok(value)
            }
            Err(err) => {
                if let Ok(mut metrics) = self.inner.metrics.lock() {
                    metrics.record_failure();
                }
                // Record the failure so the refresh loop and the slow path back
                // off together: the instant starts the cooldown, the streak
                // count widens it on each further failure.
                if let Ok(mut failures) = self.inner.failures.lock() {
                    failures.last_failure = Some(Instant::now());
                    failures.consecutive_failures = failures.consecutive_failures.saturating_add(1);
                }
                Err(err)
            }
        }
    }

    /// Log a capability error.
    pub fn capability_error<E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(
        &self,
        error: E,
    ) -> CapabilityError {
        self.inner.cap_err.error(error)
    }
}

/// Computes the next refresh instant from a freshly acquired value.
///
/// Refreshes `expiry_buffer` before expiry, but never sooner than
/// `MIN_REFRESH_INTERVAL_SECS` from now; a non-expiring value pushes the
/// next refresh far into the future (the loop is still woken by control
/// messages in the meantime).
pub(crate) fn schedule_next(
    value_expires_on: Option<Instant>,
    expiry_buffer: Duration,
) -> tokio::time::Instant {
    let now = tokio::time::Instant::now();
    let min_next = now + Duration::from_secs(MIN_REFRESH_INTERVAL_SECS);
    match value_expires_on {
        Some(expires_on) => {
            let target = tokio::time::Instant::from_std(expires_on)
                .checked_sub(expiry_buffer)
                .unwrap_or(now);
            target.max(min_next)
        }
        None => now + Duration::from_secs(NON_EXPIRING_REFRESH_SECS),
    }
}

/// Base (un-jittered) backoff before retrying after a failed acquisition.
///
/// Grows exponentially with the number of consecutive prior failures, from
/// `REFRESH_RETRY_SECS` up to `MAX_REFRESH_RETRY_SECS`, so a
/// sustained source outage settles into infrequent retries instead of a
/// tight loop.
pub(crate) fn retry_backoff_secs(consecutive_failures: u32) -> u64 {
    // Cap the shift so `1 << shift` cannot overflow; the value is clamped to the
    // max below long before the shift approaches that bound.
    let shift = consecutive_failures.min(16);
    REFRESH_RETRY_SECS
        .saturating_mul(1u64 << shift)
        .min(MAX_REFRESH_RETRY_SECS)
}

/// Cooldown window during which the slow path refuses to retry, given the
/// number of consecutive failures recorded so far.
///
/// This is the same (un-jittered) delay the refresh loop is waiting out for the
/// same streak, so a sustained outage throttles both paths identically instead
/// of leaving cache-miss callers probing a source the loop has already
/// backed off from. `consecutive_failures` counts the failure that *started*
/// the current cooldown, so it is stepped back by one to line the first failure
/// up with the base delay.
///
/// The loop's own sleep is jittered down to as little as half this window, so
/// the loop always gets to retry before the slow path reopens.
pub(crate) fn negative_cache_window_secs(consecutive_failures: u32) -> u64 {
    retry_backoff_secs(consecutive_failures.saturating_sub(1))
}

/// Applies "equal jitter" to a backoff: half the delay is a fixed floor and the
/// other half is randomized, yielding a delay in `[base/2, base]`. This keeps
/// per-core extensions from retrying in lockstep during an outage.
fn jittered_backoff(base_secs: u64) -> Duration {
    let half = base_secs / 2;
    let jitter = if half == 0 {
        0
    } else {
        rand::rng().random_range(0..=half)
    };
    Duration::from_secs(half + jitter)
}

/// Subtracts random jitter (up to `REFRESH_JITTER_SECS`) from a scheduled
/// refresh instant so many per-core extensions do not refresh on the same tick.
///
/// Jitter only ever moves the refresh earlier (never later, which would risk
/// serving a value past its safety buffer) and never earlier than the
/// `MIN_REFRESH_INTERVAL_SECS` floor that `schedule_next` enforces -
/// otherwise a near-floor target could be pulled all the way to `now` and
/// busy-loop the refresh task while the value is still fresh.
pub(crate) fn jitter_refresh(target: tokio::time::Instant) -> tokio::time::Instant {
    let now = tokio::time::Instant::now();
    // Only jitter the slack *above* the minimum refresh interval, so the
    // earliest possible result is `now + MIN_REFRESH_INTERVAL_SECS`.
    let slack = target
        .saturating_duration_since(now)
        .as_secs()
        .saturating_sub(MIN_REFRESH_INTERVAL_SECS);
    let max_jitter = REFRESH_JITTER_SECS.min(slack);
    if max_jitter == 0 {
        return target;
    }
    let jitter = rand::rng().random_range(0..=max_jitter);
    target - Duration::from_secs(jitter)
}

#[async_trait]
impl<
    S: BackgroundProviderSource<T>,
    M: BackgroundProviderMetrics,
    T: Clone + Send + Sync,
    C: ExtensionCapability,
> SharedExtension for BackgroundProviderExtension<S, M, T, C>
{
    async fn start(
        self: Box<Self>,
        mut ctrl: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        let inner = Arc::clone(&self.inner);
        // Refresh immediately on startup.
        let mut next_refresh = tokio::time::Instant::now();
        // The engine holds data-path node startup until we signal readiness
        // (see `with_readiness_probe`). Fire once, after the first value is
        // published, so consumers never observe an empty cache.
        let mut ready_signaled = false;

        loop {
            tokio::select! {
                ctrl_msg = ctrl.recv() => {
                    match ctrl_msg {
                        // Graceful shutdown: return the final metric snapshot in
                        // the terminal state (the same contract nodes follow).
                        Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = inner.metrics.lock().ok().map(|m| m.snapshot());
                            return Ok(match snapshot {
                                Some(snapshot) => TerminalState::new(deadline, [snapshot]),
                                None => TerminalState::default(),
                            });
                        }
                        // Control channel closed: exit without a snapshot.
                        Err(_) => break,
                        // Refresh cadence is governed by value lifetime; live
                        // reconfiguration is a no-op in v1.
                        Ok(ExtensionControlMsg::Config { .. }) => {}
                        Ok(ExtensionControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Ok(mut metrics) = inner.metrics.lock() {
                                let _ = metrics.report(&mut metrics_reporter);
                            }
                        }
                    }
                }
                _ = tokio::time::sleep_until(next_refresh) => {
                    // The acquisition itself: take the same `fetch_lock` the
                    // slow-path `get_value` uses so a scheduled refresh and a
                    // concurrent cache-miss fetch coalesce onto one in-flight
                    // value call instead of both hitting the source.
                    let refresh = async {
                        // Note the current cache version before contending for
                        // the lock so we can tell whether another caller
                        // publishes a new value while we wait.
                        let mut rx = inner.tx.subscribe();
                        let _ = rx.borrow_and_update();
                        let _guard = inner.fetch_lock.lock().await;
                        // Only coalesce when a concurrent slow-path `get_value`
                        // actually published a fresh value while we waited for
                        // the lock. We must NOT skip merely because the cached
                        // value is still "usable": the loop refreshes ahead of
                        // expiry, but a value stays usable until ~30 s before
                        // expiry, so reusing it here would defer the planned
                        // early refresh far too long.
                        if rx.has_changed().unwrap_or(false) {
                            if let Some(value) = self.current_fresh_value() {
                                return Ok(value);
                            }
                        }
                        self.refresh_once().await
                    };
                    tokio::pin!(refresh);

                    // Keep the refresh cancellable: race it against the control
                    // channel so a slow value call cannot delay shutdown past its
                    // deadline. Config/telemetry messages are still serviced while
                    // the refresh is in flight; only shutdown or channel closure
                    // ends the loop (dropping the in-flight refresh future).
                    let outcome = loop {
                        tokio::select! {
                            outcome = &mut refresh => break outcome,
                            ctrl_msg = ctrl.recv() => {
                                match ctrl_msg {
                                    Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                                        let snapshot =
                                            inner.metrics.lock().ok().map(|m| m.snapshot());
                                        return Ok(match snapshot {
                                            Some(snapshot) => {
                                                TerminalState::new(deadline, [snapshot])
                                            }
                                            None => TerminalState::default(),
                                        });
                                    }
                                    Err(_) => return Ok(TerminalState::default()),
                                    Ok(ExtensionControlMsg::Config { .. }) => {}
                                    Ok(ExtensionControlMsg::CollectTelemetry {
                                        mut metrics_reporter,
                                    }) => {
                                        if let Ok(mut metrics) = inner.metrics.lock() {
                                            let _ = metrics.report(&mut metrics_reporter);
                                        }
                                    }
                                }
                            }
                        }
                    };

                    match outcome {
                        Ok(value) => {
                            next_refresh =
                                jitter_refresh(schedule_next(S::expires_on(&value), inner.expiry_buffer));
                            if !ready_signaled {
                                effect_handler.signal_ready();
                                ready_signaled = true;
                            }
                        }
                        Err(error) => {
                            inner.source.log_refresh_failure(&error);
                            // Bounded exponential backoff with jitter so many
                            // per-core extensions do not stampede the source
                            // on the same cadence during an outage.
                            // The streak counter lives in `Inner` (already
                            // incremented by `refresh_once`) so the slow-path
                            // negative cache widens on the same schedule.
                            let backoff = jittered_backoff(negative_cache_window_secs(
                                self.consecutive_failures(),
                            ));
                            next_refresh = tokio::time::Instant::now() + backoff;
                        }
                    }
                }
            }
        }

        Ok(TerminalState::default())
    }
}
