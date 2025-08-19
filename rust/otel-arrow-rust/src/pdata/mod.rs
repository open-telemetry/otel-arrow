// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains data structures for OTLP and OTAP pipeline data.
///
/// - Visitors for primitive data types (string, i64, ...)
/// - SliceVisitor<_, _>
/// - NoopVisitor
/// - From<_> for TraceID, SpanID
pub mod otlp;

use crate::error;

// Concerning TraceID and SpanID:
//
// Note that these types are placeholders, we probably want to share
// these definitions as well as the Prost/Tonic generation with the
// OTel-Rust SDK where they are already defined. To avoid coordinating
// these repositories in the short term, we provide definitions for
// TraceID and SpanID.
//
// In particular, the OTel specification has careful words about how
// to format and parse these two fields, which are non-standard with
// respect to JSON, and the OTel-Rust SDK implements this aspect of
// the spec.

/// TraceID identifier of a Trace
#[derive(Eq, PartialEq, Clone, Copy, Debug, Default)]
pub struct TraceID([u8; 16]);

impl TraceID {
    /// creates a new instance of the TraceID by copying the bytes
    #[must_use]
    pub fn new(value: &[u8; 16]) -> TraceID {
        TraceID(*value)
    }
}

impl From<[u8; 16]> for TraceID {
    fn from(tid: [u8; 16]) -> Self {
        TraceID(tid)
    }
}

impl TryFrom<&[u8]> for TraceID {
    type Error = error::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let id_bytes: [u8; 16] = value.try_into().map_err(|_| {
            error::InvalidIdSnafu {
                expected: 16usize,
                given: value.len(),
            }
            .build()
        })?;
        Ok(TraceID::from(id_bytes))
    }
}

impl From<TraceID> for Vec<u8> {
    fn from(tid: TraceID) -> Self {
        tid.0.to_vec()
    }
}

/// SpanID identifier of a Span
#[derive(Clone, Copy, Debug, Default)]
pub struct SpanID([u8; 8]);

impl SpanID {
    /// creates a new instance of the SpanID by copying the bytes
    #[must_use]
    pub fn new(value: &[u8; 8]) -> SpanID {
        SpanID(*value)
    }
}

impl From<SpanID> for Vec<u8> {
    fn from(sid: SpanID) -> Self {
        sid.0.to_vec()
    }
}

impl From<[u8; 8]> for SpanID {
    fn from(sid: [u8; 8]) -> Self {
        SpanID(sid)
    }
}

/// StringVisitor
pub trait StringVisitor<Argument> {
    /// Visit a string value
    fn visit_string(&mut self, arg: Argument, value: &str) -> Argument;
}

/// IntegerVisitor
pub trait I32Visitor<Argument> {
    /// Visit an integer value
    fn visit_i32(&mut self, arg: Argument, value: i32) -> Argument;
}

/// IntegerVisitor
pub trait I64Visitor<Argument> {
    /// Visit an integer value
    fn visit_i64(&mut self, arg: Argument, value: i64) -> Argument;
}

/// IntegerVisitor
pub trait U32Visitor<Argument> {
    /// Visit an integer value
    fn visit_u32(&mut self, arg: Argument, value: u32) -> Argument;
}

/// IntegerVisitor
pub trait U64Visitor<Argument> {
    /// Visit an integer value
    fn visit_u64(&mut self, arg: Argument, value: u64) -> Argument;
}

/// FloatVisitor
pub trait F64Visitor<Argument> {
    /// Visit a float value
    fn visit_f64(&mut self, arg: Argument, value: f64) -> Argument;
}

/// BooleanVisitor
pub trait BooleanVisitor<Argument> {
    /// Visit a boolean value
    fn visit_bool(&mut self, arg: Argument, value: bool) -> Argument;
}

/// BytesVisitor
pub trait BytesVisitor<Argument> {
    /// Visit a bytes value
    fn visit_bytes(&mut self, arg: Argument, value: &[u8]) -> Argument;
}

/// Visitor trait for primitive slices.
pub trait SliceVisitor<Argument, Primitive> {
    /// Visit a slice of primitives
    fn visit_slice(&mut self, arg: Argument, value: &[Primitive]) -> Argument;
}

/// NoopVisitor implements every visitor, does nothing.
pub struct NoopVisitor {}

impl<Argument> BytesVisitor<Argument> for NoopVisitor {
    fn visit_bytes(&mut self, arg: Argument, _: &[u8]) -> Argument {
        arg
    }
}

impl<Argument> StringVisitor<Argument> for NoopVisitor {
    fn visit_string(&mut self, arg: Argument, _: &str) -> Argument {
        arg
    }
}

impl<Argument> I32Visitor<Argument> for NoopVisitor {
    fn visit_i32(&mut self, arg: Argument, _: i32) -> Argument {
        arg
    }
}

impl<Argument> I64Visitor<Argument> for NoopVisitor {
    fn visit_i64(&mut self, arg: Argument, _: i64) -> Argument {
        arg
    }
}

impl<Argument> U32Visitor<Argument> for NoopVisitor {
    fn visit_u32(&mut self, arg: Argument, _: u32) -> Argument {
        arg
    }
}

impl<Argument> U64Visitor<Argument> for NoopVisitor {
    fn visit_u64(&mut self, arg: Argument, _: u64) -> Argument {
        arg
    }
}

impl<Argument> F64Visitor<Argument> for NoopVisitor {
    fn visit_f64(&mut self, arg: Argument, _: f64) -> Argument {
        arg
    }
}

impl<Argument> BooleanVisitor<Argument> for NoopVisitor {
    fn visit_bool(&mut self, arg: Argument, _: bool) -> Argument {
        arg
    }
}

impl<Argument, Primitive> SliceVisitor<Argument, Primitive> for NoopVisitor {
    fn visit_slice(&mut self, arg: Argument, _: &[Primitive]) -> Argument {
        arg
    }
}
