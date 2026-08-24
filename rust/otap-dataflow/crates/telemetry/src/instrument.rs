// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric instrument types.
//!
//! The instrumentation API intentionally distinguishes between:
//! - delta instruments (`Delta*`): you record per-interval deltas (e.g. `add(1)`),
//!   which are later accumulated by the registry.
//! - observe instruments (`Observe*`): you record the current observed value
//!   (e.g., `observe(total_bytes)`), which replaces the previous value in the registry.
//!
//! Gauges are instantaneous values that are set via `set`.

use otap_df_expohisto::{Error as HistogramError, HistogramNN};

/// Bucket totals recovered by [`DistributionValue::scan_buckets`], re-exported so
/// callers need not depend on `otap_df_expohisto` directly.
pub use otap_df_expohisto::BucketTotals;
use std::fmt::Debug;
use std::ops::{AddAssign, SubAssign};
use std::time::Instant;

/// A monotonic sum-like instrument reporting deltas over an interval.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct Counter<T>(T);

/// A sum-like instrument reporting signed deltas over an interval.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct UpDownCounter<T>(T);

/// A monotonic sum-like instrument reporting a current observed value.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct ObserveCounter<T>(T);

/// A sum-like instrument reporting a current observed value that may go up or down.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct ObserveUpDownCounter<T>(T);

/// An instantaneous measurement value.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct Gauge<T>(T);

// Counter implementation.
// =======================

impl<T: Copy + Default> Counter<T> {
    /// Creates a new delta counter with the provided initial value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Resets the counter to the default value (typically `0`).
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Returns the current accumulated delta value.
    #[inline]
    pub const fn get(&self) -> T {
        self.0
    }
}

impl Debug for Counter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter").field("value", &self.0).finish()
    }
}

impl Debug for Counter<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter").field("value", &self.0).finish()
    }
}

impl From<u64> for Counter<u64> {
    fn from(value: u64) -> Self {
        Counter(value)
    }
}

impl From<f64> for Counter<f64> {
    fn from(value: f64) -> Self {
        Counter(value)
    }
}

impl AddAssign<u64> for Counter<u64> {
    fn add_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_add(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += rhs;
        }
    }
}

impl AddAssign<f64> for Counter<f64> {
    fn add_assign(&mut self, rhs: f64) {
        debug_assert!(rhs >= 0.0, "Counter += called with negative value: {rhs}");
        self.0 += rhs;
    }
}

impl Counter<u64> {
    /// Increments the counter by `1`.
    #[inline]
    pub const fn inc(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_add(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += 1;
        }
    }

    /// Adds `v` to the counter.
    #[inline]
    pub const fn add(&mut self, v: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_add(v);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += v;
        }
    }
}

impl Counter<f64> {
    /// Increments the counter by `1.0`.
    #[inline]
    pub fn inc(&mut self) {
        self.0 += 1.0;
    }

    /// Adds `v` to the counter.
    #[inline]
    pub fn add(&mut self, v: f64) {
        debug_assert!(v >= 0.0, "Counter::add called with negative value: {v}");
        self.0 += v;
    }
}

// UpDownCounter implementation.
// =============================

impl<T> UpDownCounter<T>
where
    T: Copy
        + Default
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + AddAssign
        + SubAssign,
{
    /// Creates a new delta up/down counter with the provided initial value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Resets the counter to the default value (typically `0`).
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Returns the current accumulated delta value.
    #[inline]
    pub const fn get(&self) -> T {
        self.0
    }

    /// Adds `v` to the counter (positive or negative depending on `T`).
    #[inline]
    pub fn add(&mut self, v: T) {
        self.0 += v;
    }

    /// Subtracts `v` from the counter.
    #[inline]
    pub fn sub(&mut self, v: T) {
        self.0 -= v;
    }
}

impl Debug for UpDownCounter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpDownCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl Debug for UpDownCounter<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpDownCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl From<u64> for UpDownCounter<u64> {
    fn from(value: u64) -> Self {
        UpDownCounter(value)
    }
}

impl From<f64> for UpDownCounter<f64> {
    fn from(value: f64) -> Self {
        UpDownCounter(value)
    }
}

impl AddAssign<u64> for UpDownCounter<u64> {
    fn add_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_add(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += rhs;
        }
    }
}

impl AddAssign<f64> for UpDownCounter<f64> {
    fn add_assign(&mut self, rhs: f64) {
        self.0 += rhs;
    }
}

impl SubAssign<u64> for UpDownCounter<u64> {
    fn sub_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_sub(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 -= rhs;
        }
    }
}

impl SubAssign<f64> for UpDownCounter<f64> {
    fn sub_assign(&mut self, rhs: f64) {
        self.0 -= rhs;
    }
}

impl UpDownCounter<u64> {
    /// Increments the counter by `1`.
    #[inline]
    pub const fn inc(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_add(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += 1;
        }
    }

    /// Decrements the counter by `1`.
    #[inline]
    pub const fn dec(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            self.0 = self.0.wrapping_sub(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 -= 1;
        }
    }
}

// ObserveCounter implementation.
// ==============================

impl<T: Copy + Default> ObserveCounter<T> {
    /// Creates a new observe counter with the provided initial value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Resets the observed value to the default (typically `0`).
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Records a new observed value, replacing the previous one.
    #[inline]
    pub const fn observe(&mut self, v: T) {
        self.0 = v;
    }

    /// Returns the last observed value.
    #[inline]
    pub const fn get(&self) -> T {
        self.0
    }
}

impl Debug for ObserveCounter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserveCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl Debug for ObserveCounter<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserveCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl From<u64> for ObserveCounter<u64> {
    fn from(value: u64) -> Self {
        ObserveCounter(value)
    }
}

impl From<f64> for ObserveCounter<f64> {
    fn from(value: f64) -> Self {
        ObserveCounter(value)
    }
}

// ObserveUpDownCounter implementation.
// ====================================

impl<T: Copy + Default> ObserveUpDownCounter<T> {
    /// Creates a new observe up/down counter with the provided initial value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Resets the observed value to the default (typically `0`).
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Records a new observed value, replacing the previous one.
    #[inline]
    pub const fn observe(&mut self, v: T) {
        self.0 = v;
    }

    /// Returns the last observed value.
    #[inline]
    pub const fn get(&self) -> T {
        self.0
    }
}

impl Debug for ObserveUpDownCounter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserveUpDownCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl Debug for ObserveUpDownCounter<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObserveUpDownCounter")
            .field("value", &self.0)
            .finish()
    }
}

impl From<u64> for ObserveUpDownCounter<u64> {
    fn from(value: u64) -> Self {
        ObserveUpDownCounter(value)
    }
}

impl From<f64> for ObserveUpDownCounter<f64> {
    fn from(value: f64) -> Self {
        ObserveUpDownCounter(value)
    }
}

// Gauge implementation.
// =====================

impl<T> Gauge<T>
where
    T: Copy + Default + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    /// Creates a new gauge with the provided initial value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Resets the gauge to the default value (typically `0`).
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Sets the current gauge value.
    #[inline]
    pub const fn set(&mut self, v: T) {
        self.0 = v;
    }

    /// Returns the current gauge value.
    #[inline]
    pub const fn get(&self) -> T {
        self.0
    }
}

impl Debug for Gauge<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gauge").field("value", &self.0).finish()
    }
}

impl Debug for Gauge<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gauge").field("value", &self.0).finish()
    }
}

impl From<u64> for Gauge<u64> {
    fn from(value: u64) -> Self {
        Gauge(value)
    }
}

impl From<f64> for Gauge<f64> {
    fn from(value: f64) -> Self {
        Gauge(value)
    }
}

// Mmsc implementation.
// ====================

/// A pre-aggregated summary metric tracking min, max, sum, and count.
///
/// Records individual observations via [`record()`](Mmsc::record), maintaining
/// running min/max/sum/count. This is a delta instrument -- values are reset
/// after each reporting interval.
///
/// An `Mmsc` knows only the range its observations occupied. Its OTLP
/// projection is therefore an explicit-boundary histogram point with no
/// boundaries or bucket counts, preserving min, max, sum, and count without
/// inventing bucket membership. It is carried as [`DistributionValue::Basic`] so
/// all pre-aggregated distributions share one internal metric value kind.
///
/// Exact zeros are not tracked separately. The bucketed tiers keep a
/// `zero_count` because a positive-only exponential histogram has no bucket
/// that can hold a zero, but this tier has no buckets to be excluded from:
/// a zero is simply an observation that lowers `min`, and `min == 0.0` already
/// reports that zeros occurred.
///
/// An empty `Mmsc` is all zeros. `min` and `max` carry no sentinel, so they
/// are meaningless until an observation is recorded and must not be read
/// unless [`count`](Mmsc::count) is non-zero. Callers that render or export an
/// aggregation are expected to skip it entirely while it is empty, exactly as
/// an empty exponential histogram is skipped.
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Mmsc {
    /// Minimum observed value. Only meaningful when `count` is non-zero.
    pub min: f64,
    /// Maximum observed value. Only meaningful when `count` is non-zero.
    pub max: f64,
    /// Sum of all observed values.
    pub sum: f64,
    /// Total number of observations.
    pub count: u64,
}

impl Debug for Mmsc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mmsc")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("sum", &self.sum)
            .field("count", &self.count)
            .finish()
    }
}

impl Mmsc {
    /// Records a single non-negative observation, updating min/max/sum/count.
    ///
    /// NaN and the infinities are invalid, as they are for the histogram
    /// tiers: in debug builds they trip a debug assertion, and in release they
    /// are dropped so a misbehaving call site cannot corrupt the aggregation.
    /// Negative values are invalid too, and are recorded as given.
    ///
    /// Negative zero is counted as the zero it compares equal to. The
    /// histogram tiers assert on it instead, because the type behind them
    /// declares a non-negative domain and tests the sign bit to enforce it.
    /// Both report a plain positive zero, so the tiers differ only in whether
    /// they complain.
    #[inline]
    pub fn record(&mut self, value: f64) {
        // A non-finite value is dropped rather than folded in: `sum` is an
        // accumulator, so a single NaN or infinity would poison it for the
        // rest of the interval and every value recorded after it would be
        // lost. The histogram tiers reject them too, so this much of the
        // contract is shared.
        if !value.is_finite() {
            debug_assert!(
                false,
                "Mmsc::record called with a non-finite value: {value}"
            );
            return;
        }
        debug_assert!(
            value >= 0.0,
            "Mmsc::record called with negative value: {value}"
        );
        // Negative zero is counted as the zero it compares equal to, so it is
        // normalized here rather than left to surface as a negative minimum in
        // an export. The histogram tiers report a plain zero for it as well,
        // though they assert on it first.
        let value = if value == 0.0 { 0.0 } else { value };
        // An empty aggregation has no min/max to compare against -- both are
        // 0.0 -- so the first observation is adopted outright.
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            if value < self.min {
                self.min = value;
            }
            if value > self.max {
                self.max = value;
            }
        }
        self.sum += value;
        self.count += 1;
    }

    /// Returns a copy of the current aggregation.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> Self {
        *self
    }

    /// Returns `true` when no observation was recorded this interval.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Resets all fields for the next reporting interval.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Merge another Mmsc into this.
    ///
    /// Both empty cases are handled explicitly, since an empty aggregation's
    /// min/max are zeros that would otherwise pull the merged minimum to 0.0.
    #[inline]
    pub fn merge(&mut self, other: Self) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other;
            return;
        }
        if other.min < self.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max;
        }
        self.sum += other.sum;
        self.count += other.count;
    }
}

/// A lightweight wall-clock timer.
///
/// Call [`Timer::start`] to capture the current instant, then call
/// [`Timer::elapsed_nanos`] to get the elapsed duration.
#[must_use]
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Capture the current instant.
    #[inline]
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Consume the timer and return the elapsed wall-clock duration
    /// in nanoseconds as an f64.
    #[inline]
    #[must_use]
    pub fn elapsed_nanos(self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1e9
    }
}

// DistributionValue implementation.
// ============================

/// Number of `u64` bucket words in the "normal" resolution exponential
/// histogram tier.
///
/// Sized for a compact per-series footprint suitable for always-on internal
/// telemetry while still capturing a useful bucket range.
/// Size = (10+6)*8 = 128 bytes
pub const HISTOGRAM_NORMAL_WORDS: usize = 10;

/// Number of `u64` bucket words in the "detailed" resolution exponential
/// histogram tier.
///
/// Trades a larger per-series footprint for finer bucket coverage, for metrics
/// that warrant high-resolution distributions.
/// Size = (26+6)*8 = 256 bytes
pub const HISTOGRAM_DETAILED_WORDS: usize = 26;

/// A snapshot of a delta distribution, at one of three resolution tiers.
///
/// Every tier is a pre-aggregated distribution:
/// - [`DistributionValue::Basic`] is an [`Mmsc`] with exact min/max/sum/count and no
///   bucket structure.
/// - [`DistributionValue::Normal`] and [`DistributionValue::Detailed`] keep full
///   exponential-histogram bucket ranges sized by [`HISTOGRAM_NORMAL_WORDS`]
///   and [`HISTOGRAM_DETAILED_WORDS`] respectively.
///
/// This is what an instrument hands to the reporting path: components record
/// into [`Mmsc`], [`HistogramNormal`], or [`HistogramDetailed`], each of which
/// yields the matching variant from its `get`. Nothing records into a
/// `DistributionValue` itself, because the tier a component needs is a property of
/// the metric and is fixed by the declared field type. Selecting it at runtime
/// awaits a way to resolve it from configuration.
///
/// The tier is carried rather than erased because the reporting path needs it:
/// the OTLP bridge exports [`DistributionValue::Basic`] as a bucketless histogram
/// point and the other two as exponential-histogram points, and the descriptor
/// declares which to expect.
///
/// Tiers differ widely in size (a detailed histogram is roughly 256 bytes), so
/// each variant is boxed and the enum itself stays pointer-small. That keeps a
/// basic-tier series cheap to carry by value, which is why
/// [`crate::metrics::MetricValue`] embeds a `DistributionValue` directly rather than
/// boxing it a second time.
#[derive(Debug, Clone)]
pub enum DistributionValue {
    /// Basic tier: exact min/max/sum/count with no encoded buckets.
    Basic(Box<Mmsc>),
    /// Normal tier: exponential histogram with [`HISTOGRAM_NORMAL_WORDS`] bucket words.
    Normal(Box<HistogramNN<HISTOGRAM_NORMAL_WORDS>>),
    /// Detailed tier: exponential histogram with [`HISTOGRAM_DETAILED_WORDS`] bucket words.
    Detailed(Box<HistogramNN<HISTOGRAM_DETAILED_WORDS>>),
}

impl DistributionValue {
    /// Resets all state for the next reporting interval.
    #[inline]
    pub fn reset(&mut self) {
        match self {
            Self::Basic(mmsc) => mmsc.reset(),
            Self::Normal(hist) => hist.clear(),
            Self::Detailed(hist) => hist.clear(),
        }
    }

    /// Returns the total number of observations recorded this interval.
    #[inline]
    #[must_use]
    pub fn count(&self) -> u64 {
        self.summary().0
    }

    /// Returns `true` when no observations have been recorded this interval.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns the tier name (`"basic"`, `"normal"`, or `"detailed"`).
    #[inline]
    #[must_use]
    pub fn tier_name(&self) -> &'static str {
        match self {
            Self::Basic(_) => "basic",
            Self::Normal(_) => "normal",
            Self::Detailed(_) => "detailed",
        }
    }

    /// Returns a `(count, sum, min, max)` summary of this interval's observations.
    ///
    /// When `count` is zero every other field is zero as well: there is no
    /// observation to summarize, so `min` and `max` carry no meaning and must
    /// not be rendered or exported. Callers are expected to branch on `count`
    /// (or [`DistributionValue::is_empty`]) before reading them.
    #[must_use]
    pub fn summary(&self) -> (u64, f64, f64, f64) {
        match self {
            Self::Basic(mmsc) => (mmsc.count, mmsc.sum, mmsc.min, mmsc.max),
            Self::Normal(hist) => {
                let s = hist.view().stats();
                (s.count, s.sum, s.min, s.max)
            }
            Self::Detailed(hist) => {
                let s = hist.view().stats();
                (s.count, s.sum, s.min, s.max)
            }
        }
    }

    /// Walks this tier's buckets exactly once, passing each count to `emit`,
    /// and returns the totals learned by that walk -- including the number of
    /// observations excluded from the bucket range because they were exactly
    /// zero.
    ///
    /// A positive-only exponential histogram has no bucket that can hold a
    /// zero, so the zero count is recoverable only as `count - sum(positive
    /// bucket counts)`: a full scan of the buckets. Exposing it only here ties
    /// that cost to the encoding pass that has to walk the buckets anyway --
    /// an encoder learns the zero count for free, and no caller can pay for a
    /// scan whose bucket counts it then discards.
    ///
    /// [`DistributionValue::Basic`] encodes no buckets, so it emits nothing and
    /// reports [`BucketTotals::EMPTY`] regardless of whether zeros were
    /// observed. A zero there is an ordinary observation that lowers `min`;
    /// its OTLP summary point does not claim any bucket membership.
    pub fn scan_buckets<F>(&self, emit: F) -> BucketTotals
    where
        F: FnMut(u64),
    {
        match self {
            Self::Basic(_) => BucketTotals::EMPTY,
            Self::Normal(hist) => hist.view().scan_buckets(emit),
            Self::Detailed(hist) => hist.view().scan_buckets(emit),
        }
    }

    /// Returns the relative error bound of this tier's quantile estimates.
    ///
    /// The bound is the observable consequence of the bucket scale, which is
    /// why the scale itself is not exposed here: a consumer of the estimates
    /// needs the error they carry, not the encoding that produced it.
    ///
    /// Returns `None` for [`DistributionValue::Basic`], which encodes no buckets
    /// and so reports no estimated values, and 0.0 for an empty histogram.
    #[must_use]
    pub fn relative_error(&self) -> Option<f64> {
        match self {
            Self::Basic(_) => None,
            Self::Normal(hist) => Some(hist.view().relative_error()),
            Self::Detailed(hist) => Some(hist.view().relative_error()),
        }
    }

    /// Estimates the values at the requested quantiles into `out`, returning
    /// the [`BucketTotals`] scanned while doing so.
    ///
    /// `quantiles` must be sorted in non-decreasing order with every entry in
    /// `[0.0, 1.0]`, and `out` must be at least as long. Returns `None`
    /// without touching `out` for [`DistributionValue::Basic`], which keeps no
    /// buckets and therefore cannot estimate interior quantiles.
    ///
    /// Quantile estimation has to know the zero mass before it can walk the
    /// buckets, so the totals come free with the estimates: a caller that
    /// needs both -- the admin JSON rendering, for instance -- must not scan
    /// again via [`scan_buckets`](Self::scan_buckets).
    ///
    /// Estimates carry the error bound reported by
    /// [`relative_error`](Self::relative_error).
    pub fn quantiles(&self, quantiles: &[f64], out: &mut [f64]) -> Option<BucketTotals> {
        match self {
            Self::Basic(_) => None,
            Self::Normal(hist) => Some(hist.view().quantiles(quantiles, out)),
            Self::Detailed(hist) => Some(hist.view().quantiles(quantiles, out)),
        }
    }

    /// Merges another distribution of the same tier into this one.
    ///
    /// Merging mismatched tiers is a programming error: in debug builds it
    /// trips an assertion and in release builds it is a no-op. Histogram merges
    /// that would overflow a bucket counter are likewise reported as debug
    /// assertions.
    pub fn merge(&mut self, other: &Self) {
        match (self, other) {
            (Self::Basic(dst), Self::Basic(src)) => dst.merge(**src),
            (Self::Normal(dst), Self::Normal(src)) => {
                Self::check_hist(dst.merge_from(&**src), "merge overflow");
            }
            (Self::Detailed(dst), Self::Detailed(src)) => {
                Self::check_hist(dst.merge_from(&**src), "merge overflow");
            }
            _ => debug_assert!(false, "DistributionValue::merge across mismatched tiers"),
        }
    }

    #[inline]
    fn check_hist(result: Result<(), HistogramError>, context: &str) {
        if let Err(error) = result {
            debug_assert!(false, "DistributionValue::{context}: {error}");
        }
    }
}

/// Summary equality: two distributions are equal when they share a tier and
/// agree on the observable aggregate statistics (count, sum, min, max) and on
/// the bucket totals a scan recovers.
///
/// The vendored histogram does not implement structural equality, and the
/// bucket layout is an implementation detail; comparing the summary is
/// sufficient for the registry's equality needs and for tests. Distinct bucket
/// distributions that share a summary compare equal.
///
/// The bucket scan here is not the access pattern
/// [`DistributionValue::scan_buckets`] exists to discourage: an equality test must
/// look at the buckets, and it discards the counts only after comparing the
/// totals they produce.
impl PartialEq for DistributionValue {
    fn eq(&self, other: &Self) -> bool {
        self.tier_name() == other.tier_name()
            && self.summary() == other.summary()
            && self.scan_buckets(|_| {}) == other.scan_buckets(|_| {})
    }
}

#[inline]
fn check_hist_update(result: Result<(), HistogramError>, context: &str) {
    if let Err(error) = result {
        debug_assert!(false, "{context}: {error}");
    }
}

/// An exponential-histogram instrument holding `N` counter words.
///
/// Records non-negative observations into a [`HistogramNN`] and yields them as
/// a live [`DistributionValue`]. The tiers below differ only in how many counter
/// words they carry, so they share everything except which variant they
/// report.
#[derive(Clone, Default)]
pub struct Histogram<const N: usize>(HistogramNN<N>);

impl<const N: usize> Debug for Histogram<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Histogram")
            .field("words", &N)
            .field("count", &self.0.view().stats().count)
            .finish_non_exhaustive()
    }
}

impl<const N: usize> Histogram<N> {
    /// Records a single non-negative observation.
    ///
    /// Negative, NaN, and infinite values are invalid. In debug builds an
    /// invalid value trips a debug assertion; in release builds a non-finite
    /// one is dropped so a misbehaving call site cannot corrupt the
    /// aggregation. A negative value is counted in the bucket for its
    /// magnitude, while the extremes and the sum keep its sign, which is why
    /// the OTLP bridge withholds the sum of a population whose minimum is
    /// negative.
    ///
    /// Negative zero counts as negative here, unlike in [`Mmsc`]: the
    /// underlying histogram declares a non-negative domain and tests the sign
    /// bit to enforce it. It is still recorded as the zero it is, so both
    /// tiers report a plain positive zero for it.
    #[inline]
    pub fn record(&mut self, value: f64) {
        check_hist_update(self.0.update(value), "Histogram::record rejected value");
    }

    /// Merges another same-tier histogram into this one.
    ///
    /// Both histograms retain their exact aggregate statistics while their
    /// bucket ranges are reconciled at the coarsest scale needed to fit the
    /// combined observations.
    #[inline]
    pub fn merge(&mut self, other: Self) {
        check_hist_update(self.0.merge_from(&other.0), "Histogram::merge overflow");
    }

    /// Returns `true` when no observations have been recorded this interval.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.view().stats().count == 0
    }

    /// Resets the histogram for the next reporting interval.
    #[inline]
    pub fn reset(&mut self) {
        self.0.clear();
    }
}

/// A normal-tier exponential-histogram instrument.
///
/// Declared as a `#[metric_set]` field type to select the normal tier.
pub type HistogramNormal = Histogram<HISTOGRAM_NORMAL_WORDS>;

/// A detailed-tier exponential-histogram instrument.
///
/// Declared as a `#[metric_set]` field type to select the detailed tier.
pub type HistogramDetailed = Histogram<HISTOGRAM_DETAILED_WORDS>;

impl HistogramNormal {
    /// Returns the current aggregation as a live [`DistributionValue`].
    #[inline]
    #[must_use]
    pub fn get(&self) -> DistributionValue {
        DistributionValue::Normal(Box::new(self.0.clone()))
    }
}

impl HistogramDetailed {
    /// Returns the current aggregation as a live [`DistributionValue`].
    #[inline]
    #[must_use]
    pub fn get(&self) -> DistributionValue {
        DistributionValue::Detailed(Box::new(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a basic-tier snapshot by recording through the instrument that
    /// produces it, which is the only way a distribution is populated.
    fn basic_of(values: &[f64]) -> DistributionValue {
        let mut mmsc = Mmsc::default();
        for &value in values {
            mmsc.record(value);
        }
        DistributionValue::Basic(Box::new(mmsc))
    }

    /// Builds a normal-tier snapshot by recording through its instrument.
    fn normal_of(values: &[f64]) -> DistributionValue {
        let mut histogram = HistogramNormal::default();
        for &value in values {
            histogram.record(value);
        }
        histogram.get()
    }

    /// Builds a detailed-tier snapshot by recording through its instrument.
    fn detailed_of(values: &[f64]) -> DistributionValue {
        let mut histogram = HistogramDetailed::default();
        for &value in values {
            histogram.record(value);
        }
        histogram.get()
    }

    /// Scenario: A non-finite value is offered to each of the three tiers in a
    /// build with debug assertions on.
    /// Guarantees: Every tier rejects it loudly, so a call site recording NaN
    /// or an infinity is caught in development rather than silently producing
    /// a poisoned or truncated aggregation in production.
    #[cfg(debug_assertions)]
    #[test]
    fn every_tier_asserts_on_a_non_finite_value() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for tier in ["basic", "normal", "detailed"] {
                let result = std::panic::catch_unwind(|| match tier {
                    "basic" => Mmsc::default().record(value),
                    "normal" => HistogramNormal::default().record(value),
                    _ => HistogramDetailed::default().record(value),
                });
                assert!(
                    result.is_err(),
                    "the {tier} tier accepted {value} without asserting"
                );
            }
        }
    }

    /// Scenario: A negative value is offered to each of the three tiers in a
    /// build with debug assertions on.
    /// Guarantees: Every tier rejects it loudly. The tiers are documented as
    /// differing only in size and resolution, so they must agree on what
    /// counts as a valid observation.
    #[cfg(debug_assertions)]
    #[test]
    fn every_tier_asserts_on_a_negative_value() {
        for tier in ["basic", "normal", "detailed"] {
            let result = std::panic::catch_unwind(|| match tier {
                "basic" => Mmsc::default().record(-1.0),
                "normal" => HistogramNormal::default().record(-1.0),
                _ => HistogramDetailed::default().record(-1.0),
            });
            assert!(
                result.is_err(),
                "the {tier} tier accepted -1.0 without asserting"
            );
        }
    }

    /// Scenario: NaN and both infinities are recorded into each tier in a
    /// build with debug assertions off, which is how a released binary runs.
    /// Guarantees: All three drop the value and leave the aggregation
    /// untouched, so one bad call site cannot poison a sum for the rest of the
    /// reporting interval, and a field can be moved between tiers without
    /// changing what happens to invalid input.
    #[cfg(not(debug_assertions))]
    #[test]
    fn every_tier_drops_a_non_finite_value_in_release() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut mmsc = Mmsc::default();
            mmsc.record(1.0);
            mmsc.record(value);
            assert_eq!(
                (mmsc.count, mmsc.sum, mmsc.min, mmsc.max),
                (1, 1.0, 1.0, 1.0)
            );

            let mut normal = HistogramNormal::default();
            normal.record(1.0);
            normal.record(value);
            assert_eq!(normal.get().summary(), (1, 1.0, 1.0, 1.0));

            let mut detailed = HistogramDetailed::default();
            detailed.record(1.0);
            detailed.record(value);
            assert_eq!(detailed.get().summary(), (1, 1.0, 1.0, 1.0));
        }
    }

    /// Scenario: Negative zero is recorded into the summary tier.
    /// Guarantees: It is counted as the zero it compares equal to, and the
    /// stored minimum carries no sign bit, so an exporter reading this tier
    /// cannot report a negative minimum for a population of zeros.
    #[test]
    fn mmsc_counts_negative_zero_as_zero() {
        let mut mmsc = Mmsc::default();
        mmsc.record(-0.0);

        assert_eq!(mmsc.count, 1);
        assert_eq!(mmsc.sum, 0.0);
        assert_eq!(mmsc.min, 0.0);
        // Equality cannot see the sign bit, and a serializer can.
        assert!(
            !mmsc.min.is_sign_negative(),
            "minimum kept the sign of -0.0"
        );
    }

    /// Scenario: Negative zero is recorded into the histogram tiers in a build
    /// with debug assertions on.
    /// Guarantees: Both reject it. They are backed by a type that declares a
    /// non-negative domain and tests the sign bit to enforce it, so a value
    /// that only reaches zero from below is reported to its author rather than
    /// quietly accepted.
    #[cfg(debug_assertions)]
    #[test]
    fn histogram_tiers_assert_on_negative_zero() {
        for tier in ["normal", "detailed"] {
            let result = std::panic::catch_unwind(|| {
                if tier == "normal" {
                    HistogramNormal::default().record(-0.0);
                } else {
                    HistogramDetailed::default().record(-0.0);
                }
            });
            assert!(
                result.is_err(),
                "the {tier} tier accepted -0.0 without asserting"
            );
        }
    }

    /// Scenario: Negative zero is recorded into each tier in a build with
    /// debug assertions off, which is how a released binary runs.
    /// Guarantees: All three count it and report a plain positive zero as the
    /// minimum. The tiers differ on whether -0.0 is worth asserting about, but
    /// not on what they export for it, so moving a field between them cannot
    /// change a shipped aggregation.
    #[cfg(not(debug_assertions))]
    #[test]
    fn every_tier_reports_negative_zero_as_positive_zero_in_release() {
        let mut mmsc = Mmsc::default();
        mmsc.record(-0.0);
        let mut normal = HistogramNormal::default();
        normal.record(-0.0);
        let mut detailed = HistogramDetailed::default();
        detailed.record(-0.0);

        for (tier, summary) in [
            ("basic", (mmsc.count, mmsc.sum, mmsc.min, mmsc.max)),
            ("normal", normal.get().summary()),
            ("detailed", detailed.get().summary()),
        ] {
            assert_eq!(summary, (1, 0.0, 0.0, 0.0), "{tier} tier");
            assert!(
                !summary.2.is_sign_negative(),
                "the {tier} tier kept the sign of -0.0"
            );
        }
    }

    #[test]
    fn test_delta_counter_u64_add_inc() {
        let mut counter = Counter::new(10u64);
        counter.add(5);
        counter.inc();
        assert_eq!(counter.get(), 16);
    }

    #[test]
    fn test_delta_counter_f64_add_inc() {
        let mut counter = Counter::new(0.0f64);
        counter.add(1.5);
        counter.inc();
        assert!((counter.get() - 2.5).abs() < f64::EPSILON);
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_delta_counter_overflow_panics_without_unchecked_arithmetic() {
        let mut counter = Counter::new(u64::MAX);
        counter.inc();
    }

    #[test]
    fn test_observe_counter_observe() {
        let mut counter = ObserveCounter::new(0u64);
        counter.observe(123);
        assert_eq!(counter.get(), 123);
    }

    #[test]
    fn test_observe_up_down_counter_observe() {
        let mut counter = ObserveUpDownCounter::new(0i64);
        counter.observe(-7);
        assert_eq!(counter.get(), -7);
    }

    #[test]
    fn test_gauge_set() {
        let mut gauge = Gauge::new(0u64);
        gauge.set(42);
        assert_eq!(gauge.get(), 42);
    }

    #[test]
    fn test_mmsc_single_record() {
        let mut mmsc = Mmsc::default();
        mmsc.record(42.0);
        let snap = mmsc.get();
        assert_eq!(snap.min, 42.0);
        assert_eq!(snap.max, 42.0);
        assert_eq!(snap.sum, 42.0);
        assert_eq!(snap.count, 1);
    }

    #[test]
    fn test_mmsc_multiple_records() {
        let mut mmsc = Mmsc::default();
        mmsc.record(10.0);
        mmsc.record(5.0);
        mmsc.record(20.0);
        mmsc.record(15.0);
        let snap = mmsc.get();
        assert_eq!(snap.min, 5.0);
        assert_eq!(snap.max, 20.0);
        assert_eq!(snap.sum, 50.0);
        assert_eq!(snap.count, 4);
    }

    /// Scenario: An `Mmsc` with recorded observations is reset for the next
    /// reporting interval.
    /// Guarantees: every field returns to zero, including min and max, so a
    /// reset aggregation carries no sentinel that a consumer could mistake
    /// for an observed extreme.
    #[test]
    fn test_mmsc_reset() {
        let mut mmsc = Mmsc::default();
        mmsc.record(10.0);
        mmsc.record(20.0);
        mmsc.reset();
        assert_eq!(mmsc.get(), Mmsc::default());
        assert!(mmsc.is_empty());
    }

    /// Scenario: A default-constructed `Mmsc` is inspected before anything is
    /// recorded.
    /// Guarantees: all five fields are zero, so an empty aggregation is
    /// indistinguishable from an all-zero struct and never exposes
    /// `f64::MAX`/`f64::MIN` sentinels to a renderer or exporter.
    #[test]
    fn test_mmsc_default_no_observations() {
        let snap = Mmsc::default().get();
        assert_eq!(snap.min, 0.0);
        assert_eq!(snap.max, 0.0);
        assert_eq!(snap.sum, 0.0);
        assert_eq!(snap.count, 0);
    }

    /// Scenario: The first observation recorded into an empty `Mmsc` is
    /// greater than the zeroed min field.
    /// Guarantees: min is adopted from that first observation rather than
    /// staying at 0.0, so a population with no zeros never reports a minimum
    /// of zero.
    #[test]
    fn test_mmsc_first_record_adopts_min() {
        let mut mmsc = Mmsc::default();
        mmsc.record(42.0);
        assert_eq!(mmsc.min, 42.0);
        assert_eq!(mmsc.max, 42.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "Mmsc::record called with negative value")]
    fn test_mmsc_record_rejects_negative() {
        let mut mmsc = Mmsc::default();
        mmsc.record(-1.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "Counter::add called with negative value")]
    fn test_counter_f64_add_rejects_negative() {
        let mut counter = Counter::new(0.0f64);
        counter.add(-1.0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "Counter += called with negative value")]
    fn test_counter_f64_add_assign_rejects_negative() {
        let mut counter = Counter::new(0.0f64);
        counter += -1.0;
    }

    #[test]
    fn test_mmsc_merge_both_populated() {
        let mut a = Mmsc::default();
        a.record(2.0);
        a.record(8.0);

        let mut b = Mmsc::default();
        b.record(1.0);
        b.record(5.0);
        b.record(10.0);

        a.merge(b);
        let snap = a.get();
        assert_eq!(snap.min, 1.0);
        assert_eq!(snap.max, 10.0);
        assert_eq!(snap.sum, 26.0);
        assert_eq!(snap.count, 5);
    }

    #[test]
    fn test_mmsc_merge_into_empty() {
        let mut a = Mmsc::default();

        let mut b = Mmsc::default();
        b.record(3.0);
        b.record(7.0);

        a.merge(b);
        let snap = a.get();
        assert_eq!(snap.min, 3.0);
        assert_eq!(snap.max, 7.0);
        assert_eq!(snap.sum, 10.0);
        assert_eq!(snap.count, 2);
    }

    #[test]
    fn test_mmsc_merge_empty_into_populated() {
        let mut a = Mmsc::default();
        a.record(4.0);

        let b = Mmsc::default();
        a.merge(b);

        let snap = a.get();
        assert_eq!(snap.min, 4.0);
        assert_eq!(snap.max, 4.0);
        assert_eq!(snap.sum, 4.0);
        assert_eq!(snap.count, 1);
    }

    #[test]
    fn test_mmsc_merge_both_empty() {
        let mut a = Mmsc::default();
        let b = Mmsc::default();
        a.merge(b);
        let snap = a.get();
        assert_eq!(snap.count, 0);
    }

    /// Scenario: A basic-tier distribution records several non-negative values.
    /// Guarantees: The basic tier preserves exact min/max/sum/count, matching
    /// the standalone Mmsc instrument it wraps.
    #[test]
    fn test_distribution_basic_records_mmsc_summary() {
        let dist = basic_of(&[10.0, 5.0, 20.0, 15.0]);
        assert_eq!(dist.count(), 4);
        let DistributionValue::Basic(mmsc) = &dist else {
            panic!("expected basic tier")
        };
        let snap = mmsc.get();
        assert_eq!(snap.min, 5.0);
        assert_eq!(snap.max, 20.0);
        assert_eq!(snap.sum, 50.0);
        assert_eq!(snap.count, 4);
    }

    /// Scenario: Normal- and detailed-tier distributions record positive values.
    /// Guarantees: Both histogram tiers accept observations and expose the exact
    /// count and sum through their view, confirming the boxed histograms are
    /// wired to expohisto.
    #[test]
    fn test_distribution_histogram_tiers_record_into_buckets() {
        let values = [1.5_f64, 2.7, 4.0, 100.0];
        for dist in [normal_of(&values), detailed_of(&values)] {
            assert_eq!(dist.count(), 4);
            let stats = match &dist {
                DistributionValue::Normal(hist) => hist.view().stats(),
                DistributionValue::Detailed(hist) => hist.view().stats(),
                DistributionValue::Basic(_) => panic!("expected histogram tier"),
            };
            assert_eq!(stats.count, 4);
            assert!((stats.sum - 108.2).abs() < 1e-9);
            assert_eq!(stats.min, 1.5);
            assert_eq!(stats.max, 100.0);
        }
    }

    /// Scenario: A fresh distribution and a reset distribution are inspected.
    /// Guarantees: A new instrument is empty, and resetting after recording
    /// returns it to the empty state so each delta interval starts clean.
    #[test]
    fn test_distribution_reset_clears_all_tiers() {
        for (empty, mut recorded) in [
            (basic_of(&[]), basic_of(&[3.0])),
            (normal_of(&[]), normal_of(&[3.0])),
            (detailed_of(&[]), detailed_of(&[3.0])),
        ] {
            assert!(empty.is_empty());
            assert!(!recorded.is_empty());
            recorded.reset();
            assert!(recorded.is_empty());
            assert_eq!(recorded.count(), 0);
        }
    }

    /// Scenario: Two same-tier distributions with disjoint observations are
    /// merged, for both the basic and histogram tiers.
    /// Guarantees: Merging accumulates counts and sums across tiers, which the
    /// registry relies on to fold per-thread aggregations together.
    #[test]
    fn test_distribution_merge_same_tier_accumulates() {
        let mut basic_a = basic_of(&[2.0, 8.0]);
        let basic_b = basic_of(&[1.0, 10.0]);
        basic_a.merge(&basic_b);
        let DistributionValue::Basic(mmsc) = &basic_a else {
            panic!("expected basic tier")
        };
        let snap = mmsc.get();
        assert_eq!(snap.count, 4);
        assert_eq!(snap.min, 1.0);
        assert_eq!(snap.max, 10.0);
        assert_eq!(snap.sum, 21.0);

        let mut hist_a = normal_of(&[1.5, 2.5]);
        let hist_b = normal_of(&[3.5]);
        hist_a.merge(&hist_b);
        assert_eq!(hist_a.count(), 3);
    }

    /// Scenario: Two normal histogram instruments contain disjoint observation ranges.
    /// Guarantees: Direct instrument merging retains their combined count, sum, minimum, and maximum.
    #[test]
    fn histogram_merge_accumulates_same_tier_observations() {
        let mut left = HistogramNormal::default();
        left.record(1.0);
        left.record(4.0);
        let mut right = HistogramNormal::default();
        right.record(16.0);
        right.record(64.0);

        left.merge(right);

        assert_eq!(left.get().summary(), (4, 85.0, 1.0, 64.0));
    }

    /// Scenario: The normal tier is asked to record a negative value in a debug
    /// build.
    /// Guarantees: Invalid observations are rejected the same way the basic tier
    /// rejects them, tripping the shared debug assertion rather than silently
    /// corrupting the aggregation.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn test_distribution_histogram_rejects_negative() {
        let mut histogram = HistogramNormal::default();
        histogram.record(-1.0);
    }

    /// Scenario: A normal-tier distribution records exact zeros alongside
    /// positive values and its buckets are scanned once.
    /// Guarantees: Zeros count toward the total and toward `min` without being
    /// bucketed, and the scan's totals recover them so that
    /// `zero_count + sum(positive bucket counts) == count`, with the bucket
    /// counts delivered by the same pass.
    #[test]
    fn test_distribution_recovers_zero_count_from_total() {
        let dist = normal_of(&[0.0, 1.5, 2.7, 4.0]);

        let (count, sum, min, max) = dist.summary();
        assert_eq!(count, 4);
        assert!((sum - 8.2).abs() < 1e-9);
        assert_eq!(min, 0.0);
        assert_eq!(max, 4.0);

        let mut counts = Vec::new();
        let totals = dist.scan_buckets(|c| counts.push(c));
        assert_eq!(totals.zero_count, 1);
        assert_eq!(counts.iter().sum::<u64>(), totals.positive_total);
        assert_eq!(totals.zero_count + totals.positive_total, count);
    }

    /// Scenario: A basic-tier distribution records exact zeros alongside
    /// positive values.
    /// Guarantees: The basic tier does not track zeros separately -- its bucket
    /// scan emits nothing and reports empty totals, folding zeros into min
    /// instead -- and its OTLP projection does not fabricate bucket counts.
    #[test]
    fn test_basic_tier_folds_zeros_into_min() {
        let dist = basic_of(&[0.0, 0.0, 4.0]);

        assert_eq!(dist.count(), 3);
        let mut emitted = 0_usize;
        let totals = dist.scan_buckets(|_| emitted += 1);
        assert_eq!(emitted, 0);
        assert_eq!(totals, BucketTotals::EMPTY);
        let (_count, _sum, min, max) = dist.summary();
        assert_eq!(min, 0.0);
        assert_eq!(max, 4.0);
    }

    /// Scenario: Two basic-tier distributions, one of which recorded only exact
    /// zeros, are merged.
    /// Guarantees: The zero population survives the merge through count and
    /// min, so a registry fold of per-thread aggregations neither loses those
    /// observations nor reports a spurious minimum.
    #[test]
    fn test_basic_tier_merge_preserves_zero_observations() {
        let mut a = basic_of(&[2.0, 2.0]);
        let b = basic_of(&[0.0, 0.0]);

        a.merge(&b);

        assert_eq!(a.count(), 4);
        assert_eq!(a.scan_buckets(|_| {}), BucketTotals::EMPTY);
        let (_count, _sum, min, max) = a.summary();
        assert_eq!(min, 0.0);
        assert_eq!(max, 2.0);
    }
}
