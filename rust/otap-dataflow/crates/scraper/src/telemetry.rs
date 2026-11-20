// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared telemetry for database receivers.
//!
//! Collection ends after validation and encoding, before downstream handoff.
//! Diagnostic attributes use closed enums, never SQL or configured source IDs.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::{
    ReceiverPullAttempt, ReceiverPullCancellation, ReceiverPullMetrics,
};
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Shared collection metrics and database-specific diagnostics.
pub struct DatabaseReceiverMetrics {
    boundary: ReceiverPullMetrics,
    // The local guard never travels to native or encoding workers.
    collection: Option<ReceiverPullAttempt>,
    pub(crate) operational: MetricSet<DatabaseOperationalMetrics>,
    pub(crate) lifecycle: MeasurementMetricSet<DatabaseLifecycleMetrics>,
    pub(crate) feedback: MeasurementMetricSet<DatabaseFeedbackMetrics>,
    pub(crate) checkpoints: MeasurementMetricSet<DatabaseCheckpointMetrics>,
}

impl DatabaseReceiverMetrics {
    /// Registers shared metrics according to node interests and database diagnostics.
    #[must_use]
    pub fn register(pipeline: &PipelineContext) -> Self {
        Self {
            boundary: ReceiverPullMetrics::register(pipeline),
            collection: None,
            operational: DatabaseOperationalMetrics::register(pipeline),
            lifecycle: DatabaseLifecycleMetrics::register(pipeline),
            feedback: DatabaseFeedbackMetrics::register(pipeline),
            checkpoints: DatabaseCheckpointMetrics::register(pipeline),
        }
    }

    pub(crate) fn begin_collection(&mut self) -> ReceiverPullCancellation {
        debug_assert!(self.collection.is_none());
        let attempt = self.boundary.processing();
        let cancellation = attempt.cancellation_handle();
        self.collection = Some(attempt);
        cancellation
    }

    pub(crate) fn collection_succeeded(&mut self) {
        if let Some(attempt) = self.collection.take() {
            attempt.success();
        }
    }

    pub(crate) fn collection_failed(&mut self) {
        // Drop records failure exactly once, including before native cleanup.
        drop(self.collection.take());
    }

    pub(crate) fn record_feedback(&mut self, outcome: Outcome) {
        self.feedback
            .with(OutcomeAttributes { outcome })
            .responses
            .inc();
    }

    pub(crate) fn record_checkpoint(&mut self, outcome: Outcome) {
        self.checkpoints
            .with(OutcomeAttributes { outcome })
            .writes
            .inc();
    }

    pub(crate) fn record_lifecycle(&mut self, transition: DatabaseLifecycle) {
        self.lifecycle
            .with(DatabaseLifecycleAttributes { transition })
            .transitions
            .inc();
    }

    /// Reports completed observations, retaining deltas deferred by backpressure.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report(&mut self.operational)?;
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.lifecycle)?;
        reporter.report_measurement(&mut self.feedback)?;
        reporter.report_measurement(&mut self.checkpoints)
    }

    /// Ends any abandoned collection before taking final observations.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.collection_failed();
        let mut snapshots = self.operational.terminal_snapshots();
        snapshots.extend(self.boundary.terminal_snapshots());
        snapshots.extend(self.lifecycle.terminal_snapshots());
        snapshots.extend(self.feedback.terminal_snapshots());
        snapshots.extend(self.checkpoints.terminal_snapshots());
        snapshots
    }
}

/// Observed receiver lifecycle transitions, not successful cleanup.
#[derive(Clone, Copy, Debug, AttributeEnum)]
pub(crate) enum DatabaseLifecycle {
    Start,
    Drain,
    Shutdown,
}

#[attribute_set(item, measurement)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct DatabaseLifecycleAttributes {
    pub transition: DatabaseLifecycle,
}

/// Receiver starts and received stop requests.
#[metric_set(
    name = "receiver.database.lifecycle",
    measurement_attributes = DatabaseLifecycleAttributes
)]
#[derive(Clone, Debug, Default)]
pub(crate) struct DatabaseLifecycleMetrics {
    /// Lifecycle transitions observed, including during I/O waits.
    #[metric(unit = "{transition}")]
    pub transitions: Counter<u64>,
}

/// Matching downstream feedback, independent of durable checkpoint success.
#[metric_set(
    name = "receiver.database.feedback",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Clone, Debug, Default)]
pub(crate) struct DatabaseFeedbackMetrics {
    /// ACKs (success), retryable NACKs (failure), or permanent NACKs (refused).
    #[metric(unit = "{feedback}")]
    pub responses: Counter<u64>,
}

/// Completed durable checkpoint writes, independent of collection outcomes.
#[metric_set(
    name = "receiver.database.checkpoints",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Clone, Debug, Default)]
pub(crate) struct DatabaseCheckpointMetrics {
    /// Checkpoint writes reaching success or failure.
    #[metric(unit = "{write}")]
    pub writes: Counter<u64>,
}

/// Operational diagnostics not covered by collection or categorical outcomes.
#[metric_set(name = "receiver.database")]
#[derive(Clone, Debug, Default)]
pub(crate) struct DatabaseOperationalMetrics {
    /// Query-page attempts that failed, including operation setup.
    #[metric(unit = "{failure}")]
    pub query_failures: Counter<u64>,
    /// Reconnect operations started after transient database failures.
    #[metric(unit = "{attempt}")]
    pub reconnects: Counter<u64>,
    /// Pages sent downstream.
    #[metric(unit = "{batch}")]
    pub batches_sent: Counter<u64>,
    /// Database rows sent downstream.
    #[metric(unit = "{row}")]
    pub rows_sent: Counter<u64>,
    /// Encoded OTLP bytes sent downstream.
    #[metric(unit = "By")]
    pub encoded_bytes_sent: Counter<u64>,
    /// Records whose source event time cannot fit OTLP's timestamp range.
    #[metric(unit = "{record}")]
    pub event_time_fallbacks: Counter<u64>,
    /// One while polling is paused by permanent rejection, otherwise zero.
    #[metric(unit = "1")]
    pub rejection_paused: Gauge<u64>,
    /// Replay schedules created after a NACK, not completed query attempts.
    #[metric(unit = "{replay}")]
    pub replays: Counter<u64>,
    /// ACK/NACK feedback discarded as malformed, wrong-generation, stale, or duplicate.
    #[metric(unit = "{feedback}")]
    pub stale_feedback: Counter<u64>,
    /// Stale checkpoint revisions that could not be removed.
    #[metric(unit = "{failure}")]
    pub checkpoint_cleanup_failures: Counter<u64>,
    /// Active database operations cancelled by a control message.
    #[metric(unit = "{cancellation}")]
    pub cancellations: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_telemetry::metrics::MetricValue;
    use std::time::Duration;

    /// Scenario: Operational counters are handed off at termination more than once.
    /// Guarantees: The first handoff transfers the pending delta; later snapshots cannot duplicate it.
    #[test]
    fn terminal_snapshots_transfer_operational_counters() {
        let (pipeline, _registry) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = DatabaseReceiverMetrics::register(&pipeline);
        metrics.operational.query_failures.inc();
        for expected in [1, 0] {
            let snapshots = metrics.terminal_snapshots();
            let operational = snapshots
                .iter()
                .find(|snapshot| snapshot.descriptor().name == "receiver.database")
                .expect("operational snapshot");
            let index = operational
                .descriptor()
                .metrics
                .iter()
                .position(|metric| metric.name == "query.failures")
                .expect("query failure counter");
            assert_eq!(operational.get_metrics()[index], MetricValue::U64(expected));
        }
    }

    /// Scenario: Collections succeed or fail under each count/duration policy, separated by delivery waits.
    /// Guarantees: Only enabled shared sets emit, downstream failures do not change collection outcomes,
    /// and processing duration excludes idle, feedback, checkpoint, and cleanup time.
    #[tokio::test(start_paused = true)]
    async fn collection_policy_and_delivery_boundaries_are_independent() {
        for interests in [
            Interests::empty(),
            Interests::NODE_OUTPUT_METRICS,
            Interests::NODE_LOCAL_DURATION,
            Interests::NODE_OUTPUT_METRICS | Interests::NODE_LOCAL_DURATION,
        ] {
            let (pipeline, _registry) = test_pipeline_ctx_with_interests(interests);
            let mut metrics = DatabaseReceiverMetrics::register(&pipeline);
            tokio::time::sleep(Duration::from_secs(10)).await;
            _ = metrics.begin_collection();
            tokio::time::sleep(Duration::from_secs(2)).await;
            metrics.collection_succeeded();
            metrics.record_feedback(Outcome::Failure);
            metrics.record_checkpoint(Outcome::Failure);
            tokio::time::sleep(Duration::from_secs(20)).await;
            _ = metrics.begin_collection();
            tokio::time::sleep(Duration::from_secs(3)).await;
            metrics.collection_failed();
            tokio::time::sleep(Duration::from_secs(30)).await;
            metrics.collection_succeeded();
            let snapshots = metrics.terminal_snapshots();
            let pulled: Vec<_> = snapshots
                .iter()
                .filter(|snapshot| snapshot.descriptor().name == "receiver.pulled")
                .collect();
            assert_eq!(
                pulled.len(),
                2 * usize::from(interests.contains(Interests::NODE_OUTPUT_METRICS))
            );
            for snapshot in pulled {
                assert_eq!(snapshot.get_metrics(), &[MetricValue::U64(1)]);
                assert!(matches!(
                    snapshot.measurement_attribute_value("outcome"),
                    Some("success" | "failure")
                ));
                assert_eq!(snapshot.measurement_attribute_value("signal"), None);
            }
            let durations: Vec<_> = snapshots
                .iter()
                .filter(|snapshot| snapshot.descriptor().name == "receiver.processing")
                .collect();
            assert_eq!(
                durations.len(),
                usize::from(interests.contains(Interests::NODE_LOCAL_DURATION))
            );
            for snapshot in durations {
                let MetricValue::Distribution(duration) = &snapshot.get_metrics()[0] else {
                    panic!("processing duration distribution");
                };
                assert_eq!(duration.summary(), (2, 5.0, 2.0, 3.0));
                assert_eq!(snapshot.measurement_attribute_value("signal"), None);
                assert_eq!(snapshot.measurement_attribute_value("outcome"), None);
            }
        }
    }
}
