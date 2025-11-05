// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains data structures for OTLP and OTAP pipeline data.
///
/// - Visitors for primitive data types (string, i64, ...)
/// - SliceVisitor<_, _>
/// - NoopVisitor
/// - From<_> for TraceID, SpanID
pub mod otlp;

use crate::error::{Error, Result};

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
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<TraceID> {
        let id_bytes: [u8; 16] = value.try_into().map_err(|_| Error::InvalidId {
            expected: 16usize,
            given: value.len(),
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
