// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the Kubernetes Service Account Token extension.

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Telemetry metrics for the Kubernetes Service Account Token extension.
#[metric_set(name = "extension.k8s_service_account_token")]
#[derive(Debug, Default, Clone)]
pub struct K8sServiceAccountTokenMetrics {
    /// Number of successful token reads.
    #[metric(unit = "{read}")]
    pub token_reads: Counter<u64>,
    /// Number of failed token reads.
    #[metric(unit = "{read}")]
    pub token_read_failures: Counter<u64>,
    /// Number of tokens published to consumers via the watch channel.
    #[metric(unit = "{token}")]
    pub token_publish: Counter<u64>,
    /// Latency of successful token reads in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub token_read_latency: Mmsc,
}

/// Tracks and flushes the extension's metric set.
pub struct K8sServiceAccountTokenMetricsTracker {
    metrics: MetricSet<K8sServiceAccountTokenMetrics>,
}

impl std::fmt::Debug for K8sServiceAccountTokenMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("K8sServiceAccountTokenMetricsTracker")
            .finish()
    }
}

impl K8sServiceAccountTokenMetricsTracker {
    /// Creates a new tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: MetricSet<K8sServiceAccountTokenMetrics>) -> Self {
        Self { metrics }
    }

    /// Flushes the metric set to the telemetry reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.metrics)
    }

    /// Returns a point-in-time snapshot of the metric set, e.g. to attach to the
    /// terminal state on shutdown.
    #[must_use]
    pub fn snapshot(&self) -> MetricSetSnapshot {
        self.metrics.snapshot()
    }

    /// Records a successful token read with its latency in milliseconds.
    pub fn record_success(&mut self, latency_ms: f64) {
        self.metrics.token_reads.inc();
        self.metrics.token_read_latency.record(latency_ms);
    }

    /// Records a failed token read.
    pub fn record_failure(&mut self) {
        self.metrics.token_read_failures.inc();
    }

    /// Records a token publication to consumers.
    pub fn record_publish(&mut self) {
        self.metrics.token_publish.inc();
    }
}
