// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free GCRA bucket backing engine-owned rate admission.
//!
//! The bucket answers exactly one question -- "does charging `units` fit inside
//! the configured rate and burst window?" -- and knows nothing about memory
//! pressure, enforcement mode, telemetry, or which receiver is asking. Those
//! belong to the gate that owns the bucket, which keeps this type small enough
//! to reason about and cheap enough to sit on the ingress path.
//!
//! # Representation
//!
//! State is a single `AtomicU64`: the *theoretical arrival time* (TAT) of the
//! next conforming request, in nanoseconds since the bucket's clock epoch. A
//! charge of `units` costs `units * interval / allow` nanoseconds. A request
//! conforms when its projected TAT stays within `now + burst_window`.
//!
//! One word of state means one contended cache line per bucket and no
//! allocation per decision, which is what lets the same type serve a shared
//! multi-threaded receiver and a single-threaded local one.

use super::clock::AdmissionClock;
use otap_df_config::policy::RateLimiterPolicy;
use std::sync::atomic::{AtomicU64, Ordering};

/// Raw result of charging the bucket, before any pressure or enforcement policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucketOutcome {
    /// The charge fits within the configured rate and burst window.
    WithinLimit,
    /// The bucket is over its current rate or burst window; retrying later can succeed.
    OverLimit {
        /// Earliest delay after which the same weighted charge can conform.
        retry_after_nanos: u64,
    },
    /// The charge exceeds total burst capacity; retrying later can never succeed.
    Oversized,
}

/// A GCRA rate bucket with bounded debt.
#[derive(Debug)]
pub struct RateBucket {
    allow: u64,
    interval_nanos: u64,
    burst: u64,
    burst_window_nanos: u64,
    clock: AdmissionClock,
    theoretical_arrival_nanos: AtomicU64,
}

impl RateBucket {
    /// Builds a bucket from a policy against an explicit clock.
    ///
    /// Tests pass a manual clock so refill, burst, and debt behaviour can be
    /// asserted without sleeping.
    #[must_use]
    pub fn with_clock(policy: &RateLimiterPolicy, clock: AdmissionClock) -> Self {
        let allow = policy.token_bucket.allow;
        let interval_nanos =
            u64::try_from(policy.token_bucket.interval.as_nanos()).unwrap_or(u64::MAX);
        let burst = policy.burst_or_allow();
        let burst_window_nanos = if burst == 0 || allow == 0 || interval_nanos == 0 {
            0
        } else {
            Self::nanos_for_rate(allow, interval_nanos, burst)
        };
        let start = clock.now_nanos();

        Self {
            allow,
            interval_nanos,
            burst,
            burst_window_nanos,
            clock,
            theoretical_arrival_nanos: AtomicU64::new(start),
        }
    }

    fn nanos_for_units(&self, units: u64) -> u64 {
        Self::nanos_for_rate(self.allow, self.interval_nanos, units)
    }

    fn nanos_for_rate(allow: u64, interval_nanos: u64, units: u64) -> u64 {
        if units == 0 {
            return 0;
        }
        if allow == 0 || interval_nanos == 0 {
            return u64::MAX;
        }

        let nanos = (u128::from(units) * u128::from(interval_nanos)).div_ceil(u128::from(allow));
        u64::try_from(nanos).unwrap_or(u64::MAX)
    }

    /// Computes the TAT to publish, clamped so the bucket can never move backwards.
    ///
    /// `debt_limit` caps how far past `now` an over-limit charge may push the
    /// bucket, so a sustained flood cannot build unbounded debt that would
    /// starve traffic long after the flood stops. The trailing `.max(current)`
    /// is the safety net: another thread may already have published a TAT beyond
    /// a `debt_limit` this caller computed from an older `now`, and regressing
    /// it would hand out capacity that was already spent.
    fn next_theoretical_arrival(current: u64, now: u64, cost: u64, debt_limit: u64) -> u64 {
        current
            .max(now)
            .saturating_add(cost)
            .min(debt_limit)
            .max(current)
    }

    /// Charges `units`, publishing state only when the charge conforms.
    ///
    /// Used under active enforcement: a rejected request must not consume
    /// capacity, otherwise callers that are already being refused would keep
    /// pushing the recovery point further out.
    #[must_use]
    pub fn check_units(&self, units: u64) -> BucketOutcome {
        self.apply_units(units, false)
    }

    /// Charges `units` unconditionally, accruing bounded debt when over limit.
    ///
    /// Used when enforcement is inactive (no pressure, or observe-only) so the
    /// bucket carries real traffic history and enforcement, if it activates,
    /// starts from an accurate position rather than a full bucket.
    #[must_use]
    pub fn observe_units(&self, units: u64) -> BucketOutcome {
        self.apply_units(units, true)
    }

    fn apply_units(&self, units: u64, charge_over_limit: bool) -> BucketOutcome {
        if units == 0 {
            return BucketOutcome::WithinLimit;
        }

        let cost = self.nanos_for_units(units);
        let oversized = units > self.burst;

        loop {
            // Recompute every time-derived bound after a failed CAS. Another
            // caller may have advanced the atomic past a bound derived from an
            // older `now`; reusing that stale bound could move the bucket
            // backwards and leak capacity.
            let now = self.clock.now_nanos();
            let burst_window = self.burst_window_nanos;
            let limit = now.saturating_add(burst_window);
            let debt_limit = limit.saturating_add(burst_window);
            let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
            let candidate = current.max(now).saturating_add(cost);
            let over_limit = oversized || candidate > limit;

            let retry_after_nanos = candidate.saturating_sub(limit);
            if over_limit && !charge_over_limit {
                return if oversized {
                    BucketOutcome::Oversized
                } else {
                    BucketOutcome::OverLimit { retry_after_nanos }
                };
            }

            let next = Self::next_theoretical_arrival(current, now, cost, debt_limit);
            if self
                .theoretical_arrival_nanos
                .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return if oversized {
                    BucketOutcome::Oversized
                } else if over_limit {
                    BucketOutcome::OverLimit { retry_after_nanos }
                } else {
                    BucketOutcome::WithinLimit
                };
            }
        }
    }

    /// Reports whether even a single-unit charge would currently be refused.
    ///
    /// Read-only: it never publishes state, so a cheap pre-filter cannot consume
    /// capacity that the authoritative decision later needs.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.retry_after_nanos(1).is_some()
    }

    /// Returns the earliest delay after which `units` can conform.
    ///
    /// This is read-only and uses the same GCRA boundary as [`check_units`](Self::check_units),
    /// so protocol retry guidance describes bucket recovery rather than the
    /// memory-pressure sampling interval.
    #[must_use]
    pub fn retry_after_nanos(&self, units: u64) -> Option<u64> {
        if units == 0 {
            return None;
        }
        if units > self.burst {
            return None;
        }
        let now = self.clock.now_nanos();
        let limit = now.saturating_add(self.burst_window_nanos);
        let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
        let candidate = current.max(now).saturating_add(self.nanos_for_units(units));
        (candidate > limit).then(|| candidate.saturating_sub(limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::clock::ManualClock;
    use otap_df_config::policy::{
        RateLimitAggregation, RateLimitEnforcement, RateLimitPressure, RateLimitUnit,
        TokenBucketPolicy,
    };
    use std::sync::Arc;
    use std::time::Duration;

    fn policy(allow: u64, burst: Option<u64>) -> RateLimiterPolicy {
        RateLimiterPolicy {
            enforcement: RateLimitEnforcement::Enforce,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::RequestBytes,
            pressure: RateLimitPressure::Soft,
            token_bucket: TokenBucketPolicy {
                allow,
                interval: Duration::from_secs(1),
                burst,
            },
        }
    }

    fn manual_bucket(allow: u64, burst: Option<u64>) -> (RateBucket, Arc<ManualClock>) {
        let clock = Arc::new(ManualClock::new(0));
        let bucket = RateBucket::with_clock(
            &policy(allow, burst),
            AdmissionClock::Manual(Arc::clone(&clock)),
        );
        (bucket, clock)
    }

    /// Scenario: charges are issued up to the configured burst without advancing time.
    /// Guarantees: the bucket admits exactly its burst capacity and refuses the next
    /// unit, so burst is a hard ceiling rather than an approximate one.
    #[test]
    fn admits_exactly_burst_capacity_before_refusing() {
        let (bucket, _clock) = manual_bucket(10, Some(10));

        for index in 0..10 {
            assert_eq!(
                bucket.check_units(1),
                BucketOutcome::WithinLimit,
                "charge {index} should fit in burst"
            );
        }

        assert!(matches!(
            bucket.check_units(1),
            BucketOutcome::OverLimit { .. }
        ));
    }

    /// Scenario: an exhausted bucket is retried after enough time to refill one unit.
    /// Guarantees: capacity returns at the configured rate, so throttling is temporary
    /// and recovery does not require the full burst window to elapse.
    #[test]
    fn refills_at_the_configured_rate() {
        let (bucket, clock) = manual_bucket(10, Some(10));
        for _ in 0..10 {
            let _ = bucket.check_units(1);
        }
        assert_eq!(
            bucket.check_units(1),
            BucketOutcome::OverLimit {
                retry_after_nanos: 100_000_000
            }
        );

        // 10 units per second means one unit costs 100ms.
        clock.advance(100_000_000);

        assert_eq!(bucket.check_units(1), BucketOutcome::WithinLimit);
        assert!(matches!(
            bucket.check_units(1),
            BucketOutcome::OverLimit { .. }
        ));
    }

    /// Scenario: a weighted request is refused after smaller requests consume the burst.
    /// Guarantees: the retry delay is calculated for that request's full weight, so the
    /// advertised recovery point is neither the one-unit delay nor a pressure poll hint.
    #[test]
    fn weighted_refusal_reports_its_earliest_conforming_delay() {
        let (bucket, clock) = manual_bucket(10, Some(10));
        for _ in 0..8 {
            assert_eq!(bucket.check_units(1), BucketOutcome::WithinLimit);
        }

        assert_eq!(
            bucket.check_units(5),
            BucketOutcome::OverLimit {
                retry_after_nanos: 300_000_000
            }
        );
        assert_eq!(bucket.retry_after_nanos(5), Some(300_000_000));

        clock.advance(299_999_999);
        assert!(matches!(
            bucket.check_units(5),
            BucketOutcome::OverLimit { .. }
        ));
        clock.advance(1);
        assert_eq!(bucket.check_units(5), BucketOutcome::WithinLimit);
    }

    /// Scenario: a single charge is larger than the bucket's entire burst capacity.
    /// Guarantees: the outcome is `Oversized` rather than `OverLimit`, letting callers
    /// tell "retry later" apart from "this request can never succeed".
    #[test]
    fn charge_larger_than_burst_is_reported_as_oversized() {
        let (bucket, _clock) = manual_bucket(10, Some(10));

        assert_eq!(bucket.check_units(11), BucketOutcome::Oversized);
    }

    /// Scenario: an over-limit charge is refused while enforcement is active.
    /// Guarantees: a refused charge publishes no state, so a client being throttled
    /// cannot push its own recovery point further away by retrying.
    #[test]
    fn refused_charge_does_not_consume_capacity() {
        let (bucket, clock) = manual_bucket(10, Some(10));
        for _ in 0..10 {
            let _ = bucket.check_units(1);
        }

        for _ in 0..50 {
            assert!(matches!(
                bucket.check_units(1),
                BucketOutcome::OverLimit { .. }
            ));
        }
        clock.advance(100_000_000);

        assert_eq!(
            bucket.check_units(1),
            BucketOutcome::WithinLimit,
            "50 refusals must not have delayed recovery"
        );
    }

    /// Scenario: observe-only traffic keeps charging a bucket that is already over limit.
    /// Guarantees: accrued debt is capped at one extra burst window, so switching to
    /// enforcement after a long flood does not block traffic for an unbounded time.
    #[test]
    fn observed_debt_is_bounded_by_one_extra_burst_window() {
        let (bucket, clock) = manual_bucket(10, Some(10));

        for _ in 0..10_000 {
            let _ = bucket.observe_units(1);
        }

        // Burst window is 1s; debt is capped at now + 2 * burst window.
        clock.advance(2_000_000_000);
        assert_eq!(bucket.check_units(1), BucketOutcome::WithinLimit);
    }

    /// Scenario: normal-pressure observation reaches the two-window debt ceiling before
    /// active enforcement begins for a one-unit bucket.
    /// Guarantees: the maximum-weight charge remains refused immediately before two burst
    /// windows elapse and conforms exactly at that worst-case recovery boundary.
    #[test]
    fn maximum_observed_debt_recovers_at_two_burst_windows() {
        let (bucket, clock) = manual_bucket(1, Some(1));

        for _ in 0..10 {
            let _ = bucket.observe_units(1);
        }

        assert_eq!(
            bucket.check_units(1),
            BucketOutcome::OverLimit {
                retry_after_nanos: 2_000_000_000
            }
        );

        clock.advance(1_999_999_999);
        assert_eq!(
            bucket.check_units(1),
            BucketOutcome::OverLimit {
                retry_after_nanos: 1
            }
        );

        clock.advance(1);
        assert_eq!(bucket.check_units(1), BucketOutcome::WithinLimit);
    }

    /// Scenario: `next_theoretical_arrival` is handed a debt limit below the published TAT.
    /// Guarantees: the returned timestamp never regresses, which is what stops a losing
    /// CAS racer from handing back capacity another thread already spent.
    #[test]
    fn stale_debt_bound_never_moves_the_bucket_backwards() {
        let current = 1_000;
        let stale_debt_limit = 900;

        assert_eq!(
            RateBucket::next_theoretical_arrival(current, 100, 10, stale_debt_limit),
            current
        );
    }

    /// Scenario: `is_exhausted` is polled on a saturated and then recovered bucket.
    /// Guarantees: the probe reflects saturation without charging the bucket, so cheap
    /// pre-filters cannot consume capacity the authoritative decision needs.
    #[test]
    fn exhaustion_probe_is_read_only() {
        let (bucket, clock) = manual_bucket(10, Some(10));
        assert!(!bucket.is_exhausted());

        for _ in 0..10 {
            let _ = bucket.check_units(1);
        }
        assert!(bucket.is_exhausted());
        assert!(bucket.is_exhausted(), "probing must not change state");

        clock.advance(100_000_000);
        assert!(!bucket.is_exhausted());
        assert_eq!(
            bucket.check_units(1),
            BucketOutcome::WithinLimit,
            "probes must not have consumed the refilled unit"
        );
    }

    /// Scenario: a zero-unit charge is submitted after observation accrued the maximum debt.
    /// Guarantees: zero-cost charges always conform and have no retry delay, even when the
    /// bucket's published TAT is beyond the active enforcement boundary.
    #[test]
    fn zero_unit_charge_always_conforms() {
        let (bucket, _clock) = manual_bucket(10, Some(10));
        for _ in 0..20 {
            let _ = bucket.observe_units(1);
        }

        assert_eq!(bucket.check_units(0), BucketOutcome::WithinLimit);
        assert_eq!(bucket.observe_units(0), BucketOutcome::WithinLimit);
        assert_eq!(bucket.retry_after_nanos(0), None);
    }

    /// Scenario: many threads charge one bucket concurrently with time frozen.
    /// Guarantees: the CAS loop hands out no more than the burst capacity in total,
    /// so concurrency cannot inflate the effective limit.
    #[test]
    fn concurrent_charges_never_exceed_burst_capacity() {
        let clock = Arc::new(ManualClock::new(0));
        let bucket = Arc::new(RateBucket::with_clock(
            &policy(64, Some(64)),
            AdmissionClock::Manual(Arc::clone(&clock)),
        ));

        let admitted: usize = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let bucket = Arc::clone(&bucket);
                    scope.spawn(move || {
                        (0..100)
                            .filter(|_| bucket.check_units(1) == BucketOutcome::WithinLimit)
                            .count()
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("charging thread"))
                .sum()
        });

        assert_eq!(
            admitted, 64,
            "burst capacity must be exact under contention"
        );
    }

    /// Scenario: a policy configures a zero allowance.
    /// Guarantees: every positive charge is refused instead of dividing by zero, so a
    /// misconfigured limiter fails closed rather than panicking. `burst_or_allow`
    /// floors burst at 1, so the refusal is the retryable `OverLimit` rather than
    /// `Oversized`.
    #[test]
    fn zero_allowance_refuses_all_positive_charges() {
        let (bucket, _clock) = manual_bucket(0, Some(0));

        assert!(matches!(
            bucket.check_units(1),
            BucketOutcome::OverLimit { .. }
        ));
        assert_eq!(bucket.check_units(0), BucketOutcome::WithinLimit);
    }

    /// Scenario: a policy sets burst above allow, then charges the extra headroom.
    /// Guarantees: burst is independent of the steady-state rate, so short spikes above
    /// `allow` are admitted while the sustained rate stays bounded.
    #[test]
    fn burst_headroom_exceeds_steady_state_allowance() {
        let (bucket, _clock) = manual_bucket(10, Some(25));

        for index in 0..25 {
            assert_eq!(
                bucket.check_units(1),
                BucketOutcome::WithinLimit,
                "charge {index} should fit in the larger burst"
            );
        }
        assert!(matches!(
            bucket.check_units(1),
            BucketOutcome::OverLimit { .. }
        ));
    }
}
