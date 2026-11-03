// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Self-metrics for the Windows performance-counter receiver.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::Outcome;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge, HistogramNormal};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Attributes for performance counter scrape metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct WindowsPerfCountersScrapeAttributes {
    /// Terminal outcome of the recorded collection attempt.
    pub(super) outcome: Outcome,
}

/// Collections partitioned by outcome.
#[metric_set(
    name = "receiver.windows_perf_counters.scrapes",
    measurement_attributes = WindowsPerfCountersScrapeAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersScrapeMetrics {
    /// Number of collections reaching the specified outcome.
    #[metric(unit = "{scrape}")]
    pub(super) collections: Counter<u64>,
}

/// Reason a performance counter batch was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum WindowsPerfCountersDropReason {
    /// Downstream channel full or disconnected.
    DownstreamSendError,
}

/// Attributes for dropped batch metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct WindowsPerfCountersDropAttributes {
    /// Bounded reason describing why the batch was dropped.
    pub(super) reason: WindowsPerfCountersDropReason,
}

/// Drops partitioned by reason.
#[metric_set(
    name = "receiver.windows_perf_counters.drops",
    measurement_attributes = WindowsPerfCountersDropAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersDropMetrics {
    /// Number of dropped batches.
    #[metric(unit = "{batch}")]
    pub(super) batches: Counter<u64>,
}

/// Operational self-metrics for the Windows performance-counter receiver.
#[metric_set(name = "receiver.windows_perf_counters")]
#[derive(Debug, Default, Clone)]
pub(super) struct WindowsPerfCountersOperationalMetrics {
    /// Ticks skipped because the previous scrape was still running.
    #[metric(unit = "{tick}")]
    pub(super) scrape_overruns: Counter<u64>,
    /// Time from scrape request to report.
    #[metric(unit = "s")]
    pub(super) scrape_duration: HistogramNormal,
    /// Failed counter adds, including retries.
    #[metric(unit = "{failure}")]
    pub(super) counter_add_failures: Counter<u64>,
    /// Counters currently added to a query.
    #[metric(unit = "{counter}")]
    pub(super) active_counters: Gauge<u64>,
}

/// Combined metrics collection for the Windows performance-counter receiver.
pub(super) struct WindowsPerfCountersMetrics {
    /// Collections partitioned by outcome.
    pub(super) scrapes: MeasurementMetricSet<WindowsPerfCountersScrapeMetrics>,
    /// Drops partitioned by reason.
    pub(super) drops: MeasurementMetricSet<WindowsPerfCountersDropMetrics>,
    /// Operational gauges, overruns, and duration.
    pub(super) operational: MetricSet<WindowsPerfCountersOperationalMetrics>,
}

impl WindowsPerfCountersMetrics {
    /// Registers all metric sets with the pipeline context.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            scrapes: WindowsPerfCountersScrapeMetrics::register(pipeline_ctx),
            drops: WindowsPerfCountersDropMetrics::register(pipeline_ctx),
            operational: WindowsPerfCountersOperationalMetrics::register(pipeline_ctx),
        }
    }

    /// Records a collection reaching a terminal outcome.
    pub(super) fn record_scrape(&mut self, outcome: Outcome) {
        self.scrapes
            .with(WindowsPerfCountersScrapeAttributes { outcome })
            .collections
            .inc();
    }

    /// Records dropped batches for a specific reason.
    pub(super) fn record_dropped(&mut self, reason: WindowsPerfCountersDropReason) {
        self.drops
            .with(WindowsPerfCountersDropAttributes { reason })
            .batches
            .inc();
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.scrapes.terminal_snapshots();
        snapshots.extend(self.drops.terminal_snapshots());
        if self.operational.needs_flush() {
            snapshots.extend(self.operational.terminal_snapshots());
        }
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.scrapes)?;
        reporter.report_measurement(&mut self.drops)?;
        reporter.report(&mut self.operational)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::attributes::AttributeEnum as _;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn test_pipeline_context() -> PipelineContext {
        let registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry_handle);
        controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0)
    }

    /// Scenario: Scrape metrics record successful and failed collections.
    /// Guarantees: Scrapes are partitioned by outcome attribute without bucket collisions.
    #[test]
    fn scrape_metrics_are_partitioned_by_outcome() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_scrape(Outcome::Success);
        metrics.record_scrape(Outcome::Success);
        metrics.record_scrape(Outcome::Failure);

        assert_eq!(
            metrics
                .scrapes
                .get(WindowsPerfCountersScrapeAttributes {
                    outcome: Outcome::Success
                })
                .collections
                .get(),
            2
        );
        assert_eq!(
            metrics
                .scrapes
                .get(WindowsPerfCountersScrapeAttributes {
                    outcome: Outcome::Failure
                })
                .collections
                .get(),
            1
        );
    }

    /// Scenario: Drop metrics record dropped batches with specific drop reasons.
    /// Guarantees: Dropped batches are correctly dimensioned by reason.
    #[test]
    fn drop_metrics_are_partitioned_by_reason() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);
        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);

        assert_eq!(
            metrics
                .drops
                .get(WindowsPerfCountersDropAttributes {
                    reason: WindowsPerfCountersDropReason::DownstreamSendError
                })
                .batches
                .get(),
            2
        );
    }

    /// Scenario: Snapshots are taken across consecutive terminal collection cycles.
    /// Guarantees: Measurement metric buckets emit on the first snapshot and are cleared on the
    /// second snapshot while operational counters reset.
    #[test]
    fn terminal_snapshots_emit_once_and_clear_hot_state() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_scrape(Outcome::Success);
        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);
        metrics.operational.scrape_overruns.inc();

        let first = metrics.terminal_snapshots();
        assert!(!first.is_empty());

        assert!(metrics.scrapes.terminal_snapshots().is_empty());
        assert!(metrics.drops.terminal_snapshots().is_empty());
        assert_eq!(metrics.operational.scrape_overruns.get(), 0);
    }

    /// Scenario: Enum attribute representation stability test.
    /// Guarantees: Drop reason serializes to the exact expected lowercase snake_case
    /// attribute string.
    #[test]
    fn enum_attribute_representation_is_stable() {
        assert_eq!(
            WindowsPerfCountersDropReason::DownstreamSendError.as_str(),
            "downstream_send_error"
        );
    }

    /// Scenario: Verification of metric set names, instrument names, and units.
    /// Guarantees: Telemetry contracts strictly adhere to `{scrape}`, `{batch}`, `{tick}`, and
    /// `{counter}` units.
    #[test]
    fn metric_descriptors_and_units_are_exact() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_scrape(Outcome::Success);
        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);
        metrics.operational.counter_add_failures.add(1);

        let snapshots = metrics.terminal_snapshots();
        let mut found_scrapes = false;
        let mut found_drops = false;
        let mut found_operational = false;

        for snapshot in &snapshots {
            let desc = snapshot.descriptor();
            if desc.name == "receiver.windows_perf_counters.scrapes" {
                found_scrapes = true;
                assert_eq!(desc.metrics[0].name, "collections");
                assert_eq!(desc.metrics[0].unit, "{scrape}");
            } else if desc.name == "receiver.windows_perf_counters.drops" {
                found_drops = true;
                assert_eq!(desc.metrics[0].name, "batches");
                assert_eq!(desc.metrics[0].unit, "{batch}");
            } else if desc.name == "receiver.windows_perf_counters" {
                found_operational = true;
                for m in desc.metrics {
                    match m.name {
                        "scrape.overruns" => assert_eq!(m.unit, "{tick}"),
                        "scrape.duration" => assert_eq!(m.unit, "s"),
                        "counter.add.failures" => assert_eq!(m.unit, "{failure}"),
                        "active.counters" => assert_eq!(m.unit, "{counter}"),
                        _ => {}
                    }
                }
            }
        }

        assert!(found_scrapes, "scrapes metric set descriptor present");
        assert!(found_drops, "drops metric set descriptor present");
        assert!(
            found_operational,
            "operational metric set descriptor present"
        );
    }
}
