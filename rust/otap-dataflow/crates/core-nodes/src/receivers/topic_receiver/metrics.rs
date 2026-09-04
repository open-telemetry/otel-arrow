// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the topic receiver node.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

use otel_arrow_dfe_telemetry::common_attributes::OutcomeAttributes;

/// Forward metrics for the topic receiver.
#[metric_set(
    name = "receiver.topic.forward",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TopicForwardMetrics {
    /// Number of messages forwarded.
    #[metric(unit = "{message}")]
    pub messages: Counter<u64>,
}

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

/// Other un-dimensioned metrics for the topic receiver.
#[metric_set(name = "receiver.topic.other")]
#[derive(Debug, Default, Clone)]
pub struct TopicOtherMetrics {
    /// Total messages missed across lag notifications.
    #[metric(unit = "{message}")]
    pub lagged_messages: Counter<u64>,
    /// Number of downstream backpressure events (>= 500ms blocked).
    #[metric(unit = "{event}")]
    pub downstream_backpressure_events: Counter<u64>,
    /// Total milliseconds blocked while forwarding to downstream.
    #[metric(unit = "ms")]
    pub downstream_blocked_ms: Counter<u64>,
}

/// Topic receiver metrics collection.
pub struct TopicReceiverMetrics {
    /// Forward metrics.
    pub forward: MeasurementMetricSet<TopicForwardMetrics>,
    /// Lag event metrics.
    pub lag_events: MeasurementMetricSet<TopicLagEventMetrics>,
    /// Bridge control metrics.
    pub bridge: MeasurementMetricSet<TopicBridgeMetrics>,
    /// Other un-dimensioned metrics.
    pub other: MetricSet<TopicOtherMetrics>,
}

impl TopicReceiverMetrics {
    /// Registers topic receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext, topic_name: String) -> Self {
        Self {
            forward: pipeline_ctx.register_measurement_metrics_with_topic::<TopicForwardMetrics>(
                topic_name.clone().into(),
            ),
            lag_events: pipeline_ctx
                .register_measurement_metrics_with_topic::<TopicLagEventMetrics>(
                    topic_name.clone().into(),
                ),
            bridge: pipeline_ctx.register_measurement_metrics_with_topic::<TopicBridgeMetrics>(
                topic_name.clone().into(),
            ),
            other: pipeline_ctx.register_metrics_with_topic::<TopicOtherMetrics>(topic_name.into()),
        }
    }

    /// Takes every touched metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.forward.terminal_snapshots();
        snapshots.extend(self.lag_events.terminal_snapshots());
        snapshots.extend(self.bridge.terminal_snapshots());
        snapshots.extend(self.other.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        reporter.report_measurement(&mut self.forward)?;
        reporter.report_measurement(&mut self.lag_events)?;
        reporter.report_measurement(&mut self.bridge)?;
        reporter.report(&mut self.other)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::common_attributes::Outcome;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    fn new_test_metrics() -> TopicReceiverMetrics {
        let handle = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(handle);
        let pipeline_ctx =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        TopicReceiverMetrics::register(&pipeline_ctx, "test-topic".into())
    }

    #[test]
    fn receiver_metrics_are_partitioned_by_context() {
        let mut metrics = new_test_metrics();

        metrics
            .forward
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .messages
            .add(1);
        metrics
            .forward
            .with(OutcomeAttributes {
                outcome: Outcome::Failure,
            })
            .messages
            .add(2);

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
                .forward
                .get(OutcomeAttributes {
                    outcome: Outcome::Success
                })
                .messages
                .get(),
            1
        );
        assert_eq!(
            metrics
                .forward
                .get(OutcomeAttributes {
                    outcome: Outcome::Failure
                })
                .messages
                .get(),
            2
        );

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

    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let mut metrics = new_test_metrics();
        metrics
            .forward
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .messages
            .add(1);
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
        metrics.other.lagged_messages.add(42);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 4);

        for snapshot in &snapshots {
            println!("Metric name: {}", snapshot.descriptor().name);
            for attr in snapshot.measurement_attributes() {
                println!("  Attribute: {} = {:?}", attr.0, attr.1);
            }
        }

        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.topic.forward"
                && snapshot.measurement_attribute_value("outcome") == Some("success")
        }));
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
                .any(|snapshot| { snapshot.descriptor().name == "receiver.topic.other" })
        );

        let second = metrics.terminal_snapshots();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].descriptor().name, "receiver.topic.other");
    }
}
