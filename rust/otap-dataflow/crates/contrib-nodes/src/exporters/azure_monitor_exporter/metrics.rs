// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Azure Monitor Exporter node.

use std::cell::RefCell;
use std::rc::Rc;

use otap_df_config::SignalType;
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::{
    HttpResponse, Outcome, OutcomeAttributes, SignalRegistrationAttributes,
};
pub use otap_df_telemetry::common_attributes::{
    HttpResponseAttributes, OutcomeAttributes as ExportOutcomeAttributes,
    SignalRegistrationAttributes as ExportSignalAttributes,
};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc, MmscSnapshot};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Shared handle to the metrics tracker.
///
/// The exporter runs on a single-threaded runtime (`#[async_trait(?Send)]`),
/// so `Rc<RefCell<...>>` is sufficient--no `Arc`/`Mutex` overhead needed.
pub type AzureMonitorExporterMetricsRc = Rc<RefCell<AzureMonitorExporterMetricsTracker>>;

/// Metrics without bounded item dimensions.
#[metric_set(name = "exporter.azure_monitor")]
#[derive(Debug, Default, Clone)]
pub struct AzureMonitorExporterOperationalMetrics {
    /// Compressed batch size in bytes (min/max/sum/count).
    /// Recorded once per batch; HTTP retries do not produce additional observations.
    #[metric(unit = "By")]
    pub batch_size: Mmsc,
    /// Uncompressed batch size in bytes (min/max/sum/count).
    /// Recorded once per batch, before compression.
    #[metric(unit = "By")]
    pub batch_uncompressed_size: Mmsc,
    /// Current number of in-flight export requests.
    #[metric(unit = "{export}")]
    pub in_flight_exports: Gauge<u64>,
    /// Current number of log records in-flight at the exporter (enqueued export
    /// requests awaiting completion, including records being retried).
    #[metric(unit = "{log}")]
    pub in_flight_log_records: Gauge<u64>,
    /// Number of log entries rejected for exceeding the batch size limit.
    #[metric(unit = "{entry}")]
    pub log_entries_too_large: Counter<u64>,
}

/// Export completion metrics partitioned by outcome.
#[metric_set(
    name = "exporter.azure_monitor.exports",
    registration_attributes = SignalRegistrationAttributes,
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureMonitorExporterExportMetrics {
    /// Number of items resolved by export outcome.
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
    /// Number of batches resolved by export outcome.
    #[metric(unit = "{batch}")]
    pub batches: Counter<u64>,
    /// Number of messages resolved by export outcome.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

/// HTTP export attempts partitioned by response category.
#[metric_set(
    name = "exporter.azure_monitor.http",
    measurement_attributes = HttpResponseAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureMonitorExporterHttpMetrics {
    /// Number of HTTP export attempts by response category.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
    /// HTTP export attempt latency in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub latency: Mmsc,
}

#[derive(Debug, Clone, Copy, AttributeEnum)]
pub(super) enum StateMapping {
    BatchToMessage,
    MessageToBatch,
    MessageToData,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct StateMappingAttributes {
    mapping: StateMapping,
}

/// Exporter state-map entry counts partitioned by mapping type.
#[metric_set(
    name = "exporter.azure_monitor.state",
    measurement_attributes = StateMappingAttributes
)]
#[derive(Debug, Default, Clone)]
struct AzureMonitorExporterStateMetrics {
    /// Current number of entries in the exporter state mapping.
    #[metric(unit = "{entry}")]
    mappings: Gauge<u64>,
}

/// Heartbeat send metrics partitioned by outcome.
#[metric_set(
    name = "exporter.azure_monitor.heartbeats",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AzureMonitorExporterHeartbeatMetrics {
    /// Number of heartbeat sends resolved by outcome.
    #[metric(unit = "{heartbeat}")]
    pub sends: Counter<u64>,
}

/// Full metrics tracker for the Azure Monitor exporter.
pub struct AzureMonitorExporterMetricsTracker {
    operational_metrics: MetricSet<AzureMonitorExporterOperationalMetrics>,
    export_metrics: MeasurementMetricSet<AzureMonitorExporterExportMetrics>,
    http_metrics: MeasurementMetricSet<AzureMonitorExporterHttpMetrics>,
    state_metrics: MeasurementMetricSet<AzureMonitorExporterStateMetrics>,
    heartbeat_metrics: MeasurementMetricSet<AzureMonitorExporterHeartbeatMetrics>,
}

impl std::fmt::Debug for AzureMonitorExporterMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureMonitorExporterMetricsTracker")
            .finish()
    }
}

impl AzureMonitorExporterMetricsTracker {
    /// Register the exporter's metric sets with the telemetry system.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operational_metrics: AzureMonitorExporterOperationalMetrics::register(pipeline_ctx),
            export_metrics: AzureMonitorExporterExportMetrics::register(
                pipeline_ctx,
                &SignalRegistrationAttributes {
                    signal: SignalType::Logs,
                },
            ),
            http_metrics: AzureMonitorExporterHttpMetrics::register(pipeline_ctx),
            state_metrics: AzureMonitorExporterStateMetrics::register(pipeline_ctx),
            heartbeat_metrics: AzureMonitorExporterHeartbeatMetrics::register(pipeline_ctx),
        }
    }

    /// Report metrics to the telemetry system.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report(&mut self.operational_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.export_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.http_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.state_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.heartbeat_metrics))
    }

    /// Take snapshots of every metric set for terminal state reporting.
    #[must_use]
    pub(super) fn terminal_snapshots(
        &mut self,
    ) -> Vec<otap_df_telemetry::metrics::MetricSetSnapshot> {
        let mut snapshots = self.operational_metrics.terminal_snapshots();
        snapshots.extend(self.export_metrics.terminal_snapshots());
        snapshots.extend(self.http_metrics.terminal_snapshots());
        snapshots.extend(self.state_metrics.terminal_snapshots());
        snapshots.extend(self.heartbeat_metrics.terminal_snapshots());
        snapshots
    }

    #[inline]
    #[must_use]
    pub(super) fn export_for(&self, outcome: Outcome) -> &AzureMonitorExporterExportMetrics {
        self.export_metrics.get(OutcomeAttributes { outcome })
    }

    #[inline]
    #[must_use]
    pub(super) fn http_for(&self, response: HttpResponse) -> &AzureMonitorExporterHttpMetrics {
        self.http_metrics.get(HttpResponseAttributes { response })
    }

    #[inline]
    #[must_use]
    pub(super) fn batch_size(&self) -> MmscSnapshot {
        self.operational_metrics.batch_size.get()
    }

    #[inline]
    pub(super) fn record_export(&mut self, outcome: Outcome, items: u64, messages: u64) {
        let metrics = self.export_metrics.with(OutcomeAttributes { outcome });
        metrics.items.add(items);
        metrics.batches.inc();
        metrics.messages.add(messages);
    }

    #[inline]
    pub(super) fn record_http_attempt(&mut self, response: HttpResponse, latency_ms: f64) {
        let metrics = self.http_metrics.with(HttpResponseAttributes { response });
        metrics.responses.inc();
        metrics.latency.record(latency_ms);
    }

    #[inline]
    pub(super) fn add_batch_size(&mut self, size_bytes: f64) {
        self.operational_metrics.batch_size.record(size_bytes);
    }

    #[inline]
    pub(super) fn add_batch_uncompressed_size(&mut self, size_bytes: f64) {
        self.operational_metrics
            .batch_uncompressed_size
            .record(size_bytes);
    }

    #[inline]
    pub(super) fn set_in_flight_exports(&mut self, count: u64) {
        self.operational_metrics.in_flight_exports.set(count);
    }

    #[inline]
    pub(super) fn set_in_flight_log_records(&mut self, count: u64) {
        self.operational_metrics.in_flight_log_records.set(count);
    }

    #[inline]
    pub(super) fn set_state_mapping(&mut self, mapping: StateMapping, count: u64) {
        self.state_metrics
            .with(StateMappingAttributes { mapping })
            .mappings
            .set(count);
    }

    #[inline]
    pub(super) fn add_log_entry_too_large(&mut self) {
        self.operational_metrics.log_entries_too_large.inc();
    }

    #[inline]
    pub(super) fn record_heartbeat(&mut self, outcome: Outcome) {
        self.heartbeat_metrics
            .with(OutcomeAttributes { outcome })
            .sends
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_tracker() -> AzureMonitorExporterMetricsTracker {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        AzureMonitorExporterMetricsTracker::register(&pipeline_ctx)
    }

    /// Scenario: Export attempts complete with successful and failed outcomes.
    /// Guarantees: Each outcome records items, batches, and messages in its own metric bucket.
    #[test]
    fn export_metrics_are_partitioned_by_outcome() {
        let mut metrics = new_test_tracker();
        metrics.record_export(Outcome::Success, 100, 50);
        metrics.record_export(Outcome::Failure, 10, 5);

        let success = metrics.export_for(Outcome::Success);
        assert_eq!(success.items.get(), 100);
        assert_eq!(success.batches.get(), 1);
        assert_eq!(success.messages.get(), 50);

        let failure = metrics.export_for(Outcome::Failure);
        assert_eq!(failure.items.get(), 10);
        assert_eq!(failure.batches.get(), 1);
        assert_eq!(failure.messages.get(), 5);
    }

    /// Scenario: HTTP attempts receive successful, throttled, and network-error responses.
    /// Guarantees: Attempts and latency are partitioned by their bounded response dimension.
    #[test]
    fn http_attempts_are_partitioned_by_response() {
        let mut metrics = new_test_tracker();
        metrics.record_http_attempt(HttpResponse::Http2xx, 10.0);
        metrics.record_http_attempt(HttpResponse::Http2xx, 20.0);
        metrics.record_http_attempt(HttpResponse::Http400, 25.0);
        metrics.record_http_attempt(HttpResponse::Http429, 30.0);
        metrics.record_http_attempt(HttpResponse::Http404, 35.0);
        metrics.record_http_attempt(HttpResponse::NetworkError, 40.0);

        assert_eq!(metrics.http_for(HttpResponse::Http2xx).responses.get(), 2);
        assert_eq!(
            metrics.http_for(HttpResponse::Http2xx).latency.get().sum,
            30.0
        );
        assert_eq!(metrics.http_for(HttpResponse::Http400).responses.get(), 1);
        assert_eq!(metrics.http_for(HttpResponse::Http429).responses.get(), 1);
        assert_eq!(metrics.http_for(HttpResponse::Http404).responses.get(), 1);
        assert_eq!(
            metrics.http_for(HttpResponse::NetworkError).responses.get(),
            1
        );
    }

    /// Scenario: State mapping gauges are updated for every supported mapping type.
    /// Guarantees: Each mapping type retains its independently reported gauge value.
    #[test]
    fn state_mappings_are_partitioned_by_mapping_type() {
        let mut metrics = new_test_tracker();
        metrics.set_state_mapping(StateMapping::BatchToMessage, 1);
        metrics.set_state_mapping(StateMapping::MessageToBatch, 2);
        metrics.set_state_mapping(StateMapping::MessageToData, 3);

        assert_eq!(
            metrics
                .state_metrics
                .get(StateMappingAttributes {
                    mapping: StateMapping::BatchToMessage,
                })
                .mappings
                .get(),
            1
        );
        assert_eq!(
            metrics
                .state_metrics
                .get(StateMappingAttributes {
                    mapping: StateMapping::MessageToBatch,
                })
                .mappings
                .get(),
            2
        );
        assert_eq!(
            metrics
                .state_metrics
                .get(StateMappingAttributes {
                    mapping: StateMapping::MessageToData,
                })
                .mappings
                .get(),
            3
        );
    }

    /// Scenario: Heartbeat sends succeed and fail.
    /// Guarantees: Heartbeat counts are partitioned by export outcome.
    #[test]
    fn heartbeat_metrics_are_partitioned_by_outcome() {
        let mut metrics = new_test_tracker();
        metrics.record_heartbeat(Outcome::Success);
        metrics.record_heartbeat(Outcome::Failure);
        metrics.record_heartbeat(Outcome::Failure);

        assert_eq!(
            metrics
                .heartbeat_metrics
                .get(OutcomeAttributes {
                    outcome: Outcome::Success,
                })
                .sends
                .get(),
            1
        );
        assert_eq!(
            metrics
                .heartbeat_metrics
                .get(OutcomeAttributes {
                    outcome: Outcome::Failure,
                })
                .sends
                .get(),
            2
        );
    }

    /// Scenario: A measurement metric bucket is recorded and terminal snapshots are requested twice.
    /// Guarantees: The touched bucket is emitted once and cleared after terminal handoff.
    #[test]
    fn terminal_snapshots_include_touched_measurement_metrics() {
        let mut metrics = new_test_tracker();
        metrics.record_export(Outcome::Success, 10, 1);

        let snapshots = metrics.terminal_snapshots();
        let export_snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.descriptor().name == "exporter.azure_monitor.exports")
            .expect("export metrics should be included in terminal snapshots");
        assert_eq!(
            export_snapshot.measurement_attribute_value("outcome"),
            Some("success")
        );

        let next_snapshots = metrics.terminal_snapshots();
        assert!(
            next_snapshots
                .iter()
                .all(|snapshot| snapshot.descriptor().name != "exporter.azure_monitor.exports")
        );
    }

    /// Scenario: Operational and measurement metrics are recorded before reporting.
    /// Guarantees: Reporting emits both the operational and touched measurement metric sets.
    #[test]
    fn operational_metrics_are_reported() {
        let mut metrics = new_test_tracker();
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);
        metrics.add_batch_size(42.0);
        metrics.record_export(Outcome::Success, 42, 1);

        metrics.report(&mut reporter).unwrap();

        assert_eq!(receiver.try_iter().count(), 2);
    }
}
