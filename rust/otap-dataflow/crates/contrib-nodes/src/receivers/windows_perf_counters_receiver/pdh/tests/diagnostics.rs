// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::super::config::InstanceConfig;
use super::super::super::model::{
    Number, SampleDiagnostics, SampleFailure, SampleMember, SamplePoint,
};
use super::super::tests::counter;
use super::*;

/// Scenario: Unique wildcard members fail and disappear repeatedly under a one-member admission limit.
/// Guarantees: Warning suppression retains current/retrying identities, drops departed identities, and re-arms when a removed identity returns.
#[test]
fn wildcard_warning_state_is_bounded_by_current_membership() {
    let mut config = counter(r"\Process(*)\Private Bytes", "process");
    config.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut state = SampleLogState::new("test".to_owned(), vec![config]);
    for index in 0..100 {
        let path: Arc<str> = Arc::from(format!(r"\Process(worker{index})\Private Bytes"));
        let mut failed = Sample::empty(100);
        failed.members.push(SampleMember {
            counter_index: 0,
            path: Arc::clone(&path),
            instance: None,
            emit_instance_attribute: false,
        });
        failed.metadata.failures.push(SampleFailure {
            counter_index: 0,
            path,
            reason: "counter CStatus",
            error: "invalid data".to_owned(),
            diagnostic_only: false,
        });
        state.log(&failed).unwrap();
        assert_eq!(state.active_counter_failures.len(), 1);
        failed.metadata.failures.clear();
        state.log(&failed).unwrap();
        assert_eq!(state.active_counter_failures.len(), 1);
        state.log(&Sample::empty(110)).unwrap();
        assert!(state.active_counter_failures.is_empty());
    }
    let path: Arc<str> = Arc::from(r"\Process(worker0)\Private Bytes");
    let mut removed = Sample::empty(120);
    removed.metadata.failures.push(SampleFailure {
        counter_index: 0,
        path,
        reason: "PdhRemoveCounter",
        error: "cleanup pending".to_owned(),
        diagnostic_only: true,
    });
    state.log(&removed).unwrap();
    state.log(&removed).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
    state.log(&Sample::empty(130)).unwrap();
    assert!(state.active_counter_failures.is_empty());
    removed.metadata.failures[0].reason = "counter CStatus";
    state.log(&removed).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
}

/// Scenario: A concrete counter fails, is absent and omitted on later scrapes, then produces a valid value before failing again.
/// Guarantees: Warning suppression persists until real value recovery and re-arms only after that recovery.
#[test]
fn counter_failure_suppression_requires_valid_recovery() {
    let config = counter(r"\Memory\Available Bytes", "memory");
    let mut state = SampleLogState::new("test".to_owned(), vec![config]);
    let path = Arc::<str>::from(r"\Memory\Available Bytes");
    let mut failed = Sample::empty(10);
    failed.metadata.failures.push(SampleFailure {
        counter_index: 0,
        path: Arc::clone(&path),
        reason: "counter CStatus",
        error: "PDH_INVALID_DATA".to_owned(),
        diagnostic_only: false,
    });
    state.log(&failed).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
    state.log(&Sample::empty(20)).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
    let mut omitted = Sample::empty(30);
    omitted.points.push(SamplePoint {
        counter_index: 0,
        path,
        instance: None,
        emit_instance_attribute: false,
        timestamp_unix_nano: 30,
        start_time_unix_nano: None,
        value: SampleValue::NoObservation,
    });
    state.log(&omitted).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
    omitted.points[0].timestamp_unix_nano = 40;
    omitted.points[0].value = SampleValue::Value(Number::Integer(42));
    state.log(&omitted).unwrap();
    assert!(state.active_counter_failures.is_empty());
    state.log(&failed).unwrap();
    assert_eq!(state.active_counter_failures.len(), 1);
}

/// Scenario: Steady samples surround each counter-retry and query-rebuild transition.
/// Guarantees: Only recovery activity emits recovery.change, including a rebuild recovery without a concurrent attempt.
#[test]
fn recovery_events_are_silent_without_activity() {
    use tracing_subscriber::layer::SubscriberExt;
    // Subscriber capture is thread-safe, but scoped to this test rather than production state.
    struct Events(Arc<std::sync::Mutex<Vec<&'static str>>>);
    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Events {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _: tracing_subscriber::layer::Context<'_, S>,
        ) {
            self.0.lock().unwrap().push(event.metadata().name());
        }
    }
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Events(Arc::clone(&events)));
    let mut state = SampleLogState::new("test".to_owned(), Vec::new());
    tracing::subscriber::with_default(subscriber, || {
        let mut sample = Sample::empty(100);
        state.log(&sample).unwrap();
        for diagnostics in [
            SampleDiagnostics {
                retry_attempts: 1,
                ..Default::default()
            },
            SampleDiagnostics {
                retry_recoveries: 1,
                ..Default::default()
            },
            SampleDiagnostics {
                query_rebuild_attempts: 1,
                ..Default::default()
            },
            SampleDiagnostics {
                query_rebuild_recoveries: 1,
                ..Default::default()
            },
        ] {
            sample.metadata.diagnostics = diagnostics;
            state.log(&sample).unwrap();
        }
        state.log(&Sample::empty(101)).unwrap();
    });
    assert_eq!(
        *events.lock().unwrap(),
        ["windows_perf_counters_receiver.recovery.change"; 4]
    );
}

/// Scenario: Query ends and reopens selector or native-identity failure episodes without a valued counter observation.
/// Guarantees: Each query-owned episode emits its warning instead of being suppressed until value recovery.
#[test]
fn query_owned_episodes_rearm_without_valued_observations() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tracing_subscriber::layer::SubscriberExt;

    // Event capture is scoped to this test's subscriber, not shared production state.
    struct Warnings(Arc<AtomicUsize>);
    impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for Warnings {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _: tracing_subscriber::layer::Context<'_, S>,
        ) {
            if event.metadata().name() == "windows_perf_counters_receiver.counter.fail" {
                let _ = self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    let warnings = Arc::new(AtomicUsize::new(0));
    let subscriber = tracing_subscriber::registry().with(Warnings(Arc::clone(&warnings)));
    tracing::subscriber::with_default(subscriber, || {
        for reason in ["selector_unresolved", "native_identity_unavailable"] {
            let path = r"\Process(*)\Private Bytes";
            let mut state = SampleLogState::new("test".to_owned(), vec![counter(path, "process")]);
            let mut sample = Sample::empty(100);
            let failure = SampleFailure {
                counter_index: 0,
                path: Arc::from(path),
                reason,
                error: "identity or selector unavailable".to_owned(),
                diagnostic_only: true,
            };
            sample.metadata.failures.push(failure.clone());
            state.log(&sample).unwrap();
            sample.metadata.failures.clear();
            state.log(&sample).unwrap();
            sample.metadata.failures.push(failure);
            state.log(&sample).unwrap();
        }
    });
    assert_eq!(warnings.load(Ordering::Relaxed), 4);
}
