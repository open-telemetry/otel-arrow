// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).
//!

use crate::grpc::OTAPData;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

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
pub mod fake_data_generator;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OTAPData)]
static OTAP_PIPELINE_FACTORY: PipelineFactory<OTAPData> = build_factory();

#[cfg(test)]
mod tests {
    use crate::OTAP_PIPELINE_FACTORY;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;
    use crate::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
    use crate::perf_exporter::exporter::OTAP_PERF_EXPORTER_URN;

    #[test]
    fn test_mini_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("receiver", OTAP_FAKE_DATA_GENERATOR_URN, Some(json!({
                "batch_count": 10000000 
            })))
            .add_exporter("exporter", OTAP_PERF_EXPORTER_URN, Some(json!({
                "disk_usage": false,
                "io_usage": false
            })))
            // ToDo(LQ): Check the validity of the outport.
            .broadcast(
                "receiver",
                "out_port",
                ["exporter"],
            )
            .build(PipelineType::OTAP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        
        let runtime_pipeline = OTAP_PIPELINE_FACTORY
            .build(config)
            .expect("Failed to create runtime pipeline");
        assert_eq!(
            runtime_pipeline.node_count(),
            2,
            "Expected 2 nodes in the pipeline"
        );

        _ = runtime_pipeline.start().expect("Failed to start pipeline");
    }
}