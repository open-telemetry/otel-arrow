// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use crate::grpc::OTLPData;
use otap_df_engine::create_factory_registry;

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

// Create the factory registry with distributed slices for OTLP data
create_factory_registry!(OTLPData, OtlpFactoryRegistry);



#[cfg(test)]
mod tests {
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};

    use crate::create_runtime_pipeline;

    #[test]
    fn test_plugins() {
        for receiver in crate::RECEIVER_FACTORIES.iter() {
            println!("Receiver: {}", receiver.name);
        }

        for exporter in crate::EXPORTER_FACTORIES.iter() {
            println!("Exporter: {}", exporter.name);
        }
    }

    #[test]
    fn test_create_runtime_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("otlp_receiver", "urn:otel:otlp:receiver", None)
            .add_exporter("otlp_exporter1", "urn:otel:otlp:exporter", None)
            .add_exporter("otlp_exporter2", "urn:otel:otlp:exporter", None)
            // ToDo(LQ): Check the validity of the outport.
            .broadcast("otlp_receiver", "out_port", ["otlp_exporter1", "otlp_exporter2"])
            .build(PipelineType::OTLP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = create_runtime_pipeline(config);
        assert!(result.is_ok(), "Failed to create runtime pipeline: {:?}", result.err());
    }
}
