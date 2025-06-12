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
pub mod encoders;
pub use encoders::{
    // The encoders are exposed by this crate.
    Accumulate,
    BooleanEncodedLen,
    BytesEncodedLen,
    DoubleEncodedLen,
    I32FixedEncodedLen,
    I32VarintEncodedLen,
    I64FixedEncodedLen,
    I64VarintEncodedLen,
    SliceBooleanEncodedLen,
    SliceBytesEncodedLen,
    SliceDoubleEncodedLen,
    SliceI32FixedEncodedLen,
    SliceI32VarintEncodedLen,
    SliceI64FixedEncodedLen,
    SliceI64VarintEncodedLen,
    SliceStringEncodedLen,
    SliceU32FixedEncodedLen,
    SliceU32VarintEncodedLen,
    SliceU64FixedEncodedLen,
    SliceU64VarintEncodedLen,
    StringEncodedLen,
    U32FixedEncodedLen,
    U32VarintEncodedLen,
    U64FixedEncodedLen,
    U64VarintEncodedLen,
};

use crate::proto::opentelemetry::logs::v1::{
    LogRecordVisitable, LogRecordVisitor, LogsDataVisitable, LogsDataVisitor,
    ResourceLogsVisitable, ResourceLogsVisitor, ScopeLogsVisitable, ScopeLogsVisitor,
};

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
#[derive(Default)]
pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    #[allow(dead_code)] // Will be used when full adapter pattern is implemented
    fn borrow_mut(&mut self) -> &mut Self {
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
#[derive(Default)]
pub struct PrecomputedSizes {
    sizes: Vec<usize>,
}

impl PrecomputedSizes {
    /// Reset to the empty state.
    pub fn clear(&mut self) {
        self.sizes.clear();
    }

    /// Get the size at a specific index (for reading child sizes)
    #[must_use]
    pub fn get_size(&self, idx: usize) -> usize {
        self.sizes[idx]
    }

    /// Get the current length for tracking child positions
    #[must_use]
    pub fn position(&self) -> usize {
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
    #[must_use]
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
#[allow(clippy::from_over_into)]
impl Into<ArrayValue> for Vec<AnyValue> {
    fn into(self) -> ArrayValue {
        ArrayValue { values: self }
    }
}

/// Convert Vec<KeyValue> into KeyValueList for builder APIs
#[allow(clippy::from_over_into)]
impl Into<KeyValueList> for Vec<KeyValue> {
    fn into(self) -> KeyValueList {
        KeyValueList { values: self }
    }
}
