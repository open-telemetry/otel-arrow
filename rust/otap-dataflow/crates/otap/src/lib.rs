// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

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

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAPData)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();

#[cfg(test)]
mod tests {
    use crate::OTAP_PIPELINE_FACTORY;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};

    #[test]
    fn test_build_runtime_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("receiver", "urn:otel:otap:receiver", None)
            .add_exporter("exporter1", "urn:otel:otap:exporter", None)
            .add_exporter("exporter2", "urn:otel:otap:exporter", None)
            // ToDo(LQ): Check the validity of the outport.
            .broadcast(
                "receiver",
                "out_port",
                ["exporter1", "exporter2"],
            )
            .build(PipelineType::OTAP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = OTAP_PIPELINE_FACTORY.build(config);
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