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
}

impl PartitionTracker {
    fn new() -> Self {
        Self {
            pending: BTreeSet::new(),
            high_water_mark: None,
            last_lowest: None,
        }
    }

    /// Record an offset as pending (in-flight).
    fn track(&mut self, offset: i64) {
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
/// [`TopicPartitionList`] that is updated in-place for efficient commits.
///
/// The nested `HashMap` structure allows lookups via `&str` without
/// allocating an owned `String` on every call.
///
/// Single-threaded — no internal synchronization required.
pub struct OffsetTracker {
    partitions: HashMap<String, HashMap<i32, PartitionTracker>>,
    /// Persistent TPL for efficient commits. Partitions are added once
    /// when first seen via [`track`]; offsets are updated in-place by
    /// [`committable_tpl`] before each commit.
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
    /// On the first call for a given `(topic, partition)` pair the partition
    /// is also registered in the internal [`TopicPartitionList`].
    ///
    /// Only allocates a `String` when a topic is seen for the first time;
    /// subsequent calls for the same topic use `&str` lookups.
    pub fn track(&mut self, topic: &str, partition: i32, offset: i64) {
        if let Some(partitions) = self.partitions.get_mut(topic) {
            // Known topic — zero allocation.
            let entry = partitions.entry(partition);
            if matches!(&entry, std::collections::hash_map::Entry::Vacant(_)) {
                let _ = self.tpl.add_partition(topic, partition);
            }
            entry.or_insert_with(PartitionTracker::new).track(offset);
        } else {
            // New topic — allocate once.
            let _ = self.tpl.add_partition(topic, partition);
            let mut tracker = PartitionTracker::new();
            tracker.track(offset);
            let mut partitions = HashMap::new();
            let _ = partitions.insert(partition, tracker);
            let _ = self.partitions.insert(topic.to_string(), partitions);
        }
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

    /// Update the internal [`TopicPartitionList`] with current committable
    /// offsets and return a reference suitable for passing to
    /// `consumer.commit()`.
    ///
    /// Partitions are registered in the TPL lazily by [`track`], and their
    /// offsets are updated in-place here. If no partitions have been tracked
    /// yet the returned TPL is empty, which is safe to commit.
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

    /// Compute the offsets that should be committed to Kafka.
    ///
    /// For each tracked partition returns `(topic, partition, offset)` where
    /// `offset` is:
    /// - The lowest pending offset (commit *up to but not including* this one), or
    /// - `high_water_mark + 1` if no offsets are pending (all have been acked).
    #[cfg(test)]
    #[must_use]
    pub fn committable_offsets(&self) -> Vec<(String, i32, i64)> {
        let mut result = Vec::new();
        for (topic, partitions) in &self.partitions {
            for (&partition, tracker) in partitions {
                if let Some(offset) = tracker.committable_offset() {
                    result.push((topic.clone(), partition, offset));
                }
            }
        }
        result
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

    // ---- PartitionTracker tests ----

    #[test]
    fn partition_basic_track_and_ack() {
        let mut pt = PartitionTracker::new();

        pt.track(100);
        pt.track(101);
        pt.track(102);

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
        let mut pt = PartitionTracker::new();

        pt.track(100);
        pt.track(101);
        pt.track(102);
        pt.track(103);
        pt.track(104);

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
        let mut pt = PartitionTracker::new();

        pt.track(100);
        pt.track(100);
        pt.track(100);

        assert_eq!(pt.pending_count(), 1);
        assert!(pt.acknowledge(100));
        assert_eq!(pt.pending_count(), 0);
    }

    #[test]
    fn partition_ack_unknown_offset_is_noop() {
        let mut pt = PartitionTracker::new();

        pt.track(100);
        // Ack a non-existent offset — nothing should change.
        assert!(!pt.acknowledge(999));
        assert_eq!(pt.pending_count(), 1);
        assert_eq!(pt.lowest_pending(), Some(100));
        // HWM must not be set by an untracked offset.
        assert_eq!(pt.high_water_mark(), None);
    }

    #[test]
    fn partition_high_water_mark_after_all_acked() {
        let mut pt = PartitionTracker::new();

        pt.track(100);
        pt.track(101);
        pt.track(102);

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

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);
        tracker.track("traces", 0, 102);

        assert_eq!(tracker.pending_count("traces", 0), 3);

        // Committable should be the lowest pending.
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 100));

        // Ack lowest — should advance.
        assert!(tracker.acknowledge("traces", 0, 100));
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets[0], ("traces".to_string(), 0, 101));
    }

    #[test]
    fn tracker_out_of_order_acks() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);
        tracker.track("traces", 0, 102);

        // Ack 102 first — should NOT advance (100 still pending).
        assert!(!tracker.acknowledge("traces", 0, 102));
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets[0], ("traces".to_string(), 0, 100));

        // Ack 100 — advances to 101.
        assert!(tracker.acknowledge("traces", 0, 100));
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets[0], ("traces".to_string(), 0, 101));

        // Ack 101 — all acked, commits hwm + 1.
        assert!(tracker.acknowledge("traces", 0, 101));
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets[0], ("traces".to_string(), 0, 103)); // hwm=102, commit 103
    }

    #[test]
    fn tracker_multiple_partitions() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);
        tracker.track("traces", 1, 200);
        tracker.track("traces", 1, 201);
        tracker.track("metrics", 0, 300);

        assert_eq!(tracker.pending_count("traces", 0), 2);
        assert_eq!(tracker.pending_count("traces", 1), 2);
        assert_eq!(tracker.pending_count("metrics", 0), 1);
        assert_eq!(tracker.total_pending(), 5);

        // Ack from different partitions.
        assert!(tracker.acknowledge("traces", 0, 100));
        assert!(tracker.acknowledge("traces", 1, 200));

        let offsets = tracker.committable_offsets();
        let mut sorted: Vec<_> = offsets.into_iter().collect();
        sorted.sort();
        assert_eq!(sorted.len(), 3);
        assert!(sorted.contains(&("metrics".to_string(), 0, 300)));
        assert!(sorted.contains(&("traces".to_string(), 0, 101)));
        assert!(sorted.contains(&("traces".to_string(), 1, 201)));
    }

    #[test]
    fn tracker_all_acked_uses_high_water_mark() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

        let _ = tracker.acknowledge("traces", 0, 100);
        let _ = tracker.acknowledge("traces", 0, 101);

        assert_eq!(tracker.pending_count("traces", 0), 0);

        // Should commit hwm + 1 = 102.
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets.len(), 1);
        assert_eq!(offsets[0], ("traces".to_string(), 0, 102));
    }

    #[test]
    fn tracker_empty_returns_no_committable() {
        let tracker = OffsetTracker::new();
        assert!(tracker.committable_offsets().is_empty());
    }

    #[test]
    fn tracker_ack_unknown_partition_is_noop() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        // Ack for unknown topic/partition.
        assert!(!tracker.acknowledge("unknown", 99, 100));
        assert_eq!(tracker.pending_count("traces", 0), 1);
    }

    #[test]
    fn tracker_mixed_ack_nack_pattern() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);
        tracker.track("traces", 0, 102);
        tracker.track("traces", 0, 103);
        tracker.track("traces", 0, 104);

        // Simulate: ack 100, nack 102 (treated as ack), ack 101, nack 104.
        assert!(tracker.acknowledge("traces", 0, 100));
        assert!(!tracker.acknowledge("traces", 0, 102));
        assert!(tracker.acknowledge("traces", 0, 101));
        assert!(!tracker.acknowledge("traces", 0, 104));

        // 103 still pending.
        assert_eq!(tracker.pending_count("traces", 0), 1);
        let offsets = tracker.committable_offsets();
        assert_eq!(offsets[0], ("traces".to_string(), 0, 103));
    }

    #[test]
    fn tracker_multiple_topics() {
        let mut tracker = OffsetTracker::new();

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

        tracker.track("metrics", 0, 200);

        tracker.track("logs", 1, 300);
        tracker.track("logs", 1, 301);

        assert_eq!(tracker.total_pending(), 5);

        // Ack all of metrics.
        assert!(tracker.acknowledge("metrics", 0, 200));

        let offsets = tracker.committable_offsets();
        let mut sorted: Vec<_> = offsets.into_iter().collect();
        sorted.sort();

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

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

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

        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

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

    // ---- is_known_offset tests ----

    #[test]
    fn is_known_returns_false_for_unknown_partition() {
        let tracker = OffsetTracker::new();
        assert!(!tracker.is_known_offset("traces", 0, 100));
    }

    #[test]
    fn is_known_returns_true_for_pending_offset() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

        assert!(tracker.is_known_offset("traces", 0, 100));
        assert!(tracker.is_known_offset("traces", 0, 101));
        assert!(!tracker.is_known_offset("traces", 0, 102));
    }

    #[test]
    fn is_known_returns_true_for_offset_at_or_below_hwm() {
        let mut tracker = OffsetTracker::new();
        tracker.track("traces", 0, 100);
        tracker.track("traces", 0, 101);

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
        tracker.track("traces", 0, 100);
        let _ = tracker.acknowledge("traces", 0, 100);

        // hwm = 100, offset 101 has never been seen.
        assert!(!tracker.is_known_offset("traces", 0, 101));
    }
}
