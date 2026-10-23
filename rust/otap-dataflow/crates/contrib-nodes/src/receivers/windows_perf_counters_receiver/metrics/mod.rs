// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded operational metrics for the Windows performance-counter receiver.

use super::model::SampleDiagnostics;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge, HistogramNormal};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;

/// PDH scrape outcomes for one Windows performance-counter receiver node.
#[metric_set(
    name = "receiver.windows_perf_counters.scrapes",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersScrapeMetrics {
    /// Number of PDH collection attempts by terminal outcome.
    #[metric(unit = "{scrape}")]
    pub attempts: Counter<u64>,
}

#[cfg(test)]
mod tests;

/// Lifecycle, inventory, and collection metrics.
#[metric_set(name = "receiver.windows_perf_counters")]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersMetrics {
    /// Configured exact counter paths.
    #[metric(unit = "{counter}")]
    pub configured_exact: Gauge<u64>,
    /// Configured wildcard counter paths.
    #[metric(unit = "{counter}")]
    pub configured_wildcard: Gauge<u64>,
    /// Currently active expanded wildcard counters.
    #[metric(unit = "{counter}")]
    pub active_expanded: Gauge<u64>,
    /// Whether query recovery or initial counter availability is pending.
    #[metric(unit = "{query}")]
    pub query_pending: Gauge<u64>,
    /// Successful collections.
    #[metric(unit = "{scrape}")]
    pub scrapes: Counter<u64>,
    /// Query-level collection failures.
    #[metric(unit = "{collection}")]
    pub scrape_failures: Counter<u64>,
    /// Accepted scrapes that crossed their timing threshold.
    #[metric(unit = "{scrape}")]
    pub scrape_overruns: Counter<u64>,
    /// Worker commands rejected because another command was already pending.
    #[metric(unit = "{rejection}")]
    pub worker_busy: Counter<u64>,
    /// Collection duration.
    #[metric(unit = "s")]
    pub scrape_duration: HistogramNormal,
    /// Counter observations lost to read, status, calculation, or scaling failures.
    #[metric(unit = "{failure}")]
    pub counter_read_failures: Counter<u64>,
    /// Wildcard discovery refreshes attempted.
    #[metric(unit = "{refresh}")]
    pub discovery_refreshes: Counter<u64>,
    /// Wildcard discovery refreshes that failed.
    #[metric(unit = "{refresh}")]
    pub discovery_failures: Counter<u64>,
    /// Expanded instances added.
    #[metric(unit = "{instance}")]
    pub instances_added: Counter<u64>,
    /// Expanded instances removed.
    #[metric(unit = "{instance}")]
    pub instances_removed: Counter<u64>,
    /// Expanded instances omitted by configured limits.
    #[metric(unit = "{instance}")]
    pub instances_omitted_over_limit: Counter<u64>,
    /// Counter add attempts that failed.
    #[metric(unit = "{failure}")]
    pub counter_add_failures: Counter<u64>,
    /// Deferred counter or collection retry attempts.
    #[metric(unit = "{attempt}")]
    pub retry_attempts: Counter<u64>,
    /// Deferred counter or collection retries that recovered.
    #[metric(unit = "{recovery}")]
    pub retry_recoveries: Counter<u64>,
    /// Worker-owned query rebuild attempts.
    #[metric(unit = "{attempt}")]
    pub query_rebuild_attempts: Counter<u64>,
    /// Worker-owned query rebuilds that recovered.
    #[metric(unit = "{recovery}")]
    pub query_rebuild_recoveries: Counter<u64>,
    /// Points omitted while independently warming.
    #[metric(unit = "{metric}")]
    pub warmup_omissions: Counter<u64>,
}

impl WindowsPerfCountersMetrics {
    pub(super) fn apply(&mut self, diagnostics: &SampleDiagnostics) {
        self.active_expanded
            .set(diagnostics.active_expanded_counters as u64);
        self.query_pending.set(u64::from(diagnostics.query_pending));
        self.discovery_refreshes
            .add(diagnostics.discovery_refreshes);
        self.discovery_failures.add(diagnostics.discovery_failures);
        self.instances_added.add(diagnostics.instances_added);
        self.instances_removed.add(diagnostics.instances_removed);
        self.instances_omitted_over_limit
            .add(diagnostics.instances_omitted_over_limit);
        self.counter_add_failures
            .add(diagnostics.counter_add_failures);
        self.retry_attempts.add(diagnostics.retry_attempts);
        self.retry_recoveries.add(diagnostics.retry_recoveries);
        self.query_rebuild_attempts
            .add(diagnostics.query_rebuild_attempts);
        self.query_rebuild_recoveries
            .add(diagnostics.query_rebuild_recoveries);
        self.warmup_omissions.add(diagnostics.warmup_omissions);
    }
}

/// Receiver metric sets with bounded-cardinality scrape outcomes.
pub(super) struct WindowsPerfCountersReceiverMetrics {
    scrapes: MeasurementMetricSet<WindowsPerfCountersScrapeMetrics>,
    pub(super) health: MetricSet<WindowsPerfCountersMetrics>,
}

impl WindowsPerfCountersReceiverMetrics {
    pub(super) fn register(pipeline: &PipelineContext) -> Self {
        Self {
            scrapes: WindowsPerfCountersScrapeMetrics::register(pipeline),
            health: WindowsPerfCountersMetrics::register(pipeline),
        }
    }

    pub(super) fn record_scrape(&mut self, outcome: Outcome) {
        self.scrapes
            .with(OutcomeAttributes { outcome })
            .attempts
            .inc();
    }

    #[allow(
        dead_code,
        reason = "reporter is connected when receiver runtime is wired"
    )]
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.scrapes)?;
        reporter.report(&mut self.health)
    }

    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.scrapes.terminal_snapshots();
        snapshots.extend(self.health.terminal_snapshots());
        snapshots
    }
}
