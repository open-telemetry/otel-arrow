// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_engine::create_factory_registry;

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

// Create the factory registry with distributed slices for OTAP data
create_factory_registry!(OTAPData, OtapFactoryRegistry);


