// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Small end-to-end liveness tests for core-node combinations.
//!
//! These tests intentionally complement the node-local harnesses. They reuse the
//! real runtime pipeline wiring to validate that retry recovery and shutdown-time
//! batch flushing still make progress once the nodes are connected through the
//! engine, without standing up heavier transport-specific topologies.

mod common;

use common::counting_exporter::{self, COUNTING_EXPORTER_URN};
use common::flaky_exporter::{self, FLAKY_EXPORTER_URN};
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::{PipelineConfig, PipelineConfigBuilder, PipelineType};
use otap_df_config::policy::{ChannelCapacityPolicy, TelemetryPolicy};
use otap_df_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otap_df_core_nodes::processors::batch_processor::OTAP_BATCH_PROCESSOR_URN;
use otap_df_core_nodes::processors::retry_processor::RETRY_PROCESSOR_URN;
use otap_df_core_nodes::receivers::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{
    RuntimeControlMsg, pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
};
use otap_df_engine::entity_context::set_pipeline_entity_key;
use otap_df_engine::testing::liveness::wait_for_condition;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

fn fake_receiver_config(
    max_signal_count: u64,
    max_batch_size: usize,
    enable_ack_nack: bool,
) -> serde_json::Value {
    json!({
        "traffic_config": {
            "signals_per_second": null,
            "max_signal_count": max_signal_count,
            "max_batch_size": max_batch_size,
            "metric_weight": 0,
            "trace_weight": 0,
            "log_weight": 100
        },
        "data_source": "static",
        "enable_ack_nack": enable_ack_nack
    })
}

fn rate_limited_fake_receiver_config(
    max_signal_count: u64,
    max_batch_size: usize,
    signals_per_second: usize,
    enable_ack_nack: bool,
) -> serde_json::Value {
    json!({
        "traffic_config": {
            "signals_per_second": signals_per_second,
            "max_signal_count": max_signal_count,
            "max_batch_size": max_batch_size,
            "metric_weight": 0,
            "trace_weight": 0,
            "log_weight": 100
        },
        "data_source": "static",
        "generation_strategy": "pre_generated",
        "enable_ack_nack": enable_ack_nack
    })
}

fn build_retry_pipeline_config(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    flaky_id: &str,
) -> PipelineConfig {
    PipelineConfigBuilder::new()
        .add_receiver(
            "fake_receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(fake_receiver_config(6, 2, true)),
        )
        .add_processor(
            "retry",
            RETRY_PROCESSOR_URN,
            Some(json!({
                "initial_interval": "20ms",
                "max_interval": "80ms",
                "max_elapsed_time": "2s",
                "multiplier": 2.0
            })),
        )
        .add_exporter(
            "flaky_exporter",
            FLAKY_EXPORTER_URN,
            Some(json!({"flaky_id": flaky_id})),
        )
        .one_of("fake_receiver", ["retry"])
        .one_of("retry", ["flaky_exporter"])
        .build(
            PipelineType::Otap,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
        )
        .expect("failed to build retry liveness pipeline config")
}

fn build_batch_pipeline_config(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    counter_id: &str,
) -> PipelineConfig {
    PipelineConfigBuilder::new()
        .add_receiver(
            "fake_receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(fake_receiver_config(3, 3, true)),
        )
        .add_processor(
            "batch",
            OTAP_BATCH_PROCESSOR_URN,
            Some(json!({
                "otap": {
                    "min_size": 10,
                    "max_size": 10,
                    "sizer": "items"
                },
                "max_batch_duration": "50ms"
            })),
        )
        .add_exporter(
            "counting_exporter",
            COUNTING_EXPORTER_URN,
            Some(json!({"counter_id": counter_id})),
        )
        .one_of("fake_receiver", ["batch"])
        .one_of("batch", ["counting_exporter"])
        .build(
            PipelineType::Otap,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
        )
        .expect("failed to build batch liveness pipeline config")
}

fn build_otlp_batch_local_wakeup_pipeline_config(
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    counter_id: &str,
) -> PipelineConfig {
    PipelineConfigBuilder::new()
        .add_receiver(
            "fake_receiver",
            OTAP_FAKE_DATA_GENERATOR_URN,
            Some(rate_limited_fake_receiver_config(5, 1, 1, true)),
        )
        .add_processor(
            "batch",
            OTAP_BATCH_PROCESSOR_URN,
            Some(json!({
                "format": "otlp",
                "otlp": {
                    "min_size": 262144,
                    "sizer": "bytes"
                },
                "max_batch_duration": "250ms"
            })),
        )
        .add_exporter(
            "counting_exporter",
            COUNTING_EXPORTER_URN,
            Some(json!({"counter_id": counter_id})),
        )
        .one_of("fake_receiver", ["batch"])
        .one_of("batch", ["counting_exporter"])
        .build(
            PipelineType::Otap,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
        )
        .expect("failed to build local wakeup batch liveness pipeline config")
}

fn run_pipeline_with_condition<F>(
    config: PipelineConfig,
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    max_duration: Duration,
    shutdown_deadline: Duration,
    shutdown_condition: Option<F>,
) where
    F: Fn() -> bool + Send + 'static,
{
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
    let channel_capacity_policy = ChannelCapacityPolicy::default();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx.clone(),
            config,
            channel_capacity_policy.clone(),
            TelemetryPolicy::default(),
            None, // transport_headers_policy
            None, // internal_telemetry
        )
        .expect("failed to build runtime pipeline");

    let (runtime_ctrl_tx, runtime_ctrl_rx) =
        runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let (pipeline_completion_tx, pipeline_completion_rx) =
        pipeline_completion_msg_channel(channel_capacity_policy.control.completion);
    let runtime_ctrl_tx_for_shutdown = runtime_ctrl_tx.clone();

    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id: pipeline_group_id.clone(),
        pipeline_id: pipeline_id.clone(),
        core_id: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    let shutdown_handle = std::thread::spawn(move || {
        let start = Instant::now();
        let poll_interval = Duration::from_millis(10);
        loop {
            if shutdown_condition
                .as_ref()
                .is_some_and(|condition| condition())
                || start.elapsed() >= max_duration
            {
                let deadline = Instant::now() + shutdown_deadline;
                runtime_ctrl_tx_for_shutdown
                    .try_send(RuntimeControlMsg::Shutdown {
                        deadline,
                        reason: "liveness test shutdown".to_owned(),
                    })
                    .expect("failed to send shutdown request");
                break;
            }
            std::thread::sleep(poll_interval);
        }
    });

    let run_result = {
        let _pipeline_entity_guard =
            set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
        let (_memory_pressure_tx, memory_pressure_rx) = tokio::sync::watch::channel(
            otap_df_engine::memory_limiter::MemoryPressureChanged::initial(),
        );
        runtime_pipeline.run_forever(
            pipeline_key,
            pipeline_ctx,
            event_reporter,
            metrics_reporter,
            Duration::from_secs(1),
            memory_pressure_rx,
            runtime_ctrl_tx,
            runtime_ctrl_rx,
            pipeline_completion_tx,
            pipeline_completion_rx,
        )
    };
    let _ = shutdown_handle.join();
    assert!(
        run_result.is_ok(),
        "pipeline failed to shut down cleanly: {run_result:?}"
    );
}

#[derive(Debug, Default)]
struct BatchMetricsSnapshot {
    fields: HashMap<String, u64>,
}

impl BatchMetricsSnapshot {
    fn get(&self, field: &str) -> u64 {
        self.fields.get(field).copied().unwrap_or(u64::MAX)
    }

    fn assert_eq(&self, field: &str, expected: u64) {
        let actual = self.get(field);
        assert_eq!(
            actual, expected,
            "{field}: expected {expected}, got {actual}"
        );
    }
}

fn capture_batch_metrics(
    registry: &otap_df_telemetry::registry::TelemetryRegistryHandle,
) -> BatchMetricsSnapshot {
    let mut snapshot = BatchMetricsSnapshot::default();
    registry.visit_current_metrics(|desc, _attrs, iter| {
        if desc.name == "otap.processor.batch" {
            for (field, value) in iter {
                let _ = snapshot
                    .fields
                    .insert(field.name.to_owned(), value.to_u64_lossy());
            }
        }
    });
    snapshot
}

fn run_pipeline_and_capture_batch_metrics<F>(
    config: PipelineConfig,
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    max_duration: Duration,
    shutdown_deadline: Duration,
    shutdown_condition: F,
) -> BatchMetricsSnapshot
where
    F: Fn() -> bool + Send + 'static,
{
    let telemetry_system = InternalTelemetrySystem::default();
    let registry = telemetry_system.registry();
    let collector = telemetry_system.collector();
    let controller_ctx = ControllerContext::new(registry.clone());
    let pipeline_ctx = controller_ctx.pipeline_context_with(
        pipeline_group_id.clone(),
        pipeline_id.clone(),
        0,
        1,
        0,
    );
    let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
    let channel_capacity_policy = ChannelCapacityPolicy::default();
    let runtime_pipeline = OTAP_PIPELINE_FACTORY
        .build(
            pipeline_ctx.clone(),
            config,
            channel_capacity_policy.clone(),
            TelemetryPolicy::default(),
            None, // transport_headers_policy
            None, // internal_telemetry
        )
        .expect("failed to build runtime pipeline");

    let (runtime_ctrl_tx, runtime_ctrl_rx) =
        runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let (pipeline_completion_tx, pipeline_completion_rx) =
        pipeline_completion_msg_channel(channel_capacity_policy.control.completion);
    let runtime_ctrl_tx_for_shutdown = runtime_ctrl_tx.clone();

    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());
    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id: pipeline_group_id.clone(),
        pipeline_id: pipeline_id.clone(),
        core_id: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    let capture_registry = registry.clone();
    let capture_collector = collector.clone();
    let shutdown_handle = std::thread::spawn(move || {
        let start = Instant::now();
        let poll_interval = Duration::from_millis(10);
        loop {
            if start.elapsed() >= max_duration || shutdown_condition() {
                break;
            }
            capture_collector.collect_pending();
            std::thread::sleep(poll_interval);
        }

        let telemetry_wait = Duration::from_millis(1500);
        let telemetry_start = Instant::now();
        while telemetry_start.elapsed() < telemetry_wait {
            capture_collector.collect_pending();
            std::thread::sleep(poll_interval);
        }
        capture_collector.collect_pending();

        let snapshot = capture_batch_metrics(&capture_registry);

        let deadline = Instant::now() + shutdown_deadline;
        let _ = runtime_ctrl_tx_for_shutdown.try_send(RuntimeControlMsg::Shutdown {
            deadline,
            reason: "batch metrics capture shutdown".to_owned(),
        });

        snapshot
    });

    let run_result = {
        let _pipeline_entity_guard =
            set_pipeline_entity_key(pipeline_ctx.metrics_registry(), pipeline_entity_key);
        let (_memory_pressure_tx, memory_pressure_rx) = tokio::sync::watch::channel(
            otap_df_engine::memory_limiter::MemoryPressureChanged::initial(),
        );
        runtime_pipeline.run_forever(
            pipeline_key,
            pipeline_ctx,
            event_reporter,
            metrics_reporter,
            Duration::from_secs(1),
            memory_pressure_rx,
            runtime_ctrl_tx,
            runtime_ctrl_rx,
            pipeline_completion_tx,
            pipeline_completion_rx,
        )
    };
    let snapshot = shutdown_handle
        .join()
        .expect("batch metrics capture thread should succeed");
    assert!(
        run_result.is_ok(),
        "pipeline failed to shut down cleanly: {run_result:?}"
    );
    snapshot
}

// This pipeline starts with a downstream exporter that transiently Nacks every
// request. Once retries are demonstrably happening, the exporter flips to Ack
// mode and the pipeline must eventually drain all admitted work.
#[test]
fn test_retry_pipeline_eventually_recovers_after_transient_nacks() {
    let pipeline_group_id: PipelineGroupId = "liveness-group".into();
    let pipeline_id: PipelineId = "retry-pipeline-liveness".into();
    let test_id = "retry-pipeline-liveness";
    let delivered_items = Arc::new(AtomicU64::new(0));
    flaky_exporter::register_state(test_id, delivered_items.clone(), false);

    let flip_done = Arc::new(AtomicBool::new(false));
    let flip_done_for_thread = flip_done.clone();
    let flip_id = test_id.to_owned();
    let flip_thread = std::thread::spawn(move || {
        assert!(
            wait_for_condition(Duration::from_secs(2), Duration::from_millis(10), || {
                flaky_exporter::nack_count_by_id(&flip_id) >= 1
            }),
            "timed out waiting for the retry pipeline to produce an initial transient Nack"
        );
        flaky_exporter::set_should_ack_by_id(&flip_id, true);
        flip_done_for_thread.store(true, Ordering::Release);
    });

    let config = build_retry_pipeline_config(&pipeline_group_id, &pipeline_id, test_id);
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(2),
        Duration::from_secs(1),
        Some({
            let delivered_items = delivered_items.clone();
            let flip_done = flip_done.clone();
            move || {
                flip_done.load(Ordering::Acquire) && delivered_items.load(Ordering::Acquire) >= 6
            }
        }),
    );

    flip_thread
        .join()
        .expect("retry flip thread should succeed");
    assert!(
        flaky_exporter::nack_count_by_id(test_id) >= 1,
        "pipeline should observe at least one transient Nack before recovery"
    );
    assert!(
        delivered_items.load(Ordering::Acquire) >= 6,
        "all generated items should eventually be delivered after recovery"
    );
    flaky_exporter::unregister_state(test_id);
}

// This pipeline never reaches the batch size threshold, so it only makes forward
// progress once the batch processor's delayed flush fires. The exporter count
// proves that partial buffered input eventually leaves the pipeline under real
// runtime scheduling instead of remaining stuck forever below the size limit.
#[test]
fn test_batch_pipeline_eventually_flushes_partial_batch() {
    let pipeline_group_id: PipelineGroupId = "liveness-group".into();
    let pipeline_id: PipelineId = "batch-pipeline-liveness".into();
    let test_id = "batch-pipeline-liveness";
    let delivered_items = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, delivered_items.clone());

    let config = build_batch_pipeline_config(&pipeline_group_id, &pipeline_id, test_id);
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(1),
        Duration::from_secs(1),
        Some({
            let delivered_items = delivered_items.clone();
            move || delivered_items.load(Ordering::Acquire) >= 3
        }),
    );

    assert_eq!(
        delivered_items.load(Ordering::Acquire),
        3,
        "the delayed batch flush should eventually export every generated item"
    );
    counting_exporter::unregister_counter(test_id);
}

// This exercises a batch pipeline where:
// - the traffic generator emits 5 single-item OTLP log batches at 1 signal/sec
// - the batch processor uses byte sizing with a 256 KiB minimum size
// - each generated batch is intentionally tiny, so a size-triggered flush is
//   impossible under this setup
//
// The test waits for all 5 items to reach the downstream counting exporter,
// then captures the batch processor metrics from the in-process telemetry
// registry. Under these inputs, the runtime guarantees we expect are:
// - the pipeline makes forward progress without any size-based flushes
// - every generated item is eventually exported downstream
// - every flush is attributed to `flushes.timer`, which means the processor's
//   node-local wakeup path fired and delivered the timeout back through the
//   real inbox/runtime path
// - the processor emits 5 output log batches after consuming 5 input log
//   batches, so the wakeup-triggered flushes are producing real downstream
//   pdata batches rather than being dropped internally
#[test]
fn test_batch_pipeline_uses_timer_wakeup_metrics_with_otlp_bytes_config() {
    let pipeline_group_id: PipelineGroupId = "liveness-group".into();
    let pipeline_id: PipelineId = "batch-pipeline-local-wakeup".into();
    let test_id = "batch-pipeline-local-wakeup";
    let delivered_items = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, delivered_items.clone());

    let config =
        build_otlp_batch_local_wakeup_pipeline_config(&pipeline_group_id, &pipeline_id, test_id);
    let metrics = run_pipeline_and_capture_batch_metrics(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(8),
        Duration::from_secs(2),
        {
            let delivered_items = delivered_items.clone();
            move || delivered_items.load(Ordering::Acquire) >= 5
        },
    );

    assert_eq!(
        delivered_items.load(Ordering::Acquire),
        5,
        "the local wakeup pipeline should export every generated item"
    );
    metrics.assert_eq("consumed.items.logs", 5);
    metrics.assert_eq("consumed.batches.logs", 5);
    metrics.assert_eq("produced.items.logs", 5);
    metrics.assert_eq("produced.batches.logs", 5);
    metrics.assert_eq("flushes.size", 0);
    metrics.assert_eq("flushes.timer", 5);

    counting_exporter::unregister_counter(test_id);
}
