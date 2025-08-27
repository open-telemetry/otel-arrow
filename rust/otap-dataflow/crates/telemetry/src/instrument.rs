// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The different types of instrument used to record the metric.
//!
//! These instruments are designed to be used in thread-per-core scenarios.
//!
//! ToDo Finish the implementation of UpDownCounter and Gauge (clean_values and needs_flush need some massage).
//! ToDo Add histogram support

use std::fmt::Debug;
use std::ops::{AddAssign, SubAssign};

/// A value that can only go up or be reset to 0, used for counts.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct Counter<T>(T);

/// A countable value that can go up and down that aggregates using the sum (e.g., items in a queue, bytes of memory).
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct UpDownCounter<T>(T);

/// A measurement value that aggregates using the average (e.g., temperature, physical dimensions, quotients).
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct Gauge<T>(T);

// Counter implementation.
// =======================

impl<T: Copy + Default> Counter<T> {
    /// Creates a new counter initialized with the provided value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Reset the counter to 0.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Returns the current value of the counter.
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }
}

impl Debug for Counter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter").field("value", &self.0).finish()
    }
}

impl From<u64> for Counter<u64> {
    fn from(value: u64) -> Self {
        Counter(value)
    }
}

impl AddAssign<u64> for Counter<u64> {
    fn add_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: Counter values are expected to be well-behaved in telemetry scenarios.
            // Wrapping behavior is acceptable for performance-critical metric collection.
            self.0 = self.0.wrapping_add(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += rhs;
        }
    }
}

impl Counter<u64> {
    /// Increments the counter by 1.
    #[inline]
    pub fn inc(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: Incrementing by 1 is safe for wrapping arithmetic in telemetry contexts
            self.0 = self.0.wrapping_add(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += 1;
        }
    }

    /// Adds an arbitrary value to the counter.
    #[inline]
    pub fn add(&mut self, v: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: Counter additions are expected to be well-behaved in telemetry scenarios
            self.0 = self.0.wrapping_add(v);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += v;
        }
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
    /// Creates a new up-down-counter initialized with the provided value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Reset the counter to 0.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Returns the current value of the counter.
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }

    /// Adds an arbitrary value to the up-down-counter.
    #[inline]
    pub fn add(&mut self, v: T) {
        self.0 += v;
    }

    /// Subs an arbitrary value to the up-down-counter.
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

impl From<u64> for UpDownCounter<u64> {
    fn from(value: u64) -> Self {
        UpDownCounter(value)
    }
}

impl AddAssign<u64> for UpDownCounter<u64> {
    fn add_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: UpDownCounter arithmetic is expected to be well-behaved in telemetry scenarios.
            // Wrapping behavior is acceptable for performance-critical metric operations.
            self.0 = self.0.wrapping_add(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += rhs;
        }
    }
}

impl SubAssign<u64> for UpDownCounter<u64> {
    fn sub_assign(&mut self, rhs: u64) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: UpDownCounter subtraction is expected to be well-behaved in telemetry scenarios.
            // Wrapping behavior is acceptable for performance-critical metric operations.
            self.0 = self.0.wrapping_sub(rhs);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 -= rhs;
        }
    }
}

impl UpDownCounter<u64> {
    /// Increments the up-down-counter by 1.
    #[inline]
    pub fn inc(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: Incrementing by 1 is safe for wrapping arithmetic in telemetry contexts
            self.0 = self.0.wrapping_add(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 += 1;
        }
    }

    /// Decrements the up-down-counter by 1.
    #[inline]
    pub fn dec(&mut self) {
        #[cfg(feature = "unchecked-arithmetic")]
        {
            // SAFETY: Decrementing by 1 is safe for wrapping arithmetic in telemetry contexts
            self.0 = self.0.wrapping_sub(1);
        }
        #[cfg(not(feature = "unchecked-arithmetic"))]
        {
            self.0 -= 1;
        }
    }
}

// Gauge implementation.
// =====================

impl<T> Gauge<T>
where
    T: Copy + Default + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
{
    /// Creates a new gauge initialized with the provided value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Reset the gauge to 0.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = T::default();
    }

    /// Sets the value of the gauge.
    #[inline]
    pub fn set(&mut self, v: T) {
        self.0 = v;
    }

    /// Returns the current value of the gauge.
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }
}

impl Debug for Gauge<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gauge").field("value", &self.0).finish()
    }
}

impl From<u64> for Gauge<u64> {
    fn from(value: u64) -> Self {
        Gauge(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_new() {
        let counter = Counter::new(42u64);
        assert_eq!(counter.get(), 42);
    }

    #[test]
    fn test_counter_default() {
        let counter: Counter<u64> = Counter::default();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_from() {
        let counter = Counter::from(100u64);
        assert_eq!(counter.get(), 100);
    }

    #[test]
    fn test_counter_reset() {
        let mut counter = Counter::new(42u64);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_counter_inc() {
        let mut counter = Counter::new(0u64);
        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.inc();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_counter_add() {
        let mut counter = Counter::new(10u64);
        counter.add(5);
        assert_eq!(counter.get(), 15);

        counter.add(0);
        assert_eq!(counter.get(), 15);

        counter.add(100);
        assert_eq!(counter.get(), 115);
    }

    #[test]
    fn test_counter_add_assign() {
        let mut counter = Counter::new(10u64);
        counter += 5;
        assert_eq!(counter.get(), 15);

        counter += 0;
        assert_eq!(counter.get(), 15);

        counter += 100;
        assert_eq!(counter.get(), 115);
    }

    #[test]
    fn test_counter_large_values() {
        let mut counter = Counter::new(u64::MAX - 10);
        counter.add(5);
        assert_eq!(counter.get(), u64::MAX - 5);
    }

    #[cfg(feature = "unchecked-arithmetic")]
    #[test]
    fn test_counter_overflow_wrapping_with_feature() {
        // When unchecked-arithmetic is enabled, operations should wrap
        let mut counter = Counter::new(u64::MAX);
        counter.inc(); // Should wrap to 0
        assert_eq!(counter.get(), 0);

        let mut counter2 = Counter::new(u64::MAX - 1);
        counter2.add(5); // Should wrap to 3
        assert_eq!(counter2.get(), 3);

        let mut counter3 = Counter::new(u64::MAX);
        counter3 += 10; // Should wrap to 9
        assert_eq!(counter3.get(), 9);
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_counter_overflow_panic_without_feature() {
        // When unchecked-arithmetic is disabled, operations should panic on overflow
        let mut counter = Counter::new(u64::MAX);
        counter.inc(); // Should panic
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_counter_add_overflow_panic_without_feature() {
        // When unchecked-arithmetic is disabled, operations should panic on overflow
        let mut counter = Counter::new(u64::MAX - 1);
        counter.add(5); // Should panic
    }

    #[cfg(not(feature = "unchecked-arithmetic"))]
    #[test]
    #[should_panic]
    fn test_counter_add_assign_overflow_panic_without_feature() {
        // When unchecked-arithmetic is disabled, operations should panic on overflow
        let mut counter = Counter::new(u64::MAX);
        counter += 10; // Should panic
    }

    #[test]
    fn test_counter_copy() {
        let counter1 = Counter::new(42u64);
        let counter2 = counter1; // Should copy, not move
        assert_eq!(counter1.get(), 42); // counter1 should still be usable
        assert_eq!(counter2.get(), 42);
    }

    #[test]
    fn test_counter_sequential_operations() {
        let mut counter = Counter::new(0u64);

        // Test a sequence of operations
        counter.inc(); // 1
        counter.add(10); // 11
        counter += 5; // 16
        counter.inc(); // 17
        counter.add(3); // 20

        assert_eq!(counter.get(), 20);
    }

    #[test]
    fn test_counter_edge_cases() {
        // Test with maximum safe values
        let mut counter = Counter::new(u64::MAX / 2);
        counter.add(100);
        assert_eq!(counter.get(), (u64::MAX / 2) + 100);

        // Test reset after operations
        counter.reset();
        assert_eq!(counter.get(), 0);

        // Test operations after reset
        counter.inc();
        assert_eq!(counter.get(), 1);
    }
}
