// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Linux user_events receiver.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetRegistrar, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

// -- Sample outcome attributes ------------------------------------------------

/// Outcome of a sample received from the perf ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum SampleOutcome {
    /// Sample was successfully received.
    Received,
    /// Sample was successfully forwarded downstream.
    Forwarded,
    /// Sample was lost (reported by the perf ring).
    Lost,
}

/// Attributes for sample metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SampleAttributes {
    /// The outcome of the sample.
    pub outcome: SampleOutcome,
}

/// Sample metrics.
#[metric_set(
    name = "receiver.user_events.samples",
    measurement_attributes = SampleAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct UserEventsSampleMetrics {
    /// Number of perf samples.
    #[metric(unit = "{item}")]
    pub samples: Counter<u64>,
}

// -- Drop reason attributes ---------------------------------------------------

/// Reason a sample or record was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum DropReason {
    /// Dropped due to process-wide memory pressure.
    MemoryPressure,
    /// Dropped because no matching subscription was found.
    NoSubscription,
    /// Dropped before allocation because the adapter pending queue reached its cap.
    PendingOverflow,
    /// Dropped because a downstream send failed.
    SendError,
}

/// Attributes for drop metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct DropAttributes {
    /// The reason the sample/record was dropped.
    pub reason: DropReason,
}

/// Drop metrics.
#[metric_set(
    name = "receiver.user_events.dropped",
    measurement_attributes = DropAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct UserEventsDroppedMetrics {
    /// Number of samples or records dropped.
    #[metric(unit = "{item}")]
    pub dropped: Counter<u64>,
}

// -- Other (non-dimensionable) metrics ----------------------------------------

/// Scalar metrics for the user_events receiver that do not fit a single enum dimension.
#[metric_set(name = "receiver.user_events.other")]
#[derive(Debug, Default, Clone)]
pub struct UserEventsOtherMetrics {
    /// Total time spent waiting for downstream channel capacity.
    #[metric(unit = "ns")]
    pub downstream_send_blocked_ns: Counter<u64>,
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

// -- Top-level wrapper ---------------------------------------------------------

/// Linux user_events receiver metrics collection.
pub struct UserEventsReceiverMetrics {
    /// Sample outcome metrics.
    pub samples: MeasurementMetricSet<UserEventsSampleMetrics>,
    /// Drop metrics.
    pub dropped: MeasurementMetricSet<UserEventsDroppedMetrics>,
    /// Scalar metrics.
    pub other: MetricSet<UserEventsOtherMetrics>,
}

impl UserEventsReceiverMetrics {
    /// Registers all metric sets with the pipeline context.
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            samples: UserEventsSampleMetrics::register(pipeline_ctx),
            dropped: UserEventsDroppedMetrics::register(pipeline_ctx),
            other: pipeline_ctx.register_metric_set::<UserEventsOtherMetrics>(),
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.samples.terminal_snapshots();
        snapshots.extend(self.dropped.terminal_snapshots());
        snapshots.extend(self.other.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        reporter.report_measurement(&mut self.samples)?;
        reporter.report_measurement(&mut self.dropped)?;
        reporter.report(&mut self.other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    fn new_test_metrics() -> UserEventsReceiverMetrics {
        let (pipeline_ctx, _) = test_pipeline_ctx();
        UserEventsReceiverMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Sample outcome enum variants and attributes are constructed.
    /// Guarantees: Enum equality and attributes capture the right outcome.
    #[test]
    fn test_sample_attributes() {
        assert_eq!(SampleOutcome::Received, SampleOutcome::Received);
        assert_eq!(SampleOutcome::Forwarded, SampleOutcome::Forwarded);
        assert_eq!(SampleOutcome::Lost, SampleOutcome::Lost);
        assert_ne!(SampleOutcome::Received, SampleOutcome::Forwarded);

        let attr = SampleAttributes {
            outcome: SampleOutcome::Received,
        };
        assert_eq!(attr.outcome, SampleOutcome::Received);
    }

    /// Scenario: Drop reason enum variants and attributes are constructed.
    /// Guarantees: Enum equality and attributes capture the right reason.
    #[test]
    fn test_drop_attributes() {
        assert_eq!(DropReason::MemoryPressure, DropReason::MemoryPressure);
        assert_eq!(DropReason::NoSubscription, DropReason::NoSubscription);
        assert_eq!(DropReason::PendingOverflow, DropReason::PendingOverflow);
        assert_eq!(DropReason::SendError, DropReason::SendError);
        assert_ne!(DropReason::MemoryPressure, DropReason::SendError);

        let attr = DropAttributes {
            reason: DropReason::MemoryPressure,
        };
        assert_eq!(attr.reason, DropReason::MemoryPressure);
    }

    /// Scenario: User events receiver metrics are recorded across samples, dropped, and other metric sets.
    /// Guarantees: Metrics are properly partitioned by their enum attributes.
    #[test]
    fn user_events_metrics_are_partitioned_by_attributes() {
        let mut metrics = new_test_metrics();

        metrics
            .samples
            .with(SampleAttributes {
                outcome: SampleOutcome::Received,
            })
            .samples
            .add(10);
        metrics
            .samples
            .with(SampleAttributes {
                outcome: SampleOutcome::Forwarded,
            })
            .samples
            .add(8);
        metrics
            .samples
            .with(SampleAttributes {
                outcome: SampleOutcome::Lost,
            })
            .samples
            .add(2);

        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .dropped
            .add(5);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::NoSubscription,
            })
            .dropped
            .add(3);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::PendingOverflow,
            })
            .dropped
            .add(4);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::SendError,
            })
            .dropped
            .add(1);

        metrics.other.downstream_send_blocked_ns.add(100);
        metrics.other.late_registration_retries.add(2);
        metrics.other.sessions_started.add(1);
        metrics.other.flushed_batches.add(7);

        assert_eq!(
            metrics
                .samples
                .get(SampleAttributes {
                    outcome: SampleOutcome::Received,
                })
                .samples
                .get(),
            10
        );
        assert_eq!(
            metrics
                .samples
                .get(SampleAttributes {
                    outcome: SampleOutcome::Forwarded,
                })
                .samples
                .get(),
            8
        );
        assert_eq!(
            metrics
                .samples
                .get(SampleAttributes {
                    outcome: SampleOutcome::Lost,
                })
                .samples
                .get(),
            2
        );

        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::MemoryPressure,
                })
                .dropped
                .get(),
            5
        );
        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::NoSubscription,
                })
                .dropped
                .get(),
            3
        );
        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::PendingOverflow,
                })
                .dropped
                .get(),
            4
        );
        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::SendError,
                })
                .dropped
                .get(),
            1
        );

        assert_eq!(metrics.other.downstream_send_blocked_ns.get(), 100);
        assert_eq!(metrics.other.late_registration_retries.get(), 2);
        assert_eq!(metrics.other.sessions_started.get(), 1);
        assert_eq!(metrics.other.flushed_batches.get(), 7);
    }

    /// Scenario: Terminal snapshots are collected from user events receiver metrics.
    /// Guarantees: Snapshots capture the measurement attributes and clear measurement buckets on handoff.
    #[test]
    fn user_events_terminal_snapshots_capture_and_clear() {
        let mut metrics = new_test_metrics();
        metrics
            .samples
            .with(SampleAttributes {
                outcome: SampleOutcome::Received,
            })
            .samples
            .add(1);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .dropped
            .add(1);
        metrics.other.flushed_batches.add(1);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);

        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.user_events.samples"
                && s.measurement_attribute_value("outcome") == Some("received")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.user_events.dropped"
                && s.measurement_attribute_value("reason") == Some("memory_pressure")
        }));
        assert!(
            snapshots
                .iter()
                .any(|s| s.descriptor().name == "receiver.user_events.other")
        );

        // Measurement buckets are cleared after first snapshot collection
        let second = metrics.terminal_snapshots();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].descriptor().name, "receiver.user_events.other");
    }

    /// Scenario: User events receiver metrics are reported to a reporter.
    /// Guarantees: All metric sets are successfully handed over to the reporter.
    #[test]
    fn user_events_report_emits_metric_sets() {
        let mut metrics = new_test_metrics();
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        metrics
            .samples
            .with(SampleAttributes {
                outcome: SampleOutcome::Received,
            })
            .samples
            .add(1);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .dropped
            .add(1);
        metrics.other.flushed_batches.add(1);

        metrics.report(&mut reporter).unwrap();
        assert_eq!(receiver.try_iter().count(), 3);
    }
}
