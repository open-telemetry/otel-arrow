// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies pipeline entity lifecycle handling across build/run/shutdown.
//!
//! This test constructs a minimal pipeline (fake data generator -> noop exporter),
//! asserts that pipeline/node/channel entities are registered after build, runs the
//! pipeline until a graceful shutdown, and then confirms that all related entities
//! and metric sets are unregistered to avoid registry leaks.

use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::{PipelineConfig, PipelineConfigBuilder, PipelineType};
use otap_df_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
use otap_df_engine::entity_context::set_pipeline_entity_key;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_otap::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
use otap_df_otap::fake_data_generator::config::{Config as FakeDataGeneratorConfig, TrafficConfig};
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use serde_json::to_value;
use std::time::{Duration, Instant};
use weaver_common::vdir::VirtualDirectoryPath;

#[test]
fn test_telemetry_registries_cleanup() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "test-pipeline".into();
    let config = build_test_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let node_count = config.node_iter().count();
    let edge_count = config
        .node_iter()
        .map(|(_, node)| {
            node.out_ports
                .values()
                .map(|edge| edge.destinations.len())
                .sum::<usize>()
        })
        .sum::<usize>();
    let channel_metrics_enabled = config.pipeline_settings().telemetry.channel_metrics;
    assert!(
        channel_metrics_enabled,
        "channel metrics should be enabled for this test"
    );

    // Pipeline + nodes + control channels (one per node) + pdata channels (sender+receiver per edge).
    let expected_entities = 1 + node_count + node_count + (edge_count * 2);

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx =
        controller_ctx.pipeline_context_with(pipeline_group_id.clone(), pipeline_id.clone(), 0, 0);

    let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(pipeline_ctx.clone(), config.clone(), None)
        .expect("failed to build runtime pipeline");

    assert_eq!(registry.entity_count(), expected_entities);

    let (pipeline_ctrl_tx, pipeline_ctrl_rx) = pipeline_ctrl_msg_channel(
        config
            .pipeline_settings()
            .default_pipeline_ctrl_msg_channel_size,
    );
    let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
    let observed_state_store = ObservedStateStore::new(&ObservedStateSettings::default());

    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id,
        pipeline_id,
        core_id: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    let shutdown_handle = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        let deadline = Instant::now() + Duration::from_millis(200);
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
    assert!(run_result.is_ok(), "pipeline failed to shut down cleanly");

    assert_eq!(registry.metric_set_count(), 0);
    assert_eq!(registry.entity_count(), 0);
}

fn build_test_pipeline_config(
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
) -> PipelineConfig {
    let traffic_config = TrafficConfig::new(Some(1), Some(1), 1, 1, 1, 1);
    let registry_path = VirtualDirectoryPath::GitRepo {
        url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
        sub_folder: Some("model".to_owned()),
        refspec: None,
    };
    let receiver_config = FakeDataGeneratorConfig::new(traffic_config, registry_path);
    let receiver_config_value =
        to_value(receiver_config).expect("failed to serialize receiver config");

    PipelineConfigBuilder::new()
        .add_receiver(
            "receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(receiver_config_value),
        )
        .add_exporter("exporter", "urn:otel:noop:exporter", None)
        .round_robin("receiver", "out", ["exporter"])
        .build(PipelineType::Otap, pipeline_group_id, pipeline_id)
        .expect("failed to build pipeline config")
}
