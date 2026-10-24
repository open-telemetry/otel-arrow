// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Structured AnyValue event fields using a downcast to `&(dyn
//! std::error::Error + 'static)`.
//!
//! A [`StructBox`] owns a [`StructuredValue`] and exposes it through
//! [`StructBox::as_struct`]; the direct field encoder
//! recognizes it and writes an AnyValue map/array instead of text.
//!
//! Callers, to avoid allocations near logging callsites, store a
//! `StructBox<T>` instead of a `Box<T>` and use `as_struct()` inside
//! logging statements.

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Maximum nesting of maps and arrays; deeper values are dropped.
pub const MAX_DEPTH: usize = 16;

/// A value that can encode itself as an OTLP AnyValue.
pub trait StructuredValue: Any {
    /// Writes exactly one value to `w`.
    fn write(&self, w: &mut dyn ValueWriter);
}

/// Receives one AnyValue. Writing nothing is an empty value.
pub trait ValueWriter {
    /// Writes a string, maybe truncated.
    fn write_str(&mut self, value: &str);
    /// Writes the `Debug` value as a string, maybe truncated.
    fn write_debug(&mut self, value: &dyn fmt::Debug);
    /// Writes an i64.
    fn write_i64(&mut self, value: i64);
    /// Writes an f64.
    fn write_f64(&mut self, value: f64);
    /// Writes a boolean.
    fn write_bool(&mut self, value: bool);
    /// Writes a map; `f` adds the entries.
    fn write_map(&mut self, f: &mut dyn FnMut(&mut dyn MapWriter));
    /// Writes an array; `f` adds the elements.
    fn write_array(&mut self, f: &mut dyn FnMut(&mut dyn ArrayWriter));
}

/// Receives the entries of a map.
pub trait MapWriter {
    /// Adds one entry; `f` writes its value.
    fn write_entry(&mut self, key: &str, f: &mut dyn FnMut(&mut dyn ValueWriter));
}

/// Receives the elements of an array.
pub trait ArrayWriter {
    /// Adds one element; `f` writes its value.
    fn write_element(&mut self, f: &mut dyn FnMut(&mut dyn ValueWriter));
}

impl dyn ValueWriter + '_ {
    /// Writes a map, with `f` adding the entries.
    pub fn map(&mut self, mut f: impl FnMut(&mut dyn MapWriter)) {
        self.write_map(&mut f);
    }

    /// Writes an array, with `f` adding the elements.
    pub fn array(&mut self, mut f: impl FnMut(&mut dyn ArrayWriter)) {
        self.write_array(&mut f);
    }
}

impl dyn MapWriter + '_ {
    /// Adds an entry, with `f` writing its value.
    pub fn entry(&mut self, key: &str, mut f: impl FnMut(&mut dyn ValueWriter)) {
        self.write_entry(key, &mut f);
    }
}

impl dyn ArrayWriter + '_ {
    /// Adds an element, with `f` writing its value.
    pub fn element(&mut self, mut f: impl FnMut(&mut dyn ValueWriter)) {
        self.write_element(&mut f);
    }
}

/// A typed, exclusively owned value that can be borrowed as a structured field.
///
/// Like [`Box`], dereferencing borrows `T` and mutable dereferencing provides
/// exclusive access. Logging borrows the stored carrier without allocating.
pub struct StructBox<T: StructuredValue> {
    carrier: StructCarrier,
    marker: PhantomData<T>,
}

impl<T: StructuredValue> StructBox<T> {
    /// Allocates exclusive storage for a value that will be logged repeatedly.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self::from(Box::new(value))
    }

    /// Borrows the stored carrier for a logging field.
    ///
    /// Pass directly as a field value, without `?` or `%`.
    #[must_use]
    pub fn as_struct(&self) -> &(dyn Error + 'static) {
        &self.carrier
    }
}

impl<T: StructuredValue> From<Box<T>> for StructBox<T> {
    fn from(value: Box<T>) -> Self {
        Self {
            carrier: StructCarrier(value),
            marker: PhantomData,
        }
    }
}

impl<T: StructuredValue> Deref for StructBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let value: &dyn Any = self.carrier.value();
        value
            .downcast_ref::<T>()
            .expect("StructBox carrier must contain T")
    }
}

impl<T: StructuredValue> DerefMut for StructBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let value: &mut dyn Any = self.carrier.0.as_mut();
        value
            .downcast_mut::<T>()
            .expect("StructBox carrier must contain T")
    }
}

impl<T: StructuredValue + fmt::Debug> fmt::Debug for StructBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

/// One downcast target for structured fields, regardless of the stored value type.
pub(super) struct StructCarrier(Box<dyn StructuredValue>);

impl StructCarrier {
    pub(super) fn value(&self) -> &dyn StructuredValue {
        self.0.as_ref()
    }
}

// Subscribers that do not understand structured fields see a placeholder.
impl fmt::Debug for StructCarrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<structured>")
    }
}

impl fmt::Display for StructCarrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for StructCarrier {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct State {
        count: i64,
    }

    impl StructuredValue for State {
        fn write(&self, w: &mut dyn ValueWriter) {
            w.write_i64(self.count);
        }
    }

    /// Scenario: non-Clone values are stored in new and converted structured boxes.
    /// Guarantees: handles add no storage overhead, conversion reuses the allocation, and typed mutation and field borrowing work.
    #[test]
    fn boxed_storage_preserves_ownership_and_typed_access() {
        assert_eq!(
            size_of::<StructBox<State>>(),
            size_of::<Box<dyn StructuredValue>>(),
        );
        let original = Box::new(State { count: 7 });
        let address = std::ptr::from_ref(&*original);
        let converted = StructBox::from(original);
        assert_eq!(std::ptr::from_ref(&*converted), address);
        assert_eq!(converted.count, 7);
        for mut state in [StructBox::new(State { count: 1 }), converted] {
            state.count = 2;
            assert_eq!(state.count, 2);
            assert!(std::ptr::eq(
                state
                    .as_struct()
                    .downcast_ref::<StructCarrier>()
                    .expect("known carrier"),
                &state.carrier,
            ));
            state.count = 3;
            assert_eq!(state.count, 3);
        }
    }
}
