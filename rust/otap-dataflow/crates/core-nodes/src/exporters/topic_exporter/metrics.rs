// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the topic exporter.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Bounded reason why the topic exporter refused a publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum TopicRejectionReason {
    /// The topic queue was full and the configured policy dropped the publish.
    QueueFull,
    /// The tracked-publish outcome capacity was exhausted.
    OutcomeCapacity,
    /// Shutdown interrupted the publish before topic admission.
    Shutdown,
}

/// Bounded result observed for an admitted tracked publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum TopicTrackedResult {
    /// Downstream processing acknowledged the publish.
    Ack,
    /// Downstream processing negatively acknowledged the publish.
    Nack,
    /// The tracked publish exceeded its configured outcome timeout.
    Timeout,
    /// The topic closed before the tracked publish resolved.
    TopicClosed,
    /// Exporter shutdown interrupted the tracked publish.
    Shutdown,
}

/// Signal and rejection reason for refused topic publishes.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct TopicRejectionAttributes {
    signal: SignalType,
    reason: TopicRejectionReason,
}

/// Topic publishes refused before admission.
#[metric_set(
    name = "exporter.topic.rejections",
    measurement_attributes = TopicRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
struct TopicRejectionMetrics {
    /// Number of topic publishes refused for the classified reason.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Signal and terminal result for admitted tracked publishes.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct TopicTrackedAttributes {
    signal: SignalType,
    result: TopicTrackedResult,
}

/// Terminal results bridged from admitted tracked publishes.
#[metric_set(
    name = "exporter.topic.tracked",
    measurement_attributes = TopicTrackedAttributes
)]
#[derive(Debug, Default, Clone)]
struct TopicTrackedMetrics {
    /// Number of admitted tracked publishes reaching the classified result.
    #[metric(unit = "{message}")]
    messages: Counter<u64>,
}

/// Current topic exporter state.
#[metric_set(name = "exporter.topic")]
#[derive(Debug, Default, Clone)]
struct TopicOperationalMetrics {
    /// Number of admitted tracked publishes waiting for a terminal result.
    #[metric(unit = "{message}")]
    tracked_in_flight: Gauge<u64>,
}

/// Shared boundary and topic-specific metrics emitted by a topic exporter.
pub(super) struct TopicExporterMetrics {
    pub(super) boundary: ExporterMetrics,
    rejections: MeasurementMetricSet<TopicRejectionMetrics>,
    tracked: MeasurementMetricSet<TopicTrackedMetrics>,
    operational: MetricSet<TopicOperationalMetrics>,
}

impl TopicExporterMetrics {
    /// Registers topic exporter metrics with the configured topic entity.
    #[must_use]
    pub(super) fn register(pipeline_ctx: &PipelineContext, topic: String) -> Self {
        let registrar = pipeline_ctx.metric_set_registrar_with_topic(topic.into());
        Self {
            boundary: ExporterMetrics::register_with_distribution(
                &registrar,
                pipeline_ctx.node_interests(),
                pipeline_ctx.node_duration_distribution(),
            ),
            rejections: TopicRejectionMetrics::register(&registrar),
            tracked: TopicTrackedMetrics::register(&registrar),
            operational: TopicOperationalMetrics::register(&registrar),
        }
    }

    /// Records one refused topic publish.
    pub(super) fn record_rejection(&mut self, signal: SignalType, reason: TopicRejectionReason) {
        self.rejections
            .with(TopicRejectionAttributes { signal, reason })
            .messages
            .inc();
    }

    /// Records one terminal result for an admitted tracked publish.
    pub(super) fn record_tracked(&mut self, signal: SignalType, result: TopicTrackedResult) {
        self.tracked
            .with(TopicTrackedAttributes { signal, result })
            .messages
            .inc();
    }

    /// Updates the current number of admitted tracked publishes.
    pub(super) fn set_tracked_in_flight(&mut self, count: usize) {
        self.operational
            .tracked_in_flight
            .set(u64::try_from(count).unwrap_or(u64::MAX));
    }

    /// Reports shared boundary and topic-specific metric sets.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.rejections)?;
        reporter.report_measurement(&mut self.tracked)?;
        reporter.report(&mut self.operational)
    }

    /// Takes terminal snapshots of every touched metric bucket.
    #[must_use]
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots.extend(self.tracked.terminal_snapshots());
        snapshots.extend(self.operational.terminal_snapshots());
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;

    fn metric_value(
        snapshots: &[MetricSetSnapshot],
        set_name: &str,
        metric_name: &str,
        attributes: &[(&str, &str)],
    ) -> u64 {
        let snapshot = snapshots
            .iter()
            .find(|snapshot| {
                snapshot.descriptor().name == set_name
                    && attributes.iter().all(|(key, value)| {
                        snapshot.measurement_attribute_value(key) == Some(*value)
                    })
            })
            .expect("matching metric snapshot");
        let metric = snapshot
            .descriptor()
            .metrics
            .iter()
            .position(|metric| metric.name == metric_name)
            .expect("metric exists");
        snapshot.get_metrics()[metric].to_u64_lossy()
    }

    /// Scenario: Topic publishes succeed and are refused for distinct bounded reasons.
    /// Guarantees: Shared outcomes and topic-specific rejection reasons remain consistently partitioned.
    #[tokio::test]
    async fn attempts_and_rejections_use_bounded_outcome_buckets() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = TopicExporterMetrics::register(&pipeline_ctx, "raw".to_owned());

        let completed = metrics
            .boundary
            .attempt(SignalType::Logs)
            .run(async |_| Ok::<(), ErrorWithOutcome<()>>(()))
            .await;
        metrics
            .boundary
            .record(completed)
            .expect("attempt succeeds");

        for reason in [
            TopicRejectionReason::QueueFull,
            TopicRejectionReason::OutcomeCapacity,
            TopicRejectionReason::Shutdown,
        ] {
            let completed = metrics
                .boundary
                .attempt(SignalType::Logs)
                .run(async |attempt| Err::<(), _>(attempt.refused(reason)))
                .await;
            let result = metrics.boundary.record(completed);
            assert_eq!(result, Err(reason));
            metrics.record_rejection(SignalType::Logs, reason);
        }

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.attempted",
                "messages",
                &[("signal", "logs"), ("outcome", "success")],
            ),
            1
        );
        assert_eq!(
            metric_value(
                &snapshots,
                "exporter.attempted",
                "messages",
                &[("signal", "logs"), ("outcome", "refused")],
            ),
            3
        );
        for reason in ["queue_full", "outcome_capacity", "shutdown"] {
            assert_eq!(
                metric_value(
                    &snapshots,
                    "exporter.topic.rejections",
                    "messages",
                    &[("signal", "logs"), ("reason", reason)],
                ),
                1
            );
        }
    }

    /// Scenario: Admitted tracked publishes resolve through every supported terminal path.
    /// Guarantees: Each tracked result uses an isolated bounded attribute bucket.
    #[test]
    fn tracked_results_use_isolated_bounded_buckets() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_INPUT_METRICS);
        let mut metrics = TopicExporterMetrics::register(&pipeline_ctx, "raw".to_owned());

        for result in [
            TopicTrackedResult::Ack,
            TopicTrackedResult::Nack,
            TopicTrackedResult::Timeout,
            TopicTrackedResult::TopicClosed,
            TopicTrackedResult::Shutdown,
        ] {
            metrics.record_tracked(SignalType::Metrics, result);
        }

        let snapshots = metrics.terminal_snapshots();
        for result in ["ack", "nack", "timeout", "topic_closed", "shutdown"] {
            assert_eq!(
                metric_value(
                    &snapshots,
                    "exporter.topic.tracked",
                    "messages",
                    &[("signal", "metrics"), ("result", result)],
                ),
                1
            );
        }
    }
}
