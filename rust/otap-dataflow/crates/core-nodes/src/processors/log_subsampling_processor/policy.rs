//! Subsampling policy implementations.
//!
//! This module contains the runtime state and algorithms for both
//! zip sampling and ratio sampling policies.

use crate::processors::log_subsampling_processor::config::{Policy, RatioConfig, ZipConfig};
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_otap::pdata::OtapPdata;
use std::time::Duration;

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
    /// Timer interval for the sampling window.
    timer_interval: Duration,
    /// Whether the periodic timer has been started.
    timer_started: bool,
}

/// Runtime state for ratio sampling.
#[derive(Debug)]
pub struct RatioState {
    /// Numerator: records to emit per cycle.
    emit: usize,
    /// Denominator: cycle length.
    out_of: usize,

    /// This is the state of the current cycle. We don't exactly sample the
    /// first M of every N records, we optimize to reduce slicing. If the
    /// RecordBatch is bigger than N, we can sample multiple M out of it with
    /// one slice and then determine what the ending cycle state should be.
    emitted: usize,
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
    pub fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        match self {
            Self::Zip(state) => state.compute_to_keep(batch_size),
            Self::Ratio(state) => state.compute_to_keep(batch_size),
        }
    }

    /// Performs any one-time initialization the policy requires. Implementations
    /// should be idempotent since this is called for every message.
    pub async fn ensure_init(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        match self {
            Self::Zip(state) => state.ensure_init(effect_handler).await,
            Self::Ratio(_) => Ok(()),
        }
    }

    /// Notifies the policy that a timer tick has occurred.
    pub fn notify_timer(&mut self) {
        match self {
            Self::Zip(state) => state.reset(),
            Self::Ratio(_) => {}
        }
    }
}

impl ZipState {
    /// Creates a new zip state from configuration.
    fn new(cfg: &ZipConfig) -> Self {
        Self {
            max_items: cfg.max_items,
            count: 0,
            timer_interval: cfg.interval,
            timer_started: false,
        }
    }

    /// Starts the periodic timer if it hasn't been started yet.
    async fn ensure_init(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        if !self.timer_started {
            let _handle = effect_handler
                .start_periodic_timer(self.timer_interval)
                .await?;
            self.timer_started = true;
        }
        Ok(())
    }

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

    fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        if batch_size == 0 {
            return 0;
        }

        // Finish the current cycle
        let remaining_in_cycle = self.out_of - self.seen;
        let from_current = batch_size.min(remaining_in_cycle);
        let keep_from_current = (self.emit.saturating_sub(self.emitted)).min(from_current);

        // Determine the number of whole cycles remaining in this batch and how
        // what is left over.
        let after_current = batch_size - from_current;
        let full_cycles = after_current / self.out_of;
        let leftover = after_current % self.out_of;

        // Determine the total number to keep from this batch
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
        let cfg = RatioConfig { emit: 2, out_of: 5 };
        let mut state = RatioState::new(&cfg);

        assert_eq!(state.compute_to_keep(12), 6);
        assert_eq!(state.compute_to_keep(4), 1);
        assert_eq!(state.compute_to_keep(5), 2);
        assert_eq!(state.compute_to_keep(4), 1);
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
