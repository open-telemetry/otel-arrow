// SPDX-License-Identifier: Apache-2.0

//! # OTLP Crate
//!
//! This crate provides OpenTelemetry Protocol (OTLP) support for the OTAP dataflow system.
//! It includes functionality for:
//! - gRPC service implementation for OTLP protocol
//! - Batch processing of telemetry data (traces, metrics, logs)
//! - OTLP exporter for sending telemetry data
//! - Mock services for testing all OTLP service types (traces, metrics, logs, and profiles)
//!
//! The crate is designed to handle OpenTelemetry data in a high-performance, reliable manner
//! as part of the OTAP dataflow pipeline.
//!
//! ## Mock Services
//! The `mock` module provides mock implementations for all OTLP services:
//! - `LogsServiceMock`: Mocks the OTLP Logs service
//! - `MetricsServiceMock`: Mocks the OTLP Metrics service
//! - `TraceServiceMock`: Mocks the OTLP Trace service
//! - `ProfilesServiceMock`: Mocks the OTLP Profiles service
//!
//! These mocks are useful for testing OTLP clients and verifying that the correct data is being sent.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use crate::grpc::OTLPData;
use linkme::distributed_slice;
use otap_df_engine::local::{LocalExporterFactory, LocalProcessorFactory, LocalReceiverFactory};
use otap_df_engine::shared::{
    SharedExporterFactory, SharedProcessorFactory, SharedReceiverFactory,
};

/// compression formats
pub mod compression;
/// gRPC service implementation
pub mod grpc;
/// OTLP Batch Processor implementation
pub mod otlp_batch_processor;
pub use otlp_batch_processor::{ExportTraceServiceRequest, HierarchicalBatchSplit};

/// otlp exporter implementation
pub mod otlp_exporter;
/// Implementation of OTLP Receiver that implements the receiver trait
pub mod otlp_receiver;
/// Generated protobuf files
pub mod proto;

/// grpc mock server for testing
#[cfg(test)]
mod mock;

/// A slice of local receiver factories for OTLP data.
#[distributed_slice]
pub static LOCAL_RECEIVERS: [LocalReceiverFactory<OTLPData>] = [..];

/// A slice of local processor factories for OTLP data.
#[distributed_slice]
pub static LOCAL_PROCESSORS: [LocalProcessorFactory<OTLPData>] = [..];

/// A slice of local exporter factories for OTLP data.
#[distributed_slice]
pub static LOCAL_EXPORTERS: [LocalExporterFactory<OTLPData>] = [..];

/// A slice of shared receiver factories for OTLP data.
#[distributed_slice]
pub static SHARED_RECEIVERS: [SharedReceiverFactory<OTLPData>] = [..];

/// A slice of shared processor factories for OTLP data.
#[distributed_slice]
pub static SHARED_PROCESSORS: [SharedProcessorFactory<OTLPData>] = [..];

/// A slice of shared exporter factories for OTLP data.
#[distributed_slice]
pub static SHARED_EXPORTERS: [SharedExporterFactory<OTLPData>] = [..];

#[cfg(test)]
mod tests {
    #[test]
    fn test_plugins() {
        for receiver in crate::SHARED_RECEIVERS.iter() {
            println!("Receiver: {}", receiver.name);
        }

        for exporter in crate::LOCAL_EXPORTERS.iter() {
            println!("Exporter: {}", exporter.name);
        }
    }
}
