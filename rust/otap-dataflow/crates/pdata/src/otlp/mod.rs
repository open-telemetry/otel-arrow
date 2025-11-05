// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains traits and utilities for OTLP (OpenTelemetry
//! Protocol) message types.

pub use otap_df_pdata_otlp_macros::Message; // Required for derived code
pub use otap_df_pdata_otlp_macros::qualified; // Required for derived code

use crate::proto::opentelemetry::common::v1::{AnyValue, ArrayValue, KeyValue, KeyValueList};

// TODO write documentation for this crate
//#![allow(missing_docs)]

pub mod attributes;
pub mod logs;
pub mod metrics;
pub mod traces;

mod common;
pub use common::ProtoBuffer;

use crate::{error::Result, otap::OtapArrowRecords};

#[cfg(test)]
mod tests;

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
