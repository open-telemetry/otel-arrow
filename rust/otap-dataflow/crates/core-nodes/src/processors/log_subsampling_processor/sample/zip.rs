// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Zip sampler -- emit at most N log records per time window.
//!
//! # Algorithm
//!
//! The zip sampler maintains a running `count` of records emitted in the
//! current window. On each incoming batch of size B:
//!
//! 1. `budget = max_items - count`
//! 2. `to_keep = min(budget, B)`
//! 3. `count += to_keep`
//!
//! A `BooleanArray` selection vector is produced with the first `to_keep`
//! entries set to `true` and the rest `false`.
//!
//! # Timer lifecycle
//!
//! On the first message, [`ZipSampler::ensure_init`] registers a periodic
//! timer via the engine's effect handler. The engine delivers
//! `NodeControlMsg::TimerTick` at a fixed, drift-free cadence equal to
//! `interval`. On each tick, [`ZipSampler::notify_timer`] resets `count`
//! to zero, opening a fresh budget for the next window.
//!
//! # Configuration
//!
//! | Field       | Type     | Required | Description                    |
//! |-------------|----------|----------|--------------------------------|
//! | `interval`  | duration | yes      | Length of the sampling window   |
//! | `max_items` | integer  | yes      | Max records to emit per window  |
//!
//! Constraints: `interval > 0`, `max_items > 0`.

use super::Sampler;
use arrow::array::BooleanArray;
use arrow::buffer::BooleanBuffer;
use async_trait::async_trait;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::otap::OtapArrowRecords;
use serde::Deserialize;
use std::time::Duration;

/// Configuration for zip sampling.
///
/// Emits at most `max_items` log records per `interval` time window.
#[derive(Debug, Clone, Deserialize)]
pub struct ZipConfig {
    /// Length of the sampling window (e.g., "60s", "1m").
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Maximum records to emit per window.
    pub max_items: usize,
}

/// Zip sampler state.
///
/// Keeps a running count of emitted records and resets on each timer tick.
#[derive(Debug)]
pub struct ZipSampler {
    /// Maximum items to emit per window.
    max_items: usize,
    /// Count of items emitted in the current window.
    count: usize,
    /// Timer interval for the sampling window.
    timer_interval: Duration,
    /// Whether the periodic timer has been started.
    timer_started: bool,
}

impl ZipSampler {
    /// Creates a new zip sampler from configuration.
    pub fn new(cfg: &ZipConfig) -> Self {
        Self {
            max_items: cfg.max_items,
            count: 0,
            timer_interval: cfg.interval,
            timer_started: false,
        }
    }

    /// Compute how many records to keep from a batch of the given size.
    fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        let budget = self.max_items.saturating_sub(self.count);
        let to_keep = budget.min(batch_size);
        self.count += to_keep;
        to_keep
    }

    /// Reset the count for a new time window.
    fn reset(&mut self) {
        self.count = 0;
    }
}

#[async_trait(?Send)]
impl Sampler for ZipSampler {
    fn sample_arrow_records(&mut self, records: &OtapArrowRecords) -> BooleanArray {
        let total = records.root_record_batch().map_or(0, |rb| rb.num_rows());
        let to_keep = self.compute_to_keep(total);
        BooleanArray::new(BooleanBuffer::collect_bool(total, |i| i < to_keep), None)
    }

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

    fn notify_timer(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::log_subsampling_processor::sample::testing::make_log_records;

    #[test]
    fn test_zip_within_budget() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut sampler = ZipSampler::new(&cfg);

        let records = make_log_records(30);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 30);

        let records = make_log_records(50);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 50);
    }

    #[test]
    fn test_zip_exceeds_budget() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut sampler = ZipSampler::new(&cfg);

        let records = make_log_records(90);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 90);

        let records = make_log_records(50);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 10);
    }

    #[test]
    fn test_zip_budget_exhausted() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut sampler = ZipSampler::new(&cfg);

        let records = make_log_records(100);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 100);

        let records = make_log_records(50);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 0);
    }

    #[test]
    fn test_zip_reset_via_notify_timer() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut sampler = ZipSampler::new(&cfg);

        // Fill budget
        let records = make_log_records(100);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 100);

        let records = make_log_records(50);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 0);

        // Timer resets the budget
        sampler.notify_timer();

        let records = make_log_records(50);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 50);
    }

    #[test]
    fn test_zip_empty_batch() {
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 100,
        };
        let mut sampler = ZipSampler::new(&cfg);

        let records = make_log_records(0);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.len(), 0);
        assert_eq!(sel.true_count(), 0);
    }

    #[test]
    fn test_zip_selection_vector_shape() {
        // Verify the selection vector has the right pattern: first N true, rest false
        let cfg = ZipConfig {
            interval: Duration::from_secs(60),
            max_items: 3,
        };
        let mut sampler = ZipSampler::new(&cfg);

        let records = make_log_records(5);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.len(), 5);
        assert!(sel.value(0)); // kept
        assert!(sel.value(1)); // kept
        assert!(sel.value(2)); // kept
        assert!(!sel.value(3)); // dropped
        assert!(!sel.value(4)); // dropped
    }
}
