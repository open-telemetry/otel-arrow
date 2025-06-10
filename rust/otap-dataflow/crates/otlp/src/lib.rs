// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use crate::grpc::OTLPData;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;
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

/// Factory for OTLP-based pipeline
#[pipeline_factory(OTLPData)]
static OTLP_PIPELINE_FACTORY: PipelineFactory<OTLPData> = build_factory();

#[cfg(test)]
mod tests {
    use crate::OTLP_PIPELINE_FACTORY;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};

    #[test]
    fn test_build_runtime_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("otlp_receiver", "urn:otel:otlp:receiver", None)
            .add_exporter("otlp_exporter1", "urn:otel:otlp:exporter", None)
            .add_exporter("otlp_exporter2", "urn:otel:otlp:exporter", None)
            // ToDo(LQ): Check the validity of the outport.
            .broadcast(
                "otlp_receiver",
                "out_port",
                ["otlp_exporter1", "otlp_exporter2"],
            )
            .build(PipelineType::OTLP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = OTLP_PIPELINE_FACTORY.build(config);
        assert!(
            result.is_ok(),
            "Failed to create runtime pipeline: {:?}",
            result.err()
        );
        let runtime_pipeline = result.unwrap();
        assert_eq!(
            runtime_pipeline.node_count(),
            3,
            "Expected 3 nodes in the pipeline"
        );
        dbg!(runtime_pipeline.config());
    }
}
