// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::OutcomeAttributes;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Transition types for journald receiver lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum TransitionType {
    /// Started.
    Start,
    /// Drained.
    Drain,
    /// Shut down.
    Shutdown,
}

/// Attributes for transition metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct TransitionAttributes {
    /// Type of transition.
    pub transition_type: TransitionType,
}

/// Lifecycle metrics for the journald receiver.
#[metric_set(
    name = "receiver.journald.lifecycle",
    measurement_attributes = TransitionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct JournaldLifecycleMetrics {
    /// Number of lifecycle transitions.
    #[metric(unit = "{transition}")]
    pub transitions: Counter<u64>,
}

/// Acknowledgement metrics for downstream responses.
#[metric_set(
    name = "receiver.journald.acknowledgements",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct JournaldAcknowledgmentMetrics {
    /// Number of downstream responses.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
    /// Number of rewinds triggered by NACKs.
    #[metric(unit = "{rewind}")]
    pub rewinds: Counter<u64>,
}

/// Checkpoint metrics for durable cursor commits.
#[metric_set(
    name = "receiver.journald.checkpoints",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct JournaldCheckpointMetrics {
    /// Number of durable cursor commits.
    #[metric(unit = "{commit}")]
    pub commits: Counter<u64>,
}

/// Error types for journald source read failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
#[allow(dead_code)]
pub enum SourceErrorType {
    /// Permission denied.
    Permission,
    /// Corrupt journal.
    CorruptJournal,
    /// I/O failure.
    IoFailure,
    /// Other read failures.
    Other,
}

/// Attributes for source error events.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SourceErrorAttributes {
    /// Type of source error.
    #[attribute_key = "error.type"]
    pub error_type: SourceErrorType,
}

/// Source event metrics.
#[metric_set(
    name = "receiver.journald.source",
    measurement_attributes = SourceErrorAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct JournaldSourceErrorMetrics {
    /// Number of source read failures.
    #[metric(unit = "{event}")]
    pub events: Counter<u64>,
}

/// Output and extraction metrics for the journald receiver.
#[metric_set(name = "receiver.journald.output")]
#[derive(Debug, Default, Clone)]
pub struct JournaldOutputMetrics {
    /// Number of log batches emitted downstream.
    #[metric(unit = "{batch}")]
    pub batches: Counter<u64>,
    /// Number of log records emitted downstream.
    #[metric(unit = "{record}")]
    pub records: Counter<u64>,
    /// Number of journald fields dropped by extraction limits.
    #[metric(unit = "{field}")]
    pub dropped_fields: Counter<u64>,
}

/// Journald receiver metrics collection.
pub struct JournaldReceiverMetrics {
    /// Lifecycle transition metrics.
    pub lifecycle: MeasurementMetricSet<JournaldLifecycleMetrics>,
    /// Acknowledgement metrics.
    pub acknowledgements: MeasurementMetricSet<JournaldAcknowledgmentMetrics>,
    /// Checkpoint metrics.
    pub checkpoints: MeasurementMetricSet<JournaldCheckpointMetrics>,
    /// Source error metrics.
    pub source_errors: MeasurementMetricSet<JournaldSourceErrorMetrics>,
    /// Output throughput and extraction metrics.
    pub output: MetricSet<JournaldOutputMetrics>,
}

impl JournaldReceiverMetrics {
    /// Registers the metric sets with the pipeline context.
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            lifecycle: JournaldLifecycleMetrics::register(pipeline_ctx),
            acknowledgements: JournaldAcknowledgmentMetrics::register(pipeline_ctx),
            checkpoints: JournaldCheckpointMetrics::register(pipeline_ctx),
            source_errors: JournaldSourceErrorMetrics::register(pipeline_ctx),
            output: pipeline_ctx.register_metrics::<JournaldOutputMetrics>(),
        }
    }

    /// Snapshots all metric sets.
    pub fn snapshot(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.lifecycle.terminal_snapshots();
        snapshots.extend(self.acknowledgements.terminal_snapshots());
        snapshots.extend(self.checkpoints.terminal_snapshots());
        snapshots.extend(self.source_errors.terminal_snapshots());
        if self.output.needs_flush() {
            snapshots.push(self.output.snapshot());
        }
        snapshots
    }

    /// Reports touched metric buckets.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), Error> {
        reporter.report_measurement(&mut self.lifecycle)?;
        reporter.report_measurement(&mut self.acknowledgements)?;
        reporter.report_measurement(&mut self.checkpoints)?;
        reporter.report_measurement(&mut self.source_errors)?;
        reporter.report(&mut self.output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::common_attributes::Outcome;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    #[test]
    fn test_journald_receiver_metrics() {
        let registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut metrics = JournaldReceiverMetrics::register(&pipeline_ctx);

        // Record some lifecycle metrics
        metrics
            .lifecycle
            .with(TransitionAttributes {
                transition_type: TransitionType::Start,
            })
            .transitions
            .add(1);
        metrics
            .lifecycle
            .with(TransitionAttributes {
                transition_type: TransitionType::Start,
            })
            .transitions
            .add(2);

        // Record completions
        metrics
            .acknowledgements
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .responses
            .add(5);
        metrics
            .acknowledgements
            .with(OutcomeAttributes {
                outcome: Outcome::Refused,
            })
            .responses
            .add(2);
        metrics
            .acknowledgements
            .with(OutcomeAttributes {
                outcome: Outcome::Refused,
            })
            .rewinds
            .add(1);

        // Record output
        metrics.output.batches.add(10);
        metrics.output.records.add(100);

        let snapshots = metrics.snapshot();

        // Assertions
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.journald.lifecycle"
                && s.measurement_attribute_value("transition.type") == Some("start")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.journald.acknowledgements"
                && s.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(
            snapshots
                .iter()
                .any(|s| { s.descriptor().name == "receiver.journald.output" })
        );

        let snapshots2 = metrics.snapshot();
        // Terminal snapshots from MeasurementMetricSet should only be returned once,
        // so they should not appear in the second snapshot.
        assert!(
            !snapshots2
                .iter()
                .any(|s| s.descriptor().name == "receiver.journald.lifecycle")
        );
        assert!(
            !snapshots2
                .iter()
                .any(|s| s.descriptor().name == "receiver.journald.acknowledgements")
        );
    }
}
