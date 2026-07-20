// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-offset tracking for Kafka consumer offset management.
//!
//! Tracks individual message offsets per topic-partition using a `BTreeSet`,
//! enabling correct at-least-once semantics even with out-of-order
//! acknowledgements. Only the lowest un-acknowledged offset is committed,
//! preventing offset skipping.
//!
//! Maintains an internal [`TopicPartitionList`] that is updated in-place on
//! each commit cycle, avoiding repeated allocation and FFI construction.
//!
//! Designed for single-threaded use on a `LocalSet` runtime — no internal
//! synchronization.

use rdkafka::Offset;
use rdkafka::topic_partition_list::TopicPartitionList;
use std::collections::{BTreeSet, HashMap};

/// Per-partition offset state.
struct PartitionTracker {
    /// Pending (un-acked) offsets. `BTreeSet` keeps them sorted so
    /// `first()` gives the lowest pending offset in O(log n).
    pending: BTreeSet<i64>,
    /// The highest acknowledged offset for this partition.
    /// Used as a commit fallback when all pending offsets have been cleared.
    high_water_mark: Option<i64>,
    /// Cached lowest pending offset from the last mutation.
    /// Used to detect when the committable watermark advances.
    last_lowest: Option<i64>,
    /// Assignment generation this partition's current state belongs to.
    ///
    /// Set from the generation current when the partition was (re)tracked.
    /// Used to distinguish ownership periods so a stale revocation cannot
    /// purge state created after the partition was reassigned to this consumer.
    generation: u64,
}

impl PartitionTracker {
    fn new(generation: u64) -> Self {
        Self {
            pending: BTreeSet::new(),
            high_water_mark: None,
            last_lowest: None,
            generation,
        }
    }

    /// Record an offset as pending (in-flight) under `generation`.
    ///
    /// The stored generation is advanced to `generation` when it is newer, so a
    /// partition reassigned to this consumer adopts the new ownership period.
    fn track(&mut self, offset: i64, generation: u64) {
        if generation > self.generation {
            self.generation = generation;
        }
        let _ = self.pending.insert(offset);
        // Update cached lowest if this is lower or first entry.
        match self.last_lowest {
            None => self.last_lowest = Some(offset),
            Some(prev) if offset < prev => self.last_lowest = Some(offset),
            _ => {}
        }
    }

    /// Mark an offset as acknowledged.
    ///
    /// Returns `true` if the lowest pending offset changed (i.e., the
    /// committable watermark advanced), signalling that a commit may be
    /// warranted.
    fn acknowledge(&mut self, offset: i64) -> bool {
        if !self.pending.remove(&offset) {
            // Offset was never tracked (or already acked) — no-op.
            return false;
        }

        // Update high-water mark.
        match self.high_water_mark {
            None => self.high_water_mark = Some(offset),
            Some(h) if offset > h => self.high_water_mark = Some(offset),
            _ => {}
        }

        // Check whether the lowest pending offset changed.
        let new_lowest = self.pending.first().copied();
        let advanced = new_lowest != self.last_lowest;
        self.last_lowest = new_lowest;
        advanced
    }

    /// The lowest un-acknowledged offset, if any.
    fn lowest_pending(&self) -> Option<i64> {
        self.last_lowest
    }

    /// The highest acknowledged offset.
    fn high_water_mark(&self) -> Option<i64> {
        self.high_water_mark
    }

    /// Check whether an offset is currently pending or has already been processed.
    ///
    /// Returns `true` if the offset is in the pending set (in-flight) or has
    /// already been acknowledged (`offset <= high_water_mark`).
    fn is_known(&self, offset: i64) -> bool {
        self.pending.contains(&offset) || self.high_water_mark.is_some_and(|hwm| offset <= hwm)
    }

    /// The offset that should be committed for this partition.
    ///
    /// Returns the lowest pending offset if any are in-flight, otherwise
    /// `high_water_mark + 1` if all offsets have been acknowledged.
    fn committable_offset(&self) -> Option<i64> {
        self.lowest_pending()
            .or_else(|| self.high_water_mark().map(|h| h + 1))
    }

    /// Number of pending (un-acked) offsets.
    #[cfg(test)]
    fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// Tracks offsets across all topic-partitions.
///
/// Keyed by topic name, then by partition. Maintains a persistent
/// [`TopicPartitionList`] (`tpl`) whose **partition membership mirrors the
/// tracked partition set**: [`track`](Self::track) registers a partition the
/// first time it is seen and [`revoke`](Self::revoke) rebuilds the list without
/// it. [`committable_tpl`](Self::committable_tpl) then only has to update
/// offsets in place each commit, avoiding per-commit reallocation and FFI
/// reconstruction.
///
/// The nested `HashMap` structure allows lookups via `&str` without
/// allocating an owned `String` on every call.
///
/// Single-threaded — no internal synchronization required.
pub struct OffsetTracker {
    partitions: HashMap<String, HashMap<i32, PartitionTracker>>,
    /// Persistent TPL reused across commits. Its partition membership is kept
    /// in sync with `partitions` by [`track`](Self::track) (adds) and
    /// [`revoke`](Self::revoke) (rebuilds); [`committable_tpl`](Self::committable_tpl)
    /// updates offsets in place.
    tpl: TopicPartitionList,
}

impl OffsetTracker {
    /// Create a new empty offset tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            tpl: TopicPartitionList::new(),
        }
    }

    /// Record a message offset as pending (in-flight).
    ///
    /// On the first sight of a `(topic, partition)` the partition is also
    /// registered in the persistent [`TopicPartitionList`], keeping its
    /// membership in sync with the tracked set so that
    /// [`committable_tpl`](Self::committable_tpl) can update offsets in place.
    ///
    /// Only allocates a `String` when a topic is seen for the first time;
    /// subsequent calls for the same topic use `&str` lookups.
    pub fn track(&mut self, topic: &str, partition: i32, offset: i64, generation: u64) {
        if let Some(partitions) = self.partitions.get_mut(topic) {
            // Known topic — zero allocation.
            let entry = partitions.entry(partition);
            if matches!(&entry, std::collections::hash_map::Entry::Vacant(_)) {
                // First sight of this partition — register it in the TPL.
                let _ = self.tpl.add_partition(topic, partition);
            }
            entry
                .or_insert_with(|| PartitionTracker::new(generation))
                .track(offset, generation);
        } else {
            // New topic — allocate once and register the partition in the TPL.
            let _ = self.tpl.add_partition(topic, partition);
            let mut tracker = PartitionTracker::new(generation);
            tracker.track(offset, generation);
            let mut partitions = HashMap::new();
            let _ = partitions.insert(partition, tracker);
            let _ = self.partitions.insert(topic.to_string(), partitions);
        }
    }

    /// The assignment generation the given partition's tracked state belongs to,
    /// or `None` if the partition is not tracked.
    #[must_use]
    pub fn partition_generation(&self, topic: &str, partition: i32) -> Option<u64> {
        self.partitions
            .get(topic)
            .and_then(|parts| parts.get(&partition))
            .map(|t| t.generation)
    }

    /// Revoke a partition only if its tracked state is *not newer* than
    /// `revoke_generation`.
    ///
    /// This is the generation-aware form of [`revoke`](Self::revoke). If the
    /// partition was reassigned to this consumer and re-tracked under a newer
    /// generation, a stale revocation (carrying an older generation) is a no-op,
    /// preserving the fresh tracking state. Returns `true` if the partition was
    /// removed.
    pub fn revoke_if_older(&mut self, topic: &str, partition: i32, revoke_generation: u64) -> bool {
        match self
            .partitions
            .get(topic)
            .and_then(|parts| parts.get(&partition))
        {
            // Tracked state belongs to a newer ownership period; keep it.
            Some(t) if t.generation > revoke_generation => false,
            Some(_) => {
                self.revoke(topic, partition);
                true
            }
            // Not tracked — nothing to revoke.
            None => false,
        }
    }

    /// Stop tracking a topic-partition, dropping all of its pending offsets and
    /// high-water-mark state.
    ///
    /// Called by the receive loop when a partition has been revoked during a
    /// consumer-group rebalance, so that the tracker no longer retains state
    /// (or attempts to commit offsets) for a partition this consumer no longer
    /// owns. Revoking an unknown topic-partition is a no-op.
    ///
    /// [`TopicPartitionList`] has no per-partition removal API, so the
    /// persistent `tpl` is rebuilt from the remaining tracked partitions. This
    /// only happens on the (rare) revoke path; steady-state commits update
    /// offsets in place.
    pub fn revoke(&mut self, topic: &str, partition: i32) {
        let Some(partitions) = self.partitions.get_mut(topic) else {
            // Unknown topic — nothing tracked, TPL already excludes it.
            return;
        };
        if partitions.remove(&partition).is_none() {
            // Unknown partition — TPL already excludes it.
            return;
        }
        if partitions.is_empty() {
            let _ = self.partitions.remove(topic);
        }
        self.rebuild_tpl();
    }

    /// Rebuild the persistent [`TopicPartitionList`] so its partition
    /// membership matches the currently tracked partitions.
    ///
    /// Offsets are materialized later by [`committable_tpl`](Self::committable_tpl);
    /// here we only need the partition entries to exist.
    fn rebuild_tpl(&mut self) {
        let mut tpl =
            TopicPartitionList::with_capacity(self.partitions.values().map(HashMap::len).sum());
        for (topic, partitions) in &self.partitions {
            for &partition in partitions.keys() {
                let _ = tpl.add_partition(topic, partition);
            }
        }
        self.tpl = tpl;
    }

    /// Acknowledge a message offset.
    ///
    /// Returns `true` if the lowest pending offset for this partition changed,
    /// indicating the committable watermark advanced.
    pub fn acknowledge(&mut self, topic: &str, partition: i32, offset: i64) -> bool {
        self.partitions
            .get_mut(topic)
            .and_then(|parts| parts.get_mut(&partition))
            .map(|tracker| tracker.acknowledge(offset))
            .unwrap_or(false)
    }

    /// Check whether an offset has already been seen for this topic+partition.
    ///
    /// Returns `true` if the offset is currently pending (in-flight) or has
    /// already been acknowledged (`offset <= high_water_mark`).
    #[must_use]
    pub fn is_known_offset(&self, topic: &str, partition: i32, offset: i64) -> bool {
        self.partitions
            .get(topic)
            .and_then(|parts| parts.get(&partition))
            .map(|tracker| tracker.is_known(offset))
            .unwrap_or(false)
    }

    /// Update the persistent [`TopicPartitionList`] with current committable
    /// offsets and return a reference suitable for passing to
    /// `consumer.commit()`.
    ///
    /// Offsets are updated **in place**: the TPL's partition membership already
    /// mirrors the tracked set (maintained by [`track`](Self::track) and
    /// [`revoke`](Self::revoke)), so revoked partitions are never present and
    /// no per-commit reallocation is needed. `set_partition_offset` targets the
    /// existing entry for each tracked partition.
    ///
    /// If no partitions are tracked the returned TPL is empty, which is safe to
    /// commit.
    pub fn committable_tpl(&mut self) -> &TopicPartitionList {
        for (topic, partitions) in &self.partitions {
            for (&partition, tracker) in partitions {
                if let Some(offset) = tracker.committable_offset() {
                    let _ = self
                        .tpl
                        .set_partition_offset(topic, partition, Offset::Offset(offset));
                }
            }
        }
        &self.tpl
    }

    /// Snapshot the committable offset for every tracked partition.
    ///
    /// Returns a map keyed by `(topic, partition)` to the offset that would be
    /// committed (lowest pending, or `high_water_mark + 1` once all offsets are
    /// acknowledged). Used to feed the shared rebalance state so that the
    /// pre-rebalance callback can commit owned partitions before they are
    /// revoked.
    #[must_use]
    pub fn committable_snapshot(&self) -> HashMap<(String, i32), i64> {
        let mut snapshot = HashMap::new();
        for (topic, partitions) in &self.partitions {
            for (&partition, tracker) in partitions {
                if let Some(offset) = tracker.committable_offset() {
                    let _ = snapshot.insert((topic.clone(), partition), offset);
                }
            }
        }
        snapshot
    }

    /// Number of pending offsets for a specific partition.
    #[cfg(test)]
    #[must_use]
    pub fn pending_count(&self, topic: &str, partition: i32) -> usize {
        self.partitions
            .get(topic)
            .and_then(|parts| parts.get(&partition))
            .map(|t| t.pending_count())
            .unwrap_or(0)
    }

    /// Total number of pending offsets across all partitions.
    #[cfg(test)]
    #[must_use]
    pub fn total_pending(&self) -> usize {
        self.partitions
            .values()
            .flat_map(|parts| parts.values())
            .map(|t| t.pending_count())
            .sum()
    }
}

impl Default for OffsetTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // ---- PartitionTracker tests ----

    #[test]
    fn partition_basic_track_and_ack() {
        let mut pt = PartitionTracker::new(0);

        pt.track(100, 0);
        pt.track(101, 0);
        pt.track(102, 0);

        assert_eq!(pt.pending_count(), 3);
        assert_eq!(pt.lowest_pending(), Some(100));

        // Ack the lowest — should advance.
        assert!(pt.acknowledge(100));
        assert_eq!(pt.pending_count(), 2);
        assert_eq!(pt.lowest_pending(), Some(101));
        assert_eq!(pt.high_water_mark(), Some(100));
    }

    #[test]
    fn partition_out_of_order_acks() {
        let mut pt = PartitionTracker::new(0);

        pt.track(100, 0);
        pt.track(101, 0);
        pt.track(102, 0);
        pt.track(103, 0);
        pt.track(104, 0);

        // Ack 102, 104 — lowest stays at 100.
        assert!(!pt.acknowledge(102));
        assert!(!pt.acknowledge(104));
        assert_eq!(pt.lowest_pending(), Some(100));

        // Ack 100 — lowest moves to 101.
        assert!(pt.acknowledge(100));
        assert_eq!(pt.lowest_pending(), Some(101));

        // Ack 101 — lowest moves to 103 (102 already acked).
        assert!(pt.acknowledge(101));
        assert_eq!(pt.lowest_pending(), Some(103));

        // Ack 103 — all clear.
        assert!(pt.acknowledge(103));
        assert_eq!(pt.lowest_pending(), None);
        assert_eq!(pt.pending_count(), 0);
        assert_eq!(pt.high_water_mark(), Some(104));
    }

    #[test]
    fn partition_duplicate_track_is_idempotent() {
        let mut pt = PartitionTracker::new(0);

        pt.track(100, 0);
        pt.track(100, 0);
        pt.track(100, 0);

        assert_eq!(pt.pending_count(), 1);
        assert!(pt.acknowledge(100));
        assert_eq!(pt.pending_count(), 0);
    }

    #[test]
    fn partition_ack_unknown_offset_is_noop() {
        let mut pt = PartitionTracker::new(0);

        pt.track(100, 0);
        // Ack a non-existent offset — nothing should change.
        assert!(!pt.acknowledge(999));
        assert_eq!(pt.pending_count(), 1);
        assert_eq!(pt.lowest_pending(), Some(100));
        // HWM must not be set by an untracked offset.
        assert_eq!(pt.high_water_mark(), None);
    }

    #[test]
    fn partition_high_water_mark_after_all_acked() {
        let mut pt = PartitionTracker::new(0);

        pt.track(100, 0);
        pt.track(101, 0);
        pt.track(102, 0);

        let _ = pt.acknowledge(100);
        let _ = pt.acknowledge(101);
        let _ = pt.acknowledge(102);

        assert_eq!(pt.lowest_pending(), None);
        assert_eq!(pt.high_water_mark(), Some(102));
    }

    // ---- OffsetTracker tests ----

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

        // Ack lowest — should advance.
        assert!(tracker.acknowledge("traces", 0, 100));
        let offsets = committable_sorted(&tracker);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 101));
    }

    #[test]
    fn tracker_out_of_order_acks() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100, 0);
        tracker.track("traces", 0, 101, 0);
        tracker.track("traces", 0, 102, 0);

        // Ack 102 first — should NOT advance (100 still pending).
        assert!(!tracker.acknowledge("traces", 0, 102));
        let offsets = committable_sorted(&tracker);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 100));

        // Ack 100 — advances to 101.
        assert!(tracker.acknowledge("traces", 0, 100));
        let offsets = committable_sorted(&tracker);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 101));

        // Ack 101 — all acked, commits hwm + 1.
        assert!(tracker.acknowledge("traces", 0, 101));
        let offsets = committable_sorted(&tracker);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 103)); // hwm=102, commit 103
    }

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

    #[test]
    fn tracker_empty_returns_no_committable() {
        let tracker = OffsetTracker::new();
        assert!(committable_sorted(&tracker).is_empty());
    }

    #[test]
    fn tracker_ack_unknown_partition_is_noop() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100, 0);
        // Ack for unknown topic/partition.
        assert!(!tracker.acknowledge("unknown", 99, 100));
        assert_eq!(tracker.pending_count("traces", 0), 1);
    }

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

    #[test]
    fn committable_tpl_returns_empty_when_no_partitions() {
        let mut tracker = OffsetTracker::new();
        let tpl = tracker.committable_tpl();
        assert_eq!(tpl.count(), 0);
    }

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

    #[test]
    fn committable_tpl_updates_in_place_after_ack() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100, 0);
        tracker.track("traces", 0, 101, 0);

        // Initial: committable is 100.
        let tpl = tracker.committable_tpl();
        let map = tpl.to_topic_map();
        assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(100));

        // Ack 100 → committable advances to 101.
        let _ = tracker.acknowledge("traces", 0, 100);
        let tpl = tracker.committable_tpl();
        let map = tpl.to_topic_map();
        assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(101));

        // Ack 101 → all acked, committable is hwm + 1 = 102.
        let _ = tracker.acknowledge("traces", 0, 101);
        let tpl = tracker.committable_tpl();
        let map = tpl.to_topic_map();
        assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(102));
    }

    // ---- revoke tests ----

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

    #[test]
    fn revoke_unknown_partition_is_noop() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);

        tracker.revoke("traces", 99);
        tracker.revoke("unknown", 0);

        assert_eq!(tracker.pending_count("traces", 0), 1);
    }

    #[test]
    fn revoke_dropping_last_partition_clears_topic() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);

        tracker.revoke("traces", 0);

        assert!(committable_sorted(&tracker).is_empty());
        assert_eq!(tracker.committable_tpl().count(), 0);
    }

    // ---- TPL membership invariant tests ----

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

        // Ack 100 → committable advances to 101, same TPL updated in place.
        let _ = tracker.acknowledge("traces", 0, 100);
        let map = tracker.committable_tpl().to_topic_map();
        assert_eq!(map[&("traces".to_string(), 0)], Offset::Offset(101));
        // No stale entries.
        assert_eq!(map.len(), 1);
    }

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

    // ---- committable_snapshot tests ----

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

    #[test]
    fn committable_snapshot_uses_hwm_after_all_acked() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);
        let _ = tracker.acknowledge("traces", 0, 100);

        let snap = tracker.committable_snapshot();
        // hwm = 100, commit 101.
        assert_eq!(snap.get(&("traces".to_string(), 0)), Some(&101));
    }

    #[test]
    fn committable_snapshot_empty_when_no_partitions() {
        let tracker = OffsetTracker::new();
        assert!(tracker.committable_snapshot().is_empty());
    }

    // ---- is_known_offset tests ----

    #[test]
    fn is_known_returns_false_for_unknown_partition() {
        let tracker = OffsetTracker::new();
        assert!(!tracker.is_known_offset("traces", 0, 100));
    }

    #[test]
    fn is_known_returns_true_for_pending_offset() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);
        tracker.track("traces", 0, 101, 0);

        assert!(tracker.is_known_offset("traces", 0, 100));
        assert!(tracker.is_known_offset("traces", 0, 101));
        assert!(!tracker.is_known_offset("traces", 0, 102));
    }

    #[test]
    fn is_known_returns_true_for_offset_at_or_below_hwm() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);
        tracker.track("traces", 0, 101, 0);

        // Ack both → hwm = 101
        let _ = tracker.acknowledge("traces", 0, 100);
        let _ = tracker.acknowledge("traces", 0, 101);

        // Offsets at or below hwm are known (already processed).
        assert!(tracker.is_known_offset("traces", 0, 99));
        assert!(tracker.is_known_offset("traces", 0, 100));
        assert!(tracker.is_known_offset("traces", 0, 101));
    }

    #[test]
    fn is_known_returns_false_for_offset_above_hwm() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 0);
        let _ = tracker.acknowledge("traces", 0, 100);

        // hwm = 100, offset 101 has never been seen.
        assert!(!tracker.is_known_offset("traces", 0, 101));
    }

    // ---- assignment-generation tests ----

    #[test]
    fn track_records_partition_generation() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100, 5);
        assert_eq!(tracker.partition_generation("traces", 0), Some(5));
        assert_eq!(tracker.partition_generation("traces", 9), None);
        assert_eq!(tracker.partition_generation("metrics", 0), None);
    }

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

    #[test]
    fn revoke_if_older_unknown_partition_is_noop() {
        let mut tracker = OffsetTracker::new();
        assert!(!tracker.revoke_if_older("traces", 0, 5));
    }
}
