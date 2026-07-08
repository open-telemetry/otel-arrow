// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the Azure Identity Auth extension.

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Telemetry metrics for the Azure Identity Auth extension.
#[metric_set(name = "extension.azure_identity_auth")]
#[derive(Debug, Default, Clone)]
pub struct AzureIdentityAuthMetrics {
    /// Number of successful credential acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_successes: Counter<u64>,
    /// Number of failed credential acquisitions.
    #[metric(unit = "{acquisition}")]
    pub auth_failures: Counter<u64>,
    /// Number of tokens published to consumers via the watch channel.
    #[metric(unit = "{token}")]
    pub auth_token_publish: Counter<u64>,
    /// Latency of successful acquisitions in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub auth_success_latency: Mmsc,
}

/// Tracks and flushes the extension's metric set.
pub struct AzureIdentityAuthMetricsTracker {
    metrics: MetricSet<AzureIdentityAuthMetrics>,
}

impl std::fmt::Debug for AzureIdentityAuthMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AzureIdentityAuthMetricsTracker").finish()
    }
}

impl AzureIdentityAuthMetricsTracker {
    /// Creates a new tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: MetricSet<AzureIdentityAuthMetrics>) -> Self {
        Self { metrics }
    }

    /// Flushes the metric set to the telemetry reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.metrics)
    }

    /// Records a successful acquisition with its latency in milliseconds.
    pub fn record_success(&mut self, latency_ms: f64) {
        self.metrics.auth_successes.inc();
        self.metrics.auth_success_latency.record(latency_ms);
    }

    /// Records a failed acquisition.
    pub fn record_failure(&mut self) {
        self.metrics.auth_failures.inc();
    }

    /// Records a token publication to consumers.
    pub fn record_publish(&mut self) {
        self.metrics.auth_token_publish.inc();
    }
}
