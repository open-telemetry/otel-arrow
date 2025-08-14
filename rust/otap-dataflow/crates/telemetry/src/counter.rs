// SPDX-License-Identifier: Apache-2.0

//! Primitive metric value types (counters, more in the future).
//! Fast-path: plain additions on core-local memory.
//! ToDo Gauges, Histograms, etc.

use std::fmt::Debug;
use std::ops::AddAssign;

#[repr(transparent)]
/// Single-producer counter storing a numeric value.
#[derive(Default, Clone, Copy)]
pub struct Counter<T>(T);

impl<T: Copy + Default> Counter<T> {
    /// Creates a new counter initialized with the provided value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(v)
    }

    /// Sets the counter to an absolute value.
    #[inline]
    pub fn set(&mut self, v: T) {
        self.0 = v;
    }

    /// Returns the current value of the counter.
    #[inline]
    pub fn get(&self) -> T {
        self.0
    }
}

impl Debug for Counter<u64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Counter")
            .field("value", &self.0)
            .finish()
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
