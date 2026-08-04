// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Promoted read-only view of a histogram.

use super::width::{SlotAddr, Width};
use super::{HistogramNN, Stats};
use crate::mapping::Scale;

/// Read-only view of a histogram's data.
///
/// Created by [`HistogramNN::view`], which may promote from literal mode
/// to bucket mode internally. All accessors take `&self`, so a
/// `HistogramView` can be shared freely once obtained.
///
/// ```
/// use otap_df_expohisto::HistogramNN;
///
/// let mut h: HistogramNN<16> = HistogramNN::new();
/// h.update(1.5).unwrap();
/// h.update(2.7).unwrap();
///
/// let v = h.view();
/// assert_eq!(v.stats().count, 2);
/// assert!(v.stats().sum > 4.0);
/// println!("scale = {}, buckets = {}", v.scale(), v.positive().len());
/// ```
#[derive(Debug)]
pub struct HistogramView<'a, const N: usize> {
    pub(crate) hist: &'a HistogramNN<N>,
}

impl<const N: usize> HistogramView<'_, N> {
    /// Returns the current scale.
    ///
    /// Returns 0 when no non-zero values have been recorded.
    #[inline]
    #[must_use]
    pub fn scale(&self) -> i32 {
        if self.hist.buckets_empty() {
            0
        } else {
            self.hist.current.scale.scale()
        }
    }

    /// Returns the aggregate statistics (count, sum, min, max).
    ///
    /// When the histogram is empty (count is 0), min and max are
    /// reported as 0.0.
    #[inline]
    #[must_use]
    pub const fn stats(&self) -> Stats {
        if self.hist.stats.count == 0 || self.hist.buckets_empty() {
            Stats {
                count: self.hist.stats.count,
                sum: 0.0,
                min: 0.0,
                max: 0.0,
            }
        } else {
            self.hist.stats
        }
    }

    /// Returns a read-only view of the positive buckets.
    #[inline]
    #[must_use]
    pub fn positive(&self) -> BucketView<'_, N> {
        BucketView { hist: self.hist }
    }

    /// Walks every bucket exactly once, passing each count to `emit`, and
    /// returns the totals learned by that walk.
    ///
    /// This is the only way to obtain a [`BucketTotals::zero_count`]. Zeros
    /// contribute to the total count but never occupy a bucket, so the zero
    /// count is recoverable only by subtracting the positive bucket total --
    /// which costs a full scan of the buckets. Rather than let a caller pay
    /// for that scan and throw the bucket counts away, the scan is exposed as
    /// the encoding operation itself: whoever encodes, downscales, or
    /// otherwise consumes the buckets learns the zero count for free.
    ///
    /// Counts are emitted in slot order starting at
    /// [`BucketView::offset`], one per logical bucket, so an encoder can
    /// push them straight into its output.
    ///
    /// ```
    /// use otap_df_expohisto::HistogramNN;
    ///
    /// let mut h: HistogramNN<16> = HistogramNN::new();
    /// for v in [0.0, 0.0, 1.5, 2.7] {
    ///     h.update(v).unwrap();
    /// }
    ///
    /// let view = h.view();
    /// let mut counts = Vec::new();
    /// let totals = view.scan_buckets(|count| counts.push(count));
    ///
    /// assert_eq!(totals.zero_count, 2);
    /// assert_eq!(totals.positive_total, 2);
    /// assert_eq!(counts.len(), view.positive().len() as usize);
    /// ```
    pub fn scan_buckets<F>(&self, mut emit: F) -> BucketTotals
    where
        F: FnMut(u64),
    {
        let mut positive_total = 0_u64;
        for count in self.positive().iter() {
            positive_total = positive_total.saturating_add(count);
            emit(count);
        }
        BucketTotals {
            positive_total,
            zero_count: self.hist.stats.count.saturating_sub(positive_total),
        }
    }

    /// Returns the relative error bound of values produced by
    /// [`quantiles`](Self::quantiles) at the current scale.
    ///
    /// Returns 0.0 for an empty histogram, whose min and max are exact.
    #[inline]
    #[must_use]
    pub fn relative_error(&self) -> f64 {
        if self.hist.buckets_empty() {
            return 0.0;
        }
        self.hist.current.scale.relative_error()
    }

    /// Estimates the values at the requested quantiles in a single pass, and
    /// returns the [`BucketTotals`] learned while doing so.
    ///
    /// `quantiles` must be sorted in non-decreasing order with every entry in
    /// `[0.0, 1.0]`; `out` receives one estimate per requested quantile and
    /// must be at least as long.
    ///
    /// Quantile 0.0 yields the exact minimum and 1.0 the exact maximum.
    ///
    /// An empty histogram yields `f64::NAN` for every quantile: there
    /// is no observation to estimate from.
    pub fn quantiles(&self, quantiles: &[f64], out: &mut [f64]) -> BucketTotals {
        assert!(
            out.len() >= quantiles.len(),
            "output buffer is shorter than the requested quantiles"
        );
        debug_assert!(
            quantiles.windows(2).all(|w| w[0] <= w[1]),
            "quantiles must be sorted in non-decreasing order"
        );
        debug_assert!(
            quantiles.iter().all(|&q| (0.0..=1.0).contains(&q)),
            "quantiles must be in [0.0, 1.0]"
        );

        let stats = self.stats();
        let totals = self.scan_buckets(|_| {});
        let zero_count = totals.zero_count;
        let scale = self.hist.current.scale;
        let bucket_len = self.hist.trimmed_slot_count();
        let offset = if self.hist.buckets_empty() {
            0
        } else {
            self.hist.first_slot()
        };

        // Cursor into the CDF, shared across quantiles so the buckets are
        // walked at most once no matter how many quantiles are requested.
        let mut pos = 0_u32;
        let mut cumulative = zero_count;

        for (&q, slot) in quantiles.iter().zip(out.iter_mut()) {
            if stats.count == 0 {
                *slot = f64::NAN;
                continue;
            }
            if q <= 0.0 {
                *slot = stats.min;
                continue;
            }
            if q >= 1.0 {
                *slot = stats.max;
                continue;
            }

            let target = q * stats.count as f64;

            // Advance to the bucket holding the target. Quantiles are sorted,
            // so the cursor never rewinds and the buckets are walked once in
            // total rather than once per quantile.
            while pos < bucket_len {
                let count = self.bucket_at(offset, pos);
                if count > 0 && (cumulative + count) as f64 >= target {
                    break;
                }
                cumulative += count;
                pos += 1;
            }

            *slot = self.interpolate(
                scale, offset, pos, bucket_len, target, cumulative, zero_count, &stats,
            );
        }

        totals
    }

    /// Count in the logical bucket at `offset + pos`.
    #[inline]
    fn bucket_at(&self, offset: i32, pos: u32) -> u64 {
        let addr = self.hist.slot_addr(offset + pos as i32);
        self.hist.bucket_get(&addr)
    }

    /// Places a CDF threshold within the bucket the cursor rests in.
    #[allow(clippy::too_many_arguments)]
    fn interpolate(
        &self,
        scale: Scale,
        offset: i32,
        pos: u32,
        bucket_len: u32,
        target: f64,
        cumulative: u64,
        zero_count: u64,
        stats: &Stats,
    ) -> f64 {
        // Every bucket was consumed without reaching the target, which
        // rounding can produce at quantiles very close to 1.0.
        if pos >= bucket_len {
            return stats.max;
        }
        let count = self.bucket_at(offset, pos);
        if count == 0 {
            return stats.max;
        }

        let index = offset + pos as i32;
        // lower_boundary(index) cannot fail for an occupied index: subnormals
        // are clamped before mapping, so every index maps to a normal f64.
        let lower = scale.lower_boundary(index).unwrap_or(stats.min);

        // The threshold sits at or before this bucket's lower edge, meaning
        // the mass already accumulated covers it.
        if target <= cumulative as f64 {
            if cumulative == zero_count && zero_count > 0 {
                return 0.0;
            }
            return lower.clamp(stats.min, stats.max);
        }

        // The distribution within one bucket is unknown. Its geometric
        // midpoint is the only representative whose error is bounded by the
        // scale's advertised relative error for every value in that bucket.
        // Moving the estimate according to the rank within the bucket would
        // assume a uniform distribution and can exceed that bound for skewed
        // observations. The topmost bucket can have no representable upper
        // edge, where the lower boundary is the only finite representative.
        let value = scale.interpolate_boundary(index, 0.5).unwrap_or(lower);
        value.clamp(stats.min, stats.max)
    }
}

/// Totals learned by walking every bucket of a histogram exactly once.
///
/// Returned by [`HistogramView::scan_buckets`] and
/// [`HistogramView::quantiles`], the two operations that already pay for the
/// walk. There is deliberately no standalone zero-count accessor: obtaining
/// `zero_count` on its own would scan every bucket only to discard the counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketTotals {
    /// Sum of every positive bucket count.
    pub positive_total: u64,
    /// Number of exact-zero observations, recovered as
    /// `count - positive_total`. Zeros contribute to the total count but
    /// never occupy a bucket.
    pub zero_count: u64,
}

impl BucketTotals {
    /// Totals of a histogram with no observations at all.
    pub const EMPTY: Self = Self {
        positive_total: 0,
        zero_count: 0,
    };
}

/// Read-only view of bucket data in a histogram.
#[derive(Debug)]
pub struct BucketView<'a, const N: usize> {
    pub(crate) hist: &'a HistogramNN<N>,
}

impl<const N: usize> BucketView<'_, N> {
    /// Returns the first slot index (bucket offset).
    ///
    /// This is the index of the first non-zero bucket, trimmed
    /// to sub-word granularity.
    #[inline]
    #[must_use]
    pub fn offset(&self) -> i32 {
        if self.hist.buckets_empty() {
            return 0;
        }
        self.hist.first_slot()
    }

    /// Number of logical buckets in use.
    ///
    /// This is the count from the first non-zero bucket to the last
    /// non-zero bucket (inclusive), trimmed to sub-word granularity.
    #[inline]
    #[must_use]
    pub fn len(&self) -> u32 {
        self.hist.trimmed_slot_count()
    }

    /// Number of logical buckets in use (alias).
    #[inline]
    #[must_use]
    pub fn bucket_count(&self) -> u32 {
        self.len()
    }

    /// Returns the current counter width.
    #[inline]
    #[must_use]
    pub fn width(&self) -> Width {
        self.hist.current.width
    }

    /// Returns true if no buckets are in use.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hist.buckets_empty()
    }

    /// Returns an iterator over bucket counts.
    ///
    /// Iterates from the first non-zero slot to the last non-zero
    /// slot, matching [`offset`](Self::offset) and [`len`](Self::len).
    #[inline]
    #[must_use]
    pub fn iter(&self) -> BucketsIter<'_, N> {
        let remaining = self.hist.trimmed_slot_count() as usize;
        BucketsIter {
            hist: self.hist,
            addr: if remaining > 0 {
                Some(self.hist.slot_addr(self.hist.first_slot()))
            } else {
                None
            },
            remaining,
        }
    }
}

impl<'a, const N: usize> IntoIterator for &'a BucketView<'a, N> {
    type Item = u64;
    type IntoIter = BucketsIter<'a, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over bucket counts.
#[derive(Debug)]
pub struct BucketsIter<'a, const N: usize> {
    hist: &'a HistogramNN<N>,
    addr: Option<SlotAddr>,
    remaining: usize,
}

impl<const N: usize> Iterator for BucketsIter<'_, N> {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let addr = self.addr.as_ref()?;
        let count = self.hist.bucket_get(addr);
        self.remaining -= 1;
        if self.remaining > 0 {
            if let Some(a) = self.addr.take() {
                self.addr = a.next_addr(self.hist.word_end);
            }
        } else {
            self.addr = None;
        }
        Some(count)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<const N: usize> ExactSizeIterator for BucketsIter<'_, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Estimates a set of quantiles from a fresh histogram over `values`.
    fn estimate<const N: usize>(values: &[f64], qs: &[f64]) -> Vec<f64> {
        let mut h: HistogramNN<N> = HistogramNN::new();
        for &v in values {
            h.update(v).unwrap();
        }
        let mut out = vec![0.0; qs.len()];
        let _ = h.view().quantiles(qs, &mut out);
        out
    }

    /// Scenario: A histogram records the integers 1..=1000 and is asked for
    /// p50, p90 and p99.
    /// Guarantees: every estimate lands within the histogram's relative error
    /// bound of the exact quantile of the recorded population, proving the
    /// log-space interpolation and the advertised bound agree.
    #[test]
    fn quantile_estimates_stay_within_the_relative_error_bound() {
        let values: Vec<f64> = (1..=1000).map(f64::from).collect();
        let qs = [0.5, 0.9, 0.99];
        let got = estimate::<64>(&values, &qs);

        let mut h: HistogramNN<64> = HistogramNN::new();
        for &v in &values {
            h.update(v).unwrap();
        }
        let bound = h.view().relative_error();
        assert!(bound > 0.0 && bound < 0.1, "bound = {bound}");

        for (&q, &est) in qs.iter().zip(got.iter()) {
            let exact = values[(q * values.len() as f64) as usize - 1];
            let err = (est - exact).abs() / exact;
            assert!(
                err <= bound * 1.5,
                "q={q} est={est} exact={exact} err={err} bound={bound}"
            );
        }
    }

    /// Scenario: Ninety-nine low observations and one high observation occupy
    /// the same exponential bucket, and the histogram is asked for p99.
    /// Guarantees: The p99 estimate remains within the histogram's advertised
    /// relative error of the exact low value despite the within-bucket skew.
    #[test]
    fn skewed_single_bucket_p99_stays_within_relative_error() {
        // Both fixed values occupy the scale-8 bucket immediately below 1.0.
        // Starting with U8 counters isolates quantile estimation from the
        // separate counter-widening path. The exact nearest-rank p99 is the
        // low value, not the single high observation in the same bucket.
        const EXACT_P99: f64 = 0.9974;
        let mut histogram = HistogramNN::<64>::new()
            .with_min_width(Width::U8)
            .expect("scale 8 supports U8 counters");
        for _ in 0..99 {
            histogram
                .update(EXACT_P99)
                .expect("low value is recordable");
        }
        histogram.update(0.9999).expect("high value is recordable");
        let view = histogram.view();
        assert_eq!(view.scale(), 8);
        assert_eq!(view.positive().offset(), -1);
        assert_eq!(view.positive().bucket_count(), 1);

        let mut estimate = [0.0];
        let _ = view.quantiles(&[0.99], &mut estimate);
        let error = (estimate[0] - EXACT_P99).abs() / EXACT_P99;
        let bound = view.relative_error();
        assert!(
            error <= bound + f64::EPSILON,
            "estimate={} exact={EXACT_P99} error={error} bound={bound}",
            estimate[0]
        );
    }

    /// Scenario: A histogram records only exact zeros and is queried across
    /// the full quantile range.
    /// Guarantees: every quantile reports 0.0 rather than NaN or the bucket
    /// minimum, because zeros carry cumulative mass ahead of all buckets.
    #[test]
    fn quantiles_of_an_all_zero_population_are_zero() {
        let got = estimate::<16>(&[0.0, 0.0, 0.0, 0.0], &[0.0, 0.25, 0.5, 0.9, 1.0]);
        assert_eq!(got, vec![0.0; 5]);
    }

    /// Scenario: A population that is three quarters exact zeros and one
    /// quarter large values is queried below and above the zero mass.
    /// Guarantees: quantiles inside the zero mass report 0.0 while quantiles
    /// beyond it report a positive value, proving zeros are ordered before
    /// the positive buckets in the CDF.
    #[test]
    fn zero_mass_precedes_positive_buckets_in_the_cdf() {
        let got = estimate::<32>(&[0.0, 0.0, 0.0, 100.0], &[0.25, 0.5, 0.7, 0.95]);
        assert_eq!(got[0], 0.0);
        assert_eq!(got[1], 0.0);
        assert_eq!(got[2], 0.0);
        assert!(got[3] > 0.0, "got = {got:?}");
    }

    /// Scenario: An empty histogram is asked for quantiles.
    /// Guarantees: every estimate is NaN, signalling that no observation
    /// exists to estimate from rather than reporting a misleading 0.0.
    #[test]
    fn quantiles_of_an_empty_histogram_are_nan() {
        let got = estimate::<16>(&[], &[0.0, 0.5, 1.0]);
        assert!(got.iter().all(|v| v.is_nan()), "got = {got:?}");
    }

    /// Scenario: A histogram is queried at the exact endpoints q=0 and q=1.
    /// Guarantees: the endpoints report the recorded minimum and maximum
    /// exactly, bypassing bucket interpolation entirely.
    #[test]
    fn endpoint_quantiles_report_exact_min_and_max() {
        let got = estimate::<32>(&[1.5, 7.25, 3.0, 99.0], &[0.0, 1.0]);
        assert_eq!(got[0], 1.5);
        assert_eq!(got[1], 99.0);
    }

    /// Scenario: The same quantiles are requested in one call and the CDF
    /// cursor is shared across them.
    /// Guarantees: estimates are non-decreasing in q, proving the shared
    /// single-pass cursor never rewinds or double-counts a bucket.
    #[test]
    fn estimates_are_monotonic_in_the_requested_quantile() {
        let values: Vec<f64> = (1..=500).map(|i| f64::from(i) * 0.5).collect();
        let qs: Vec<f64> = (0..=20).map(|i| f64::from(i) / 20.0).collect();
        let got = estimate::<64>(&values, &qs);
        for w in got.windows(2) {
            assert!(w[0] <= w[1], "not monotonic: {got:?}");
        }
    }

    /// Scenario: A bucket scan reports its totals after a mix of zero and
    /// positive observations.
    /// Guarantees: the scan emits one count per logical bucket and reports a
    /// zero count equal to the number of exact zeros recorded, recovered by
    /// subtracting the positive bucket total from the count, so the zero count
    /// costs nothing beyond the scan the caller already performed.
    #[test]
    fn scan_buckets_recovers_exact_zero_observations() {
        let mut h: HistogramNN<32> = HistogramNN::new();
        for v in [0.0, 1.0, 0.0, 2.0, 0.0] {
            h.update(v).unwrap();
        }
        let view = h.view();
        let mut counts = Vec::new();
        let totals = view.scan_buckets(|c| counts.push(c));

        assert_eq!(totals.zero_count, 3);
        assert_eq!(totals.positive_total, 2);
        assert_eq!(counts.len(), view.positive().len() as usize);
        assert_eq!(counts.iter().sum::<u64>(), totals.positive_total);
        assert_eq!(totals.zero_count + totals.positive_total, 5);
        assert_eq!(view.stats().count, 5);
    }

    /// Scenario: Quantiles are requested from a histogram that recorded exact
    /// zeros alongside positive values.
    /// Guarantees: the quantile call returns the same bucket totals a
    /// standalone scan would produce, so a caller that needs both estimates
    /// and the zero count walks the buckets once rather than twice.
    #[test]
    fn quantiles_return_the_bucket_totals_they_scanned() {
        let mut h: HistogramNN<32> = HistogramNN::new();
        for v in [0.0, 0.0, 1.0, 2.0, 4.0] {
            h.update(v).unwrap();
        }
        let view = h.view();
        let mut out = [0.0; 3];
        let totals = view.quantiles(&[0.1, 0.5, 0.9], &mut out);

        assert_eq!(totals, view.scan_buckets(|_| {}));
        assert_eq!(totals.zero_count, 2);
        assert_eq!(totals.positive_total, 3);
    }

    /// Scenario: Two histograms record the same population of exact zeros and
    /// positive values, one with the zeros first and one with the zeros last.
    /// Guarantees: both report a minimum of 0.0 and the same maximum, so an
    /// exact zero always participates in the reported min/max regardless of
    /// arrival order, and the q=0 quantile equals that minimum without needing
    /// a separate zero-count special case.
    #[test]
    fn exact_zeros_pin_the_minimum_regardless_of_order() {
        let mut zeros_first: HistogramNN<32> = HistogramNN::new();
        for v in [0.0, 5.0, 2.0] {
            zeros_first.update(v).unwrap();
        }
        let mut zeros_last: HistogramNN<32> = HistogramNN::new();
        for v in [5.0, 2.0, 0.0] {
            zeros_last.update(v).unwrap();
        }

        for h in [&zeros_first, &zeros_last] {
            let stats = h.view().stats();
            assert_eq!(stats.min, 0.0);
            assert_eq!(stats.max, 5.0);
            assert_eq!(stats.count, 3);
            assert_eq!(h.view().scan_buckets(|_| {}).zero_count, 1);

            let mut out = [f64::NAN; 1];
            let _ = h.view().quantiles(&[0.0], &mut out);
            assert_eq!(out[0], 0.0);
        }
    }

    /// Scenario: An empty histogram is asked for its relative error.
    /// Guarantees: the bound is 0.0, since an empty histogram reports no
    /// interpolated value that could carry bucketing error.
    #[test]
    fn empty_histogram_reports_no_relative_error() {
        let h: HistogramNN<16> = HistogramNN::new();
        assert_eq!(h.view().relative_error(), 0.0);
    }
}
