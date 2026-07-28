// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free exponential histogram with a unified flat memory layout.
//!
//! `HistogramNN<N>` stores everything in fixed struct fields plus a `[u64; N]`
//! data pool used for bucket counters.

mod downscale;
mod merge;
mod swar;
mod view;
pub mod width;

use crate::float64::{NAN_INF_BIASED, get_biased_exponent, get_significand, unbias_exponent};
use crate::mapping::{Scale, ScaleError, range_buckets_at_min_scale, table_scale};
pub use view::{BucketView, BucketsIter, HistogramView};
pub use width::{SlotAddr, Width};

use core::fmt;

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
    /// Invalid value: NaN or +/-Inf for all histogram types, or negative
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
///
/// `count` is the sole indicator of emptiness, and it separates two states
/// that the bucket contents alone cannot:
///
/// - `count == 0` -- the aggregator is empty. `sum`, `min`, and `max` are
///   0.0 sentinels standing for "no observation", not recorded values. A merge
///   must adopt the source's `min`/`max` outright rather than folding them
///   against these, or every merged result reports a minimum of 0.0.
/// - `count != 0` with empty buckets -- every observation was an exact zero,
///   so `zero_count == count`. Here `min`, `max`, and `sum` are all genuinely
///   0.0, and folding them against another population is correct.
///
/// Exact zeros never occupy a bucket, so "no buckets" does not mean "no
/// observations". [`HistogramNN::buckets_empty`] distinguishes the two only in
/// combination with `count`.
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
        min: 0.0,
        max: 0.0,
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
pub struct HistogramNN<const N: usize> {
    initial: Settings,
    current: Settings,

    word_base: i32,
    word_start: i32,
    word_end: i32,

    // This field omits zero_count; it is implied by the difference
    // between Stats.count and sum of data counts.
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
    ///
    /// An empty destination has no min/max to merge against -- its sentinels
    /// are both 0.0 -- so the source values are adopted outright. Folding them
    /// in with `f64::min` would otherwise pin the merged minimum at 0.0 and
    /// report a zero observation that was never recorded.
    ///
    /// `count` is the only safe test for that empty state: a destination that
    /// recorded nothing but exact zeros also has empty buckets and a 0.0
    /// minimum, but its zeros are real observations that must participate in
    /// the fold. See [`Stats`].
    fn commit_stats(&mut self, stats: &Stats) {
        if self.stats.count == 0 {
            self.stats.min = stats.min;
            self.stats.max = stats.max;
        } else {
            self.stats.min = self.stats.min.min(stats.min);
            self.stats.max = self.stats.max.max(stats.max);
        }
        self.stats.sum = stats.sum;
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

    /// Debug-only check of the range-coverage invariant: the two u64
    /// words every histogram has must not hold more buckets than the
    /// value range covers at the current scale (see
    /// [`Width::min_scale`]). Equivalently, the scale can still pay
    /// for widening these counters to [`Width::U64`].
    ///
    /// The margin `scale - width.min_scale()` is the histogram's
    /// slack: the range relaxation it can still absorb, which
    /// `do_downscale` spends one step at a time.
    #[inline]
    pub(crate) fn debug_assert_range_coverage(&self) {
        debug_assert!(
            self.current.scale.scale() >= self.current.width.min_scale(),
            "scale {} is below the floor {} for width {:?}",
            self.current.scale.scale(),
            self.current.width.min_scale(),
            self.current.width,
        );
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

    /// Fewest words a histogram may hold.
    ///
    /// At `MIN_SCALE` the counters have widened to `Width::U64`, where
    /// a word is a single bucket, so the window needs one word per
    /// bucket the value range spans there. Below this the histogram
    /// could run out of range before it runs out of scale.
    pub const MIN_WORDS: usize = range_buckets_at_min_scale() as usize;

    /// Most words a histogram may hold.
    ///
    /// This allows up to 16k single-bit buckets and limits the struct
    /// to 2048 bytes, noting that the structure itself uses 6 u64.
    /// Nothing breaks above this limit, only performance: the
    /// algorithms here are designed for cache-line sized data.
    pub const MAX_WORDS: usize = 250;

    /// Compile-time bounds check on `N`, forced by `new`. A violation
    /// is a compile error, not a runtime panic.
    const CHECK_N: () = {
        assert!(N >= Self::MIN_WORDS, "HistogramNN requires N >= MIN_WORDS");
        assert!(N <= Self::MAX_WORDS, "HistogramNN requires N <= MAX_WORDS");
    };

    /// Creates a new histogram at the maximum supported scale.
    ///
    /// `N` is checked at compile time against [`Self::MIN_WORDS`] and
    /// [`Self::MAX_WORDS`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let () = Self::CHECK_N;

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
    /// The histogram's `N` words hold `N * slots_per_u64(width)`
    /// buckets at the configured minimum width. That many buckets must
    /// fit within the range of buckets covering the normal IEEE 754
    /// range at `scale`; see [`Self::min_scale`]. Set
    /// [`with_min_width`](Self::with_min_width) first when using a
    /// scale near the floor.
    ///
    /// # Errors
    ///
    /// Returns [`ScaleError::InvalidScale`] if `scale` is outside
    /// [`MIN_SCALE`](crate::MIN_SCALE)..=[`table_scale()`](crate::table_scale),
    /// or [`ScaleError::RangeCoverage`] if `scale` is below
    /// [`Self::min_scale`] for the current minimum width.
    #[inline]
    pub fn with_max_scale(mut self, scale: i32) -> Result<Self, ScaleError> {
        let s = Scale::new(scale)?;
        if scale < Self::min_scale(self.initial.width) {
            return Err(ScaleError::RangeCoverage);
        }
        self.initial.scale = s;
        self.current.scale = s;
        Ok(self)
    }

    /// Sets the minimum bucket width.
    ///
    /// # Errors
    ///
    /// Returns [`ScaleError::RangeCoverage`] if the current scale is
    /// below [`Self::min_scale`] for `width`, i.e. if these `N` words
    /// would define more buckets than the value range covers.
    #[inline]
    pub fn with_min_width(mut self, width: Width) -> Result<Self, ScaleError> {
        if self.initial.scale.scale() < Self::min_scale(width) {
            return Err(ScaleError::RangeCoverage);
        }
        self.initial.width = width;
        self.current.width = width;
        Ok(self)
    }

    /// Number of buckets this histogram defines at `width`.
    #[inline]
    #[must_use]
    pub const fn bucket_capacity(width: Width) -> u64 {
        N as u64 * width.slots_per_u64() as u64
    }

    /// Lowest scale at which all of this histogram's buckets still
    /// fit inside the value range, given counters of `width`.
    ///
    /// The normal IEEE 754 range spans about `2^(scale + 11)` buckets
    /// (see the internal `min_scale_for` helper). A
    /// histogram configured with more buckets than that has buckets no
    /// value can ever land in, so this is the floor the builders
    /// enforce.
    ///
    /// Operations may still take a large histogram below this scale --
    /// merging in a coarser histogram, for instance -- which only
    /// leaves words unused. The floor that always holds is
    /// [`Width::min_scale`], the same bound for the two words every
    /// histogram has.
    #[must_use]
    pub fn min_scale(width: Width) -> i32 {
        crate::mapping::min_scale_for(Self::bucket_capacity(width))
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
    /// Returns [`Error::Extreme`] if the value is NaN, +/-Inf, or negative.
    /// Returns [`Error::Overflow`] if the total count would exceed `u64::MAX`.
    #[inline]
    pub fn update(&mut self, value: f64) -> Result<(), Error> {
        self.record_incr(value, 1)
    }

    /// Records a value with a specified increment.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Extreme`] if the value is NaN, +/-Inf, or negative.
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
                    // Zero case: no bucket. Stats start at 0.0, and this
                    // histogram is positive-only, so a zero observation can
                    // only lower an established minimum to exactly 0.0 and
                    // can never raise the maximum.
                    self.stats.min = 0.0;
                    self.stats.count = new_count;
                    return Ok(());
                } else if value.is_sign_negative() {
                    return Err(Error::Extreme);
                } else {
                    // Round subnorms to MIN_VALUE
                    biased_exp = 1;
                    significand = 0;
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

        if self.stats.count != 0 {
            self.stats.min = self.stats.min.min(value);
            self.stats.max = self.stats.max.max(value);
        } else {
            self.stats.min = value;
            self.stats.max = value;
        }
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
    /// This holds because of the range-coverage invariant: the buckets
    /// a histogram defines always fit inside the buckets covering the
    /// value range at its current scale (see [`Self::min_scale`]), so
    /// no caller can need to downscale past the point where the range
    /// collapses onto the buckets it already has. At `MIN_SCALE` the
    /// whole f64 range is 2 buckets, which `N >= 2` words hold at
    /// `Width::U64`.
    ///
    /// The invariant is established by `with_max_scale`/`with_min_width`
    /// and preserved by `do_downscale`, which spends exactly the slack
    /// the requested range relaxation costs.
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

    pub(crate) fn downscale_by(&mut self, range_steps: u32) {
        if range_steps == 0 {
            return;
        }
        let actual = self.do_downscale(range_steps);
        self.change_scale(actual);
    }

    /// Widens every counter to `target`, which must not be narrower
    /// than the current width.
    ///
    /// Widening sums adjacent buckets, so it costs one scale step per
    /// level and leaves the range covered unchanged. An empty
    /// histogram has nothing to sum and so pays nothing.
    pub(crate) fn widen_to(&mut self, target: Width) {
        let start = self.current.width;
        debug_assert!(target >= start, "{target:?} is narrower than {start:?}");
        if target == start {
            return;
        }
        if !self.buckets_empty() {
            let _ = self.widen_words(start, target);
            self.change_scale(target.subtract(start) as u32);
        }
        self.current.width = target;
        self.debug_assert_range_coverage();
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
                // The value already fits the range; only the counter
                // is too small.
                self.widen_to(Width::from_max_value(total));
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

// Compile-time test that HistogramNN is Send + Sync
const fn _assert_send_sync<T: Send + Sync>() {}
const _: () = _assert_send_sync::<HistogramNN<2>>();
