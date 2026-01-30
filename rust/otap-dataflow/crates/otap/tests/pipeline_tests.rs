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
use otap_df_otap::fake_data_generator::config::{
    Config as FakeDataGeneratorConfig, DataSource, TrafficConfig,
};
use otap_df_otap::noop_exporter::NOOP_EXPORTER_URN;
use otap_df_otap::otlp_receiver::OTLP_RECEIVER_URN;
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use serde_json::{json, to_value};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use weaver_common::vdir::VirtualDirectoryPath;

#[test]
fn test_telemetry_registries_cleanup() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "test-pipeline".into();
    let config = build_test_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let channel_metrics_enabled = config.pipeline_settings().telemetry.channel_metrics;
    assert!(
        channel_metrics_enabled,
        "channel metrics should be enabled for this test"
    );

    // Pipeline + nodes + control channels (one per node) + pdata channels (sender+receiver per edge).
    let expected_entities = expected_entity_count(&config);

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
    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());

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
    assert!(
        run_result.is_ok(),
        "pipeline failed to shut down cleanly: {run_result:?}"
    );

    assert_eq!(registry.metric_set_count(), 0);
    assert_eq!(registry.entity_count(), 0);
}

#[test]
fn test_pipeline_fan_in_builds() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "fan-in-pipeline".into();
    let config = build_fan_in_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let channel_metrics_enabled = config.pipeline_settings().telemetry.channel_metrics;
    assert!(
        channel_metrics_enabled,
        "channel metrics should be enabled for this test"
    );

    // Pipeline + nodes + control channels (one per node) + pdata channels (sender+receiver per edge).
    let expected_entities = expected_entity_count(&config);

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx = controller_ctx.pipeline_context_with(pipeline_group_id, pipeline_id, 0, 0);

    let _pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let _runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(pipeline_ctx, config, None)
        .expect("failed to build fan-in pipeline");

    assert_eq!(registry.entity_count(), expected_entities);
}

#[test]
fn test_pipeline_mixed_receivers_shared_channel_builds() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "mixed-receiver-pipeline".into();
    let config =
        build_mixed_receiver_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let channel_metrics_enabled = config.pipeline_settings().telemetry.channel_metrics;
    assert!(
        channel_metrics_enabled,
        "channel metrics should be enabled for this test"
    );

    // Pipeline + nodes + control channels (one per node) + pdata channels (sender+receiver per edge).
    let expected_entities = expected_entity_count(&config);

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx = controller_ctx.pipeline_context_with(pipeline_group_id, pipeline_id, 0, 0);

    let _pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let _runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(pipeline_ctx, config, None)
        .expect("failed to build mixed receiver pipeline");

    assert_eq!(registry.entity_count(), expected_entities);
}

fn build_test_pipeline_config(
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
) -> PipelineConfig {
    let receiver_config_value = fake_receiver_config_value();

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

fn build_fan_in_pipeline_config(
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
) -> PipelineConfig {
    let receiver_config_value = fake_receiver_config_value();

    PipelineConfigBuilder::new()
        .add_receiver(
            "receiver_a",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(receiver_config_value.clone()),
        )
        .add_receiver(
            "receiver_b",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(receiver_config_value),
        )
        .add_exporter("exporter", "urn:otel:noop:exporter", None)
        .round_robin("receiver_a", "out", ["exporter"])
        .round_robin("receiver_b", "out", ["exporter"])
        .build(PipelineType::Otap, pipeline_group_id, pipeline_id)
        .expect("failed to build pipeline config")
}

fn build_mixed_receiver_pipeline_config(
    pipeline_group_id: PipelineGroupId,
    pipeline_id: PipelineId,
) -> PipelineConfig {
    let local_receiver_config_value = fake_receiver_config_value();
    let shared_receiver_config_value = otlp_receiver_config_value();

    PipelineConfigBuilder::new()
        .add_receiver(
            "local_receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(local_receiver_config_value),
        )
        .round_robin("local_receiver", "out", ["exporter"])
        .add_receiver(
            "shared_receiver",
            OTLP_RECEIVER_URN,
            Some(shared_receiver_config_value),
        )
        .add_exporter("exporter", NOOP_EXPORTER_URN, None)
        .round_robin("shared_receiver", "out", ["exporter"])
        .build(PipelineType::Otap, pipeline_group_id, pipeline_id)
        .expect("failed to build pipeline config")
}

fn fake_receiver_config_value() -> serde_json::Value {
    let traffic_config = TrafficConfig::new(Some(1), Some(1), 1, 1, 1, 1);
    let registry_path = VirtualDirectoryPath::GitRepo {
        url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
        sub_folder: Some("model".to_owned()),
        refspec: None,
    };
    let receiver_config = FakeDataGeneratorConfig::new(traffic_config, registry_path)
        .with_data_source(DataSource::Static);
    to_value(receiver_config).expect("failed to serialize receiver config")
}

fn otlp_receiver_config_value() -> serde_json::Value {
    json!({
        "protocols": {
            "grpc": {
                "listening_addr": "127.0.0.1:0"
            }
        }
    })
}

fn expected_entity_count(config: &PipelineConfig) -> usize {
    let node_count = config.node_iter().count();
    let mut edge_count = 0;
    let mut destination_nodes = HashSet::new();

    for (_, node) in config.node_iter() {
        for edge in node.out_ports.values() {
            edge_count += edge.destinations.len();
            destination_nodes.extend(edge.destinations.iter().cloned());
        }
    }

    // Pipeline + nodes + control channels + pdata senders + pdata receivers.
    1 + node_count + node_count + edge_count + destination_nodes.len()
}
