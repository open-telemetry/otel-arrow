// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Task periodically collecting the internal signals emitted by the engine and the pipelines.

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use otap_df_config::pipeline::telemetry::TelemetryConfig;
use tokio::sync::{RwLock, watch};
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::registry::TelemetryRegistryHandle;
use crate::reporter::{
    MetricsCollectionMessage, MetricsCollectorStatus, MetricsFlushHandle, MetricsReporter,
};

/// Internal collector responsible for gathering internal telemetry signals (currently metric
/// sets, also known as multivariate metrics).
pub struct InternalCollector {
    /// The registry where entities and metrics are declared and aggregated.
    registry: TelemetryRegistryHandle,

    /// Ordered snapshots and finite flush barriers from metrics reporters.
    metrics_receiver: flume::Receiver<MetricsCollectionMessage>,

    /// Publishes whether a collector loop is available to service barriers.
    collector_status: watch::Sender<MetricsCollectorStatus>,

    /// Enforces the single-consumer invariant required by FIFO barriers.
    collector_running: Arc<AtomicBool>,

    /// Serializes reliable sends against the final cancellation drain.
    shutdown_gate: Arc<RwLock<()>>,
}

struct CollectorRunGuard {
    collector_status: watch::Sender<MetricsCollectorStatus>,
    collector_running: Arc<AtomicBool>,
}

struct CollectorDrainGuard {
    collector_running: Arc<AtomicBool>,
}

impl CollectorDrainGuard {
    fn start(collector_running: &Arc<AtomicBool>) -> Option<Self> {
        collector_running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self {
                collector_running: collector_running.clone(),
            })
    }
}

impl Drop for CollectorDrainGuard {
    fn drop(&mut self) {
        self.collector_running.store(false, Ordering::Release);
    }
}

impl CollectorRunGuard {
    fn start(
        collector_status: &watch::Sender<MetricsCollectorStatus>,
        collector_running: &Arc<AtomicBool>,
    ) -> Result<Self, Error> {
        let _ = collector_running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::MetricsCollectorAlreadyRunning)?;
        let collector_status = collector_status.clone();
        let _ = collector_status.send_replace(MetricsCollectorStatus::Running);
        Ok(Self {
            collector_status,
            collector_running: collector_running.clone(),
        })
    }
}

impl Drop for CollectorRunGuard {
    fn drop(&mut self) {
        let _ = self
            .collector_status
            .send_replace(MetricsCollectorStatus::Stopped);
        self.collector_running.store(false, Ordering::Release);
    }
}

impl InternalCollector {
    /// Creates a collector and the reporter that sends metric-set snapshots to it.
    pub(crate) fn new(
        config: &TelemetryConfig,
        registry: TelemetryRegistryHandle,
    ) -> (Self, MetricsReporter) {
        let (metrics_sender, metrics_receiver) =
            flume::bounded::<MetricsCollectionMessage>(config.reporting_channel_size);
        let (collector_status, collector_status_rx) =
            watch::channel(MetricsCollectorStatus::NotStarted);
        let shutdown_gate = Arc::new(RwLock::new(()));
        let metrics_flusher = MetricsFlushHandle::new(
            metrics_sender.clone(),
            collector_status_rx,
            shutdown_gate.clone(),
        );
        registry.configure_metrics_collector(metrics_flusher.clone());

        let reporter = MetricsReporter::new_collector(metrics_flusher);
        (
            Self {
                registry,
                metrics_receiver,
                collector_status,
                collector_running: Arc::new(AtomicBool::new(false)),
                shutdown_gate,
            },
            reporter,
        )
    }

    fn collect_message(&self, message: MetricsCollectionMessage) {
        match message {
            MetricsCollectionMessage::Snapshot(metrics) => self
                .registry
                .accumulate_metric_set_snapshot(metrics.key, metrics.bucket, &metrics.metrics),
            MetricsCollectionMessage::Flush(ack_sender) => {
                let _ = ack_sender.send(());
            }
        }
    }

    fn drain_pending(&self) {
        while let Ok(message) = self.metrics_receiver.try_recv() {
            self.collect_message(message);
        }
    }

    /// Drains all currently pending metrics without starting a long-lived task.
    ///
    /// If another collector consumer is active, this call is a no-op so FIFO
    /// barrier ordering cannot be split across consumers.
    pub fn collect_pending(&self) {
        let Some(_running) = CollectorDrainGuard::start(&self.collector_running) else {
            return;
        };
        self.drain_pending();
    }

    async fn collection_loop(&self) -> Result<(), Error> {
        while let Ok(message) = self.metrics_receiver.recv_async().await {
            self.collect_message(message);
        }
        Ok(())
    }

    /// Collects metrics from the reporting channel and aggregates them into the `registry`.
    /// The collection runs indefinitely until the metrics channel is closed.
    pub fn run_collection_loop(self: Arc<Self>) -> impl Future<Output = Result<(), Error>> {
        let running = CollectorRunGuard::start(&self.collector_status, &self.collector_running);
        async move {
            let _running = running?;
            self.collection_loop().await
        }
    }

    /// Runs the collection loop until cancellation is requested.
    ///
    /// This method starts the internal signal collection loop and listens for a shutdown signal.
    /// It returns when either the collection loop ends (Ok/Err) or the shutdown signal fires.
    pub fn run(
        self: Arc<Self>,
        cancel: CancellationToken,
    ) -> impl Future<Output = Result<(), Error>> {
        let running = CollectorRunGuard::start(&self.collector_status, &self.collector_running);
        async move {
            let _running = running?;
            tokio::select! {
                biased;

                _ = cancel.cancelled() => {
                    // Queue the exclusive shutdown gate while continuing to
                    // consume. A reliable sender may already hold a read guard
                    // while blocked on a full channel; consuming lets that send
                    // complete and release its guard instead of deadlocking the
                    // final drain.
                    let shutdown_guard = self.shutdown_gate.clone().write_owned();
                    tokio::pin!(shutdown_guard);
                    let _shutdown_guard = loop {
                        tokio::select! {
                            biased;

                            guard = &mut shutdown_guard => break guard,
                            message = self.metrics_receiver.recv_async() => {
                                match message {
                                    Ok(message) => self.collect_message(message),
                                    Err(_) => break shutdown_guard.await,
                                }
                            }
                        }
                    };
                    let _ = self
                        .collector_status
                        .send_replace(MetricsCollectorStatus::Stopped);
                    // Preserve snapshots already queued before shutdown so a final
                    // registry drain can export them.
                    self.drain_pending();
                    Ok(())
                }
                res = self.collection_loop() => {
                    res
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use otap_df_config::settings::telemetry::logs::LogsConfig;

    use super::*;
    use crate::attributes::{AttributeSetHandler, AttributeValue};
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, Instrument, MetricValueType,
        MetricsDescriptor, MetricsField, Temporality,
    };
    use crate::metrics::MetricSetHandler;
    use crate::metrics::MetricSetSnapshot;
    use crate::metrics::MetricValue;
    use crate::registry::MetricSetKey;
    use std::collections::HashMap;
    use std::fmt::Debug;
    use std::time::Duration;

    // --- Test-only mock metric/attributes definitions (no pipeline required) ---

    #[derive(Debug)]
    struct MockMetricSet {
        values: Vec<MetricValue>,
    }

    impl MockMetricSet {
        fn new() -> Self {
            Self {
                values: vec![MetricValue::from(0u64), MetricValue::from(0u64)],
            }
        }
    }

    impl Default for MockMetricSet {
        fn default() -> Self {
            Self::new()
        }
    }

    static MOCK_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test_metrics",
        metrics: &[
            MetricsField {
                name: "counter1",
                unit: "1",
                brief: "Test counter 1",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "counter2",
                unit: "1",
                brief: "Test counter 2",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
        ],
    };

    static MOCK_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test_attributes",
        fields: &[AttributeField {
            key: "test_key",
            r#type: AttributeValueType::String,
            brief: "Test attribute",
        }],
    };

    impl MetricSetHandler for MockMetricSet {
        fn descriptor(&self) -> &'static MetricsDescriptor {
            &MOCK_METRICS_DESCRIPTOR
        }
        fn snapshot_values(&self) -> Vec<MetricValue> {
            self.values.clone()
        }
        fn clear_values(&mut self) {
            self.values.iter_mut().for_each(MetricValue::reset);
        }
        fn needs_flush(&self) -> bool {
            self.values.iter().any(|&v| !v.is_zero())
        }
    }

    #[derive(Debug)]
    struct MockAttributeSet {
        _value: String,
        attribute_values: Vec<AttributeValue>,
    }

    impl MockAttributeSet {
        fn new(value: impl Into<String>) -> Self {
            let v = value.into();
            Self {
                _value: v.clone(),
                attribute_values: vec![AttributeValue::String(v)],
            }
        }
    }

    impl AttributeSetHandler for MockAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            &MOCK_ATTRIBUTES_DESCRIPTOR
        }
        fn iter_attributes<'a>(&'a self) -> crate::attributes::AttributeIterator<'a> {
            crate::attributes::AttributeIterator::new(
                MOCK_ATTRIBUTES_DESCRIPTOR.fields,
                &self.attribute_values,
            )
        }
        fn attribute_values(&self) -> &[AttributeValue] {
            &self.attribute_values
        }
    }

    fn create_test_config(reporting_interval_ms: u64) -> TelemetryConfig {
        // Flush interval is irrelevant when no pipeline is configured; keep field for completeness.
        TelemetryConfig {
            reporting_channel_size: 10,
            reporting_interval: Duration::from_millis(reporting_interval_ms),
            logs: LogsConfig::default(),
            resource: HashMap::new(),
            detectors: Vec::new(),
        }
    }

    fn create_test_snapshot(key: MetricSetKey, values: Vec<MetricValue>) -> MetricSetSnapshot {
        MetricSetSnapshot {
            key,
            descriptor: &MOCK_METRICS_DESCRIPTOR,
            measurement_attributes: &[],
            bucket: 0,
            metrics: values,
        }
    }

    fn create_test_registry() -> TelemetryRegistryHandle {
        TelemetryRegistryHandle::new()
    }

    // --- Tests without any pipeline, asserting on the registry state ---

    /// Scenario: the always-on collector is started without any queued snapshots.
    /// Guarantees: explicit cancellation stops it cleanly despite the retained flush handle.
    #[tokio::test]
    async fn test_collector_without_snapshots_stops_on_cancellation() {
        let config = create_test_config(100);
        let telemetry_registry = create_test_registry();
        let (collector, _reporter) = InternalCollector::new(&config, telemetry_registry);
        let cancel = CancellationToken::new();
        let handle = tokio::spawn(Arc::new(collector).run(cancel.clone()));

        cancel.cancel();
        handle
            .await
            .expect("collector task should join")
            .expect("collector should stop cleanly");
    }

    #[tokio::test]
    async fn test_accumulates_snapshots_into_registry() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();

        // Register a metric set to get a valid key
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());

        let cancel = CancellationToken::new();
        let handle = tokio::spawn(Arc::new(collector).run(cancel.clone()));

        // Send two snapshots that should be accumulated: [10,20] + [5,15] => [15,35]
        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(10u64), MetricValue::from(20u64)],
            ))
            .await
            .unwrap();
        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(5u64), MetricValue::from(15u64)],
            ))
            .await
            .unwrap();

        reporter
            .flush()
            .await
            .expect("collector should aggregate both snapshots before inspection");

        // Inspect current metrics without resetting
        let mut collected = Vec::new();
        telemetry_registry.visit_current_metrics(|_desc, _attrs, iter| {
            for (field, value) in iter {
                collected.push((field.name, value));
            }
        });

        assert_eq!(collected.len(), 2);
        // Order follows descriptor order
        assert_eq!(collected[0].0, "counter1");
        assert_eq!(collected[0].1, MetricValue::from(15u64));
        assert_eq!(collected[1].0, "counter2");
        assert_eq!(collected[1].1, MetricValue::from(35u64));

        cancel.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_visit_then_reset_via_registry_api() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;

        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let cancel = CancellationToken::new();
        let handle = tokio::spawn(Arc::new(collector).run(cancel.clone()));

        reporter
            .report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(7u64), MetricValue::from(0u64)],
            ))
            .await
            .unwrap();
        reporter
            .flush()
            .await
            .expect("collector should aggregate the snapshot before inspection");

        // First visit should see the non-zero and then reset
        let mut first = Vec::new();
        telemetry_registry.visit_metrics_and_reset(|_d, _a, iter| {
            for (f, v) in iter {
                first.push((f.name, v));
            }
        });
        assert_eq!(
            first,
            vec![
                ("counter1", MetricValue::from(7u64)),
                ("counter2", MetricValue::from(0u64))
            ]
        );

        // Second visit should see nothing
        let mut count = 0;
        telemetry_registry.visit_metrics_and_reset(|_, _, _| {
            count += 1;
        });
        assert_eq!(count, 0);

        cancel.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_flush_barrier_aggregates_queued_snapshots() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let cancel = CancellationToken::new();
        let handle = tokio::spawn(Arc::new(collector).run(cancel.clone()));

        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(7u64), MetricValue::from(9u64)],
            ))
            .expect("snapshot fits in the reporting channel");
        reporter
            .flush()
            .await
            .expect("flush barrier should be acknowledged");

        let batch = telemetry_registry.drain_metric_export_batch();
        assert_eq!(batch.metric_sets.len(), 1);
        assert_eq!(
            batch.metric_sets[0].values,
            vec![MetricValue::from(7u64), MetricValue::from(9u64)]
        );

        cancel.cancel();
        handle.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_flush_barrier_fails_when_retained_collector_is_stopped() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry);
        let collector = Arc::new(collector);
        let retained_collector = collector.clone();
        let cancel = CancellationToken::new();
        let handle = tokio::spawn(collector.run(cancel.clone()));

        reporter
            .flush()
            .await
            .expect("running collector should acknowledge a barrier");
        cancel.cancel();
        handle
            .await
            .expect("collector task should join")
            .expect("collector should stop cleanly");

        let error = reporter
            .flush()
            .await
            .expect_err("a retained but stopped collector must not leave flush hanging");
        assert!(matches!(error, Error::MetricsCollectorNotRunning));

        drop(retained_collector);
    }

    #[tokio::test]
    async fn test_flush_barrier_fails_before_collector_starts() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let (_collector, reporter) = InternalCollector::new(&config, telemetry_registry);

        let error = reporter
            .flush()
            .await
            .expect_err("an unstarted collector must fail instead of hanging");
        assert!(matches!(error, Error::MetricsCollectorNotRunning));
    }

    #[tokio::test]
    async fn test_collect_pending_supports_nonblocking_manual_drain_mode() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());

        let outcome = reporter
            .report_snapshot_reliably(create_test_snapshot(
                key,
                vec![MetricValue::from(7u64), MetricValue::from(9u64)],
            ))
            .await
            .expect("manual mode accepts snapshots before a long-lived collector starts");
        assert_eq!(outcome, crate::reporter::ReportOutcome::Sent);

        collector.collect_pending();
        let batch = telemetry_registry.drain_metric_export_batch();
        assert_eq!(batch.metric_sets.len(), 1);
        assert_eq!(
            batch.metric_sets[0].values,
            vec![MetricValue::from(7u64), MetricValue::from(9u64)]
        );
    }

    #[tokio::test]
    async fn test_second_collector_consumer_is_rejected() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let (collector, _reporter) = InternalCollector::new(&config, telemetry_registry);
        let collector = Arc::new(collector);

        let first_consumer = collector.clone().run_collection_loop();
        let error = collector
            .run_collection_loop()
            .await
            .expect_err("only one collector consumer may run at a time");
        assert!(matches!(error, Error::MetricsCollectorAlreadyRunning));
        drop(first_consumer);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_reliable_snapshot_waits_for_full_collector_channel() {
        let mut config = create_test_config(10);
        config.reporting_channel_size = 1;
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let collector = Arc::new(collector);

        // Block aggregation after the collector dequeues the first snapshot so
        // the second snapshot deterministically fills the one-slot channel.
        let locked_registry = telemetry_registry.clone();
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let lock_thread = std::thread::spawn(move || {
            let _registry_guard = locked_registry.registry.lock();
            locked_tx.send(()).expect("lock notification receiver");
            release_rx.recv().expect("lock release sender");
        });
        locked_rx.recv().expect("registry lock thread should start");

        let cancel = CancellationToken::new();
        let collector_task = tokio::spawn(collector.clone().run(cancel.clone()));
        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(1u64), MetricValue::from(1u64)],
            ))
            .expect("first snapshot should fit");
        while !collector.metrics_receiver.is_empty() {
            tokio::task::yield_now().await;
        }
        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(2u64), MetricValue::from(2u64)],
            ))
            .expect("second snapshot should fill the channel");

        let error = reporter
            .report_snapshot_reliably_until(
                create_test_snapshot(key, vec![MetricValue::from(3u64), MetricValue::from(3u64)]),
                std::time::Instant::now() + Duration::from_millis(20),
            )
            .await
            .expect_err("reliable reporting must honor its terminal deadline");
        assert!(matches!(error, Error::ShutdownError(_)));

        {
            let reliable_reporter = reporter.clone();
            let reliable_report = async move {
                reliable_reporter
                    .report_snapshot_reliably(create_test_snapshot(
                        key,
                        vec![MetricValue::from(3u64), MetricValue::from(3u64)],
                    ))
                    .await
            };
            tokio::pin!(reliable_report);
            assert!(
                tokio::time::timeout(Duration::from_millis(20), &mut reliable_report)
                    .await
                    .is_err(),
                "reliable terminal reporting should wait instead of dropping a full-channel snapshot"
            );

            release_tx
                .send(())
                .expect("registry lock thread should wait");
            let outcome = reliable_report
                .await
                .expect("reliable snapshot should be accepted after capacity returns");
            assert_eq!(outcome, crate::reporter::ReportOutcome::Sent);
        }
        reporter
            .flush()
            .await
            .expect("collector should reach barrier");

        let batch = telemetry_registry.drain_metric_export_batch();
        assert_eq!(
            batch.metric_sets[0].values,
            vec![MetricValue::from(6u64), MetricValue::from(6u64)]
        );

        lock_thread
            .join()
            .expect("registry lock thread should join");
        cancel.cancel();
        collector_task
            .await
            .expect("collector task should join")
            .expect("collector should stop cleanly");
    }

    #[tokio::test]
    async fn test_cancellation_aggregates_snapshots_already_in_the_channel() {
        let config = create_test_config(10);
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let cancel = CancellationToken::new();
        let collector_task = tokio::spawn(Arc::new(collector).run(cancel.clone()));

        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(7u64), MetricValue::from(9u64)],
            ))
            .expect("snapshot fits in the reporting channel");
        cancel.cancel();

        collector_task
            .await
            .expect("collector task should join")
            .expect("collector shuts down cleanly");

        let batch = telemetry_registry.drain_metric_export_batch();
        assert_eq!(batch.metric_sets.len(), 1);
        assert_eq!(
            batch.metric_sets[0].values,
            vec![MetricValue::from(7u64), MetricValue::from(9u64)]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cancellation_unblocks_reliable_sender_waiting_on_full_channel() {
        let mut config = create_test_config(10);
        config.reporting_channel_size = 1;
        let telemetry_registry = create_test_registry();
        let metric_set: crate::metrics::MetricSet<MockMetricSet> =
            telemetry_registry.register_metric_set(MockAttributeSet::new("attr"));
        let key = metric_set.key;
        let (collector, reporter) = InternalCollector::new(&config, telemetry_registry.clone());
        let collector = Arc::new(collector);

        // Hold the registry lock after the collector dequeues the first
        // snapshot. The second snapshot then fills the channel and the third,
        // reliable send holds the shutdown read gate while waiting for room.
        let locked_registry = telemetry_registry.clone();
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let lock_thread = std::thread::spawn(move || {
            let _registry_guard = locked_registry.registry.lock();
            locked_tx.send(()).expect("lock notification receiver");
            release_rx.recv().expect("lock release sender");
        });
        locked_rx.recv().expect("registry lock thread should start");

        let cancel = CancellationToken::new();
        let collector_task = tokio::spawn(collector.clone().run(cancel.clone()));
        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(1u64), MetricValue::from(1u64)],
            ))
            .expect("first snapshot should fit");
        while !collector.metrics_receiver.is_empty() {
            tokio::task::yield_now().await;
        }
        reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(2u64), MetricValue::from(2u64)],
            ))
            .expect("second snapshot should fill the channel");

        let reliable_reporter = reporter.clone();
        let reliable_task = tokio::spawn(async move {
            reliable_reporter
                .report_snapshot_reliably(create_test_snapshot(
                    key,
                    vec![MetricValue::from(3u64), MetricValue::from(3u64)],
                ))
                .await
        });
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(
            !reliable_task.is_finished(),
            "reliable send should be waiting on the full channel"
        );

        cancel.cancel();
        release_tx
            .send(())
            .expect("registry lock thread should wait");
        let outcome = tokio::time::timeout(Duration::from_secs(1), reliable_task)
            .await
            .expect("reliable send must not deadlock collector cancellation")
            .expect("reliable task should join")
            .expect("in-flight reliable snapshot should be accepted");
        assert_eq!(outcome, crate::reporter::ReportOutcome::Sent);
        tokio::time::timeout(Duration::from_secs(1), collector_task)
            .await
            .expect("collector cancellation must not deadlock")
            .expect("collector task should join")
            .expect("collector should stop cleanly");

        let batch = telemetry_registry.drain_metric_export_batch();
        assert_eq!(
            batch.metric_sets[0].values,
            vec![MetricValue::from(6u64), MetricValue::from(6u64)]
        );
        let error = reporter
            .try_report_snapshot(create_test_snapshot(
                key,
                vec![MetricValue::from(4u64), MetricValue::from(4u64)],
            ))
            .expect_err("non-blocking sends must be rejected after the final drain");
        assert!(matches!(error, Error::MetricsCollectorNotRunning));

        lock_thread
            .join()
            .expect("registry lock thread should join");
    }
}
