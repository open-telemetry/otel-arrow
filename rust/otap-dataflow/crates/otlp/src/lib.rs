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

/// OTLP batch processor implementation
mod otlp_batch_processor;

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
#[pipeline_factory(OTLP, OTLPData)]
static OTLP_PIPELINE_FACTORY: PipelineFactory<OTLPData> = build_factory();

#[cfg(test)]
mod tests {
    use crate::OTLP_PIPELINE_FACTORY;
    use otap_df_config::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;

    #[test]
    fn test_build_runtime_pipeline() {
        let config = PipelineConfigBuilder::new()
            .add_receiver(
                "otlp_receiver",
                "urn:otel:otlp:receiver",
                Some(json!({
                    "listening_addr": "127.0.0.1:4317"
                })),
            )
            .add_exporter(
                "otlp_exporter1",
                "urn:otel:otlp:exporter",
                Some(json!({
                    "grpc_endpoint": "http://127.0.0.1:1234"
                })),
            )
            .add_exporter(
                "otlp_exporter2",
                "urn:otel:otlp:exporter",
                Some(json!({
                    "grpc_endpoint": "http://127.0.0.1:1235"
                })),
            )
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
    }

    #[test]
    fn test2() {
        let config = PipelineConfigBuilder::new()
            .add_receiver("otlp_receiver", "urn:otel:otlp:receiver", Some(json!({
                "listening_addr": "127.0.0.1:4317"
            })))
            .add_exporter(
                "otlp_exporter",
                "urn:otel:otlp:exporter",
                Some(json!({
                    "grpc_endpoint": "127.0.0.1:4318"
                })),
            )
            .broadcast("otlp_receiver", "out_port", ["otlp_exporter"])
            .build(PipelineType::OTLP, "namespace", "pipeline")
            .expect("Failed to build pipeline config");
        let result = OTLP_PIPELINE_FACTORY.build(config);
        assert!(
            result.is_ok(),
            "Failed to create runtime pipeline: {:?}",
            result.err()
        );
        let _runtime_pipeline = result.unwrap();
        //runtime_pipeline.start().expect("Failed to start pipeline");
    }
}
