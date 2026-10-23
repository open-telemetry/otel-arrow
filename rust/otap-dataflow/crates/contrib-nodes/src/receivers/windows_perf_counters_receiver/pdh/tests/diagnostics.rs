// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::super::config::InstanceConfig;
use super::super::super::model::{
    Number, SampleDiagnostics, SampleFailure, SampleMember, SamplePoint,
};
use super::super::tests::{CapturedEvents, assert_isolated_tracing, counter};
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
        assert_eq!(state.log(&failed).unwrap(), 1);
        assert_eq!(state.active_counter_failures.len(), 1);
        failed.metadata.failures.clear();
        assert_eq!(state.log(&failed).unwrap(), 0);
        assert_eq!(state.active_counter_failures.len(), 1);
        assert_eq!(state.log(&Sample::empty(110)).unwrap(), 0);
        assert!(state.active_counter_failures.is_empty());
    }
    let path: Arc<str> = Arc::from(r"\Process(worker0)\Private Bytes");
    let mut removed = Sample::empty(120);
    removed.metadata.pending_cleanup = Some(vec![(0, Arc::clone(&path))]);
    removed.metadata.failures.push(SampleFailure {
        counter_index: 0,
        path,
        reason: "PdhRemoveCounter",
        error: "cleanup pending".to_owned(),
        diagnostic_only: true,
    });
    assert_eq!(state.log(&removed).unwrap(), 1);
    assert_eq!(state.log(&removed).unwrap(), 0);
    assert_eq!(state.active_counter_failures.len(), 1);
    let mut cleaned = Sample::empty(130);
    cleaned.metadata.pending_cleanup = Some(Vec::new());
    assert_eq!(state.log(&cleaned).unwrap(), 0);
    assert!(state.active_counter_failures.is_empty());
    removed.metadata.failures[0].reason = "counter CStatus";
    assert_eq!(state.log(&removed).unwrap(), 1);
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
    assert_eq!(state.log(&failed).unwrap(), 1);
    assert_eq!(state.active_counter_failures.len(), 1);
    assert_eq!(state.log(&Sample::empty(20)).unwrap(), 0);
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
    assert_eq!(state.log(&omitted).unwrap(), 0);
    assert_eq!(state.active_counter_failures.len(), 1);
    omitted.points[0].timestamp_unix_nano = 40;
    omitted.points[0].value = SampleValue::Value(Number::Integer(42));
    assert_eq!(state.log(&omitted).unwrap(), 0);
    assert!(state.active_counter_failures.is_empty());
    assert_eq!(state.log(&failed).unwrap(), 1);
    assert_eq!(state.active_counter_failures.len(), 1);
}

/// Scenario: Two mappings share a concrete path, and only one produces a valid value beside an unhealthy peer.
/// Guarantees: Recovery clears that mapping's concrete and template warnings without clearing other paths or no-observation warnings.
#[test]
fn valued_points_recover_only_matching_warning_identities() {
    let template = r"\Process(*)\Private Bytes";
    let worker = r"\Process(worker)\Private Bytes";
    let peer = r"\Process(peer)\Private Bytes";
    let mut wildcard = counter(template, "wildcard");
    wildcard.instance_config = InstanceConfig::Wildcard {
        excluded_aggregation: None,
    };
    let mut state =
        SampleLogState::new("test".to_owned(), vec![wildcard, counter(worker, "exact")]);
    let mut sample = Sample::empty(10);
    sample.members = [(0, worker), (0, peer), (1, worker)]
        .into_iter()
        .map(|(counter_index, path)| SampleMember {
            counter_index,
            path: Arc::from(path),
            instance: None,
            emit_instance_attribute: false,
        })
        .collect();
    sample.metadata.failures = [(0, template), (0, worker), (0, peer), (1, worker)]
        .into_iter()
        .map(|(counter_index, path)| SampleFailure {
            counter_index,
            path: Arc::from(path),
            reason: "counter CStatus",
            error: "invalid data".to_owned(),
            diagnostic_only: false,
        })
        .collect();
    assert_eq!(state.log(&sample).unwrap(), 4);
    let failures = std::mem::take(&mut sample.metadata.failures);
    sample.points = [0, 1]
        .into_iter()
        .map(|counter_index| SamplePoint {
            counter_index,
            path: Arc::from(worker),
            instance: None,
            emit_instance_attribute: false,
            timestamp_unix_nano: 20,
            start_time_unix_nano: None,
            value: if counter_index == 0 {
                SampleValue::Value(Number::Integer(42))
            } else {
                SampleValue::NoObservation
            },
        })
        .collect();
    assert_eq!(state.log(&sample).unwrap(), 0);
    sample.points.clear();
    sample.metadata.failures = failures;
    assert_eq!(state.log(&sample).unwrap(), 2);
}

/// Scenario: Three counters share collection and rebuild failures separated by backoff, then collection recovers.
/// Guarantees: One query warning covers the episode across reasons, backoff has no counter warnings, and successful collection rearms it.
#[test]
fn query_failure_warning_is_shared_until_successful_collection() {
    assert_isolated_tracing(|| {
        let configs = [
            r"\Memory\Available Bytes",
            r"\Memory\Committed Bytes",
            r"\Memory\Cache Bytes",
        ]
        .into_iter()
        .map(|path| counter(path, "memory"))
        .collect::<Vec<_>>();
        let events = CapturedEvents::default();
        tracing::dispatcher::with_default(&events.dispatch(), || {
            let mut state = SampleLogState::new("test".to_owned(), configs.clone());
            let failed = Error::Pdh {
                operation: super::super::error::PdhOperation::CollectQueryData,
                path: "<query>".to_owned(),
                status: 1,
            };
            assert_eq!(state.log_error(&failed).unwrap(), 1);
            let mut backoff = Sample::empty(10);
            super::super::query::record_query_failure(
                &mut backoff.metadata,
                &configs,
                &Error::QueryRecoveryPending {
                    retry_after_ms: 100,
                },
            );
            assert_eq!(backoff.metadata.failures.len(), 3);
            assert_eq!(state.log(&backoff).unwrap(), 0);
            assert_eq!(state.log_error(&failed).unwrap(), 0);
            assert_eq!(
                state
                    .log_error(&Error::Pdh {
                        operation: super::super::error::PdhOperation::OpenQuery,
                        path: "<query>".to_owned(),
                        status: 2,
                    })
                    .unwrap(),
                0
            );
            let mut rebuilt = Sample::empty(15);
            rebuilt.metadata.diagnostics.query_rebuild_recoveries = 1;
            assert_eq!(state.log(&rebuilt).unwrap(), 0);
            assert_eq!(state.log_error(&failed).unwrap(), 0);
            let mut recovered = Sample::empty(20);
            recovered.metadata.diagnostics.any_query_collected = true;
            assert_eq!(state.log(&recovered).unwrap(), 0);
            assert_eq!(state.log_error(&failed).unwrap(), 1);
        });
        assert_eq!(
            events.names(),
            [
                "windows_perf_counters_receiver.query.fail",
                "windows_perf_counters_receiver.recovery.change",
                "windows_perf_counters_receiver.query.fail",
            ]
        );
        assert!(
            events
                .query_failures()
                .iter()
                .all(|detail| detail.contains("0x00000001"))
        );
    });
}

/// Scenario: Three configured counters report add, read, data-status and removal failures in one query.
/// Guarantees: Query-level suppression never coalesces counter-local warning episodes.
#[test]
fn counter_local_failures_remain_per_counter() {
    let paths = [
        r"\Memory\Available Bytes",
        r"\Memory\Committed Bytes",
        r"\Memory\Cache Bytes",
    ];
    let mut state = SampleLogState::new(
        "test".to_owned(),
        paths.iter().map(|path| counter(path, "memory")).collect(),
    );
    let mut sample = Sample::empty(10);
    for reason in [
        "PdhAddEnglishCounterW",
        "PdhGetRawCounterValue",
        "raw counter CStatus",
        "PdhRemoveCounter",
    ] {
        sample.metadata.failures = paths
            .iter()
            .enumerate()
            .map(|(counter_index, path)| SampleFailure {
                counter_index,
                path: Arc::from(*path),
                reason,
                error: "PDH status 0x00000001".to_owned(),
                diagnostic_only: false,
            })
            .collect();
        assert_eq!(state.log(&sample).unwrap(), 3);
        assert_eq!(state.log(&sample).unwrap(), 0);
    }
}

/// Scenario: Steady samples surround each counter-retry and query-rebuild transition.
/// Guarantees: Only recovery activity emits recovery.change, including a rebuild recovery without a concurrent attempt.
#[test]
fn recovery_events_are_silent_without_activity() {
    assert_isolated_tracing(|| {
        let events = CapturedEvents::default();
        let mut state = SampleLogState::new("test".to_owned(), Vec::new());
        tracing::dispatcher::with_default(&events.dispatch(), || {
            let mut sample = Sample::empty(100);
            let _ = state.log(&sample).unwrap();
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
                let _ = state.log(&sample).unwrap();
            }
            let _ = state.log(&Sample::empty(101)).unwrap();
        });
        assert_eq!(
            events.names(),
            ["windows_perf_counters_receiver.recovery.change"; 4]
        );
    });
}

/// Scenario: Query ends and reopens selector or native-identity failure episodes without a valued counter observation.
/// Guarantees: Each query-owned episode emits its warning instead of being suppressed until value recovery.
#[test]
fn query_owned_episodes_rearm_without_valued_observations() {
    let mut warnings = 0;
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
        warnings += state.log(&sample).unwrap();
        sample.metadata.failures.clear();
        warnings += state.log(&sample).unwrap();
        sample.metadata.failures.push(failure);
        warnings += state.log(&sample).unwrap();
    }
    assert_eq!(warnings, 4);
}
