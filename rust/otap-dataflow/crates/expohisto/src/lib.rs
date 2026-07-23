// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free OpenTelemetry exponential histogram.
//!
//! Exponential histograms provide a compact, high-resolution representation of
//! value distributions using logarithmically-spaced bucket boundaries. This
//! implementation stores bucket counters in a fixed-size data pool sized with a
//! const generic (`Histogram<N>`), performs no heap allocation, and contains no
//! `unsafe` code.
//!
//! - [`HistogramNN<N>`] (aliased as [`Histogram<N>`]) is positive-only:
//!   negative values are rejected. This suits the common case of non-negative
//!   measurements such as latencies, sizes, and counts.
//! - [`HistogramPN<K, L>`] maintains independent positive and negative bucket
//!   ranges with synchronized scales for values of any sign.
//!
//! Bucket index mapping is accelerated by a compile-time lookup table checked in
//! under `src/lookup_tables.rs` (and `src/inverse_factors.rs` for boundary
//! computation). These tables are generated data and require no build step.

#![cfg_attr(not(feature = "std"), no_std)]

pub(crate) mod exponent;
pub(crate) mod float64;
pub mod histogram;
pub mod mapping;

#[cfg(feature = "boundary")]
mod boundary;

#[doc(hidden)]
pub mod lookup;

pub use histogram::{
    BucketView, BucketsIter, Error, Histogram, HistogramNN, HistogramPN, HistogramPNView,
    HistogramView, Settings, Stats, Width,
};
#[cfg(feature = "quantile")]
pub use histogram::{QuantileIter, QuantileValue};
pub use mapping::{MAX_SCALE, MIN_SCALE, Scale, ScaleError, table_scale};

#[cfg(test)]
mod tests {
    use super::Histogram;

    /// Scenario: A fresh positive-only histogram records several finite
    /// observations and exposes aggregate statistics through its view.
    /// Guarantees: `update` accepts positive values, and the resulting view
    /// reports the exact observation count and sum, confirming the vendored
    /// lookup tables and bucket accounting are wired correctly.
    #[test]
    fn records_observations_and_reports_stats() {
        let mut hist: Histogram<16> = Histogram::new();
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
        let mut hist: Histogram<16> = Histogram::new();
        assert!(hist.update(-1.0).is_err());
    }
}
