// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ratio sampler -- emit a fixed fraction of log records.
//!
//! # Algorithm
//!
//! The ratio sampler keeps `emit` out of every `out_of` records.
//! Conceptually, for each record: increment `seen`, and if
//! `emitted < emit` then keep the record and increment `emitted`. When
//! `seen` reaches `out_of`, reset both counters to zero.
//!
//! At the batch level, the number of records to keep is computed in O(1)
//! using a closed-form formula rather than iterating per record. See
//! [RatioSampler::compute_to_keep].

use super::Sampler;
use arrow::array::BooleanArray;
use arrow::buffer::BooleanBuffer;
use async_trait::async_trait;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::otap::OtapArrowRecords;
use serde::Deserialize;

/// Configuration for ratio sampling.
///
/// Emits `emit` out of every `out_of` log records.
#[derive(Debug, Clone, Deserialize)]
pub struct RatioConfig {
    /// Numerator of the sampling fraction.
    pub emit: usize,
    /// Denominator of the sampling fraction.
    pub out_of: usize,
}

impl RatioConfig {
    /// Validates the ratio sampling configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.emit == 0 {
            return Err(ConfigError::InvalidUserConfig {
                error: "ratio.emit must be greater than 0".to_string(),
            });
        }
        if self.out_of == 0 {
            return Err(ConfigError::InvalidUserConfig {
                error: "ratio.out_of must be greater than 0".to_string(),
            });
        }
        if self.emit > self.out_of {
            return Err(ConfigError::InvalidUserConfig {
                error: "ratio.emit must be less than or equal to ratio.out_of".to_string(),
            });
        }
        Ok(())
    }
}

/// Ratio sampler state.
///
/// Tracks position within the emit/out_of cycle across batches.
/// Uses an O(1) formula to compute how many records to keep from each
/// batch without iterating per-record.
#[derive(Debug)]
pub struct RatioSampler {
    /// Numerator: records to emit per cycle.
    emit: usize,
    /// Denominator: cycle length.
    out_of: usize,
    /// Records emitted in the current (partial) cycle.
    emitted: usize,
    /// Records seen in the current (partial) cycle.
    seen: usize,
}

impl RatioSampler {
    /// Creates a new ratio sampler from configuration.
    pub fn new(cfg: &RatioConfig) -> Self {
        Self {
            emit: cfg.emit,
            out_of: cfg.out_of,
            emitted: 0,
            seen: 0,
        }
    }

    /// Compute how many records to keep from a batch of the given size.
    ///
    /// Uses an O(1) closed-form formula that accounts for the current
    /// position within the emit/out_of cycle, any number of full cycles
    /// within the batch, and the leftover partial cycle at the end.
    ///
    /// ## Example
    ///
    /// With `emit: 2, out_of: 5`:
    ///
    /// | Batch | Size | State before      | to_keep | State after       |
    /// |-------|------|-------------------|---------|-------------------|
    /// | 1     | 12   | emitted=0, seen=0 | 6       | emitted=2, seen=2 |
    /// | 2     | 4    | emitted=2, seen=2 | 1       | emitted=1, seen=1 |
    /// | 3     | 5    | emitted=1, seen=1 | 2       | emitted=1, seen=1 |
    /// | 4     | 4    | emitted=1, seen=1 | 1       | emitted=2, seen=4 |
    ///
    /// Total in: 25, total kept: 10 (exactly 2 out of 5).
    fn compute_to_keep(&mut self, batch_size: usize) -> usize {
        if batch_size == 0 {
            return 0;
        }

        // Finish the current cycle
        let remaining_in_cycle = self.out_of - self.seen;
        let from_current = batch_size.min(remaining_in_cycle);
        let keep_from_current = (self.emit.saturating_sub(self.emitted)).min(from_current);

        // Full cycles + leftover
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

#[async_trait(?Send)]
impl Sampler for RatioSampler {
    fn sample_arrow_records(&mut self, records: &OtapArrowRecords) -> BooleanArray {
        let total = records.root_record_batch().map_or(0, |rb| rb.num_rows());
        let to_keep = self.compute_to_keep(total);
        BooleanArray::new(BooleanBuffer::collect_bool(total, |i| i < to_keep), None)
    }

    async fn ensure_init(
        &mut self,
        _effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError> {
        Ok(())
    }

    fn notify_timer_tick(&mut self) {
        // Noop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::log_sampling_processor::sample::testing::make_log_records;

    #[test]
    fn test_ratio_basic_1_10() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        let mut sampler = RatioSampler::new(&cfg);

        let records = make_log_records(100);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 10);
    }

    #[test]
    fn test_ratio_full_passthrough() {
        let cfg = RatioConfig { emit: 1, out_of: 1 };
        let mut sampler = RatioSampler::new(&cfg);

        let records = make_log_records(100);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.true_count(), 100);
    }

    #[test]
    fn test_ratio_across_batches() {
        let cfg = RatioConfig { emit: 2, out_of: 5 };
        let mut sampler = RatioSampler::new(&cfg);

        let cases = [(12, 6), (4, 1), (5, 2), (4, 1)];
        for (batch_size, expected_kept) in cases {
            let records = make_log_records(batch_size);
            let sel = sampler.sample_arrow_records(&records);
            assert_eq!(
                sel.true_count(),
                expected_kept,
                "batch_size={batch_size}: expected {expected_kept}, got {}",
                sel.true_count()
            );
        }
    }

    #[test]
    fn test_ratio_empty_batch() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        let mut sampler = RatioSampler::new(&cfg);

        let records = make_log_records(0);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.len(), 0);
        assert_eq!(sel.true_count(), 0);
    }

    #[test]
    fn test_ratio_selection_vector_shape() {
        // With emit=1 out_of=3 and batch=6, we should keep 2 records at positions 0 and 3
        let cfg = RatioConfig { emit: 1, out_of: 3 };
        let mut sampler = RatioSampler::new(&cfg);

        let records = make_log_records(6);
        let sel = sampler.sample_arrow_records(&records);
        assert_eq!(sel.len(), 6);
        assert_eq!(sel.true_count(), 2);
        // First 2 are true (the "first N" pattern), rest false
        assert!(sel.value(0));
        assert!(sel.value(1));
        assert!(!sel.value(2));
    }

    #[test]
    fn test_ratio_matches_reference_impl() {
        let test_cases: Vec<(usize, usize, Vec<usize>)> = vec![
            (1, 10, vec![100, 50, 37, 13]),
            (2, 5, vec![12, 4, 5, 4]),
            (3, 7, vec![21, 7, 14, 28]),
            (1, 1, vec![100, 50]),
            (5, 5, vec![100]),
        ];

        for (emit, out_of, batches) in test_cases {
            let cfg = RatioConfig { emit, out_of };
            let mut sampler = RatioSampler::new(&cfg);
            let mut ref_emitted = 0;
            let mut ref_seen = 0;

            for batch_size in batches {
                let records = make_log_records(batch_size);
                let sel = sampler.sample_arrow_records(&records);
                let actual = sel.true_count();
                let expected = reference_compute_to_keep(
                    emit,
                    out_of,
                    &mut ref_emitted,
                    &mut ref_seen,
                    batch_size,
                );
                assert_eq!(
                    actual, expected,
                    "emit={emit}, out_of={out_of}, batch_size={batch_size}: O(1)={actual}, O(B)={expected}",
                );
            }
        }
    }

    /// Reference implementation using O(B) loop to verify O(1) formula.
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

    // ==================== Config Validation Tests ====================

    #[test]
    fn test_valid_config() {
        let cfg = RatioConfig {
            emit: 1,
            out_of: 10,
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_valid_config_equal() {
        let cfg = RatioConfig { emit: 1, out_of: 1 };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_invalid_emit_zero() {
        let cfg = RatioConfig {
            emit: 0,
            out_of: 10,
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("emit"));
    }

    #[test]
    fn test_invalid_out_of_zero() {
        let cfg = RatioConfig { emit: 1, out_of: 0 };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("out_of"));
    }

    #[test]
    fn test_invalid_emit_greater_than_out_of() {
        let cfg = RatioConfig {
            emit: 10,
            out_of: 5,
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.to_string().contains("emit"));
    }
}
