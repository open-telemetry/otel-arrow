// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use crate::grpc::OTLPData;
use otap_df_engine_macros::factory_registry;

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

/// Factory registry for OTLP data processing
#[factory_registry(OTLPData)]
static FACTORY_REGISTRY: FactoryRegistry<OTLPData> = unsafe { std::mem::zeroed() };


#[cfg(test)]
mod tests {
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};

    use crate::FACTORY_REGISTRY;

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
        let result = FACTORY_REGISTRY.create_runtime_pipeline(config);
        assert!(result.is_ok(), "Failed to create runtime pipeline: {:?}", result.err());
    }
}
