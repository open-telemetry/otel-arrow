// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry for the Kubernetes SAT authorizer extension.

use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::{MetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;

/// Telemetry metrics for the Kubernetes SAT authorizer extension.
#[metric_set(name = "extension.k8s_sat_token_authorizer")]
#[derive(Debug, Default, Clone)]
pub struct K8sSatTokenAuthorizerMetrics {
    /// Number of requests admitted (Allow decisions).
    #[metric(unit = "{decision}")]
    pub authz_allow: Counter<u64>,
    /// Number of requests denied (Deny decisions).
    #[metric(unit = "{decision}")]
    pub authz_deny: Counter<u64>,
    /// Number of undetermined outcomes (backend unreachable); callers fail closed.
    #[metric(unit = "{decision}")]
    pub authz_error: Counter<u64>,
    /// Number of `TokenReview` calls issued to the API server (cache misses).
    #[metric(unit = "{review}")]
    pub token_review_calls: Counter<u64>,
    /// Number of decisions served from the local decision cache (cache hits).
    #[metric(unit = "{lookup}")]
    pub cache_hits: Counter<u64>,
    /// Latency of `TokenReview` calls in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub token_review_latency: Mmsc,
}

/// Tracks and flushes the extension's metric set.
pub struct K8sSatTokenAuthorizerMetricsTracker {
    metrics: MetricSet<K8sSatTokenAuthorizerMetrics>,
}

impl std::fmt::Debug for K8sSatTokenAuthorizerMetricsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("K8sSatTokenAuthorizerMetricsTracker")
            .finish()
    }
}

impl K8sSatTokenAuthorizerMetricsTracker {
    /// Creates a new tracker wrapping a registered metric set.
    #[must_use]
    pub fn new(metrics: MetricSet<K8sSatTokenAuthorizerMetrics>) -> Self {
        Self { metrics }
    }

    /// Flushes the metric set to the telemetry reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.metrics)
    }

    /// Returns a point-in-time snapshot of the metric set, e.g. to attach to
    /// the terminal state on shutdown.
    #[must_use]
    pub fn snapshot(&self) -> MetricSetSnapshot {
        self.metrics.snapshot()
    }

    /// Records an Allow decision.
    pub fn record_allow(&mut self) {
        self.metrics.authz_allow.inc();
    }

    /// Records a Deny decision.
    pub fn record_deny(&mut self) {
        self.metrics.authz_deny.inc();
    }

    /// Records an undetermined outcome (backend error).
    pub fn record_error(&mut self) {
        self.metrics.authz_error.inc();
    }

    /// Records a `TokenReview` call and its latency in milliseconds.
    pub fn record_token_review(&mut self, latency_ms: f64) {
        self.metrics.token_review_calls.inc();
        self.metrics.token_review_latency.record(latency_ms);
    }

    /// Records a decision served from the local cache.
    pub fn record_cache_hit(&mut self) {
        self.metrics.cache_hits.inc();
    }
}
