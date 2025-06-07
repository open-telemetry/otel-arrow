// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_engine::FactoryRegistry;
use otap_df_engine_macros::factory_registry;

/// gRPC service implementation
pub mod grpc;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;
/// Generated protobuf files
pub mod proto;

pub mod parquet_exporter;

/// testing utilities
#[cfg(test)]
mod mock;

// Factory registry for OTAP data with distributed slices
#[factory_registry(OTAPData)]
static FACTORY_REGISTRY: FactoryRegistry<OTAPData> = FactoryRegistry::new();


