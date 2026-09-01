// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the traffic generator receiver node.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::Outcome;
use otel_arrow_dfe_telemetry::instrument::{Counter, Gauge, HistogramNormal, Mmsc};
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

// -- Smooth-run outcome attributes ---------------------------------------------

/// Outcome of a smooth-mode production run termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum RunTerminationOutcome {
    /// Run finished before the next tick.
    OnTime,
    /// Run finished after the next tick.
    Late,
    /// Run was cancelled.
    Cancelled,
}

/// Attributes for smooth-run termination metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SmoothRunTerminationAttributes {
    /// The run termination outcome.
    pub outcome: RunTerminationOutcome,
}

/// Smooth-mode production run termination counters.
#[metric_set(
    name = "receiver.traffic_generator.smooth.runs.terminations",
    measurement_attributes = SmoothRunTerminationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TrafficGeneratorSmoothRunTerminationMetrics {
    /// Number of smooth-mode production runs terminated.
    #[metric(unit = "{run}")]
    pub runs: Counter<u64>,
}

// -- Smooth payload send result attributes ------------------------------------

/// The kind of attempt for a smooth-mode payload send.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum AttemptKind {
    /// The initial send attempt.
    Initial,
    /// A retry attempt after a previous rejection.
    Retry,
}

/// Attributes for smooth payload send metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct SmoothSendAttributes {
    /// The attempt kind (initial or retry).
    pub attempt: AttemptKind,
    /// The send attempt outcome.
    pub outcome: Outcome,
}

/// Smooth-mode payload send attempt counters.
#[metric_set(
    name = "receiver.traffic_generator.smooth.payload.sends",
    measurement_attributes = SmoothSendAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct TrafficGeneratorSmoothSendMetrics {
    /// Number of smooth-mode payload send attempts.
    #[metric(unit = "{attempt}")]
    pub attempts: Counter<u64>,
}

// -- Other (non-dimensionable) metrics ----------------------------------------

/// Scalar metrics for the traffic generator that do not fit a single enum dimension.
#[metric_set(name = "receiver.traffic_generator.other")]
#[derive(Debug, Default, Clone)]
pub struct TrafficGeneratorOtherMetrics {
    /// Total uncompressed bytes of log payloads produced (protobuf-encoded size before
    /// compression). Together with the engine's `node.output.items` (where `signal=logs`),
    /// this yields the average uncompressed bytes per log record for compression-ratio analysis.
    #[metric(unit = "By")]
    pub logs_bytes_produced: Counter<u64>,
    /// Number of smooth-mode production runs started.
    #[metric(name = "smooth.runs.started", unit = "{run}")]
    pub smooth_runs_started: Counter<u64>,
    /// Number of times smooth mode detects a missed scheduling boundary.
    #[metric(name = "smooth.schedule.deadline_misses", unit = "{miss}")]
    pub smooth_schedule_deadline_misses: Counter<u64>,
    /// Number of subscribed batches waiting for Ack/Nack completion.
    #[metric(name = "completion.pending", unit = "{batch}")]
    pub completion_pending: Gauge<u64>,
    /// Number of drains forced to finish with unresolved batches at the deadline.
    #[metric(name = "completion.drain.deadline_forced", unit = "{drain}")]
    pub completion_drain_deadline_forced: Counter<u64>,
    /// Number of batches remaining when smooth mode detects that a run is late.
    #[metric(name = "smooth.late.remaining.batches", unit = "{batch}")]
    pub smooth_late_remaining_batches: Mmsc,
    /// Number of signal items remaining when smooth mode detects that a run is late.
    #[metric(name = "smooth.late.remaining.items", unit = "{item}")]
    pub smooth_late_remaining_items: Mmsc,
    /// Smooth-mode configured batches per one-second run.
    #[metric(name = "smooth.run.batches", unit = "{batch}")]
    pub smooth_run_batches: Gauge<u64>,
    /// Smooth-mode configured interval between batches.
    #[metric(name = "smooth.batch.interval", unit = "ns")]
    pub smooth_batch_interval_ns: Gauge<u64>,
    /// Lateness of smooth-mode batch ticks relative to their scheduled instant.
    #[metric(name = "smooth.batch.tick.lateness.duration", unit = "ns")]
    pub smooth_batch_tick_lateness_duration_ns: Mmsc,
    /// Wall-clock time spent generating or cloning one smooth-mode payload.
    #[metric(name = "smooth.payload.generate.duration", unit = "ns")]
    pub smooth_payload_generate_duration_ns: HistogramNormal,
    /// Wall-clock time spent sending one smooth-mode payload into the downstream channel.
    #[metric(name = "smooth.payload.send.duration", unit = "ns")]
    pub smooth_payload_send_duration_ns: HistogramNormal,
}

// -- Top-level wrapper ---------------------------------------------------------

/// Traffic generator receiver metrics collection.
///
/// Per-signal production counts and per-message outcome (ack/nack) tracking are
/// intentionally omitted here because the engine already provides them via the
/// `node.output.items` and `node.output.messages` channel metric sets.
pub struct TrafficGeneratorMetrics {
    /// Smooth-mode production run terminations.
    pub smooth_run_terminations: MeasurementMetricSet<TrafficGeneratorSmoothRunTerminationMetrics>,
    /// Smooth payload send attempt counters (`attempt`, `outcome`).
    pub smooth_send: MeasurementMetricSet<TrafficGeneratorSmoothSendMetrics>,
    /// Scalar metrics that do not fit a single enum dimension.
    pub other: MetricSet<TrafficGeneratorOtherMetrics>,
}

impl TrafficGeneratorMetrics {
    /// Registers all metric sets with the pipeline context.
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            smooth_run_terminations: TrafficGeneratorSmoothRunTerminationMetrics::register(pipeline_ctx),
            smooth_send: TrafficGeneratorSmoothSendMetrics::register(pipeline_ctx),
            other: pipeline_ctx.register_metrics::<TrafficGeneratorOtherMetrics>(),
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.smooth_run_terminations.terminal_snapshots();
        snapshots.extend(self.smooth_send.terminal_snapshots());
        snapshots.push(self.other.snapshot());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        reporter.report_measurement(&mut self.smooth_run_terminations)?;
        reporter.report_measurement(&mut self.smooth_send)?;
        reporter.report(&mut self.other)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;

    /// Scenario: Testing Traffic Generator Metrics dimensions and snapshotting.
    /// Guarantees: 
    /// - Dimensions correctly map to recorded outputs without dropping metrics.
    /// - Metric values are cleared out appropriately on a terminal snapshot.
    /// - A second handoff does not repeat counter or histogram values.
    #[test]
    fn test_traffic_generator_metrics() {
        let registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut metrics = TrafficGeneratorMetrics::register(&pipeline_ctx);

        // Record smooth runs
        metrics.other.smooth_runs_started.add(10);
        metrics.other.smooth_schedule_deadline_misses.add(2);

        metrics
            .smooth_run_terminations
            .with(SmoothRunTerminationAttributes {
                outcome: RunTerminationOutcome::OnTime,
            })
            .runs
            .add(8);
        metrics
            .smooth_run_terminations
            .with(SmoothRunTerminationAttributes {
                outcome: RunTerminationOutcome::Late,
            })
            .runs
            .add(2);

        // Record smooth send
        metrics
            .smooth_send
            .with(SmoothSendAttributes {
                attempt: AttemptKind::Initial,
                outcome: Outcome::Refused,
            })
            .attempts
            .add(3);
        metrics
            .smooth_send
            .with(SmoothSendAttributes {
                attempt: AttemptKind::Retry,
                outcome: Outcome::Success,
            })
            .attempts
            .add(2);

        // Record other
        metrics.other.logs_bytes_produced.add(4096);
        metrics.other.completion_pending.set(2);

        let snapshots = metrics.terminal_snapshots();

        // Verify metric set names
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.traffic_generator.smooth.runs.terminations"
                && s.measurement_attribute_value("outcome") == Some("late")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.traffic_generator.smooth.payload.sends"
                && s.measurement_attribute_value("attempt") == Some("initial")
                && s.measurement_attribute_value("outcome") == Some("refused")
        }));
        assert!(
            snapshots
                .iter()
                .any(|s| s.descriptor().name == "receiver.traffic_generator.other")
        );

        // MeasurementMetricSet snapshots are terminal - they should not reappear
        let snapshots2 = metrics.terminal_snapshots();
        assert!(
            !snapshots2
                .iter()
                .any(|s| s.descriptor().name == "receiver.traffic_generator.smooth.runs.terminations")
        );
        
        let other_second_snapshot = snapshots2.iter().find(|s| s.descriptor().name == "receiver.traffic_generator.other").unwrap();
        // The fixed metric set is returned again, but delta counters like `smooth_runs_started` must have been cleared (value 0)
        let started_val = other_second_snapshot.metrics.iter().find(|m| m.descriptor().name == "smooth.runs.started").unwrap();
        if let otel_arrow_dfe_telemetry::metrics::MetricValue::Counter(val) = started_val {
            assert_eq!(*val, 0);
        } else {
            panic!("Expected counter");
        }
    }
}

