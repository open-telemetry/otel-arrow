// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use linkme::distributed_slice;
use otap_df_engine::local::{LocalExporterFactory, LocalProcessorFactory, LocalReceiverFactory};
use otap_df_engine::shared::{
    SharedExporterFactory, SharedProcessorFactory, SharedReceiverFactory,
};

/// Code for encoding OTAP batch from pdata view
pub mod encoder;
/// gRPC service implementation
pub mod grpc;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;
/// Generated protobuf files
pub mod proto;

pub mod parquet_exporter;

pub mod perf_exporter;

/// testing utilities
#[cfg(test)]
mod mock;

/// A slice of local receiver factories for OTAP data.
#[distributed_slice]
pub static LOCAL_RECEIVERS: [LocalReceiverFactory<OTAPData>] = [..];

/// A slice of local processor factories for OTAP data.
#[distributed_slice]
pub static LOCAL_PROCESSORS: [LocalProcessorFactory<OTAPData>] = [..];

/// A slice of local exporter factories for OTAP data.
#[distributed_slice]
pub static LOCAL_EXPORTERS: [LocalExporterFactory<OTAPData>] = [..];

/// A slice of shared receiver factories for OTAP data.
#[distributed_slice]
pub static SHARED_RECEIVERS: [SharedReceiverFactory<OTAPData>] = [..];

/// A slice of shared processor factories for OTAP data.
#[distributed_slice]
pub static SHARED_PROCESSORS: [SharedProcessorFactory<OTAPData>] = [..];

/// A slice of shared exporter factories for OTAP data.
#[distributed_slice]
pub static SHARED_EXPORTERS: [SharedExporterFactory<OTAPData>] = [..];
