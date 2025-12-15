// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains traits and utilities for OTLP (OpenTelemetry
//! Protocol) message types.

use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue, KeyValueList};
use crate::{error::Result, otap::OtapArrowRecords};
use bytes::Bytes;
use otap_df_config::SignalType;

pub use common::ProtoBuffer;
pub use otap_df_pdata_otlp_macros::Message; // Required for derived code
pub use otap_df_pdata_otlp_macros::qualified; // Required for derived code

/// Common methods for OTLP/OTAP attributes.
pub mod attributes;
/// Common methods for batching.
pub mod batching;
/// Common methods for OTLP/OTAP logs.
pub mod logs;
/// Common methods for OTLP/OTAP metrics.
pub mod metrics;
/// Common methods for OTLP/OTAP traces.
pub mod traces;

mod common;
#[cfg(test)]
mod tests;

/// Pipeline data represented as protobuf serialized OTLP request messages
#[derive(Clone, Debug)]
pub enum OtlpProtoBytes {
    /// protobuf serialized ExportLogsServiceRequest
    ExportLogsRequest(Bytes),
    /// protobuf serialized ExportMetricsServiceRequest
    ExportMetricsRequest(Bytes),
    /// protobuf serialized ExportTracesServiceRequest
    ExportTracesRequest(Bytes),
}

impl OtlpProtoBytes {
    /// Constructs a new message from bytes and signal type.
    #[must_use]
    pub fn new_from_bytes<B>(signal: SignalType, b: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        let bytes: Bytes = b.into().into();
        match signal {
            SignalType::Logs => Self::ExportLogsRequest(bytes),
            SignalType::Metrics => Self::ExportMetricsRequest(bytes),
            SignalType::Traces => Self::ExportTracesRequest(bytes),
        }
    }

    /// Create a new empty request object of a certain signal type.
    #[must_use]
    pub fn empty(signal: SignalType) -> Self {
        let b = Bytes::new();
        match signal {
            SignalType::Logs => Self::ExportLogsRequest(b),
            SignalType::Metrics => Self::ExportMetricsRequest(b),
            SignalType::Traces => Self::ExportTracesRequest(b),
        }
    }

    /// Get a borrowed reference to the serialized proto bytes slice
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            OtlpProtoBytes::ExportLogsRequest(bytes)
            | OtlpProtoBytes::ExportMetricsRequest(bytes)
            | OtlpProtoBytes::ExportTracesRequest(bytes) => bytes.as_ref(),
        }
    }

    /// Constructs a new message from bytes and signal type.
    #[must_use]
    pub fn byte_size(&self) -> usize {
        self.as_bytes().len()
    }
}

/// Trait for types that can convert OTAP arrow records into the OTLP proto bytes representation
pub trait ProtoBytesEncoder {
    /// Converts the OTAP arrow records into OTLP proto bytes representation
    fn encode(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
        proto_buffer: &mut ProtoBuffer,
    ) -> Result<()>;
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
