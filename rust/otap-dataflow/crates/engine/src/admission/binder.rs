// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Construction-time binding for engine-owned ingress admission.

use super::clock::AdmissionClock;
use super::gate::RateGate;
use super::metrics::{AdmissionMetricsHandle, RefusalCounters};
use super::{
    AdmissionBindError, AdmissionDimension, AdmissionDimensionSet, LocalAdmissionGate,
    SharedAdmissionGate,
};
use crate::memory_limiter::{LocalReceiverAdmissionState, SharedReceiverAdmissionState};
use otap_df_config::policy::{RateLimitUnit, RateLimiterDeclarationScope, RateLimiterPolicy};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

const UNBOUND: u8 = 0;
const BOUND: u8 = 1;

/// How a node acquired its effective admission policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionBindingProvenance {
    /// The node explicitly named the limiter.
    Explicit,
    /// The node inherited the sole effective limiter.
    ImplicitSingleton,
}

#[derive(Debug)]
struct BindingInner {
    limiter_name: Arc<str>,
    declaration_scope: Option<RateLimiterDeclarationScope>,
    policy: RateLimiterPolicy,
    provenance: AdmissionBindingProvenance,
    state: AtomicU8,
    clock: AdmissionClock,
    counters: Arc<RefusalCounters>,
}

/// Clone-safe admission identity and policy stored in [`crate::PipelineContext`].
///
/// Cloning this value never clones or broadens bucket state. The receiver-instance
/// bucket is created exactly once by `bind_local` or `bind_shared`, during the
/// component factory call. A component then clones the returned gate when several
/// protocol stacks need to charge the same bucket.
#[derive(Clone, Debug, Default)]
pub struct AdmissionBinder {
    inner: Option<Arc<BindingInner>>,
    ambiguous_candidates: Option<Arc<[String]>>,
}

impl AdmissionBinder {
    /// Creates an unconfigured binder.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            inner: None,
            ambiguous_candidates: None,
        }
    }

    /// Creates a configured binder using the production monotonic clock.
    #[must_use]
    pub fn configured(
        limiter_name: impl Into<Arc<str>>,
        policy: RateLimiterPolicy,
        provenance: AdmissionBindingProvenance,
    ) -> Self {
        Self::configured_at_scope(limiter_name, None, policy, provenance)
    }

    /// Creates a configured binder while preserving declaration identity.
    #[must_use]
    pub fn configured_at_scope(
        limiter_name: impl Into<Arc<str>>,
        declaration_scope: Option<RateLimiterDeclarationScope>,
        policy: RateLimiterPolicy,
        provenance: AdmissionBindingProvenance,
    ) -> Self {
        Self::with_clock(
            limiter_name,
            declaration_scope,
            policy,
            provenance,
            AdmissionClock::system(),
        )
    }

    pub(crate) fn with_clock(
        limiter_name: impl Into<Arc<str>>,
        declaration_scope: Option<RateLimiterDeclarationScope>,
        policy: RateLimiterPolicy,
        provenance: AdmissionBindingProvenance,
        clock: AdmissionClock,
    ) -> Self {
        Self {
            inner: Some(Arc::new(BindingInner {
                limiter_name: limiter_name.into(),
                declaration_scope,
                policy,
                provenance,
                state: AtomicU8::new(UNBOUND),
                clock,
                counters: Arc::new(RefusalCounters::default()),
            })),
            ambiguous_candidates: None,
        }
    }

    /// Creates an unresolved implicit binding for several effective limiters.
    #[must_use]
    pub fn ambiguous(candidates: Vec<String>) -> Self {
        Self {
            inner: None,
            ambiguous_candidates: Some(candidates.into()),
        }
    }

    /// Returns whether a limiter is available to bind.
    #[must_use]
    pub fn is_configured(&self) -> bool {
        self.inner.is_some() || self.ambiguous_candidates.is_some()
    }

    /// Binds one deliberately thread-confined gate.
    ///
    /// The returned gate is `!Send` and uses `Rc` internally by design. Clones
    /// share one bucket without adding atomic reference counting to local ingress.
    pub fn bind_local(
        &self,
        dimension: AdmissionDimension,
        pressure: LocalReceiverAdmissionState,
    ) -> Result<Option<LocalAdmissionGate>, AdmissionBindError> {
        self.validate_unambiguous()?;
        let Some(inner) = &self.inner else {
            return Ok(None);
        };
        inner.validate_dimension(dimension)?;
        inner.claim()?;
        Ok(Some(LocalAdmissionGate::new(RateGate::new(
            inner.policy,
            pressure,
            inner.clock.clone(),
            Arc::clone(&inner.counters),
        ))))
    }

    /// Binds one `Send + Sync` gate whose clones share one bucket.
    pub fn bind_shared(
        &self,
        dimension: AdmissionDimension,
        pressure: SharedReceiverAdmissionState,
    ) -> Result<Option<SharedAdmissionGate>, AdmissionBindError> {
        self.validate_unambiguous()?;
        let Some(inner) = &self.inner else {
            return Ok(None);
        };
        inner.validate_dimension(dimension)?;
        inner.claim()?;
        Ok(Some(SharedAdmissionGate::new(RateGate::new(
            inner.policy,
            pressure,
            inner.clock.clone(),
            Arc::clone(&inner.counters),
        ))))
    }

    /// Enforces that an explicit binding was consumed during factory construction.
    pub fn validate_factory_consumption(
        &self,
        component_urn: &str,
    ) -> Result<(), AdmissionBindError> {
        let Some(inner) = &self.inner else {
            return Ok(());
        };
        if inner.provenance == AdmissionBindingProvenance::Explicit
            && inner.state.load(Ordering::Acquire) != BOUND
        {
            return Err(AdmissionBindError::ExplicitBindingNotConsumed {
                limiter: inner.limiter_name.to_string(),
                component: component_urn.to_owned(),
            });
        }
        Ok(())
    }

    /// Returns true when a component consumed this binding during construction.
    #[must_use]
    pub fn was_bound(&self) -> bool {
        self.inner
            .as_ref()
            .is_some_and(|inner| inner.state.load(Ordering::Acquire) == BOUND)
    }

    /// Returns the binding provenance for startup reporting.
    #[must_use]
    pub fn provenance(&self) -> Option<AdmissionBindingProvenance> {
        self.inner.as_ref().map(|inner| inner.provenance)
    }

    /// Returns true when inherited limiter configuration was not consumed.
    ///
    /// This includes both a sole implicit limiter and an ambiguous set of
    /// inherited limiters. Ambiguity is only an error when a component opts in
    /// by binding; non-participating components remain valid and are reported
    /// in the aggregate startup summary.
    #[must_use]
    pub fn is_implicit_unbound(&self) -> bool {
        !self.was_bound()
            && (self.ambiguous_candidates.is_some()
                || self.provenance() == Some(AdmissionBindingProvenance::ImplicitSingleton))
    }

    /// Returns the configured limiter name.
    #[must_use]
    pub fn limiter_name(&self) -> Option<&str> {
        self.inner.as_ref().map(|inner| inner.limiter_name.as_ref())
    }

    /// Returns the configuration scope that declared the effective limiter family.
    #[must_use]
    pub fn declaration_scope(&self) -> Option<&RateLimiterDeclarationScope> {
        self.inner
            .as_ref()
            .and_then(|inner| inner.declaration_scope.as_ref())
    }

    pub(crate) fn metrics_handle(
        &self,
        pipeline_ctx: &crate::PipelineContext,
    ) -> Option<AdmissionMetricsHandle> {
        let inner = self.inner.as_ref()?;
        (inner.state.load(Ordering::Acquire) == BOUND).then(|| {
            AdmissionMetricsHandle::new(
                pipeline_ctx,
                AdmissionDimension::from(inner.policy.unit),
                Arc::clone(&inner.counters),
            )
        })
    }

    fn validate_unambiguous(&self) -> Result<(), AdmissionBindError> {
        if let Some(candidates) = &self.ambiguous_candidates {
            return Err(AdmissionBindError::AmbiguousBinding {
                candidates: candidates.to_vec(),
            });
        }
        Ok(())
    }
}

impl BindingInner {
    fn validate_dimension(&self, requested: AdmissionDimension) -> Result<(), AdmissionBindError> {
        let supported = AdmissionDimension::from(self.policy.unit);
        if requested == supported {
            return Ok(());
        }
        Err(AdmissionBindError::UnsupportedDimension {
            requested,
            supported: AdmissionDimensionSet::single(supported),
        })
    }

    fn claim(&self) -> Result<(), AdmissionBindError> {
        self.state
            .compare_exchange(UNBOUND, BOUND, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| AdmissionBindError::AlreadyBound)
    }
}

impl From<RateLimitUnit> for AdmissionDimension {
    fn from(unit: RateLimitUnit) -> Self {
        match unit {
            RateLimitUnit::RequestBytes => Self::Bytes,
            RateLimitUnit::Messages => Self::Messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{AdmissionContext, AdmissionDecision};
    use crate::memory_limiter::{MemoryPressureState, SharedReceiverAdmissionState};
    use otap_df_config::policy::{
        RateLimitAggregation, RateLimitEnforcement, RateLimitPressure, TokenBucketPolicy,
    };
    use std::time::Duration;

    fn policy(unit: RateLimitUnit) -> RateLimiterPolicy {
        RateLimiterPolicy {
            enforcement: RateLimitEnforcement::Enforce,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit,
            pressure: RateLimitPressure::Soft,
            token_bucket: TokenBucketPolicy {
                allow: 1,
                interval: Duration::from_secs(1),
                burst: Some(1),
            },
        }
    }

    /// Scenario: a configured node claims its admission binding twice.
    /// Guarantees: the second claim fails at startup rather than creating an independent bucket.
    #[test]
    fn binding_is_one_shot_across_context_clones() {
        let binder = AdmissionBinder::configured_at_scope(
            "ingress",
            Some(RateLimiterDeclarationScope::Engine),
            policy(RateLimitUnit::RequestBytes),
            AdmissionBindingProvenance::Explicit,
        );
        let clone = binder.clone();
        let pressure =
            SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default());

        let gate = binder
            .bind_shared(AdmissionDimension::Bytes, pressure.clone())
            .expect("first bind")
            .expect("configured gate");
        assert!(matches!(
            clone.bind_shared(AdmissionDimension::Bytes, pressure),
            Err(AdmissionBindError::AlreadyBound)
        ));
        assert_eq!(
            gate.admit(1, AdmissionContext::EMPTY),
            AdmissionDecision::Admit
        );
        assert_eq!(
            clone.declaration_scope(),
            Some(&RateLimiterDeclarationScope::Engine)
        );
    }

    /// Scenario: one receiver clones its bound shared gate into two protocol stacks.
    /// Guarantees: both handles charge the same receiver-instance bucket rather than receiving
    /// independent allowances.
    #[test]
    fn shared_gate_clones_charge_one_bucket() {
        let binder = AdmissionBinder::configured(
            "ingress",
            policy(RateLimitUnit::RequestBytes),
            AdmissionBindingProvenance::Explicit,
        );
        let process_state = MemoryPressureState::default();
        process_state.set_level_for_tests(crate::memory_limiter::MemoryPressureLevel::Soft);
        let pressure = SharedReceiverAdmissionState::from_process_state(&process_state);
        pressure.apply(process_state.current_update(1));
        let grpc_gate = binder
            .bind_shared(AdmissionDimension::Bytes, pressure)
            .expect("bind shared gate")
            .expect("configured gate");
        let http_gate = grpc_gate.clone();

        assert_eq!(
            grpc_gate.admit(1, AdmissionContext::EMPTY),
            AdmissionDecision::Admit
        );
        assert!(matches!(
            http_gate.admit(1, AdmissionContext::EMPTY),
            AdmissionDecision::Throttle { .. }
        ));
    }

    /// Scenario: a participating component sees several inherited limiters without selecting one.
    /// Guarantees: binding fails at startup while a non-participating component may leave the
    /// ambiguous binder untouched.
    #[test]
    fn ambiguous_implicit_binding_fails_only_when_consumed() {
        let binder = AdmissionBinder::ambiguous(vec!["bytes".into(), "messages".into()]);
        assert!(binder.is_implicit_unbound());
        assert!(
            binder
                .validate_factory_consumption("urn:otel:receiver:host_metrics")
                .is_ok()
        );
        let pressure =
            SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default());
        assert!(matches!(
            binder.bind_shared(AdmissionDimension::Bytes, pressure),
            Err(AdmissionBindError::AmbiguousBinding { .. })
        ));
    }

    /// Scenario: a component requests a dimension different from the configured policy.
    /// Guarantees: validation fails before consuming the binding, allowing the diagnostic to
    /// report a dimension mismatch rather than an already-bound error.
    #[test]
    fn dimension_is_validated_before_binding_is_consumed() {
        let binder = AdmissionBinder::configured(
            "ingress",
            policy(RateLimitUnit::Messages),
            AdmissionBindingProvenance::Explicit,
        );
        let pressure =
            SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default());

        assert!(matches!(
            binder.bind_shared(AdmissionDimension::Bytes, pressure.clone()),
            Err(AdmissionBindError::UnsupportedDimension { .. })
        ));
        assert!(
            binder
                .bind_shared(AdmissionDimension::Messages, pressure)
                .expect("binding remains available")
                .is_some()
        );
    }

    /// Scenario: an explicitly configured component returns without binding admission.
    /// Guarantees: post-factory validation rejects the configuration with limiter and component
    /// identity, while an implicit unconsumed binding remains valid.
    #[test]
    fn only_explicit_unconsumed_bindings_are_errors() {
        let explicit = AdmissionBinder::configured(
            "ingress",
            policy(RateLimitUnit::Messages),
            AdmissionBindingProvenance::Explicit,
        );
        assert!(matches!(
            explicit.validate_factory_consumption("urn:otel:receiver:host_metrics"),
            Err(AdmissionBindError::ExplicitBindingNotConsumed { .. })
        ));

        let implicit = AdmissionBinder::configured(
            "ingress",
            policy(RateLimitUnit::Messages),
            AdmissionBindingProvenance::ImplicitSingleton,
        );
        assert!(
            implicit
                .validate_factory_consumption("urn:otel:receiver:host_metrics")
                .is_ok()
        );
    }
}
