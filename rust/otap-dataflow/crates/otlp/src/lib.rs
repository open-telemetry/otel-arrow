// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

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
