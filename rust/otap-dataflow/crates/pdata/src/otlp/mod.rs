// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains traits and utilities for OTLP (OpenTelemetry
//! Protocol) message types.

use bytes::Bytes;
pub use otap_df_pdata_otlp_macros::Message; // Required for derived code
pub use otap_df_pdata_otlp_macros::qualified; // Required for derived code

use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue, KeyValueList};

/// Common methods for OTLP/OTAP attributes.
pub mod attributes;
/// Common methods for OTLP/OTAP logs.
pub mod logs;
/// Common methods for OTLP/OTAP metrics.
pub mod metrics;
/// Common methods for OTLP/OTAP traces.
pub mod traces;

mod common;
pub use common::ProtoBuffer;

use crate::{error::Result, otap::OtapArrowRecords};

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
    /// Get a borrowed reference to the serialized proto bytes slice
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            OtlpProtoBytes::ExportLogsRequest(bytes)
            | OtlpProtoBytes::ExportMetricsRequest(bytes)
            | OtlpProtoBytes::ExportTracesRequest(bytes) => bytes.as_ref(),
        }
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
