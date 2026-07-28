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
    BucketView, BucketsIter, Error, HistogramNN, HistogramView, Settings, Stats, Width,
};
pub use mapping::{MIN_SCALE, Scale, ScaleError, table_scale};

#[cfg(test)]
mod tests {
    use super::HistogramNN;

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

    /// Scenario: The positive-only histogram is asked to record a negative
    /// value.
    /// Guarantees: `update` rejects the value with an error instead of
    /// silently mis-recording it, preserving the positive-only invariant.
    #[test]
    fn rejects_negative_values() {
        let mut hist: HistogramNN<16> = HistogramNN::new();
        assert!(hist.update(-1.0).is_err());
    }
}
