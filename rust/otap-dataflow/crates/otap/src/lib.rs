// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use linkme::distributed_slice;
use otap_df_engine::{ExporterFactory, ProcessorFactory, ReceiverFactory};

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

/// A slice of receiver factories for OTAP data.
#[distributed_slice]
pub static RECEIVER_FACTORIES: [ReceiverFactory<OTAPData>] = [..];

/// A slice of local processor factories for OTAP data.
#[distributed_slice]
pub static PROCESSOR_FACTORIES: [ProcessorFactory<OTAPData>] = [..];

/// A slice of local exporter factories for OTAP data.
#[distributed_slice]
pub static EXPORTER_FACTORIES: [ExporterFactory<OTAPData>] = [..];
