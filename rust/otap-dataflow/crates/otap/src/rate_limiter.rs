// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver-local pressure-aware rate admission.

use otap_df_config::policy::{RateLimitMode, RateLimitPolicy, RateLimitPressure};
use otap_df_engine::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureLevel, SharedReceiverAdmissionState,
};
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
}

struct TokenBucket {
    allow: u64,
    interval_nanos: u64,
    burst: u64,
    epoch: Instant,
    theoretical_arrival_nanos: AtomicU64,
}

impl TokenBucket {
    fn new(policy: &RateLimitPolicy) -> Self {
        Self {
            allow: policy.allow,
            interval_nanos: u64::try_from(policy.interval.as_nanos()).unwrap_or(u64::MAX),
            burst: policy.burst_or_allow(),
            epoch: Instant::now(),
            theoretical_arrival_nanos: AtomicU64::new(0),
        }
    }

    fn now_nanos(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn nanos_for_units(&self, units: u64) -> u64 {
        if units == 0 {
            return 0;
        }
        if self.allow == 0 || self.interval_nanos == 0 {
            return u64::MAX;
        }

        let nanos =
            (u128::from(units) * u128::from(self.interval_nanos)).div_ceil(u128::from(self.allow));
        u64::try_from(nanos).unwrap_or(u64::MAX)
    }

    fn burst_window_nanos(&self) -> u64 {
        if self.burst == 0 || self.allow == 0 || self.interval_nanos == 0 {
            return 0;
        }

        self.nanos_for_units(self.burst)
    }

    fn charge(
        &self,
        weight: u64,
        pressure_active: bool,
        mode: RateLimitMode,
    ) -> RateAdmissionDecision {
        let cost = self.nanos_for_units(weight);
        let now = self.now_nanos();
        let burst_window = self.burst_window_nanos();
        let limit = now.saturating_add(burst_window);
        let debt_limit = limit.saturating_add(burst_window);

        loop {
            let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
            let candidate = current.max(now).saturating_add(cost);
            let over_limit = weight > self.burst || candidate > limit;

            if pressure_active && over_limit && mode == RateLimitMode::Enforce {
                return RateAdmissionDecision::Reject;
            }

            let next = candidate.min(debt_limit);
            if self
                .theoretical_arrival_nanos
                .compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                if pressure_active && over_limit {
                    return RateAdmissionDecision::WouldThrottle;
                }
                return RateAdmissionDecision::Admit;
            }
        }
    }

    fn is_exhausted(&self) -> bool {
        let now = self.now_nanos();
        let limit = now.saturating_add(self.burst_window_nanos());
        let current = self.theoretical_arrival_nanos.load(Ordering::Acquire);
        let candidate = current.max(now).saturating_add(self.nanos_for_units(1));
        candidate > limit
    }
}

impl std::fmt::Debug for TokenBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenBucket")
            .field("allow", &self.allow)
            .field("interval_nanos", &self.interval_nanos)
            .field("burst", &self.burst)
            .field(
                "theoretical_arrival_nanos",
                &self.theoretical_arrival_nanos.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
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
        self.level()
    }

    fn retry_after_secs(&self) -> u32 {
        self.retry_after_secs()
    }
}

impl AdmissionPressure for LocalReceiverAdmissionState {
    fn level(&self) -> MemoryPressureLevel {
        self.level()
    }

    fn retry_after_secs(&self) -> u32 {
        self.retry_after_secs()
    }
}

/// Receiver-instance rate gate.
#[derive(Clone, Debug)]
pub struct GenericRateLimiter<P> {
    policy: Arc<RateLimitPolicy>,
    admission_state: P,
    bucket: Arc<TokenBucket>,
}

/// Rate gate for receivers whose tasks may move between runtime workers.
pub type RateLimiter = GenericRateLimiter<SharedReceiverAdmissionState>;

/// Rate gate for receivers pinned to a local task set.
pub type LocalRateLimiter = GenericRateLimiter<LocalReceiverAdmissionState>;

impl<P: AdmissionPressure> GenericRateLimiter<P> {
    /// Creates a receiver-local limiter from the effective policy.
    #[must_use]
    pub fn new(policy: RateLimitPolicy, admission_state: P) -> Self {
        Self {
            bucket: Arc::new(TokenBucket::new(&policy)),
            policy: Arc::new(policy),
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
        self.bucket.charge(units, pressure_active, self.policy.mode)
    }

    /// Returns true when any positive-weight request would be rejected without charging the bucket.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        if !self.pressure_active() || self.policy.mode != RateLimitMode::Enforce {
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
    use otap_df_config::policy::{RateLimitAggregation, RateLimitPressure, RateLimitUnit};
    use otap_df_engine::memory_limiter::{MemoryPressureState, SharedReceiverAdmissionState};
    use std::time::Duration;

    fn policy(mode: RateLimitMode) -> RateLimitPolicy {
        RateLimitPolicy {
            mode,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::RequestBytesPerSecond,
            allow: 10,
            interval: Duration::from_secs(1),
            burst: Some(10),
            pressure: RateLimitPressure::Soft,
        }
    }

    /// Scenario: a scope exhausts its local byte bucket while memory pressure is normal.
    /// Guarantees: over-limit traffic is observed but not rejected before soft pressure.
    #[test]
    fn normal_pressure_charges_without_rejecting() {
        let state = MemoryPressureState::default();
        let limiter = RateLimiter::new(
            policy(RateLimitMode::Enforce),
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
        let limiter = RateLimiter::new(policy(RateLimitMode::Enforce), admission.clone());

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
        let limiter = RateLimiter::new(policy(RateLimitMode::ObserveOnly), admission.clone());

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
        let limiter = RateLimiter::new(policy(RateLimitMode::Enforce), admission.clone());

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
        let limiter = RateLimiter::new(policy(RateLimitMode::Enforce), admission.clone());

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
        let limiter = RateLimiter::new(policy(RateLimitMode::Enforce), admission);

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);

        assert!(!limiter.is_exhausted());
    }

    /// Scenario: traffic exceeds the burst budget while memory pressure is still normal.
    /// Guarantees: the bucket carries bounded debt into soft pressure instead of recovering from zero debt.
    #[test]
    fn normal_pressure_overage_accrues_debt_for_soft_pressure() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitMode::Enforce);
        policy.allow = 1_000;
        policy.burst = Some(1_000);
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
        let mut policy = policy(RateLimitMode::Enforce);
        policy.allow = 3;
        policy.burst = Some(10);
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
        let mut policy = policy(RateLimitMode::Enforce);
        policy.allow = u64::MAX;
        policy.burst = Some(1);
        let limiter = RateLimiter::new(policy, admission.clone());
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Admit);
        assert_eq!(limiter.check_units(1024), RateAdmissionDecision::Reject);
    }

    /// Scenario: a programmatic caller constructs a limiter with a zero refill interval.
    /// Guarantees: the defensive refill guard avoids invalid division and keeps admission checks stable.
    #[test]
    fn zero_interval_policy_does_not_break_refill() {
        let state = MemoryPressureState::default();
        let admission = SharedReceiverAdmissionState::from_process_state(&state);
        let mut policy = policy(RateLimitMode::Enforce);
        policy.interval = Duration::ZERO;
        let limiter = RateLimiter::new(policy, admission.clone());

        assert_eq!(limiter.check_units(20), RateAdmissionDecision::Admit);
        state.set_level_for_tests(MemoryPressureLevel::Soft);
        admission.apply(state.current_update(1));

        assert_eq!(limiter.check_units(1), RateAdmissionDecision::Reject);
    }
}
