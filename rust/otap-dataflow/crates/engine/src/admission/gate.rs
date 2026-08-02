// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pressure-aware rate gates produced by the engine-owned binder.
//!
//! A gate is the object a receiver actually calls on the ingress path. It owns
//! one [`RateBucket`], one concrete pressure handle, and the staging counters
//! for its node -- nothing else. `admit` is therefore a pressure read, at most
//! one atomic CAS loop, and (only when refusing) one relaxed increment.
//!
//! # Why the pressure handle is a type parameter
//!
//! `RateGate<P>` is monomorphised over the concrete pressure state, so reading
//! pressure compiles to a `Cell::get()` or a relaxed atomic load. Storing
//! pressure behind `dyn` would add virtual dispatch on the hottest path to
//! abstract over exactly two types.

use super::bucket::{BucketOutcome, RateBucket};
use super::clock::AdmissionClock;
use super::metrics::{AdmissionRefusal, RefusalCounters};
use super::{AdmissionContext, AdmissionDecision};
use crate::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureLevel, SharedReceiverAdmissionState,
};
use otap_df_config::policy::{RateLimitEnforcement, RateLimitPressure, RateLimiterPolicy};
use std::rc::Rc;
use std::sync::Arc;

/// Receiver ingress pressure, read through the concrete state type.
///
/// Both implementations call the inherent method explicitly
/// (`SharedReceiverAdmissionState::level(self)`), never `self.level()`: with a
/// trait method of the same name in scope, `self.level()` resolves back to the
/// trait and recurses until the stack overflows. Keeping the fully-qualified
/// form makes that mistake impossible to write by accident.
pub(crate) trait PressureSource {
    /// Current receiver ingress pressure level.
    fn level(&self) -> MemoryPressureLevel;
}

impl PressureSource for SharedReceiverAdmissionState {
    #[inline]
    fn level(&self) -> MemoryPressureLevel {
        SharedReceiverAdmissionState::level(self)
    }
}

impl PressureSource for LocalReceiverAdmissionState {
    #[inline]
    fn level(&self) -> MemoryPressureLevel {
        LocalReceiverAdmissionState::level(self)
    }
}

const NANOS_PER_SECOND: u64 = 1_000_000_000;

fn retry_after_secs(retry_after_nanos: u64) -> u32 {
    let seconds = retry_after_nanos.div_ceil(NANOS_PER_SECOND).max(1);
    u32::try_from(seconds).unwrap_or(u32::MAX)
}

/// A bound rate gate: one bucket, one pressure source, one node's counters.
#[derive(Debug)]
pub(crate) struct RateGate<P> {
    policy: RateLimiterPolicy,
    pressure: P,
    bucket: RateBucket,
    counters: Arc<RefusalCounters>,
}

impl<P: PressureSource> RateGate<P> {
    /// Builds a gate with a **fresh** bucket.
    ///
    /// A new bucket per bind is what implements `receiver_instance` aggregation:
    /// two receiver nodes sharing one limiter declaration each get their own
    /// capacity, so a noisy receiver cannot starve a quiet one. It is
    /// deliberately independent of `PipelineContext` cloning -- aggregation is
    /// a property of the resolved policy, not construction-context ownership.
    pub(crate) fn new(
        policy: RateLimiterPolicy,
        pressure: P,
        clock: AdmissionClock,
        counters: Arc<RefusalCounters>,
    ) -> Self {
        Self {
            bucket: RateBucket::with_clock(&policy, clock),
            policy,
            pressure,
            counters,
        }
    }

    /// Whether configured pressure conditions currently hold.
    #[inline]
    fn pressure_active(&self) -> bool {
        let level = P::level(&self.pressure);
        match self.policy.pressure {
            RateLimitPressure::Soft => {
                matches!(level, MemoryPressureLevel::Soft | MemoryPressureLevel::Hard)
            }
        }
    }

    /// Applies the policy to a weighted charge.
    ///
    /// When enforcement is inactive the bucket is still *observed*, so if
    /// pressure arrives later the limiter starts from real traffic history
    /// rather than a full bucket and cannot let a burst through on arrival.
    #[inline]
    fn decide(&self, units: u64) -> AdmissionDecision {
        let pressure_active = self.pressure_active();
        let enforce = pressure_active && self.policy.enforcement == RateLimitEnforcement::Enforce;
        let outcome = if enforce {
            self.bucket.check_units(units)
        } else {
            self.bucket.observe_units(units)
        };

        match outcome {
            BucketOutcome::WithinLimit => AdmissionDecision::Admit,
            _ if !pressure_active => AdmissionDecision::Admit,
            BucketOutcome::OverLimit { .. } | BucketOutcome::Oversized
                if self.policy.enforcement == RateLimitEnforcement::ObserveOnly =>
            {
                self.counters.record(AdmissionRefusal::WouldThrottle);
                AdmissionDecision::WouldThrottle
            }
            BucketOutcome::OverLimit { retry_after_nanos } => {
                self.counters.record(AdmissionRefusal::Throttle);
                AdmissionDecision::Throttle {
                    retry_after_secs: retry_after_secs(retry_after_nanos),
                }
            }
            BucketOutcome::Oversized => {
                self.counters.record(AdmissionRefusal::Oversized);
                AdmissionDecision::Oversized
            }
        }
    }

    /// Whether every positive-weight request would currently be refused.
    #[inline]
    fn saturated(&self) -> bool {
        if !self.pressure_active() || self.policy.enforcement != RateLimitEnforcement::Enforce {
            return false;
        }
        self.bucket.is_exhausted()
    }

    fn record_instance_saturation_refusal(&self) -> u32 {
        self.counters.record(AdmissionRefusal::Throttle);
        self.bucket
            .retry_after_nanos(1)
            .map(retry_after_secs)
            .unwrap_or(1)
    }
}

/// Cloneable thread-confined admission handle.
///
/// This type is deliberately `!Send`: clones share one bucket through `Rc`,
/// preserving the local receiver's `Cell`-based pressure path without atomic
/// reference counting merely for symmetry with the shared handle.
#[derive(Clone, Debug)]
pub struct LocalAdmissionGate {
    inner: Rc<RateGate<LocalReceiverAdmissionState>>,
}

impl LocalAdmissionGate {
    pub(crate) fn new(inner: RateGate<LocalReceiverAdmissionState>) -> Self {
        Self {
            inner: Rc::new(inner),
        }
    }

    /// Charges `units` in the bound dimension and returns the admission decision.
    ///
    /// V1 intentionally ignores `context`; it is the stable seam for later
    /// tenant- or scope-keyed bucket selection.
    #[inline]
    #[must_use]
    pub fn admit(&self, units: u64, _context: AdmissionContext<'_>) -> AdmissionDecision {
        self.inner.decide(units)
    }

    /// Returns whether the receiver-instance ceiling currently rejects minimum weight.
    #[inline]
    #[must_use]
    pub fn is_instance_saturated(&self) -> bool {
        self.inner.saturated()
    }
}

/// Cloneable `Send + Sync` admission handle.
///
/// Every clone shares the same receiver-instance bucket.
#[derive(Clone, Debug)]
pub struct SharedAdmissionGate {
    inner: Arc<RateGate<SharedReceiverAdmissionState>>,
}

impl SharedAdmissionGate {
    pub(crate) fn new(inner: RateGate<SharedReceiverAdmissionState>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Charges `units` in the bound dimension and returns the admission decision.
    ///
    /// V1 intentionally ignores `context`; it is the stable seam for later
    /// tenant- or scope-keyed bucket selection.
    #[inline]
    #[must_use]
    pub fn admit(&self, units: u64, _context: AdmissionContext<'_>) -> AdmissionDecision {
        self.inner.decide(units)
    }

    /// Returns whether the receiver-instance ceiling currently rejects minimum weight.
    #[inline]
    #[must_use]
    pub fn is_instance_saturated(&self) -> bool {
        self.inner.saturated()
    }

    /// Records and returns a fast refusal when the receiver-instance bucket is saturated.
    #[must_use]
    pub fn refuse_if_instance_saturated(&self) -> Option<u32> {
        self.inner
            .saturated()
            .then(|| self.inner.record_instance_saturation_refusal())
    }

    /// Records a refusal already selected by an earlier saturation probe.
    ///
    /// Tower's `poll_ready` has no request and must remain a read-only probe. Its
    /// subsequent `call` uses this method exactly once when it returns the fast
    /// rejection, so central admission telemetry counts the decision without
    /// charging the bucket or counting readiness polls.
    #[must_use]
    pub fn record_probed_instance_saturation_refusal(&self) -> u32 {
        self.inner.record_instance_saturation_refusal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::clock::ManualClock;
    use crate::memory_limiter::{MemoryPressureChanged, MemoryPressureState};
    use otap_df_config::policy::{RateLimitAggregation, RateLimitUnit, TokenBucketPolicy};
    use std::time::Duration;

    const RETRY_AFTER_SECS: u32 = 7;

    fn policy(enforcement: RateLimitEnforcement) -> RateLimiterPolicy {
        RateLimiterPolicy {
            enforcement,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::RequestBytes,
            pressure: RateLimitPressure::Soft,
            token_bucket: TokenBucketPolicy {
                allow: 10,
                interval: Duration::from_secs(1),
                burst: Some(10),
            },
        }
    }

    /// Test rig sharing one manual clock and one counter block with the gate.
    struct Harness<P> {
        gate: RateGate<P>,
        pressure: P,
        clock: Arc<ManualClock>,
        counters: Arc<RefusalCounters>,
    }

    impl<P: PressureSource + Clone> Harness<P> {
        fn new(enforcement: RateLimitEnforcement, pressure: P) -> Self {
            let clock = Arc::new(ManualClock::new(0));
            let counters = Arc::new(RefusalCounters::default());
            let gate = RateGate::new(
                policy(enforcement),
                pressure.clone(),
                AdmissionClock::Manual(Arc::clone(&clock)),
                Arc::clone(&counters),
            );
            Self {
                gate,
                pressure,
                clock,
                counters,
            }
        }
    }

    fn shared_harness(enforcement: RateLimitEnforcement) -> Harness<SharedReceiverAdmissionState> {
        Harness::new(
            enforcement,
            SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        )
    }

    fn local_harness(enforcement: RateLimitEnforcement) -> Harness<LocalReceiverAdmissionState> {
        Harness::new(
            enforcement,
            LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default()),
        )
    }

    fn raise_pressure<P: PressureSource + ApplyPressure>(pressure: &P, level: MemoryPressureLevel) {
        pressure.apply_change(MemoryPressureChanged {
            generation: 1,
            level,
            retry_after_secs: RETRY_AFTER_SECS,
            usage_bytes: 1,
        });
    }

    /// Uniform way to drive either pressure state from a test.
    trait ApplyPressure {
        fn apply_change(&self, change: MemoryPressureChanged);
    }

    impl ApplyPressure for SharedReceiverAdmissionState {
        fn apply_change(&self, change: MemoryPressureChanged) {
            self.apply(change);
        }
    }

    impl ApplyPressure for LocalReceiverAdmissionState {
        fn apply_change(&self, change: MemoryPressureChanged) {
            self.apply(change);
        }
    }

    fn admit_shared(
        gate: &RateGate<SharedReceiverAdmissionState>,
        units: u64,
    ) -> AdmissionDecision {
        gate.decide(units)
    }

    fn admit_local(gate: &RateGate<LocalReceiverAdmissionState>, units: u64) -> AdmissionDecision {
        gate.decide(units)
    }

    /// Scenario: traffic far exceeds the configured rate while pressure stays normal.
    /// Guarantees: nothing is throttled and the gate never advertises saturation, so an
    /// enabled limiter costs a healthy pipeline no admissions and no false rejections.
    #[test]
    fn over_limit_traffic_is_admitted_without_pressure() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);

        for _ in 0..100 {
            assert_eq!(admit_shared(&harness.gate, 1), AdmissionDecision::Admit);
        }

        assert!(!harness.gate.saturated());
        assert_eq!(harness.counters.drain(), [0, 0, 0]);
    }

    /// Scenario: over-limit traffic is observed at normal pressure, then pressure rises.
    /// Guarantees: enforcement resumes from the observed backlog instead of a full
    /// bucket, so a sustained flood cannot win a fresh burst the moment pressure hits.
    #[test]
    fn enforcement_starts_from_observed_history() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        for _ in 0..100 {
            let _ = admit_shared(&harness.gate, 1);
        }

        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);

        assert!(matches!(
            admit_shared(&harness.gate, 1),
            AdmissionDecision::Throttle { .. }
        ));
    }

    /// Scenario: capacity is exhausted under soft pressure, then time advances one unit.
    /// Guarantees: the refusal carries the bucket-derived recovery delay rather than the
    /// unrelated pressure hint and clears once that advertised capacity refills.
    #[test]
    fn throttling_reports_retry_hint_and_recovers() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);

        for _ in 0..10 {
            assert_eq!(admit_shared(&harness.gate, 1), AdmissionDecision::Admit);
        }

        let decision = admit_shared(&harness.gate, 1);
        assert_eq!(
            decision,
            AdmissionDecision::Throttle {
                retry_after_secs: 1
            }
        );
        assert!(harness.gate.saturated());

        harness.clock.advance(100_000_000);

        assert_eq!(admit_shared(&harness.gate, 1), AdmissionDecision::Admit);
    }

    /// Scenario: an over-limit request arrives under pressure in observe-only mode.
    /// Guarantees: it is admitted but reported as `WouldThrottle`, and the gate reports
    /// itself unsaturated so fast-reject layers cannot turn observation into refusal.
    #[test]
    fn observe_only_reports_without_refusing() {
        let harness = shared_harness(RateLimitEnforcement::ObserveOnly);
        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);

        for _ in 0..10 {
            assert_eq!(admit_shared(&harness.gate, 1), AdmissionDecision::Admit);
        }

        assert_eq!(
            admit_shared(&harness.gate, 1),
            AdmissionDecision::WouldThrottle
        );
        assert!(!harness.gate.saturated());
        assert_eq!(harness.counters.drain(), [1, 0, 0]);
    }

    /// Scenario: a request larger than total burst capacity arrives under pressure.
    /// Guarantees: the decision is `Oversized`, letting a receiver answer permanently
    /// instead of handing out a retry hint that could never succeed.
    #[test]
    fn oversized_request_is_distinguished_from_throttling() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);

        assert_eq!(
            admit_shared(&harness.gate, 11),
            AdmissionDecision::Oversized
        );
        assert_eq!(harness.counters.drain(), [0, 0, 1]);
    }

    /// Scenario: admissions and then refusals of each kind pass through the gate.
    /// Guarantees: only refusals are staged for telemetry, keeping the normal ingress
    /// path free of metric writes and their cache-line contention.
    #[test]
    fn only_refusals_are_staged_for_telemetry() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        for _ in 0..10 {
            let _ = admit_shared(&harness.gate, 1);
        }
        assert_eq!(harness.counters.drain(), [0, 0, 0]);

        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);
        let _ = admit_shared(&harness.gate, 1);
        let _ = admit_shared(&harness.gate, 11);

        assert_eq!(harness.counters.drain(), [0, 1, 1]);
    }

    /// Scenario: a transport fast-fails after a read-only saturation probe.
    /// Guarantees: recording the selected refusal increments the same central
    /// throttle counter exactly once and returns the minimum-weight bucket recovery hint.
    #[test]
    fn probed_saturation_refusal_is_staged_once() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);
        for _ in 0..10 {
            let _ = admit_shared(&harness.gate, 1);
        }

        assert!(harness.gate.saturated());
        assert_eq!(harness.gate.record_instance_saturation_refusal(), 1);
        assert_eq!(harness.counters.drain(), [0, 1, 0]);
    }

    /// Scenario: two gates are bound from the same policy, as two receiver nodes are.
    /// Guarantees: each bind owns its capacity, so `receiver_instance` aggregation keeps
    /// one busy receiver from consuming another receiver's allowance.
    #[test]
    fn each_bind_gets_independent_capacity() {
        let first = shared_harness(RateLimitEnforcement::Enforce);
        let second = shared_harness(RateLimitEnforcement::Enforce);
        raise_pressure(&first.pressure, MemoryPressureLevel::Soft);
        raise_pressure(&second.pressure, MemoryPressureLevel::Soft);

        for _ in 0..10 {
            let _ = admit_shared(&first.gate, 1);
        }
        assert!(matches!(
            admit_shared(&first.gate, 1),
            AdmissionDecision::Throttle { .. }
        ));

        assert_eq!(admit_shared(&second.gate, 1), AdmissionDecision::Admit);
    }

    /// Scenario: hard pressure is reported to a policy whose threshold is soft pressure.
    /// Guarantees: hard pressure also activates the limiter, so escalating pressure can
    /// never silently disable enforcement.
    #[test]
    fn hard_pressure_activates_a_soft_threshold_policy() {
        let harness = shared_harness(RateLimitEnforcement::Enforce);
        raise_pressure(&harness.pressure, MemoryPressureLevel::Hard);

        for _ in 0..10 {
            let _ = admit_shared(&harness.gate, 1);
        }

        assert!(matches!(
            admit_shared(&harness.gate, 1),
            AdmissionDecision::Throttle { .. }
        ));
    }

    /// Scenario: the same policy is exercised through the thread-confined gate impl.
    /// Guarantees: the local gate enforces, reports, and recovers identically to the
    /// shared one, so a receiver's execution model never changes admission semantics.
    #[test]
    fn local_gate_matches_shared_gate_semantics() {
        let harness = local_harness(RateLimitEnforcement::Enforce);

        for _ in 0..100 {
            assert_eq!(admit_local(&harness.gate, 1), AdmissionDecision::Admit);
        }

        raise_pressure(&harness.pressure, MemoryPressureLevel::Soft);

        assert_eq!(
            admit_local(&harness.gate, 1),
            AdmissionDecision::Throttle {
                retry_after_secs: 2
            }
        );
        assert!(harness.gate.saturated());
        assert_eq!(admit_local(&harness.gate, 11), AdmissionDecision::Oversized);

        harness.clock.advance(2_000_000_000);

        assert_eq!(admit_local(&harness.gate, 1), AdmissionDecision::Admit);
        assert_eq!(harness.counters.drain(), [0, 1, 1]);
    }
}
