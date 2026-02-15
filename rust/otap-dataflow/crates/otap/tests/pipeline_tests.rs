// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Verifies pipeline entity lifecycle handling across build/run/shutdown.
//!
//! This test constructs a minimal pipeline (fake data generator -> noop exporter),
//! asserts that pipeline/node/channel entities are registered after build, runs the
//! pipeline until a graceful shutdown, and then confirms that all related entities
//! and metric sets are unregistered to avoid registry leaks.

use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::{
    DispatchPolicy, PipelineConfig, PipelineConfigBuilder, PipelineType,
};
use otap_df_config::policy::{FlowPolicy, TelemetryPolicy};
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
use std::collections::HashMap;
use std::time::{Duration, Instant};
use weaver_common::vdir::VirtualDirectoryPath;

#[test]
fn test_telemetry_registries_cleanup() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "test-pipeline".into();
    let config = build_test_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let telemetry_policy = TelemetryPolicy::default();
    let channel_metrics_enabled = telemetry_policy.channel_metrics;
    assert!(
        channel_metrics_enabled,
        "channel metrics should be enabled for this test"
    );

    // Pipeline + nodes + control channels (one per node) + pdata channels (sender+receiver per edge).
    let expected_entities = expected_entity_count(&config);

    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx = controller_ctx.pipeline_context_with(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        0,
        1,
        0,
    );

    let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let flow_policy = FlowPolicy::default();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx.clone(),
            config.clone(),
            flow_policy.clone(),
            telemetry_policy,
            None,
        )
        .expect("failed to build runtime pipeline");

    assert_eq!(registry.entity_count(), expected_entities);

    let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
        pipeline_ctrl_msg_channel(flow_policy.channel_capacity.control.pipeline);
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

    let telemetry_policy = TelemetryPolicy::default();
    let channel_metrics_enabled = telemetry_policy.channel_metrics;
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
        controller_ctx.pipeline_context_with(pipeline_group_id, pipeline_id, 0, 1, 0);

    let _pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let _runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx,
            config,
            FlowPolicy::default(),
            telemetry_policy,
            None,
        )
        .expect("failed to build fan-in pipeline");

    assert_eq!(registry.entity_count(), expected_entities);
}

#[test]
fn test_pipeline_mixed_receivers_shared_channel_builds() {
    let pipeline_group_id: PipelineGroupId = "test-group".into();
    let pipeline_id: PipelineId = "mixed-receiver-pipeline".into();
    let config =
        build_mixed_receiver_pipeline_config(pipeline_group_id.clone(), pipeline_id.clone());

    let telemetry_policy = TelemetryPolicy::default();
    let channel_metrics_enabled = telemetry_policy.channel_metrics;
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
        controller_ctx.pipeline_context_with(pipeline_group_id, pipeline_id, 0, 1, 0);

    let _pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let _runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx,
            config,
            FlowPolicy::default(),
            telemetry_policy,
            None,
        )
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
        .one_of("receiver", ["exporter"])
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
        .one_of("receiver_a", ["exporter"])
        .one_of("receiver_b", ["exporter"])
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
        .one_of("local_receiver", ["exporter"])
        .add_receiver(
            "shared_receiver",
            OTLP_RECEIVER_URN,
            Some(shared_receiver_config_value),
        )
        .add_exporter("exporter", NOOP_EXPORTER_URN, None)
        .one_of("shared_receiver", ["exporter"])
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
    let (sender_count, receiver_count) = expected_channel_counts(config);

    // Pipeline + nodes + control channels + pdata senders + pdata receivers.
    1 + node_count + node_count + sender_count + receiver_count
}

fn expected_channel_counts(config: &PipelineConfig) -> (usize, usize) {
    #[derive(Hash, PartialEq, Eq, Clone)]
    struct EdgeKey {
        dispatch: std::mem::Discriminant<DispatchPolicy>,
        destinations: Vec<String>,
    }

    #[derive(Clone)]
    struct Edge {
        sources: Vec<(String, String)>,
        destinations: Vec<String>,
    }

    let mut edges: Vec<Edge> = Vec::new();
    let mut edge_index: HashMap<EdgeKey, Vec<usize>> = HashMap::new();

    for connection in config.connection_iter() {
        let mut destinations = connection
            .to_nodes()
            .into_iter()
            .map(|node_id| node_id.to_string())
            .collect::<Vec<_>>();
        if destinations.is_empty() {
            continue;
        }
        destinations.sort();
        destinations.dedup();

        let mut sources = connection
            .from_sources()
            .into_iter()
            .map(|source| {
                (
                    source.node_id().to_string(),
                    source.resolved_output_port().to_string(),
                )
            })
            .collect::<Vec<_>>();
        if sources.is_empty() {
            continue;
        }
        sources.sort();
        sources.dedup();

        let key = EdgeKey {
            dispatch: std::mem::discriminant(&connection.effective_dispatch_policy()),
            destinations: destinations.clone(),
        };

        let mut match_index = None;
        if let Some(indexes) = edge_index.get(&key) {
            'candidate: for &index in indexes {
                let edge = &edges[index];
                for source in &sources {
                    if edge
                        .sources
                        .iter()
                        .any(|existing| existing.0 == source.0 && existing.1 != source.1)
                    {
                        continue 'candidate;
                    }
                }
                match_index = Some(index);
                break;
            }
        }

        if let Some(index) = match_index {
            edges[index].sources.extend(sources);
            edges[index].sources.sort();
            edges[index].sources.dedup();
        } else {
            edges.push(Edge {
                sources,
                destinations: destinations.clone(),
            });
            edge_index.entry(key).or_default().push(edges.len() - 1);
        }
    }

    let sender_count = edges.iter().map(|edge| edge.sources.len()).sum();
    let receiver_count = edges.iter().map(|edge| edge.destinations.len()).sum();
    (sender_count, receiver_count)
}
