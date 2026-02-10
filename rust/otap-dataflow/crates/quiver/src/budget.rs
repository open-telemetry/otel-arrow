// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Watermark-based disk budget for enforcing storage caps.
//!
//! The [`DiskBudget`] is a self-accounting watermark, not a filesystem
//! reservation. It tracks what Quiver has written and deleted, providing
//! two thresholds:
//!
//! - **`soft_cap`** — gates ingest. When `used > soft_cap`, the engine
//!   should stop accepting new data (backpressure or drop-oldest).
//! - **`hard_cap`** — the ceiling that `used` never exceeds under normal
//!   operation. Because `soft_cap = hard_cap - segment_headroom`, the
//!   maximum overshoot from a single segment finalization stays within
//!   `hard_cap`.
//!
//! # Operations
//!
//! - **`add(bytes)`** — records bytes written to disk (WAL appends,
//!   segment writes, files discovered at startup).
//! - **`remove(bytes)`** — records bytes deleted from disk (WAL purge,
//!   segment deletion). Saturates at zero.
//! - **`is_over_soft_cap()`** — returns `true` when ingest should be
//!   gated.
//!
//! # Hard Cap Guarantee
//!
//! The minimum configuration requirement is:
//!
//! ```text
//! hard_cap >= wal_max + 2 * segment_target_size
//! ```
//!
//! This ensures that when the WAL is full and the budget is at `soft_cap`,
//! there is still room for exactly one segment finalization (up to
//! `segment_target_size` bytes) without exceeding `hard_cap`.
//!
//! **Assumption:** only one segment is accumulating at a time. If the
//! engine is extended to support multiple concurrent open segments, the
//! headroom must be widened to `max_concurrent_segments * segment_target_size`
//! and the minimum budget raised accordingly.
//!
//! # Multi-Engine Deployment (Phase 1: Static Quotas)
//!
//! Each engine receives its own `DiskBudget` with a static quota
//! (global cap / number of engines). No cross-engine coordination.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::config::RetentionPolicy;

/// Watermark-based disk budget for enforcing storage caps.
///
/// Thread-safe for sharing between WAL writer, segment store,
/// and engine components via `Arc`.
pub struct DiskBudget {
    /// Hard ceiling: `used` must never exceed this.
    hard_cap: u64,
    /// Soft threshold: ingest is gated when `used > soft_cap`.
    /// Equals `hard_cap - segment_headroom`.
    soft_cap: u64,
    /// Current bytes tracked on disk.
    used: AtomicU64,
    /// Retention policy (Backpressure or DropOldest).
    policy: RetentionPolicy,
}

impl std::fmt::Debug for DiskBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskBudget")
            .field("hard_cap", &self.hard_cap)
            .field("soft_cap", &self.soft_cap)
            .field("used", &self.used.load(Ordering::Relaxed))
            .field("policy", &self.policy)
            .finish()
    }
}

impl DiskBudget {
    /// Creates a new disk budget.
    ///
    /// # Arguments
    ///
    /// * `hard_cap` — Maximum bytes allowed on disk. Use `u64::MAX` for
    ///   effectively unlimited.
    /// * `segment_headroom` — Space reserved for one segment finalization.
    ///   Typically `segment_target_size`. The soft cap is computed as
    ///   `hard_cap - segment_headroom`.
    /// * `policy` — Retention policy when the soft cap is exceeded.
    #[must_use]
    pub fn new(hard_cap: u64, segment_headroom: u64, policy: RetentionPolicy) -> Self {
        Self {
            hard_cap,
            soft_cap: hard_cap.saturating_sub(segment_headroom),
            used: AtomicU64::new(0),
            policy,
        }
    }

    /// Creates an unlimited budget (no cap enforcement).
    ///
    /// Useful for testing or when disk limits are managed externally.
    #[must_use]
    pub fn unlimited() -> Self {
        Self::new(u64::MAX, 0, RetentionPolicy::Backpressure)
    }

    /// Returns the hard cap (maximum bytes on disk).
    #[must_use]
    pub const fn hard_cap(&self) -> u64 {
        self.hard_cap
    }

    /// Returns the soft cap (ingest threshold).
    #[must_use]
    pub const fn soft_cap(&self) -> u64 {
        self.soft_cap
    }

    /// Returns the total bytes currently tracked on disk.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }

    /// Returns remaining headroom before the soft cap.
    #[must_use]
    pub fn headroom(&self) -> u64 {
        self.soft_cap.saturating_sub(self.used())
    }

    /// Returns `true` when `used` exceeds the soft cap.
    ///
    /// The engine should stop accepting new ingest when this returns `true`.
    #[must_use]
    pub fn is_over_soft_cap(&self) -> bool {
        self.used() > self.soft_cap
    }

    /// Returns the configured retention policy.
    #[must_use]
    pub const fn policy(&self) -> RetentionPolicy {
        self.policy
    }

    /// Records bytes written to disk.
    ///
    /// Called after WAL appends, segment writes, or when discovering
    /// existing files at startup. May push `used` above `soft_cap`
    /// (that's expected — finalization overshoot is bounded by
    /// `segment_headroom`).
    pub fn add(&self, bytes: u64) {
        let _ = self.used.fetch_add(bytes, Ordering::Release);
    }

    /// Records bytes deleted from disk.
    ///
    /// Called after WAL purge or segment deletion. Saturates at zero.
    pub fn remove(&self, bytes: u64) {
        let _ = self
            .used
            .fetch_update(Ordering::Release, Ordering::Acquire, |current| {
                Some(current.saturating_sub(bytes))
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_budget_starts_empty() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        assert_eq!(budget.used(), 0);
        assert_eq!(budget.hard_cap(), 1000);
        assert_eq!(budget.soft_cap(), 800);
        assert_eq!(budget.headroom(), 800);
    }

    #[test]
    fn unlimited_budget() {
        let budget = DiskBudget::unlimited();
        assert_eq!(budget.hard_cap(), u64::MAX);
        assert_eq!(budget.soft_cap(), u64::MAX);
        assert!(!budget.is_over_soft_cap());
    }

    #[test]
    fn add_tracks_used_bytes() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        budget.add(300);
        assert_eq!(budget.used(), 300);
        assert_eq!(budget.headroom(), 500);
        assert!(!budget.is_over_soft_cap());
    }

    #[test]
    fn is_over_soft_cap_at_threshold() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        // soft_cap = 800

        budget.add(800);
        assert!(!budget.is_over_soft_cap()); // at cap but not over

        budget.add(1);
        assert!(budget.is_over_soft_cap()); // over soft_cap

        budget.remove(1);
        assert!(!budget.is_over_soft_cap()); // back to exactly soft_cap
    }

    #[test]
    fn add_can_exceed_hard_cap() {
        // add() doesn't gate — it reflects reality
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        budget.add(1500);
        assert_eq!(budget.used(), 1500);
        assert!(budget.is_over_soft_cap());
        assert_eq!(budget.headroom(), 0);
    }

    #[test]
    fn remove_frees_space() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        budget.add(500);
        assert_eq!(budget.used(), 500);

        budget.remove(200);
        assert_eq!(budget.used(), 300);
    }

    #[test]
    fn remove_saturates_at_zero() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        budget.add(100);
        budget.remove(200);
        assert_eq!(budget.used(), 0);
    }

    #[test]
    fn finalization_stays_within_hard_cap() {
        // Simulates: WAL at max (128), segments use soft_cap - 128 = 672.
        // Finalization adds one segment (200 = segment_headroom).
        // Total: 672 + 128 + 200 = 1000 = hard_cap. ✓
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);

        // WAL bytes
        budget.add(128);
        // Existing segments fill to soft_cap
        budget.add(672);
        assert_eq!(budget.used(), 800);
        assert!(!budget.is_over_soft_cap());

        // Finalization writes one more segment (up to segment_headroom)
        budget.add(200);
        assert_eq!(budget.used(), 1000);
        assert!(budget.is_over_soft_cap());
        // At hard_cap but not exceeding it
        assert_eq!(budget.used(), budget.hard_cap());

        // WAL purge after finalization
        budget.remove(128);
        assert_eq!(budget.used(), 872);
    }

    #[test]
    fn finalization_without_double_charge() {
        // Simulates the finalization flow: WAL bytes → segment bytes → WAL purge.
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);

        // WAL has 400 bytes
        budget.add(400);
        assert_eq!(budget.used(), 400);

        // Segment finalization writes 350 bytes
        budget.add(350);
        assert_eq!(budget.used(), 750);

        // WAL purge releases 400 bytes
        budget.remove(400);
        assert_eq!(budget.used(), 350);
    }

    #[test]
    fn replay_finalization_without_double_charge() {
        let budget = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);

        // At startup: WAL=500, segments=300
        budget.add(500);
        budget.add(300);
        assert_eq!(budget.used(), 800);

        // Replay finalization: write segment (100 bytes)
        budget.add(100);
        assert_eq!(budget.used(), 900);

        // Purge WAL files (200 bytes)
        budget.remove(200);
        assert_eq!(budget.used(), 700);
    }

    #[test]
    fn independent_budgets_do_not_share_state() {
        let budget1 = DiskBudget::new(500, 100, RetentionPolicy::Backpressure);
        let budget2 = DiskBudget::new(500, 100, RetentionPolicy::Backpressure);

        budget1.add(200);
        assert_eq!(budget1.used(), 200);
        assert_eq!(budget2.used(), 0);

        budget2.add(300);
        assert_eq!(budget1.used(), 200);
        assert_eq!(budget2.used(), 300);
    }

    #[test]
    fn policy_is_accessible() {
        let bp = DiskBudget::new(1000, 200, RetentionPolicy::Backpressure);
        assert_eq!(bp.policy(), RetentionPolicy::Backpressure);

        let drop = DiskBudget::new(1000, 200, RetentionPolicy::DropOldest);
        assert_eq!(drop.policy(), RetentionPolicy::DropOldest);
    }

    #[test]
    fn soft_cap_saturates_when_headroom_exceeds_cap() {
        // Edge case: segment_headroom > hard_cap
        let budget = DiskBudget::new(100, 200, RetentionPolicy::Backpressure);
        assert_eq!(budget.soft_cap(), 0);
        // Any bytes at all would be over soft cap
        budget.add(1);
        assert!(budget.is_over_soft_cap());
    }
}
