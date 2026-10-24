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

pub use super::encoder::{TruncateResult, Truncation};

/// Maximum nesting of maps and arrays; deeper values are dropped.
pub const MAX_DEPTH: usize = 16;

/// A value that can encode itself as an OTLP AnyValue.
pub trait StructuredValue: Any {
    /// Encodes exactly one value, returning its truncation or failure.
    fn encode(&self, encoder: &mut dyn ValueWriter) -> TruncateResult;
}

/// Receives one AnyValue. Returning `Ok(Truncation::None)` without writing
/// produces an empty value.
///
/// This object-safe interface lets type-erased structured values use the same
/// encoder with either a heap-backed or stack-backed buffer.
pub trait ValueWriter {
    /// Encodes a string, maybe truncated.
    fn encode_str(&mut self, value: &str) -> TruncateResult;
    /// Encodes the `Debug` value as a string, maybe truncated.
    fn encode_debug(&mut self, value: &dyn fmt::Debug) -> TruncateResult;
    /// Encodes an i64.
    fn encode_i64(&mut self, value: i64) -> TruncateResult;
    /// Encodes an f64.
    fn encode_f64(&mut self, value: f64) -> TruncateResult;
    /// Encodes a boolean.
    fn encode_bool(&mut self, value: bool) -> TruncateResult;
    /// Encodes a map; `f` propagates member failures with `?` and combines
    /// successful truncation results. A member failure retains the map's prefix.
    fn encode_map(
        &mut self,
        f: &mut dyn FnMut(&mut dyn MapWriter) -> TruncateResult,
    ) -> TruncateResult;
    /// Encodes an array with the same prefix-retention policy as `encode_map`.
    fn encode_array(
        &mut self,
        f: &mut dyn FnMut(&mut dyn ArrayWriter) -> TruncateResult,
    ) -> TruncateResult;
}

/// Receives the entries of a map.
pub trait MapWriter {
    /// Adds one entry; failure rolls back the entry and must be propagated.
    fn encode_entry(
        &mut self,
        key: &str,
        f: &mut dyn FnMut(&mut dyn ValueWriter) -> TruncateResult,
    ) -> TruncateResult;
}

/// Receives the elements of an array.
pub trait ArrayWriter {
    /// Adds one element; failure rolls back the element and must be propagated.
    fn encode_element(
        &mut self,
        f: &mut dyn FnMut(&mut dyn ValueWriter) -> TruncateResult,
    ) -> TruncateResult;
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
        fn encode(&self, encoder: &mut dyn ValueWriter) -> TruncateResult {
            encoder.encode_i64(self.count)
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
