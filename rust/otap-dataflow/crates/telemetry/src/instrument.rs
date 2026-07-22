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

use otap_df_expohisto::{Error as HistogramError, Histogram};
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

/// A pre-aggregated summary metric tracking min, max, sum, count.
///
/// Records individual observations via [`record()`](Mmsc::record), maintaining
/// running min/max/sum/count. Exported as a synthetic OTel histogram preserving
/// exact MMSC values. This is a delta instrument — values are reset after each
/// reporting interval.
///
/// The OTLP projection emits this as a bucketless histogram, so min, max, sum,
/// and count are preserved without reconstructing observations.
#[derive(Clone, Copy)]
pub struct Mmsc {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

impl Default for Mmsc {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: f64::MIN,
            sum: 0.0,
            count: 0,
        }
    }
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
    /// Records a single observation, updating min/max/sum/count.
    #[inline]
    pub fn record(&mut self, value: f64) {
        debug_assert!(
            value >= 0.0,
            "Mmsc::record called with negative value: {value}"
        );
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
        self.count += 1;
    }

    /// Returns the current MMSC snapshot.
    #[inline]
    #[must_use]
    pub const fn get(&self) -> MmscSnapshot {
        MmscSnapshot {
            min: self.min,
            max: self.max,
            sum: self.sum,
            count: self.count,
        }
    }

    /// Resets all fields for the next reporting interval.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Merge another Mmsc into this.
    ///
    /// Handles the empty-receiver case explicitly so the result is
    /// correct regardless of sentinel values in a default/reset `Mmsc`.
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

/// An immutable snapshot of MMSC (min, max, sum, count) values.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MmscSnapshot {
    /// Minimum observed value.
    pub min: f64,
    /// Maximum observed value.
    pub max: f64,
    /// Sum of all observed values.
    pub sum: f64,
    /// Number of observations.
    pub count: u64,
}

// Distribution implementation.
// ============================

/// Number of `u64` bucket words in the "normal" resolution exponential
/// histogram tier.
///
/// Sized for a compact per-series footprint suitable for always-on internal
/// telemetry while still capturing a useful bucket range.
pub const HISTOGRAM_NORMAL_WORDS: usize = 10;

/// Number of `u64` bucket words in the "detailed" resolution exponential
/// histogram tier.
///
/// Trades a larger per-series footprint for finer bucket coverage, for metrics
/// that warrant high-resolution distributions.
pub const HISTOGRAM_DETAILED_WORDS: usize = 26;

/// A delta distribution instrument with three resolution tiers.
///
/// Every tier projects onto the OTLP exponential-histogram point type, so
/// distributions are represented consistently regardless of resolution:
/// - [`Distribution::Basic`] keeps only exact min/max/sum/count and encodes as
///   a bucketless point.
/// - [`Distribution::Normal`] and [`Distribution::Detailed`] keep full
///   exponential-histogram bucket ranges sized by [`HISTOGRAM_NORMAL_WORDS`]
///   and [`HISTOGRAM_DETAILED_WORDS`] respectively.
///
/// Like [`Mmsc`], this is a delta instrument: observations are recorded over an
/// interval and then cleared via [`reset`](Distribution::reset) after each
/// report. All tiers are boxed so the enum stays pointer-small when carried by
/// value.
#[derive(Debug, Clone)]
pub enum Distribution {
    /// Basic tier: exact min/max/sum/count with no buckets.
    Basic(Box<Mmsc>),
    /// Normal tier: exponential histogram with [`HISTOGRAM_NORMAL_WORDS`] bucket words.
    Normal(Box<Histogram<HISTOGRAM_NORMAL_WORDS>>),
    /// Detailed tier: exponential histogram with [`HISTOGRAM_DETAILED_WORDS`] bucket words.
    Detailed(Box<Histogram<HISTOGRAM_DETAILED_WORDS>>),
}

impl Distribution {
    /// Creates a basic-tier distribution tracking only min/max/sum/count.
    #[inline]
    #[must_use]
    pub fn basic() -> Self {
        Self::Basic(Box::<Mmsc>::default())
    }

    /// Creates a normal-tier exponential-histogram distribution.
    #[inline]
    #[must_use]
    pub fn normal() -> Self {
        Self::Normal(Box::new(Histogram::new()))
    }

    /// Creates a detailed-tier exponential-histogram distribution.
    #[inline]
    #[must_use]
    pub fn detailed() -> Self {
        Self::Detailed(Box::new(Histogram::new()))
    }

    /// Records a single non-negative observation.
    ///
    /// Mirrors [`Mmsc::record`]: negative, NaN, and infinite values are
    /// invalid. In debug builds an invalid value trips a debug assertion; in
    /// release builds it is dropped so a misbehaving call site cannot corrupt
    /// the aggregation.
    #[inline]
    pub fn record(&mut self, value: f64) {
        match self {
            Self::Basic(mmsc) => mmsc.record(value),
            Self::Normal(hist) => Self::check_hist(hist.update(value), "record rejected value"),
            Self::Detailed(hist) => Self::check_hist(hist.update(value), "record rejected value"),
        }
    }

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
        match self {
            Self::Basic(mmsc) => mmsc.get().count,
            Self::Normal(hist) => hist.view().stats().count,
            Self::Detailed(hist) => hist.view().stats().count,
        }
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
    /// For an empty distribution `min`/`max` are the aggregation's sentinels.
    #[must_use]
    pub fn summary(&self) -> (u64, f64, f64, f64) {
        match self {
            Self::Basic(mmsc) => {
                let s = mmsc.get();
                (s.count, s.sum, s.min, s.max)
            }
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
            _ => debug_assert!(false, "Distribution::merge across mismatched tiers"),
        }
    }

    #[inline]
    fn check_hist(result: Result<(), HistogramError>, context: &str) {
        if let Err(error) = result {
            debug_assert!(false, "Distribution::{context}: {error}");
        }
    }
}

/// Summary equality: two distributions are equal when they share a tier and
/// agree on the observable aggregate statistics (count, sum, min, max).
///
/// The vendored [`Histogram`] does not implement structural equality, and the
/// bucket layout is an implementation detail; comparing the summary is
/// sufficient for the registry's equality needs and for tests. Distinct bucket
/// distributions that share a summary compare equal.
impl PartialEq for Distribution {
    fn eq(&self, other: &Self) -> bool {
        fn summary(dist: &Distribution) -> (u8, u64, f64, f64, f64) {
            match dist {
                Distribution::Basic(mmsc) => {
                    let s = mmsc.get();
                    (0, s.count, s.sum, s.min, s.max)
                }
                Distribution::Normal(hist) => {
                    let s = hist.view().stats();
                    (1, s.count, s.sum, s.min, s.max)
                }
                Distribution::Detailed(hist) => {
                    let s = hist.view().stats();
                    (2, s.count, s.sum, s.min, s.max)
                }
            }
        }
        summary(self) == summary(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_mmsc_reset() {
        let mut mmsc = Mmsc::default();
        mmsc.record(10.0);
        mmsc.record(20.0);
        mmsc.reset();
        let snap = mmsc.get();
        assert_eq!(snap.min, f64::MAX);
        assert_eq!(snap.max, f64::MIN);
        assert_eq!(snap.sum, 0.0);
        assert_eq!(snap.count, 0);
    }

    #[test]
    fn test_mmsc_default_no_observations() {
        let mmsc = Mmsc::default();
        let snap = mmsc.get();
        assert_eq!(snap.min, f64::MAX);
        assert_eq!(snap.max, f64::MIN);
        assert_eq!(snap.sum, 0.0);
        assert_eq!(snap.count, 0);
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

    // Scenario: A basic-tier distribution records several non-negative values.
    // Guarantees: The basic tier preserves exact min/max/sum/count, matching
    // the standalone Mmsc instrument it wraps.
    #[test]
    fn test_distribution_basic_records_mmsc_summary() {
        let mut dist = Distribution::basic();
        for v in [10.0, 5.0, 20.0, 15.0] {
            dist.record(v);
        }
        assert_eq!(dist.count(), 4);
        let Distribution::Basic(mmsc) = &dist else {
            panic!("expected basic tier")
        };
        let snap = mmsc.get();
        assert_eq!(snap.min, 5.0);
        assert_eq!(snap.max, 20.0);
        assert_eq!(snap.sum, 50.0);
        assert_eq!(snap.count, 4);
    }

    // Scenario: Normal- and detailed-tier distributions record positive values.
    // Guarantees: Both histogram tiers accept observations and expose the exact
    // count and sum through their view, confirming the boxed histograms are
    // wired to the vendored expohisto aggregation.
    #[test]
    fn test_distribution_histogram_tiers_record_into_buckets() {
        for mut dist in [Distribution::normal(), Distribution::detailed()] {
            for v in [1.5_f64, 2.7, 4.0, 100.0] {
                dist.record(v);
            }
            assert_eq!(dist.count(), 4);
            let stats = match &dist {
                Distribution::Normal(hist) => hist.view().stats(),
                Distribution::Detailed(hist) => hist.view().stats(),
                Distribution::Basic(_) => panic!("expected histogram tier"),
            };
            assert_eq!(stats.count, 4);
            assert!((stats.sum - 108.2).abs() < 1e-9);
            assert_eq!(stats.min, 1.5);
            assert_eq!(stats.max, 100.0);
        }
    }

    // Scenario: A fresh distribution and a reset distribution are inspected.
    // Guarantees: A new instrument is empty, and resetting after recording
    // returns it to the empty state so each delta interval starts clean.
    #[test]
    fn test_distribution_reset_clears_all_tiers() {
        for mut dist in [
            Distribution::basic(),
            Distribution::normal(),
            Distribution::detailed(),
        ] {
            assert!(dist.is_empty());
            dist.record(3.0);
            assert!(!dist.is_empty());
            dist.reset();
            assert!(dist.is_empty());
            assert_eq!(dist.count(), 0);
        }
    }

    // Scenario: Two same-tier distributions with disjoint observations are
    // merged, for both the basic and histogram tiers.
    // Guarantees: Merging accumulates counts and sums across tiers, which the
    // registry relies on to fold per-thread aggregations together.
    #[test]
    fn test_distribution_merge_same_tier_accumulates() {
        let mut basic_a = Distribution::basic();
        basic_a.record(2.0);
        basic_a.record(8.0);
        let mut basic_b = Distribution::basic();
        basic_b.record(1.0);
        basic_b.record(10.0);
        basic_a.merge(&basic_b);
        let Distribution::Basic(mmsc) = &basic_a else {
            panic!("expected basic tier")
        };
        let snap = mmsc.get();
        assert_eq!(snap.count, 4);
        assert_eq!(snap.min, 1.0);
        assert_eq!(snap.max, 10.0);
        assert_eq!(snap.sum, 21.0);

        let mut hist_a = Distribution::normal();
        hist_a.record(1.5);
        hist_a.record(2.5);
        let mut hist_b = Distribution::normal();
        hist_b.record(3.5);
        hist_a.merge(&hist_b);
        assert_eq!(hist_a.count(), 3);
    }

    // Scenario: The normal tier is asked to record a negative value in a debug
    // build.
    // Guarantees: Invalid observations are rejected the same way the basic tier
    // rejects them, tripping the shared debug assertion rather than silently
    // corrupting the aggregation.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "Distribution::record rejected value")]
    fn test_distribution_histogram_rejects_negative() {
        let mut dist = Distribution::normal();
        dist.record(-1.0);
    }
}
