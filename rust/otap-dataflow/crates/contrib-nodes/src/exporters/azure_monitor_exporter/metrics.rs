// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Azure Monitor Exporter node.

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

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
    /// Average client latency in milliseconds.
    #[metric(unit = "ms")]
    pub laclient_http_avglatency: Gauge<f64>,
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
    /// Number of successful authentication attempts.
    pub auth_success: Counter<u64>,
    /// Number of failed authentication attempts.
    pub auth_failure: Counter<u64>,
    /// Average authentication latency in milliseconds.
    #[metric(unit = "ms")]
    pub auth_avglatency: Gauge<f64>,
}

/// Full metrics tracker for the Azure Monitor exporter.
///
/// Wraps a [`MetricSet<AzureMonitorExporterMetrics>`] (registered with the
/// telemetry system for automatic collection).
pub struct AzureMonitorExporterMetricsTracker {
    metrics: MetricSet<AzureMonitorExporterMetrics>,
    /// Internal counter for running-average client latency calculation.
    la_client_latency_request_count: f64,
    /// Internal counter for running-average auth latency calculation.
    auth_latency_request_count: f64,
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
        Self {
            metrics,
            la_client_latency_request_count: 0.0,
            auth_latency_request_count: 0.0,
        }
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

    /// Get the average client latency in milliseconds.
    #[inline]
    #[must_use]
    pub fn client_avg_latency_ms(&self) -> f64 {
        self.metrics.laclient_http_avglatency.get()
    }

    /// Get the average auth latency in milliseconds.
    #[inline]
    #[must_use]
    pub fn auth_avg_latency_ms(&self) -> f64 {
        self.metrics.auth_avglatency.get()
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

    /// Update average client latency with a new measurement in milliseconds.
    /// Uses a running average calculation.
    pub fn add_client_latency(&mut self, latency_ms: f64) {
        self.la_client_latency_request_count += 1.0;
        let avg = ((self.metrics.laclient_http_avglatency.get()
            * (self.la_client_latency_request_count - 1.0))
            + latency_ms)
            / self.la_client_latency_request_count;
        self.metrics.laclient_http_avglatency.set(avg);
    }

    /// Update average auth latency with a new measurement in milliseconds.
    /// Uses a running average calculation.
    pub fn add_auth_latency(&mut self, latency_ms: f64) {
        self.auth_latency_request_count += 1.0;
        let avg = ((self.metrics.auth_avglatency.get()
            * (self.auth_latency_request_count - 1.0))
            + latency_ms)
            / self.auth_latency_request_count;
        self.metrics.auth_avglatency.set(avg);
    }

    /// Record a successful authentication attempt.
    #[inline]
    pub fn add_auth_success(&mut self) {
        self.metrics.auth_success.inc();
    }

    /// Record a failed authentication attempt.
    #[inline]
    pub fn add_auth_failure(&mut self) {
        self.metrics.auth_failure.inc();
    }

    /// Record an HTTP response status code.
    ///
    /// Increments the appropriate status-class counter.
    pub fn record_status_code(&mut self, status: u16) {
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
    fn test_latency_calculation() {
        let mut stats = new_test_tracker();

        // First request: 100ms
        stats.add_client_latency(100.0);
        assert_eq!(stats.client_avg_latency_ms(), 100.0);

        // Second request: 200ms -> avg 150ms
        stats.add_client_latency(200.0);
        assert_eq!(stats.client_avg_latency_ms(), 150.0);

        // Third request: 0ms -> avg 100ms
        stats.add_client_latency(0.0);
        assert_eq!(stats.client_avg_latency_ms(), 100.0);
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
