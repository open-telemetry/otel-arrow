// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Disk budget management for enforcing storage caps.
//!
//! The [`DiskBudget`] provides a thread-safe mechanism for tracking and limiting
//! disk usage within a single Quiver engine. It is deliberately unaware of
//! internal Quiver concepts like WAL files and segments — it simply tracks
//! bytes on disk against a configured cap.
//!
//! # Operations
//!
//! Three operations cover all budget accounting:
//!
//! - **`record(bytes)`** – unconditionally adds bytes (WAL writes, discovered
//!   files at startup). May exceed the cap to accurately reflect reality.
//! - **`release(bytes)`** – removes bytes (WAL purge, segment deletion).
//!   Saturates at zero.
//! - **`try_reserve(bytes)`** – atomically checks `used + bytes <= cap` and
//!   increments on success. Used for segment finalization where exceeding
//!   the cap must be prevented.
//!
//! # Reservation Pattern
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
pub struct DiskBudget {
    /// Maximum allowed bytes on disk.
    cap: u64,
    /// Current bytes in use on disk.
    used: AtomicU64,
    /// Policy when cap is exceeded.
    policy: RetentionPolicy,
    /// Callback for reclaiming space (used with `DropOldest`).
    reclaim_callback: Mutex<Option<ReclaimCallback>>,
    /// Callback for cleanup (completed segments only, safe for `Backpressure`).
    cleanup_callback: Mutex<Option<CleanupCallback>>,
}

impl std::fmt::Debug for DiskBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskBudget")
            .field("cap", &self.cap)
            .field("used", &self.used.load(Ordering::Relaxed))
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

    /// Returns the total bytes in use.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    /// Returns the remaining headroom before the cap.
    #[must_use]
    pub fn headroom(&self) -> u64 {
        self.cap.saturating_sub(self.used())
    }

    /// Checks if there is sufficient headroom for ingestion.
    ///
    /// Returns `true` if the available headroom is at least `bytes`.
    /// This is a "soft" check — it does not reserve space. Use this to decide
    /// whether to accept new data, then use [`try_reserve`](Self::try_reserve)
    /// for the actual reservation.
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
            let new = current.saturating_add(bytes);

            if new > self.cap {
                // Would exceed cap — handle based on policy
                match self.policy {
                    RetentionPolicy::Backpressure => {
                        // Try cleanup first (completed segments only, no data loss)
                        if let Some(callback) = self.cleanup_callback.lock().as_ref() {
                            let cleaned = callback();
                            if cleaned > 0 {
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
                            available: self.cap.saturating_sub(current),
                            cap: self.cap,
                        });
                    }
                    RetentionPolicy::DropOldest => {
                        let needed = new.saturating_sub(self.cap);
                        if let Some(callback) = self.reclaim_callback.lock().as_ref() {
                            let freed = callback(needed);
                            if freed > 0 {
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
                                available: self.cap.saturating_sub(current),
                                cap: self.cap,
                            });
                        } else {
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
            match self
                .used
                .compare_exchange_weak(current, new, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    return Ok(PendingWrite {
                        budget: self.clone(),
                        reserved: bytes,
                        committed: AtomicBool::new(false),
                    });
                }
                Err(_) => continue,
            }
        }
    }

    /// Records bytes on disk without going through reservation.
    ///
    /// Called to account for files that already exist (WAL writes, files
    /// discovered at startup). This can exceed the cap to accurately
    /// reflect reality on disk.
    pub fn record(&self, bytes: u64) {
        let _ = self.used.fetch_add(bytes, Ordering::Release);
    }

    /// Releases bytes when files are deleted or purged.
    ///
    /// Saturates at zero — releasing more than `used` will not underflow.
    pub fn release(&self, bytes: u64) {
        let _ = self
            .used
            .fetch_update(Ordering::Release, Ordering::Relaxed, |current| {
                Some(current.saturating_sub(bytes))
            });
    }
}

/// Guard for a pending write reservation.
///
/// Holds reserved bytes until the write completes. Must call
/// [`commit`](Self::commit) with the actual bytes written, or the
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
    /// If `actual` differs from the reserved amount, the budget is adjusted.
    /// - If `actual < reserved`: releases the difference
    /// - If `actual > reserved`: records the additional bytes (may exceed cap briefly)
    pub fn commit(self, actual: u64) {
        self.committed.store(true, Ordering::Release);

        if actual < self.reserved {
            self.budget.release(self.reserved - actual);
        } else if actual > self.reserved {
            self.budget.record(actual - self.reserved);
        }
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
        budget.record(800);

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
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record(500);
        assert_eq!(budget.used(), 500);

        budget.release(200);
        assert_eq!(budget.used(), 300);

        // Release more than used (shouldn't underflow)
        budget.release(500);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn record_can_exceed_cap() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record(1500);
        assert_eq!(budget.used(), 1500);
        assert_eq!(budget.headroom(), 0);
    }

    #[test]
    fn try_reserve_accounts_for_recorded_bytes() {
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // Record 600 bytes (e.g., WAL on startup)
        budget.record(600);

        // Reservation should only have 400 bytes of headroom
        let pending = budget.try_reserve(400).unwrap();
        pending.commit(400);
        assert_eq!(budget.used(), 1000);

        // Further reservation should fail
        let result = budget.try_reserve(1);
        assert!(matches!(result, Err(QuiverError::StorageAtCapacity { .. })));
    }

    #[test]
    fn finalization_without_double_charge() {
        // Simulates the finalization flow where WAL bytes are converted to a segment.
        // The engine records WAL bytes via `record`, reserves segment space via
        // `try_reserve`, then releases WAL bytes via `release` after purge.
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // Step 1: WAL has 400 bytes of entries
        budget.record(400);
        assert_eq!(budget.used(), 400);

        // Step 2: Reserve segment space (400 bytes estimated)
        // Check: 400 (used) + 400 (new) = 800 <= 1000 ✓
        let pending = budget.try_reserve(400).unwrap();

        // Step 3: Write segment (actual 350 bytes)
        pending.commit(350);
        // Now: used = 750 (400 WAL + 350 segment on disk)
        assert_eq!(budget.used(), 750);

        // Step 4: Purge WAL (releases 400 bytes)
        budget.release(400);
        // Now: used = 350 (only segment remains)
        assert_eq!(budget.used(), 350);
    }

    #[test]
    fn replay_finalization_without_double_charge() {
        // Simulates WAL replay at startup under a tight budget.
        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::Backpressure));

        // At startup: WAL has 500 bytes, segments have 400 bytes
        budget.record(500); // WAL files
        budget.record(400); // existing segments
        assert_eq!(budget.used(), 900);

        // Replay finalization: reserve ~100 bytes for a new segment
        // Check: 900 + 100 = 1000 <= 1000 ✓
        let pending = budget.try_reserve(100).unwrap();
        pending.commit(100);
        assert_eq!(budget.used(), 1000);

        // Purge WAL files that were finalized (say 200 bytes)
        budget.release(200);
        assert_eq!(budget.used(), 800);
    }

    #[test]
    fn drop_oldest_invokes_reclaim_callback() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();
        let budget_for_callback = Arc::downgrade(&budget);

        budget.set_reclaim_callback(move |needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            if let Some(b) = budget_for_callback.upgrade() {
                let to_release = needed.min(500);
                b.release(to_release);
                to_release
            } else {
                0
            }
        });

        let pending = budget.try_reserve(200).unwrap();
        assert!(reclaim_count.load(Ordering::Relaxed) >= 1);
        pending.commit(200);
    }

    #[test]
    fn drop_oldest_returns_backpressure_when_reclaim_fails() {
        use std::sync::atomic::AtomicUsize;

        let budget = Arc::new(DiskBudget::new(1000, RetentionPolicy::DropOldest));
        budget.record(900);

        let reclaim_count = Arc::new(AtomicUsize::new(0));
        let reclaim_count_clone = reclaim_count.clone();

        budget.set_reclaim_callback(move |_needed| {
            let _ = reclaim_count_clone.fetch_add(1, Ordering::Relaxed);
            0
        });

        let result = budget.try_reserve(200);
        assert!(result.is_err());
        match result {
            Err(QuiverError::StorageAtCapacity { .. }) => {}
            _ => panic!("Expected StorageAtCapacity error"),
        }
        assert_eq!(reclaim_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn zero_reservation_succeeds() {
        let budget = Arc::new(DiskBudget::new(100, RetentionPolicy::Backpressure));
        budget.record(100);

        let pending = budget.try_reserve(0).unwrap();
        assert_eq!(pending.reserved(), 0);
        pending.commit(0);
    }

    #[test]
    fn has_ingest_headroom() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);

        assert!(budget.has_ingest_headroom(1000));
        assert!(!budget.has_ingest_headroom(1001));

        budget.record(300);
        assert!(budget.has_ingest_headroom(700));
        assert!(!budget.has_ingest_headroom(701));

        budget.record(200);
        assert!(budget.has_ingest_headroom(500));
        assert!(!budget.has_ingest_headroom(501));
    }

    #[test]
    fn independent_budgets_do_not_share_state() {
        let budget1 = Arc::new(DiskBudget::new(500, RetentionPolicy::Backpressure));
        let budget2 = Arc::new(DiskBudget::new(500, RetentionPolicy::Backpressure));

        assert_eq!(budget1.used(), 0);
        assert_eq!(budget2.used(), 0);

        budget1.record(200);
        assert_eq!(budget1.used(), 200);
        assert_eq!(budget2.used(), 0);

        let pending = budget1.try_reserve(100).unwrap();
        pending.commit(100);
        assert_eq!(budget1.used(), 300);
        assert_eq!(budget1.headroom(), 200);
        assert_eq!(budget2.headroom(), 500);

        budget1.record(200);
        assert_eq!(budget1.headroom(), 0);
        assert!(budget1.try_reserve(1).is_err());

        let pending2 = budget2.try_reserve(100).unwrap();
        pending2.commit(100);
        assert_eq!(budget2.used(), 100);
    }

    #[test]
    fn release_saturates_at_zero() {
        let budget = DiskBudget::new(1000, RetentionPolicy::Backpressure);
        budget.record(100);
        budget.release(200);
        assert_eq!(budget.used(), 0);
    }
}
