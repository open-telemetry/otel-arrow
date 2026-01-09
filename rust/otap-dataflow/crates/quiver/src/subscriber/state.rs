// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-subscriber state tracking.
//!
//! Each subscriber maintains independent progress through the segment stream.
//! State is rebuilt from progress files on recovery and updated in-memory during
//! normal operation.
//!
//! # Tracking Model
//!
//! Quiver uses a hybrid tracking model supporting both in-order and out-of-order
//! delivery:
//!
//! - **Per-segment completion**: Track which bundles are resolved (acked or
//!   dropped) within each segment using a bitmap.
//! - **Oldest incomplete segment**: Derived from per-segment state, equivalent
//!   to a traditional high-water mark (HWM).
//! - **Claimed bundles**: Track bundles currently being processed to prevent
//!   double-delivery.
//!
//! This model supports priority delivery scenarios where newer data is processed
//! before older data, while maintaining accurate progress tracking.

use std::collections::{BTreeMap, HashSet};

use crate::segment::SegmentSeq;

use super::progress::SegmentProgressEntry;
use super::types::{AckOutcome, BundleIndex, BundleRef, SubscriberId};

// ─────────────────────────────────────────────────────────────────────────────
// SegmentProgress
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks per-bundle resolution status within a single segment.
///
/// Uses a bitmap to track which bundles have been resolved (acked or dropped).
/// A segment is complete when all bundles up to `bundle_count` are resolved.
#[derive(Debug, Clone)]
pub struct SegmentProgress {
    /// Total number of bundles in this segment.
    bundle_count: u32,
    /// Bitmap of resolved bundles (1 = resolved).
    /// Uses a Vec<u64> for segments with more than 64 bundles.
    resolved: Vec<u64>,
    /// Count of resolved bundles (for efficient completion check).
    resolved_count: u32,
}

impl SegmentProgress {
    /// Creates progress tracking for a segment with the given bundle count.
    #[must_use]
    pub fn new(bundle_count: u32) -> Self {
        let word_count = (bundle_count as usize).div_ceil(64);
        Self {
            bundle_count,
            resolved: vec![0u64; word_count],
            resolved_count: 0,
        }
    }

    /// Marks a bundle as resolved.
    ///
    /// Returns `true` if this was a new resolution, `false` if already resolved.
    pub fn mark_resolved(&mut self, index: BundleIndex) -> bool {
        let idx = index.raw() as usize;
        if idx >= self.bundle_count as usize {
            return false;
        }

        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        let mask = 1u64 << bit_idx;

        if self.resolved[word_idx] & mask != 0 {
            // Already resolved
            return false;
        }

        self.resolved[word_idx] |= mask;
        self.resolved_count += 1;
        true
    }

    /// Checks if a bundle is resolved.
    #[must_use]
    pub fn is_resolved(&self, index: BundleIndex) -> bool {
        let idx = index.raw() as usize;
        if idx >= self.bundle_count as usize {
            return false;
        }

        let word_idx = idx / 64;
        let bit_idx = idx % 64;
        (self.resolved[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// Checks if all bundles in this segment are resolved.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.resolved_count == self.bundle_count
    }

    /// Returns the total number of bundles in this segment.
    #[must_use]
    pub fn bundle_count(&self) -> u32 {
        self.bundle_count
    }

    /// Returns the number of resolved bundles.
    #[must_use]
    pub fn resolved_count(&self) -> u32 {
        self.resolved_count
    }

    /// Returns a clone of the resolved bitmap.
    ///
    /// Used for serializing progress to disk.
    #[must_use]
    pub fn resolved_bitmap(&self) -> Vec<u64> {
        self.resolved.clone()
    }

    /// Returns the next unresolved bundle index, if any.
    ///
    /// Scans from the beginning to find the first unresolved bundle.
    #[must_use]
    pub fn next_unresolved(&self) -> Option<BundleIndex> {
        for (word_idx, &word) in self.resolved.iter().enumerate() {
            if word == u64::MAX {
                continue; // All bits set in this word
            }

            // Find first zero bit
            let inverted = !word;
            let bit_idx = inverted.trailing_zeros() as usize;
            let bundle_idx = word_idx * 64 + bit_idx;

            if bundle_idx < self.bundle_count as usize {
                return Some(BundleIndex::new(bundle_idx as u32));
            }
        }
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SubscriberState
// ─────────────────────────────────────────────────────────────────────────────

/// Complete state for a single subscriber.
///
/// Tracks which bundles have been resolved across all segments, which bundles
/// are currently claimed, and the subscriber's lifecycle status.
#[derive(Debug)]
pub struct SubscriberState {
    /// The subscriber's identifier.
    id: SubscriberId,
    /// Per-segment progress tracking.
    /// Segments are removed once complete and older than all other subscribers.
    segments: BTreeMap<SegmentSeq, SegmentProgress>,
    /// Bundles currently claimed by this subscriber (being processed).
    claimed: HashSet<BundleRef>,
    /// Whether the subscriber is active (receiving new bundles).
    active: bool,
}

impl SubscriberState {
    /// Creates new state for a subscriber.
    #[must_use]
    pub fn new(id: SubscriberId) -> Self {
        Self {
            id,
            segments: BTreeMap::new(),
            claimed: HashSet::new(),
            active: false,
        }
    }

    /// Returns the subscriber's identifier.
    #[must_use]
    pub fn id(&self) -> &SubscriberId {
        &self.id
    }

    /// Returns whether the subscriber is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Activates the subscriber to receive new bundles.
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivates the subscriber.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Initializes tracking for a new segment.
    ///
    /// Called when a segment is finalized and this subscriber should start
    /// tracking it.
    pub fn add_segment(&mut self, segment_seq: SegmentSeq, bundle_count: u32) {
        let _ = self
            .segments
            .entry(segment_seq)
            .or_insert_with(|| SegmentProgress::new(bundle_count));
    }

    /// Records a terminal outcome for a bundle.
    ///
    /// Removes the bundle from claimed set and marks it as resolved.
    /// Returns `true` if this was a new resolution.
    pub fn record_outcome(&mut self, bundle_ref: BundleRef, _outcome: AckOutcome) -> bool {
        // Remove from claimed set
        let _ = self.claimed.remove(&bundle_ref);

        // Mark as resolved in segment progress
        if let Some(progress) = self.segments.get_mut(&bundle_ref.segment_seq) {
            progress.mark_resolved(bundle_ref.bundle_index)
        } else {
            // Segment not tracked - possibly already complete
            false
        }
    }

    /// Claims a bundle for processing.
    ///
    /// Returns `true` if the bundle was successfully claimed, `false` if it
    /// was already claimed or resolved.
    pub fn claim(&mut self, bundle_ref: BundleRef) -> bool {
        // Check if already resolved
        if let Some(progress) = self.segments.get(&bundle_ref.segment_seq) {
            if progress.is_resolved(bundle_ref.bundle_index) {
                return false;
            }
        }

        // Try to claim
        self.claimed.insert(bundle_ref)
    }

    /// Releases a claimed bundle without resolving it.
    ///
    /// Used when deferring a bundle for later retry.
    pub fn release(&mut self, bundle_ref: BundleRef) {
        let _ = self.claimed.remove(&bundle_ref);
    }

    /// Checks if a bundle is currently claimed.
    #[must_use]
    pub fn is_claimed(&self, bundle_ref: &BundleRef) -> bool {
        self.claimed.contains(bundle_ref)
    }

    /// Checks if a bundle is resolved.
    #[must_use]
    pub fn is_resolved(&self, bundle_ref: &BundleRef) -> bool {
        self.segments
            .get(&bundle_ref.segment_seq)
            .is_some_and(|p| p.is_resolved(bundle_ref.bundle_index))
    }

    /// Returns the oldest segment sequence that still has pending bundles.
    ///
    /// This is the equivalent of the traditional high-water mark (HWM).
    /// Returns `None` if all tracked segments are complete.
    #[must_use]
    pub fn oldest_incomplete_segment(&self) -> Option<SegmentSeq> {
        self.segments
            .iter()
            .find(|(_, progress)| !progress.is_complete())
            .map(|(seq, _)| *seq)
    }

    /// Returns the highest tracked segment sequence.
    ///
    /// This represents the high-water mark of segments this subscriber knows about.
    /// Returns `None` if no segments are tracked.
    #[must_use]
    pub fn highest_tracked_segment(&self) -> Option<SegmentSeq> {
        self.segments.keys().next_back().copied()
    }

    /// Returns the next pending bundle to deliver.
    ///
    /// Scans segments in order (oldest first) and returns the first bundle
    /// that is neither resolved nor currently claimed.
    #[must_use]
    pub fn next_pending(&self) -> Option<BundleRef> {
        for (&segment_seq, progress) in &self.segments {
            // Find first unresolved in this segment
            if let Some(bundle_index) = progress.next_unresolved() {
                let bundle_ref = BundleRef::new(segment_seq, bundle_index);

                // Skip if claimed
                if !self.claimed.contains(&bundle_ref) {
                    return Some(bundle_ref);
                }

                // Continue scanning this segment for unclaimed bundles
                for idx in (bundle_index.raw() + 1)..progress.bundle_count() {
                    let idx = BundleIndex::new(idx);
                    if !progress.is_resolved(idx) {
                        let bundle_ref = BundleRef::new(segment_seq, idx);
                        if !self.claimed.contains(&bundle_ref) {
                            return Some(bundle_ref);
                        }
                    }
                }
            }
        }
        None
    }

    /// Removes completed segments older than the given sequence.
    ///
    /// Called during segment cleanup to free memory for segments that all
    /// subscribers have completed.
    pub fn remove_completed_segments_before(&mut self, before: SegmentSeq) {
        self.segments
            .retain(|seq, progress| *seq >= before || !progress.is_complete());
    }

    /// Returns the number of tracked segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Returns the number of currently claimed bundles.
    #[must_use]
    pub fn claimed_count(&self) -> usize {
        self.claimed.len()
    }

    /// Returns total pending bundle count across all segments.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.segments
            .values()
            .map(|p| (p.bundle_count() - p.resolved_count()) as usize)
            .sum()
    }

    /// Checks if this subscriber has any claimed bundles in the given segment.
    ///
    /// Used by DropOldest policy to determine if a segment has active readers.
    #[must_use]
    pub fn has_claimed_in_segment(&self, segment_seq: SegmentSeq) -> bool {
        self.claimed
            .iter()
            .any(|bundle_ref| bundle_ref.segment_seq == segment_seq)
    }

    /// Force-completes a segment by marking all bundles as resolved.
    ///
    /// Used by DropOldest policy to forcibly drop pending segments.
    /// This releases any claimed bundles and marks all bundles as resolved.
    ///
    /// Returns `true` if the segment was found and completed, `false` if not tracked.
    pub fn force_complete_segment(&mut self, segment_seq: SegmentSeq) -> bool {
        // Release any claimed bundles in this segment
        self.claimed
            .retain(|bundle_ref| bundle_ref.segment_seq != segment_seq);

        // Mark all bundles as resolved
        if let Some(progress) = self.segments.get_mut(&segment_seq) {
            let bundle_count = progress.bundle_count();
            for i in 0..bundle_count {
                let _ = progress.mark_resolved(BundleIndex::new(i));
            }
            true
        } else {
            false
        }
    }

    /// Converts this subscriber's state to progress entries for persistence.
    ///
    /// Returns a vector of segment progress entries in segment order (oldest first).
    /// Used by the registry to serialize state to progress files.
    #[must_use]
    pub fn to_progress_entries(&self) -> Vec<SegmentProgressEntry> {
        self.segments
            .iter()
            .map(|(&seg_seq, progress)| {
                SegmentProgressEntry::from_bitmap(
                    seg_seq,
                    progress.bundle_count(),
                    progress.resolved_bitmap(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
#[allow(unused_results, unused_mut)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // SegmentProgress tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn segment_progress_basic() {
        let mut progress = SegmentProgress::new(10);
        assert_eq!(progress.bundle_count(), 10);
        assert_eq!(progress.resolved_count(), 0);
        assert!(!progress.is_complete());
    }

    #[test]
    fn segment_progress_mark_resolved() {
        let mut progress = SegmentProgress::new(5);

        assert!(progress.mark_resolved(BundleIndex::new(2)));
        assert!(progress.is_resolved(BundleIndex::new(2)));
        assert!(!progress.is_resolved(BundleIndex::new(0)));
        assert_eq!(progress.resolved_count(), 1);
    }

    #[test]
    fn segment_progress_double_resolve() {
        let mut progress = SegmentProgress::new(5);

        assert!(progress.mark_resolved(BundleIndex::new(2)));
        assert!(!progress.mark_resolved(BundleIndex::new(2))); // Already resolved
        assert_eq!(progress.resolved_count(), 1);
    }

    #[test]
    fn segment_progress_complete() {
        let mut progress = SegmentProgress::new(3);

        progress.mark_resolved(BundleIndex::new(0));
        progress.mark_resolved(BundleIndex::new(1));
        assert!(!progress.is_complete());

        progress.mark_resolved(BundleIndex::new(2));
        assert!(progress.is_complete());
    }

    #[test]
    fn segment_progress_next_unresolved() {
        let mut progress = SegmentProgress::new(5);

        assert_eq!(progress.next_unresolved(), Some(BundleIndex::new(0)));

        progress.mark_resolved(BundleIndex::new(0));
        assert_eq!(progress.next_unresolved(), Some(BundleIndex::new(1)));

        progress.mark_resolved(BundleIndex::new(1));
        progress.mark_resolved(BundleIndex::new(2));
        assert_eq!(progress.next_unresolved(), Some(BundleIndex::new(3)));
    }

    #[test]
    fn segment_progress_next_unresolved_none() {
        let mut progress = SegmentProgress::new(2);
        progress.mark_resolved(BundleIndex::new(0));
        progress.mark_resolved(BundleIndex::new(1));
        assert_eq!(progress.next_unresolved(), None);
    }

    #[test]
    fn segment_progress_large_segment() {
        // Test with more than 64 bundles
        let mut progress = SegmentProgress::new(100);

        progress.mark_resolved(BundleIndex::new(0));
        progress.mark_resolved(BundleIndex::new(63));
        progress.mark_resolved(BundleIndex::new(64));
        progress.mark_resolved(BundleIndex::new(99));

        assert!(progress.is_resolved(BundleIndex::new(0)));
        assert!(progress.is_resolved(BundleIndex::new(63)));
        assert!(progress.is_resolved(BundleIndex::new(64)));
        assert!(progress.is_resolved(BundleIndex::new(99)));
        assert!(!progress.is_resolved(BundleIndex::new(50)));
        assert_eq!(progress.resolved_count(), 4);
    }

    #[test]
    fn segment_progress_out_of_bounds() {
        let mut progress = SegmentProgress::new(5);
        assert!(!progress.mark_resolved(BundleIndex::new(10)));
        assert!(!progress.is_resolved(BundleIndex::new(10)));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SubscriberState tests
    // ─────────────────────────────────────────────────────────────────────────

    fn make_state(id: &str) -> SubscriberState {
        SubscriberState::new(SubscriberId::new(id).unwrap())
    }

    #[test]
    fn subscriber_state_new() {
        let state = make_state("test-sub");
        assert_eq!(state.id().as_str(), "test-sub");
        assert!(!state.is_active());
        assert_eq!(state.segment_count(), 0);
    }

    #[test]
    fn subscriber_state_activate() {
        let mut state = make_state("test-sub");
        state.activate();
        assert!(state.is_active());

        state.deactivate();
        assert!(!state.is_active());
    }

    #[test]
    fn subscriber_state_add_segment() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 10);
        state.add_segment(SegmentSeq::new(2), 5);

        assert_eq!(state.segment_count(), 2);
        assert_eq!(state.pending_count(), 15);
    }

    #[test]
    fn subscriber_state_claim_and_release() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 5);

        let bundle_ref = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(2));

        assert!(state.claim(bundle_ref));
        assert!(state.is_claimed(&bundle_ref));
        assert!(!state.claim(bundle_ref)); // Already claimed

        state.release(bundle_ref);
        assert!(!state.is_claimed(&bundle_ref));
    }

    #[test]
    fn subscriber_state_record_outcome() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 3);

        let bundle_ref = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(1));

        // Claim first
        state.claim(bundle_ref);
        assert!(state.is_claimed(&bundle_ref));

        // Record outcome
        assert!(state.record_outcome(bundle_ref, AckOutcome::Acked));
        assert!(!state.is_claimed(&bundle_ref));
        assert!(state.is_resolved(&bundle_ref));

        // Can't record twice
        assert!(!state.record_outcome(bundle_ref, AckOutcome::Acked));
    }

    #[test]
    fn subscriber_state_next_pending() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 3);
        state.add_segment(SegmentSeq::new(2), 2);

        // First pending should be segment 1, bundle 0
        let pending = state.next_pending().unwrap();
        assert_eq!(pending.segment_seq, SegmentSeq::new(1));
        assert_eq!(pending.bundle_index, BundleIndex::new(0));

        // Claim it
        state.claim(pending);

        // Next pending should skip claimed
        let pending = state.next_pending().unwrap();
        assert_eq!(pending.bundle_index, BundleIndex::new(1));

        // Resolve first two bundles of segment 1
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0)),
            AckOutcome::Acked,
        );
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(1)),
            AckOutcome::Acked,
        );

        // Next should be segment 1 bundle 2
        let pending = state.next_pending().unwrap();
        assert_eq!(pending.segment_seq, SegmentSeq::new(1));
        assert_eq!(pending.bundle_index, BundleIndex::new(2));
    }

    #[test]
    fn subscriber_state_oldest_incomplete() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 2);
        state.add_segment(SegmentSeq::new(2), 2);

        assert_eq!(state.oldest_incomplete_segment(), Some(SegmentSeq::new(1)));

        // Complete segment 1
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0)),
            AckOutcome::Acked,
        );
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(1)),
            AckOutcome::Acked,
        );

        assert_eq!(state.oldest_incomplete_segment(), Some(SegmentSeq::new(2)));

        // Complete segment 2
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(2), BundleIndex::new(0)),
            AckOutcome::Acked,
        );
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(2), BundleIndex::new(1)),
            AckOutcome::Acked,
        );

        assert_eq!(state.oldest_incomplete_segment(), None);
    }

    #[test]
    fn subscriber_state_remove_completed_segments() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 1);
        state.add_segment(SegmentSeq::new(2), 1);
        state.add_segment(SegmentSeq::new(3), 1);

        // Complete segments 1 and 2
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0)),
            AckOutcome::Acked,
        );
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(2), BundleIndex::new(0)),
            AckOutcome::Acked,
        );

        // Remove completed before segment 3
        state.remove_completed_segments_before(SegmentSeq::new(3));

        assert_eq!(state.segment_count(), 1);
        assert!(!state.is_resolved(&BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0))));
    }

    #[test]
    fn subscriber_state_pending_count() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 5);
        state.add_segment(SegmentSeq::new(2), 3);

        assert_eq!(state.pending_count(), 8);

        state.record_outcome(
            BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0)),
            AckOutcome::Acked,
        );
        state.record_outcome(
            BundleRef::new(SegmentSeq::new(2), BundleIndex::new(0)),
            AckOutcome::Dropped,
        );

        assert_eq!(state.pending_count(), 6);
    }

    #[test]
    fn subscriber_state_claim_resolved_fails() {
        let mut state = make_state("test-sub");
        state.add_segment(SegmentSeq::new(1), 2);

        let bundle_ref = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(0));

        // Resolve first
        state.claim(bundle_ref);
        state.record_outcome(bundle_ref, AckOutcome::Acked);

        // Can't claim resolved bundle
        assert!(!state.claim(bundle_ref));
    }
}
