// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the topic receiver node.

use otel_arrow_dfe_config::policy::DistributionTier;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ReceiverMetrics;
use otel_arrow_dfe_telemetry::instrument::{Counter, HistogramDetailed, HistogramNormal, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetRegistrar, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Duration;

const DOWNSTREAM_BACKPRESSURE_WARNING_THRESHOLD: Duration = Duration::from_millis(500);

/// Lag events for topic receiver broadcast subscriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum LagEventType {
    /// Lag notification emitted.
    Notification,
    /// Subscription disconnected because of lag.
    Disconnect,
}

/// Attributes for lag events.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct LagEventAttributes {
    /// Type of the lag event.
    pub event_type: LagEventType,
}

/// Lag event metrics for the topic receiver.
#[metric_set(
    name = "receiver.topic.lag",
    measurement_attributes = LagEventAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TopicLagEventMetrics {
    /// Number of lag events.
    #[metric(unit = "{event}")]
    pub events: Counter<u64>,
}

/// Downstream control type bridged to topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum BridgeControl {
    /// Downstream ACK.
    Ack,
    /// Downstream NACK.
    Nack,
}

/// Bridge result for topic Ack/Nack controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum BridgeResult {
    /// Control successfully bridged.
    Success,
    /// Control ignored because Ack/Nack propagation is disabled.
    IgnoredPropagationDisabled,
    /// Control missing the bridged topic message id in calldata.
    MissingCalldata,
    /// Control carrying an id not tracked by the topic runtime.
    InvalidOrUntrackedId,
    /// Failed to bridge for some runtime reason other than an unknown id.
    RuntimeFailure,
}

/// Attributes for bridge controls.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct BridgeAttributes {
    /// The type of downstream control.
    pub control: BridgeControl,
    /// Result of the bridge control.
    pub result: BridgeResult,
}

/// Bridge control metrics for the topic receiver.
#[metric_set(
    name = "receiver.topic.bridge",
    measurement_attributes = BridgeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TopicBridgeMetrics {
    /// Number of bridge controls.
    #[metric(unit = "{control}")]
    pub controls: Counter<u64>,
}

/// General un-dimensioned metrics for the topic receiver.
#[metric_set(name = "receiver.topic")]
#[derive(Debug, Default, Clone)]
pub struct TopicGeneralMetrics {
    /// Total messages missed across lag notifications.
    #[metric(unit = "{message}")]
    pub lagged_messages: Counter<u64>,
    /// Number of downstream backpressure events (>= 500ms blocked).
    #[metric(unit = "{event}")]
    pub downstream_backpressure_events: Counter<u64>,
}

/// Basic-resolution downstream blocking duration.
#[metric_set(name = "receiver.topic.downstream.blocked")]
#[derive(Debug, Default, Clone)]
struct TopicBlockedDurationBasicMetrics {
    /// Time blocked while forwarding one message to downstream.
    #[metric(unit = "s")]
    duration: Mmsc,
}

/// Normal-resolution downstream blocking duration.
#[metric_set(name = "receiver.topic.downstream.blocked")]
#[derive(Debug, Default, Clone)]
struct TopicBlockedDurationNormalMetrics {
    /// Time blocked while forwarding one message to downstream.
    #[metric(unit = "s")]
    duration: HistogramNormal,
}

/// Detailed-resolution downstream blocking duration.
#[metric_set(name = "receiver.topic.downstream.blocked")]
#[derive(Debug, Default, Clone)]
struct TopicBlockedDurationDetailedMetrics {
    /// Time blocked while forwarding one message to downstream.
    #[metric(unit = "s")]
    duration: HistogramDetailed,
}

enum TopicBlockedDurationMetricSet {
    Basic(MetricSet<TopicBlockedDurationBasicMetrics>),
    Normal(MetricSet<TopicBlockedDurationNormalMetrics>),
    Detailed(MetricSet<TopicBlockedDurationDetailedMetrics>),
}

impl TopicBlockedDurationMetricSet {
    fn register(registrar: &impl MetricSetRegistrar, tier: DistributionTier) -> Self {
        match tier {
            DistributionTier::Basic => {
                Self::Basic(TopicBlockedDurationBasicMetrics::register(registrar))
            }
            DistributionTier::Normal => {
                Self::Normal(TopicBlockedDurationNormalMetrics::register(registrar))
            }
            DistributionTier::Detailed => {
                Self::Detailed(TopicBlockedDurationDetailedMetrics::register(registrar))
            }
        }
    }

    fn record(&mut self, duration: Duration) {
        let value = duration.as_secs_f64();
        match self {
            Self::Basic(metrics) => metrics.duration.record(value),
            Self::Normal(metrics) => metrics.duration.record(value),
            Self::Detailed(metrics) => metrics.duration.record(value),
        }
    }

    fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        match self {
            Self::Basic(metrics) => reporter.report(metrics),
            Self::Normal(metrics) => reporter.report(metrics),
            Self::Detailed(metrics) => reporter.report(metrics),
        }
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        match self {
            Self::Basic(metrics) => metrics.terminal_snapshots(),
            Self::Normal(metrics) => metrics.terminal_snapshots(),
            Self::Detailed(metrics) => metrics.terminal_snapshots(),
        }
    }
}

/// Topic receiver metrics collection.
pub struct TopicReceiverMetrics {
    /// Shared receiver boundary metrics.
    pub boundary: ReceiverMetrics,
    /// Lag event metrics.
    pub lag_events: MeasurementMetricSet<TopicLagEventMetrics>,
    /// Bridge control metrics.
    pub bridge: MeasurementMetricSet<TopicBridgeMetrics>,
    /// General un-dimensioned metrics.
    pub general: MetricSet<TopicGeneralMetrics>,
    blocked_duration: Option<TopicBlockedDurationMetricSet>,
}

impl TopicReceiverMetrics {
    /// Registers topic receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext, topic_name: String) -> Self {
        let registrar = pipeline_ctx.metric_set_registrar_with_topic(topic_name.into());
        let interests = pipeline_ctx.node_interests();
        let duration_distribution = pipeline_ctx.node_duration_distribution();
        Self {
            boundary: ReceiverMetrics::register_with_distribution(
                &registrar,
                interests,
                duration_distribution,
            ),
            lag_events: TopicLagEventMetrics::register(&registrar),
            bridge: TopicBridgeMetrics::register(&registrar),
            general: TopicGeneralMetrics::register(&registrar),
            blocked_duration: interests.contains(Interests::NODE_LOCAL_DURATION).then(|| {
                TopicBlockedDurationMetricSet::register(&registrar, duration_distribution)
            }),
        }
    }

    /// Takes every touched metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        snapshots.extend(self.lag_events.terminal_snapshots());
        snapshots.extend(self.bridge.terminal_snapshots());
        snapshots.extend(self.general.terminal_snapshots());
        if let Some(blocked_duration) = &mut self.blocked_duration {
            snapshots.extend(blocked_duration.terminal_snapshots());
        }
        snapshots
    }

    /// Reports touched metric buckets.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        self.boundary.report(reporter)?;
        reporter.report_measurement(&mut self.lag_events)?;
        reporter.report_measurement(&mut self.bridge)?;
        reporter.report(&mut self.general)?;
        if let Some(blocked_duration) = &mut self.blocked_duration {
            blocked_duration.report(reporter)?;
        }
        Ok(())
    }

    /// Records a bridge operation metric.
    #[inline]
    pub fn record_bridge(&mut self, control: BridgeControl, result: BridgeResult) {
        self.bridge
            .with(BridgeAttributes { control, result })
            .controls
            .add(1);
    }

    /// Records one downstream channel wait and returns whether it crossed the
    /// backpressure warning threshold.
    pub fn record_downstream_blocked(&mut self, duration: Duration) -> bool {
        if let Some(blocked_duration) = &mut self.blocked_duration {
            blocked_duration.record(duration);
        }
        let backpressured = duration >= DOWNSTREAM_BACKPRESSURE_WARNING_THRESHOLD;
        if backpressured {
            self.general.downstream_backpressure_events.inc();
        }
        backpressured
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_config::SignalType;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::testing::{
        test_pipeline_ctx_with_interests,
        test_pipeline_ctx_with_interests_and_duration_distribution,
    };
    use otel_arrow_dfe_otap::metrics::ErrorWithOutcome;
    use otel_arrow_dfe_telemetry::metrics::MetricValue;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> TopicReceiverMetrics {
        let handle = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(handle);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        TopicReceiverMetrics::register(&pipeline_ctx, "test-topic".into())
    }

    /// Scenario: A topic receiver is instantiated and produces metrics for various events.
    /// Guarantees: Metrics are properly grouped by their enum attributes into the right dimensions.
    #[test]
    fn receiver_metrics_are_partitioned_by_context() {
        let mut metrics = new_test_metrics();

        metrics
            .bridge
            .with(BridgeAttributes {
                control: BridgeControl::Ack,
                result: BridgeResult::Success,
            })
            .controls
            .add(3);
        metrics
            .bridge
            .with(BridgeAttributes {
                control: BridgeControl::Nack,
                result: BridgeResult::InvalidOrUntrackedId,
            })
            .controls
            .add(4);

        assert_eq!(
            metrics
                .bridge
                .get(BridgeAttributes {
                    control: BridgeControl::Ack,
                    result: BridgeResult::Success
                })
                .controls
                .get(),
            3
        );
        assert_eq!(
            metrics
                .bridge
                .get(BridgeAttributes {
                    control: BridgeControl::Nack,
                    result: BridgeResult::InvalidOrUntrackedId
                })
                .controls
                .get(),
            4
        );
    }

    /// Scenario: A topic delivery completes receiver-local processing before downstream forwarding.
    /// Guarantees: The shared receiver boundary records one successful message for the delivered signal.
    #[test]
    fn receiver_boundary_records_topic_delivery() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::NODE_OUTPUT_METRICS);
        let mut metrics = TopicReceiverMetrics::register(&pipeline_ctx, "test-topic".into());
        let completed = metrics
            .boundary
            .processing()
            .run(|_| Ok::<_, ErrorWithOutcome<std::convert::Infallible>>((SignalType::Logs, ())));
        metrics
            .boundary
            .record(completed)
            .expect("topic message processing is infallible");

        let snapshots = metrics.terminal_snapshots();
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .position(|metric| metric.name == "messages")
                    .is_some_and(|index| snapshot.get_metrics()[index].to_u64_lossy() == 1)
        }));
    }

    /// Scenario: A topic receiver waits for downstream capacity with each duration tier enabled.
    /// Guarantees: The wait uses the configured distribution tier and crossing 500ms records one backpressure event.
    #[test]
    fn downstream_blocking_uses_configured_duration_distribution() {
        for (tier, expected_tier) in [
            (DistributionTier::Basic, "basic"),
            (DistributionTier::Normal, "normal"),
            (DistributionTier::Detailed, "detailed"),
        ] {
            let (pipeline_ctx, _) = test_pipeline_ctx_with_interests_and_duration_distribution(
                Interests::NODE_LOCAL_DURATION,
                tier,
            );
            let mut metrics = TopicReceiverMetrics::register(&pipeline_ctx, "test-topic".into());

            assert!(metrics.record_downstream_blocked(Duration::from_millis(600)));
            assert_eq!(metrics.general.downstream_backpressure_events.get(), 1);

            let snapshot = metrics
                .terminal_snapshots()
                .into_iter()
                .find(|snapshot| snapshot.descriptor().name == "receiver.topic.downstream.blocked")
                .expect("downstream blocked duration snapshot");
            let [MetricValue::Distribution(value)] = snapshot.get_metrics() else {
                panic!("expected downstream blocked duration distribution");
            };
            assert_eq!(
                value.tier_name(),
                expected_tier,
                "configured tier: {tier:?}"
            );
            assert_eq!(metrics.general.downstream_backpressure_events.get(), 0);
        }
    }

    /// Scenario: A topic receiver waits for downstream capacity without duration telemetry enabled.
    /// Guarantees: The warning event remains active while no blocked-duration distribution is emitted.
    #[test]
    fn downstream_blocking_duration_is_interest_gated() {
        let (pipeline_ctx, _) = test_pipeline_ctx_with_interests(Interests::empty());
        let mut metrics = TopicReceiverMetrics::register(&pipeline_ctx, "test-topic".into());

        assert!(metrics.record_downstream_blocked(Duration::from_millis(600)));
        assert_eq!(metrics.general.downstream_backpressure_events.get(), 1);

        let snapshots = metrics.terminal_snapshots();
        assert!(
            snapshots
                .iter()
                .all(|snapshot| snapshot.descriptor().name != "receiver.topic.downstream.blocked")
        );
    }

    /// Scenario: A terminal snapshot is taken from the topic receiver.
    /// Guarantees: Terminal snapshots capture the measurement attributes properly and clear the buffers so subsequent snapshots are empty.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_test_metrics();
        metrics
            .bridge
            .with(BridgeAttributes {
                control: BridgeControl::Ack,
                result: BridgeResult::Success,
            })
            .controls
            .add(1);
        metrics
            .lag_events
            .with(LagEventAttributes {
                event_type: LagEventType::Disconnect,
            })
            .events
            .add(1);
        metrics.general.lagged_messages.add(42);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);

        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.topic.bridge"
                && snapshot.measurement_attribute_value("control") == Some("ack")
                && snapshot.measurement_attribute_value("result") == Some("success")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.topic.lag"
                && snapshot.measurement_attribute_value("event.type") == Some("disconnect")
        }));
        assert!(
            snapshots
                .iter()
                .any(|snapshot| { snapshot.descriptor().name == "receiver.topic" })
        );

        let second = metrics.terminal_snapshots();
        assert_eq!(second.len(), 1);
        assert!(
            second
                .iter()
                .any(|snapshot| { snapshot.descriptor().name == "receiver.topic" })
        );
    }
}
