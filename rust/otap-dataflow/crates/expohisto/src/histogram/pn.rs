// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free exponential histogram with positive and negative ranges.
//!
//! [`HistogramPN<K, L>`] supports both positive and negative values, with
//! `K` u64 words for positive buckets and `L` u64 words for negative
//! buckets.

use core::fmt;

use crate::float64::{NAN_INF_BIASED, get_biased_exponent, get_significand};
use crate::mapping::ScaleError;

use super::view::BucketView;
use super::width::Width;
use super::{Error, HistogramNN, Settings, Stats};

/// An allocation-free exponential histogram for values of any sign.
///
/// `K` is the number of u64 words for positive-range buckets;
/// `L` is the number for negative-range buckets.
///
/// # Examples
///
/// ```
/// use otap_df_expohisto::HistogramPN;
///
/// // 8 words positive, 2 words negative — mostly-positive gauge.
/// let mut h: HistogramPN<8, 2> = HistogramPN::new();
/// h.update(100.0).unwrap();
/// h.update(-0.5).unwrap();
///
/// let v = h.view();
/// assert_eq!(v.stats().count, 2);
/// assert!(!v.positive().is_empty());
/// assert!(!v.negative().is_empty());
/// ```
pub struct HistogramPN<const K: usize, const L: usize> {
    positive: HistogramNN<K>,
    negative: HistogramNN<L>,

    /// Aggregate sum across both ranges (using actual signed values).
    sum: f64,
    /// Aggregate minimum (smallest signed value seen).
    min: f64,
    /// Aggregate maximum (largest signed value seen).
    max: f64,
    /// Count of exactly zero observations.
    zero_count: u64,
}

impl<const K: usize, const L: usize> Clone for HistogramPN<K, L> {
    fn clone(&self) -> Self {
        Self {
            positive: self.positive.clone(),
            negative: self.negative.clone(),
            sum: self.sum,
            min: self.min,
            max: self.max,
            zero_count: self.zero_count,
        }
    }
}

impl<const K: usize, const L: usize> fmt::Debug for HistogramPN<K, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.aggregate_stats();
        f.debug_struct("HistogramPN")
            .field("count", &stats.count)
            .field("sum", &stats.sum)
            .field("min", &stats.min)
            .field("max", &stats.max)
            .field("scale", &self.scale())
            .field("zero_count", &self.zero_count)
            .field("positive_slots", &self.positive.view().positive().len())
            .field("negative_slots", &self.negative.view().positive().len())
            .finish()
    }
}

impl<const K: usize, const L: usize> Default for HistogramPN<K, L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const K: usize, const L: usize> HistogramPN<K, L> {
    /// Creates a new histogram at the maximum supported scale.
    ///
    /// # Panics
    ///
    /// Panics if `K < 2`, `K > 250`, `L < 2`, or `L > 250` (same
    /// constraints as [`HistogramNN::new`]).
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            positive: HistogramNN::new(),
            negative: HistogramNN::new(),
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            zero_count: 0,
        }
    }

    /// Sets the maximum scale for both ranges.
    ///
    /// # Errors
    ///
    /// Returns [`ScaleError::InvalidScale`] if `scale` is outside
    /// [`MIN_SCALE`](crate::MIN_SCALE)..=[`table_scale()`](crate::table_scale).
    #[inline]
    pub fn with_scale(mut self, scale: i32) -> Result<Self, ScaleError> {
        self.positive = self.positive.with_scale(scale)?;
        self.negative = self.negative.with_scale(scale)?;
        Ok(self)
    }

    /// Sets the minimum bucket width for both ranges.
    #[inline]
    #[must_use]
    pub fn with_min_width(mut self, width: Width) -> Self {
        self.positive = self.positive.with_min_width(width);
        self.negative = self.negative.with_min_width(width);
        self
    }

    /// Returns the current scale (minimum of both sub-histograms).
    #[inline]
    fn scale(&self) -> i32 {
        self.positive
            .current_settings()
            .scale()
            .scale()
            .min(self.negative.current_settings().scale().scale())
    }

    /// Returns a read-only view of the histogram.
    #[inline]
    #[must_use]
    pub fn view(&self) -> HistogramPNView<'_, K, L> {
        HistogramPNView { hist: self }
    }

    /// Returns the initial settings (from construction, same for both ranges).
    #[inline]
    #[must_use]
    pub const fn initial_settings(&self) -> Settings {
        self.positive.initial_settings()
    }

    /// Returns the current settings of the positive range.
    #[inline]
    #[must_use]
    pub const fn positive_settings(&self) -> Settings {
        self.positive.current_settings()
    }

    /// Returns the current settings of the negative range.
    #[inline]
    #[must_use]
    pub const fn negative_settings(&self) -> Settings {
        self.negative.current_settings()
    }

    /// Swaps contents with another histogram.
    #[inline]
    pub fn swap(&mut self, other: &mut Self) {
        core::mem::swap(self, other);
    }

    /// Resets the histogram to its initial state.
    pub fn clear(&mut self) {
        self.positive.clear();
        self.negative.clear();
        self.sum = 0.0;
        self.min = f64::INFINITY;
        self.max = f64::NEG_INFINITY;
        self.zero_count = 0;
    }

    /// Records a single value (positive, negative, or zero).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Extreme`] if the value is NaN, +Inf, or -Inf.
    /// Returns [`Error::Overflow`] if the total count would exceed `u64::MAX`.
    #[inline]
    pub fn update(&mut self, value: f64) -> Result<(), Error> {
        self.record_incr(value, 1)
    }

    /// Records a value with a specified increment.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Extreme`] if the value is NaN, +Inf, or -Inf.
    /// Returns [`Error::Overflow`] if the total count would exceed `u64::MAX`.
    pub fn record_incr(&mut self, value: f64, incr: u64) -> Result<(), Error> {
        let biased_exp = get_biased_exponent(value);
        let significand = get_significand(value);

        // Check for total count overflow across both sub-histograms.
        let total = self.total_count();
        let _ = total.checked_add(incr).ok_or(Error::Overflow)?;

        match biased_exp {
            0 if significand == 0 => {
                // Both +0.0 and -0.0 are treated as zero.
                self.zero_count = self.zero_count.checked_add(incr).ok_or(Error::Overflow)?;
                self.min = self.min.min(0.0);
                self.max = self.max.max(0.0);
                return Ok(());
            }
            NAN_INF_BIASED => {
                return Err(Error::Extreme);
            }
            _ => {}
        }

        // After pre-validation above, the inner record_incr cannot
        // fail: NaN/Inf are rejected, zero is handled, and the total
        // overflow check covers each sub-histogram's individual count.
        // The inner call drives downscale/widen internally and always
        // succeeds for valid non-zero finite values.
        if value.is_sign_negative() {
            self.negative.record_incr(-value, incr)?;
        } else {
            self.positive.record_incr(value, incr)?;
        }

        self.sum += value * incr as f64;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sync_scales();
        Ok(())
    }

    /// Returns the total count across both ranges and zeros.
    #[inline]
    fn total_count(&self) -> u64 {
        let pos = self.positive.view().stats().count;
        let neg = self.negative.view().stats().count;
        pos.saturating_add(neg).saturating_add(self.zero_count)
    }

    /// Computes aggregate stats from both sub-histograms.
    fn aggregate_stats(&self) -> Stats {
        let total = self.total_count();
        if total == 0 {
            return Stats {
                count: 0,
                sum: 0.0,
                min: 0.0,
                max: 0.0,
            };
        }

        Stats {
            count: total,
            sum: self.sum,
            min: if self.min == f64::INFINITY {
                0.0
            } else {
                self.min
            },
            max: if self.max == f64::NEG_INFINITY {
                0.0
            } else {
                self.max
            },
        }
    }

    /// Synchronizes both sub-histograms to the same (lower) scale.
    fn sync_scales(&mut self) {
        let ps = self.positive.current_settings().scale().scale();
        let ns = self.negative.current_settings().scale().scale();

        if ps < ns && !self.negative.view().positive().is_empty() {
            let change = (ns - ps) as u32;
            self.negative.downscale_by(change);
        } else if ns < ps && !self.positive.view().positive().is_empty() {
            let change = (ps - ns) as u32;
            self.positive.downscale_by(change);
        }
    }

    /// Merges another histogram into this one.
    ///
    /// The source histogram may have different pool sizes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Overflow`] if the combined total count would
    /// exceed `u64::MAX`.
    pub fn merge_from<const K2: usize, const L2: usize>(
        &mut self,
        other: &HistogramPN<K2, L2>,
    ) -> Result<(), Error> {
        let other_total = other.total_count();
        if other_total == 0 {
            return Ok(());
        }

        // Check total count overflow.  This covers both sub-histograms:
        // since pos_count ≤ total and neg_count ≤ total, if the combined
        // totals fit in u64, each sub-histogram's individual merge also
        // fits.  merge_buckets is infallible, so the inner merge_from
        // calls cannot fail after this check passes.
        let _ = self
            .total_count()
            .checked_add(other_total)
            .ok_or(Error::Overflow)?;

        // Merge sub-histograms.
        self.positive.merge_from(&other.positive)?;
        self.negative.merge_from(&other.negative)?;

        // Merge aggregate stats.
        self.sum += other.sum;
        self.zero_count = self.zero_count.saturating_add(other.zero_count);

        if other.total_count() > 0 {
            if other.min != f64::INFINITY {
                self.min = self.min.min(other.min);
            }
            if other.max != f64::NEG_INFINITY {
                self.max = self.max.max(other.max);
            }
        }

        // Both sub-histograms may now be at different scales due to
        // independent merge operations; synchronize them.
        self.sync_scales();

        Ok(())
    }
}

// Compile-time test that HistogramPN is Send + Sync.
const fn _assert_pn_send_sync<T: Send + Sync>() {}
const _: () = _assert_pn_send_sync::<HistogramPN<2, 2>>();

/// Read-only view of a [`HistogramPN`].
///
/// Created by [`HistogramPN::view`].
#[derive(Debug)]
pub struct HistogramPNView<'a, const K: usize, const L: usize> {
    hist: &'a HistogramPN<K, L>,
}

impl<const K: usize, const L: usize> HistogramPNView<'_, K, L> {
    /// Returns the current scale (shared between positive and negative ranges).
    ///
    /// Returns 0 when no non-zero values have been recorded.
    #[inline]
    #[must_use]
    pub fn scale(&self) -> i32 {
        let pos_empty = self.hist.positive.buckets_empty();
        let neg_empty = self.hist.negative.buckets_empty();
        if pos_empty && neg_empty {
            0
        } else {
            self.hist.scale()
        }
    }

    /// Returns the aggregate statistics (count, sum, min, max).
    ///
    /// When the histogram is empty (count is 0), min and max are
    /// reported as 0.0.
    #[inline]
    #[must_use]
    pub fn stats(&self) -> Stats {
        self.hist.aggregate_stats()
    }

    /// Returns a read-only view of the positive buckets.
    #[inline]
    #[must_use]
    pub fn positive(&self) -> BucketView<'_, K> {
        BucketView {
            hist: &self.hist.positive,
        }
    }

    /// Returns a read-only view of the negative buckets.
    ///
    /// The bucket indices correspond to the absolute value of the
    /// recorded negative values. That is, `negative().offset()` and
    /// the bucket counts describe the distribution of `|value|` for
    /// all recorded negative values.
    #[inline]
    #[must_use]
    pub fn negative(&self) -> BucketView<'_, L> {
        BucketView {
            hist: &self.hist.negative,
        }
    }

    /// Returns the count of exactly-zero observations.
    #[inline]
    #[must_use]
    pub fn zero_count(&self) -> u64 {
        self.hist.zero_count
    }
}
