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
