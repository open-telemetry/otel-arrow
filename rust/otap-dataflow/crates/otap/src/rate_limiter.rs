// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver-local pressure-aware rate admission.

use otap_df_config::policy::{RateLimitMode, RateLimitPolicy};
use otap_df_engine::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureLevel, SharedReceiverAdmissionState,
};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

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

#[derive(Debug)]
struct TokenBucket {
    allow: f64,
    interval_secs: f64,
    burst: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(policy: &RateLimitPolicy) -> Self {
        let burst = policy.burst_or_allow() as f64;
        Self {
            allow: policy.allow as f64,
            interval_secs: policy.interval.as_secs_f64(),
            burst,
            tokens: burst,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.last_refill = now;
        if elapsed <= 0.0 || self.interval_secs <= 0.0 {
            return;
        }
        let refill = elapsed * (self.allow / self.interval_secs);
        self.tokens = (self.tokens + refill).min(self.burst);
    }

    fn charge(
        &mut self,
        weight: u64,
        pressure_active: bool,
        mode: RateLimitMode,
    ) -> RateAdmissionDecision {
        self.refill();
        let weight = weight as f64;
        let over_limit = self.tokens < weight;

        if pressure_active && over_limit {
            if mode == RateLimitMode::Enforce {
                return RateAdmissionDecision::Reject;
            }
            self.tokens = (self.tokens - weight).max(-self.burst);
            return RateAdmissionDecision::WouldThrottle;
        }

        self.tokens = (self.tokens - weight).max(-self.burst);
        RateAdmissionDecision::Admit
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
    bucket: Arc<Mutex<TokenBucket>>,
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
            bucket: Arc::new(Mutex::new(TokenBucket::new(&policy))),
            policy: Arc::new(policy),
            admission_state,
        }
    }

    /// Applies a weighted admission check against the current pressure level.
    #[must_use]
    pub fn check_units(&self, units: u64) -> RateAdmissionDecision {
        let pressure_active = matches!(
            self.admission_state.level(),
            MemoryPressureLevel::Soft | MemoryPressureLevel::Hard
        );
        self.bucket
            .lock()
            .charge(units, pressure_active, self.policy.mode)
    }

    /// Returns true when any positive-weight request would be rejected without charging the bucket.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        let pressure_active = matches!(
            self.admission_state.level(),
            MemoryPressureLevel::Soft | MemoryPressureLevel::Hard
        );
        if !pressure_active || self.policy.mode != RateLimitMode::Enforce {
            return false;
        }

        let mut bucket = self.bucket.lock();
        bucket.refill();
        bucket.tokens < 1.0
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
