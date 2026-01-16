// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Disk budget management for enforcing storage caps.
//!
//! The [`DiskBudget`] provides a thread-safe mechanism for tracking and limiting
//! disk usage within a single Quiver engine (WAL, segments, progress files).
//!
//! # Hard Cap Enforcement
//!
//! The budget uses atomic operations to guarantee that concurrent writers
//! cannot collectively exceed the configured cap. Each write must reserve
//! space before proceeding:
//!
//! ```ignore
//! let pending = budget.try_reserve(estimated_bytes)?;
//! let actual = write_file(...)?;
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

/// Error returned when budget configuration is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetConfigError {
    /// The cap is too small for the required reserved headroom.
    CapTooSmall {
        /// The requested cap.
        cap: u64,
        /// The calculated reserved headroom.
        reserved_headroom: u64,
        /// The minimum cap required.
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
                reserved_headroom,
                min_cap,
                segment_size_bytes,
                wal_max_size_bytes,
            } => {
                let cap_mb = cap / (1024 * 1024);
                let reserved_mb = reserved_headroom / (1024 * 1024);
                let min_mb = min_cap / (1024 * 1024);
                let seg_mb = segment_size_bytes / (1024 * 1024);
                let wal_mb = wal_max_size_bytes / (1024 * 1024);
                write!(
                    f,
                    "Budget cap {} MB is too small. With segment_size={} MB and WAL max={} MB, \
                     reserved headroom is {} MB. Minimum cap required: {} MB",
                    cap_mb, seg_mb, wal_mb, reserved_mb, min_mb
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
/// # Headroom Reservation
///
/// The budget supports a "reserved headroom" which is space held back for
/// internal operations like WAL rotation and segment finalization. When
/// checking if there's enough headroom for a new write, the reserved portion
/// is subtracted from the available space. This prevents deadlocks where
/// ingestion fills the budget completely, leaving no room for cleanup operations.
pub struct DiskBudget {
    /// Maximum allowed bytes.
    cap: u64,
    /// Current bytes in use.
    used: AtomicU64,
    /// Reserved headroom for internal operations (WAL rotation, segment finalization).
    /// This is subtracted from the available headroom in `has_ingest_headroom()`.
    reserved_headroom: u64,
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
            .field("used", &self.used.load(Ordering::Relaxed))
            .field("reserved_headroom", &self.reserved_headroom)
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
            used: AtomicU64::new(0),
            reserved_headroom: 0,
            policy,
            reclaim_callback: Mutex::new(None),
            cleanup_callback: Mutex::new(None),
        }
    }

    /// Creates a new disk budget with reserved headroom.
    ///
    /// The `reserved_headroom` is space held back for internal operations like
    /// WAL rotation and segment finalization. Ingestion will be backpressured
    /// when available headroom drops below this threshold.
    ///
    /// # Arguments
    ///
    /// * `cap` - Maximum bytes allowed.
    /// * `policy` - What to do when the cap is exceeded.
    /// * `reserved_headroom` - Bytes to reserve for internal operations.
    #[must_use]
    pub fn with_reserved_headroom(
        cap: u64,
        policy: RetentionPolicy,
        reserved_headroom: u64,
    ) -> Self {
        Self {
            cap,
            used: AtomicU64::new(0),
            reserved_headroom,
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

    /// Creates a disk budget for a single engine with automatically calculated headroom.
    ///
    /// This is the recommended way to create a budget for production use. It calculates
    /// the appropriate reserved headroom based on segment and WAL sizes to ensure the
    /// engine can always complete segment finalization without exceeding the cap.
    ///
    /// # Phase 1: Static Quotas
    ///
    /// When running multiple engines (e.g., one per CPU core), divide the global cap
    /// by the number of engines and call this for each:
    ///
    /// ```ignore
    /// let per_engine_cap = global_cap / num_engines;
    /// let budget = DiskBudget::for_engine(per_engine_cap, policy, segment_size, wal_max_size)?;
    /// ```
    ///
    /// # Headroom Calculation
    ///
    /// Reserved headroom = `segment_size + (wal_max_size / 4)`:
    /// - **segment_size**: Covers the transient overlap during finalization when both
    ///   the WAL entries AND the new segment file exist before the WAL is purged.
    /// - **wal_max_size / 4**: Buffer for bundles arriving during finalization and
    ///   early accumulation for the next segment.
    ///
    /// # Errors
    ///
    /// Returns an error if the cap is too small for the reserved headroom plus at
    /// least one segment's worth of working space.
    pub fn for_engine(
        cap: u64,
        policy: RetentionPolicy,
        segment_size_bytes: u64,
        wal_max_size_bytes: u64,
    ) -> std::result::Result<Self, BudgetConfigError> {
        let reserved_headroom = Self::calculate_headroom(segment_size_bytes, wal_max_size_bytes);
        let min_cap = reserved_headroom + segment_size_bytes;

        if cap < min_cap {
            return Err(BudgetConfigError::CapTooSmall {
                cap,
                reserved_headroom,
                min_cap,
                segment_size_bytes,
                wal_max_size_bytes,
            });
        }

        Ok(Self::with_reserved_headroom(cap, policy, reserved_headroom))
    }

    /// Calculates the recommended reserved headroom for given segment and WAL sizes.
    ///
    /// Formula: `segment_size + (wal_max_size / 4)`
    ///
    /// See [`for_engine`](Self::for_engine) for rationale.
    #[must_use]
    pub const fn calculate_headroom(segment_size_bytes: u64, wal_max_size_bytes: u64) -> u64 {
        segment_size_bytes + (wal_max_size_bytes / 4)
    }

    /// Returns the minimum cap required for given segment and WAL sizes.
    ///
    /// This is `reserved_headroom + segment_size` (need room for at least one segment).
    #[must_use]
    pub const fn minimum_cap(segment_size_bytes: u64, wal_max_size_bytes: u64) -> u64 {
        let reserved = Self::calculate_headroom(segment_size_bytes, wal_max_size_bytes);
        reserved + segment_size_bytes
    }

    /// Returns the configured cap.
    #[must_use]
    pub const fn cap(&self) -> u64 {
        self.cap
    }

    /// Returns the current bytes in use.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    /// Returns the remaining headroom before the cap.
    #[must_use]
    pub fn headroom(&self) -> u64 {
        self.cap.saturating_sub(self.used.load(Ordering::Relaxed))
    }

    /// Returns the reserved headroom for internal operations.
    #[must_use]
    pub const fn reserved_headroom(&self) -> u64 {
        self.reserved_headroom
    }

    /// Checks if there is sufficient headroom for ingestion.
    ///
    /// Returns `true` if the available headroom (after accounting for reserved
    /// headroom) is at least `bytes`. This is used to apply backpressure at
    /// the ingestion boundary, leaving room for internal operations like WAL
    /// rotation and segment finalization.
    ///
    /// This is a "soft" check - it does not reserve space. Use this to decide
    /// whether to accept new data, then use `try_reserve` for the actual
    /// reservation.
    #[must_use]
    pub fn has_ingest_headroom(&self, bytes: u64) -> bool {
        let available = self.headroom().saturating_sub(self.reserved_headroom);
        available >= bytes
    }

    /// Returns the configured retention policy.
    #[must_use]
    pub fn policy(&self) -> RetentionPolicy {
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

    /// Attempts to reserve bytes for a pending write.
    ///
    /// Returns a [`PendingWrite`] guard that holds the reservation. The guard
    /// must be committed with the actual bytes written, or it will release
    /// the reservation on drop (for error handling).
    ///
    /// # Errors
    ///
    /// Returns [`QuiverError::StorageAtCapacity`] if the reservation would
    /// exceed the cap and the policy is `Backpressure`.
    ///
    /// With `DropOldest` policy, this will invoke the reclaim callback to
    /// attempt to free space before failing.
    pub fn try_reserve(self: &Arc<Self>, bytes: u64) -> Result<PendingWrite> {
        // Fast path: if bytes is 0, no reservation needed
        if bytes == 0 {
            return Ok(PendingWrite {
                budget: self.clone(),
                reserved: 0,
                committed: AtomicBool::new(false),
            });
        }

        // Try to atomically reserve the space
        loop {
            let current = self.used.load(Ordering::Acquire);
            let new_used = current.saturating_add(bytes);

            if new_used > self.cap {
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
                        return Err(QuiverError::StorageAtCapacity {
                            requested: bytes,
                            available: self.cap.saturating_sub(current),
                            cap: self.cap,
                        });
                    }
                    RetentionPolicy::DropOldest => {
                        // Try to reclaim space
                        let needed = new_used - self.cap;
                        if let Some(callback) = self.reclaim_callback.lock().as_ref() {
                            let freed = callback(needed);
                            if freed > 0 {
                                // Reclaim freed some space, retry the reservation
                                continue;
                            }
                            // Reclaim couldn't free any space, fall back to backpressure
                            return Err(QuiverError::StorageAtCapacity {
                                requested: bytes,
                                available: self.cap.saturating_sub(current),
                                cap: self.cap,
                            });
                        } else {
                            // No callback registered, fall back to backpressure
                            return Err(QuiverError::StorageAtCapacity {
                                requested: bytes,
                                available: self.cap.saturating_sub(current),
                                cap: self.cap,
                            });
                        }
                    }
                }
            }

            // Try to claim the space atomically
            match self.used.compare_exchange_weak(
                current,
                new_used,
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
                    // CAS failed, retry
                    continue;
                }
            }
        }
    }

    /// Records existing usage without going through reservation.
    ///
    /// Called during startup to account for files from previous runs.
    /// This can exceed the cap (to accurately reflect reality).
    pub fn record_existing(&self, bytes: u64) {
        let _ = self.used.fetch_add(bytes, Ordering::Release);
    }

    /// Releases bytes when files are deleted.
    ///
    /// Called when WAL files are purged or segments are deleted.
    pub fn release(&self, bytes: u64) {
        // Saturating sub to avoid underflow if accounting is slightly off
        let _ = self
            .used
            .fetch_update(Ordering::Release, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(bytes))
            });
    }
}

/// Guard for a pending write reservation.
///
/// Holds reserved bytes until the write completes. Must call [`commit`](Self::commit)
/// with the actual bytes written, or the reservation is released on drop.
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
    /// If `actual` differs from the reserved amount, the budget is adjusted.
    /// - If `actual < reserved`: releases the difference
    /// - If `actual > reserved`: records the additional bytes (may exceed cap briefly)
    pub fn commit(self, actual: u64) {
        self.committed.store(true, Ordering::Release);

        if actual < self.reserved {
            // Release the unused portion
            self.budget.release(self.reserved - actual);
        } else if actual > self.reserved {
            // Record the additional bytes (we reserved less than needed)
            self.budget.record_existing(actual - self.reserved);
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
        // If not committed, release the reserved bytes
        if !self.committed.load(Ordering::Acquire) && self.reserved > 0 {
            self.budget.release(self.reserved);
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
        assert_eq!(budget.headroom(), 1000);
        assert_eq!(budget.cap(), 1000);
    }

    #[test]
    fn unlimited_budget() {
        let budget = DiskBudget::unlimited();
        assert_eq!(budget.cap(), u64::MAX);
    }

    #[test]
    fn try_reserve_succeeds_when_under_cap() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        let pending = budget.try_reserve(500).unwrap();
        assert_eq!(budget.used(), 500);
        assert_eq!(pending.reserved(), 500);
        pending.commit(500);
        assert_eq!(budget.used(), 500);
    }

    #[test]
    fn try_reserve_fails_when_over_cap_backpressure() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        budget.record_existing(800);

        let result = budget.try_reserve(300);
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
            let _pending = budget.try_reserve(500).unwrap();
            assert_eq!(budget.used(), 500);
            // pending drops here without commit
        }
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn commit_adjusts_for_actual_size() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // Reserved more than actual
        let pending = budget.try_reserve(500).unwrap();
        pending.commit(300);
        assert_eq!(budget.used(), 300);

        // Reserved less than actual (rare but possible with estimates)
        let pending = budget.try_reserve(100).unwrap();
        pending.commit(150);
        assert_eq!(budget.used(), 450);
    }

    #[test]
    fn release_frees_space() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));
        budget.record_existing(500);
        assert_eq!(budget.used(), 500);

        budget.release(200);
        assert_eq!(budget.used(), 300);

        // Release more than used (shouldn't underflow)
        budget.release(500);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn record_existing_can_exceed_cap() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record_existing(1500);
        assert_eq!(budget.used(), 1500);
        assert_eq!(budget.headroom(), 0);
    }

    #[test]
    fn drop_oldest_invokes_reclaim_callback() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record_existing(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();
        let budget_for_callback = Arc::downgrade(&budget);

        budget.set_reclaim_callback(move |needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            // Simulate reclaiming by releasing from budget
            if let Some(b) = budget_for_callback.upgrade() {
                let to_release = needed.min(500); // Release up to 500 bytes
                b.release(to_release);
                to_release
            } else {
                0
            }
        });

        // This would exceed cap, should trigger reclaim
        let pending = budget.try_reserve(200).unwrap();
        assert!(reclaim_count.load(Ordering::Relaxed) >= 1);
        pending.commit(200);
    }

    #[test]
    fn drop_oldest_returns_backpressure_when_reclaim_fails() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record_existing(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();

        // This callback doesn't free any space, simulating a failed reclaim
        budget.set_reclaim_callback(move |_needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            0 // No space freed
        });

        // This would exceed cap, reclaim will be tried but returns 0
        // So we should get a backpressure error (not an infinite loop)
        let result = budget.try_reserve(200);
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
        budget.record_existing(100); // At capacity

        // Zero reservation should still work
        let pending = budget.try_reserve(0).unwrap();
        assert_eq!(pending.reserved(), 0);
        pending.commit(0);
    }

    #[test]
    fn with_reserved_headroom_constructor() {
        let budget = DiskBudget::with_reserved_headroom(1000, RetentionPolicy::Backpressure, 200);
        assert_eq!(budget.cap(), 1000);
        assert_eq!(budget.reserved_headroom(), 200);
        assert_eq!(budget.used(), 0);
        assert_eq!(budget.headroom(), 1000);
    }

    #[test]
    fn has_ingest_headroom_accounts_for_reserved() {
        // 1000 cap with 200 reserved = 800 available for ingest
        let budget = DiskBudget::with_reserved_headroom(1000, RetentionPolicy::Backpressure, 200);

        // With nothing used, we have 800 available for ingest
        assert!(budget.has_ingest_headroom(800));
        assert!(budget.has_ingest_headroom(799));
        assert!(!budget.has_ingest_headroom(801));

        // After using 300, we have 500 available for ingest (1000 - 300 - 200)
        budget.record_existing(300);
        assert!(budget.has_ingest_headroom(500));
        assert!(!budget.has_ingest_headroom(501));

        // After using 800, we have 0 available for ingest (1000 - 800 - 200)
        budget.record_existing(500); // Now at 800 used
        assert!(budget.has_ingest_headroom(0));
        assert!(!budget.has_ingest_headroom(1));

        // Note: headroom() still reports 200, but has_ingest_headroom sees 0
        assert_eq!(budget.headroom(), 200);
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
        budget1.record_existing(200);
        assert_eq!(budget1.used(), 200);
        assert_eq!(budget2.used(), 0);

        // Each budget enforces its own cap independently
        let pending = budget1.try_reserve(100).unwrap();
        pending.commit(100);
        assert_eq!(budget1.used(), 300);
        assert_eq!(budget1.headroom(), 200); // 500 - 300

        // budget2 still has full headroom
        assert_eq!(budget2.headroom(), per_engine_cap);

        // Fill budget1 to capacity
        budget1.record_existing(200); // Now at 500 (at cap)
        assert_eq!(budget1.headroom(), 0);

        // budget1 at capacity should reject
        assert!(budget1.try_reserve(1).is_err());

        // budget2 should still accept (independent)
        let pending2 = budget2.try_reserve(100).unwrap();
        pending2.commit(100);
        assert_eq!(budget2.used(), 100);
    }

    #[test]
    fn per_engine_headroom_isolation() {
        // Each engine in Phase 1 has its own reserved headroom
        let per_engine_cap = 500;
        let reserved = 100;

        let budget1 = DiskBudget::with_reserved_headroom(
            per_engine_cap,
            RetentionPolicy::Backpressure,
            reserved,
        );
        let budget2 = DiskBudget::with_reserved_headroom(
            per_engine_cap,
            RetentionPolicy::Backpressure,
            reserved,
        );

        // Each has 400 available for ingest (500 - 100 reserved)
        assert!(budget1.has_ingest_headroom(400));
        assert!(budget2.has_ingest_headroom(400));

        // Use up budget1's ingest headroom
        budget1.record_existing(400);
        assert!(!budget1.has_ingest_headroom(1));

        // budget2 is unaffected
        assert!(budget2.has_ingest_headroom(400));
    }

    #[test]
    fn calculate_headroom_formula() {
        // Formula: segment_size + (wal_max_size / 4)
        let segment = 32 * 1024 * 1024; // 32 MB
        let wal_max = 128 * 1024 * 1024; // 128 MB

        let headroom = DiskBudget::calculate_headroom(segment, wal_max);
        // 32 MB + 32 MB = 64 MB
        assert_eq!(headroom, 64 * 1024 * 1024);

        // Edge case: no WAL
        let headroom_no_wal = DiskBudget::calculate_headroom(segment, 0);
        assert_eq!(headroom_no_wal, segment);
    }

    #[test]
    fn minimum_cap_calculation() {
        let segment = 32 * 1024 * 1024; // 32 MB
        let wal_max = 128 * 1024 * 1024; // 128 MB

        let min_cap = DiskBudget::minimum_cap(segment, wal_max);
        // headroom (64 MB) + segment (32 MB) = 96 MB
        assert_eq!(min_cap, 96 * 1024 * 1024);
    }

    #[test]
    fn for_engine_succeeds_with_sufficient_cap() {
        let segment = 32 * 1024 * 1024; // 32 MB
        let wal_max = 128 * 1024 * 1024; // 128 MB
        let cap = 150 * 1024 * 1024; // 150 MB (> 96 MB minimum)

        let budget = DiskBudget::for_engine(cap, RetentionPolicy::Backpressure, segment, wal_max)
            .expect("should succeed");

        assert_eq!(budget.cap(), cap);
        assert_eq!(budget.reserved_headroom(), 64 * 1024 * 1024);
        // Available for ingest = 150 - 64 = 86 MB
        assert!(budget.has_ingest_headroom(86 * 1024 * 1024));
        assert!(!budget.has_ingest_headroom(87 * 1024 * 1024));
    }

    #[test]
    fn for_engine_fails_with_insufficient_cap() {
        let segment = 32 * 1024 * 1024; // 32 MB
        let wal_max = 128 * 1024 * 1024; // 128 MB
        let cap = 64 * 1024 * 1024; // 64 MB (< 96 MB minimum)

        let result = DiskBudget::for_engine(cap, RetentionPolicy::Backpressure, segment, wal_max);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BudgetConfigError::CapTooSmall { .. }));

        // Check error message formatting
        let msg = format!("{}", err);
        assert!(msg.contains("64 MB is too small"));
        assert!(msg.contains("Minimum cap required: 96 MB"));
    }

    #[test]
    fn for_engine_exactly_at_minimum() {
        let segment = 32 * 1024 * 1024; // 32 MB
        let wal_max = 128 * 1024 * 1024; // 128 MB
        let min_cap = DiskBudget::minimum_cap(segment, wal_max); // 96 MB

        // Exactly at minimum should succeed
        let budget =
            DiskBudget::for_engine(min_cap, RetentionPolicy::Backpressure, segment, wal_max)
                .expect("should succeed at minimum cap");

        assert_eq!(budget.cap(), min_cap);

        // One byte below should fail
        let result =
            DiskBudget::for_engine(min_cap - 1, RetentionPolicy::Backpressure, segment, wal_max);
        assert!(result.is_err());
    }
}
