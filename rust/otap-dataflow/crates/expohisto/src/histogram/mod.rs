// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free exponential histogram with a unified flat memory layout.
//!
//! `HistogramNN<N>` stores everything in fixed struct fields plus a `[u64; N]`
//! data pool used for bucket counters.

use core::fmt;

use crate::float64::{NAN_INF_BIASED, get_biased_exponent, get_significand, unbias_exponent};
use crate::mapping::{Scale, ScaleError, table_scale};

mod downscale;
mod merge;
mod swar;
pub mod width;

#[cfg(feature = "quantile")]
mod quantile;
#[cfg(feature = "quantile")]
pub use quantile::{QuantileIter, QuantileValue};

mod view;
pub use view::{BucketView, BucketsIter, HistogramView};

pub use width::{SlotAddr, Width};

/// Compact histogram configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Settings {
    scale: Scale,
    width: Width,
}

impl Settings {
    /// Creates settings from a scale and width.
    #[inline]
    #[must_use]
    pub const fn new(scale: Scale, width: Width) -> Self {
        Self { scale, width }
    }

    /// Returns the scale.
    #[inline]
    #[must_use]
    pub const fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns the width.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> Width {
        self.width
    }
}

/// Error returned when the total count would exceed `u64::MAX`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Overflow of a u64 counter.
    Overflow,
    /// Invalid value: NaN or ±Inf for all histogram types, or negative
    /// for [`HistogramNN`].
    Extreme,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Overflow => "histogram total count overflow",
            Self::Extreme => "invalid extreme value",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Aggregate statistics of a histogram: count, sum, min, max.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    /// Total number of observations.
    pub count: u64,
    /// Sum of all observed values.
    pub sum: f64,
    /// Minimum observed value.
    pub min: f64,
    /// Maximum observed value.
    pub max: f64,
}

impl Stats {
    /// Empty stats.
    pub const EMPTY: Self = Self {
        count: 0,
        sum: 0.0,
        min: f64::INFINITY,
        max: f64::NEG_INFINITY,
    };
}

/// High-low range helper.
#[derive(Debug, Clone, Copy)]
struct HighLow {
    low: i32,
    high: i32,
}

impl HighLow {
    /// An empty range sentinel.
    #[inline]
    const fn empty() -> Self {
        Self {
            low: i32::MAX,
            high: i32::MIN,
        }
    }

    /// Union of two ranges.
    #[inline]
    const fn merge(self, other: Self) -> Self {
        Self {
            low: if self.low < other.low {
                self.low
            } else {
                other.low
            },
            high: if self.high > other.high {
                self.high
            } else {
                other.high
            },
        }
    }

    /// Computes how much downscaling is needed.
    #[inline]
    const fn change_steps(mut self, size: usize) -> u32 {
        let mut change = 0;
        while (self.high - self.low) as usize >= size {
            self.high >>= 1;
            self.low >>= 1;
            change += 1;
        }
        change
    }
}

/// Result of attempting to increment a bucket.
enum IncrResult {
    Ok,
    NeedsDownscale(HighLow),
    CounterOverflow(u64),
}

/// An allocation-free exponential histogram for non-negative values.
///
/// Use [`HistogramPN`] when values can be negative.
pub struct HistogramNN<const N: usize> {
    initial: Settings,
    current: Settings,

    word_base: i32,
    word_start: i32,
    word_end: i32,

    stats: Stats,

    data: [u64; N],
}

impl<const N: usize> Clone for HistogramNN<N> {
    fn clone(&self) -> Self {
        Self {
            initial: self.initial,
            current: self.current,
            word_base: self.word_base,
            word_start: self.word_start,
            word_end: self.word_end,
            stats: self.stats,
            data: self.data,
        }
    }
}

impl<const N: usize> fmt::Debug for HistogramNN<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("HistogramNN");
        let stats = self.stats();
        s.field("width", &self.current.width)
            .field("count", &stats.count)
            .field("sum", &stats.sum)
            .field("min", &stats.min)
            .field("max", &stats.max)
            .field("scale", &self.current.scale.scale())
            .field("slot_count", &self.current_slot_count())
            .finish()
    }
}

impl<const N: usize> Default for HistogramNN<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> HistogramNN<N> {
    /// Returns the aggregate statistics (count, sum, min, max).
    #[inline]
    pub(crate) const fn stats(&self) -> Stats {
        self.stats
    }

    /// Checked increment of count by `incr`. Returns `None` on overflow.
    #[inline]
    const fn checked_add_count(&self, incr: u64) -> Option<u64> {
        self.stats.count.checked_add(incr)
    }

    /// Commits merged statistics. The incoming `stats` carry the
    /// already-computed `sum` and `count` (self + other) and the
    /// other side's `min`/`max` which are merged via `f64::min`/`max`.
    fn commit_stats(&mut self, stats: &Stats) {
        self.stats.sum = stats.sum;
        self.stats.min = self.stats.min.min(stats.min);
        self.stats.max = self.stats.max.max(stats.max);
        self.stats.count = stats.count;
    }

    /// Returns true if no non-zero values have been recorded.
    #[inline]
    pub(crate) const fn buckets_empty(&self) -> bool {
        if self.word_end != self.word_start {
            return false;
        }
        self.data[0] == 0
    }

    /// Number of u64 data words in use at the current width.
    #[inline]
    const fn current_word_count(&self) -> i32 {
        if self.buckets_empty() {
            0
        } else {
            self.word_end - self.word_start + 1
        }
    }

    /// Number of buckets defined at the current width.
    #[inline]
    const fn current_slot_count(&self) -> i32 {
        self.current
            .width
            .word_to_slot_index(self.current_word_count())
    }

    /// Number of leading zero lanes in the first used word.
    #[inline]
    fn leading_zero_lanes(&self) -> u32 {
        let word = self.data[self.data_idx(self.word_start)];
        word.trailing_zeros() / self.current.width.bits_per_slot()
    }

    /// Number of trailing zero lanes in the last used word.
    #[inline]
    fn trailing_zero_lanes(&self) -> u32 {
        let word = self.data[self.data_idx(self.word_end)];
        word.leading_zeros() / self.current.width.bits_per_slot()
    }

    /// Slot index of the first non-zero bucket.
    #[inline]
    pub(crate) fn first_slot(&self) -> i32 {
        self.current.width.word_to_slot_index(self.word_start) + self.leading_zero_lanes() as i32
    }

    /// Slot index of the last non-zero bucket.
    #[inline]
    pub(crate) fn last_slot(&self) -> i32 {
        self.current.width.word_to_slot_index(self.word_end + 1)
            - 1
            - self.trailing_zero_lanes() as i32
    }

    /// Number of slots from first to last non-zero bucket (inclusive).
    #[inline]
    pub(crate) fn trimmed_slot_count(&self) -> u32 {
        if self.buckets_empty() {
            return 0;
        }
        (self.last_slot() - self.first_slot() + 1) as u32
    }

    /// Returns the slot index range `[first_slot, last_slot]`.
    #[inline]
    fn slot_range(&self) -> HighLow {
        if self.buckets_empty() {
            return HighLow::empty();
        }
        HighLow {
            low: self.first_slot(),
            high: self.last_slot(),
        }
    }

    /// Projects the slot range to a (coarser) target scale.
    #[inline]
    fn slot_range_at_scale(&self, target_scale: i32) -> HighLow {
        let shift = self.current.scale.scale() - target_scale;
        let r = self.slot_range();
        HighLow {
            low: r.low >> shift,
            high: r.high >> shift,
        }
    }

    /// Returns the slot address of a bucket indx.
    #[inline]
    const fn slot_addr(&self, slot: i32) -> SlotAddr<'_> {
        self.current.width.slot_addr(slot)
    }

    /// Shifts all three index fields right by `by` positions.
    #[inline]
    fn shift_indices(&mut self, by: u32) {
        self.word_start >>= by;
        self.word_end >>= by;
        self.word_base >>= by;
    }

    /// Physical data index for a word index under the current mapping.
    #[inline]
    const fn data_idx(&self, widx: i32) -> usize {
        (widx - self.word_base).rem_euclid(N as i32) as usize
    }

    /// Gets the value at a slot address.
    #[inline]
    pub(super) const fn bucket_get(&self, addr: &SlotAddr<'_>) -> u64 {
        let idx = addr.data_index(N, self.word_base);
        let word = self.data[idx];
        addr.retrieve_counter(word)
    }

    /// Attempts to add `incr` to a physical slot. Returns false on overflow.
    #[inline]
    fn bucket_try_increment(&mut self, addr: &SlotAddr<'_>, incr: u64) -> Result<(), u64> {
        let idx = addr.data_index(N, self.word_base);
        let word = self.data[idx];
        let count = addr.retrieve_counter(word);

        let new_count = match count.checked_add(incr) {
            None => {
                // Safety: the total count would overflow before the
                // try_increment of an individual bucket would.
                unreachable!()
            }
            Some(c) => {
                if c > self.current.width.counter_max() {
                    return Err(c);
                }
                c
            }
        };

        self.data[idx] = addr.update_counter_in_word(word, new_count);
        Ok(())
    }

    /// Creates a new histogram at the maximum supported scale.
    ///
    /// # Panics
    ///
    /// Panics if `N < 2` or `N > 250`. These are compile-time constant
    /// constraints: `N >= 2` ensures the minimum scale covers the full
    /// exponent range; `N <= 250` caps the struct at 2 KiB.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        // The limit at 2 ensures MIN_SCALE is sufficient to cover the
        // entire range.
        assert!(N >= 2, "requires >= 2 u64 buckets");

        // The limit at 250 allows up to 16k single-bit buckets and
        // limits the histogram struct to 2048 bytes, noting that the
        // structure itself uses 6 u64.
        //
        // Note that nothing breaks when we allow N to grow above this
        // limit, just performance. The algorithms here are designed
        // for cache-line sized data.
        assert!(N <= 250, "requires <= 250 u64 buckets");

        let settings = Settings::new(
            Scale::new(table_scale()).expect("table scale is valid"),
            Width::B1,
        );
        Self {
            initial: settings,
            current: settings,
            word_base: 0,
            word_start: 0,
            word_end: 0,
            stats: Stats::EMPTY,
            data: [0u64; N],
        }
    }

    /// Sets the maximum scale.
    ///
    /// # Errors
    ///
    /// Returns [`ScaleError::InvalidScale`] if `scale` is outside
    /// [`MIN_SCALE`](crate::MIN_SCALE)..=[`table_scale()`](crate::table_scale).
    #[inline]
    pub fn with_scale(mut self, scale: i32) -> Result<Self, ScaleError> {
        let s = Scale::new(scale)?;
        self.initial.scale = s;
        self.current.scale = s;
        Ok(self)
    }

    /// Sets the minimum bucket width.
    #[inline]
    #[must_use]
    pub fn with_min_width(mut self, width: Width) -> Self {
        self.initial.width = width;
        self.current.width = width;
        self
    }

    /// Returns a read-only view of the histogram.
    ///
    #[inline]
    #[must_use]
    pub fn view(&self) -> HistogramView<'_, N> {
        HistogramView { hist: self }
    }

    /// Returns the initial settings (as configured at construction).
    #[inline]
    #[must_use]
    pub const fn initial_settings(&self) -> Settings {
        self.initial
    }

    /// Returns the current settings (scale may have decreased and
    /// width may have increased due to insertions or merges).
    #[inline]
    #[must_use]
    pub const fn current_settings(&self) -> Settings {
        self.current
    }

    /// Returns the current counter width.
    #[inline]
    #[must_use]
    pub const fn width(&self) -> Width {
        self.current.width
    }

    /// Swaps contents with another histogram.
    #[inline]
    pub fn swap(&mut self, other: &mut Self) {
        core::mem::swap(self, other);
    }

    /// Resets the histogram to its initial state.
    pub fn clear(&mut self) {
        self.current = self.initial;
        self.word_base = 0;
        self.word_start = 0;
        self.word_end = 0;
        self.stats = Stats::EMPTY;
        self.data.fill(0);
    }

    /// Records a single value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Extreme`] if the value is NaN, ±Inf, or negative.
    /// Returns [`Error::Overflow`] if the total count would exceed `u64::MAX`.
    #[inline]
    pub fn update(&mut self, value: f64) -> Result<(), Error> {
        self.record_incr(value, 1)
    }

    /// Records a value with a specified increment.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Extreme`] if the value is NaN, ±Inf, or negative.
    /// Returns [`Error::Overflow`] if the total count would exceed `u64::MAX`.
    pub fn record_incr(&mut self, value: f64, incr: u64) -> Result<(), Error> {
        // Extract the raw exponent and significand (sign bit is ignored).
        let mut biased_exp = get_biased_exponent(value);
        let mut significand = get_significand(value);

        let new_count = self.checked_add_count(incr).ok_or(Error::Overflow)?;

        // Handle the extreme cases.
        match biased_exp {
            0 => {
                if significand == 0 {
                    // Zero case: no bucket, no min/max update.
                    self.stats.count = new_count;
                    return Ok(());
                } else if value.is_sign_negative() {
                    return Err(Error::Extreme);
                } else {
                    // Round subnormals into the first bucket that
                    // contains normal values (just above MIN_VALUE).
                    // significand = 1 avoids the power-of-two special
                    // case (significand 0 = exact 2^exp boundary).
                    biased_exp = 1;
                    significand = 1;
                }
            }
            NAN_INF_BIASED => {
                // Inf and NaN cases.
                return Err(Error::Extreme);
            }
            _ => {
                // Normal exponents, only positive.
                if value.is_sign_negative() {
                    return Err(Error::Extreme);
                }
            }
        }

        let base2_exp = unbias_exponent(biased_exp);

        self.stats.min = self.stats.min.min(value);
        self.stats.max = self.stats.max.max(value);
        self.update_decomposed(significand, base2_exp, incr)?;
        self.stats.sum += value * incr as f64;
        self.stats.count = new_count;
        Ok(())
    }

    /// Updates buckets for a decomposed value.
    fn update_decomposed(
        &mut self,
        significand: u64,
        base2_exp: i32,
        incr: u64,
    ) -> Result<(), Error> {
        self.retry_increment(incr, |h| {
            h.current.scale.map_decomposed(significand, base2_exp)
        })
    }

    /// Retries an increment until it succeeds, performing downscale or
    /// widen as needed between attempts.  `index_fn` is called each
    /// iteration because the scale may have changed.
    fn retry_increment(
        &mut self,
        incr: u64,
        mut index_fn: impl FnMut(&Self) -> i32,
    ) -> Result<(), Error> {
        loop {
            let index = index_fn(self);
            let result = self.try_increment(index, incr);
            if self.resolve_increment(result)? {
                return Ok(());
            }
        }
    }

    /// Decreases the scale by `decrease` steps.
    ///
    /// # Invariant
    ///
    /// Callers must ensure `decrease` does not push below `MIN_SCALE`.
    /// This is maintained by:
    /// - `do_downscale`: caps at `abs_budget` (scale − MIN_SCALE)
    /// - merge prepare: clamps at `MIN_SCALE` and pre-reserves headroom
    /// - `resolve_increment` empty path: only sets width, no scale change
    ///
    /// At `MIN_SCALE` the entire f64 exponent range maps to ≤ 2 bucket
    /// indices, so `N ≥ 2` guarantees the range always fits.
    pub(crate) fn change_scale(&mut self, decrease: u32) {
        let new_scale = self.current.scale.scale() - decrease as i32;
        debug_assert!(
            new_scale >= crate::mapping::MIN_SCALE,
            "change_scale({decrease}) would push scale from {} below MIN_SCALE ({})",
            self.current.scale.scale(),
            crate::mapping::MIN_SCALE,
        );
        self.current.scale = Scale::new(new_scale).expect("invariant: callers cap at MIN_SCALE");
    }

    pub(crate) fn downscale_by(&mut self, change: u32) {
        self.downscale_by_min(change, self.current.width);
    }

    /// Like `downscale_by` but guarantees the output width is at least
    /// `min_output_width`. Used by merge to prevent the narrow step
    /// from undoing the widening the merge path needs.
    fn downscale_by_min(&mut self, change: u32, min_output_width: Width) {
        if change == 0 {
            return;
        }

        let actual = self.do_downscale(change, min_output_width);
        self.change_scale(actual);
    }

    /// Attempts to add `incr` into the bucket at `index`.
    fn try_increment(&mut self, slot_index: i32, incr: u64) -> IncrResult {
        if incr == 0 {
            return IncrResult::Ok;
        }

        let width = self.current.width;
        let addr = width.slot_addr(slot_index);
        let word_index = addr.word_index();

        if self.buckets_empty() {
            self.word_start = word_index;
            self.word_end = self.word_start;
            self.word_base = self.word_start;
        } else if word_index < self.word_start {
            let diff = (self.word_end - word_index) as usize;
            if diff >= N {
                return IncrResult::NeedsDownscale(HighLow {
                    low: word_index,
                    high: self.word_end,
                });
            }
            for w in word_index..self.word_start {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_start = word_index;
        } else if word_index > self.word_end {
            let diff = (word_index - self.word_start) as usize;
            if diff >= N {
                return IncrResult::NeedsDownscale(HighLow {
                    low: self.word_start,
                    high: word_index,
                });
            }
            for w in (self.word_end + 1)..=word_index {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_end = word_index;
        }

        if let Err(oflow) = self.bucket_try_increment(&addr, incr) {
            return IncrResult::CounterOverflow(oflow);
        }

        IncrResult::Ok
    }

    /// Handles the result of `try_increment`, performing downscale
    /// or widen as needed.  Returns `Ok(true)` when the increment
    /// succeeded, `Ok(false)` when the caller should retry.
    fn resolve_increment(&mut self, result: IncrResult) -> Result<bool, Error> {
        match result {
            IncrResult::Ok => Ok(true),
            IncrResult::CounterOverflow(total) => {
                let new_width = Width::from_max_value(total);
                let change = new_width.subtract(self.current.width) as u32;
                if self.buckets_empty() {
                    // Empty histogram: no data to transform and the
                    // retry will place the first value fresh. Just
                    // widen the counter — no scale change needed.
                    self.current.width = new_width;
                } else {
                    // Route through do_downscale so the headroom
                    // invariant is maintained (narrowing is capped
                    // to leave room for future widening).
                    self.downscale_by_min(change, new_width);
                }
                Ok(false)
            }
            IncrResult::NeedsDownscale(hl) => {
                let change = hl.change_steps(N);
                self.downscale_by(change);
                Ok(false)
            }
        }
    }
}

/// Backward-compatible alias: `Histogram<N>` is `HistogramNN<N>`.
pub type Histogram<const N: usize> = HistogramNN<N>;

// Compile-time test that HistogramNN is Send + Sync
const fn _assert_send_sync<T: Send + Sync>() {}
const _: () = _assert_send_sync::<HistogramNN<2>>();

mod pn;
pub use pn::{HistogramPN, HistogramPNView};
