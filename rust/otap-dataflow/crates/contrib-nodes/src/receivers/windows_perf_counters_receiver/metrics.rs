// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Self-metrics for the Windows performance-counter receiver.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::{ReceiverProcessingMetricSet, ReceiverPulledMetrics};
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Duration;

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
    /// Failed counter adds, including retries.
    #[metric(unit = "{failure}")]
    pub(super) counter_add_failures: Counter<u64>,
    /// Counters currently added to a query.
    #[metric(unit = "{counter}")]
    pub(super) active_counters: Gauge<u64>,
}

/// Combined metrics collection for the Windows performance-counter receiver.
pub(super) struct WindowsPerfCountersMetrics {
    /// Accepted collection attempts partitioned by outcome.
    pub(super) pulled: MeasurementMetricSet<ReceiverPulledMetrics>,
    /// Drops partitioned by reason.
    pub(super) drops: MeasurementMetricSet<WindowsPerfCountersDropMetrics>,
    /// Operational gauges and overruns.
    pub(super) operational: MetricSet<WindowsPerfCountersOperationalMetrics>,
    /// Receiver-local processing duration.
    pub(super) processing: ReceiverProcessingMetricSet,
}

impl WindowsPerfCountersMetrics {
    /// Registers all metric sets with the pipeline context.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            pulled: ReceiverPulledMetrics::register(pipeline_ctx),
            drops: WindowsPerfCountersDropMetrics::register(pipeline_ctx),
            operational: WindowsPerfCountersOperationalMetrics::register(pipeline_ctx),
            processing: ReceiverProcessingMetricSet::register(
                pipeline_ctx,
                pipeline_ctx.node_duration_distribution(),
            ),
        }
    }

    /// Records an accepted collection attempt reaching a terminal outcome.
    pub(super) fn record_pulled(&mut self, outcome: Outcome) {
        self.pulled
            .with(OutcomeAttributes { outcome })
            .attempts
            .inc();
    }

    /// Records dropped batches for a specific reason.
    pub(super) fn record_dropped(&mut self, reason: WindowsPerfCountersDropReason) {
        self.drops
            .with(WindowsPerfCountersDropAttributes { reason })
            .batches
            .inc();
    }

    /// Records receiver processing duration for a collection attempt.
    pub(super) fn record_duration(&mut self, duration: Duration) {
        self.processing.record(duration);
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.pulled.terminal_snapshots();
        snapshots.extend(self.drops.terminal_snapshots());
        snapshots.extend(self.processing.terminal_snapshots());
        if self.operational.needs_flush() {
            snapshots.extend(self.operational.terminal_snapshots());
        }
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.pulled)?;
        reporter.report_measurement(&mut self.drops)?;
        self.processing.report(reporter)?;
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

    /// Scenario: Pulled metrics record successful and failed collections.
    /// Guarantees: Collection attempts are partitioned by outcome attribute without bucket collisions.
    #[test]
    fn pulled_metrics_are_partitioned_by_outcome() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_pulled(Outcome::Success);
        metrics.record_pulled(Outcome::Success);
        metrics.record_pulled(Outcome::Failure);

        assert_eq!(
            metrics
                .pulled
                .get(OutcomeAttributes {
                    outcome: Outcome::Success
                })
                .attempts
                .get(),
            2
        );
        assert_eq!(
            metrics
                .pulled
                .get(OutcomeAttributes {
                    outcome: Outcome::Failure
                })
                .attempts
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

        metrics.record_pulled(Outcome::Success);
        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);
        metrics.operational.scrape_overruns.inc();
        metrics.record_duration(Duration::from_millis(50));

        let first = metrics.terminal_snapshots();
        assert!(!first.is_empty());

        assert!(metrics.pulled.terminal_snapshots().is_empty());
        assert!(metrics.drops.terminal_snapshots().is_empty());
        assert!(metrics.processing.terminal_snapshots().is_empty());
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
    /// Guarantees: Telemetry contracts strictly adhere to `{attempt}`, `{batch}`, `{tick}`, and
    /// `{counter}` units, and receiver.processing duration is in seconds.
    #[test]
    fn metric_descriptors_and_units_are_exact() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = WindowsPerfCountersMetrics::register(&pipeline_ctx);

        metrics.record_pulled(Outcome::Success);
        metrics.record_dropped(WindowsPerfCountersDropReason::DownstreamSendError);
        metrics.operational.counter_add_failures.add(1);
        metrics.record_duration(Duration::from_millis(50));

        let snapshots = metrics.terminal_snapshots();
        let mut found_pulled = false;
        let mut found_drops = false;
        let mut found_processing = false;
        let mut found_operational = false;

        for snapshot in &snapshots {
            let desc = snapshot.descriptor();
            if desc.name == "receiver.pulled" {
                found_pulled = true;
                assert_eq!(desc.metrics[0].name, "attempts");
                assert_eq!(desc.metrics[0].unit, "{attempt}");
            } else if desc.name == "receiver.processing" {
                found_processing = true;
                assert_eq!(desc.metrics[0].name, "duration");
                assert_eq!(desc.metrics[0].unit, "s");
            } else if desc.name == "receiver.windows_perf_counters.drops" {
                found_drops = true;
                assert_eq!(desc.metrics[0].name, "batches");
                assert_eq!(desc.metrics[0].unit, "{batch}");
            } else if desc.name == "receiver.windows_perf_counters" {
                found_operational = true;
                for m in desc.metrics {
                    match m.name {
                        "scrape.overruns" => assert_eq!(m.unit, "{tick}"),
                        "counter.add.failures" => assert_eq!(m.unit, "{failure}"),
                        "active.counters" => assert_eq!(m.unit, "{counter}"),
                        _ => {}
                    }
                }
            }
        }

        assert!(found_pulled, "receiver.pulled metric set descriptor present");
        assert!(found_drops, "drops metric set descriptor present");
        assert!(
            found_processing,
            "receiver.processing metric set descriptor present"
        );
        assert!(
            found_operational,
            "operational metric set descriptor present"
        );
    }
}
