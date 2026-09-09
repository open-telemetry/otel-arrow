// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-offset tracker tests: ack/nack ordering, committable watermark, and generation fencing.

use super::*;

// ---- Shared test helpers ----

/// Committable offsets as a deterministic, sorted `Vec`, derived from the
/// production [`OffsetTracker::committable_snapshot`]. Sorting makes
/// single-entry indexed assertions (`offsets[0]`) order-independent.
fn committable_sorted(tracker: &OffsetTracker) -> Vec<(String, i32, i64)> {
    let mut offsets: Vec<(String, i32, i64)> = tracker
        .committable_snapshot()
        .into_iter()
        .map(|((topic, partition), offset)| (topic, partition, offset))
        .collect();
    offsets.sort();
    offsets
}

/// Collect the `(topic, partition)` membership of the committable TPL.
fn tpl_membership(tracker: &mut OffsetTracker) -> BTreeSet<(String, i32)> {
    tracker
        .committable_tpl()
        .to_topic_map()
        .into_keys()
        .collect()
}

/// Collect the tracked `(topic, partition)` set from committable offsets.
fn tracked_membership(tracker: &OffsetTracker) -> BTreeSet<(String, i32)> {
    committable_sorted(tracker)
        .into_iter()
        .map(|(t, p, _)| (t, p))
        .collect()
}

// ---- Offset guarantees ----

/// Scenario (offset guarantees): a single partition tracks then acks a record.
/// Guarantees: the committable offset advances past the acked offset, so the basic
/// per-partition watermark works.
#[test]
fn partition_basic_track_and_ack() {
    let mut pt = PartitionTracker::new(0);

    assert_eq!(pt.track(100, 0), 1);
    assert_eq!(pt.track(101, 0), 1);
    assert_eq!(pt.track(102, 0), 1);

    assert_eq!(pt.pending_count(), 3);
    assert_eq!(pt.lowest_pending(), Some(100));

    // Ack the lowest -- should advance.
    assert_eq!(pt.acknowledge(100), (true, true));
    assert_eq!(pt.pending_count(), 2);
    assert_eq!(pt.lowest_pending(), Some(101));
    assert_eq!(pt.high_water_mark(), Some(100));
}

/// Scenario (offset guarantees): a partition acks records out of offset order.
/// Guarantees: the committable offset only advances to the lowest contiguous acked
/// offset, so out-of-order acks never skip an un-acked offset.
#[test]
fn partition_out_of_order_acks() {
    let mut pt = PartitionTracker::new(0);

    let _ = pt.track(100, 0);
    let _ = pt.track(101, 0);
    let _ = pt.track(102, 0);
    let _ = pt.track(103, 0);
    let _ = pt.track(104, 0);

    // Ack 102, 104 -- lowest stays at 100.
    assert!(!pt.acknowledge(102).1);
    assert!(!pt.acknowledge(104).1);
    assert_eq!(pt.lowest_pending(), Some(100));

    // Ack 100 -- lowest moves to 101.
    assert!(pt.acknowledge(100).1);
    assert_eq!(pt.lowest_pending(), Some(101));

    // Ack 101 -- lowest moves to 103 (102 already acked).
    assert!(pt.acknowledge(101).1);
    assert_eq!(pt.lowest_pending(), Some(103));

    // Ack 103 -- all clear.
    assert!(pt.acknowledge(103).1);
    assert_eq!(pt.lowest_pending(), None);
    assert_eq!(pt.pending_count(), 0);
    assert_eq!(pt.high_water_mark(), Some(104));
}

/// Scenario (offset guarantees): the same offset is tracked twice on a partition.
/// Guarantees: the duplicate track is a no-op, so re-tracking cannot corrupt the
/// pending set.
#[test]
fn partition_duplicate_track_is_idempotent() {
    let mut pt = PartitionTracker::new(0);

    assert_eq!(pt.track(100, 0), 1, "first insert adds one pending offset");
    assert_eq!(pt.track(100, 0), 0, "duplicate track is a no-op delta");
    assert_eq!(pt.track(100, 0), 0, "duplicate track is a no-op delta");

    assert_eq!(pt.pending_count(), 1);
    assert!(pt.acknowledge(100).1);
    assert_eq!(pt.pending_count(), 0);
}

/// Scenario (offset guarantees): an unknown offset is acked on a partition.
/// Guarantees: nothing changes, so a spurious ack cannot advance the watermark.
#[test]
fn partition_ack_unknown_offset_is_noop() {
    let mut pt = PartitionTracker::new(0);

    let _ = pt.track(100, 0);
    // Ack a non-existent offset -- nothing should change.
    assert_eq!(pt.acknowledge(999), (false, false));
    assert_eq!(pt.pending_count(), 1);
    assert_eq!(pt.lowest_pending(), Some(100));
    // HWM must not be set by an untracked offset.
    assert_eq!(pt.high_water_mark(), None);
}

/// Scenario (offset guarantees): every pending offset on a partition is acked.
/// Guarantees: the committable offset becomes the high-water mark, so a fully-drained
/// partition commits past its last record.
#[test]
fn partition_high_water_mark_after_all_acked() {
    let mut pt = PartitionTracker::new(0);

    let _ = pt.track(100, 0);
    let _ = pt.track(101, 0);
    let _ = pt.track(102, 0);

    let _ = pt.acknowledge(100);
    let _ = pt.acknowledge(101);
    let _ = pt.acknowledge(102);

    assert_eq!(pt.lowest_pending(), None);
    assert_eq!(pt.high_water_mark(), Some(102));
}

/// Scenario (offset guarantees): a partition tracks a record under a newer ownership
/// generation.
/// Guarantees: the pending state is reset for the new generation, so ownership periods
/// do not bleed offsets across a reassignment.
#[test]
fn partition_track_resets_state_on_newer_generation() {
    let mut pt = PartitionTracker::new(1);

    // Generation 1: own offsets 100..=104 and ack them all.
    for offset in 100..=104 {
        let _ = pt.track(offset, 1);
    }
    for offset in 100..=104 {
        let _ = pt.acknowledge(offset);
    }
    assert_eq!(pt.high_water_mark(), Some(104));
    assert_eq!(pt.committable_offset(), Some(105));

    // Generation 2 (reacquired): the first fetched offset is lower than the
    // prior high-water mark. The stale state must be discarded so the
    // committable offset follows the new ownership period, not the old HWM.
    // All pending were already acked (pending_count 0), so the reset delta
    // is 1 - 0 = 1.
    assert_eq!(pt.track(50, 2), 1);
    assert_eq!(pt.generation, 2);
    assert_eq!(pt.pending_count(), 1);
    assert_eq!(pt.high_water_mark(), None);
    assert_eq!(pt.committable_offset(), Some(50));
}

/// Scenario (offset guarantees): a partition tracks a record stamped with an older
/// generation than it currently holds.
/// Guarantees: the stale track is ignored, so a late record from a prior ownership
/// period cannot mutate current state.
#[test]
fn partition_track_ignores_stale_generation() {
    let mut pt = PartitionTracker::new(1);

    // Establish current ownership period (generation 2) with one offset.
    let _ = pt.track(200, 2);
    assert_eq!(pt.generation, 2);
    assert_eq!(pt.pending_count(), 1);
    assert_eq!(pt.lowest_pending(), Some(200));

    // A stale generation-1 track must not touch any current-period state,
    // and reports a zero delta.
    assert_eq!(pt.track(100, 1), 0);
    assert_eq!(
        pt.generation, 2,
        "stale track must not lower the generation"
    );
    assert_eq!(pt.pending_count(), 1, "stale offset must not be inserted");
    assert_eq!(
        pt.lowest_pending(),
        Some(200),
        "watermark must be unchanged"
    );
    assert_eq!(pt.high_water_mark(), None);
    assert_eq!(pt.committable_offset(), Some(200));
}

/// Scenario: A later in-flight offset is transiently NACKed while an earlier offset is unresolved.
/// Guarantees: Replay rewinds to the earliest pending offset and no old later ACK can advance the watermark.
#[test]
fn partition_replay_rewinds_to_earliest_unresolved_offset() {
    let mut pt = PartitionTracker::new(1);
    for offset in 100..=105 {
        let _ = pt.track(offset, 1);
    }
    let _ = pt.acknowledge(100);
    let _ = pt.acknowledge(102);
    let _ = pt.acknowledge(104);

    let (rewind, delta) = pt.prepare_replay(105).expect("failed offset is pending");

    assert_eq!(rewind, 101);
    assert_eq!(delta, -2);
    assert_eq!(pt.pending_count(), 1);
    assert_eq!(pt.committable_offset(), Some(101));
    assert_eq!(pt.high_water_mark(), Some(100));
    assert_eq!(pt.acknowledge(105), (false, false));
    assert_eq!(pt.committable_offset(), Some(101));
}

/// Scenario: A replay is prepared after later offsets were already acknowledged.
/// Guarantees: The old high-water mark is truncated so replay ACKs cannot commit past undelivered records.
#[test]
fn partition_replay_truncates_old_high_water_mark() {
    let mut pt = PartitionTracker::new(1);
    for offset in 10..=12 {
        let _ = pt.track(offset, 1);
    }
    let _ = pt.acknowledge(10);
    let _ = pt.acknowledge(12);

    let (rewind, _) = pt.prepare_replay(11).expect("failed offset is pending");
    assert_eq!(rewind, 11);
    assert_eq!(pt.high_water_mark(), Some(10));

    assert_eq!(pt.acknowledge(11), (true, true));
    assert_eq!(pt.committable_offset(), Some(12));
}

/// Scenario: A transient NACK rewinds one of two independently tracked partitions.
/// Guarantees: The failed partition stays at its rewind offset while the sibling can advance.
#[test]
fn tracker_replay_does_not_block_other_partitions() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 10, 1);
    tracker.track("traces", 1, 20, 1);

    assert_eq!(tracker.prepare_replay("traces", 0, 10, 1), Some(10));
    assert!(tracker.acknowledge("traces", 1, 20));

    assert_eq!(
        committable_sorted(&tracker),
        vec![("traces".to_string(), 0, 10), ("traces".to_string(), 1, 21),]
    );
}

/// Scenario (offset guarantees): the multi-partition tracker tracks and acks a record.
/// Guarantees: the committable TPL advances for that partition, so the tracker composes
/// per-partition watermarks correctly.
#[test]
fn tracker_basic_track_and_ack() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("traces", 0, 102, 0);

    assert_eq!(tracker.pending_count("traces", 0), 3);

    // Committable should be the lowest pending.
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets.len(), 1);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 100));

    // Ack lowest -- should advance.
    assert!(tracker.acknowledge("traces", 0, 100));
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 101));
}

/// Scenario (offset guarantees): records are acked out of order across the tracker.
/// Guarantees: the committable offset holds at the lowest un-acked offset, preserving
/// at-least-once across out-of-order completion.
#[test]
fn tracker_out_of_order_acks() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("traces", 0, 102, 0);

    // Ack 102 first -- should NOT advance (100 still pending).
    assert!(!tracker.acknowledge("traces", 0, 102));
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 100));

    // Ack 100 -- advances to 101.
    assert!(tracker.acknowledge("traces", 0, 100));
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 101));

    // Ack 101 -- all acked, commits hwm + 1.
    assert!(tracker.acknowledge("traces", 0, 101));
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 103)); // hwm=102, commit 103
}

/// Scenario (offset guarantees): records are tracked and acked across several
/// partitions.
/// Guarantees: each partition's committable offset advances independently, so
/// partitions do not interfere.
#[test]
fn tracker_multiple_partitions() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("traces", 1, 200, 0);
    tracker.track("traces", 1, 201, 0);
    tracker.track("metrics", 0, 300, 0);

    assert_eq!(tracker.pending_count("traces", 0), 2);
    assert_eq!(tracker.pending_count("traces", 1), 2);
    assert_eq!(tracker.pending_count("metrics", 0), 1);
    assert_eq!(tracker.total_pending(), 5);

    // Ack from different partitions.
    assert!(tracker.acknowledge("traces", 0, 100));
    assert!(tracker.acknowledge("traces", 1, 200));

    let sorted = committable_sorted(&tracker);
    assert_eq!(sorted.len(), 3);
    assert!(sorted.contains(&("metrics".to_string(), 0, 300)));
    assert!(sorted.contains(&("traces".to_string(), 0, 101)));
    assert!(sorted.contains(&("traces".to_string(), 1, 201)));
}

/// Scenario (offset guarantees): all tracked records across the tracker are acked.
/// Guarantees: each partition commits at its high-water mark, so fully-drained
/// partitions commit past their last record.
#[test]
fn tracker_all_acked_uses_high_water_mark() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    let _ = tracker.acknowledge("traces", 0, 100);
    let _ = tracker.acknowledge("traces", 0, 101);

    assert_eq!(tracker.pending_count("traces", 0), 0);

    // Should commit hwm + 1 = 102.
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets.len(), 1);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 102));
}

/// Scenario (offset guarantees): the tracker has no tracked records.
/// Guarantees: it reports nothing committable, so an idle tracker commits nothing.
#[test]
fn tracker_empty_returns_no_committable() {
    let tracker = OffsetTracker::new();
    assert!(committable_sorted(&tracker).is_empty());
}

/// Scenario (offset guarantees): an ack arrives for a partition the tracker does not
/// know.
/// Guarantees: nothing changes, so an ack for an untracked partition is safely ignored.
#[test]
fn tracker_ack_unknown_partition_is_noop() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    // Ack for unknown topic/partition.
    assert!(!tracker.acknowledge("unknown", 99, 100));
    assert_eq!(tracker.pending_count("traces", 0), 1);
}

/// Scenario (offset guarantees): a partition receives an interleaved ack/nack pattern.
/// Guarantees: the watermark advances only across contiguous completed offsets, so
/// mixed ack/nack still preserves at-least-once.
#[test]
fn tracker_mixed_ack_nack_pattern() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("traces", 0, 102, 0);
    tracker.track("traces", 0, 103, 0);
    tracker.track("traces", 0, 104, 0);

    // Simulate: ack 100, nack 102 (treated as ack), ack 101, nack 104.
    assert!(tracker.acknowledge("traces", 0, 100));
    assert!(!tracker.acknowledge("traces", 0, 102));
    assert!(tracker.acknowledge("traces", 0, 101));
    assert!(!tracker.acknowledge("traces", 0, 104));

    // 103 still pending.
    assert_eq!(tracker.pending_count("traces", 0), 1);
    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets[0], ("traces".to_string(), 0, 103));
}

/// Scenario (offset guarantees): records are tracked across multiple topics.
/// Guarantees: each (topic, partition) is tracked independently, so topics do not share
/// offset state.
#[test]
fn tracker_multiple_topics() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    tracker.track("metrics", 0, 200, 0);

    tracker.track("logs", 1, 300, 0);
    tracker.track("logs", 1, 301, 0);

    assert_eq!(tracker.total_pending(), 5);

    // Ack all of metrics.
    assert!(tracker.acknowledge("metrics", 0, 200));

    let sorted = committable_sorted(&tracker);

    assert_eq!(sorted.len(), 3);
    assert!(sorted.contains(&("logs".to_string(), 1, 300))); // lowest pending
    assert!(sorted.contains(&("metrics".to_string(), 0, 201))); // hwm + 1
    assert!(sorted.contains(&("traces".to_string(), 0, 100))); // lowest pending
}

/// Scenario (offset guarantees): the committable TPL is requested with nothing tracked.
/// Guarantees: an empty TPL is returned, so no commit is issued when there is nothing
/// to commit.
#[test]
fn committable_tpl_returns_empty_when_no_partitions() {
    let mut tracker = OffsetTracker::new();
    let tpl = tracker.committable_tpl();
    assert_eq!(tpl.count(), 0);
}

/// Scenario (offset guarantees): the committable TPL is requested with pending offsets.
/// Guarantees: the TPL carries each partition's committable offset, so the receiver
/// commits the correct positions.
#[test]
fn committable_tpl_returns_tpl_with_offsets() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    let tpl = tracker.committable_tpl();
    assert_eq!(tpl.count(), 1);

    let map = tpl.to_topic_map();
    assert_eq!(
        map.get(&("traces".to_string(), 0)),
        Some(&Offset::Offset(100)),
    );
}

/// Scenario (offset guarantees): the committable TPL is rebuilt after an ack advances a
/// watermark.
/// Guarantees: the TPL reflects the advanced offset, so successive commits move
/// forward.
#[test]
fn committable_tpl_updates_in_place_after_ack() {
    let mut tracker = OffsetTracker::new();

    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    // Initial: committable is 100.
    let tpl = tracker.committable_tpl();
    let map = tpl.to_topic_map();
    assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(100));

    // Ack 100 -> committable advances to 101.
    let _ = tracker.acknowledge("traces", 0, 100);
    let tpl = tracker.committable_tpl();
    let map = tpl.to_topic_map();
    assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(101));

    // Ack 101 -> all acked, committable is hwm + 1 = 102.
    let _ = tracker.acknowledge("traces", 0, 101);
    let tpl = tracker.committable_tpl();
    let map = tpl.to_topic_map();
    assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(102));
}

/// Scenario (offset guarantees): a partition is revoked from the tracker.
/// Guarantees: its pending state is dropped, so a revoked partition's offsets are no
/// longer committable.
#[test]
fn revoke_removes_pending_state() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("traces", 1, 200, 0);

    assert_eq!(tracker.total_pending(), 3);

    tracker.revoke("traces", 0);

    // Partition 0 state is gone; partition 1 remains.
    assert_eq!(tracker.pending_count("traces", 0), 0);
    assert_eq!(tracker.pending_count("traces", 1), 1);
    assert_eq!(tracker.total_pending(), 1);
}

/// Scenario (offset guarantees): a revoked partition is checked against the committable
/// TPL.
/// Guarantees: the revoked partition is absent from the TPL, so it is never committed
/// after revocation.
#[test]
fn revoke_excludes_partition_from_committable_tpl() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 1, 200, 0);

    // Before revoke: both partitions are committable.
    assert_eq!(tracker.committable_tpl().count(), 2);

    tracker.revoke("traces", 0);

    // After revoke: only partition 1 remains in the TPL.
    let tpl = tracker.committable_tpl();
    assert_eq!(tpl.count(), 1);
    let map = tpl.to_topic_map();
    assert!(!map.contains_key(&("traces".to_string(), 0)));
    assert_eq!(map[&("traces".to_string(), 1)], Offset::Offset(200));
}

/// Scenario (offset guarantees): a revoked partition is checked against the
/// committable-offsets snapshot.
/// Guarantees: the revoked partition is excluded, so the snapshot never reports a
/// revoked partition's offset.
#[test]
fn revoke_excludes_partition_from_committable_offsets() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("metrics", 0, 300, 0);

    tracker.revoke("traces", 0);

    let offsets = committable_sorted(&tracker);
    assert_eq!(offsets.len(), 1);
    assert_eq!(offsets[0], ("metrics".to_string(), 0, 300));
}

/// Scenario (offset guarantees): an unknown partition is revoked.
/// Guarantees: nothing changes, so a spurious revocation is safe.
#[test]
fn revoke_unknown_partition_is_noop() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);

    tracker.revoke("traces", 99);
    tracker.revoke("unknown", 0);

    assert_eq!(tracker.pending_count("traces", 0), 1);
}

/// Scenario (offset guarantees): the last tracked partition of a topic is revoked.
/// Guarantees: the topic entry is removed, so the tracker does not retain empty topic
/// state.
#[test]
fn revoke_dropping_last_partition_clears_topic() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);

    tracker.revoke("traces", 0);

    assert!(committable_sorted(&tracker).is_empty());
    assert_eq!(tracker.committable_tpl().count(), 0);
}

/// Scenario (offset guarantees): the committable TPL's membership is compared to the
/// tracked set after tracks and revokes.
/// Guarantees: the TPL membership exactly matches the tracked partitions, so the commit
/// set never drifts from tracked state.
#[test]
fn tpl_membership_matches_tracked_after_track_and_revoke() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 1, 200, 0);
    tracker.track("metrics", 0, 300, 0);

    assert_eq!(tpl_membership(&mut tracker), tracked_membership(&tracker));

    // Revoke one partition; membership must stay in sync.
    tracker.revoke("traces", 0);
    assert_eq!(tpl_membership(&mut tracker), tracked_membership(&tracker));

    // Revoke the last partition of a topic.
    tracker.revoke("metrics", 0);
    assert_eq!(tpl_membership(&mut tracker), tracked_membership(&tracker));
    let expected: BTreeSet<_> = [("traces".to_string(), 1)].into_iter().collect();
    assert_eq!(tpl_membership(&mut tracker), expected);
}

/// Scenario (offset guarantees): the committable TPL is queried repeatedly across a
/// sequence of acks.
/// Guarantees: each query reflects the latest advanced offsets in place, so commits
/// always use current watermarks.
#[test]
fn committable_tpl_updates_offsets_in_place_across_acks() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    // Initial committable offset is 100.
    assert_eq!(
        tracker.committable_tpl().to_topic_map()[&("traces".to_string(), 0)],
        Offset::Offset(100)
    );

    // Ack 100 -> committable advances to 101, same TPL updated in place.
    let _ = tracker.acknowledge("traces", 0, 100);
    let map = tracker.committable_tpl().to_topic_map();
    assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(101));
    // No stale entries.
    assert_eq!(map.len(), 1);
}

/// Scenario (offset guarantees): a partition is revoked and the TPL is queried
/// afterwards.
/// Guarantees: the revoked partition never reappears in the TPL, protecting the
/// invariant that revoked partitions are not committed.
#[test]
fn revoked_partition_never_reappears_in_tpl() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 1, 200, 0);
    tracker.revoke("traces", 0);

    // Tracking a *different* partition must not resurrect the revoked one.
    tracker.track("traces", 1, 201, 0);
    let map = tracker.committable_tpl().to_topic_map();
    assert!(!map.contains_key(&("traces".to_string(), 0)));
    assert!(map.contains_key(&("traces".to_string(), 1)));
}

/// Scenario (offset guarantees): a previously-revoked partition is tracked again
/// (reassigned).
/// Guarantees: it re-registers in the committable TPL, so a reacquired partition
/// resumes committing.
#[test]
fn retrack_revoked_partition_re_registers_in_tpl() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.revoke("traces", 0);
    assert_eq!(tracker.committable_tpl().count(), 0);

    // A partition can be reassigned later; re-tracking must re-register it.
    tracker.track("traces", 0, 150, 0);
    let map = tracker.committable_tpl().to_topic_map();
    assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(150));
}

/// Scenario (offset guarantees): a committable snapshot is taken with pending offsets.
/// Guarantees: the snapshot reports each partition's lowest un-acked offset, so
/// pre-rebalance commits use the safe watermark.
#[test]
fn committable_snapshot_reflects_lowest_pending() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);
    tracker.track("metrics", 1, 200, 0);

    let snap = tracker.committable_snapshot();
    assert_eq!(snap.get(&("traces".to_string(), 0)), Some(&100));
    assert_eq!(snap.get(&("metrics".to_string(), 1)), Some(&200));
}

/// Scenario (offset guarantees): a committable snapshot is taken after all records are
/// acked.
/// Guarantees: the snapshot reports the high-water mark, so a fully-drained partition's
/// snapshot commits past its last record.
#[test]
fn committable_snapshot_uses_hwm_after_all_acked() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    let _ = tracker.acknowledge("traces", 0, 100);

    let snap = tracker.committable_snapshot();
    // hwm = 100, commit 101.
    assert_eq!(snap.get(&("traces".to_string(), 0)), Some(&101));
}

/// Scenario (offset guarantees): a committable snapshot is taken with nothing tracked.
/// Guarantees: the snapshot is empty, so no stale offsets feed a pre-rebalance commit.
#[test]
fn committable_snapshot_empty_when_no_partitions() {
    let tracker = OffsetTracker::new();
    assert!(tracker.committable_snapshot().is_empty());
}

/// Scenario (offset guarantees): a record is tracked with an ownership generation.
/// Guarantees: the partition's tracked generation is recorded, so acks can be matched
/// to their ownership period.
#[test]
fn track_records_partition_generation() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 5);
    assert_eq!(tracker.partition_generation("traces", 0), Some(5));
    assert_eq!(tracker.partition_generation("traces", 9), None);
    assert_eq!(tracker.partition_generation("metrics", 0), None);
}

/// Scenario (offset guarantees): a record is tracked under a newer generation than the
/// partition currently holds.
/// Guarantees: the tracked generation advances, so state follows the latest ownership
/// period.
#[test]
fn track_advances_partition_generation_when_newer() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 1);
    // Re-tracking the same partition under a newer generation adopts it.
    tracker.track("traces", 0, 101, 3);
    assert_eq!(tracker.partition_generation("traces", 0), Some(3));
    // An older generation does not regress the stored value.
    tracker.track("traces", 0, 102, 2);
    assert_eq!(tracker.partition_generation("traces", 0), Some(3));
}

/// Scenario (offset guarantees): a generation-aware revoke targets a partition at the
/// same or an older generation than tracked.
/// Guarantees: the state is removed, so a revocation of the current-or-prior ownership
/// period purges correctly.
#[test]
fn revoke_if_older_removes_same_or_older_generation() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 2);

    // Revocation from an older period: no-op.
    assert!(!tracker.revoke_if_older("traces", 0, 1));
    assert_eq!(tracker.pending_count("traces", 0), 1);

    // Revocation from the same period: removes.
    assert!(tracker.revoke_if_older("traces", 0, 2));
    assert_eq!(tracker.pending_count("traces", 0), 0);
}

/// Scenario (offset guarantees): a generation-aware revoke targets a partition whose
/// tracked generation is newer than the revocation.
/// Guarantees: the newer state is preserved, so a stale revocation cannot drop
/// freshly-reacquired state.
#[test]
fn revoke_if_older_preserves_newer_generation_state() {
    // Regression for the revoke/reassign race: a stale revocation
    // (generation 1) must not delete state re-tracked under generation 2
    // after the partition was reassigned to this consumer.
    let mut tracker = OffsetTracker::new();

    // Ownership period 1 tracked, then partition revoked (queued as gen 1).
    tracker.track("traces", 0, 100, 1);
    // Partition reassigned; a new record is tracked under generation 2.
    tracker.track("traces", 0, 250, 2);
    assert_eq!(tracker.partition_generation("traces", 0), Some(2));

    // The stale generation-1 revocation is now applied: it must be a no-op.
    assert!(!tracker.revoke_if_older("traces", 0, 1));
    assert!(tracker.is_known_offset("traces", 0, 250));
    assert_eq!(tracker.partition_generation("traces", 0), Some(2));
}

/// Scenario (offset guarantees): a generation-aware revoke targets an unknown
/// partition.
/// Guarantees: nothing changes, so a stale revocation of an untracked partition is
/// safe.
#[test]
fn revoke_if_older_unknown_partition_is_noop() {
    let mut tracker = OffsetTracker::new();
    assert!(!tracker.revoke_if_older("traces", 0, 5));
}

/// Scenario (offset guarantees): a partition with pending offsets and a high-water mark
/// is revoked.
/// Guarantees: both the pending set and the high-water mark are dropped, so no residual
/// state can be committed for a revoked partition.
#[test]
fn revoke_drops_all_pending_and_hwm() {
    let mut tracker = OffsetTracker::new();

    // Track 5,6,7 under generation 1 and ack 5 so there is both pending
    // state (6,7) and a high-water mark (5).
    for offset in 5..=7 {
        tracker.track("traces", 0, offset, 1);
    }
    let _ = tracker.acknowledge("traces", 0, 5);
    assert_eq!(tracker.pending_count("traces", 0), 2);
    assert_eq!(
        committable_sorted(&tracker),
        vec![("traces".to_string(), 0, 6)],
        "the committable offset is the lowest pending (6) before revoke",
    );

    // Revoke the partition: all of its state must be gone.
    tracker.revoke("traces", 0);
    assert_eq!(tracker.pending_count("traces", 0), 0);
    assert_eq!(tracker.partition_generation("traces", 0), None);
    assert!(
        !tracker
            .committable_snapshot()
            .contains_key(&("traces".to_string(), 0)),
        "a revoked partition contributes no committable offset",
    );
}

/// Scenario (offset guarantees): a partition is revoked and reassigned, then acked
/// under the new generation.
/// Guarantees: only the new generation's offset is committable, so an old-generation
/// ack cannot advance the reassigned partition.
#[test]
fn revoke_reassign_commits_only_new_generation() {
    let mut tracker = OffsetTracker::new();

    // Generation 1: own partition 0, track and ack offsets 100..=104. The
    // committable offset is high_water_mark + 1 = 105.
    for offset in 100..=104 {
        tracker.track("traces", 0, offset, 1);
    }
    for offset in 100..=104 {
        let _ = tracker.acknowledge("traces", 0, offset);
    }
    assert_eq!(
        committable_sorted(&tracker),
        vec![("traces".to_string(), 0, 105)],
        "generation 1 commits its own high-water mark",
    );

    // Partition 0 is revoked (revocation carries generation 1). Its state
    // is purged, so nothing is committable for it anymore.
    assert!(tracker.revoke_if_older("traces", 0, 1));
    assert!(
        committable_sorted(&tracker).is_empty(),
        "a revoked partition contributes no committable offset",
    );

    // Generation 2: partition 0 is reassigned to this consumer. It resumes
    // from the group's committed position (200), lower than generation 1's
    // high-water mark, and a single new record is tracked.
    tracker.track("traces", 0, 200, 2);
    assert_eq!(
        tracker.partition_generation("traces", 0),
        Some(2),
        "the reassigned partition adopts generation 2",
    );

    // A stale generation-1 ack that arrives after reassignment targets an
    // offset the generation-2 state has never seen; it must be a no-op and
    // must not advance or roll back the generation-2 committable offset.
    assert!(
        !tracker.acknowledge("traces", 0, 104),
        "a stale generation-1 offset is not pending under generation 2",
    );

    // The committable offset reflects only the generation-2 record (200),
    // never generation 1's 105.
    assert_eq!(
        committable_sorted(&tracker),
        vec![("traces".to_string(), 0, 200)],
        "only generation-2 records drive the commit after reassignment",
    );
}

// ---- Routing and payload correctness ----

/// Scenario (routing and payload correctness): `is_known_offset` is queried for a
/// partition the tracker has never seen.
/// Guarantees: it returns false, so a record on an unknown partition is not treated as
/// a duplicate.
#[test]
fn is_known_returns_false_for_unknown_partition() {
    let tracker = OffsetTracker::new();
    assert!(!tracker.is_known_offset("traces", 0, 100));
}

/// Scenario (routing and payload correctness): `is_known_offset` is queried for an
/// offset currently pending.
/// Guarantees: it returns true, so a redelivered pending offset is recognized as
/// already-seen (idempotency dedupe).
#[test]
fn is_known_returns_true_for_pending_offset() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    assert!(tracker.is_known_offset("traces", 0, 100));
    assert!(tracker.is_known_offset("traces", 0, 101));
    assert!(!tracker.is_known_offset("traces", 0, 102));
}

/// Scenario (routing and payload correctness): `is_known_offset` is queried for an
/// offset at or below the high-water mark.
/// Guarantees: it returns true, so an offset already processed is recognized as a
/// duplicate.
#[test]
fn is_known_returns_true_for_offset_at_or_below_hwm() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    tracker.track("traces", 0, 101, 0);

    // Ack both -> hwm = 101
    let _ = tracker.acknowledge("traces", 0, 100);
    let _ = tracker.acknowledge("traces", 0, 101);

    // Offsets at or below hwm are known (already processed).
    assert!(tracker.is_known_offset("traces", 0, 99));
    assert!(tracker.is_known_offset("traces", 0, 100));
    assert!(tracker.is_known_offset("traces", 0, 101));
}

/// Scenario (routing and payload correctness): `is_known_offset` is queried for an
/// offset above the high-water mark.
/// Guarantees: it returns false, so a genuinely new offset is not skipped as a
/// duplicate.
#[test]
fn is_known_returns_false_for_offset_above_hwm() {
    let mut tracker = OffsetTracker::new();
    tracker.track("traces", 0, 100, 0);
    let _ = tracker.acknowledge("traces", 0, 100);

    // hwm = 100, offset 101 has never been seen.
    assert!(!tracker.is_known_offset("traces", 0, 101));
}

/// Scenario (routing and payload correctness): a partition's offsets are
/// tracked and acked under generation 1 (so they are "known" via the pending
/// set and high-water mark), then a record of generation 2 is tracked for the
/// same partition -- the in-place reset path taken when a partition is
/// reassigned to this consumer under a newer generation.
/// Guarantees: the newer-generation `track` clears the old pending set and
/// high-water mark, so the old-generation offsets are no longer reported as
/// known. A redelivered old offset after a generation bump is therefore
/// treated as new (reprocessed) rather than idempotently skipped, while the
/// new-generation offset is known. This proves the idempotency dedupe memory
/// is correctly cleared by a generation change (via the `track` reset path).
#[test]
fn is_known_offset_false_for_old_offset_after_newer_generation_track() {
    let mut tracker = OffsetTracker::new();

    // Generation 1: track and ack 100, 101 so both are "known" (100 <= hwm,
    // 101 <= hwm after acking).
    tracker.track("traces", 0, 100, 1);
    tracker.track("traces", 0, 101, 1);
    let _ = tracker.acknowledge("traces", 0, 100);
    let _ = tracker.acknowledge("traces", 0, 101);
    assert!(tracker.is_known_offset("traces", 0, 100));
    assert!(tracker.is_known_offset("traces", 0, 101));

    // Generation 2: the partition was revoked and reassigned; a new record is
    // tracked under the newer generation, which resets the partition state.
    tracker.track("traces", 0, 200, 2);
    assert_eq!(tracker.partition_generation("traces", 0), Some(2));

    // The old-generation offsets are no longer known: their pending/hwm state
    // was cleared, so a redelivery of them would be reprocessed, not skipped.
    assert!(
        !tracker.is_known_offset("traces", 0, 100),
        "an old-generation offset must not remain known after a newer-generation track",
    );
    assert!(!tracker.is_known_offset("traces", 0, 101));
    assert!(!tracker.is_known_offset("traces", 0, 50));

    // The new-generation offset is known (it is pending under generation 2).
    assert!(tracker.is_known_offset("traces", 0, 200));
}

/// Scenario (routing and payload correctness): a partition's offsets are
/// tracked and acked under generation 1 (so they are "known"), the partition
/// is then revoked via the generation-aware purge (`revoke_if_older`), and
/// finally reassigned with a new record tracked under generation 2.
/// Guarantees: the revoke purges the partition's entire state (pending set and
/// high-water mark), so the old-generation offsets are no longer known; after
/// reassignment only the new-generation offset is known. This proves the
/// idempotency dedupe memory is also cleared by the revoke/reassign purge
/// path, so an old offset redelivered after a new generation is reprocessed
/// rather than skipped.
#[test]
fn is_known_offset_false_for_old_offset_after_revoke_reassign() {
    let mut tracker = OffsetTracker::new();

    // Generation 1: track and ack 100..=104 so they are known.
    for offset in 100..=104 {
        tracker.track("traces", 0, offset, 1);
    }
    for offset in 100..=104 {
        let _ = tracker.acknowledge("traces", 0, offset);
    }
    assert!(tracker.is_known_offset("traces", 0, 104));

    // The partition is revoked (revocation carries generation 1). The purge
    // removes all of its state, including the known-offset memory.
    assert!(tracker.revoke_if_older("traces", 0, 1));
    assert!(
        !tracker.is_known_offset("traces", 0, 104),
        "a revoked partition's offsets must no longer be known",
    );

    // Generation 2: the partition is reassigned and a new record is tracked.
    tracker.track("traces", 0, 200, 2);
    assert!(
        !tracker.is_known_offset("traces", 0, 104),
        "an old-generation offset must not become known again after reassignment",
    );
    assert!(tracker.is_known_offset("traces", 0, 200));
}

/// Scenario (routing and payload correctness): an offset is tracked and acked
/// under generation 1 (so it is a known duplicate within that generation),
/// and `is_known_offset_for_generation` is then queried with the same offset
/// under an older, equal, and newer generation.
/// Guarantees: within the same (or an older) ownership generation the offset
/// is still reported as known (idempotent dedupe applies within a
/// generation), but under a NEWER generation it is reported as NOT known --
/// so a message redelivered under a new ownership period (same offset, newer
/// generation after a revoke+reassign) is never skipped as a duplicate and is
/// reprocessed instead. This is the generation-aware idempotency contract.
#[test]
fn is_known_offset_for_generation_allows_newer_generation_same_offset() {
    let mut tracker = OffsetTracker::new();

    // Generation 1: track+ack offset 100 so it is "known" within generation 1.
    tracker.track("traces", 0, 100, 1);
    let _ = tracker.acknowledge("traces", 0, 100);
    assert_eq!(tracker.partition_generation("traces", 0), Some(1));

    // Same generation: the offset is a known duplicate (idempotent dedupe
    // applies within the ownership period).
    assert!(
        tracker.is_known_offset_for_generation("traces", 0, 100, 1),
        "offset 100 must be known within its own generation (same-period dedupe)",
    );

    // Newer generation: the same offset belongs to a new ownership period and
    // must NOT be treated as a known duplicate -- it is allowed through so the
    // new owner reprocesses it.
    assert!(
        !tracker.is_known_offset_for_generation("traces", 0, 100, 2),
        "offset 100 under a newer generation must not be known (reprocessed)",
    );
    assert!(
        !tracker.is_known_offset_for_generation("traces", 0, 100, 5),
        "any generation newer than the tracked one makes the offset unknown",
    );

    // An untracked partition is never known, regardless of generation.
    assert!(!tracker.is_known_offset_for_generation("traces", 9, 100, 1));
    assert!(!tracker.is_known_offset_for_generation("traces", 9, 100, 2));
}

// ---- Aggregate in-flight (total_pending) ----

/// O(n) reference rescan of the aggregate pending count, summed directly
/// from every partition's pending set. This is the linear-scan baseline the
/// production O(1) [`OffsetTracker::total_pending`] cache must always agree
/// with; it is defined only in tests so the production path never rescans.
fn scan_total_pending(tracker: &OffsetTracker) -> usize {
    tracker
        .partitions
        .values()
        .flat_map(|parts| parts.values())
        .map(PartitionTracker::pending_count)
        .sum()
}

/// Assert the O(1) cached aggregate equals the O(n) full rescan and both
/// equal `expected`.
///
/// This is the core behavior-preserving check: the constant-time counter
/// returned by [`OffsetTracker::total_pending`] must always match a fresh
/// linear scan over every partition's pending set.
fn assert_in_flight_agrees(tracker: &OffsetTracker, expected: usize) {
    let full_scan = scan_total_pending(tracker); // O(n) over all partitions.
    let constant_time = tracker.total_pending(); // O(1) cached aggregate.
    assert_eq!(
        full_scan, expected,
        "O(n) full scan diverged from the expected in-flight count",
    );
    assert_eq!(
        constant_time, full_scan,
        "O(1) cached total_pending diverged from the O(n) full scan",
    );
}

/// Scenario (runtime and performance): the aggregate in-flight count is
/// driven through track, duplicate track, stale-generation track, ack,
/// unknown ack, revoke, newer-generation reset, and a full drain to zero.
/// Guarantees: after every mutation the O(1) cached `total_pending` exactly
/// equals an O(n) full rescan of all partitions and the expected value, so
/// the constant-time counter never drifts from the true in-flight depth and
/// the metric stays behavior-preserving relative to a full rescan.
#[test]
fn total_pending_cache_matches_scan_across_mutations() {
    let mut tracker = OffsetTracker::new();

    // in flight: {} => total 0
    assert_in_flight_agrees(&tracker, 0);

    // Fresh inserts across two topics/partitions raise the aggregate.
    // in flight: traces/0={100,101}, traces/1={200}, metrics/0={300} => total 4
    tracker.track("traces", 0, 100, 1);
    tracker.track("traces", 0, 101, 1);
    tracker.track("traces", 1, 200, 1);
    tracker.track("metrics", 0, 300, 1);
    assert_in_flight_agrees(&tracker, 4);

    // A duplicate offset does not change the aggregate.
    // duplicate insert, in flight unchanged => total 4
    tracker.track("traces", 0, 100, 1);
    assert_in_flight_agrees(&tracker, 4);

    // A stale-generation track is a no-op for the aggregate.
    // stale-gen no-op, in flight unchanged => total 4
    tracker.track("traces", 0, 50, 0);
    assert_in_flight_agrees(&tracker, 4);

    // Acking a pending offset decrements by one.
    // in flight: traces/0={101}, traces/1={200}, metrics/0={300} => total 3
    assert!(tracker.acknowledge("traces", 0, 100));
    assert_in_flight_agrees(&tracker, 3);

    // Acking an unknown offset/partition leaves the aggregate unchanged.
    // unknown acks are no-ops, in flight unchanged => total 3
    assert!(!tracker.acknowledge("traces", 0, 999));
    assert!(!tracker.acknowledge("unknown", 7, 1));
    assert_in_flight_agrees(&tracker, 3);

    // Revoking a partition subtracts its remaining pending offsets.
    // traces/0 still has offset 101 pending (100 was acked).
    // drop traces/0={101}; in flight: traces/1={200}, metrics/0={300} => total 2
    tracker.revoke("traces", 0);
    assert_in_flight_agrees(&tracker, 2);

    // A newer-generation track on an existing partition resets its pending
    // set: metrics/0 drops its single old-generation offset (300) and adds
    // one new offset (400), a net zero change for that partition.
    // reset metrics/0 {300}->{400}; in flight: traces/1={200}, metrics/0={400} => total 2
    tracker.track("metrics", 0, 400, 2);
    assert_in_flight_agrees(&tracker, 2);

    // Grow traces/1 to two generation-1 pending offsets (200, 201) so the
    // upcoming newer-generation reset must remove more than it adds,
    // exercising the negative-delta path (delta = 1 - old_len = 1 - 2 = -1).
    // in flight: traces/1={200,201}, metrics/0={400} => total 3
    tracker.track("traces", 1, 201, 1);
    assert_in_flight_agrees(&tracker, 3);

    // A newer-generation track on traces/1 clears its two pending offsets
    // (200, 201) and inserts one (210): net -1, so the aggregate shrinks.
    // reset traces/1 {200,201}->{210}; in flight: traces/1={210}, metrics/0={400} => total 2
    tracker.track("traces", 1, 210, 2);
    assert_in_flight_agrees(&tracker, 2);

    // Drain everything to zero via acks; the cache must land exactly at 0.
    // ack metrics/0 400 and traces/1 210; in flight: {} => total 0
    assert!(tracker.acknowledge("metrics", 0, 400));
    assert!(tracker.acknowledge("traces", 1, 210));
    assert_in_flight_agrees(&tracker, 0);
}
