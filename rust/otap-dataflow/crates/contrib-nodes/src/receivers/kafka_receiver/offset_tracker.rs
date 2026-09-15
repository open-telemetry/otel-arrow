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

    /// Rewind this partition to the earliest unresolved offset after a
    /// non-permanent NACK.
    ///
    /// The failed offset must still be pending. Every pending or acknowledged
    /// offset at or after the rewind point belongs to the obsolete delivery
    /// generation and is discarded; the rewind point remains pending so the
    /// committable offset cannot advance past it.
    fn prepare_replay(&mut self, failed_offset: i64) -> Option<(i64, isize)> {
        if !self.pending.contains(&failed_offset) {
            return None;
        }

        let rewind_offset = self.pending.first().copied()?;
        let old_pending = self.pending.len();
        self.pending.clear();
        let _ = self.pending.insert(rewind_offset);
        self.last_lowest = Some(rewind_offset);
        self.high_water_mark = self
            .high_water_mark
            .zip(rewind_offset.checked_sub(1))
            .map(|(high_water_mark, cap)| high_water_mark.min(cap));
        Some((rewind_offset, 1 - old_pending as isize))
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
    /// Maintained incrementally at the mutation sites ([`track`](Self::track),
    /// [`acknowledge`](Self::acknowledge), [`prepare_replay`](Self::prepare_replay),
    /// and [`revoke`](Self::revoke)) so
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
    /// insert, duplicate, or newer-generation reset), replay rewind, or an
    /// acknowledge/revoke removal.
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

    /// Prepare a partition for deliberate replay after a non-permanent NACK.
    ///
    /// Returns the earliest unresolved offset that must be used for the Kafka
    /// seek. Returning `None` means the failed offset is no longer pending, so
    /// duplicate or obsolete feedback must not initiate a replay.
    pub fn prepare_replay(
        &mut self,
        topic: &str,
        partition: i32,
        failed_offset: i64,
        ownership_generation: u64,
    ) -> Option<i64> {
        let (rewind_offset, delta) = self
            .partitions
            .get_mut(topic)
            .and_then(|parts| parts.get_mut(&partition))
            .filter(|tracker| tracker.generation == ownership_generation)
            .and_then(|tracker| tracker.prepare_replay(failed_offset))?;
        self.apply_pending_delta(delta);
        Some(rewind_offset)
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
mod tests;
