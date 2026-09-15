// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalancing tests: assign/revoke, generation handling,
//! assignment-resume retries, and the shared rebalance-state bookkeeping
//! (construction, committable/offset guarantees, and metric counters).

use super::*;

// ---- Consumer-group rebalancing ----

/// Scenario (consumer-group rebalancing): partitions are added to, removed from, and
/// queried on an `AssignedPartitions` set.
/// Guarantees: add/remove/contains behave consistently and removing a topic's last
/// partition drops the topic entry, so the owned-set bookkeeping stays accurate.
#[test]
fn assigned_partitions_add_remove_contains() {
    let mut ap = AssignedPartitions::new();
    assert!(!ap.contains("traces", 0));

    ap.add_partition("traces", 0, 1);
    ap.add_partition("traces", 1, 1);
    ap.add_partition("metrics", 0, 1);

    assert!(ap.contains("traces", 0));
    assert!(ap.contains("traces", 1));
    assert!(ap.contains("metrics", 0));
    assert!(!ap.contains("traces", 2));
    assert!(!ap.contains("logs", 0));

    // remove_partition reports whether an owned partition was removed.
    assert!(ap.remove_partition("traces", 0));
    assert!(!ap.contains("traces", 0));
    assert!(ap.contains("traces", 1));

    // Removing the last partition drops the topic entry.
    assert!(ap.remove_partition("metrics", 0));
    assert!(!ap.contains("metrics", 0));
    assert!(!ap.topics.contains_key("metrics"));
}

/// Scenario (consumer-group rebalancing): an already-owned partition is re-added with a
/// different generation.
/// Guarantees: the existing generation is retained (not overwritten), so a partition
/// kept across a rebalance keeps its ownership generation.
#[test]
fn add_partition_keeps_existing_generation() {
    let mut ap = AssignedPartitions::new();
    ap.add_partition("traces", 0, 5);
    // Re-adding an already-owned partition must not change its generation
    // (retained across a rebalance).
    ap.add_partition("traces", 0, 9);
    assert_eq!(ap.generation("traces", 0), Some(5));
    assert_eq!(ap.generation("traces", 9), None);
}

/// Scenario (consumer-group rebalancing): an unknown partition or topic is removed from
/// the assigned set.
/// Guarantees: the removal reports `false` and changes nothing, so spurious revocations
/// cannot corrupt the owned set.
#[test]
fn remove_unknown_partition_is_noop() {
    let mut ap = AssignedPartitions::new();
    ap.add_partition("traces", 0, 1);
    // Unknown partition/topic removals report false and change nothing.
    assert!(!ap.remove_partition("traces", 99));
    assert!(!ap.remove_partition("unknown", 0));
    assert!(ap.contains("traces", 0));
}

/// Scenario (consumer-group rebalancing): a full assignment is applied that retains
/// some partitions and acquires others.
/// Guarantees: only newly-acquired partitions get a fresh generation while retained
/// ones keep theirs, so generation churn is scoped to genuine acquisitions.
#[test]
fn set_assignment_allocates_generation_for_new_partitions_only() {
    let state = RebalanceState::new(false);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    state.set_assignment(&tpl);
    let e0 = state.current_generation("traces", 0);
    assert!(e0 > 0);

    // Re-assigning the same set retains the partition -> generation unchanged.
    state.set_assignment(&tpl);
    assert_eq!(state.current_generation("traces", 0), e0);

    // Adding a new partition allocates a fresh, strictly-greater generation for
    // it while the retained partition keeps its generation.
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("traces", 0);
    let _ = tpl2.add_partition("traces", 1);
    state.set_assignment(&tpl2);
    assert_eq!(state.current_generation("traces", 0), e0);
    assert!(state.current_generation("traces", 1) > e0);
}

/// Scenario (consumer-group rebalancing): a single rebalance acquires several
/// partitions at once.
/// Guarantees: all partitions acquired in that rebalance share one freshly-allocated
/// generation (the allocator bumps at most once per call).
#[test]
fn set_assignment_shares_one_generation_across_partitions() {
    // All partitions acquired in a single rebalance share one generation,
    // and the allocator advances by exactly one per rebalance.
    let state = RebalanceState::new(false);

    // First rebalance acquires two partitions at once -> same generation.
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    let _ = tpl.add_partition("traces", 1);
    state.set_assignment(&tpl);
    let g0 = state.current_generation("traces", 0);
    let g1 = state.current_generation("traces", 1);
    assert_eq!(g0, g1, "partitions acquired together share one generation");
    assert_eq!(g0, 1, "first generation is 1");

    // Second rebalance retains 0 and 1, acquires 2 -> single bump to 2.
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("traces", 0);
    let _ = tpl2.add_partition("traces", 1);
    let _ = tpl2.add_partition("traces", 2);
    state.set_assignment(&tpl2);
    assert_eq!(state.current_generation("traces", 0), g0);
    assert_eq!(state.current_generation("traces", 1), g1);
    assert_eq!(
        state.current_generation("traces", 2),
        2,
        "one bump for the rebalance that acquired partition 2"
    );

    // A pure-retain rebalance acquires nothing -> allocator not bumped.
    state.set_assignment(&tpl2);
    assert_eq!(state.current_generation("traces", 0), g0);
    assert_eq!(state.current_generation("traces", 2), 2);
}

/// Scenario (consumer-group rebalancing): the additive-merge fallback adds several
/// partitions in one call.
/// Guarantees: all partitions merged in that call share one freshly-allocated
/// generation, matching the full-assignment path.
#[test]
fn merge_assignment_shares_one_generation_across_partitions() {
    let state = RebalanceState::new(false);

    // Seed one owned partition (generation 1).
    let mut initial = TopicPartitionList::new();
    let _ = initial.add_partition("traces", 0);
    state.set_assignment(&initial);

    // Merge a delta adding two partitions at once -> both share one new
    // generation (a single bump to 2).
    let mut delta = TopicPartitionList::new();
    let _ = delta.add_partition("traces", 1);
    let _ = delta.add_partition("traces", 2);
    state.merge_assignment(&delta);
    assert_eq!(state.current_generation("traces", 0), 1);
    assert_eq!(state.current_generation("traces", 1), 2);
    assert_eq!(state.current_generation("traces", 2), 2);
}

/// Scenario (consumer-group rebalancing): a partition is revoked and later reassigned
/// to this consumer via `set_assignment`.
/// Guarantees: the reacquired partition receives a strictly greater generation than its
/// prior ownership period, so stale acks cannot be mistaken for the new period.
#[test]
fn reacquired_partition_gets_greater_generation() {
    // A partition revoked and later reassigned to this consumer must get a
    // strictly greater generation than its prior ownership period.
    let state = RebalanceState::new(false);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    state.set_assignment(&tpl);
    let first = state.current_generation("traces", 0);

    // Revoke it (drops from the assigned set).
    {
        let mut assigned = state.lock_assigned();
        assert!(assigned.remove_partition("traces", 0));
    }

    // Reassign -> new, greater generation.
    state.set_assignment(&tpl);
    assert!(state.current_generation("traces", 0) > first);
}

/// Scenario (consumer-group rebalancing): the assignment-query-failure fallback merges
/// a cooperative delta that reports only a newly-gained partition.
/// Guarantees: retained partitions survive with their original generation and only the
/// new partition is added and counted, so the fallback never drops an owned partition.
#[test]
fn merge_assignment_adds_new_without_dropping_retained() {
    // The assignment-query-failure fallback must never drop a retained
    // partition: merging a cooperative delta only adds new partitions.
    let state = RebalanceState::new(false);

    let mut initial = TopicPartitionList::new();
    let _ = initial.add_partition("traces", 0);
    state.set_assignment(&initial);
    let g0 = state.current_generation("traces", 0);
    let _ = state.drain_metrics(); // reset counters

    // A delta reporting only the newly-gained partition 1.
    let mut delta = TopicPartitionList::new();
    let _ = delta.add_partition("traces", 1);
    state.merge_assignment(&delta);

    // Retained partition 0 survives with its original generation; partition
    // 1 is added with a fresh, strictly-greater generation.
    assert!(state.is_assigned("traces", 0));
    assert_eq!(state.current_generation("traces", 0), g0);
    assert!(state.is_assigned("traces", 1));
    assert!(state.current_generation("traces", 1) > g0);

    // Only the newly-added partition is counted.
    assert_eq!(state.drain_metrics().partition_assignments, 1);
}

/// Scenario (consumer-group rebalancing): the merge fallback is given a partition the
/// consumer already owns.
/// Guarantees: nothing changes -- no generation churn and no assignment count -- so
/// re-reporting an owned partition is inert.
#[test]
fn merge_assignment_is_noop_for_already_owned() {
    let state = RebalanceState::new(false);

    let mut initial = TopicPartitionList::new();
    let _ = initial.add_partition("traces", 0);
    state.set_assignment(&initial);
    let g0 = state.current_generation("traces", 0);
    let _ = state.drain_metrics();

    // Merging a partition we already own changes nothing.
    state.merge_assignment(&initial);
    assert_eq!(state.current_generation("traces", 0), g0);
    assert_eq!(state.drain_metrics().partition_assignments, 0);
}

/// Scenario (consumer-group rebalancing): revoked partitions are queued with their
/// ownership generation and then drained.
/// Guarantees: each drained revocation carries its generation tag, so the receive loop
/// can purge only same-or-older tracker state.
#[test]
fn drain_revoked_preserves_generation_tag() {
    // Revocations carry the generation of the ownership period being
    // revoked, so the receive loop can purge only same-or-older tracker
    // state. (handle_revoke stamps this; exercised end-to-end in the
    // receiver integration tests.)
    let state = RebalanceState::new(false);
    state.push_revoked_for_test("traces", 0, 1);
    state.push_revoked_for_test("traces", 1, 2);

    let mut drained = state.drain_revoked();
    drained.sort_by_key(|r| r.partition);
    assert_eq!(drained[0].partition, 0);
    assert_eq!(drained[0].generation, 1);
    assert_eq!(drained[1].partition, 1);
    assert_eq!(drained[1].generation, 2);
}

/// Scenario (consumer-group rebalancing): a new full assignment replaces a prior one
/// that owned different partitions.
/// Guarantees: the old partitions are dropped, the new ones owned, and the assignment
/// counter reflects only newly-acquired partitions across both assignments.
#[test]
fn set_assignment_replaces_and_counts_new_partitions() {
    let state = RebalanceState::new(false);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    let _ = tpl.add_partition("traces", 1);
    state.set_assignment(&tpl);

    assert!(state.is_assigned("traces", 0));
    assert!(state.is_assigned("traces", 1));

    // A new full assignment replaces the old one.
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("metrics", 0);
    state.set_assignment(&tpl2);

    assert!(!state.is_assigned("traces", 0));
    assert!(!state.is_assigned("traces", 1));
    assert!(state.is_assigned("metrics", 0));

    // 2 (initial) + 1 (metrics-0 is newly added).
    let delta = state.drain_metrics();
    assert_eq!(delta.partition_assignments, 3);
}

/// Scenario (consumer-group rebalancing): a cooperative-sticky rebalance stores the
/// full queried assignment that keeps one partition, drops another, and gains a third.
/// Guarantees: the retained partition stays assigned (its acks are not rejected as
/// revoked), so storing the full set -- not the delta -- preserves ownership.
#[test]
fn set_assignment_retains_partitions_across_cooperative_rebalance() {
    // Regression: under the cooperative-sticky protocol, post_rebalance
    // reports only the delta, but we store the full assignment queried from
    // the consumer. Simulate the full set the query would return.
    let state = RebalanceState::new(false);

    // Initially own partitions 0 and 1.
    let mut initial = TopicPartitionList::new();
    let _ = initial.add_partition("traces", 0);
    let _ = initial.add_partition("traces", 1);
    state.set_assignment(&initial);

    // After the rebalance: kept 0, dropped 1, gained 2. The full assignment
    // is {0, 2}.
    let mut full = TopicPartitionList::new();
    let _ = full.add_partition("traces", 0);
    let _ = full.add_partition("traces", 2);
    state.set_assignment(&full);

    // Retained partition 0 must still be assigned (previously this was
    // cleared, causing ACK/NACK for it to be rejected as revoked).
    assert!(state.is_assigned("traces", 0));
    assert!(state.is_assigned("traces", 2));
    assert!(!state.is_assigned("traces", 1));

    // Only partition 2 is newly added on the second assignment: 2 + 1.
    let delta = state.drain_metrics();
    assert_eq!(delta.partition_assignments, 3);
}

/// Scenario (consumer-group rebalancing): `set_assignment` counts only genuinely-new partitions while
/// retaining previously-owned ones.
/// Guarantees: the `partition_assignments` counter is bumped solely for
/// newly-acquired partitions (retained ones excluded) and current ownership
/// reflects the full set, so the metric never double-counts a retained
/// partition across cooperative rebalances.
#[test]
fn set_assignment_counts_only_newly_acquired_partitions() {
    let state = RebalanceState::new(false);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    state.set_assignment(&tpl);
    // First assignment acquires one partition.
    assert_eq!(state.drain_metrics().partition_assignments, 1);
    assert!(state.is_assigned("traces", 0));

    // Retaining 0 and gaining 1: only 1 is newly acquired.
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("traces", 0);
    let _ = tpl2.add_partition("traces", 1);
    state.set_assignment(&tpl2);
    assert_eq!(state.drain_metrics().partition_assignments, 1);
    assert!(state.is_assigned("traces", 0));
    assert!(state.is_assigned("traces", 1));
}

/// Scenario (consumer-group rebalancing): partitions are revoked down to an empty assignment.
/// Guarantees: `AssignedPartitions::len` and the drained `partitions_owned`
/// snapshot both reach zero, which is the signal the receiver uses to emit
/// the `kafka.assignment.became_empty` event.
#[test]
fn owned_count_reaches_zero_after_full_revocation() {
    let state = RebalanceState::new(false);
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    state.set_assignment(&tpl);
    assert_eq!(state.lock_assigned().len(), 1);

    // Simulate the revoke bookkeeping dropping the only owned partition.
    {
        let mut assigned = state.lock_assigned();
        assert!(assigned.remove_partition("traces", 0));
        assert_eq!(assigned.len(), 0);
    }
    assert_eq!(state.drain_metrics().partitions_owned, 0);
}

/// Scenario (consumer-group rebalancing): the revoked queue is drained while empty and
/// again after revocations are queued.
/// Guarantees: an empty drain returns nothing and a populated drain returns then clears
/// the queue, so revocations are delivered exactly once to the receive loop.
#[test]
fn drain_revoked_empty_then_populated() {
    let state = RebalanceState::new(false);
    assert!(state.drain_revoked().is_empty());

    state.push_revoked_for_test("traces", 0, 0);
    state.push_revoked_for_test("traces", 1, 0);
    let drained = state.drain_revoked();
    assert_eq!(drained.len(), 2);
    // Second drain is empty again.
    assert!(state.drain_revoked().is_empty());
}

/// Scenario: an immediate assignment resume fails once and the receive-loop retry succeeds.
/// Guarantees: the failed assignment remains scheduled with bounded backoff until a later
/// resume succeeds, after which no retry deadline remains to leave the partition stuck paused.
#[test]
fn assignment_resume_failure_is_retried_until_success() {
    let state = RebalanceState::new(false);
    state.assign_for_test("traces", 0, 7);
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    let consumer = ScriptedResumeConsumer::new(vec![
        Err(rdkafka::error::KafkaError::ClientCreation(
            "resume failed".to_string(),
        )),
        Ok(()),
    ]);

    state.resume_rebalance_partitions(&consumer, &tpl, ResumePhase::Assign);
    let retry_deadline = state
        .next_assignment_resume_deadline()
        .expect("failed assignment resume must schedule a retry");
    assert_eq!(consumer.attempts.load(Ordering::Relaxed), 1);
    assert_eq!(state.drain_metrics().rebalance_resume_errors, 1);

    state.process_due_assignment_resumes_at(&consumer, retry_deadline);

    assert_eq!(consumer.attempts.load(Ordering::Relaxed), 2);
    assert!(state.next_assignment_resume_deadline().is_none());
    assert_eq!(state.drain_metrics().rebalance_resume_errors, 0);
}

/// Scenario (consumer-group rebalancing): `handle_assign` runs against a live
/// consumer whose `assignment()` query succeeds, so it takes the full-query
/// (`Ok`) path rather than the merge fallback.
/// Guarantees: the queried full assignment is stored (the assigned partition
/// becomes owned with a real generation) and the rebalance/assignment
/// counters advance, so the cooperative full-`assignment()` dispatch records
/// the complete owned set.
#[tokio::test]
async fn handle_assign_query_path_stores_full_assignment() {
    const TOPIC: &str = "rebalance-handle-assign-traces";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
        |cluster| async move {
            let state = Arc::new(RebalanceState::new(false));
            let consumer = mock_base_consumer(&cluster, "handle-assign-group", Arc::clone(&state));

            // Assign the partition so the consumer's own `assignment()` query
            // returns a non-empty full set (the Ok dispatch path).
            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);
            consumer.assign(&tpl).expect("assign");

            state.handle_assign(&consumer, &tpl);

            assert!(
                state.is_assigned(TOPIC, 0),
                "the queried full assignment should be stored as owned",
            );
            assert!(
                state.current_generation(TOPIC, 0) >= 1,
                "a newly-acquired partition should get a real (>=1) generation",
            );
            let delta = state.drain_metrics();
            assert_eq!(delta.rebalances_total, 1, "one assign event counted");
            assert_eq!(
                delta.partition_assignments, 1,
                "one newly-acquired partition counted",
            );
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): `handle_revoke` runs against a live
/// consumer for a subset of currently-owned partitions, with a committable
/// snapshot in place.
/// Guarantees: the revoke is scoped to only the requested partitions -- they
/// are dropped from the assigned set, queued for tracker purge tagged with
/// their ownership generation, and counted in `partition_revocations`, while
/// a non-revoked owned partition stays assigned -- so a rebalance commits and
/// releases only the partitions being revoked.
#[tokio::test]
async fn handle_revoke_is_scoped_and_tags_generation() {
    const TOPIC: &str = "rebalance-handle-revoke-traces";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let state = Arc::new(RebalanceState::new(false));
            let consumer = mock_base_consumer(&cluster, "handle-revoke-group", Arc::clone(&state));

            // Own both partitions with a known generation.
            let mut full = TopicPartitionList::new();
            let _ = full.add_partition(TOPIC, 0);
            let _ = full.add_partition(TOPIC, 1);
            state.set_assignment(&full);
            let gen0 = state.current_generation(TOPIC, 0);
            assert!(gen0 >= 1);
            let _ = state.drain_metrics(); // reset counters after setup

            // A committable snapshot so the scoped commit has offsets to use.
            let mut snapshot = HashMap::new();
            let _ = snapshot.insert((TOPIC.to_string(), 0), 42_i64);
            let _ = snapshot.insert((TOPIC.to_string(), 1), 99_i64);
            state.set_committable_snapshot(snapshot);

            // Revoke only partition 0.
            let mut revoke = TopicPartitionList::new();
            let _ = revoke.add_partition(TOPIC, 0);
            state.handle_revoke(&consumer, &revoke);

            // Scoping: partition 0 released, partition 1 retained.
            assert!(
                !state.is_assigned(TOPIC, 0),
                "revoked partition 0 must be dropped from the assigned set",
            );
            assert!(
                state.is_assigned(TOPIC, 1),
                "non-revoked partition 1 must remain assigned",
            );

            // Revoked partition queued for purge, tagged with its generation.
            let drained = state.drain_revoked();
            assert_eq!(drained.len(), 1, "only the revoked partition is queued");
            assert_eq!(drained[0].partition, 0);
            assert_eq!(
                drained[0].generation, gen0,
                "the revocation carries the ownership generation being revoked",
            );

            // Only the genuinely-owned revoked partition is counted.
            assert_eq!(
                state.drain_metrics().partition_revocations,
                1,
                "exactly one owned partition was revoked",
            );
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): a partition is assigned, revoked,
/// then reassigned to the same consumer through the real
/// `handle_assign`/`handle_revoke` methods.
/// Guarantees: the reacquired partition receives a strictly greater ownership
/// generation than its prior period, so acks stamped with the old generation
/// cannot be mistaken for the new ownership after a return.
#[tokio::test]
async fn reacquired_partition_via_handle_gets_greater_generation() {
    const TOPIC: &str = "rebalance-handle-reacquire-traces";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
        |cluster| async move {
            let state = Arc::new(RebalanceState::new(false));
            let consumer =
                mock_base_consumer(&cluster, "handle-reacquire-group", Arc::clone(&state));

            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);

            // Assign -> generation g1.
            consumer.assign(&tpl).expect("assign");
            state.handle_assign(&consumer, &tpl);
            let g1 = state.current_generation(TOPIC, 0);
            assert!(g1 >= 1);

            // Revoke (drops from the assigned set; also unassign on the
            // consumer so the next query reflects the reacquisition).
            state.handle_revoke(&consumer, &tpl);
            consumer.unassign().expect("unassign");
            assert!(!state.is_assigned(TOPIC, 0));

            // Reassign -> strictly greater generation g2.
            consumer.assign(&tpl).expect("reassign");
            state.handle_assign(&consumer, &tpl);
            let g2 = state.current_generation(TOPIC, 0);
            assert!(
                g2 > g1,
                "a reacquired partition must get a strictly greater generation ({g2} > {g1})",
            );
        },
    )
    .await;
}

// ---- Construction and configuration ----

/// Scenario (construction and configuration): a `RebalanceState` is built with
/// auto-commit on and off.
/// Guarantees: `is_auto_commit()` reflects the constructor argument, so the rebalance
/// callbacks know whether to short-circuit.
#[test]
fn state_auto_commit_flag() {
    assert!(RebalanceState::new(true).is_auto_commit());
    assert!(!RebalanceState::new(false).is_auto_commit());
}

/// Scenario (construction and configuration): a `RebalanceState` is shared via `Arc`
/// across the callback thread and the receive loop.
/// Guarantees: the auto-commit flag is visible through every clone, so all holders
/// agree on whether manual commit handling is active.
#[test]
fn auto_commit_state_shared_across_arc() {
    let state = Arc::new(RebalanceState::new(true));
    let clone = Arc::clone(&state);
    assert!(clone.is_auto_commit());
}

// ---- Offset guarantees ----

/// Scenario (offset guarantees): a committable snapshot is set and a commit TPL is
/// built scoped to a subset of revoked partitions.
/// Guarantees: only the requested revoked partitions with a known committable offset
/// appear in the commit TPL, so a pre-revoke commit is scoped to exactly those
/// partitions.
#[test]
fn refresh_and_drain_committable_via_build() {
    let state = RebalanceState::new(false);
    let mut snapshot = HashMap::new();
    let _ = snapshot.insert(("traces".to_string(), 0), 100);
    let _ = snapshot.insert(("traces".to_string(), 1), 200);
    let _ = snapshot.insert(("metrics".to_string(), 0), 300);
    state.set_committable_snapshot(snapshot);

    // Only the requested revoked partitions appear in the commit TPL.
    let revoked = vec![("traces".to_string(), 0), ("metrics".to_string(), 0)];
    let committable = lock_ignore_poison(&state.committable);
    let tpl = build_commit_tpl(&committable, &revoked);
    assert_eq!(tpl.count(), 2);
    let map = tpl.to_topic_map();
    assert_eq!(
        map.get(&("traces".to_string(), 0)),
        Some(&Offset::Offset(100))
    );
    assert_eq!(
        map.get(&("metrics".to_string(), 0)),
        Some(&Offset::Offset(300))
    );
    // Partition 1 of traces was not in the revoked list.
    assert!(!map.contains_key(&("traces".to_string(), 1)));
}

/// Scenario (offset guarantees): a commit TPL is built for a revoked partition that has
/// no committable offset.
/// Guarantees: partitions without a committable offset are omitted from the TPL, so the
/// receiver never commits an offset it does not have.
#[test]
fn build_commit_tpl_skips_unknown_offsets() {
    let mut committable = HashMap::new();
    let _ = committable.insert(("traces".to_string(), 0), 100);
    // partition 5 has no committable offset
    let revoked = vec![("traces".to_string(), 0), ("traces".to_string(), 5)];
    let tpl = build_commit_tpl(&committable, &revoked);
    assert_eq!(tpl.count(), 1);
}

// ---- Operational visibility ----

/// Scenario (operational visibility): a full assignment is applied via `handle_assign`'s
/// `set_assignment` and then drained.
/// Guarantees: `partitions_owned` reports the current assignment size (an
/// absolute snapshot) even though it is delivered alongside counter deltas,
/// so `receiver.kafka.consumer.group.partitions` tracks live ownership.
#[test]
fn drain_metrics_reports_current_owned_count() {
    let state = RebalanceState::new(false);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    let _ = tpl.add_partition("traces", 1);
    state.set_assignment(&tpl);

    assert_eq!(state.drain_metrics().partitions_owned, 2);

    // A subsequent full assignment that shrinks ownership is reflected.
    let mut smaller = TopicPartitionList::new();
    let _ = smaller.add_partition("traces", 0);
    state.set_assignment(&smaller);
    assert_eq!(state.drain_metrics().partitions_owned, 1);
}

/// Scenario (operational visibility): each `handle_assign`-driven assignment increments the rebalance
/// event counter once.
/// Guarantees: `rebalances_total` counts assign events (not partitions), so
/// operators can distinguish rebalance frequency from partition churn.
#[test]
fn rebalances_total_counts_assign_events() {
    let state = RebalanceState::new(false);
    assert_eq!(state.drain_metrics().rebalances_total, 0);

    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    // set_assignment alone does not bump the event counter; handle_assign
    // does. Exercise the counter directly via the shared atomic path by
    // simulating two assign events.
    let _ = state.rebalances_total.fetch_add(1, Ordering::Relaxed);
    state.set_assignment(&tpl);
    let _ = state.rebalances_total.fetch_add(1, Ordering::Relaxed);

    assert_eq!(state.drain_metrics().rebalances_total, 2);
    assert_eq!(state.drain_metrics().rebalances_total, 0, "drain resets");
}

/// Scenario (operational visibility): a freshly-drained rebalance metrics delta with no
/// events is inspected.
/// Guarantees: `is_empty()` reports true only when every counter delta is zero, so the
/// receive loop skips redundant metric folds on idle ticks.
#[test]
fn metrics_delta_is_empty() {
    let state = RebalanceState::new(false);
    assert!(state.drain_metrics().is_empty());
}

/// Scenario (operational visibility): successful and failed commit outcomes are
/// recorded on the shared rebalance state.
/// Guarantees: successes increment `offset_commits` and failures increment
/// `offset_commit_errors`, so a commit failure is counted distinctly from a success.
#[test]
fn record_commit_result_counts_success_and_failure() {
    let state = RebalanceState::new(false);

    state.record_commit_result(&Ok(()));
    state.record_commit_result(&Ok(()));
    state.record_commit_result(&Err(rdkafka::error::KafkaError::ClientCreation(
        "boom".to_string(),
    )));

    let delta = state.drain_metrics();
    assert_eq!(delta.offset_commits, 2);
    assert_eq!(delta.offset_commit_errors, 1);

    // Draining resets the counters.
    let empty = state.drain_metrics();
    assert_eq!(empty.offset_commits, 0);
    assert_eq!(empty.offset_commit_errors, 0);
}

/// Scenario (operational visibility): a commit outcome is recorded and the metrics
/// delta is inspected.
/// Guarantees: the recorded outcome makes the delta non-empty, so the receive loop
/// folds commit metrics on the next reconcile rather than skipping them.
#[test]
fn commit_result_makes_delta_non_empty() {
    let state = RebalanceState::new(false);
    state.record_commit_result(&Ok(()));
    assert!(!state.drain_metrics().is_empty());
}

/// Scenario (operational visibility): commit outcomes are delivered through the real
/// `ConsumerContext::commit_callback`.
/// Guarantees: the callback folds one success and one failure into
/// `offset_commits`/`offset_commit_errors`, so the librdkafka wiring records async
/// commit outcomes correctly.
#[test]
fn commit_callback_folds_results_via_context() {
    // Exercise the callback through the real ConsumerContext impl to ensure
    // the wiring (auto-commit gate + state folding) is correct.
    let state = Arc::new(RebalanceState::new(false));
    let ctx = RebalancingConsumerContext::Default(Arc::clone(&state));
    let empty_tpl = TopicPartitionList::new();

    ctx.commit_callback(Ok(()), &empty_tpl);
    ctx.commit_callback(
        Err(rdkafka::error::KafkaError::ClientCreation(
            "boom".to_string(),
        )),
        &empty_tpl,
    );

    let delta = state.drain_metrics();
    assert_eq!(delta.offset_commits, 1);
    assert_eq!(delta.offset_commit_errors, 1);
}

/// Scenario (operational visibility): commit callbacks arrive while the receiver is in
/// auto-commit mode.
/// Guarantees: the callback records nothing (librdkafka owns offsets), so auto-commit
/// mode keeps the manual commit metrics clean.
#[test]
fn commit_callback_is_noop_in_auto_commit() {
    let state = Arc::new(RebalanceState::new(true));
    let ctx = RebalancingConsumerContext::Default(Arc::clone(&state));
    let empty_tpl = TopicPartitionList::new();

    ctx.commit_callback(Ok(()), &empty_tpl);
    ctx.commit_callback(
        Err(rdkafka::error::KafkaError::ClientCreation(
            "boom".to_string(),
        )),
        &empty_tpl,
    );

    let delta = state.drain_metrics();
    assert_eq!(delta.offset_commits, 0);
    assert_eq!(delta.offset_commit_errors, 0);
}

/// Scenario (operational visibility): partition tokens are appended to a structured-log buffer.
/// Guarantees: `PartitionListFmt` eagerly builds a stable, compact,
/// comma-separated `topic-partition` string (no leading/trailing comma, no
/// truncation marker when nothing was dropped) so assignment/revocation logs
/// are human- and machine-readable, and reports an accurate un-truncated
/// count/listed_count.
#[test]
fn append_partition_builds_comma_separated_list() {
    let mut fmt = PartitionListFmt::default();
    // An empty buffer is the empty string.
    assert_eq!(fmt.as_str(), "");
    assert_eq!(fmt.count(), 0);
    fmt.append("traces", 0);
    fmt.append("traces", 1);
    fmt.append("metrics", 3);
    assert_eq!(fmt.as_str(), "traces:0,traces:1,metrics:3");
    assert_eq!(fmt.count(), 3);
    assert_eq!(fmt.listed_count(), 3);
    // Nothing was dropped, so no truncation marker is present.
    assert!(!fmt.truncated());
    assert!(!fmt.as_str().ends_with("..."));
}

/// Scenario (operational visibility): more `topic-partition` tokens are appended than the entry cap
/// `MAX_LISTED_PARTITIONS` permits.
/// Guarantees: the rendered list is capped at `MAX_LISTED_PARTITIONS`
/// tokens, a single trailing `...` marker signals the drop, `truncated()`
/// flips to `true`, `listed_count()` equals the cap, and `count()` still
/// reports the full total -- so a huge rebalance never emits an unbounded log
/// line yet the true magnitude is preserved.
#[test]
fn partition_list_fmt_truncates_beyond_cap() {
    let mut fmt = PartitionListFmt::default();
    let total = MAX_LISTED_PARTITIONS + 10;
    for partition in 0..total {
        fmt.append("traces", partition as i32);
    }
    assert_eq!(fmt.count(), total);
    assert_eq!(fmt.listed_count(), MAX_LISTED_PARTITIONS);
    assert!(fmt.truncated());
    let rendered = fmt.as_str();
    // Truncation appends exactly one `...` marker.
    assert!(rendered.ends_with("..."), "rendered = {rendered}");
    assert_eq!(rendered.matches("...").count(), 1, "exactly one marker");
    // The rendered list contains exactly `MAX_LISTED_PARTITIONS` tokens plus
    // the marker: one comma between each of the cap tokens and one before
    // the marker == `MAX_LISTED_PARTITIONS` commas.
    let commas = rendered.matches(',').count() as u64;
    assert_eq!(commas, MAX_LISTED_PARTITIONS);
    // The last listed token is the one at index `MAX_LISTED_PARTITIONS - 1`;
    // nothing past the cap leaks in.
    assert!(rendered.contains(&format!("traces:{}", MAX_LISTED_PARTITIONS - 1)));
    assert!(!rendered.contains(&format!("traces:{MAX_LISTED_PARTITIONS}")));
}

/// Scenario (operational visibility): exactly the cap number of tokens are appended.
/// Guarantees: a list at the cap boundary is fully rendered with no trailing
/// `...` marker and `truncated()` stays `false`, so the marker has no
/// off-by-one at the boundary.
#[test]
fn partition_list_fmt_at_cap_is_not_truncated() {
    let mut fmt = PartitionListFmt::default();
    for partition in 0..MAX_LISTED_PARTITIONS {
        fmt.append("traces", partition as i32);
    }
    assert_eq!(fmt.count(), MAX_LISTED_PARTITIONS);
    assert_eq!(fmt.listed_count(), MAX_LISTED_PARTITIONS);
    assert!(!fmt.truncated());
    let rendered = fmt.as_str();
    assert!(!rendered.ends_with("..."), "rendered = {rendered}");
    // Exactly `MAX_LISTED_PARTITIONS` tokens => one fewer comma.
    let commas = rendered.matches(',').count() as u64;
    assert_eq!(commas, MAX_LISTED_PARTITIONS - 1);
}
