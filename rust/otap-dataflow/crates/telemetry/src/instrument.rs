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

use std::fmt::Debug;
use std::ops::{AddAssign, SubAssign};

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
/// The dispatcher uses `.with_boundaries(vec![])` when building the OTel
/// histogram to disable bucket counting, so only min, max, sum, and count
/// are exported. See [`record_synthetic_histogram`] for details.
///
/// [`record_synthetic_histogram`]: crate::metrics::dispatcher::MetricsDispatcher
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

    #[test]
    fn test_mmsc_negative_values() {
        let mut mmsc = Mmsc::default();
        mmsc.record(-5.0);
        mmsc.record(-10.0);
        mmsc.record(-1.0);
        let snap = mmsc.get();
        assert_eq!(snap.min, -10.0);
        assert_eq!(snap.max, -1.0);
        assert_eq!(snap.sum, -16.0);
        assert_eq!(snap.count, 3);
    }
}
