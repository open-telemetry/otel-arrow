// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO write documentation for this crate
#![allow(missing_docs)]

pub mod attributes;
pub mod logs;
pub mod metrics;
pub mod traces;

mod common;
pub use common::ProtoBuffer;

use crate::{error::Result, otap::OtapArrowRecords};

/// Trait for types that can convert OTAP arrow records into the OTLP proto bytes representation
pub trait ProtoBytesEncoder {
    /// Converts the OTAP arrow records into OTLP proto bytes representation
    fn encode(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
        proto_buffer: &mut ProtoBuffer,
    ) -> Result<()>;
}
