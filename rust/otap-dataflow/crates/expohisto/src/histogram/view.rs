// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Promoted read-only view of a histogram.

use super::width::{SlotAddr, Width};
use super::{HistogramNN, Stats};

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
    addr: Option<SlotAddr<'a>>,
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
