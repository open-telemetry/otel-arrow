// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains traits and utilities for OTLP (OpenTelemetry
//! Protocol) message types.
///
/// - LogsVisitor<_>: prototype for top-level visitor interface, this could
///   support both Export requests and the assocaited Data.
/// - Counting items via the visitor pattern demonstration (ItemCounter)
///   note that this is not necessarily a good application of the pattern.
/// - PrecomputedSizes: used in first-pass of OTLP protobuf encoding visitor
/// - Into<_> for KeyValueList, ArrayValue.
pub use otlp_derive::Message; // Required for derived code
pub use otlp_derive::qualified; // Required for derived code

// Primitive encoders for the first pass of two-pass encoding
pub mod primitive_encoders;
pub use primitive_encoders::{
    Accumulate, BooleanEncodedLen, BytesEncodedLen, DoubleEncodedLen, Fixed32EncodedLen,
    Fixed64EncodedLen, I32EncodedLen, I64EncodedLen, Sfixed64EncodedLen, Sint32EncodedLen,
    SliceBooleanEncodedLen, SliceBytesEncodedLen, SliceDoubleEncodedLen, SliceFixed32EncodedLen,
    SliceFixed64EncodedLen, SliceI32EncodedLen, SliceI64EncodedLen, SliceStringEncodedLen,
    SliceU32EncodedLen, SliceU64EncodedLen, StringEncodedLen, U32EncodedLen, U64EncodedLen,
};

use crate::proto::opentelemetry::logs::v1::LogRecordVisitable;
use crate::proto::opentelemetry::logs::v1::LogRecordVisitor;
use crate::proto::opentelemetry::logs::v1::LogsDataVisitable;
use crate::proto::opentelemetry::logs::v1::LogsDataVisitor;
use crate::proto::opentelemetry::logs::v1::ResourceLogsVisitable;
use crate::proto::opentelemetry::logs::v1::ResourceLogsVisitor;
use crate::proto::opentelemetry::logs::v1::ScopeLogsVisitable;
use crate::proto::opentelemetry::logs::v1::ScopeLogsVisitor;

use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue, KeyValueList};

#[cfg(test)]
mod tests;

/// LogsVisitor is the entry point for visiting OTLP logs data.
pub trait LogsVisitor<Argument> {
    /// The return type of the visitor
    type Return;

    /// Visit logs data and return the computed result
    fn visit_logs(self, v: impl LogsDataVisitable<Argument>) -> Self::Return;
}

/// ItemCounter implements counting log records. This sort of item
/// counting is a built-in feature of the Golang Pdata API.
pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    /// Create a new ItemCounter starting at 0
    pub fn new() -> Self {
        Self { count: 0 }
    }

    #[allow(dead_code)] // Will be used when full adapter pattern is implemented
    fn borrow_mut<'a>(&'a mut self) -> &'a mut Self {
        self
    }
}

impl LogsVisitor<()> for ItemCounter {
    /// The return type of the visitor
    type Return = usize;

    /// Visit logs data and return the computed result
    fn visit_logs(mut self, mut v: impl LogsDataVisitable<()>) -> Self::Return {
        v.accept_logs_data((), &mut self);
        self.count
    }
}

impl<Argument> LogsDataVisitor<Argument> for ItemCounter {
    fn visit_logs_data(
        &mut self,
        arg: Argument,
        mut v: impl LogsDataVisitable<Argument>,
    ) -> Argument {
        v.accept_logs_data(arg, self.borrow_mut())
    }
}

impl<Argument> ResourceLogsVisitor<Argument> for &mut ItemCounter {
    fn visit_resource_logs(
        &mut self,
        arg: Argument,
        mut v: impl ResourceLogsVisitable<Argument>,
    ) -> Argument {
        v.accept_resource_logs(
            arg,
            super::NoopVisitor {},
            self.borrow_mut(),
            super::NoopVisitor {},
        )
    }
}

impl<Argument> ScopeLogsVisitor<Argument> for &mut ItemCounter {
    fn visit_scope_logs(
        &mut self,
        arg: Argument,
        mut sv: impl ScopeLogsVisitable<Argument>,
    ) -> Argument {
        sv.accept_scope_logs(
            arg,
            super::NoopVisitor {},
            self.borrow_mut(),
            super::NoopVisitor {},
        )
    }
}

impl<Argument> LogRecordVisitor<Argument> for &mut ItemCounter {
    fn visit_log_record(
        &mut self,
        arg: Argument,
        _: impl LogRecordVisitable<Argument>,
    ) -> Argument {
        self.count += 1;
        arg
    }
}

/// PrecomputeSizes is an argument to the PrecomputedSize visitor
pub struct PrecomputedSizes {
    sizes: Vec<usize>,
}

impl PrecomputedSizes {
    /// Create a new PrecomputedSizes with initial capacity
    pub fn default() -> Self {
        Self { sizes: Vec::new() }
    }

    /// Calculate the length in bytes needed to encode a varint
    pub fn varint_len(value: usize) -> usize {
        // TODO: use a Prost helper, otherwise this has duplication
        // with primitive_encoders.rs.
        if value == 0 {
            1
        } else {
            ((64 - value.leading_zeros()) as usize + 6) / 7
        }
    }

    /// Get the size at a specific index (for reading child sizes)
    pub fn get_size(&self, idx: usize) -> usize {
        self.sizes[idx]
    }

    /// Get the current length (for tracking child positions)
    pub fn len(&self) -> usize {
        self.sizes.len()
    }

    /// Push a size value (used for reserving space)
    pub fn reserve(&mut self) {
        self.sizes.push(0);
    }

    /// Push a size value (used for reserving space)
    pub fn push_size(&mut self, value: usize) {
        self.sizes.push(value);
    }

    /// Gets the last size.
    pub fn last(&self) -> usize {
        *self.sizes.last().expect("has a size")
    }

    /// Update a previously reserved space with the calculated size
    pub fn set_size(&mut self, idx: usize, value: usize) {
        self.sizes[idx] = value;
    }
}

// Into implementations for OTLP common types to support builder APIs

/// Convert Vec<AnyValue> into ArrayValue for builder APIs
impl Into<ArrayValue> for Vec<AnyValue> {
    fn into(self) -> ArrayValue {
        ArrayValue { values: self }
    }
}

/// Convert Vec<KeyValue> into KeyValueList for builder APIs
impl Into<KeyValueList> for Vec<KeyValue> {
    fn into(self) -> KeyValueList {
        KeyValueList { values: self }
    }
}
