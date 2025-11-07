// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! PData support modules.

/// Module contains traits and utilities for OTLP (OpenTelemetry
/// Protocol) message types.
pub mod otlp;

/// Module contains the underlying OTLP and OTAP protobuf objects.
pub mod proto;

/// Module contains the views for OTAP <-> OTLP objects.
pub mod views;

pub mod error;
pub mod otap;
pub mod schema;

pub(crate) mod arrays;
pub(crate) mod decode;
pub mod encode;
pub(crate) mod payload;

pub use otap::OtapArrowRecords;
pub use otlp::OtlpProtoBytes;
pub use payload::{OtapPayload, OtapPayloadHelpers};

/// Testing support
// #[cfg(test)] ?
pub mod testing;

#[cfg(test)]
mod validation;

pub use decode::decoder::Consumer;
pub use encode::producer::Producer;

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

impl From<TraceID> for Vec<u8> {
    fn from(tid: TraceID) -> Self {
        tid.0.to_vec()
    }
}

impl TryFrom<&[u8]> for TraceID {
    type Error = std::array::TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let id_bytes: [u8; 16] = value.try_into()?;
        Ok(TraceID::from(id_bytes))
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

impl From<[u8; 8]> for SpanID {
    fn from(sid: [u8; 8]) -> Self {
        SpanID(sid)
    }
}

impl From<SpanID> for Vec<u8> {
    fn from(sid: SpanID) -> Self {
        sid.0.to_vec()
    }
}

impl TryFrom<&[u8]> for SpanID {
    type Error = std::array::TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let id_bytes: [u8; 8] = value.try_into()?;
        Ok(SpanID::from(id_bytes))
    }
}
