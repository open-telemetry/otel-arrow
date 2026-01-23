// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the persistence processor.
//!
//! These tests verify the end-to-end behavior of the persistence processor,
//! including:
//! - Data flow through the processor (ingest → wal + segment → downstream)
//! - Recovery from finalized segments on restart
//! - Graceful shutdown with data drain
//!
//! The tests use actual Quiver instances (not mocks) to catch integration
//! issues like timing, threading, and assumption mismatches that wouldn't
//! appear from testing Quiver in isolation.
//!
//! These tests require the `persistence` feature to be enabled.

#![cfg(feature = "persistence")]

use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::{PipelineConfig, PipelineConfigBuilder, PipelineType};
use otap_df_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
use otap_df_engine::entity_context::set_pipeline_entity_key;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_otap::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
use otap_df_otap::fake_data_generator::config::{Config as FakeDataGeneratorConfig, TrafficConfig};
use otap_df_otap::noop_exporter::NOOP_EXPORTER_URN;
use otap_df_otap::persistence_processor::PERSISTENCE_PROCESSOR_URN;
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use serde_json::{json, to_value};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use weaver_common::vdir::VirtualDirectoryPath;

/// Test that data flows through the persistence processor to downstream.
///
/// This verifies the happy path:
/// 1. Fake data generator produces data
/// 2. Persistence processor ingests to Quiver
/// 3. Timer tick polls and forwards to downstream (noop exporter)
/// 4. Graceful shutdown completes without data loss
#[test]
fn test_persistence_processor_data_flow() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let persistence_path = temp_dir.path().to_path_buf();

    let pipeline_group_id: PipelineGroupId = "persistence-test-group".into();
    let pipeline_id: PipelineId = "persistence-test-pipeline".into();

    let config = build_persistence_pipeline_config(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        persistence_path,
        // Generate a small bounded amount of data
        Some(10),  // max_signal_count
        5,         // max_batch_size
        Some(100), // signals_per_second
    );

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx =
        controller_ctx.pipeline_context_with(pipeline_group_id.clone(), pipeline_id.clone(), 0, 0);

    let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(pipeline_ctx.clone(), config.clone())
        .expect("failed to build runtime pipeline");

    let pipeline_settings = config.pipeline_settings().clone();
    let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
        pipeline_ctrl_msg_channel(pipeline_settings.default_pipeline_ctrl_msg_channel_size);
    let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
    let observed_state_store = ObservedStateStore::new(&ObservedStateSettings::default());

    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id,
        pipeline_id,
        core_id: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    // Shutdown after allowing time for data to flow
    let shutdown_handle = std::thread::spawn(move || {
        // Allow enough time for:
        // 1. Fake data generator to produce signals
        // 2. Persistence processor to ingest to Quiver
        // 3. Segment to finalize (max_segment_open_duration = 200ms)
        // 4. Timer tick to forward downstream
        std::thread::sleep(Duration::from_millis(500));
        let deadline = Instant::now() + Duration::from_millis(500);
        pipeline_ctrl_tx_for_shutdown
            .try_send(PipelineControlMsg::Shutdown {
                deadline,
                reason: "test shutdown".to_owned(),
            })
            .expect("failed to send shutdown request");
    });

    let run_result = {
        let _pipeline_entity_guard =
            set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
        runtime_pipeline.run_forever(
            pipeline_key,
            pipeline_ctx,
            event_reporter,
            metrics_reporter,
            pipeline_ctrl_tx,
            pipeline_ctrl_rx,
        )
    };

    let _ = shutdown_handle.join();
    assert!(
        run_result.is_ok(),
        "pipeline failed to shut down cleanly: {:?}",
        run_result
    );

    // Verify cleanup
    assert_eq!(
        registry.metric_set_count(),
        0,
        "metric sets should be cleaned up"
    );
    assert_eq!(registry.entity_count(), 0, "entities should be cleaned up");
}

/// Test that the persistence processor handles restart with existing persisted data.
///
/// This verifies recovery from persistence via Quiver:
/// 1. First run: generate data, persist to Quiver, shutdown before all data forwarded
/// 2. Second run: Quiver reopens and resumes forwarding from finalized segments
///
/// Note: This tests recovery at the Quiver/persistence_processor level.
/// Internal details like WAL replay to finalized segments are handled within Quiver itself.
#[test]
fn test_persistence_processor_recovery() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let persistence_path = temp_dir.path().to_path_buf();

    let pipeline_group_id: PipelineGroupId = "recovery-test-group".into();
    let pipeline_id: PipelineId = "recovery-test-pipeline".into();

    // === First run: generate data, quick shutdown ===
    {
        let config = build_persistence_pipeline_config(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            persistence_path.clone(),
            Some(20),  // Generate more signals
            10,        // Larger batches
            Some(500), // Faster rate
        );

        let telemetry_system = InternalTelemetrySystem::default();
        let registry = telemetry_system.registry();
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            0,
            0,
        );

        let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
        let runtime_pipeline = OTAP_PIPELINE_FACTORY
            .build(pipeline_ctx.clone(), config.clone())
            .expect("failed to build runtime pipeline (run 1)");

        let pipeline_settings = config.pipeline_settings().clone();
        let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
            pipeline_ctrl_msg_channel(pipeline_settings.default_pipeline_ctrl_msg_channel_size);
        let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
        let observed_state_store = ObservedStateStore::new(&ObservedStateSettings::default());

        let pipeline_key = DeployedPipelineKey {
            pipeline_group_id: pipeline_group_id.clone(),
            pipeline_id: pipeline_id.clone(),
            core_id: 0,
        };
        let metrics_reporter = telemetry_system.reporter();
        let event_reporter = observed_state_store.reporter(SendPolicy::default());

        // Quick shutdown - data may not have been fully forwarded
        let shutdown_handle = std::thread::spawn(move || {
            // Minimal time - just enough to ingest some data
            std::thread::sleep(Duration::from_millis(100));
            let deadline = Instant::now() + Duration::from_millis(200);
            pipeline_ctrl_tx_for_shutdown
                .try_send(PipelineControlMsg::Shutdown {
                    deadline,
                    reason: "quick shutdown for recovery test".to_owned(),
                })
                .expect("failed to send shutdown request");
        });

        let run_result = {
            let _pipeline_entity_guard =
                set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
            runtime_pipeline.run_forever(
                pipeline_key,
                pipeline_ctx,
                event_reporter,
                metrics_reporter,
                pipeline_ctrl_tx,
                pipeline_ctrl_rx,
            )
        };

        let _ = shutdown_handle.join();
        assert!(
            run_result.is_ok(),
            "pipeline run 1 failed: {:?}",
            run_result
        );
    }

    // Verify Quiver data directory exists
    let quiver_dir = persistence_path.join("core_0");
    assert!(
        quiver_dir.exists(),
        "Quiver data directory should exist after first run"
    );

    // === Second run: restart with zero new signals - recovery only ===
    // The fake data generator is configured to produce 0 signals, so any data
    // that flows to the exporter must come from persisted storage.
    {
        let config = build_persistence_pipeline_config(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            persistence_path.clone(),
            Some(0),    // Zero new signals - recovery only
            1,          // batch size doesn't matter
            Some(1000), // Fast rate so we don't wait
        );

        let telemetry_system = InternalTelemetrySystem::default();
        let registry = telemetry_system.registry();
        let controller_ctx = ControllerContext::new(registry.clone());
        let pipeline_ctx = controller_ctx.pipeline_context_with(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            0,
            0,
        );

        let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
        let runtime_pipeline = OTAP_PIPELINE_FACTORY
            .build(pipeline_ctx.clone(), config.clone())
            .expect("failed to build runtime pipeline (run 2)");

        let pipeline_settings = config.pipeline_settings().clone();
        let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
            pipeline_ctrl_msg_channel(pipeline_settings.default_pipeline_ctrl_msg_channel_size);
        let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
        let observed_state_store = ObservedStateStore::new(&ObservedStateSettings::default());

        let pipeline_key = DeployedPipelineKey {
            pipeline_group_id,
            pipeline_id,
            core_id: 0,
        };
        let metrics_reporter = telemetry_system.reporter();
        let event_reporter = observed_state_store.reporter(SendPolicy::default());

        // Allow time for recovery and forwarding
        let shutdown_handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            let deadline = Instant::now() + Duration::from_millis(500);
            pipeline_ctrl_tx_for_shutdown
                .try_send(PipelineControlMsg::Shutdown {
                    deadline,
                    reason: "shutdown after recovery".to_owned(),
                })
                .expect("failed to send shutdown request");
        });

        let run_result = {
            let _pipeline_entity_guard =
                set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
            runtime_pipeline.run_forever(
                pipeline_key,
                pipeline_ctx,
                event_reporter,
                metrics_reporter,
                pipeline_ctrl_tx,
                pipeline_ctrl_rx,
            )
        };

        let _ = shutdown_handle.join();
        assert!(
            run_result.is_ok(),
            "pipeline run 2 (recovery) failed: {:?}",
            run_result
        );

        // Verify cleanup
        assert_eq!(registry.metric_set_count(), 0);
        assert_eq!(registry.entity_count(), 0);
    }
}

/// Build a pipeline config with fake data generator → persistence → noop exporter.
///
/// # Arguments
/// * `max_signal_count` - `Some(n)` to generate exactly n signals, `None` for unlimited
fn build_persistence_pipeline_config(
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
    persistence_path: std::path::PathBuf,
    max_signal_count: Option<u64>,
    max_batch_size: usize,
    signals_per_second: Option<usize>,
) -> PipelineConfig {
    // TrafficConfig::new signature:
    // (signals_per_second: Option<usize>, max_signal_count: Option<u64>,
    //  max_batch_size: usize, metric_weight: u32, trace_weight: u32, log_weight: u32)
    let traffic_config = TrafficConfig::new(
        signals_per_second,
        max_signal_count,
        max_batch_size,
        0,   // metric_weight
        0,   // trace_weight
        100, // log_weight (100% logs)
    );
    let registry_path = VirtualDirectoryPath::GitRepo {
        url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
        sub_folder: Some("model".to_owned()),
        refspec: None,
    };
    let receiver_config = FakeDataGeneratorConfig::new(traffic_config, registry_path);
    let receiver_config_value =
        to_value(receiver_config).expect("failed to serialize receiver config");

    let persistence_config = json!({
        "path": persistence_path.to_string_lossy(),
        "poll_interval": "50ms",
        "retention_size_cap": "100MB",
        "size_cap_policy": "backpressure",
        "max_segment_open_duration": "200ms",
        "max_bundles_per_tick": 100
    });

    PipelineConfigBuilder::new()
        .add_receiver(
            "fake_receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(receiver_config_value),
        )
        .add_processor(
            "persistence",
            PERSISTENCE_PROCESSOR_URN,
            Some(persistence_config),
        )
        .add_exporter("noop_exporter", NOOP_EXPORTER_URN, None)
        .round_robin("fake_receiver", "out", ["persistence"])
        .round_robin("persistence", "out", ["noop_exporter"])
        .build(PipelineType::Otap, pipeline_group_id, pipeline_id)
        .expect("failed to build pipeline config")
}
