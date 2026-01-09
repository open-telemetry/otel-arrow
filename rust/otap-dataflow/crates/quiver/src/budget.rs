// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Disk budget management for enforcing storage caps.
//!
//! The [`DiskBudget`] provides a shared, thread-safe mechanism for tracking
//! and limiting disk usage across multiple Quiver engines and their components
//! (WAL, segments, progress files).
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
//! # Sharing Across Engines
//!
//! A single `DiskBudget` can be shared across multiple `QuiverEngine` instances
//! to enforce a global disk cap:
//!
//! ```ignore
//! let budget = Arc::new(DiskBudget::new(global_cap));
//! let engine1 = QuiverEngine::new(config1, budget.clone())?;
//! let engine2 = QuiverEngine::new(config2, budget.clone())?;
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::config::RetentionPolicy;
use crate::error::{QuiverError, Result};

/// Callback type for reclaiming space when using `DropOldest` policy.
///
/// The callback receives the number of bytes needed and should attempt to
/// free at least that much space by deleting old segments.
pub type ReclaimCallback = Box<dyn Fn(u64) + Send + Sync>;

/// Callback type for cleanup (completed segments only, no data loss).
///
/// Returns the number of segments cleaned up.
pub type CleanupCallback = Box<dyn Fn() -> usize + Send + Sync>;

/// Shared disk budget for enforcing storage caps.
///
/// Thread-safe and designed to be shared across multiple engines via `Arc`.
pub struct DiskBudget {
    /// Maximum allowed bytes.
    cap: u64,
    /// Current bytes in use.
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
            .finish()
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
    pub fn cap(&self) -> u64 {
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

    /// Returns the configured retention policy.
    #[must_use]
    pub fn policy(&self) -> RetentionPolicy {
        self.policy
    }

    /// Sets a callback for reclaiming space when using `DropOldest` policy.
    ///
    /// The callback is invoked when a reservation would exceed the cap and
    /// the policy is `DropOldest`. It receives the number of bytes needed
    /// and should attempt to free space by deleting old segments.
    pub fn set_reclaim_callback<F>(&self, callback: F)
    where
        F: Fn(u64) + Send + Sync + 'static,
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
                            callback(needed);
                            // Retry after reclaim attempt
                            continue;
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
        let _ = self.used.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Releases bytes when files are deleted.
    ///
    /// Called when WAL files are purged or segments are deleted.
    pub fn release(&self, bytes: u64) {
        // Saturating sub to avoid underflow if accounting is slightly off
        let _ = self.used.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
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
    pub fn reserved(&self) -> u64 {
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
                b.release(needed.min(500)); // Release up to 500 bytes
            }
        });

        // This would exceed cap, should trigger reclaim
        let pending = budget.try_reserve(200).unwrap();
        assert!(reclaim_count.load(Ordering::Relaxed) >= 1);
        pending.commit(200);
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
}
