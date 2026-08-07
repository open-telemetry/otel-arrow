// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation-free exponential histogram with a unified flat memory layout.
//!
//! `HistogramNN<N>` stores everything in fixed struct fields plus a `[u64; N]`
//! data pool used for bucket counters.

mod downscale;
mod merge;
mod swar;
#[cfg(test)]
mod verify;
mod view;
pub mod width;

use crate::float64::{NAN_INF_BIASED, get_biased_exponent, get_significand, unbias_exponent};
use crate::mapping::{Scale, ScaleError, table_scale};
use core::num::NonZeroU64;
pub use view::{BucketTotals, BucketView, BucketsIter, HistogramView};
pub use width::{SlotAddr, Width};

use core::fmt;

const ONE: NonZeroU64 = NonZeroU64::new(1).expect("one is non-zero");

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
    /// Invalid value: NaN or +/-Inf.
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
/// observations". `HistogramNN::buckets_empty` distinguishes the two only in
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
/// Negative values trigger a debug_assert! and record absolute
/// value with assertions disabled. The test is on the sign bit, so negative
/// zero counts as negative here even though it compares equal to zero; it is
/// still recorded as the zero it is.
pub struct HistogramNN<const N: usize> {
    initial: Settings,
    current: Settings,

    word_base: i32,
    word_start: i32,
    word_end: i32,

    // This field omits zero_count; it is implied by the difference
    // between Stats.count and the sum of the data counts.
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

    /// Folds `other`'s statistics into this histogram's, taking the merged
    /// total count from `merged_count`.
    ///
    /// `merged_count` is passed in rather than computed here because the
    /// caller must check it for overflow before it touches any bucket; `sum`
    /// and `min`/`max` cannot overflow, so they are combined here.
    ///
    /// An empty destination has no min/max to merge against -- its sentinels
    /// are both 0.0 -- so the source values are adopted outright. Folding them
    /// in with `f64::min` would otherwise pin the merged minimum at 0.0 and
    /// report a zero observation that was never recorded.
    ///
    /// `count` is the only safe test for that empty state, and it must be read
    /// before `merged_count` overwrites it: a destination that recorded
    /// nothing but exact zeros also has empty buckets and a 0.0 minimum, but
    /// its zeros are real observations that must participate in the fold. See
    /// [`Stats`].
    fn commit_merge(&mut self, other: &Stats, merged_count: u64) {
        if self.stats.count == 0 {
            self.stats.min = other.min;
            self.stats.max = other.max;
        } else {
            self.stats.min = self.stats.min.min(other.min);
            self.stats.max = self.stats.max.max(other.max);
        }
        self.stats.sum += other.sum;
        self.stats.count = merged_count;
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

    /// Returns the slot address of a bucket index.
    #[inline]
    const fn slot_addr(&self, slot: i32) -> SlotAddr {
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
    ///
    /// `word_base` always sits inside the active window and the window never
    /// exceeds `N` words, so the offset is within one turn of the ring and a
    /// single correction replaces a modulo.
    #[inline]
    const fn data_idx(&self, widx: i32) -> usize {
        let size = N as i32;
        let offset = widx - self.word_base;
        debug_assert!(
            offset > -size && offset < size,
            "word index is more than one turn from the base"
        );
        if offset < 0 {
            (offset + size) as usize
        } else {
            offset as usize
        }
    }

    /// Gets the value at a slot address.
    #[inline]
    pub(super) const fn bucket_get(&self, addr: &SlotAddr) -> u64 {
        let idx = addr.data_index(N, self.word_base);
        let word = self.data[idx];
        addr.retrieve_counter(word)
    }

    /// Attempts to add `incr` to a physical slot. Returns false on overflow.
    #[inline]
    fn bucket_try_increment(&mut self, addr: &SlotAddr, incr: u64) -> Result<(), u64> {
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

    /// The fewest words a histogram may hold, which is what lets `MIN_SCALE`
    /// cover the whole value range at u64 width.
    pub const MIN_WORDS: usize = 2;

    /// The most words a histogram may hold, enforced as a compile-time bound
    /// so that an implausible pool size fails to build rather than reserving
    /// the space.
    pub const MAX_WORDS: usize = 250;

    /// Compile-time bounds check on `N`, forced by `new`. A violation
    /// is a compile error, not a runtime panic.
    const CHECK_N: () = {
        assert!(N >= Self::MIN_WORDS, "HistogramNN requires N >= MIN_WORDS");
        assert!(N <= Self::MAX_WORDS, "HistogramNN requires N <= MAX_WORDS");
    };

    /// Creates a new histogram at the maximum supported scale.
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
        // The reset mapping exposes only physical word 0. Other words are
        // cleared lazily by try_increment before they re-enter the window.
        self.data[0] = 0;
    }

    /// Records a single value.
    #[inline]
    pub fn update(&mut self, value: f64) -> Result<(), Error> {
        self.record_incr(value, ONE)
    }

    /// Records a value with a specified increment.
    #[inline]
    pub fn record_incr(&mut self, value: f64, incr: NonZeroU64) -> Result<(), Error> {
        let incr = incr.get();

        // Extract the raw exponent and significand.
        let mut biased_exp = get_biased_exponent(value);
        let mut significand = get_significand(value);

        // Reject NaN and the infinities before the sign check below, so that
        // -Inf is reported as the extreme value it is rather than as a
        // negative one, and so NaN is not measured against a bound it compares
        // false against.
        if biased_exp == NAN_INF_BIASED {
            return Err(Error::Extreme);
        }

        let new_count = self.checked_add_count(incr).ok_or(Error::Overflow)?;

        // This type declares a non-negative domain, so it enforces the sign
        // bit rather than a comparison: -0.0 compares equal to zero and would
        // pass `value >= 0.0`. Reaching it takes a negation, a multiply or
        // divide by a negative, or rounding a small negative toward zero, each
        // of which suggests the caller computed something that was not
        // non-negative to begin with. Bucketing itself uses the magnitude.
        debug_assert!(!value.is_sign_negative(), "value must be non-negative");

        // Zeros are counted but never bucketed, and subnormals round up to the
        // smallest normal value.
        if biased_exp == 0 {
            if significand == 0 {
                // Zero case: update min, zero_count implicit in count.
                self.stats.min = 0.0;
                self.stats.count = new_count;
                return Ok(());
            }
            biased_exp = 1;
            significand = 0;
        }

        let base2_exp = unbias_exponent(biased_exp);

        if self.stats.count != 0 {
            // NaN was rejected above, so plain comparisons are safe and avoid
            // the NaN and signed-zero handling f64::min and f64::max carry.
            if value < self.stats.min {
                self.stats.min = value;
            }
            if value > self.stats.max {
                self.stats.max = value;
            }
        } else {
            self.stats.min = value;
            self.stats.max = value;
        }
        self.update_decomposed(significand, base2_exp, incr)?;
        // The recording path always increments by one, and converting that
        // to a float sits on the dependency chain for the sum.
        self.stats.sum += if incr == 1 {
            value
        } else {
            value * incr as f64
        };
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
        let index = self.current.scale.map_decomposed(significand, base2_exp);
        if self.fast_increment(index, incr) {
            return Ok(());
        }
        self.retry_increment(incr, |h| {
            h.current.scale.map_decomposed(significand, base2_exp)
        })
    }

    /// Adds `incr` to the bucket at `index` when doing so needs no change of
    /// geometry, returning whether it succeeded.
    ///
    /// This is the case almost every observation takes, and separating it lets
    /// it compile to straight-line code: the retry loop below has to reload the
    /// scale and window on each turn, because widening or downscaling between
    /// turns invalidates both, and that pessimizes the common path where
    /// neither happens.
    ///
    /// Bailing out is always safe. Every rejection here is a case the retry
    /// loop handles, and nothing has been written yet.
    #[inline]
    fn fast_increment(&mut self, index: i32, incr: u64) -> bool {
        let width = self.current.width;
        let addr = width.slot_addr(index);
        let word_index = addr.word_index();

        // Outside the active window the bucket may need the window to grow,
        // which can in turn need a downscale.
        if word_index < self.word_start || word_index > self.word_end {
            return false;
        }

        let data_index = addr.data_index(N, self.word_base);
        let word = self.data[data_index];
        let count = addr.retrieve_counter(word);
        // Written as headroom so the check cannot overflow, whatever `incr` is.
        if incr > width.counter_max() - count {
            return false;
        }

        self.data[data_index] = addr.add_counter_in_word(word, incr);
        true
    }

    /// Retries an increment until it succeeds, performing downscale or
    /// widen as needed between attempts. `index_fn` is called each
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
    /// Callers must ensure `decrease` does not push below `MIN_SCALE`.
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

    /// Widens counters to at least `target`.
    ///
    /// Wider counters are paid for in one of two ways. Spreading them into
    /// words the histogram is not using keeps every bucket boundary, so the
    /// scale survives; that is the choice whenever the wider layout still fits
    /// `N` words. Otherwise one level is folded in place, which sums adjacent
    /// buckets and costs a scale step, and the choice is made again.
    ///
    /// Folding one level at a time is what keeps the cost minimal: each fold
    /// halves the span the target width has to cover, so the loop stops at the
    /// first scale that fits rather than paying a step per width level.
    pub(crate) fn widen_to(&mut self, target: Width) {
        loop {
            let start = self.current.width;
            if target <= start {
                return;
            }
            if self.buckets_empty() {
                self.current.width = target;
                self.debug_assert_range_coverage();
                return;
            }

            let words = HighLow {
                low: target.slot_to_word_index(self.first_slot()),
                high: target.slot_to_word_index(self.last_slot()),
            };
            if words.change_steps(N) == 0 {
                self.spread_widen(start, target, words.low, words.high);
                self.debug_assert_range_coverage();
                return;
            }

            // Folding pairs of lanes cannot overflow the doubled lane, so this
            // step always succeeds and always moves one level closer.
            let next = start.wider_by(1).expect("target is wider than start");
            let _ = self.widen_words(start, next);
            self.current.width = next;
            self.change_scale(1);
        }
    }

    /// Spreads packed counters into unused words without changing scale.
    fn spread_widen(&mut self, before: Width, after: Width, low_word: i32, high_word: i32) {
        let source = self.clone();
        let steps = after.subtract(before) as u32;
        let words_per_source = 1_i32 << steps;
        let packed_bits = 64_u32 >> steps;

        self.current.width = after;
        self.word_start = low_word;
        self.word_end = high_word;
        self.word_base = low_word;

        for word_index in low_word..=high_word {
            let source_word = word_index >> steps;
            let chunk = word_index.rem_euclid(words_per_source) as u32;
            let packed = source.data[source.data_idx(source_word)] >> (chunk * packed_bits);
            let data_index = self.data_idx(word_index);
            self.data[data_index] = swar::spread(before, after, packed);
        }
    }

    /// Attempts to add `incr` into the bucket at `index`.
    fn try_increment(&mut self, slot_index: i32, incr: u64) -> IncrResult {
        if incr == 0 {
            return IncrResult::Ok;
        }

        let width = self.current.width;
        let addr = width.slot_addr(slot_index);
        let word_index = addr.word_index();

        // The common case is a word already inside the active window, which
        // moves nothing. Testing that first keeps the emptiness check, and the
        // load it makes, off the recording path.
        if word_index < self.word_start || word_index > self.word_end {
            if let Some(blocked) = self.open_word(word_index) {
                return blocked;
            }
        }

        if let Err(oflow) = self.bucket_try_increment(&addr, incr) {
            return IncrResult::CounterOverflow(oflow);
        }

        IncrResult::Ok
    }

    /// Brings `word_index` into the active window, zeroing whatever it exposes.
    ///
    /// Returns the downscale the caller must perform first when the word lies
    /// further than `N` words from the opposite end of the window.
    ///
    /// An empty histogram keeps its one-word window anchored at `word_base`,
    /// so its physical word is always index 0, which is the word `clear` and
    /// `new` leave zeroed.
    #[cold]
    fn open_word(&mut self, word_index: i32) -> Option<IncrResult> {
        if self.buckets_empty() {
            self.word_start = word_index;
            self.word_end = word_index;
            self.word_base = word_index;
            return None;
        }

        if word_index < self.word_start {
            let diff = (self.word_end - word_index) as usize;
            if diff >= N {
                return Some(IncrResult::NeedsDownscale(HighLow {
                    low: word_index,
                    high: self.word_end,
                }));
            }
            for w in word_index..self.word_start {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_start = word_index;
        } else {
            let diff = (word_index - self.word_start) as usize;
            if diff >= N {
                return Some(IncrResult::NeedsDownscale(HighLow {
                    low: self.word_start,
                    high: word_index,
                }));
            }
            for w in (self.word_end + 1)..=word_index {
                self.data[self.data_idx(w)] = 0;
            }
            self.word_end = word_index;
        }

        None
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
