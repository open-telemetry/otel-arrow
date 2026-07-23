// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Quantile estimation — CDF walk over bucket view.
//!
//! This module is gated behind `#[cfg(feature = "quantile")]`.

use crate::mapping::Scale;

use super::HistogramNN;
use super::view::HistogramView;

/// A quantile–value pair estimated from a histogram's bucket distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantileValue {
    /// The requested quantile, in `[0.0, 1.0]`.
    pub quantile: f64,
    /// The estimated value at that quantile.
    pub value: f64,
}

/// Iterator that walks the histogram CDF and yields [`QuantileValue`]s.
///
/// Created by [`HistogramView::quantiles`].
///
/// The iterator walks through zero-valued observations first, then through
/// positive buckets in index order, using linear interpolation within the
/// bucket that straddles each quantile threshold.
///
/// By definition, quantile 0.0 yields the minimum and quantile 1.0 yields
/// the maximum.
#[derive(Debug)]
pub struct QuantileIter<'a, const N: usize> {
    hist: &'a HistogramNN<N>,
    scale: Scale,
    quantiles: &'a [f64],
    qi: usize,

    // Bucket walk state
    bucket_len: u32,
    offset: i32,
    pos: u32,

    // CDF accumulator
    cumulative: u64,
    total_count: u64,
    zero_count: u64,
    zeros_processed: bool,

    min: f64,
    max: f64,
}

impl<const N: usize> HistogramView<'_, N> {
    /// Returns an iterator that estimates values at the requested quantiles.
    ///
    /// Each quantile must be in `[0.0, 1.0]` and the slice must be sorted
    /// in non-decreasing order. By definition, quantile 0.0 yields the
    /// minimum and quantile 1.0 yields the maximum.
    ///
    /// The iterator walks the histogram's CDF exactly once, using linear
    /// interpolation within the bucket that straddles each threshold.
    /// Zero-valued observations contribute CDF mass at value 0.0 before
    /// any positive buckets.
    ///
    /// # Panics
    ///
    /// Debug-asserts that every quantile is in `[0.0, 1.0]` and that the
    /// slice is sorted.
    #[must_use]
    pub fn quantiles<'a>(&'a self, quantiles: &'a [f64]) -> QuantileIter<'a, N> {
        debug_assert!(
            quantiles.windows(2).all(|w| w[0] <= w[1]),
            "quantiles must be sorted in non-decreasing order"
        );
        debug_assert!(
            quantiles.iter().all(|&q| (0.0..=1.0).contains(&q)),
            "quantiles must be in [0.0, 1.0]"
        );

        let stats = self.stats();
        let total_count = stats.count;
        let min = stats.min;
        let max = stats.max;

        let bucket_len = self.hist.trimmed_slot_count();
        let offset = if self.hist.buckets_empty() {
            0
        } else {
            self.hist.first_slot()
        };

        // Derive zero_count by subtracting positive bucket sum from total.
        let positive_count: u64 = self.positive().iter().sum();
        let zero_count = total_count.saturating_sub(positive_count);

        let scale = if positive_count == 0 {
            // Cannot fail: scale 0 is always valid.
            Scale::new(0).expect("scale 0 is always valid")
        } else {
            self.hist.current.scale
        };

        QuantileIter {
            hist: self.hist,
            scale,
            quantiles,
            qi: 0,
            bucket_len,
            offset,
            pos: 0,
            cumulative: 0,
            total_count,
            zero_count,
            zeros_processed: false,
            min,
            max,
        }
    }
}

impl<const N: usize> Iterator for QuantileIter<'_, N> {
    type Item = QuantileValue;

    fn next(&mut self) -> Option<QuantileValue> {
        let &q = self.quantiles.get(self.qi)?;
        self.qi += 1;

        // Empty histogram — no meaningful estimate.
        if self.total_count == 0 {
            return Some(QuantileValue {
                quantile: q,
                value: f64::NAN,
            });
        }

        // Boundary quantiles use exact stats.
        if q <= 0.0 {
            let value = if self.zero_count > 0 { 0.0 } else { self.min };
            return Some(QuantileValue { quantile: q, value });
        }
        if q >= 1.0 {
            return Some(QuantileValue {
                quantile: q,
                value: self.max,
            });
        }

        let target = q * self.total_count as f64;

        // Account for zero-valued observations (CDF mass at value 0.0).
        if !self.zeros_processed {
            self.cumulative = self.zero_count;
            self.zeros_processed = true;
        }
        if self.cumulative as f64 >= target {
            return Some(QuantileValue {
                quantile: q,
                value: 0.0,
            });
        }

        // Walk positive buckets until cumulative count reaches the target.
        while self.pos < self.bucket_len {
            let index = self.offset + self.pos as i32;
            let addr = self.hist.slot_addr(index);
            let count = self.hist.bucket_get(&addr);

            if count == 0 {
                self.pos += 1;
                continue;
            }

            let new_cumulative = self.cumulative + count;

            if new_cumulative as f64 >= target {
                // lower_boundary(index) cannot fail: bucket indices in
                // the histogram always correspond to normal f64 values
                // (subnormals are clamped to MIN_VALUE before mapping).
                //
                // lower_boundary(index + 1) can return Overflow when
                // the uppermost bucket spans the boundary of
                // representable f64.  In that case self.max is the
                // correct upper bound for interpolation.
                let lower = self.scale.lower_boundary(index).unwrap_or(0.0);
                let upper = self.scale.lower_boundary(index + 1).unwrap_or(self.max);
                let fraction = (target - self.cumulative as f64) / count as f64;
                let value = (lower + fraction * (upper - lower)).clamp(self.min, self.max);

                // Don't advance pos/cumulative — next quantile may land
                // in the same bucket.
                return Some(QuantileValue { quantile: q, value });
            }

            self.cumulative = new_cumulative;
            self.pos += 1;
        }

        // All buckets exhausted — return max.
        Some(QuantileValue {
            quantile: q,
            value: self.max,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.quantiles.len() - self.qi;
        (remaining, Some(remaining))
    }
}

impl<const N: usize> ExactSizeIterator for QuantileIter<'_, N> {}
