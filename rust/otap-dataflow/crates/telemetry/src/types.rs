// SPDX-License-Identifier: Apache-2.0

//! Primitive metric value types (counters, more in the future).
//! Fast-path: plain additions on core-local memory.
//! ToDo Gauges, Histograms, etc.

use core::cell::Cell;

#[repr(transparent)]
/// Single-producer counter storing a numeric value with interior mutability.
pub struct Counter<T>(Cell<T>);

impl<T: Copy + Default> Counter<T> {
    /// Creates a new counter initialized with the provided value.
    #[inline]
    pub const fn new(v: T) -> Self {
        Self(Cell::new(v))
    }

    /// Sets the counter to an absolute value.
    #[inline]
    pub fn set(&self, v: T) {
        self.0.set(v);
    }

    /// Returns the current value of the counter.
    #[inline]
    pub fn get(&self) -> T {
        self.0.get()
    }
}

impl Counter<u64> {
    /// Increments the counter by 1.
    #[inline]
    pub fn inc(&mut self) {
        self.0.set(self.0.get() + 1);
    }

    /// Adds an arbitrary value to the counter.
    #[inline]
    pub fn add(&mut self, v: u64) {
        self.0.set(self.0.get() + v);
    }
}
