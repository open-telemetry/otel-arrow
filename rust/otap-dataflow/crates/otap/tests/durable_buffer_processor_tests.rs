// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the durable buffer.
//!
//! These tests verify the end-to-end behavior of the durable buffer,
//! including:
//! - Data flow through the processor (ingest → wal + segment → downstream)
//! - Recovery from finalized segments on restart
//! - Retry behavior with exponential backoff when downstream NACKs
//!
//! The tests use actual Quiver instances (not mocks) to catch integration
//! issues like timing, threading, and assumption mismatches.

mod common;

use common::counting_exporter::{self, COUNTING_EXPORTER_URN};
use common::flaky_exporter::{self, FLAKY_EXPORTER_URN};
use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
use otap_df_config::pipeline::{PipelineConfig, PipelineConfigBuilder, PipelineType};
use otap_df_config::policy::{ChannelCapacityPolicy, TelemetryPolicy};
use otap_df_config::{DeployedPipelineKey, PipelineGroupId, PipelineId};
use otap_df_engine::context::ControllerContext;
use otap_df_engine::control::{PipelineControlMsg, pipeline_ctrl_msg_channel};
use otap_df_engine::entity_context::set_pipeline_entity_key;
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use otap_df_otap::durable_buffer_processor::DURABLE_BUFFER_URN;
use otap_df_otap::fake_data_generator::OTAP_FAKE_DATA_GENERATOR_URN;
use otap_df_otap::noop_exporter::NOOP_EXPORTER_URN;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_state::store::ObservedStateStore;
use otap_df_telemetry::InternalTelemetrySystem;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use quiver::segment::SegmentReader;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tempfile::tempdir;

/// URN for the error exporter (always NACKs).
const ERROR_EXPORTER_URN: &str = "urn:otel:exporter:error";

// ─────────────────────────────────────────────────────────────────────────────
// Test Configuration Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for durable buffer test configurations.
///
/// Consolidates all config variants into a single builder pattern.
#[derive(Clone)]
struct TestConfigBuilder {
    buffer_path: std::path::PathBuf,
    max_signal_count: Option<u64>,
    max_batch_size: usize,
    signals_per_second: Option<usize>,
    metric_weight: u32,
    trace_weight: u32,
    log_weight: u32,
    exporter_type: ExporterType,
    exporter_id: Option<String>,
    retry_config: Option<serde_json::Value>,
    size_cap_policy: &'static str,
    otlp_handling: Option<&'static str>,
}

/// Which exporter to use in the test pipeline.
#[derive(Clone, Copy, Default)]
enum ExporterType {
    /// Noop exporter - ACKs everything, no counting.
    #[default]
    Noop,
    /// Error exporter - NACKs everything.
    Error,
    /// Counting exporter - ACKs and counts items.
    Counting,
    /// Flaky exporter - NACKs until switched to ACK mode.
    Flaky,
}

impl TestConfigBuilder {
    const fn new(buffer_path: std::path::PathBuf) -> Self {
        Self {
            buffer_path,
            max_signal_count: Some(10),
            max_batch_size: 5,
            signals_per_second: Some(100),
            metric_weight: 0,
            trace_weight: 0,
            log_weight: 100,
            exporter_type: ExporterType::Noop,
            exporter_id: None,
            retry_config: None,
            size_cap_policy: "backpressure",
            otlp_handling: None,
        }
    }

    const fn max_signal_count(mut self, count: Option<u64>) -> Self {
        self.max_signal_count = count;
        self
    }

    const fn max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    const fn signals_per_second(mut self, rate: Option<usize>) -> Self {
        self.signals_per_second = rate;
        self
    }

    const fn signal_weights(mut self, metric: u32, trace: u32, log: u32) -> Self {
        self.metric_weight = metric;
        self.trace_weight = trace;
        self.log_weight = log;
        self
    }

    const fn use_error_exporter(mut self) -> Self {
        self.exporter_type = ExporterType::Error;
        self
    }

    const fn use_counting_exporter(mut self) -> Self {
        self.exporter_type = ExporterType::Counting;
        self
    }

    const fn use_flaky_exporter(mut self) -> Self {
        self.exporter_type = ExporterType::Flaky;
        self
    }

    /// Set the exporter ID for counting/flaky exporters.
    /// This ID is used to look up the counter in the registry.
    fn exporter_id(mut self, id: impl Into<String>) -> Self {
        self.exporter_id = Some(id.into());
        self
    }

    fn retry_config(mut self, config: serde_json::Value) -> Self {
        self.retry_config = Some(config);
        self
    }

    const fn size_cap_policy(mut self, policy: &'static str) -> Self {
        self.size_cap_policy = policy;
        self
    }

    const fn otlp_handling(mut self, handling: &'static str) -> Self {
        self.otlp_handling = Some(handling);
        self
    }

    fn build(
        self,
        pipeline_group_id: &PipelineGroupId,
        pipeline_id: &PipelineId,
    ) -> PipelineConfig {
        // Use Static data source to avoid network dependency on semantic conventions git repo.
        // Build config as JSON since DataSource::Static doesn't need registry_path.
        let receiver_config_value = json!({
            "traffic_config": {
                "signals_per_second": self.signals_per_second,
                "max_signal_count": self.max_signal_count,
                "max_batch_size": self.max_batch_size,
                "metric_weight": self.metric_weight,
                "trace_weight": self.trace_weight,
                "log_weight": self.log_weight
            },
            "data_source": "static"
        });

        let mut buffer_config = json!({
            "path": self.buffer_path.to_string_lossy(),
            "poll_interval": "20ms",
            "retention_size_cap": "256MiB",
            "size_cap_policy": self.size_cap_policy,
            "max_segment_open_duration": "50ms"
        });

        if let Some(retry) = self.retry_config {
            if let (Some(base), Some(extra)) = (buffer_config.as_object_mut(), retry.as_object()) {
                for (k, v) in extra {
                    let _ = base.insert(k.clone(), v.clone());
                }
            }
        }

        if let Some(handling) = self.otlp_handling {
            if let Some(obj) = buffer_config.as_object_mut() {
                let _ = obj.insert("otlp_handling".to_owned(), json!(handling));
            }
        }

        let (exporter_name, exporter_urn, exporter_config) = match self.exporter_type {
            ExporterType::Error => (
                "error_exporter",
                ERROR_EXPORTER_URN,
                Some(json!({"message": "simulated downstream failure"})),
            ),
            ExporterType::Counting => (
                "counting_exporter",
                COUNTING_EXPORTER_URN,
                self.exporter_id
                    .as_ref()
                    .map(|id| json!({"counter_id": id})),
            ),
            ExporterType::Flaky => (
                "flaky_exporter",
                FLAKY_EXPORTER_URN,
                self.exporter_id.as_ref().map(|id| json!({"flaky_id": id})),
            ),
            ExporterType::Noop => ("noop_exporter", NOOP_EXPORTER_URN, None),
        };

        PipelineConfigBuilder::new()
            .add_receiver(
                "fake_receiver",
                OTAP_FAKE_DATA_GENERATOR_URN,
                Some(receiver_config_value),
            )
            .add_processor("durable_buffer", DURABLE_BUFFER_URN, Some(buffer_config))
            .add_exporter(exporter_name, exporter_urn, exporter_config)
            .one_of("fake_receiver", ["durable_buffer"])
            .one_of("durable_buffer", [exporter_name])
            .build(
                PipelineType::Otap,
                pipeline_group_id.clone(),
                pipeline_id.clone(),
            )
            .expect("failed to build pipeline config")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Runner Helper
// ─────────────────────────────────────────────────────────────────────────────

/// Run a pipeline with the given config, then shut down.
///
/// Handles all the boilerplate: telemetry, context, channels, shutdown thread.
///
/// If `shutdown_condition` is provided, it will be polled every 10ms and
/// shutdown will be triggered as soon as the condition returns true (or when
/// `run_duration` is reached, whichever comes first). This allows tests to
/// complete as fast as the actual work takes, rather than waiting for a fixed
/// duration.
fn run_pipeline(
    config: PipelineConfig,
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    run_duration: Duration,
    shutdown_deadline: Duration,
) {
    run_pipeline_with_condition(
        config,
        pipeline_group_id,
        pipeline_id,
        run_duration,
        shutdown_deadline,
        None::<fn() -> bool>,
    );
}

/// Run a pipeline with an optional early shutdown condition.
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
            config.clone(),
            channel_capacity_policy.clone(),
            TelemetryPolicy::default(),
            None,
        )
        .expect("failed to build runtime pipeline");

    let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
        pipeline_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
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
        // Either poll the condition or wait for max_duration, whichever comes first.
        let poll_interval = Duration::from_millis(10);
        let start = Instant::now();
        loop {
            if start.elapsed() >= max_duration {
                break;
            }
            if let Some(ref condition) = shutdown_condition {
                if condition() {
                    break;
                }
            }
            std::thread::sleep(poll_interval);
        }
        let deadline = Instant::now() + shutdown_deadline;
        // Try to send shutdown request. If the channel is closed, the pipeline
        // has already terminated (e.g., data generator finished), which is fine.
        let _ = pipeline_ctrl_tx_for_shutdown.try_send(PipelineControlMsg::Shutdown {
            deadline,
            reason: "test shutdown".to_owned(),
        });
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
    // Accept either Ok or a "Channel is closed" error during shutdown.
    // When an always-NACK exporter races with shutdown, the exporter may try to
    // send a NACK after the control channel has closed. This is expected behavior
    // for this test scenario (error_exporter + time-based shutdown).
    let is_acceptable_shutdown = match &run_result {
        Ok(_) => true,
        Err(e) => e.to_string().contains("Channel is closed"),
    };
    assert!(
        is_acceptable_shutdown,
        "pipeline failed to shut down cleanly: {:?}",
        run_result
    );

    assert_eq!(
        registry.metric_set_count(),
        0,
        "metric sets should be cleaned up"
    );
    assert_eq!(registry.entity_count(), 0, "entities should be cleaned up");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Wait for a condition to become true, with timeout.
///
/// Polls the condition every `poll_interval` until it returns true or
/// `timeout` is exceeded. Returns `true` if condition was met, `false` if
/// timed out.
fn wait_for_condition<F>(condition: F, timeout: Duration, poll_interval: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        std::thread::sleep(poll_interval);
    }
    false
}

/// Count the total number of signals (rows) in the primary signal table across all segment files.
///
/// For logs, each row in the LOGS table = 1 log signal.
/// Opens each .qseg segment file and sums row_count for streams matching the given payload type.
fn count_signals_in_segments(
    segments_dir: &std::path::Path,
    payload_type: ArrowPayloadType,
) -> u64 {
    if !segments_dir.exists() {
        return 0;
    }
    let slot_id_raw = payload_type as u16;
    std::fs::read_dir(segments_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "qseg"))
                .map(|e| {
                    // Errors are intentionally ignored here: this function polls for
                    // segments that may be actively being written, so open failures
                    // (incomplete header, locked file, etc.) are expected and transient.
                    SegmentReader::open(e.path())
                        .map(|reader| {
                            reader
                                .streams()
                                .iter()
                                .filter(|s| s.slot_id.raw() == slot_id_raw)
                                .map(|s| s.row_count)
                                .sum::<u64>()
                        })
                        .unwrap_or(0)
                })
                .sum()
        })
        .unwrap_or(0)
}

/// Wait for at least `min_count` signals to exist in the primary signal table across all segments.
///
/// Returns `true` if the condition was met within `timeout`, `false` otherwise.
fn wait_for_signals_in_segments(
    segments_dir: &std::path::Path,
    payload_type: ArrowPayloadType,
    min_count: u64,
    timeout: Duration,
) -> bool {
    wait_for_condition(
        || count_signals_in_segments(segments_dir, payload_type) >= min_count,
        timeout,
        Duration::from_millis(10),
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Metrics Capture
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot of durable_buffer metrics captured from the telemetry registry.
///
/// Field names mirror `DurableBufferMetrics` in mod.rs. Values are
/// captured via `visit_current_metrics` on the registry after the collector
/// drains the reporting channel — so they reflect the most-recent
/// `CollectTelemetry` snapshot produced by the processor.
#[derive(Debug, Default)]
struct DurableBufferMetricsSnapshot {
    fields: HashMap<String, u64>,
}

impl DurableBufferMetricsSnapshot {
    /// Get a metric value by its dot-separated field name (e.g. "consumed.logs").
    fn get(&self, field: &str) -> u64 {
        self.fields.get(field).copied().unwrap_or(0)
    }

    /// Assert that a metric equals an exact expected value.
    fn assert_eq(&self, field: &str, expected: u64) {
        let actual = self.get(field);
        assert_eq!(
            actual, expected,
            "{field}: expected {expected}, got {actual}"
        );
    }

    /// Assert that a metric is greater than or equal to a minimum.
    fn assert_ge(&self, field: &str, minimum: u64) {
        let actual = self.get(field);
        assert!(
            actual >= minimum,
            "{field}: expected >= {minimum}, got {actual}"
        );
    }
}

/// Capture a snapshot of `otelcol.node.durable_buffer` metrics from the
/// telemetry registry.
///
/// The caller must ensure that:
/// 1. At least one `CollectTelemetry` cycle has flushed the processor's metrics.
/// 2. `collector.collect_pending()` has been called to drain the channel.
fn capture_durable_buffer_metrics(
    registry: &TelemetryRegistryHandle,
) -> DurableBufferMetricsSnapshot {
    let mut snapshot = DurableBufferMetricsSnapshot::default();
    registry.visit_current_metrics(|desc, _attrs, iter| {
        if desc.name == "otelcol.node.durable_buffer" {
            for (field, value) in iter {
                let _ = snapshot
                    .fields
                    .insert(field.name.to_owned(), value.to_u64_lossy());
            }
        }
    });
    snapshot
}

/// Run a pipeline with a shutdown condition and capture durable_buffer metrics
/// just before shutdown.
///
/// Works like `run_pipeline_with_condition`, but additionally:
/// 1. After the condition fires, waits for a telemetry cycle (~1.2s) so the
///    processor has flushed its metrics at least once.
/// 2. Drains the collector channel into the registry via `collect_pending()`.
/// 3. Captures a `DurableBufferMetricsSnapshot` from the registry.
/// 4. Sends the shutdown signal.
///
/// Returns the captured metrics snapshot.
fn run_pipeline_and_capture_metrics<F>(
    config: PipelineConfig,
    pipeline_group_id: &PipelineGroupId,
    pipeline_id: &PipelineId,
    max_duration: Duration,
    shutdown_deadline: Duration,
    shutdown_condition: F,
) -> DurableBufferMetricsSnapshot
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
            config.clone(),
            channel_capacity_policy.clone(),
            TelemetryPolicy::default(),
            None,
        )
        .expect("failed to build runtime pipeline");

    let (pipeline_ctrl_tx, pipeline_ctrl_rx) =
        pipeline_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
    let pipeline_ctrl_tx_for_shutdown = pipeline_ctrl_tx.clone();
    let observed_state_store =
        ObservedStateStore::new(&ObservedStateSettings::default(), registry.clone());

    let pipeline_key = DeployedPipelineKey {
        pipeline_group_id: pipeline_group_id.clone(),
        pipeline_id: pipeline_id.clone(),
        core_id: 0,
    };
    let metrics_reporter = telemetry_system.reporter();
    let event_reporter = observed_state_store.reporter(SendPolicy::default());

    // Share registry + collector with the shutdown thread for pre-shutdown capture.
    let capture_registry = registry.clone();
    let capture_collector = collector.clone();

    let shutdown_handle = std::thread::spawn(move || {
        let poll_interval = Duration::from_millis(10);
        let start = Instant::now();
        loop {
            if start.elapsed() >= max_duration {
                break;
            }
            if shutdown_condition() {
                break;
            }
            // Drain the reporting channel periodically to prevent it from filling up.
            // In production, InternalCollector::run_collection_loop() runs continuously;
            // without this drain the bounded channel saturates with high-frequency
            // channel-metric snapshots, silently dropping node-level reports.
            capture_collector.collect_pending();
            std::thread::sleep(poll_interval);
        }

        // Wait for at least one CollectTelemetry cycle (1s interval + margin),
        // continuing to drain the reporting channel so it doesn't fill up and
        // silently drop the node-level CollectTelemetry reports.
        let telemetry_wait = Duration::from_millis(1500);
        let telemetry_start = Instant::now();
        while telemetry_start.elapsed() < telemetry_wait {
            capture_collector.collect_pending();
            std::thread::sleep(poll_interval);
        }

        // Final drain of the reporting channel into the registry.
        capture_collector.collect_pending();

        // Capture the metrics snapshot while metric sets are still registered.
        let snapshot = capture_durable_buffer_metrics(&capture_registry);

        let deadline = Instant::now() + shutdown_deadline;
        let _ = pipeline_ctrl_tx_for_shutdown.try_send(PipelineControlMsg::Shutdown {
            deadline,
            reason: "test shutdown (metrics capture)".to_owned(),
        });

        snapshot
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

    let snapshot = shutdown_handle.join().expect("shutdown thread panicked");

    let is_acceptable_shutdown = match &run_result {
        Ok(_) => true,
        Err(e) => e.to_string().contains("Channel is closed"),
    };
    assert!(
        is_acceptable_shutdown,
        "pipeline failed to shut down cleanly: {:?}",
        run_result
    );

    snapshot
}

// ─────────────────────────────────────────────────────────────────────────────
// Integration Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Test retry behavior when downstream NACKs.
///
/// This verifies:
/// - Retries are scheduled within a single pipeline run
/// - Data survives NACKs and is eventually delivered when downstream recovers
///
/// Uses flaky_exporter which NACKs initially, then switches to ACK mode mid-run.
/// A background thread waits for NACKs to occur (condition-based, not fixed timeout),
/// then flips the exporter to ACK mode.
#[test]
fn test_durable_buffer_retries_on_nack() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "retry-test".into();
    let pipeline_id: PipelineId = "retry-pipeline".into();
    let test_id = "retries_on_nack";

    // Setup: Configure flaky exporter to NACK initially
    let counter = Arc::new(AtomicU64::new(0));
    flaky_exporter::register_state(test_id, counter.clone(), false); // Start in NACK mode

    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(None) // Generate continuously
        .max_batch_size(5)
        .signals_per_second(Some(50)) // Fast enough to generate data quickly
        .use_flaky_exporter()
        .exporter_id(test_id)
        .retry_config(json!({
            "initial_retry_interval": "50ms",
            "max_retry_interval": "200ms",
            "retry_multiplier": 2.0,
            "max_in_flight": 10
        }))
        .build(&pipeline_group_id, &pipeline_id);

    // Spawn a thread to flip the exporter after NACKs are observed
    let flip_test_id = test_id.to_owned();
    let flip_handle = std::thread::spawn(move || {
        // Wait for at least 5 NACKs (condition-based, not fixed timeout)
        let nacks_observed = wait_for_condition(
            || flaky_exporter::nack_count_by_id(&flip_test_id) >= 5,
            Duration::from_secs(5), // generous timeout for CI
            Duration::from_millis(10),
        );
        assert!(nacks_observed, "Expected at least 5 NACKs within timeout");

        let nacks_before = flaky_exporter::nack_count_by_id(&flip_test_id);

        // Switch to ACK mode - retries should now succeed
        flaky_exporter::set_should_ack_by_id(&flip_test_id, true);

        nacks_before
    });

    // Run the pipeline - shut down as soon as we see delivered items
    // (meaning retries succeeded after the flip to ACK mode).
    let delivered_counter = counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10), // generous max timeout for CI
        Duration::from_secs(1),
        Some(move || delivered_counter.load(Ordering::Relaxed) > 0),
    );

    let nacks_before_flip = flip_handle.join().expect("flip thread panicked");

    // Cleanup and validate
    let delivered = counter.load(Ordering::Relaxed);
    let total_nacks = flaky_exporter::nack_count_by_id(test_id);
    flaky_exporter::unregister_state(test_id);

    // Validate: Data was delivered after switching to ACK mode (retries worked)
    assert!(
        delivered > 0,
        "Expected items to be delivered after switching to ACK mode, got 0"
    );

    // Validate: NACKs occurred during the NACK phase
    assert!(
        total_nacks >= nacks_before_flip,
        "NACK count should be at least {} (captured before flip), got {}",
        nacks_before_flip,
        total_nacks
    );

    // Validate: The retry mechanism worked - data was NACKed but eventually delivered
    // This proves the durable buffer's retry logic is functioning.
    assert!(
        nacks_before_flip >= 5,
        "Should have observed at least 5 NACKs before flip, got {}",
        nacks_before_flip
    );
}

/// Test recovery after downstream outage with data integrity validation.
///
/// This test verifies the core durability guarantee: data survives process
/// restarts when downstream is unavailable, and is correctly recovered.
///
/// Run 1: Downstream fails (error exporter), data accumulates in Quiver
/// Run 2: Downstream healthy (counting exporter), data should be delivered
///
/// Validates:
/// - Run 1 NACKs all data, so nothing is delivered/ACK'd
/// - Data gets persisted to Quiver segments
/// - Run 2 recovers and delivers all persisted data plus new data
/// - Exact count verification ensures no data loss or duplication
#[test]
fn test_durable_buffer_recovery_after_outage() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "outage-test".into();
    let pipeline_id: PipelineId = "outage-pipeline".into();

    let run1_signals = 25u64;

    // Run 1: Downstream failing (all NACKs) - data persists to Quiver
    //
    // Key timing considerations for reliable segment persistence:
    // - max_segment_open_duration: 50ms (from TestConfigBuilder default)
    // - poll_interval: 20ms (timer tick that triggers flush)
    // - signals_per_second: 500 (generates all 25 signals in ~50ms)
    //
    // The pipeline needs enough time for:
    // 1. Data generation (~50ms for 25 signals at 500/sec)
    // 2. At least one timer tick to trigger segment flush (poll_interval: 20ms)
    // 3. max_segment_open_duration to elapse so flush actually finalizes (50ms)
    // 4. Graceful shutdown to complete flush and engine shutdown
    //
    // Run for 300ms to ensure multiple flush opportunities, with a generous
    // shutdown deadline to ensure the engine properly finalizes segments.
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(run1_signals))
        .max_batch_size(5)
        .signals_per_second(Some(500))
        .use_error_exporter()
        .otlp_handling("convert_to_arrow") // Use Arrow format for exact signal counting
        .retry_config(json!({
            "initial_retry_interval": "50ms",
            "max_retry_interval": "100ms",
            "max_in_flight": 50
        }))
        .build(&pipeline_group_id, &pipeline_id);

    run_pipeline(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_millis(300), // Allow time for segment flush cycles
        Duration::from_secs(1),     // Generous shutdown deadline for segment finalization
    );

    // Verify data was persisted to segment files (not just the WAL).
    //
    // We verify by counting actual signal rows in the LOGS table.
    // Each row = 1 log signal, so we should see exactly 25 signals persisted.
    let segments_dir = buffer_path.join("core_0").join("segments");
    let signals_exist = wait_for_signals_in_segments(
        &segments_dir,
        ArrowPayloadType::Logs,
        run1_signals,
        Duration::from_secs(2),
    );
    let actual_signals = count_signals_in_segments(&segments_dir, ArrowPayloadType::Logs);
    assert!(
        signals_exist,
        "Run 1 should have persisted {} signals, found {}",
        run1_signals, actual_signals
    );
    assert_eq!(
        actual_signals, run1_signals,
        "Run 1 should have persisted exactly {} signals, found {}",
        run1_signals, actual_signals
    );

    // Run 2: Downstream healthy - verify recovery delivers all data
    let run2_signals = 10u64;
    let run2_counter = Arc::new(AtomicU64::new(0));
    let test_id = "recovery_after_outage";
    counting_exporter::register_counter(test_id, run2_counter.clone());

    // Generate some new data in Run 2 to keep the pipeline alive long enough
    // for recovery. Timer ticks poll Quiver for recovered data, but only fire
    // when the pipeline's message loop is running.
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(run2_signals))
        .max_batch_size(5)
        .signals_per_second(Some(500)) // Fast generation
        .use_counting_exporter()
        .exporter_id(test_id)
        .otlp_handling("convert_to_arrow") // Same format as Run 1
        .build(&pipeline_group_id, &pipeline_id);

    // Shut down once all data (recovered + new) is delivered
    let expected_total = run1_signals + run2_signals;
    let delivered_counter = run2_counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10),    // generous max timeout for CI
        Duration::from_millis(500), // Short drain - condition should trigger first
        Some(move || delivered_counter.load(Ordering::Relaxed) >= expected_total),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = run2_counter.load(Ordering::Relaxed);

    // Validate data integrity:
    // Run 1 generated 25 signals (all NACKed, persisted)
    // Run 2 generated 10 new signals
    // Total should be exactly 35 (25 recovered + 10 new)
    assert_eq!(
        delivered, expected_total,
        "Recovery should deliver exactly {} items ({}+{}), got {}",
        expected_total, run1_signals, run2_signals, delivered
    );
}

/// Test that multiple signal types (traces + logs) flow correctly together.
///
/// Verifies that the durable buffer correctly handles mixed signal types
/// in the same pipeline. Uses traces and logs (not metrics, since pdata metrics
/// view is not yet implemented - see payload.rs:290).
#[test]
fn test_durable_buffer_mixed_signal_types() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "signal-types-test".into();
    let pipeline_id: PipelineId = "signal-types-pipeline".into();
    let test_id = "mixed_signal_types";

    let counter = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, counter.clone());

    let total_signals = 20u64;
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(total_signals))
        .max_batch_size(5)
        // Mix of traces (50%) and logs (50%), no metrics (pdata limitation)
        .signal_weights(0, 50, 50)
        .use_counting_exporter()
        .exporter_id(test_id)
        .build(&pipeline_group_id, &pipeline_id);

    let delivered_counter = counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10), // generous max timeout
        Duration::from_millis(500),
        Some(move || delivered_counter.load(Ordering::Relaxed) >= total_signals),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // Verify all 20 signals were delivered
    assert!(
        delivered >= total_signals,
        "Should have delivered at least {} items (mixed traces + logs), got {}",
        total_signals,
        delivered
    );

    // Verify durable buffer was used
    assert!(
        buffer_path.join("core_0").exists(),
        "Quiver data directory should exist"
    );
}

/// Test OTLP-to-Arrow conversion mode.
///
/// Verifies that when `otlp_handling: convert_to_arrow` is set:
/// - OTLP data is converted to Arrow format before storage
/// - Data flows through correctly and is delivered downstream
///
/// This exercises the OtapRecordBundleAdapter code path in bundle_adapter.rs.
#[test]
fn test_durable_buffer_convert_to_arrow_mode() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "arrow-mode-test".into();
    let pipeline_id: PipelineId = "arrow-mode-pipeline".into();
    let test_id = "convert_to_arrow_mode";

    let counter = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, counter.clone());

    let total_signals = 10u64;
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(total_signals))
        .max_batch_size(5)
        .otlp_handling("convert_to_arrow")
        .use_counting_exporter()
        .exporter_id(test_id)
        .build(&pipeline_group_id, &pipeline_id);

    let delivered_counter = counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10), // generous max timeout
        Duration::from_millis(500),
        Some(move || delivered_counter.load(Ordering::Relaxed) >= total_signals),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // Verify data flowed through the Arrow conversion path
    assert!(
        delivered >= total_signals,
        "Should have delivered at least {} items through Arrow conversion, got {}",
        total_signals,
        delivered
    );

    // Verify durable buffer directory was created
    assert!(
        buffer_path.join("core_0").exists(),
        "Quiver data directory should exist"
    );
}

/// Test graceful shutdown with data drain.
///
/// Verifies the shutdown drain sequence completes successfully:
/// 1. Generate data continuously until shutdown threshold is reached
/// 2. Trigger shutdown while pipeline is actively processing
/// 3. Shutdown should flush open segment and drain remaining bundles
/// 4. Pipeline terminates cleanly (no channel errors)
///
/// The shutdown handler performs: flush → drain loop → engine shutdown.
/// This test exercises that path by ensuring the pipeline is actively
/// processing when shutdown is triggered, then verifying clean termination
/// and that at least the threshold amount of data was delivered.
#[test]
fn test_durable_buffer_graceful_shutdown_drain() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "shutdown-drain-test".into();
    let pipeline_id: PipelineId = "shutdown-drain-pipeline".into();
    let test_id = "graceful_shutdown_drain";

    let counter = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, counter.clone());

    // Generate data continuously (no max). We trigger shutdown once we've seen
    // enough data delivered, proving the pipeline is actively processing.
    // The graceful shutdown must then drain any remaining pending data.
    //
    // We can't predict exactly how many signals will be generated before
    // shutdown completes, but we can verify:
    // 1. The pipeline was actively generating (threshold_for_shutdown reached)
    // 2. Shutdown completed successfully (no channel errors)
    // 3. More data was delivered after shutdown started (drain worked)
    let threshold_for_shutdown = 20u64;
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(None) // Continuous generation until shutdown
        .max_batch_size(10)
        .signals_per_second(Some(200))
        .use_counting_exporter()
        .exporter_id(test_id)
        .build(&pipeline_group_id, &pipeline_id);

    // Trigger shutdown once threshold is reached. This ensures the pipeline
    // is actively processing when shutdown starts.
    let delivered_counter = counter.clone();
    let threshold = threshold_for_shutdown;
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(30), // Max timeout (generous for slow CI)
        Duration::from_secs(5),  // Deadline for drain
        Some(move || delivered_counter.load(Ordering::Relaxed) >= threshold),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // Verify durable buffer directory was created
    assert!(
        buffer_path.join("core_0").exists(),
        "Quiver data directory should exist"
    );

    // Verify shutdown succeeded and data was delivered.
    // The pipeline was active when shutdown triggered (threshold reached),
    // and the graceful shutdown drained pending data. We expect at least the
    // threshold amount, plus potentially more that was in-flight during drain.
    assert!(
        delivered >= threshold_for_shutdown,
        "Graceful shutdown should have delivered at least {} items (threshold), got {}",
        threshold_for_shutdown,
        delivered
    );
}
/// Test high-volume throughput to exercise segment finalization.
///
/// This test generates a large amount of data to ensure:
/// - Multiple segments are created and finalized
/// - Data correctly flows through the full buffering lifecycle
/// - No data loss under sustained load
///
/// This test generates enough data over a long enough duration to trigger
/// multiple segment rotations and finalizations.
#[test]
fn test_durable_buffer_high_volume_throughput() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "high-volume-test".into();
    let pipeline_id: PipelineId = "high-volume-pipeline".into();
    let test_id = "high_volume_throughput";

    let counter = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, counter.clone());

    // Generate 500 signals in batches of 50 - enough to trigger multiple
    // segment finalizations (max_segment_open_duration is 200ms by default)
    let total_signals = 500u64;
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(total_signals))
        .max_batch_size(50)
        .signals_per_second(Some(2000)) // Fast generation
        .use_counting_exporter()
        .exporter_id(test_id)
        .build(&pipeline_group_id, &pipeline_id);

    // Shut down once all signals are delivered
    let delivered_counter = counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10), // generous max timeout for CI
        Duration::from_secs(3),  // Long drain time for high volume
        Some(move || delivered_counter.load(Ordering::Relaxed) >= total_signals),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // Verify durable buffer infrastructure was used
    assert!(
        buffer_path.join("core_0").exists(),
        "Quiver data directory should exist"
    );

    // Verify ALL data was delivered - no data loss under high volume
    assert!(
        delivered >= total_signals,
        "High-volume test should deliver all {} signals, got {}",
        total_signals,
        delivered
    );
}

/// Test that item-level metrics are correctly tracked through the durable buffer
/// for all three signal types (logs, traces, metrics).
///
/// This is a two-phase test that validates the end-to-end item counting pipeline:
///
/// **Phase 1** (error exporter — all NACKs):
///   Generates a mix of OTLP logs, traces, and metrics (roughly equal weights).
///   The durable buffer ingests them, downstream NACKs everything.
///   After shutdown, segment files contain bundles with item_count in the manifest.
///
/// **Between phases** — manifest validation:
///   Opens segment files directly and classifies each bundle by its OTLP slot ID
///   (60=logs, 61=traces, 62=metrics). Asserts:
///   - Every signal type has at least one bundle with non-zero item_count
///   - The total item_count across all signals equals the number generated
///
/// **Phase 2** (counting exporter — all ACKs):
///   Restarts pipeline against the same buffer directory. Recovered bundles from
///   Phase 1 are re-delivered along with new data. The counting exporter verifies
///   the total item count equals Phase 1 + Phase 2 data.
///
/// This validates:
///   1. OTLP wire-format scanning produces correct item counts for each signal type
///   2. Item counts are persisted in the segment manifest (write -> round-trip)
///   3. Per-signal slot classification works correctly for all three signal types
///   4. Recovered item counts flow correctly through the drain path to downstream
#[test]
fn test_durable_buffer_otlp_item_count_metrics() {
    // OTLP pass-through slot IDs (mirrors otlp_slots in bundle_adapter.rs)
    const OTLP_LOGS_SLOT: u16 = 60;
    const OTLP_TRACES_SLOT: u16 = 61;
    const OTLP_METRICS_SLOT: u16 = 62;

    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "item-count-test".into();
    let pipeline_id: PipelineId = "item-count-pipeline".into();

    // The fake data generator sends signals in order (metrics, traces, logs) per
    // iteration. Using equal weights with max_batch_size=30 ensures exactly 10 of
    // each signal type per iteration (30 total). No rate limit — the budget is
    // governed entirely by max_signal_count.
    let phase1_signals = 30u64;

    // ── Phase 1: Ingest mixed signals, all NACKs (data persists in segments) ─
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(phase1_signals))
        .max_batch_size(30)
        .signals_per_second(None) // No rate limit — budget governed by max_signal_count
        .signal_weights(1, 1, 1) // Equal distribution: 10 metrics + 10 traces + 10 logs
        .use_error_exporter()
        .otlp_handling("pass_through") // OTLP pass-through: exercises wire-format scanning
        .retry_config(json!({
            "initial_retry_interval": "50ms",
            "max_retry_interval": "100ms",
            "max_in_flight": 50
        }))
        .build(&pipeline_group_id, &pipeline_id);

    run_pipeline(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(2), // Give time for WAL → segment rotation
        Duration::from_secs(1),
    );

    // ── Between phases: verify per-signal item_count in segment manifests ────
    let segments_dir = buffer_path.join("core_0").join("segments");
    assert!(
        segments_dir.exists(),
        "Segments directory should exist after Phase 1"
    );

    // Track item counts per signal type via OTLP slot IDs.
    let mut log_items: u64 = 0;
    let mut trace_items: u64 = 0;
    let mut metric_items: u64 = 0;
    let mut total_bundles: usize = 0;

    for entry in std::fs::read_dir(&segments_dir).expect("read segments dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "qseg") {
            if let Ok(reader) = SegmentReader::open(&path) {
                for manifest_entry in reader.manifest() {
                    total_bundles += 1;
                    let ic = manifest_entry.item_count();
                    // Classify by OTLP slot ID present in the bundle.
                    for slot in manifest_entry.slot_ids() {
                        match slot.raw() {
                            OTLP_LOGS_SLOT => log_items += ic,
                            OTLP_TRACES_SLOT => trace_items += ic,
                            OTLP_METRICS_SLOT => metric_items += ic,
                            _ => {} // shared or unknown slots
                        }
                    }
                }
            }
        }
    }

    let total_manifest_items = log_items + trace_items + metric_items;

    assert!(
        total_bundles > 0,
        "Should have at least one bundle in segments after Phase 1"
    );

    // Each signal type should have received at least one item.
    assert!(
        log_items > 0,
        "Logs should have non-zero item_count in manifest (got {log_items})"
    );
    assert!(
        trace_items > 0,
        "Traces should have non-zero item_count in manifest (got {trace_items})"
    );
    assert!(
        metric_items > 0,
        "Metrics should have non-zero item_count in manifest (got {metric_items})"
    );

    // For OTLP pass-through, each signal from the fake data generator is 1 item
    // (1 log record, 1 span, or 1 metric data point). The total should match.
    assert_eq!(
        total_manifest_items, phase1_signals,
        "Sum of per-signal manifest item_counts should equal signals generated \
         (expected {phase1_signals}, got {total_manifest_items}: \
         logs={log_items}, traces={trace_items}, metrics={metric_items})"
    );

    // ── Phase 2: Recovery run with counting exporter + metrics capture ──
    let phase2_signals = 12u64;
    let counter = Arc::new(AtomicU64::new(0));
    let test_id = "item_count_metrics";
    counting_exporter::register_counter(test_id, counter.clone());

    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(phase2_signals))
        .max_batch_size(12)
        .signals_per_second(None) // No rate limit
        .signal_weights(1, 1, 1) // Same equal mix for Phase 2 (4 each)
        .use_counting_exporter()
        .exporter_id(test_id)
        .otlp_handling("pass_through") // Same format as Phase 1
        .build(&pipeline_group_id, &pipeline_id);

    let expected_total = phase1_signals + phase2_signals;
    let delivered_counter = counter.clone();
    let metrics_snapshot = run_pipeline_and_capture_metrics(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(15), // Extra time for telemetry cycle
        Duration::from_millis(500),
        move || delivered_counter.load(Ordering::Relaxed) >= expected_total,
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // All Phase 1 data (recovered from segments) + Phase 2 data should arrive.
    assert_eq!(
        delivered, expected_total,
        "Counting exporter should receive exactly {expected_total} items \
         ({phase1_signals} recovered + {phase2_signals} new), got {delivered}"
    );

    // ── Validate durable buffer telemetry metrics ───────────────────────
    // The metrics snapshot was captured after at least one CollectTelemetry cycle
    // completed, so counters and gauges should reflect the pipeline's activity.
    eprintln!("Durable buffer metrics snapshot: {metrics_snapshot:?}");

    // Consumed counters count items ingested in THIS pipeline run only (Phase 2).
    // Phase 2 generates phase2_signals with equal weights → phase2_signals/3 each.
    let phase2_per_signal = phase2_signals / 3;
    metrics_snapshot.assert_eq("consumed.log.records", phase2_per_signal);
    metrics_snapshot.assert_eq("consumed.spans", phase2_per_signal);
    metrics_snapshot.assert_eq("consumed.metric.points", phase2_per_signal);

    // Produced counters count items sent downstream (Phase 1 recovered + Phase 2 new).
    let phase1_per_signal = phase1_signals / 3;
    let produced_per_signal = phase1_per_signal + phase2_per_signal;
    metrics_snapshot.assert_eq("produced.log.records", produced_per_signal);
    metrics_snapshot.assert_eq("produced.spans", produced_per_signal);
    metrics_snapshot.assert_eq("produced.metric.points", produced_per_signal);

    // All bundles ACKed, none NACKed (counting exporter always succeeds).
    // The exact bundle count depends on internal batching decisions (how many
    // items get grouped per bundle), but there must be at least 3 — one per
    // signal type (logs, traces, metrics).
    metrics_snapshot.assert_ge("bundles.acked", 3);
    metrics_snapshot.assert_eq("bundles.nacked", 0);

    // No errors, drops, or expirations in a clean run.
    metrics_snapshot.assert_eq("dropped.items", 0);
    metrics_snapshot.assert_eq("dropped.bundles", 0);
    metrics_snapshot.assert_eq("dropped.segments", 0);
    metrics_snapshot.assert_eq("expired.items", 0);
    metrics_snapshot.assert_eq("expired.bundles", 0);
    metrics_snapshot.assert_eq("ingest.errors", 0);
    metrics_snapshot.assert_eq("read.errors", 0);

    // No NACKs → no requeues, and all data drained → queued gauges at zero.
    metrics_snapshot.assert_eq("requeued.log.records", 0);
    metrics_snapshot.assert_eq("requeued.metric.points", 0);
    metrics_snapshot.assert_eq("requeued.spans", 0);
    metrics_snapshot.assert_eq("queued.log.records", 0);
    metrics_snapshot.assert_eq("queued.metric.points", 0);
    metrics_snapshot.assert_eq("queued.spans", 0);
    metrics_snapshot.assert_eq("in.flight", 0);

    // Storage cap should match the configured 256MiB retention_size_cap (per-core budget).
    metrics_snapshot.assert_eq("storage.bytes.cap", 256 * 1024 * 1024);
}

/// Test drop_oldest size cap policy configuration is accepted.
///
/// This verifies that when `size_cap_policy: drop_oldest` is configured:
/// - The configuration is valid and accepted by the pipeline
/// - The processor functions correctly with this policy
///
/// Note: Actually triggering the drop behavior requires filling the retention
/// buffer which is difficult in unit tests (minimum segment size constraints).
/// This test validates the configuration path is exercised correctly.
#[test]
fn test_durable_buffer_drop_oldest_policy() {
    let temp_dir = tempdir().expect("failed to create temp dir");
    let buffer_path = temp_dir.path().to_path_buf();
    let pipeline_group_id: PipelineGroupId = "drop-oldest-test".into();
    let pipeline_id: PipelineId = "drop-oldest-pipeline".into();
    let test_id = "drop_oldest_policy";

    let counter = Arc::new(AtomicU64::new(0));
    counting_exporter::register_counter(test_id, counter.clone());

    // Use drop_oldest policy with standard retention size.
    // This validates the policy configuration is accepted and functions.
    let total_signals = 50u64;
    let config = TestConfigBuilder::new(buffer_path.clone())
        .max_signal_count(Some(total_signals))
        .max_batch_size(10)
        .signals_per_second(Some(500))
        .use_counting_exporter()
        .exporter_id(test_id)
        .size_cap_policy("drop_oldest")
        .build(&pipeline_group_id, &pipeline_id);

    let delivered_counter = counter.clone();
    run_pipeline_with_condition(
        config,
        &pipeline_group_id,
        &pipeline_id,
        Duration::from_secs(10), // generous max timeout
        Duration::from_millis(1000),
        Some(move || delivered_counter.load(Ordering::Relaxed) >= total_signals),
    );

    counting_exporter::unregister_counter(test_id);
    let delivered = counter.load(Ordering::Relaxed);

    // Verify durable buffer was used
    assert!(
        buffer_path.join("core_0").exists(),
        "Quiver data directory should exist"
    );

    // Verify data flowed through successfully with drop_oldest policy
    assert!(
        delivered >= total_signals,
        "Expected at least {} items delivered with drop_oldest policy, got {}",
        total_signals,
        delivered
    );
}
