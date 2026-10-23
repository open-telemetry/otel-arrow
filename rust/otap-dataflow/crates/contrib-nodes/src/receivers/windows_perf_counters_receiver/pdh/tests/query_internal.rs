// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: The same failed counter is scheduled repeatedly before it recovers.
/// Guarantees: One stable identity owns one retry entry and repeated failures advance its bounded backoff.
#[test]
fn retry_state_is_unique_per_stable_identity() {
    let key = CounterKey::new(0, r"\Process(worker)\Private Bytes");
    let target = CounterTarget {
        config_index: 0,
        path: Arc::from(r"\Process(worker)\Private Bytes"),
        origin: CounterOrigin::Wildcard,
        instance: Some(Arc::new(InstanceIdentity {
            name: "worker".to_owned(),
            parent: None,
            index: 0,
        })),
        emit_instance_attribute: true,
        admission_order: Some(0),
        removal_counted: false,
    };
    let now = Instant::now();
    let mut states = BTreeMap::new();

    schedule_retry_state(
        &mut states,
        key.clone(),
        target.clone(),
        now,
        Duration::from_secs(30),
    );
    schedule_retry_state(
        &mut states,
        key.clone(),
        target,
        now,
        Duration::from_secs(30),
    );

    assert_eq!(states.len(), 1);
    let state = states.get(&key).unwrap();
    assert_eq!(state.attempts, 2);
    assert_eq!(state.next_attempt, now + Duration::from_secs(2));
}

/// Scenario: Exact and wildcard counter-add failures use distinct retry clocks.
/// Guarantees: Exact targets cap backoff at the collection retry cadence while wildcard targets retain the discovery refresh cap.
#[test]
fn retry_cap_follows_counter_origin() {
    let exact_retry_cap = Duration::from_secs(2);
    let wildcard_retry_cap = Duration::from_secs(60);

    assert_eq!(
        CounterOrigin::Exact.retry_cap(exact_retry_cap, wildcard_retry_cap),
        exact_retry_cap
    );
    assert_eq!(
        CounterOrigin::Wildcard.retry_cap(exact_retry_cap, wildcard_retry_cap),
        wildcard_retry_cap
    );
}
