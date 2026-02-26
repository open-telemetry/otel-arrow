// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Azure Monitor Exporter node.

use std::cell::RefCell;
use std::rc::Rc;

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc, MmscSnapshot};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Shared handle to the metrics tracker.
///
/// The exporter runs on a single-threaded runtime (`#[async_trait(?Send)]`),
/// so `Rc<RefCell<…>>` is sufficient—no `Arc`/`Mutex` overhead needed.
pub type AzureMonitorExporterMetricsRc = Rc<RefCell<AzureMonitorExporterMetricsTracker>>;

/// Telemetry metrics for the Azure Monitor Exporter.
#[metric_set(name = "azure_monitor_exporter.metrics")]
#[derive(Debug, Default, Clone)]
pub struct AzureMonitorExporterMetrics {
    /// Number of rows successfully exported.
    #[metric(unit = "{row}")]
    pub successful_rows: Counter<u64>,
    /// Number of batches successfully exported.
    #[metric(unit = "{batch}")]
    pub successful_batches: Counter<u64>,
    /// Number of messages successfully exported.
    #[metric(unit = "{message}")]
    pub successful_messages: Counter<u64>,
    /// Number of rows that failed to export.
    #[metric(unit = "{row}")]
    pub failed_rows: Counter<u64>,
    /// Number of batches that failed to export.
    #[metric(unit = "{batch}")]
    pub failed_batches: Counter<u64>,
    /// Number of messages that failed to export.
    #[metric(unit = "{message}")]
    pub failed_messages: Counter<u64>,
    /// Client HTTP success latency in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub laclient_http_success_latency: Mmsc,
    /// Number of HTTP 2xx (success) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_2xx: Counter<u64>,
    /// Number of HTTP 401 (unauthorized) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_401: Counter<u64>,
    /// Number of HTTP 403 (forbidden) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_403: Counter<u64>,
    /// Number of HTTP 413 (payload too large) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_413: Counter<u64>,
    /// Number of HTTP 429 (rate limited) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_429: Counter<u64>,
    /// Number of HTTP 5xx (server error) responses.
    #[metric(unit = "{response}")]
    pub laclient_http_5xx: Counter<u64>,
    /// Number of failed authentication attempts.
    pub auth_failures: Counter<u64>,
    /// Authentication success latency in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub auth_success_latency: Mmsc,
    /// Batch size in bytes (min/max/sum/count).
    #[metric(unit = "By")]
    pub batch_size: Mmsc,
    /// Current number of in-flight export requests.
    #[metric(unit = "{export}")]
    pub in_flight_exports: Gauge<u64>,
    /// Current number of batch-to-message mappings (leak detector).
    #[metric(unit = "{entry}")]
    pub batch_to_msg_count: Gauge<u64>,
    /// Current number of message-to-batch mappings (leak detector).
    #[metric(unit = "{entry}")]
    pub msg_to_batch_count: Gauge<u64>,
    /// Current number of message-to-data mappings (leak detector).
    #[metric(unit = "{entry}")]
    pub msg_to_data_count: Gauge<u64>,
    /// Number of log entries rejected for exceeding the batch size limit.
    #[metric(unit = "{entry}")]
    pub log_entries_too_large: Counter<u64>,
    /// Number of successful heartbeat sends.
    #[metric(unit = "{heartbeat}")]
    pub heartbeats: Counter<u64>,
    /// Number of log records that failed to serialize during transformation.
    #[metric(unit = "{record}")]
    pub transform_failures: Counter<u64>,
}

/// Full metrics tracker for the Azure Monitor exporter.
///
/// Wraps a [`MetricSet<AzureMonitorExporterMetrics>`] (registered with the
/// telemetry system for automatic collection).
pub struct AzureMonitorExporterMetricsTracker {
    metrics: MetricSet<AzureMonitorExporterMetrics>,
}

impl std::fmt::Debug for AzureMonitorExporterMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureMonitorExporterMetricsTracker")
            .finish()
    }
}

impl AzureMonitorExporterMetricsTracker {
    /// Create a new stats tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: MetricSet<AzureMonitorExporterMetrics>) -> Self {
        Self { metrics }
    }

    /// Report metrics to the telemetry system.
    ///
    /// Snapshots current metric values, sends them over the telemetry channel,
    /// and resets delta counters on success. Call this from the node's
    /// `CollectTelemetry` handler.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.metrics)
    }

    /// Access the underlying metric set.
    #[inline]
    #[must_use]
    pub fn metrics(&self) -> &MetricSet<AzureMonitorExporterMetrics> {
        &self.metrics
    }

    /// Mutably access the underlying metric set.
    #[inline]
    pub fn metrics_mut(&mut self) -> &mut MetricSet<AzureMonitorExporterMetrics> {
        &mut self.metrics
    }

    // ── Metric accessors (delegated) ────────────────────────────────

    /// Get the total number of successfully exported rows.
    #[inline]
    #[must_use]
    pub fn successful_row_count(&self) -> u64 {
        self.metrics.successful_rows.get()
    }

    /// Get the total number of successfully exported batches.
    #[inline]
    #[must_use]
    pub fn successful_batch_count(&self) -> u64 {
        self.metrics.successful_batches.get()
    }

    /// Get the total number of successfully exported messages.
    #[inline]
    #[must_use]
    pub fn successful_msg_count(&self) -> u64 {
        self.metrics.successful_messages.get()
    }

    /// Get the total number of rows that failed to export.
    #[inline]
    #[must_use]
    pub fn failed_row_count(&self) -> u64 {
        self.metrics.failed_rows.get()
    }

    /// Get the total number of batches that failed to export.
    #[inline]
    #[must_use]
    pub fn failed_batch_count(&self) -> u64 {
        self.metrics.failed_batches.get()
    }

    /// Get the total number of messages that failed to export.
    #[inline]
    #[must_use]
    pub fn failed_msg_count(&self) -> u64 {
        self.metrics.failed_messages.get()
    }

    /// Get the client HTTP success latency snapshot (min/max/sum/count).
    #[inline]
    #[must_use]
    pub fn client_success_latency(&self) -> MmscSnapshot {
        self.metrics.laclient_http_success_latency.get()
    }

    /// Get the auth success latency snapshot (min/max/sum/count).
    #[inline]
    #[must_use]
    pub fn auth_success_latency(&self) -> MmscSnapshot {
        self.metrics.auth_success_latency.get()
    }

    /// Get the batch size snapshot (min/max/sum/count) in bytes.
    #[inline]
    #[must_use]
    pub fn batch_size(&self) -> MmscSnapshot {
        self.metrics.batch_size.get()
    }

    /// Get the current in-flight exports gauge value.
    #[inline]
    #[must_use]
    pub fn in_flight_exports(&self) -> u64 {
        self.metrics.in_flight_exports.get()
    }

    /// Get the current batch_to_msg map size.
    #[inline]
    #[must_use]
    pub fn batch_to_msg_count(&self) -> u64 {
        self.metrics.batch_to_msg_count.get()
    }

    /// Get the current msg_to_batch map size.
    #[inline]
    #[must_use]
    pub fn msg_to_batch_count(&self) -> u64 {
        self.metrics.msg_to_batch_count.get()
    }

    /// Get the current msg_to_data map size.
    #[inline]
    #[must_use]
    pub fn msg_to_data_count(&self) -> u64 {
        self.metrics.msg_to_data_count.get()
    }

    // ── Mutation helpers ────────────────────────────────────────────

    /// Increment successful row count.
    #[inline]
    pub fn add_rows(&mut self, row_count: u64) {
        self.metrics.successful_rows.add(row_count);
    }

    /// Increment successful batch count.
    #[inline]
    pub fn add_batch(&mut self) {
        self.metrics.successful_batches.inc();
    }

    /// Increment successful message count.
    #[inline]
    pub fn add_messages(&mut self, msg_count: u64) {
        self.metrics.successful_messages.add(msg_count);
    }

    /// Increment failed row count.
    #[inline]
    pub fn add_failed_rows(&mut self, row_count: u64) {
        self.metrics.failed_rows.add(row_count);
    }

    /// Increment failed batch count.
    #[inline]
    pub fn add_failed_batch(&mut self) {
        self.metrics.failed_batches.inc();
    }

    /// Increment failed message count.
    #[inline]
    pub fn add_failed_messages(&mut self, msg_count: u64) {
        self.metrics.failed_messages.add(msg_count);
    }

    /// Record a client HTTP success latency observation in milliseconds.
    #[inline]
    pub fn add_client_success_latency(&mut self, latency_ms: f64) {
        self.metrics
            .laclient_http_success_latency
            .record(latency_ms);
    }

    /// Record an auth success latency observation in milliseconds.
    #[inline]
    pub fn add_auth_success_latency(&mut self, latency_ms: f64) {
        self.metrics.auth_success_latency.record(latency_ms);
    }

    /// Record a failed authentication attempt.
    #[inline]
    pub fn add_auth_failure(&mut self) {
        self.metrics.auth_failures.inc();
    }

    /// Record a batch size observation in bytes.
    #[inline]
    pub fn add_batch_size(&mut self, size_bytes: f64) {
        self.metrics.batch_size.record(size_bytes);
    }

    /// Set the current number of in-flight exports.
    #[inline]
    pub fn set_in_flight_exports(&mut self, count: u64) {
        self.metrics.in_flight_exports.set(count);
    }

    /// Set the current batch_to_msg map size.
    #[inline]
    pub fn set_batch_to_msg_count(&mut self, count: u64) {
        self.metrics.batch_to_msg_count.set(count);
    }

    /// Set the current msg_to_batch map size.
    #[inline]
    pub fn set_msg_to_batch_count(&mut self, count: u64) {
        self.metrics.msg_to_batch_count.set(count);
    }

    /// Set the current msg_to_data map size.
    #[inline]
    pub fn set_msg_to_data_count(&mut self, count: u64) {
        self.metrics.msg_to_data_count.set(count);
    }

    /// Increment the log-entry-too-large counter.
    #[inline]
    pub fn add_log_entry_too_large(&mut self) {
        self.metrics.log_entries_too_large.inc();
    }

    /// Increment the heartbeat success counter.
    #[inline]
    pub fn add_heartbeat(&mut self) {
        self.metrics.heartbeats.inc();
    }

    /// Increment the transform failures counter.
    #[inline]
    pub fn add_transform_failures(&mut self, count: u64) {
        self.metrics.transform_failures.add(count);
    }

    /// Record an HTTP response status code.
    ///
    /// Increments the appropriate status-class counter.
    pub fn record_laclient_status_code(&mut self, status: u16) {
        match status {
            200..=299 => self.metrics.laclient_http_2xx.inc(),
            401 => self.metrics.laclient_http_401.inc(),
            403 => self.metrics.laclient_http_403.inc(),
            413 => self.metrics.laclient_http_413.inc(),
            429 => self.metrics.laclient_http_429.inc(),
            500..=599 => self.metrics.laclient_http_5xx.inc(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::testing::EmptyAttributes;

    /// Helper to create a metrics tracker with a test-registered metric set.
    fn new_test_tracker() -> AzureMonitorExporterMetricsTracker {
        let registry = TelemetryRegistryHandle::new();
        let metric_set =
            registry.register_metric_set::<AzureMonitorExporterMetrics>(EmptyAttributes());
        AzureMonitorExporterMetricsTracker::new(metric_set)
    }

    #[test]
    fn test_stats_initialization() {
        let stats = new_test_tracker();
        assert_eq!(stats.successful_row_count(), 0);
        assert_eq!(stats.failed_row_count(), 0);
    }

    #[test]
    fn test_stats_counters() {
        let mut stats = new_test_tracker();

        stats.add_rows(100);
        stats.add_batch();
        stats.add_messages(50);

        assert_eq!(stats.successful_row_count(), 100);
        assert_eq!(stats.successful_batch_count(), 1);
        assert_eq!(stats.successful_msg_count(), 50);

        stats.add_failed_rows(10);
        stats.add_failed_batch();
        stats.add_failed_messages(5);

        assert_eq!(stats.failed_row_count(), 10);
        assert_eq!(stats.failed_batch_count(), 1);
        assert_eq!(stats.failed_msg_count(), 5);
    }

    #[test]
    fn test_client_success_latency_histogram() {
        let mut stats = new_test_tracker();

        stats.add_client_success_latency(100.0);
        stats.add_client_success_latency(200.0);
        stats.add_client_success_latency(50.0);

        let snap = stats.client_success_latency();
        assert_eq!(snap.count, 3);
        assert_eq!(snap.min, 50.0);
        assert_eq!(snap.max, 200.0);
        assert_eq!(snap.sum, 350.0);
    }

    #[test]
    fn test_auth_success_latency_histogram() {
        let mut stats = new_test_tracker();

        stats.add_auth_success_latency(10.0);
        stats.add_auth_success_latency(30.0);

        let snap = stats.auth_success_latency();
        assert_eq!(snap.count, 2);
        assert_eq!(snap.min, 10.0);
        assert_eq!(snap.max, 30.0);
        assert_eq!(snap.sum, 40.0);
    }

    #[test]
    fn test_record_laclient_status_code() {
        let mut stats = new_test_tracker();

        // 2xx range
        stats.record_laclient_status_code(200);
        stats.record_laclient_status_code(204);
        stats.record_laclient_status_code(299);
        assert_eq!(stats.metrics().laclient_http_2xx.get(), 3);

        // Specific error codes
        stats.record_laclient_status_code(401);
        assert_eq!(stats.metrics().laclient_http_401.get(), 1);

        stats.record_laclient_status_code(403);
        assert_eq!(stats.metrics().laclient_http_403.get(), 1);

        stats.record_laclient_status_code(413);
        assert_eq!(stats.metrics().laclient_http_413.get(), 1);

        stats.record_laclient_status_code(429);
        assert_eq!(stats.metrics().laclient_http_429.get(), 1);

        // 5xx range
        stats.record_laclient_status_code(500);
        stats.record_laclient_status_code(503);
        stats.record_laclient_status_code(599);
        assert_eq!(stats.metrics().laclient_http_5xx.get(), 3);

        // Ignored status codes — counters should remain unchanged
        stats.record_laclient_status_code(100); // 1xx
        stats.record_laclient_status_code(301); // 3xx
        stats.record_laclient_status_code(404); // 4xx not tracked individually
        stats.record_laclient_status_code(418); // 4xx not tracked individually
        stats.record_laclient_status_code(600); // out of range

        assert_eq!(stats.metrics().laclient_http_2xx.get(), 3);
        assert_eq!(stats.metrics().laclient_http_401.get(), 1);
        assert_eq!(stats.metrics().laclient_http_403.get(), 1);
        assert_eq!(stats.metrics().laclient_http_413.get(), 1);
        assert_eq!(stats.metrics().laclient_http_429.get(), 1);
        assert_eq!(stats.metrics().laclient_http_5xx.get(), 3);
    }

    #[test]
    fn test_report() {
        let mut stats = new_test_tracker();
        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        stats.add_rows(42);
        stats.add_batch();

        // Report should succeed and reset delta counters
        stats.report(&mut reporter).unwrap();
        assert_eq!(stats.successful_row_count(), 0);
        assert_eq!(stats.successful_batch_count(), 0);

        // Verify snapshot was sent
        let snapshot = rx.try_recv().unwrap();
        assert!(!snapshot.get_metrics().is_empty());
    }
}
