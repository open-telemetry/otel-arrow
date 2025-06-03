// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use crate::grpc::OTLPData;
use linkme::distributed_slice;
use otap_df_engine::{ExporterFactory, ProcessorFactory, ReceiverFactory};

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

/// A slice of receiver factories for OTAP data.
#[distributed_slice]
pub static RECEIVER_FACTORIES: [ReceiverFactory<OTLPData>] = [..];

/// A slice of local processor factories for OTAP data.
#[distributed_slice]
pub static PROCESSOR_FACTORIES: [ProcessorFactory<OTLPData>] = [..];

/// A slice of local exporter factories for OTAP data.
#[distributed_slice]
pub static EXPORTER_FACTORIES: [ExporterFactory<OTLPData>] = [..];

#[cfg(test)]
mod tests {
    #[test]
    fn test_plugins() {
        for receiver in crate::RECEIVER_FACTORIES.iter() {
            println!("Receiver: {}", receiver.name);
        }

        for exporter in crate::EXPORTER_FACTORIES.iter() {
            println!("Exporter: {}", exporter.name);
        }
    }
}
