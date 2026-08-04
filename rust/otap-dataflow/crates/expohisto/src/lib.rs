// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free OpenTelemetry exponential histogram.
//!
//! Exponential histograms provide a compact, high-resolution representation of
//! value distributions using logarithmically-spaced bucket boundaries. This
//! implementation stores bucket counters in a fixed-size data pool sized with a
//! const generic ([`HistogramNN<N>`]), performs no heap allocation, and
//! contains no `unsafe` code.
//!
//! [`HistogramNN<N>`] is positive-only: negative values are rejected. This
//! suits the common case of non-negative measurements such as latencies,
//! sizes, and counts.
//!
//! Bucket index mapping is accelerated by a compile-time lookup table checked in
//! under `src/lookup_tables.rs`. These tables are generated data and require no build step.

#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod exponent;
pub(crate) mod float64;
pub mod histogram;
pub mod mapping;

#[doc(hidden)]
pub mod lookup;

pub use histogram::{
    BucketTotals, BucketView, BucketsIter, Error, HistogramNN, HistogramView, Settings, Stats,
    Width,
};
pub use mapping::{MIN_SCALE, Scale, ScaleError, table_scale};

#[cfg(test)]
mod tests {
    use super::{Error, HistogramNN};
    use std::num::NonZeroU64;

    /// Scenario: A subnormal value, which is too small to carry an exponent of
    /// its own, is recorded alongside the smallest normal value.
    /// Guarantees: the subnormal is rounded up into the smallest normal
    /// bucket rather than dropped or mapped outside the pool, while the sum
    /// and the minimum still report the magnitude that was actually observed.
    #[test]
    fn a_subnormal_value_rounds_up_into_the_smallest_normal_bucket() {
        let subnormal = f64::MIN_POSITIVE / 4.0;
        assert!(subnormal > 0.0 && subnormal < f64::MIN_POSITIVE);

        let mut hist: HistogramNN<8> = HistogramNN::new();
        hist.update(subnormal).expect("subnormals are recordable");
        hist.update(f64::MIN_POSITIVE)
            .expect("the smallest normal is recordable");

        let view = hist.view();
        assert_eq!(view.stats().count, 2);
        assert_eq!(
            view.positive().len(),
            1,
            "the subnormal shares the smallest normal bucket"
        );
        assert_eq!(view.positive().iter().next(), Some(2));
        assert_eq!(view.stats().min, subnormal);
        assert_eq!(view.stats().max, f64::MIN_POSITIVE);
        assert_eq!(view.stats().sum, subnormal + f64::MIN_POSITIVE);
    }

    /// Scenario: NaN and infinity are offered to a histogram that already
    /// holds an observation.
    /// Guarantees: both are refused as extreme, and the refusal leaves the
    /// count, sum, and extremes exactly as they were, so a caller that ignores
    /// the error does not silently corrupt the aggregate.
    #[test]
    fn non_finite_values_are_refused_without_recording() {
        let mut hist: HistogramNN<8> = HistogramNN::new();
        hist.update(1.0).expect("positive value is recordable");
        let before = hist.view().stats();

        for value in [f64::NAN, f64::INFINITY] {
            assert_eq!(hist.update(value), Err(Error::Extreme));
        }

        let after = hist.view().stats();
        assert_eq!(after.count, before.count);
        assert_eq!(after.sum, before.sum);
        assert_eq!(after.min, before.min);
        assert_eq!(after.max, before.max);
        assert_eq!(hist.view().positive().len(), 1);
    }

    /// Scenario: an observation is recorded into a histogram whose total count
    /// already sits at `u64::MAX`.
    /// Guarantees: the increment is refused as an overflow before anything is
    /// written, so the aggregate does not wrap around to a smaller count than
    /// it has already reported.
    #[test]
    fn an_overflowing_count_is_refused_without_recording() {
        let mut hist: HistogramNN<8> = HistogramNN::new();
        hist.record_incr(1.0, NonZeroU64::new(u64::MAX).expect("non-zero"))
            .expect("the first increment fits");
        let before = hist.view().stats();
        let buckets: Vec<u64> = hist.view().positive().iter().collect();

        assert_eq!(hist.update(1.0), Err(Error::Overflow));

        let after = hist.view().stats();
        assert_eq!(after.count, before.count);
        assert_eq!(after.sum, before.sum);
        assert_eq!(hist.view().positive().iter().collect::<Vec<_>>(), buckets);
    }

    /// Scenario: A fresh positive-only histogram records several finite
    /// observations and exposes aggregate statistics through its view.
    /// Guarantees: `update` accepts positive values, and the resulting view
    /// reports the exact observation count and sum, confirming the vendored
    /// lookup tables and bucket accounting are wired correctly.
    #[test]
    fn records_observations_and_reports_stats() {
        let mut hist: HistogramNN<16> = HistogramNN::new();
        for v in [1.5_f64, 2.7, 100.0] {
            hist.update(v).expect("positive value is recordable");
        }

        let view = hist.view();
        let stats = view.stats();
        assert_eq!(stats.count, 3);
        assert!((stats.sum - 104.2).abs() < 1e-9);
        assert_eq!(stats.min, 1.5);
        assert_eq!(stats.max, 100.0);
    }

    /// Scenario: A histogram with stale counters throughout its circular
    /// storage is cleared and then reused for a different value range.
    /// Guarantees: lazily clearing words before they re-enter the active
    /// window prevents observations from before `clear` from reappearing.
    #[test]
    fn clear_discards_stale_circular_storage() {
        let mut hist: HistogramNN<16> = HistogramNN::new();
        for exponent in -20..=20 {
            hist.update(2_f64.powi(exponent)).unwrap();
        }

        hist.clear();
        for value in [0.5, 1.0, 2.0] {
            hist.update(value).unwrap();
        }

        let view = hist.view();
        assert_eq!(view.stats().count, 3);
        assert_eq!(view.stats().sum, 3.5);
        assert_eq!(view.scan_buckets(|_| {}).positive_total, 3);
    }

    /// Scenario: A negative value is recorded in a build with debug
    /// assertions on.
    /// Guarantees: The non-negative domain is enforced loudly, so a caller
    /// recording one is told. Gated on the assertion profile because a release
    /// build compiles the check out and records the magnitude instead.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn asserts_non_negative() {
        let mut hist: HistogramNN<16> = HistogramNN::new();
        hist.update(-1.0).expect("actually not ok");
    }
}
