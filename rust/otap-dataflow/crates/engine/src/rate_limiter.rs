// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Generic token-bucket rate admission and pressure-aware receiver gating.

use crate::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureLevel, SharedReceiverAdmissionState,
};
use otap_df_config::policy::{RateLimitEnforcement, RateLimitPressure, RateLimiterPolicy};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Result of a scoped rate admission check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateAdmissionDecision {
    /// The request is admitted.
    Admit,
    /// The request would be throttled in enforce mode, but observe-only mode admits it.
    WouldThrottle,
    /// The request is rejected by the rate policy.
    Reject,
    /// The request exceeds the bucket's maximum burst and cannot succeed by retrying later.
    RejectOversized,
}

/// Allocation-free GCRA bucket that evaluates weighted admission independently
/// of any pressure or enforcement policy.
#[derive(Debug)]
pub struct GenericTokenBucket {
    allow: u64,
    interval_nanos: u64,
    burst: u64,
    burst_window_nanos: u64,
    epoch: Instant,
    theoretical_arrival_nanos: AtomicU64,
}

impl GenericTokenBucket {
    /// Creates a bucket from the configured rate and burst settings.
    #[must_use]
    pub fn new(policy: &RateLimiterPolicy) -> Self {
        let allow = policy.token_bucket.allow;
        let interval_nanos =
            u64::try_from(policy.token_bucket.interval.as_nanos()).unwrap_or(u64::MAX);
        let burst = policy.burst_or_allow();
        Self {
            allow,
            interval_nanos,
            burst,
            burst_window_nanos: if burst == 0 || allow == 0 || interval_nanos == 0 {
                0
            } else {
                Self::nanos_for_rate(allow, interval_nanos, burst)
            },
            epoch: Instant::now(),
            theoretical_arrival_nanos: AtomicU64::new(0),
        }
    }

    fn now_nanos(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_nanos()).unwrap_or(u64::MAX)
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

    fn next_theoretical_arrival(current: u64, now: u64, cost: u64, debt_limit: u64) -> u64 {
        current
            .max(now)
            .saturating_add(cost)
            .min(debt_limit)
            .max(current)
    }

    /// Evaluates weighted units, charging only admissions that are within limit.
    #[must_use]
    pub fn check_units(&self, weight: u64) -> RateBucketDecision {
        self.apply_units(weight, false)
    }

    /// Observes weighted units and records bounded debt even when over limit.
    ///
    /// This is useful for bypass and observe-only activation policies that need
    /// current traffic history if enforcement activates later.
    #[must_use]
    pub fn observe_units(&self, weight: u64) -> RateBucketDecision {
        self.apply_units(weight, true)
    }

    fn apply_units(&self, weight: u64, charge_over_limit: bool) -> RateBucketDecision {
        let cost = self.nanos_for_units(weight);

        loop {
            // Recompute all time-derived bounds after every failed CAS. Another
            // caller may have advanced the atomic beyond an earlier debt bound;
            // reusing that stale bound could move the bucket backwards.
            let now = self.now_nanos();
            let burst_window = self.burst_window_nanos;
            let limit = now.saturating_add(burst_window);
            let debt_limit = limit.saturating_add(burst_window);
            let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
            let candidate = current.max(now).saturating_add(cost);
            let oversized = weight > self.burst;
            let over_limit = oversized || candidate > limit;
            if over_limit && !charge_over_limit {
                return if oversized {
                    RateBucketDecision::Oversized
                } else {
                    RateBucketDecision::OverLimit
                };
            }
            let next = Self::next_theoretical_arrival(current, now, cost, debt_limit);
            if self
                .theoretical_arrival_nanos
                .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return if oversized {
                    RateBucketDecision::Oversized
                } else if over_limit {
                    RateBucketDecision::OverLimit
                } else {
                    RateBucketDecision::WithinLimit
                };
            }
        }
    }

    /// Returns true when a positive-weight admission would exceed the bucket.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        let now = self.now_nanos();
        let limit = now.saturating_add(self.burst_window_nanos);
        let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
        let candidate = current.max(now).saturating_add(self.nanos_for_units(1));
        candidate > limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Raw result of charging a generic rate bucket.
pub enum RateBucketDecision {
    /// The charge fits within the configured rate and burst window.
    WithinLimit,
    /// The bucket is over its current rate or burst window.
    OverLimit,
    /// The charge itself is larger than the configured burst capacity.
    Oversized,
}

/// Pressure state read by a receiver-local rate gate.
pub trait AdmissionPressure: Clone + std::fmt::Debug {
    /// Returns the current receiver ingress pressure level.
    fn level(&self) -> MemoryPressureLevel;

    /// Returns the receiver-facing retry hint.
    fn retry_after_secs(&self) -> u32;
}

impl AdmissionPressure for SharedReceiverAdmissionState {
    fn level(&self) -> MemoryPressureLevel {
        SharedReceiverAdmissionState::level(self)
    }

    fn retry_after_secs(&self) -> u32 {
        SharedReceiverAdmissionState::retry_after_secs(self)
    }
}

impl AdmissionPressure for LocalReceiverAdmissionState {
    fn level(&self) -> MemoryPressureLevel {
        LocalReceiverAdmissionState::level(self)
    }

    fn retry_after_secs(&self) -> u32 {
        LocalReceiverAdmissionState::retry_after_secs(self)
    }
}

/// Receiver-instance rate gate.
#[derive(Clone, Debug)]
pub struct GenericRateLimiter<P> {
    policy: RateLimiterPolicy,
    admission_state: P,
    bucket: Arc<GenericTokenBucket>,
}

/// Rate gate for receivers whose tasks may move between runtime workers.
pub type RateLimiter = GenericRateLimiter<SharedReceiverAdmissionState>;

/// Rate gate for receivers pinned to a local task set.
pub type LocalRateLimiter = GenericRateLimiter<LocalReceiverAdmissionState>;

impl<P: AdmissionPressure> GenericRateLimiter<P> {
    /// Creates a receiver-local limiter from the effective policy.
    #[must_use]
    pub fn new(policy: RateLimiterPolicy, admission_state: P) -> Self {
        Self {
            bucket: Arc::new(GenericTokenBucket::new(&policy)),
            policy,
            admission_state,
        }
    }

    fn pressure_active(&self) -> bool {
        let level = self.admission_state.level();
        match self.policy.pressure {
            RateLimitPressure::Soft => {
                matches!(level, MemoryPressureLevel::Soft | MemoryPressureLevel::Hard)
            }
        }
    }

    /// Applies a weighted admission check against the current pressure level.
    #[must_use]
    pub fn check_units(&self, units: u64) -> RateAdmissionDecision {
        let pressure_active = self.pressure_active();
        let enforce = pressure_active && self.policy.enforcement == RateLimitEnforcement::Enforce;
        let bucket_decision = if enforce {
            self.bucket.check_units(units)
        } else {
            self.bucket.observe_units(units)
        };
        match (bucket_decision, pressure_active) {
            (RateBucketDecision::WithinLimit, _) => RateAdmissionDecision::Admit,
            (RateBucketDecision::OverLimit | RateBucketDecision::Oversized, false) => {
                RateAdmissionDecision::Admit
            }
            (RateBucketDecision::OverLimit, true)
                if self.policy.enforcement == RateLimitEnforcement::ObserveOnly =>
            {
                RateAdmissionDecision::WouldThrottle
            }
            (RateBucketDecision::Oversized, true)
                if self.policy.enforcement == RateLimitEnforcement::ObserveOnly =>
            {
                RateAdmissionDecision::WouldThrottle
            }
            (RateBucketDecision::OverLimit, true) => RateAdmissionDecision::Reject,
            (RateBucketDecision::Oversized, true) => RateAdmissionDecision::RejectOversized,
        }
    }

    /// Returns true when any positive-weight request would be rejected without charging the bucket.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        if !self.pressure_active() || self.policy.enforcement != RateLimitEnforcement::Enforce {
            return false;
        }

        self.bucket.is_exhausted()
    }

    /// Returns the receiver-facing retry hint from the shared pressure state.
    #[must_use]
    pub fn retry_after_secs(&self) -> u32 {
        self.admission_state.retry_after_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_limiter::{MemoryPressureState, SharedReceiverAdmissionState};
    use otap_df_config::policy::{RateLimitAggregation, RateLimitPressure, RateLimitUnit};
    use std::time::Duration;

    fn policy(enforcement: RateLimitEnforcement) -> RateLimiterPolicy {
        RateLimiterPolicy {
            enforcement,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::RequestBytes,
            pressure: RateLimitPressure::Soft,
            token_bucket: otap_df_config::policy::TokenBucketPolicy {
                allow: 10,
                interval: Duration::from_secs(1),
                burst: Some(10),
            },
        }
    }

    /// Scenario: a competing admission advances the bucket beyond a caller's earlier debt bound.
    /// Guarantees: calculating the next timestamp can never move atomic bucket state backwards.
    #[test]
    fn stale_debt_bound_does_not_move_bucket_backwards() {
        let current = 1_000;
        let stale_debt_limit = 900;

        assert_eq!(
            GenericTokenBucket::next_theoretical_arrival(current, 100, 10, stale_debt_limit),
            current
        );
    }

    /// Scenario: an enforced admission is rejected after the bucket is full.
    /// Guarantees: ordinary rejection does not add debt to the generic bucket.
    #[test]
    fn rejected_check_does_not_advance_bucket() {
        let mut policy = policy(RateLimitEnforcement::Enforce);
        policy.token_bucket.allow = 1;
        policy.token_bucket.burst = Some(1);
        let bucket = GenericTokenBucket::new(&policy);

        assert_eq!(bucket.check_units(1), RateBucketDecision::WithinLimit);
        let before = bucket.theoretical_arrival_nanos.load(Ordering::Acquire);
        assert_eq!(bucket.check_units(1), RateBucketDecision::OverLimit);
        assert_eq!(
            bucket.theoretical_arrival_nanos.load(Ordering::Acquire),
            before
        );
    }

    /// Scenario: a scope exhausts its local byte bucket while memory pressure is normal.
    /// Guarantees: over-limit traffic is observed but not rejected before soft pressure.
    #[test]
    fn normal_pressure_charges_without_rejecting() {
        let state = MemoryPressureState::default();
        let limiter = RateLimiter::new(
            policy(RateLimitEnforcement::Enforce),
            SharedReceiverAdmissionState::from_process_state(&state),
        );

        assert_eq!(limiter.check_units(8), RateAdmissionDecision::Admit);
        assert_eq!(limiter.check_units(8), RateAdmissionDecision::Admit);
    }

    /// Scenario: a scope is already over its local byte bucket when soft pressure starts.
    /// Guarantees: enforce mode rejects additional over-limit traffic while pressure is active.
    #[test]
    fn soft_pressure_rejects_over_limit_in_enforce_mode() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(RateLimitEnforcement::Enforce), admission.clone());

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);
    }

    /// Scenario: a scope is over its local byte bucket with observe-only rate policy enabled.
    /// Guarantees: the limiter reports a would-throttle decision without rejecting the request.
    #[test]
    fn observe_only_reports_would_throttle() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter =
            RateLimiter::new(policy(RateLimitEnforcement::ObserveOnly), admission.clone());

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::WouldThrottle);
    }

    /// Scenario: a scope is over its local byte bucket when pressure returns to normal.
    /// Guarantees: pressure recovery stops enforced rate rejections even before the bucket refills.
    #[test]
    fn normal_pressure_recovers_from_enforced_rejection() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(RateLimitEnforcement::Enforce), admission.clone());

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));
        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);

        state.set_level_for_tests(MemoryPressureLevel::Normal);
        admission.apply(state.current_update(2));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Admit);
    }

    /// Scenario: a rate bucket is exhausted while enforce mode and soft pressure are active.
    /// Guarantees: the pre-decode peek reports exhaustion without charging the bucket.
    #[test]
    fn exhausted_peek_reports_without_charging() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(RateLimitEnforcement::Enforce), admission.clone());

        assert_eq!(limiter.check_units(10), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert!(limiter.is_exhausted());
        assert!(limiter.is_exhausted());
        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);
    }

    /// Scenario: a rate bucket is exhausted while memory pressure remains normal.
    /// Guarantees: the pre-decode peek stays disabled unless pressure would enforce throttling.
    #[test]
    fn exhausted_peek_ignores_normal_pressure() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let limiter = RateLimiter::new(policy(RateLimitEnforcement::Enforce), admission);

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);

        assert!(!limiter.is_exhausted());
    }

    /// Scenario: traffic exceeds the burst budget while memory pressure is still normal.
    /// Guarantees: the bucket carries bounded debt into soft pressure instead of recovering from zero debt.
    #[test]
    fn normal_pressure_overage_accrues_debt_for_soft_pressure() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitEnforcement::Enforce);
        policy.token_bucket.allow = 1_000;
        policy.token_bucket.burst = Some(1_000);
        let limiter = RateLimiter::new(policy, admission.clone());

        assert_eq!(limiter.check_units(2_000), RateAdmissionDecision::Admit);
        std::thread::sleep(Duration::from_millis(20));
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);
    }

    /// Scenario: the burst window is not evenly divisible by the configured rate.
    /// Guarantees: a request exactly equal to burst capacity is admitted from a full bucket.
    #[test]
    fn full_burst_request_uses_consistent_rounding() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitEnforcement::Enforce);
        policy.token_bucket.allow = 3;
        policy.token_bucket.burst = Some(10);
        let limiter = RateLimiter::new(policy, admission.clone());
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(10), RateAdmissionDecision::Admit);
    }

    /// Scenario: a very high configured rate maps different request sizes to the same GCRA tick.
    /// Guarantees: active pressure still rejects requests whose weight exceeds configured burst.
    #[test]
    fn high_rate_quantization_does_not_bypass_burst() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitEnforcement::Enforce);
        policy.token_bucket.allow = u64::MAX;
        policy.token_bucket.burst = Some(1);
        let limiter = RateLimiter::new(policy, admission.clone());
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Admit);
        assert_eq!(
            limiter.check_units(1024),
            RateAdmissionDecision::RejectOversized
        );
    }

    /// Scenario: a programmatic caller constructs a limiter with a zero refill interval.
    /// Guarantees: the defensive refill guard avoids invalid division and keeps admission checks stable.
    #[test]
    fn zero_interval_policy_does_not_break_refill() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitEnforcement::Enforce);
        policy.token_bucket.interval = Duration::ZERO;
        let limiter = RateLimiter::new(policy, admission.clone());

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);
    }
}
