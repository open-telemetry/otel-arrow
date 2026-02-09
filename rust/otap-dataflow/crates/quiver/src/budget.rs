// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Disk budget management for enforcing storage caps.
//!
//! The [`DiskBudget`] provides a thread-safe mechanism for tracking and limiting
//! disk usage within a single Quiver engine, with **separate accounting** for
//! WAL bytes and segment bytes under a shared cap.
//!
//! # Split WAL / Segment Accounting
//!
//! Two atomic counters (`wal_used`, `segment_used`) track each pool independently.
//! Segment reservations check the combined total against the cap:
//!
//! ```text
//! wal_used + segment_used + new_bytes <= cap
//! ```
//!
//! This eliminates the transient double-charge that previously occurred during
//! segment finalization, when both the WAL entries and the newly-written
//! segment file existed simultaneously on disk.
//!
//! # Reservation Pattern
//!
//! ```ignore
//! let pending = budget.try_reserve_segment(estimated_bytes)?;
//! let actual = write_segment_file(...)?;
//! pending.commit(actual);
//! ```
//!
//! # Multi-Engine Deployment (Phase 1: Static Quotas)
//!
//! When running multiple engines (e.g., one per CPU core), each engine should
//! receive its own `DiskBudget` with a **static quota** that is the global cap
//! divided by the number of engines:
//!
//! ```ignore
//! let num_engines = 4;
//! let per_engine_cap = global_cap / num_engines as u64;
//!
//! // Each engine gets its own budget - no sharing, no coordination
//! let budget1 = Arc::new(DiskBudget::new(per_engine_cap, policy));
//! let budget2 = Arc::new(DiskBudget::new(per_engine_cap, policy));
//! let engine1 = QuiverEngine::open(config1, budget1).await?;
//! let engine2 = QuiverEngine::open(config2, budget2).await?;
//! ```
//!
//! This approach has zero cross-engine coordination overhead. Each engine
//! enforces its quota independently. The tradeoff is that:
//! - An idle engine's unused quota cannot be borrowed by a busy engine
//! - Global usage can briefly exceed the total cap by up to
//!   `(N-1) * segment_size` during concurrent segment finalizations

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use parking_lot::Mutex;

use crate::config::RetentionPolicy;
use crate::error::{QuiverError, Result};
use crate::logging::otel_warn;

/// Error returned when budget configuration is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetConfigError {
    /// The cap is too small to hold at least one segment plus the WAL.
    CapTooSmall {
        /// The requested cap.
        cap: u64,
        /// The minimum cap required (`wal_max_size + segment_size`).
        min_cap: u64,
        /// The segment size used in the calculation.
        segment_size_bytes: u64,
        /// The WAL max size used in the calculation.
        wal_max_size_bytes: u64,
    },
}

impl std::fmt::Display for BudgetConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetConfigError::CapTooSmall {
                cap,
                min_cap,
                segment_size_bytes,
                wal_max_size_bytes,
            } => {
                let cap_mb = cap / (1024 * 1024);
                let min_mb = min_cap / (1024 * 1024);
                let seg_mb = segment_size_bytes / (1024 * 1024);
                let wal_mb = wal_max_size_bytes / (1024 * 1024);
                write!(
                    f,
                    "Budget cap {} MB is too small. With segment_size={} MB and WAL max={} MB, \
                     minimum cap required: {} MB",
                    cap_mb, seg_mb, wal_mb, min_mb
                )
            }
        }
    }
}

impl std::error::Error for BudgetConfigError {}

/// Callback type for reclaiming space when using `DropOldest` policy.
///
/// The callback receives the number of bytes needed and should attempt to
/// free at least that much space by deleting old segments. Returns the
/// number of bytes actually freed (0 if no space could be reclaimed).
pub type ReclaimCallback = Box<dyn Fn(u64) -> u64 + Send + Sync>;

/// Callback type for cleanup (completed segments only, no data loss).
///
/// Returns the number of segments cleaned up.
pub type CleanupCallback = Box<dyn Fn() -> usize + Send + Sync>;

/// Disk budget for enforcing storage caps within a single engine.
///
/// Thread-safe via `Arc` for internal sharing between WAL, segment store,
/// and other engine components.
///
/// # Phase 1: Per-Engine Static Quotas
///
/// In multi-engine deployments, each engine should have its own `DiskBudget`
/// with a static quota (global cap / number of engines). This avoids any
/// cross-engine coordination overhead. See the module docs for details.
///
/// # Split WAL/Segment Accounting
///
/// The budget tracks WAL bytes and segment bytes in **separate** atomic
/// counters under a shared cap. This eliminates the transient double-charge
/// that previously occurred during segment finalization, when both the WAL
/// entries and the newly-written segment file existed simultaneously.
///
/// - **WAL pool** (`wal_used`): grown by `record_wal_bytes`, shrunk by
///   `release_wal_bytes` after WAL purge.
/// - **Segment pool** (`segment_used`): grown by `try_reserve_segment` /
///   `record_existing_segment`, shrunk by `release_segment` on deletion.
///
/// Reservation checks use the combined total: `wal_used + segment_used + N <= cap`.
pub struct DiskBudget {
    /// Maximum allowed bytes (shared across WAL + segments).
    cap: u64,
    /// Current WAL bytes in use (appended entries, rotated files).
    wal_used: AtomicU64,
    /// Current segment bytes in use (finalized segment files).
    segment_used: AtomicU64,
    /// Policy when cap is exceeded.
    policy: RetentionPolicy,
    /// Callback for reclaiming space (used with `DropOldest`).
    reclaim_callback: Mutex<Option<ReclaimCallback>>,
    /// Callback for cleanup (completed segments only, safe for `Backpressure`).
    cleanup_callback: Mutex<Option<CleanupCallback>>,
}

impl std::fmt::Debug for DiskBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Note: reclaim_callback and cleanup_callback are omitted because
        // function pointers / closures don't implement Debug
        f.debug_struct("DiskBudget")
            .field("cap", &self.cap)
            .field("wal_used", &self.wal_used.load(Ordering::Relaxed))
            .field("segment_used", &self.segment_used.load(Ordering::Relaxed))
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl DiskBudget {
    /// Creates a new disk budget with the given cap and policy.
    ///
    /// # Arguments
    ///
    /// * `cap` - Maximum bytes allowed. Use `u64::MAX` for effectively unlimited.
    /// * `policy` - What to do when the cap is exceeded.
    #[must_use]
    pub fn new(cap: u64, policy: RetentionPolicy) -> Self {
        Self {
            cap,
            wal_used: AtomicU64::new(0),
            segment_used: AtomicU64::new(0),
            policy,
            reclaim_callback: Mutex::new(None),
            cleanup_callback: Mutex::new(None),
        }
    }

    /// Creates an unlimited budget (no cap enforcement).
    ///
    /// Useful for testing or when disk limits are managed externally.
    #[must_use]
    pub fn unlimited() -> Self {
        Self::new(u64::MAX, RetentionPolicy::Backpressure)
    }

    /// Returns the configured cap.
    #[must_use]
    pub const fn cap(&self) -> u64 {
        self.cap
    }

    /// Returns the total bytes in use (WAL + segments).
    #[must_use]
    pub fn used(&self) -> u64 {
        self.wal_used
            .load(Ordering::Relaxed)
            .saturating_add(self.segment_used.load(Ordering::Relaxed))
    }

    /// Returns the current WAL bytes in use.
    #[must_use]
    pub fn wal_used(&self) -> u64 {
        self.wal_used.load(Ordering::Relaxed)
    }

    /// Returns the current segment bytes in use.
    #[must_use]
    pub fn segment_used(&self) -> u64 {
        self.segment_used.load(Ordering::Relaxed)
    }

    /// Returns the remaining headroom before the cap.
    #[must_use]
    pub fn headroom(&self) -> u64 {
        self.cap.saturating_sub(self.used())
    }

    /// Checks if there is sufficient headroom for ingestion.
    ///
    /// Returns `true` if the available headroom (cap minus total WAL + segment
    /// usage) is at least `bytes`. This is used to apply backpressure at the
    /// ingestion boundary.
    ///
    /// Because WAL and segment bytes are tracked separately, the WAL's share
    /// of the cap is treated as unavoidable overhead. Ingestion is gated on
    /// the _total_ remaining space, so the engine naturally stops accepting
    /// data when the combined usage approaches the cap.
    ///
    /// This is a "soft" check - it does not reserve space. Use this to decide
    /// whether to accept new data, then use `try_reserve_segment` for the actual
    /// reservation.
    #[must_use]
    pub fn has_ingest_headroom(&self, bytes: u64) -> bool {
        self.headroom() >= bytes
    }

    /// Returns the configured retention policy.
    #[must_use]
    pub const fn policy(&self) -> RetentionPolicy {
        self.policy
    }

    /// Sets a callback for reclaiming space when using `DropOldest` policy.
    ///
    /// The callback is invoked when a reservation would exceed the cap and
    /// the policy is `DropOldest`. It receives the number of bytes needed
    /// and should attempt to free space by deleting old segments. Returns
    /// the number of bytes actually freed (0 if no space could be reclaimed).
    pub fn set_reclaim_callback<F>(&self, callback: F)
    where
        F: Fn(u64) -> u64 + Send + Sync + 'static,
    {
        *self.reclaim_callback.lock() = Some(Box::new(callback));
    }

    /// Sets a callback for cleanup (completed segments only).
    ///
    /// This callback is invoked in `Backpressure` mode before returning
    /// `StorageAtCapacity` error. It should only delete fully-processed
    /// segments (no data loss). Returns the number of segments cleaned.
    pub fn set_cleanup_callback<F>(&self, callback: F)
    where
        F: Fn() -> usize + Send + Sync + 'static,
    {
        *self.cleanup_callback.lock() = Some(Box::new(callback));
    }

    // ── Segment pool ─────────────────────────────────────────────────────

    /// Attempts to reserve segment bytes for a pending write.
    ///
    /// Returns a [`PendingWrite`] guard that holds the reservation. The guard
    /// must be committed with the actual bytes written, or it will release
    /// the reservation on drop (for error handling).
    ///
    /// The reservation checks the **combined** WAL + segment usage against the
    /// cap, treating WAL bytes as unavoidable overhead. This means that during
    /// segment finalization the WAL bytes being converted to a segment are
    /// naturally accounted for — no special "during replay" flag is needed.
    ///
    /// # Errors
    ///
    /// Returns [`QuiverError::StorageAtCapacity`] if the reservation would
    /// exceed the cap and the policy is `Backpressure`.
    ///
    /// With `DropOldest` policy, this will invoke the reclaim callback to
    /// attempt to free space before failing.
    pub fn try_reserve_segment(self: &Arc<Self>, bytes: u64) -> Result<PendingWrite> {
        // Fast path: if bytes is 0, no reservation needed
        if bytes == 0 {
            return Ok(PendingWrite {
                budget: self.clone(),
                reserved: 0,
                committed: AtomicBool::new(false),
            });
        }

        // Try to atomically reserve the space in the segment pool
        loop {
            let wal = self.wal_used.load(Ordering::Acquire);
            let current_seg = self.segment_used.load(Ordering::Acquire);
            let new_seg = current_seg.saturating_add(bytes);

            if wal.saturating_add(new_seg) > self.cap {
                // Would exceed cap - handle based on policy
                match self.policy {
                    RetentionPolicy::Backpressure => {
                        // Try cleanup first (completed segments only, no data loss)
                        if let Some(callback) = self.cleanup_callback.lock().as_ref() {
                            let cleaned = callback();
                            if cleaned > 0 {
                                // Cleanup freed some space, retry the reservation
                                continue;
                            }
                        }
                        // No cleanup possible or no callback, return backpressure error
                        otel_warn!(
                            "quiver.budget.backpressure",
                            requested = bytes,
                            available = self.cap.saturating_sub(current),
                            cap = self.cap,
                            used = current,
                            policy = "backpressure",
                        );
                        return Err(QuiverError::StorageAtCapacity {
                            requested: bytes,
                            available: self.cap.saturating_sub(wal.saturating_add(current_seg)),
                            cap: self.cap,
                        });
                    }
                    RetentionPolicy::DropOldest => {
                        // Try to reclaim space
                        let needed = wal.saturating_add(new_seg).saturating_sub(self.cap);
                        if let Some(callback) = self.reclaim_callback.lock().as_ref() {
                            let freed = callback(needed);
                            if freed > 0 {
                                // Reclaim freed some space, retry the reservation
                                continue;
                            }
                            // Reclaim couldn't free any space, fall back to backpressure
                            otel_warn!(
                                "quiver.budget.backpressure",
                                requested = bytes,
                                available = self.cap.saturating_sub(current),
                                cap = self.cap,
                                used = current,
                                policy = "drop_oldest",
                                message = "reclaim failed, falling back to backpressure",
                            );
                            return Err(QuiverError::StorageAtCapacity {
                                requested: bytes,
                                available: self
                                    .cap
                                    .saturating_sub(wal.saturating_add(current_seg)),
                                cap: self.cap,
                            });
                        } else {
                            // No callback registered, fall back to backpressure
                            return Err(QuiverError::StorageAtCapacity {
                                requested: bytes,
                                available: self
                                    .cap
                                    .saturating_sub(wal.saturating_add(current_seg)),
                                cap: self.cap,
                            });
                        }
                    }
                }
            }

            // Try to claim the space atomically in the segment pool
            match self.segment_used.compare_exchange_weak(
                current_seg,
                new_seg,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(PendingWrite {
                        budget: self.clone(),
                        reserved: bytes,
                        committed: AtomicBool::new(false),
                    });
                }
                Err(_) => {
                    // CAS failed, retry (will re-read both wal_used and segment_used)
                    continue;
                }
            }
        }
    }

    /// Records existing segment bytes without going through reservation.
    ///
    /// Called during startup to account for segment files from previous runs.
    /// This can exceed the cap (to accurately reflect reality on disk).
    pub fn record_existing_segment(&self, bytes: u64) {
        let _ = self.segment_used.fetch_add(bytes, Ordering::Release);
    }

    /// Releases segment bytes when segment files are deleted.
    pub fn release_segment(&self, bytes: u64) {
        let _ = self
            .segment_used
            .fetch_update(Ordering::Release, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(bytes))
            });
    }

    // ── WAL pool ─────────────────────────────────────────────────────────

    /// Records WAL bytes (appended entries or files discovered at startup).
    ///
    /// WAL bytes are tracked separately from segments but still count against
    /// the shared cap. This allows segment reservations to see the WAL as
    /// unavoidable overhead without double-charging during finalization.
    pub fn record_wal_bytes(&self, bytes: u64) {
        let _ = self.wal_used.fetch_add(bytes, Ordering::Release);
    }

    /// Releases WAL bytes when rotated WAL files are purged.
    pub fn release_wal_bytes(&self, bytes: u64) {
        let _ = self
            .wal_used
            .fetch_update(Ordering::Release, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(bytes))
            });
    }
}

/// Guard for a pending segment write reservation.
///
/// Holds reserved bytes in the **segment pool** until the write completes.
/// Must call [`commit`](Self::commit) with the actual bytes written, or the
/// reservation is released on drop.
pub struct PendingWrite {
    budget: Arc<DiskBudget>,
    reserved: u64,
    committed: AtomicBool,
}

impl PendingWrite {
    /// Returns the number of bytes reserved.
    #[must_use]
    pub const fn reserved(&self) -> u64 {
        self.reserved
    }

    /// Commits the write with the actual bytes written.
    ///
    /// If `actual` differs from the reserved amount, the **segment pool** is adjusted.
    /// - If `actual < reserved`: releases the difference
    /// - If `actual > reserved`: records the additional bytes (may exceed cap briefly)
    pub fn commit(self, actual: u64) {
        self.committed.store(true, Ordering::Release);

        if actual < self.reserved {
            // Release the unused portion from the segment pool
            self.budget.release_segment(self.reserved - actual);
        } else if actual > self.reserved {
            // Record the additional bytes in the segment pool
            self.budget.record_existing_segment(actual - self.reserved);
        }
        // If actual == reserved, nothing to adjust
    }

    /// Aborts the reservation, releasing all reserved bytes.
    ///
    /// Equivalent to dropping without committing, but more explicit.
    pub fn abort(self) {
        // committed stays false, drop will release
    }
}

impl Drop for PendingWrite {
    fn drop(&mut self) {
        // If not committed, release the reserved segment bytes
        if !self.committed.load(Ordering::Acquire) && self.reserved > 0 {
            self.budget.release_segment(self.reserved);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget_starts_empty() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        assert_eq!(budget.used(), 0);
        assert_eq!(budget.wal_used(), 0);
        assert_eq!(budget.segment_used(), 0);
        assert_eq!(budget.headroom(), 1000);
        assert_eq!(budget.cap(), 1000);
    }

    #[test]
    fn unlimited_budget() {
        let budget = DiskBudget::unlimited();
        assert_eq!(budget.cap(), u64::MAX);
    }

    #[test]
    fn try_reserve_segment_succeeds_when_under_cap() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        let pending = budget.try_reserve_segment(500).unwrap();
        assert_eq!(budget.segment_used(), 500);
        assert_eq!(budget.used(), 500);
        assert_eq!(pending.reserved(), 500);
        pending.commit(500);
        assert_eq!(budget.segment_used(), 500);
    }

    #[test]
    fn try_reserve_segment_fails_when_over_cap_backpressure() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        budget.record_existing_segment(800);

        let result = budget.try_reserve_segment(300);
        assert!(matches!(result, Err(QuiverError::StorageAtCapacity { .. })));

        if let Err(QuiverError::StorageAtCapacity {
            requested,
            available,
            cap,
        }) = result
        {
            assert_eq!(requested, 300);
            assert_eq!(available, 200);
            assert_eq!(cap, 1000);
        }
    }

    #[test]
    fn pending_write_releases_on_drop() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        {
            let _pending = budget.try_reserve_segment(500).unwrap();
            assert_eq!(budget.segment_used(), 500);
            // pending drops here without commit
        }
        assert_eq!(budget.segment_used(), 0);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn commit_adjusts_for_actual_size() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // Reserved more than actual
        let pending = budget.try_reserve_segment(500).unwrap();
        pending.commit(300);
        assert_eq!(budget.segment_used(), 300);

        // Reserved less than actual (rare but possible with estimates)
        let pending = budget.try_reserve_segment(100).unwrap();
        pending.commit(150);
        assert_eq!(budget.segment_used(), 450);
    }

    #[test]
    fn release_segment_frees_space() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        budget.record_existing_segment(500);
        assert_eq!(budget.segment_used(), 500);

        budget.release_segment(200);
        assert_eq!(budget.segment_used(), 300);

        // Release more than used (shouldn't underflow)
        budget.release_segment(500);
        assert_eq!(budget.segment_used(), 0);
    }

    #[test]
    fn record_existing_segment_can_exceed_cap() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record_existing_segment(1500);
        assert_eq!(budget.segment_used(), 1500);
        assert_eq!(budget.used(), 1500);
        assert_eq!(budget.headroom(), 0);
    }

    #[test]
    fn wal_and_segment_pools_are_independent() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);

        budget.record_wal_bytes(300);
        budget.record_existing_segment(200);

        assert_eq!(budget.wal_used(), 300);
        assert_eq!(budget.segment_used(), 200);
        assert_eq!(budget.used(), 500);
        assert_eq!(budget.headroom(), 500);

        budget.release_wal_bytes(100);
        assert_eq!(budget.wal_used(), 200);
        assert_eq!(budget.segment_used(), 200);
        assert_eq!(budget.used(), 400);

        budget.release_segment(50);
        assert_eq!(budget.wal_used(), 200);
        assert_eq!(budget.segment_used(), 150);
        assert_eq!(budget.used(), 350);
    }

    #[test]
    fn try_reserve_segment_accounts_for_wal_bytes() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // WAL is consuming 600 bytes
        budget.record_wal_bytes(600);

        // Segment reservation should only have 400 bytes of headroom
        let pending = budget.try_reserve_segment(400).unwrap();
        pending.commit(400);
        assert_eq!(budget.used(), 1000);

        // Further segment reservation should fail
        let result = budget.try_reserve_segment(1);
        assert!(matches!(result, Err(QuiverError::StorageAtCapacity { .. })));
    }

    #[test]
    fn finalization_without_double_charge() {
        // Simulates the finalization flow where WAL bytes are converted to a segment.
        // With split pools, this no longer double-charges.
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // Step 1: WAL has 400 bytes of entries
        budget.record_wal_bytes(400);
        assert_eq!(budget.used(), 400);

        // Step 2: Reserve segment space (400 bytes estimated)
        // Combined check: 400 (wal) + 0 (seg) + 400 (new) = 800 <= 1000 ✓
        let pending = budget.try_reserve_segment(400).unwrap();

        // Step 3: Write segment (actual 350 bytes)
        pending.commit(350);
        // Now: wal=400, seg=350, total=750
        assert_eq!(budget.wal_used(), 400);
        assert_eq!(budget.segment_used(), 350);
        assert_eq!(budget.used(), 750);

        // Step 4: Persist cursor and purge WAL (releases 400 bytes from WAL pool)
        budget.release_wal_bytes(400);
        // Now: wal=0, seg=350, total=350 — correct!
        assert_eq!(budget.wal_used(), 0);
        assert_eq!(budget.segment_used(), 350);
        assert_eq!(budget.used(), 350);
    }

    #[test]
    fn replay_finalization_without_double_charge() {
        // Simulates WAL replay at startup under a tight budget.
        // The WAL bytes were counted at startup, so try_reserve_segment must
        // check against wal_used + segment_used + new, not double-charge.
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // At startup: WAL has 500 bytes, segments have 400 bytes
        budget.record_wal_bytes(500);
        budget.record_existing_segment(400);
        assert_eq!(budget.used(), 900);

        // Replay finalization: reserve ~100 bytes for a segment
        // Combined: 500 (wal) + 400 (seg) + 100 (new) = 1000 <= 1000 ✓
        let pending = budget.try_reserve_segment(100).unwrap();
        pending.commit(100);
        assert_eq!(budget.used(), 1000);

        // Purge WAL files that were finalized (say 200 bytes)
        budget.release_wal_bytes(200);
        assert_eq!(budget.used(), 800);
        assert_eq!(budget.wal_used(), 300);
        assert_eq!(budget.segment_used(), 500);
    }

    #[test]
    fn drop_oldest_invokes_reclaim_callback() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record_existing_segment(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();
        let budget_for_callback = Arc::downgrade(&budget);

        budget.set_reclaim_callback(move |needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            // Simulate reclaiming by releasing from segment budget
            if let Some(b) = budget_for_callback.upgrade() {
                let to_release = needed.min(500); // Release up to 500 bytes
                b.release_segment(to_release);
                to_release
            } else {
                0
            }
        });

        // This would exceed cap, should trigger reclaim
        let pending = budget.try_reserve_segment(200).unwrap();
        assert!(reclaim_count.load(Ordering::Relaxed) >= 1);
        pending.commit(200);
    }

    #[test]
    fn drop_oldest_returns_backpressure_when_reclaim_fails() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record_existing_segment(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();

        // This callback doesn't free any space, simulating a failed reclaim
        budget.set_reclaim_callback(move |_needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            0 // No space freed
        });

        // This would exceed cap, reclaim will be tried but returns 0
        // So we should get a backpressure error (not an infinite loop)
        let result = budget.try_reserve_segment(200);
        assert!(result.is_err());
        match result {
            Err(QuiverError::StorageAtCapacity { .. }) => {} // Expected
            _ => panic!("Expected StorageAtCapacity error"),
        }
        // Callback was invoked exactly once (no infinite retry)
        assert_eq!(reclaim_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn zero_reservation_succeeds() {
        let budget = Arc::new(DiskBudget::new(100, RetentionPolicy::Backpressure));
        budget.record_existing_segment(100); // Segment pool at capacity

        // Zero reservation should still work
        let pending = budget.try_reserve_segment(0).unwrap();
        assert_eq!(pending.reserved(), 0);
        pending.commit(0);
    }

    #[test]
    fn has_ingest_headroom_uses_total() {
        // 1000 cap, tracks both WAL and segment usage
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);

        // With nothing used, we have 1000 available
        assert!(budget.has_ingest_headroom(1000));
        assert!(!budget.has_ingest_headroom(1001));

        // After 300 WAL bytes, we have 700 available
        budget.record_wal_bytes(300);
        assert!(budget.has_ingest_headroom(700));
        assert!(!budget.has_ingest_headroom(701));

        // After 200 segment bytes, we have 500 available
        budget.record_existing_segment(200);
        assert!(budget.has_ingest_headroom(500));
        assert!(!budget.has_ingest_headroom(501));
    }

    #[test]
    fn independent_budgets_do_not_share_state() {
        // Phase 1 approach: each engine gets its own budget
        let per_engine_cap = 500;

        let budget1 = Arc::new(DiskBudget::new(
            per_engine_cap,
            RetentionPolicy::Backpressure,
        ));
        let budget2 = Arc::new(DiskBudget::new(
            per_engine_cap,
            RetentionPolicy::Backpressure,
        ));

        // Each budget starts at 0
        assert_eq!(budget1.used(), 0);
        assert_eq!(budget2.used(), 0);

        // Recording in one budget doesn't affect others
        budget1.record_existing_segment(200);
        assert_eq!(budget1.used(), 200);
        assert_eq!(budget2.used(), 0);

        // Each budget enforces its own cap independently
        let pending = budget1.try_reserve_segment(100).unwrap();
        pending.commit(100);
        assert_eq!(budget1.used(), 300);
        assert_eq!(budget1.headroom(), 200); // 500 - 300

        // budget2 still has full headroom
        assert_eq!(budget2.headroom(), per_engine_cap);

        // Fill budget1 to capacity
        budget1.record_existing_segment(200); // Now at 500 (at cap)
        assert_eq!(budget1.headroom(), 0);

        // budget1 at capacity should reject
        assert!(budget1.try_reserve_segment(1).is_err());

        // budget2 should still accept (independent)
        let pending2 = budget2.try_reserve_segment(100).unwrap();
        pending2.commit(100);
        assert_eq!(budget2.used(), 100);
    }

    #[test]
    fn release_wal_bytes_saturates() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record_wal_bytes(100);
        // Release more than stored — should not underflow
        budget.release_wal_bytes(200);
        assert_eq!(budget.wal_used(), 0);
    }
}
