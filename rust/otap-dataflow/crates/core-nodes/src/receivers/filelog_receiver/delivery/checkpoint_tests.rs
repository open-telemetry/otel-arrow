// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// These tests live under identity's fixtures to inject faults through the existing store hooks.
use super::*;
use crate::receivers::filelog_receiver::delivery::{
    Blocked, CheckpointOutcome, Delivery, DeliveryError, PendingOwner, SendOutcome,
    SyncOutcome as DeliverySyncOutcome, send_attempt,
    tests::{Builder, builder, handler},
};
use std::time::Instant;

fn batch(seed: &ReaderSeed) -> Builder {
    builder(vec![cp::UpdateProgress {
        file_id: seed.file_id,
        expected_committed_offset: seed.offset,
        expected_file_epoch: seed.epoch,
        new_committed_offset: 3,
        new_committed_frontier_guard: cp::CommittedFrontierGuard::compute(3, b"abc").unwrap(),
        new_framing_resume: cp::FramingResume::Clean,
        new_last_seen_time_unix_nano: 1,
        finalize: false,
    }])
}
fn ack(d: &mut Delivery<Builder>, now: Instant) -> otel_arrow_dfe_engine::control::CallData {
    let attempt = d.next_attempt(now).unwrap().unwrap();
    let id = attempt.correlation;
    d.sent(id, SendOutcome::Accepted, now).unwrap();
    let data = id.calldata();
    assert!(d.complete(&data, true, now).unwrap());
    data
}

/// Scenario: A batch traverses the engine output, returns its matching Ack, and the checkpoint store is reopened afterward.
/// Guarantees: Durable application precedes release and exact carry promotion; recovered progress includes only authorized bytes.
#[tokio::test(flavor = "current_thread")]
async fn ack_commits_before_release_and_recovery() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let (tx, rx) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
    let handler = handler(tx);
    let attempt = d.next_attempt(now).unwrap().unwrap();
    let correlation = attempt.correlation;
    let outcome = tokio::time::timeout(
        Duration::from_secs(30),
        send_attempt(&handler, attempt, std::future::pending()),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(outcome, SendOutcome::Accepted);
    d.sent(correlation, outcome, now).unwrap();
    let mut received = rx.try_recv().unwrap();
    let id = received.context_mut().pop_frame().unwrap().route.calldata;
    assert_eq!(id, correlation.calldata());
    assert!(d.complete(&id, true, now).unwrap());
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        0
    );
    assert!(d.builder_mut(now).is_none());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let b = d.builder_mut(now).unwrap();
    assert!(b.released);
    assert_eq!(b.promoted, Some(vec![0, 255, 13, 10, 42]));
    assert!(b.carry.is_none());
    assert_eq!(owner.synced_sequence(), 2);
    assert!(!d.complete(&id, true, now).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::NotReady
    ));
    assert_eq!(owner.synced_sequence(), 2);
    drop(owner);
    let recovered = f.owner(&inputs);
    assert_eq!(
        recovered
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
}

/// Scenario: An Acked append fails before bytes, after a partial frame, or after the complete frame.
/// Guarantees: Original transaction resolution applies exactly once, blocks resend and carry promotion, and never appends duplicate progress.
#[test]
fn checkpoint_failure_resolves_original_transaction_once() {
    for bytes in [0, 12, usize::MAX] {
        let f = Fixture::new();
        let path = f.write("app", b"abcdef");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        let now = Instant::now();
        let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
        let id = ack(&mut d, now);
        owner.store.inject_write_failure_after(bytes);
        assert!(matches!(
            d.checkpoint(&mut owner, now),
            Ok(CheckpointOutcome::Blocked(Blocked {
                owner: PendingOwner::Batch,
                cause: Some(_)
            }))
        ));
        assert!(d.builder_mut(now).is_none());
        assert!(d.next_attempt(now).unwrap().is_none());
        assert!(!d.complete(&id, true, now).unwrap());
        assert!(matches!(
            d.checkpoint(&mut owner, now).unwrap(),
            CheckpointOutcome::Blocked(Blocked {
                owner: PendingOwner::Batch,
                cause: None
            })
        ));
        assert!(!d.is_stopped());
        let at = d.next_deadline().unwrap();
        assert!(matches!(
            d.checkpoint(&mut owner, at).unwrap(),
            CheckpointOutcome::Applied(_)
        ));
        assert_eq!(owner.synced_sequence(), 2);
        assert!(d.builder_mut(at).unwrap().released);
        assert!(matches!(
            d.checkpoint(&mut owner, at).unwrap(),
            CheckpointOutcome::NotReady
        ));
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&seed.file_id)
                .unwrap()
                .committed_offset,
            3
        );
    }
}

/// Scenario: Required file synchronization fails after an Acked append.
/// Guarantees: Neither retries nor later completions release the batch or certify progress; the authority stays stopped.
#[test]
fn failed_required_sync_is_terminal() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let b = batch(&seed);
    let drops = b.drops.clone();
    let mut d = Delivery::new(b, &inputs).unwrap();
    let id = ack(&mut d, now);
    owner.store.inject_sync_failure("append file sync");
    assert!(d.checkpoint(&mut owner, now).is_err());
    assert!(owner.is_stopped());
    assert!(d.builder_mut(now).is_none());
    assert!(d.next_deadline().is_none());
    assert!(!d.complete(&id, true, now).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::NotReady
    ));
    assert_eq!(drops.get(), 0);
    drop(d);
    assert_eq!(drops.get(), 1);
}

/// Scenario: Delivery is cancelled after publication but before a completion.
/// Guarantees: Store replay recovers the original offset and no unacknowledged progress survives.
#[test]
fn uncommitted_progress_is_not_recovered() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let attempt = d.next_attempt(now).unwrap().unwrap();
    d.sent(attempt.correlation, SendOutcome::Accepted, now)
        .unwrap();
    d.cancel();
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::NotReady
    ));
    drop(d);
    drop(owner);
    let recovered = f.owner(&inputs);
    assert_eq!(
        recovered
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        0
    );
}

/// Scenario: Deferred durability releases an applied batch, then its deadline arrives with no source events.
/// Guarantees: Source admission pauses at the deadline until sync; the periodic barrier covers the applied transaction.
#[test]
fn delayed_sync_runs_without_new_source_work() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    assert_eq!(owner.synced_sequence(), 1);
    assert!(d.builder_mut(now).unwrap().released);
    let due = d.next_deadline().unwrap();
    assert_eq!(due.duration_since(now), Duration::from_secs(2));
    assert!(d.builder_mut(due).is_none());
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Synced
    ));
    assert_eq!(owner.synced_sequence(), 2);
    assert!(d.builder_mut(due).is_some());
    assert!(d.next_deadline().is_none());
}

/// Scenario: One delta in a multi-file Ack has the wrong epoch.
/// Guarantees: Store preflight rejects the entire transaction, leaves valid-file progress unchanged and blocks release.
#[test]
fn mismatched_epoch_rejects_whole_batch() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let other = f.write("other", b"uvwxyz");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let second = commit(&mut owner, &other, Context::Unrelated);
    let mut b = batch(&seed);
    let mut invalid = batch(&second).progress[0].clone();
    invalid.expected_file_epoch += 1;
    b.progress.push(invalid);
    let now = Instant::now();
    let mut d = Delivery::new(b, &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(d.checkpoint(&mut owner, now).is_err());
    assert!(d.builder_mut(now).is_none());
    for id in [seed.file_id, second.file_id] {
        assert_eq!(owner.state().unwrap().get(&id).unwrap().committed_offset, 0);
    }
}

/// Scenario: An idle periodic sync fails after deferred progress was applied and released.
/// Guarantees: The coordinator blocks future source intake and attempts and cannot retry the failed durability barrier.
#[test]
fn periodic_sync_failure_blocks_fresh_data() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let due = d.next_deadline().unwrap();
    owner.store.inject_sync_failure("explicit file sync");
    assert!(d.sync(&mut owner, due).is_err());
    assert!(owner.is_stopped());
    assert!(d.builder_mut(due).is_none());
    assert!(d.next_attempt(due).unwrap().is_none());
    assert!(d.next_deadline().is_none());
}

/// Scenario: An admission or host-initiated fingerprint write is pending when an Ack needs persistence.
/// Guarantees: Delivery leaves resolution and admission results with the host, suppresses busy-loop deadlines, and applies the Ack only afterward.
#[test]
fn host_pending_write_blocks_without_taking_ownership() {
    for admission in [true, false] {
        let f = Fixture::new();
        let path = f.write("app", b"");
        let inputs = f.inputs(|_| {});
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        fs::write(&path, b"abcdef").unwrap();
        let now = Instant::now();
        let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
        let _ = ack(&mut d, now);
        if admission {
            let other = f.write("other", b"different");
            let plan = ready(prepare(&mut owner, &other, Context::Unrelated).unwrap());
            owner.store.inject_write_failure_after(0);
            assert!(matches!(
                owner.commit(plan),
                Err(IdentityError::PendingWrite(_))
            ));
        } else {
            assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
            owner.store.inject_write_failure_after(0);
            assert!(matches!(
                owner.flush_fingerprints(),
                Err(IdentityError::PendingWrite(_))
            ));
        }
        assert!(matches!(
            d.checkpoint(&mut owner, now).unwrap(),
            CheckpointOutcome::Blocked(Blocked {
                owner: PendingOwner::Host,
                ..
            })
        ));
        assert!(matches!(
            d.sync(&mut owner, now).unwrap(),
            DeliverySyncOutcome::Blocked(Blocked {
                owner: PendingOwner::Host,
                ..
            })
        ));
        assert!(!d.is_stopped());
        assert!(d.next_deadline().is_none());
        assert!(d.next_attempt(now).unwrap().is_none());
        assert!(d.builder_mut(now).is_none());
        assert!(!d.builder().released);
        let resolved = owner.resolve_pending().unwrap();
        assert_eq!(resolved.admissions.len(), usize::from(admission));
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&seed.file_id)
                .unwrap()
                .committed_offset,
            0
        );
        assert!(matches!(
            d.checkpoint(&mut owner, now).unwrap(),
            CheckpointOutcome::Applied(_)
        ));
        assert!(d.builder().released);
        assert_eq!(
            owner
                .state()
                .unwrap()
                .get(&seed.file_id)
                .unwrap()
                .committed_offset,
            3
        );
    }
}

fn next_progress(seed: &ReaderSeed) -> cp::UpdateProgress {
    let mut next = batch(seed).progress[0].clone();
    next.expected_committed_offset = 3;
    next.new_committed_offset = 6;
    next.new_committed_frontier_guard = cp::CommittedFrontierGuard::compute(6, b"abcdef").unwrap();
    next
}

/// Scenario: Required deferred sync succeeds, then its optional fingerprint append fails while another batch is retained.
/// Guarantees: Growth has a separate retry deadline and cannot release or authorize that batch; source work stays blocked until growth resolves.
#[test]
fn owned_growth_resolution_does_not_release_next_batch() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abcdef").unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abcdef").unwrap());
    let now = Instant::now();
    let mut b = batch(&seed);
    b.reseal = Some(next_progress(&seed));
    let mut d = Delivery::new(b, &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let due = d.next_deadline().unwrap();
    owner.store.inject_write_failure_after(0);
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            cause: Some(_)
        })
    ));
    assert_eq!(
        owner.synced_sequence(),
        2,
        "required barrier already succeeded"
    );
    let retry = d.next_deadline().unwrap();
    assert_eq!(
        retry.duration_since(due),
        inputs.settings().retry.max_backoff
    );
    assert!(matches!(
        d.checkpoint(&mut owner, due).unwrap(),
        CheckpointOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            ..
        })
    ));
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            cause: None
        })
    ));
    assert!(!d.is_stopped());
    assert!(d.builder_mut(retry).is_none());
    assert!(d.next_attempt(retry).unwrap().is_none());
    assert!(matches!(
        d.sync(&mut owner, retry).unwrap(),
        DeliverySyncOutcome::Synced
    ));
    assert!(
        d.next_deadline().is_none(),
        "completed sync deadline was cleared"
    );
    assert!(
        !d.builder().released,
        "growth did not resolve the next batch"
    );
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
    let _ = ack(&mut d, retry);
    assert!(matches!(
        d.checkpoint(&mut owner, retry).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        6
    );
}

/// Scenario: A host write blocks an overdue deferred-sync deadline.
/// Guarantees: Delivery stays live without repeated expired wakeups, never consumes the host result, and services the barrier after the host wakes it.
#[test]
fn host_pending_write_blocks_periodic_sync() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let due = d.next_deadline().unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
    owner.store.inject_write_failure_after(0);
    assert!(owner.flush_fingerprints().is_err());
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Blocked(Blocked {
            owner: PendingOwner::Host,
            ..
        })
    ));
    assert!(d.next_deadline().is_none());
    assert!(d.builder_mut(due).is_none());
    assert!(!d.is_stopped());
    let resolved = owner.resolve_pending().unwrap();
    assert!(resolved.admissions.is_empty());
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Synced
    ));
    assert!(d.builder_mut(due).is_some());
}

/// Scenario: Optional growth fails its own required sync after the ordinary progress barrier succeeded.
/// Guarantees: The completed barrier is not undone, but terminal authority failure stops all future delivery and source intake.
#[test]
fn growth_sync_failure_remains_terminal() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let due = d.next_deadline().unwrap();
    owner.store.inject_sync_failure("append file sync");
    assert!(d.sync(&mut owner, due).is_err());
    assert_eq!(owner.synced_sequence(), 2);
    assert!(owner.is_stopped());
    assert!(d.is_stopped());
    assert!(d.builder_mut(due).is_none());
    assert!(d.next_attempt(due).unwrap().is_none());
    assert!(d.next_deadline().is_none());
}

/// Scenario: An invalid optional growth proposal is rejected before any write after required synchronization.
/// Guarantees: The error is observable without stopping delivery or leaving an expired sync deadline.
#[test]
fn rejected_growth_is_reported_without_stopping() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abc").unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
    // Inject an invalid optional proposal beyond the public queue's validated boundary.
    owner.growth[0].new_fingerprint.resize(17, b'x');
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let due = d.next_deadline().unwrap();
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::MaintenanceRejected(_)
    ));
    assert!(!owner.is_stopped());
    assert!(!owner.has_pending_write());
    assert!(!d.is_stopped());
    assert!(d.next_deadline().is_none());
    assert!(d.builder_mut(due).is_some());
    assert_eq!(owner.synced_sequence(), 2);
}

/// Scenario: Resolving a batch append or a delivery-initiated growth write reaches the authority's failure limit.
/// Guarantees: Recoverable outcomes become terminal only at the configured limit, with no unauthorized release or further retry deadline.
#[test]
fn resolution_failure_budget_is_terminal() {
    for growth in [false, true] {
        let f = Fixture::new();
        let path = f.write("app", b"");
        let inputs = f.inputs(|c| {
            c.checkpoint.max_consecutive_failures = 2;
            if growth {
                c.checkpoint.sync_interval = Duration::from_secs(2);
            }
        });
        let mut owner = f.owner(&inputs);
        let seed = commit(&mut owner, &path, Context::Unrelated);
        fs::write(&path, b"abc").unwrap();
        let now = Instant::now();
        let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
        let _ = ack(&mut d, now);
        if growth {
            assert!(owner.queue_fingerprint(&seed, b"abc").unwrap());
            assert!(matches!(
                d.checkpoint(&mut owner, now).unwrap(),
                CheckpointOutcome::Applied(_)
            ));
            let due = d.next_deadline().unwrap();
            owner.store.inject_write_failure_after(0);
            assert!(matches!(
                d.sync(&mut owner, due).unwrap(),
                DeliverySyncOutcome::Blocked(_)
            ));
        } else {
            owner.store.inject_write_failure_after(0);
            assert!(matches!(
                d.checkpoint(&mut owner, now).unwrap(),
                CheckpointOutcome::Blocked(_)
            ));
        }
        assert!(!d.is_stopped());
        let retry = d.next_deadline().unwrap();
        owner.store.inject_write_failure_after(0);
        if growth {
            assert!(d.sync(&mut owner, retry).is_err());
        } else {
            assert!(d.checkpoint(&mut owner, retry).is_err());
        }
        assert!(owner.is_stopped());
        assert!(d.is_stopped());
        assert_eq!(d.builder().released, growth);
        assert!(d.next_deadline().is_none());
    }
}

/// Scenario: A builder rejects release after its batch has already been applied durably.
/// Guarantees: The original builder error survives and no retry or duplicate completion can append the progress again.
#[test]
fn release_failure_preserves_error_and_prevents_reapplication() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut b = batch(&seed);
    b.fail_release = true;
    let mut d = Delivery::new(b, &inputs).unwrap();
    let id = ack(&mut d, now);
    let Err(DeliveryError::Batch(error)) = d.checkpoint(&mut owner, now) else {
        panic!("expected release failure")
    };
    assert!(
        error
            .downcast_ref::<crate::receivers::filelog_receiver::delivery::tests::BuilderError>()
            .is_some()
    );
    assert!(d.is_stopped());
    assert!(!d.builder().released);
    assert_eq!(owner.synced_sequence(), 2);
    assert!(!d.complete(&id, true, now).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::NotReady
    ));
    assert_eq!(owner.synced_sequence(), 2);
}

/// Scenario: A deferred-durability batch suffers a partial append and is resolved under the original authority.
/// Guarantees: The matching Ack remains the sole authorization, resolution applies once and release follows its receipt.
#[test]
fn deferred_pending_batch_resolves_once() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut d = Delivery::new(batch(&seed), &inputs).unwrap();
    let id = ack(&mut d, now);
    owner.store.inject_write_failure_after(12);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Blocked(_)
    ));
    assert!(!d.complete(&id, true, now).unwrap());
    assert!(!d.builder().released);
    let at = d.next_deadline().unwrap();
    assert!(matches!(
        d.checkpoint(&mut owner, at).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    assert!(d.builder().released);
    assert_eq!(owner.synced_sequence(), 2);
    assert!(matches!(
        d.checkpoint(&mut owner, at).unwrap(),
        CheckpointOutcome::NotReady
    ));
    drop(owner);
    let recovered = f.owner(&inputs);
    assert_eq!(
        recovered
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
}

/// Scenario: Carry-over immediately seals another retained batch when its predecessor is released.
/// Guarantees: The new batch gets a fresh identity, rejects the prior Ack, and cannot apply its progress before its own Ack.
#[test]
fn resealed_carry_requires_a_new_matching_ack() {
    let f = Fixture::new();
    let path = f.write("app", b"abcdef");
    let inputs = f.inputs(|_| {});
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    let now = Instant::now();
    let mut b = batch(&seed);
    b.reseal = Some(next_progress(&seed));
    let carry = b.carry.clone();
    let mut d = Delivery::new(b, &inputs).unwrap();
    let first = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    assert_eq!(d.builder().promoted, carry);
    assert!(!d.builder().released);
    let next = d.next_attempt(now).unwrap().unwrap();
    let second = next.correlation.calldata();
    assert_ne!(first, second);
    d.sent(next.correlation, SendOutcome::Accepted, now)
        .unwrap();
    assert!(!d.complete(&first, true, now).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::NotReady
    ));
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );
    assert!(d.complete(&second, true, now).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        6
    );
}

/// Scenario: A published batch receives Ack during growth recovery, then growth resolves with no timer remaining.
/// Guarantees: The worker's immediate checkpoint step applies that authorized batch exactly once without another event or resend.
#[test]
fn ack_during_growth_block_is_serviced_before_waiting() {
    let f = Fixture::new();
    let path = f.write("app", b"");
    let inputs = f.inputs(|c| c.checkpoint.sync_interval = Duration::from_secs(2));
    let mut owner = f.owner(&inputs);
    let seed = commit(&mut owner, &path, Context::Unrelated);
    fs::write(&path, b"abcdef").unwrap();
    assert!(owner.queue_fingerprint(&seed, b"abcdef").unwrap());
    let now = Instant::now();
    let mut b = batch(&seed);
    b.reseal = Some(next_progress(&seed));
    let mut d = Delivery::new(b, &inputs).unwrap();
    let _ = ack(&mut d, now);
    assert!(matches!(
        d.checkpoint(&mut owner, now).unwrap(),
        CheckpointOutcome::Applied(_)
    ));
    let second = d.next_attempt(now).unwrap().unwrap();
    let id = second.correlation;
    d.sent(id, SendOutcome::Accepted, now).unwrap();
    let due = d.next_deadline().unwrap();
    owner.store.inject_write_failure_after(0);
    assert!(matches!(
        d.sync(&mut owner, due).unwrap(),
        DeliverySyncOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            ..
        })
    ));
    assert!(d.complete(&id.calldata(), true, due).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, due).unwrap(),
        CheckpointOutcome::Blocked(Blocked {
            owner: PendingOwner::Growth,
            ..
        })
    ));
    assert!(!d.builder().released);
    let retry = d.next_deadline().unwrap();
    assert!(matches!(
        d.sync(&mut owner, retry).unwrap(),
        DeliverySyncOutcome::Synced
    ));
    assert!(
        d.next_deadline().is_none(),
        "no timer will drive the authorized batch"
    );
    assert!(
        !d.builder().released,
        "growth resolution must not release the batch"
    );
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        3
    );

    // The worker services immediate work before choosing its next wait.
    let CheckpointOutcome::Applied(progress) = d.checkpoint(&mut owner, retry).unwrap() else {
        panic!("authorized batch must apply without a fresh event")
    };
    assert_eq!(progress, vec![next_progress(&seed)]);
    assert!(d.builder().released);
    assert_eq!(
        owner
            .state()
            .unwrap()
            .get(&seed.file_id)
            .unwrap()
            .committed_offset,
        6
    );
    assert_eq!(owner.state().unwrap().last_sequence(), 4);
    assert!(d.next_attempt(retry).unwrap().is_none());
    assert!(d.builder_mut(retry).is_some());
    assert!(!d.complete(&id.calldata(), true, retry).unwrap());
    assert!(matches!(
        d.checkpoint(&mut owner, retry).unwrap(),
        CheckpointOutcome::NotReady
    ));
    assert_eq!(owner.state().unwrap().last_sequence(), 4);
}
