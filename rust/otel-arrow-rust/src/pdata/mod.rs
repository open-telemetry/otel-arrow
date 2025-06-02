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

//! This module contains data structures for OTLP and OTAP pipeline data

pub mod otlp;

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
#[derive(Clone, Copy, Debug)]
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

impl From<TraceID> for Vec<u8> {
    fn from(tid: TraceID) -> Self {
        tid.0.to_vec()
    }
}

/// SpanID identifier of a Span
#[derive(Clone, Copy, Debug)]
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
pub trait StringVisitor {
    /// Visit a string value
    fn visit_string(&mut self, value: &str);
}

/// IntegerVisitor
pub trait I32Visitor {
    /// Visit an integer value
    fn visit_i32(&mut self, value: i32);
}

/// IntegerVisitor
pub trait I64Visitor {
    /// Visit an integer value
    fn visit_i64(&mut self, value: i64);
}

/// IntegerVisitor
pub trait U32Visitor {
    /// Visit an integer value
    fn visit_u32(&mut self, value: u32);
}

/// IntegerVisitor
pub trait U64Visitor {
    /// Visit an integer value
    fn visit_u64(&mut self, value: u64);
}

/// FloatVisitor
pub trait F64Visitor {
    /// Visit a float value
    fn visit_f64(&mut self, value: f64);
}

/// BooleanVisitor
pub trait BooleanVisitor {
    /// Visit a boolean value
    fn visit_bool(&mut self, value: bool);
}

/// BytesVisitor
pub trait BytesVisitor {
    /// Visit a bytes value
    fn visit_bytes(&mut self, value: &[u8]);
}
