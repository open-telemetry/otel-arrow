// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP nodes (receiver, exporter, processor).

use crate::pdata::OtapPdata;
use otap_df_engine::{PipelineFactory, build_factory};
use otap_df_engine_macros::pipeline_factory;

/// Code for encoding OTAP batch from pdata view
pub mod encoder;
/// Implementation of OTAP Exporter that implements the exporter trait
pub mod otap_exporter;
/// gRPC service implementation
pub mod otap_grpc;
/// Implementation of OTAP Receiver that implements the receiver trait
pub mod otap_receiver;

/// This receiver receives OTLP bytes from the grpc service request and
/// produce for the pipeline OTAP PData
pub mod otlp_receiver;

/// Implementation of OTLP exporter that implements the exporter trait
pub mod otlp_exporter;

// OTAP batch processor
pub mod otap_batch_processor;

// Retry processor that is aware of the OTAP PData/context.
pub mod retry_processor;

/// Receiver that reads in syslog data
pub mod syslog_cef_receiver;

/// Generated protobuf files
pub mod proto;

pub mod pdata;

pub mod parquet_exporter;

pub mod perf_exporter;

pub mod fake_data_generator;

/// Implementation of debug processor that outputs received signals in a string format for user view
pub mod debug_processor;

/// Implementation of a noop exporter that acts as a exporter placeholder
pub mod noop_exporter;

/// testing utilities
#[cfg(test)]
mod otap_mock;
#[cfg(test)]
mod otlp_mock;

#[cfg(test)]
mod fixtures;

/// Signal-type router processor (OTAP-based)
pub mod signal_type_router;

/// Attributes processor (OTAP-based)
pub mod attributes_processor;
/// compression formats
pub mod compression;
mod metrics;
/// gRPC service implementation
pub mod otlp_grpc;

/// Factory for OTAP-based pipeline
#[pipeline_factory(OTAP, OtapPdata)]
pub static OTAP_PIPELINE_FACTORY: PipelineFactory<OtapPdata> = build_factory();

#[cfg(test)]
mod tests {
    use otap_df_config::{
        engine::HttpAdminSettings,
        pipeline::PipelineConfig,
        pipeline_group::{CoreAllocation, Quota},
    };
    use otap_df_controller::Controller;

    use std::path::Path;
    use std::time::{Duration, Instant};

    /// Test this pipeline factory with an invalid confguration.

    #[test]
    fn test_invalid_config_fails_fast() {
        let path = Path::new("configs/test-invalid-config.yaml");
        assert!(path.exists(), "test config not found: {path:?}",);

        let pipeline_cfg =
            PipelineConfig::from_file("test_group".into(), "test_pipeline".into(), path)
                .expect("is OK");

        let controller = Controller::new(&super::OTAP_PIPELINE_FACTORY);
        let quota = Quota {
            core_allocation: CoreAllocation::CoreCount { count: 1 },
        };
        let admin_settings = HttpAdminSettings {
            bind_address: "127.0.0.1:0".to_string(),
        };

        let start_time = Instant::now();

        let result = controller.run_forever(
            "test_group".into(),
            "test_pipeline".into(),
            pipeline_cfg,
            quota,
            admin_settings,
        );

        let elapsed = start_time.elapsed();

        // Verify it failed (didn't hang), within 5s, with a "retry" substring.
        assert!(result.is_err(), "expected configuration error");
        let error_msg = format!("{}", result.unwrap_err());
        assert!(
            elapsed < Duration::from_secs(5),
            "controller should fail fast, took {:?}",
            elapsed
        );
        assert!(
            error_msg.contains("retry"),
            "error should mention retry config issue: {error_msg}",
        );
    }
}
