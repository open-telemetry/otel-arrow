// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Windows ETW receiver.
//!
//! # Counter algebra
//!
//! There is a single ingress denominator:
//!
//! ```text
//! receiver.etw.events.received
//!     = receiver.etw.events.forwarded                             (events in successfully-sent batches)
//!     + receiver.etw.dropped.dropped{reason="forward_failed"}     (events in batches that failed to build/send)
//!     + receiver.etw.dropped.dropped{reason="slow_worker"}        (events dropped by internal backpressure)
//!     + receiver.etw.dropped.dropped{reason="memory_pressure"}    (events dropped due to memory limits)
//!     + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
//! ```
//!
//! Important operational notes:
//! * `receiver.etw.events.received`: Ingress denominator, counted on producer (`ProcessTrace`) callback
//!   before channel enqueue.
//! * `receiver.etw.decode.errors`: TDH decode/parsing failures. Note that decode failures are still
//!   forwarded with empty decoded fields, so they are included in both `received` and `forwarded`.
//! * `receiver.etw.kernel.events_lost`: Events lost inside Windows kernel ETW buffers before the receiver
//!   ever saw them (kernel buffer overrun). These come from a different boundary and are distinct from
//!   `receiver.etw.events.received`.
//! * Slow-worker drop rate = `receiver.etw.dropped.dropped{reason="slow_worker"} / receiver.etw.events.received`
//! * Forward-failure rate = `receiver.etw.dropped.dropped{reason="forward_failed"} / receiver.etw.events.received`

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

// -- Delivery accounting metrics ----------------------------------------------

/// Event metrics for the ETW receiver tracking delivery accounting.
#[metric_set(name = "receiver.etw.events")]
#[derive(Debug, Default, Clone)]
pub struct EtwEventMetrics {
    /// Total events produced/received by the session before channel enqueue.
    /// This is the ingress denominator and authoritative total.
    #[metric(unit = "{event}")]
    pub received: Counter<u64>,

    /// Events successfully forwarded downstream in batches.
    #[metric(unit = "{event}")]
    pub forwarded: Counter<u64>,
}

// -- Decode quality metrics ---------------------------------------------------

/// Decode quality metrics for the ETW receiver.
#[metric_set(name = "receiver.etw.decode")]
#[derive(Debug, Default, Clone)]
pub struct EtwDecodeMetrics {
    /// Number of ETW events that failed TDH decode/parsing.
    /// Note that these events are still forwarded downstream with empty decoded fields.
    #[metric(unit = "{event}")]
    pub errors: Counter<u64>,
}

// -- Kernel loss metrics ------------------------------------------------------

/// Kernel-level event loss metrics for the ETW receiver.
#[metric_set(name = "receiver.etw.kernel")]
#[derive(Debug, Default, Clone)]
pub struct EtwKernelMetrics {
    /// Number of events lost inside kernel ETW buffers before delivery (buffer overrun).
    /// Distinct from receiver-level drops and not included in received total.
    #[metric(unit = "{event}")]
    pub events_lost: Counter<u64>,
}

// -- Drop reason attributes ---------------------------------------------------

/// Reason an ETW event was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum DropReason {
    /// Dropped because the consuming worker could not drain the internal channel fast enough.
    SlowWorker,
    /// Dropped because building or forwarding the batch downstream failed.
    ForwardFailed,
    /// Dropped due to process-wide memory pressure.
    MemoryPressure,
}

/// Attributes for ETW dropped event metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct DropAttributes {
    /// Reason the event was dropped.
    pub reason: DropReason,
}

/// Dropped event metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.dropped",
    measurement_attributes = DropAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct EtwDroppedMetrics {
    /// Number of dropped ETW events.
    #[metric(unit = "{event}")]
    pub dropped: Counter<u64>,
}

// -- Buffer outcome attributes ------------------------------------------------

/// Outcome/status of kernel ETW buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum BufferOutcome {
    /// Buffers written by the session.
    Written,
    /// Real-time delivery buffers lost because the consumer could not drain fast enough.
    RealTimeLost,
    /// Log buffers that could not be flushed.
    LogLost,
}

/// Attributes for kernel buffer metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct BufferAttributes {
    /// Outcome or status of the kernel buffer.
    pub outcome: BufferOutcome,
}

/// Kernel buffer metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.buffers",
    measurement_attributes = BufferAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct EtwBufferMetrics {
    /// Number of kernel ETW buffers.
    #[metric(unit = "{buffer}")]
    pub buffers: Counter<u64>,
}

// -- Top-level wrapper --------------------------------------------------------

/// ETW receiver metrics collection.
pub struct EtwReceiverMetrics {
    /// Delivery accounting event metrics (received, forwarded).
    pub events: MetricSet<EtwEventMetrics>,
    /// Dropped event metrics partitioned by drop reason.
    pub dropped: MeasurementMetricSet<EtwDroppedMetrics>,
    /// Decode quality metrics (TDH decode failures).
    pub decode: MetricSet<EtwDecodeMetrics>,
    /// Kernel event loss metrics.
    pub kernel: MetricSet<EtwKernelMetrics>,
    /// Kernel buffer metrics partitioned by outcome.
    pub buffers: MeasurementMetricSet<EtwBufferMetrics>,
}

impl EtwReceiverMetrics {
    /// Registers all metric sets with the pipeline context.
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            events: EtwEventMetrics::register(pipeline_ctx),
            dropped: EtwDroppedMetrics::register(pipeline_ctx),
            decode: EtwDecodeMetrics::register(pipeline_ctx),
            kernel: EtwKernelMetrics::register(pipeline_ctx),
            buffers: EtwBufferMetrics::register(pipeline_ctx),
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = Vec::new();
        if self.events.needs_flush() {
            snapshots.extend(self.events.terminal_snapshots());
        }
        snapshots.extend(self.dropped.terminal_snapshots());
        if self.decode.needs_flush() {
            snapshots.extend(self.decode.terminal_snapshots());
        }
        if self.kernel.needs_flush() {
            snapshots.extend(self.kernel.terminal_snapshots());
        }
        snapshots.extend(self.buffers.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report(&mut self.events)?;
        reporter.report_measurement(&mut self.dropped)?;
        reporter.report(&mut self.decode)?;
        reporter.report(&mut self.kernel)?;
        reporter.report_measurement(&mut self.buffers)?;
        Ok(())
    }

    /// Constructs an `EtwReceiverMetrics` instance for testing using a mock pipeline context.
    #[cfg(test)]
    #[must_use]
    pub fn new_test() -> Self {
        let (pipeline_ctx, _) = otel_arrow_dfe_engine::testing::test_pipeline_ctx();
        Self::register(&pipeline_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx;
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;

    fn new_test_metrics() -> EtwReceiverMetrics {
        let (pipeline_ctx, _) = test_pipeline_ctx();
        EtwReceiverMetrics::register(&pipeline_ctx)
    }

    /// Scenario: Drop reason enum variants and attributes are constructed.
    /// Guarantees: Enum equality and attributes capture the right reason.
    #[test]
    fn test_drop_attributes() {
        assert_eq!(DropReason::SlowWorker, DropReason::SlowWorker);
        assert_eq!(DropReason::ForwardFailed, DropReason::ForwardFailed);
        assert_eq!(DropReason::MemoryPressure, DropReason::MemoryPressure);
        assert_ne!(DropReason::SlowWorker, DropReason::ForwardFailed);

        let attr = DropAttributes {
            reason: DropReason::SlowWorker,
        };
        assert_eq!(attr.reason, DropReason::SlowWorker);
    }

    /// Scenario: Buffer outcome enum variants and attributes are constructed.
    /// Guarantees: Enum equality and attributes capture the right outcome.
    #[test]
    fn test_buffer_attributes() {
        assert_eq!(BufferOutcome::Written, BufferOutcome::Written);
        assert_eq!(BufferOutcome::RealTimeLost, BufferOutcome::RealTimeLost);
        assert_eq!(BufferOutcome::LogLost, BufferOutcome::LogLost);
        assert_ne!(BufferOutcome::Written, BufferOutcome::RealTimeLost);

        let attr = BufferAttributes {
            outcome: BufferOutcome::Written,
        };
        assert_eq!(attr.outcome, BufferOutcome::Written);
    }

    /// Scenario: Delivery, decode, kernel, drop, and buffer metrics are recorded.
    /// Guarantees: Each metric set correctly isolates its values without overlap.
    #[test]
    fn test_etw_metrics_separation() {
        let mut metrics = new_test_metrics();

        metrics.events.received.add(10);
        metrics.events.forwarded.add(7);
        metrics.decode.errors.add(2);
        metrics.kernel.events_lost.add(1);

        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .dropped
            .add(3);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::ForwardFailed,
            })
            .dropped
            .add(4);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .dropped
            .add(5);

        metrics
            .buffers
            .with(BufferAttributes {
                outcome: BufferOutcome::Written,
            })
            .buffers
            .add(12);
        metrics
            .buffers
            .with(BufferAttributes {
                outcome: BufferOutcome::RealTimeLost,
            })
            .buffers
            .add(6);
        metrics
            .buffers
            .with(BufferAttributes {
                outcome: BufferOutcome::LogLost,
            })
            .buffers
            .add(8);

        assert_eq!(metrics.events.received.get(), 10);
        assert_eq!(metrics.events.forwarded.get(), 7);
        assert_eq!(metrics.decode.errors.get(), 2);
        assert_eq!(metrics.kernel.events_lost.get(), 1);

        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::SlowWorker,
                })
                .dropped
                .get(),
            3
        );
        assert_eq!(
            metrics
                .dropped
                .get(DropAttributes {
                    reason: DropReason::ForwardFailed,
                })
                .dropped
                .get(),
            4
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
                .buffers
                .get(BufferAttributes {
                    outcome: BufferOutcome::Written,
                })
                .buffers
                .get(),
            12
        );
        assert_eq!(
            metrics
                .buffers
                .get(BufferAttributes {
                    outcome: BufferOutcome::RealTimeLost,
                })
                .buffers
                .get(),
            6
        );
        assert_eq!(
            metrics
                .buffers
                .get(BufferAttributes {
                    outcome: BufferOutcome::LogLost,
                })
                .buffers
                .get(),
            8
        );
    }

    /// Scenario: Terminal snapshots are collected from ETW receiver metrics.
    /// Guarantees: Snapshots capture descriptors and clear measurement buckets on handoff.
    #[test]
    fn test_etw_terminal_snapshots_capture_and_clear() {
        let mut metrics = new_test_metrics();
        metrics.events.received.add(5);
        metrics.decode.errors.add(1);
        metrics.kernel.events_lost.add(2);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .dropped
            .add(2);
        metrics
            .buffers
            .with(BufferAttributes {
                outcome: BufferOutcome::Written,
            })
            .buffers
            .add(9);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 5);

        assert!(
            snapshots
                .iter()
                .any(|s| s.descriptor().name == "receiver.etw.events")
        );
        assert!(
            snapshots
                .iter()
                .any(|s| s.descriptor().name == "receiver.etw.decode")
        );
        assert!(
            snapshots
                .iter()
                .any(|s| s.descriptor().name == "receiver.etw.kernel")
        );
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.etw.dropped"
                && s.measurement_attribute_value("reason") == Some("slow_worker")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.etw.buffers"
                && s.measurement_attribute_value("outcome") == Some("written")
        }));

        // Measurement buckets are cleared after first snapshot collection
        let second = metrics.terminal_snapshots();
        // Plain MetricSets always yield 1 snapshot (zeroed), while MeasurementMetricSets yield 0.
        // Needs_flush check ensures plain sets are not emitted if not modified.
        assert_eq!(second.len(), 0);
    }

    /// Scenario: ETW receiver metrics are reported to a reporter.
    /// Guarantees: All metric sets are successfully handed over to the reporter.
    #[test]
    fn test_etw_report_emits_metric_sets() {
        let mut metrics = new_test_metrics();
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        metrics.events.received.add(1);
        metrics.decode.errors.add(1);
        metrics.kernel.events_lost.add(1);
        metrics
            .dropped
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .dropped
            .add(1);
        metrics
            .buffers
            .with(BufferAttributes {
                outcome: BufferOutcome::Written,
            })
            .buffers
            .add(1);

        metrics.report(&mut reporter).unwrap();
        assert_eq!(receiver.try_iter().count(), 5);
    }
}
