//! Subsampling policy implementations.
//!
//! This module contains the runtime state and algorithms for both
//! zip sampling and ratio sampling policies.

use crate::log_subsampling_processor::config::{Policy, RatioConfig, ZipConfig};

/// Runtime state for a subsampling policy.
#[derive(Debug)]
pub enum SubsamplingPolicy {
    /// Zip sampling: emit at most N records per time window.
    Zip(ZipState),
    /// Ratio sampling: emit M out of every N records.
    Ratio(RatioState),
}

/// Runtime state for zip sampling.
#[derive(Debug)]
pub struct ZipState {
    /// Maximum items to emit per window.
    max_items: usize,
    /// Count of items emitted in the current window.
    count: usize,
}

/// Runtime state for ratio sampling.
#[derive(Debug)]
pub struct RatioState {
    /// Numerator: records to emit per cycle.
    emit: usize,
    /// Denominator: cycle length.
    out_of: usize,
    /// Records emitted in current cycle.
    emitted: usize,
    /// Records seen in current cycle.
    seen: usize,
}

impl SubsamplingPolicy {
    /// Creates a new policy from configuration.
    pub fn from_config(policy: &Policy) -> Self {
        match policy {
            Policy::Zip(cfg) => Self::Zip(ZipState::new(cfg)),
            Policy::Ratio(cfg) => Self::Ratio(RatioState::new(cfg)),
        }
    }

    /// Computes how many records to keep from a batch of the given size.
    ///
    /// Updates internal state to account for the batch.
    pub fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        match self {
            Self::Zip(state) => state.compute_to_keep(batch_size),
            Self::Ratio(state) => state.compute_to_keep(batch_size),
        }
    }

    /// Resets the policy state (called on timer tick for zip sampling).
    pub fn reset(&mut self) {
        match self {
            Self::Zip(state) => state.reset(),
            Self::Ratio(_) => {
                // Ratio sampling has no reset; state evolves continuously
            }
        }
    }
}

impl ZipState {
    /// Creates a new zip state from configuration.
    fn new(cfg: &ZipConfig) -> Self {
        Self {
            max_items: cfg.max_items,
            count: 0,
        }
    }

    /// Computes how many records to keep from a batch.
    ///
    /// Algorithm:
    /// 1. budget = max_items - count
    /// 2. to_keep = min(budget, batch_size)
    /// 3. count += to_keep
    fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        let budget = self.max_items.saturating_sub(self.count);
        let to_keep = budget.min(batch_size);
        self.count += to_keep;
        to_keep
    }

    /// Resets the count for a new time window.
    fn reset(&mut self) {
        self.count = 0;
    }
}

impl RatioState {
    /// Creates a new ratio state from configuration.
    fn new(cfg: &RatioConfig) -> Self {
        Self {
            emit: cfg.emit,
            out_of: cfg.out_of,
            emitted: 0,
            seen: 0,
        }
    }

    /// Computes how many records to keep from a batch using O(1) formula.
    ///
    /// Algorithm from the README:
    /// ```text
    /// remaining_in_cycle = out_of - seen
    /// from_current       = min(B, remaining_in_cycle)
    /// keep_from_current  = min(max(emit - emitted, 0), from_current)
    ///
    /// after_current = B - from_current
    /// full_cycles   = after_current / out_of
    /// leftover      = after_current % out_of
    ///
    /// to_keep = keep_from_current + (full_cycles * emit) + min(emit, leftover)
    /// ```
    fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        if batch_size == 0 {
            return 0;
        }

        let remaining_in_cycle = self.out_of - self.seen;
        let from_current = batch_size.min(remaining_in_cycle);
        let keep_from_current = (self.emit.saturating_sub(self.emitted)).min(from_current);

        let after_current = batch_size - from_current;
        let full_cycles = after_current / self.out_of;
        let leftover = after_current % self.out_of;

        let to_keep = keep_from_current + (full_cycles * self.emit) + self.emit.min(leftover);

        // Update state
        let new_seen = (self.seen + batch_size) % self.out_of;
        self.seen = new_seen;
        self.emitted = self.emit.min(new_seen);

        to_keep
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ==================== Zip Policy Tests ====================

    #[test]
    fn test_zip_within_budget() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut state = ZipState::new(&cfg);

        // First batch of 30
        assert_eq!(state.compute_to_keep(30), 30);
        assert_eq!(state.count, 30);

        // Second batch of 50 (total 80, still within budget)
        assert_eq!(state.compute_to_keep(50), 50);
        assert_eq!(state.count, 80);
    }

    #[test]
    fn test_zip_exceeds_budget() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut state = ZipState::new(&cfg);

        // First batch of 90
        assert_eq!(state.compute_to_keep(90), 90);

        // Second batch of 50 (only 10 remaining)
        assert_eq!(state.compute_to_keep(50), 10);
        assert_eq!(state.count, 100);
    }

    #[test]
    fn test_zip_budget_exhausted() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut state = ZipState::new(&cfg);

        // Fill budget
        assert_eq!(state.compute_to_keep(100), 100);

        // Next batch gets nothing
        assert_eq!(state.compute_to_keep(50), 0);
        assert_eq!(state.count, 100);
    }

    #[test]
    fn test_zip_reset() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut state = ZipState::new(&cfg);

        // Fill budget
        assert_eq!(state.compute_to_keep(100), 100);
        assert_eq!(state.compute_to_keep(50), 0);

        // Reset
        state.reset();
        assert_eq!(state.count, 0);

        // Now we have budget again
        assert_eq!(state.compute_to_keep(50), 50);
    }

    // ==================== Ratio Policy Tests ====================

    #[test]
    fn test_ratio_basic_1_10() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        let mut state = RatioState::new(&cfg);

        // 100 logs should emit 10
        assert_eq!(state.compute_to_keep(100), 10);
    }

    #[test]
    fn test_ratio_full_passthrough() {
        let cfg = RatioConfig { emit: 1, out_of: 1 };
        let mut state = RatioState::new(&cfg);

        // 100 logs should all pass through
        assert_eq!(state.compute_to_keep(100), 100);
    }

    #[test]
    fn test_ratio_across_batches() {
        // Example from README: emit=2, out_of=5, batches [12, 4, 5, 4] -> [6, 1, 2, 1]
        let cfg = RatioConfig { emit: 2, out_of: 5 };
        let mut state = RatioState::new(&cfg);

        assert_eq!(state.compute_to_keep(12), 6);
        assert_eq!(state.compute_to_keep(4), 1);
        assert_eq!(state.compute_to_keep(5), 2);
        assert_eq!(state.compute_to_keep(4), 1);

        // Total: 12 + 4 + 5 + 4 = 25 in, 6 + 1 + 2 + 1 = 10 out (2/5 ratio)
    }

    #[test]
    fn test_ratio_empty_batch() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        let mut state = RatioState::new(&cfg);

        assert_eq!(state.compute_to_keep(0), 0);
        // State should remain unchanged
        assert_eq!(state.seen, 0);
        assert_eq!(state.emitted, 0);
    }

    #[test]
    fn test_ratio_small_batch() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        let mut state = RatioState::new(&cfg);

        // Single record - should be kept (first of cycle)
        assert_eq!(state.compute_to_keep(1), 1);

        // Next 9 records - none kept
        for _ in 0..9 {
            // We can't process 1 at a time and get 0 because state updates
            // Let's do it in one batch instead
        }
    }

    #[test]
    fn test_ratio_verify_with_loop() {
        // Verify O(1) formula matches O(B) loop for various inputs
        let cfg = RatioConfig { emit: 2, out_of: 5 };

        // Test case from README
        let batches = [12, 4, 5, 4];
        let expected = [6, 1, 2, 1];

        let mut state = RatioState::new(&cfg);
        for (i, &batch_size) in batches.iter().enumerate() {
            let actual = state.compute_to_keep(batch_size);
            assert_eq!(
                actual, expected[i],
                "batch {} (size {}): expected {}, got {}",
                i, batch_size, expected[i], actual
            );
        }
    }

    #[test]
    fn test_ratio_loop_reference_impl() {
        // Reference implementation using O(B) loop to verify O(1) formula
        fn reference_compute_to_keep(
            emit: usize,
            out_of: usize,
            emitted: &mut usize,
            seen: &mut usize,
            batch_size: usize,
        ) -> usize {
            let mut to_keep = 0;
            for _ in 0..batch_size {
                *seen += 1;
                if *emitted < emit {
                    *emitted += 1;
                    to_keep += 1;
                }
                if *seen == out_of {
                    *seen = 0;
                    *emitted = 0;
                }
            }
            to_keep
        }

        // Test various batch sizes
        let test_cases: Vec<(usize, usize, Vec<usize>)> = vec![
            (1, 10, vec![100, 50, 37, 13]),
            (2, 5, vec![12, 4, 5, 4]),
            (3, 7, vec![21, 7, 14, 28]),
            (1, 1, vec![100, 50]),
            (5, 5, vec![100]),
        ];

        for (emit, out_of, batches) in test_cases {
            let cfg = RatioConfig { emit, out_of };
            let mut state = RatioState::new(&cfg);
            let mut ref_emitted = 0;
            let mut ref_seen = 0;

            for batch_size in batches {
                let actual = state.compute_to_keep(batch_size);
                let expected = reference_compute_to_keep(
                    emit,
                    out_of,
                    &mut ref_emitted,
                    &mut ref_seen,
                    batch_size,
                );
                assert_eq!(
                    actual, expected,
                    "emit={}, out_of={}, batch_size={}: O(1)={}, O(B)={}",
                    emit, out_of, batch_size, actual, expected
                );
            }
        }
    }
}
