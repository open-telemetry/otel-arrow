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

/// A value that can go up and down, used for sizes or amount of items in a queue.
#[repr(transparent)]
#[derive(Default, Clone, Copy)]
pub struct UpDownCounter<T>(T);

/// A value that can arbitrarily go up and down, used for temperature or current memory usage.
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
        self.0 = self.0 + rhs;
    }
}

impl Counter<u64> {
    /// Increments the counter by 1.
    #[inline]
    pub fn inc(&mut self) {
        self.0 = self.0 + 1;
    }

    /// Adds an arbitrary value to the counter.
    #[inline]
    pub fn add(&mut self, v: u64) {
        self.0 = self.0 + v;
    }
}

// UpDownCounter implementation.
// =============================

impl<T> UpDownCounter<T>
where
    T: Copy + Default + std::ops::Add<Output = T> + std::ops::Sub<Output = T>,
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
        self.0 = self.0 + v;
    }

    /// Subs an arbitrary value to the up-down-counter.
    #[inline]
    pub fn sub(&mut self, v: T) {
        self.0 = self.0 - v;
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
        self.0 = self.0 + rhs;
    }
}

impl SubAssign<u64> for UpDownCounter<u64> {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0 - rhs;
    }
}

impl UpDownCounter<u64> {
    /// Increments the up-down-counter by 1.
    #[inline]
    pub fn inc(&mut self) {
        self.0 = self.0 + 1;
    }

    /// Decrements the up-down-counter by 1.
    #[inline]
    pub fn dec(&mut self) {
        self.0 = self.0 - 1;
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

    /// Adds an arbitrary value to the gauge.
    #[inline]
    pub fn add(&mut self, v: T) {
        self.0 = self.0 + v;
    }

    /// Subs an arbitrary value to the gauge.
    #[inline]
    pub fn sub(&mut self, v: T) {
        self.0 = self.0 - v;
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

impl AddAssign<u64> for Gauge<u64> {
    fn add_assign(&mut self, rhs: u64) {
        self.0 = self.0 + rhs;
    }
}

impl SubAssign<u64> for Gauge<u64> {
    fn sub_assign(&mut self, rhs: u64) {
        self.0 = self.0 - rhs;
    }
}

impl Gauge<u64> {
    /// Increments the gauge by 1.
    #[inline]
    pub fn inc(&mut self) {
        self.0 = self.0 + 1;
    }

    /// Decrements the gauge by 1.
    #[inline]
    pub fn dec(&mut self) {
        self.0 = self.0 - 1;
    }
}
