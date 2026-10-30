// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Linux user_events receiver.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Reason a user_events sample or batch was dropped before delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum UserEventsDropReason {
    /// Dropped because process-wide memory pressure requested ingress shedding.
    MemoryPressure,
    /// Dropped because the sample did not match any configured subscription.
    NoSubscription,
    /// Dropped before allocation because the adapter pending queue reached its configured event or byte cap.
    PendingOverflow,
    /// Dropped because a downstream channel send failed.
    SendError,
}

/// Attributes for dropped user_events sample metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct UserEventsDropAttributes {
    /// Reason the sample was dropped.
    #[attribute_key = "reason"]
    pub(super) reason: UserEventsDropReason,
}

/// Dropped sample metrics for the user_events receiver.
#[metric_set(
    name = "receiver.user_events.drops",
    measurement_attributes = UserEventsDropAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct UserEventsDropMetrics {
    /// Number of dropped samples.
    #[metric(unit = "{item}")]
    pub(super) samples: Counter<u64>,
}

/// Internal operational telemetry for the Linux user_events receiver.
#[metric_set(name = "receiver.user_events")]
#[derive(Debug, Default, Clone)]
pub(super) struct UserEventsOperationalMetrics {
    /// Number of perf samples received from the kernel ring.
    #[metric(unit = "{item}")]
    pub received_samples: Counter<u64>,
    /// Number of perf samples forwarded downstream.
    #[metric(unit = "{item}")]
    pub forwarded_samples: Counter<u64>,
    /// Total time spent waiting for downstream channel capacity.
    #[metric(unit = "ns")]
    pub downstream_send_blocked_ns: Counter<u64>,
    /// Number of lost samples reported by the perf ring.
    #[metric(unit = "{item}")]
    pub lost_perf_samples: Counter<u64>,
    /// Number of late-registration retries attempted while waiting for tracepoints.
    #[metric(unit = "{event}")]
    pub late_registration_retries: Counter<u64>,
    /// Number of receiver sessions successfully started.
    #[metric(unit = "{event}")]
    pub sessions_started: Counter<u64>,
    /// Number of Arrow batches flushed downstream.
    #[metric(unit = "{event}")]
    pub flushed_batches: Counter<u64>,
}

/// Combined metrics collection for the user_events receiver.
pub(super) struct UserEventsReceiverMetrics {
    /// Operational event and throughput metrics.
    pub(super) operational: MetricSet<UserEventsOperationalMetrics>,
    /// Dropped sample metrics partitioned by drop reason.
    pub(super) drops: MeasurementMetricSet<UserEventsDropMetrics>,
}

impl UserEventsReceiverMetrics {
    /// Registers all metric sets with the pipeline context.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operational: UserEventsOperationalMetrics::register(pipeline_ctx),
            drops: UserEventsDropMetrics::register(pipeline_ctx),
        }
    }

    /// Records dropped samples for a specific reason.
    pub(super) fn record_dropped(&mut self, reason: UserEventsDropReason, count: u64) {
        if count > 0 {
            self.drops
                .with(UserEventsDropAttributes { reason })
                .samples
                .add(count);
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = Vec::new();
        if self.operational.needs_flush() {
            snapshots.extend(self.operational.terminal_snapshots());
        }
        snapshots.extend(self.drops.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report(&mut self.operational)?;
        reporter.report_measurement(&mut self.drops)?;
        Ok(())
    }

    /// Returns the dropped sample count for a specific reason (for testing).
    #[cfg(test)]
    #[must_use]
    pub(super) fn dropped_count(&self, reason: UserEventsDropReason) -> u64 {
        self.drops
            .get(UserEventsDropAttributes { reason })
            .samples
            .get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx;
    use otel_arrow_dfe_telemetry::attributes::AttributeEnum;

    fn new_test_metrics() -> UserEventsReceiverMetrics {
        let (pipeline_ctx, _) = test_pipeline_ctx();
        UserEventsReceiverMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Drop reason enum variants are rendered as metric attributes.
    /// Guarantees: Every drop reason has a stable lowercase snake_case telemetry string representation.
    #[test]
    fn drop_reason_attribute_values_are_stable() {
        assert_eq!(
            UserEventsDropReason::MemoryPressure.as_str(),
            "memory_pressure"
        );
        assert_eq!(
            UserEventsDropReason::NoSubscription.as_str(),
            "no_subscription"
        );
        assert_eq!(
            UserEventsDropReason::PendingOverflow.as_str(),
            "pending_overflow"
        );
        assert_eq!(UserEventsDropReason::SendError.as_str(), "send_error");
        assert_eq!(UserEventsDropReason::CARDINALITY, 4);
        assert_eq!(
            UserEventsDropReason::VARIANTS,
            &[
                "memory_pressure",
                "no_subscription",
                "pending_overflow",
                "send_error"
            ]
        );
    }

    /// Scenario: Operational and dropped sample metrics are recorded across multiple reasons.
    /// Guarantees: Each drop reason bucket accumulates independently and snapshots cleanly.
    #[test]
    fn test_user_events_metrics_separation_and_snapshots() {
        let mut metrics = new_test_metrics();

        metrics.operational.received_samples.add(100);
        metrics.operational.forwarded_samples.add(90);
        metrics.operational.flushed_batches.add(3);

        metrics.record_dropped(UserEventsDropReason::MemoryPressure, 4);
        metrics.record_dropped(UserEventsDropReason::NoSubscription, 1);
        metrics.record_dropped(UserEventsDropReason::PendingOverflow, 3);
        metrics.record_dropped(UserEventsDropReason::SendError, 2);

        assert_eq!(metrics.operational.received_samples.get(), 100);
        assert_eq!(metrics.operational.forwarded_samples.get(), 90);
        assert_eq!(metrics.operational.flushed_batches.get(), 3);

        assert_eq!(
            metrics.dropped_count(UserEventsDropReason::MemoryPressure),
            4
        );
        assert_eq!(
            metrics.dropped_count(UserEventsDropReason::NoSubscription),
            1
        );
        assert_eq!(
            metrics.dropped_count(UserEventsDropReason::PendingOverflow),
            3
        );
        assert_eq!(metrics.dropped_count(UserEventsDropReason::SendError), 2);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 5);

        let operational_snapshot = snapshots
            .iter()
            .find(|s| s.descriptor().name == "receiver.user_events")
            .expect("operational snapshot should be present");
        assert_eq!(
            operational_snapshot.measurement_attribute_value("reason"),
            None
        );

        let expected_drops = [
            ("memory_pressure", 4),
            ("no_subscription", 1),
            ("pending_overflow", 3),
            ("send_error", 2),
        ];

        for (reason, count) in expected_drops {
            let drop_snapshot = snapshots
                .iter()
                .find(|s| {
                    s.descriptor().name == "receiver.user_events.drops"
                        && s.measurement_attribute_value("reason") == Some(reason)
                })
                .unwrap_or_else(|| panic!("snapshot for reason `{reason}` should be present"));

            assert_eq!(
                drop_snapshot.get_metrics()[0].to_u64_lossy(),
                count,
                "unexpected drop count for reason `{reason}`"
            );
        }

        let second_snapshots = metrics.terminal_snapshots();
        assert!(
            second_snapshots.is_empty(),
            "second terminal snapshot handoff should not return the same deltas again"
        );
    }

    /// Scenario: Operational and drop metrics are reported to a MetricsReporter.
    /// Guarantees: Touched metric buckets are emitted and delivered to the reporter.
    #[test]
    fn operational_and_drop_metrics_are_reported() {
        let mut metrics = new_test_metrics();
        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        metrics.operational.received_samples.add(10);
        metrics.record_dropped(UserEventsDropReason::MemoryPressure, 2);

        metrics
            .report(&mut reporter)
            .expect("metrics reporting should succeed");

        let snapshots: Vec<_> = rx.try_iter().collect();
        assert_eq!(snapshots.len(), 2);
    }
}
