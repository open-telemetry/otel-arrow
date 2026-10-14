// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Structured (OTLP AnyValue) event fields.
//!
//! Tokio tracing has no stable way to carry a structured value through
//! `field = value`. This module uses the one channel that can be
//! downcast: `&(dyn std::error::Error + 'static)`. A [`Structured`]
//! wraps any [`StructuredValue`] and exposes itself through
//! [`Structured::as_value`]; the direct field encoder recognizes it and
//! writes an AnyValue map/array instead of text.
//!
//! ```ignore
//! struct Point { x: i64, y: i64 }
//!
//! impl StructuredValue for Point {
//!     fn write(&self, w: &mut dyn ValueWriter) {
//!         w.map(|m| {
//!             m.entry("x", |v| v.write_i64(self.x));
//!             m.entry("y", |v| v.write_i64(self.y));
//!         });
//!     }
//! }
//!
//! let point = Arc::new(Point { x: 1, y: 2 });
//! otel_info!("point.moved", to = Structured::from_arc(point.clone()).as_value());
//! ```
//!
//! Callers own the `Arc`; there is no constructor that allocates.
//!
//! Values are encoded under the same size limits as other event fields:
//! nesting is limited to [`MAX_DEPTH`] levels, and anything that does not
//! fit is dropped or truncated and counted once in the record's
//! `dropped_attributes_count`.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Maximum nesting of maps and arrays; deeper values are dropped.
pub const MAX_DEPTH: usize = 16;

/// A value that can describe itself as an OTLP AnyValue.
pub trait StructuredValue: Send + Sync + 'static {
    /// Writes exactly one value to `w`.
    fn write(&self, w: &mut dyn ValueWriter);
}

/// Receives one AnyValue. Writing nothing is an empty value.
pub trait ValueWriter {
    /// Writes a string, truncated if it does not fit.
    fn write_str(&mut self, value: &str);
    /// Writes the `Debug` rendering of `value` as a string, formatted in
    /// place and truncated if it does not fit.
    fn write_debug(&mut self, value: &dyn fmt::Debug);
    /// Writes a signed integer.
    fn write_i64(&mut self, value: i64);
    /// Writes a floating point number.
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
    /// Adds one entry; `f` writes its value. An entry that does not fit
    /// is dropped whole.
    fn write_entry(&mut self, key: &str, f: &mut dyn FnMut(&mut dyn ValueWriter));
}

/// Receives the elements of an array.
pub trait ArrayWriter {
    /// Adds one element; `f` writes its value. Once an element does not
    /// fit, it and all later elements are dropped.
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

/// Carrier that passes a [`StructuredValue`] through a tracing field.
#[derive(Clone)]
pub struct Structured(Arc<dyn StructuredValue>);

impl Structured {
    /// Wraps a shared value. The caller owns an `Arc` and clones it per
    /// event, so emitting costs a reference-count bump. Once the event
    /// completes the clone is dropped, so a sole owner can update the
    /// value in place with [`Arc::get_mut`].
    pub fn from_arc(value: Arc<impl StructuredValue>) -> Self {
        Self(value)
    }

    /// The field value to pass to an event macro.
    #[must_use]
    pub fn as_value(&self) -> &(dyn Error + 'static) {
        self
    }

    pub(super) fn value(&self) -> &dyn StructuredValue {
        &*self.0
    }
}

// Subscribers that do not understand structured fields see a placeholder.
impl fmt::Debug for Structured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<structured>")
    }
}

impl fmt::Display for Structured {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<structured>")
    }
}

impl Error for Structured {}
