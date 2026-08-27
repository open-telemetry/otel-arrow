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
//! Designed for single-threaded use on a `LocalSet` runtime -- no internal
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
    ///
    /// Returns the signed change in this partition's pending count so the
    /// enclosing [`OffsetTracker`] can maintain an O(1) aggregate without
    /// rescanning: `0` for a stale-generation no-op or a duplicate offset, `+1`
    /// for a fresh insert, and `1 - old_len` for a newer-generation reset (the
    /// prior `old_len` pending offsets are cleared and one new offset inserted).
    fn track(&mut self, offset: i64, generation: u64) -> isize {
        if generation < self.generation {
            // Stale ownership period -- do not touch current-period state.
            return 0;
        }
        let mut delta: isize = 0;
        if generation > self.generation {
            self.generation = generation;
            // The reset drops every currently-pending offset.
            delta -= self.pending.len() as isize;
            self.pending.clear();
            self.high_water_mark = None;
            self.last_lowest = None;
        }
        if self.pending.insert(offset) {
            delta += 1;
        }
        // Update cached lowest if this is lower or first entry.
        match self.last_lowest {
            None => self.last_lowest = Some(offset),
            Some(prev) if offset < prev => self.last_lowest = Some(offset),
            _ => {}
        }
        delta
    }

    /// Mark an offset as acknowledged.
    ///
    /// Returns `(removed, advanced)`:
    /// - `removed` is `true` when the offset was pending and has now been
    ///   removed, so the enclosing [`OffsetTracker`] can decrement its O(1)
    ///   aggregate pending count by one.
    /// - `advanced` is `true` when the lowest pending offset changed (i.e., the
    ///   committable watermark advanced), signalling that a commit may be
    ///   warranted.
    ///
    /// A spurious ack (offset never tracked or already acked) returns
    /// `(false, false)`.
    fn acknowledge(&mut self, offset: i64) -> (bool, bool) {
        if !self.pending.remove(&offset) {
            // Offset was never tracked (or already acked) -- no-op.
            return (false, false);
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
        (true, advanced)
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

    /// Generation-aware form of [`is_known`](Self::is_known).
    ///
    /// A message whose `generation` is **newer** than this partition's tracked
    /// generation belongs to a new ownership period (the partition was revoked
    /// and reassigned to this consumer). Its offset -- even if numerically equal
    /// to one already seen under the old period -- must NOT be treated as a
    /// known duplicate: the old period's `pending`/`high_water_mark` say nothing
    /// about the new period. So a newer-generation offset is always "unknown"
    /// (allowed through); the caller then tracks it, which resets this
    /// partition's state to the new generation via [`track`](Self::track).
    ///
    /// For a same-or-older generation the ordinary [`is_known`](Self::is_known)
    /// dedupe applies.
    fn is_known_for_generation(&self, offset: i64, generation: u64) -> bool {
        if generation > self.generation {
            return false;
        }
        self.is_known(offset)
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
/// Single-threaded -- no internal synchronization required.
pub struct OffsetTracker {
    partitions: HashMap<String, HashMap<i32, PartitionTracker>>,
    /// Persistent TPL reused across commits. Its partition membership is kept
    /// in sync with `partitions` by [`track`](Self::track) (adds) and
    /// [`revoke`](Self::revoke) (rebuilds); [`committable_tpl`](Self::committable_tpl)
    /// updates offsets in place.
    tpl: TopicPartitionList,
    /// O(1) aggregate of pending offsets across every tracked partition.
    ///
    /// Maintained incrementally at the three mutation sites ([`track`](Self::track),
    /// [`acknowledge`](Self::acknowledge), [`revoke`](Self::revoke)) so
    /// [`total_pending`](Self::total_pending) never rescans partitions on the
    /// hot receive path.
    total_pending: usize,
}

impl OffsetTracker {
    /// Create a new empty offset tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            partitions: HashMap::new(),
            tpl: TopicPartitionList::new(),
            total_pending: 0,
        }
    }

    /// Apply a signed per-partition pending-count delta to the cached
    /// aggregate. `delta` originates from [`PartitionTracker::track`] (fresh
    /// insert, duplicate, or newer-generation reset) or an acknowledge/revoke
    /// removal.
    fn apply_pending_delta(&mut self, delta: isize) {
        // A correct delta never drives the aggregate negative; a saturating
        // signed add keeps the counter well-defined even if it somehow did.
        self.total_pending = self.total_pending.saturating_add_signed(delta);
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
        let delta = if let Some(partitions) = self.partitions.get_mut(topic) {
            // Known topic -- zero allocation.
            let entry = partitions.entry(partition);
            if matches!(&entry, std::collections::hash_map::Entry::Vacant(_)) {
                // First sight of this partition -- register it in the TPL.
                let _ = self.tpl.add_partition(topic, partition);
            }
            entry
                .or_insert_with(|| PartitionTracker::new(generation))
                .track(offset, generation)
        } else {
            // New topic -- allocate once and register the partition in the TPL.
            let _ = self.tpl.add_partition(topic, partition);
            let mut tracker = PartitionTracker::new(generation);
            let delta = tracker.track(offset, generation);
            let mut partitions = HashMap::new();
            let _ = partitions.insert(partition, tracker);
            let _ = self.partitions.insert(topic.to_string(), partitions);
            delta
        };
        self.apply_pending_delta(delta);
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
            // Not tracked -- nothing to revoke.
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
            // Unknown topic -- nothing tracked, TPL already excludes it.
            return;
        };
        let Some(removed) = partitions.remove(&partition) else {
            // Unknown partition -- TPL already excludes it.
            return;
        };
        // Dropping the partition removes all of its pending offsets from the
        // aggregate. Capture the count before releasing the `partitions` borrow.
        let removed_pending = removed.pending_count();
        if partitions.is_empty() {
            let _ = self.partitions.remove(topic);
        }
        self.apply_pending_delta(-(removed_pending as isize));
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
        let (removed, advanced) = self
            .partitions
            .get_mut(topic)
            .and_then(|parts| parts.get_mut(&partition))
            .map(|tracker| tracker.acknowledge(offset))
            .unwrap_or((false, false));
        if removed {
            // A successful ack retires exactly one pending offset.
            self.apply_pending_delta(-1);
        }
        advanced
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

    /// Generation-aware form of [`is_known_offset`](Self::is_known_offset), used
    /// by the idempotency dedupe on the receive path.
    ///
    /// Returns `false` for an offset whose `generation` is newer than the
    /// partition's tracked generation, so a message redelivered under a new
    /// ownership period (same offset, newer generation after a revoke+reassign)
    /// is never skipped as a duplicate -- it is reprocessed, and tracking it
    /// resets the partition to the new generation. For a same-or-older
    /// generation the ordinary known-offset dedupe applies. An untracked
    /// partition is not known.
    #[must_use]
    pub fn is_known_offset_for_generation(
        &self,
        topic: &str,
        partition: i32,
        offset: i64,
        generation: u64,
    ) -> bool {
        self.partitions
            .get(topic)
            .and_then(|parts| parts.get(&partition))
            .map(|tracker| tracker.is_known_for_generation(offset, generation))
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

    /// Total number of pending (tracked but un-committed) offsets across all
    /// partitions.
    ///
    /// This is the receiver's in-flight depth: records that have been delivered
    /// downstream and are awaiting an Ack/Nack whose commit has not yet advanced
    /// past them. Exposed for the `records_in_flight` up/down counter.
    ///
    /// O(1): returns the aggregate maintained incrementally at the mutation
    /// sites rather than rescanning every partition on the hot receive path. The
    /// invariant that this cached value equals a full O(n) rescan is verified by
    /// the unit tests after every mutation.
    #[must_use]
    pub fn total_pending(&self) -> usize {
        self.total_pending
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
}
