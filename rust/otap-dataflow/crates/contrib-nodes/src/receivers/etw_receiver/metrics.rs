// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Windows ETW receiver.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

// -- Event outcome attributes -------------------------------------------------

/// Outcome of an ETW event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum EventOutcome {
    /// Total events produced/received by the session before channel enqueue.
    Received,
    /// Events successfully forwarded downstream.
    Forwarded,
    /// Events that failed TDH decode/parsing (forwarded with empty decoded fields).
    Invalid,
    /// Events lost inside kernel ETW buffers before delivery (buffer overrun).
    Lost,
}

/// Attributes for ETW event metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct EventAttributes {
    /// Outcome of the event.
    pub outcome: EventOutcome,
}

/// Event metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.events",
    measurement_attributes = EventAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct EtwEventMetrics {
    /// Number of ETW events.
    #[metric(unit = "{event}")]
    pub events: Counter<u64>,
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
    /// Event metrics partitioned by outcome.
    pub events: MeasurementMetricSet<EtwEventMetrics>,
    /// Dropped event metrics partitioned by drop reason.
    pub dropped: MeasurementMetricSet<EtwDroppedMetrics>,
    /// Kernel buffer metrics partitioned by outcome.
    pub buffers: MeasurementMetricSet<EtwBufferMetrics>,
}

impl EtwReceiverMetrics {
    /// Registers all metric sets with the pipeline context.
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            events: EtwEventMetrics::register(pipeline_ctx),
            dropped: EtwDroppedMetrics::register(pipeline_ctx),
            buffers: EtwBufferMetrics::register(pipeline_ctx),
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.events.terminal_snapshots();
        snapshots.extend(self.dropped.terminal_snapshots());
        snapshots.extend(self.buffers.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.events)?;
        reporter.report_measurement(&mut self.dropped)?;
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

    /// Scenario: Event outcome enum variants and attributes are constructed.
    /// Guarantees: Enum equality and attributes capture the right outcome.
    #[test]
    fn test_event_attributes() {
        assert_eq!(EventOutcome::Received, EventOutcome::Received);
        assert_eq!(EventOutcome::Forwarded, EventOutcome::Forwarded);
        assert_eq!(EventOutcome::Invalid, EventOutcome::Invalid);
        assert_eq!(EventOutcome::Lost, EventOutcome::Lost);
        assert_ne!(EventOutcome::Received, EventOutcome::Forwarded);

        let attr = EventAttributes {
            outcome: EventOutcome::Received,
        };
        assert_eq!(attr.outcome, EventOutcome::Received);
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

    /// Scenario: ETW receiver metrics are recorded across events, dropped, and buffers metric sets.
    /// Guarantees: Metrics are properly partitioned by their enum attributes and values are retrievable.
    #[test]
    fn test_etw_metrics_partitioned_by_attributes() {
        let mut metrics = new_test_metrics();

        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Received,
            })
            .events
            .add(10);
        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Forwarded,
            })
            .events
            .add(7);
        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Invalid,
            })
            .events
            .add(2);
        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Lost,
            })
            .events
            .add(1);

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

        assert_eq!(
            metrics
                .events
                .get(EventAttributes {
                    outcome: EventOutcome::Received,
                })
                .events
                .get(),
            10
        );
        assert_eq!(
            metrics
                .events
                .get(EventAttributes {
                    outcome: EventOutcome::Forwarded,
                })
                .events
                .get(),
            7
        );
        assert_eq!(
            metrics
                .events
                .get(EventAttributes {
                    outcome: EventOutcome::Invalid,
                })
                .events
                .get(),
            2
        );
        assert_eq!(
            metrics
                .events
                .get(EventAttributes {
                    outcome: EventOutcome::Lost,
                })
                .events
                .get(),
            1
        );

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
    /// Guarantees: Snapshots capture the measurement attributes and clear measurement buckets on handoff.
    #[test]
    fn test_etw_terminal_snapshots_capture_and_clear() {
        let mut metrics = new_test_metrics();
        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Received,
            })
            .events
            .add(5);
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
        assert_eq!(snapshots.len(), 3);

        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "receiver.etw.events"
                && s.measurement_attribute_value("outcome") == Some("received")
        }));
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
        assert_eq!(second.len(), 0);
    }

    /// Scenario: ETW receiver metrics are reported to a reporter.
    /// Guarantees: All metric sets are successfully handed over to the reporter.
    #[test]
    fn test_etw_report_emits_metric_sets() {
        let mut metrics = new_test_metrics();
        let (receiver, mut reporter) = MetricsReporter::create_new_and_receiver(16);

        metrics
            .events
            .with(EventAttributes {
                outcome: EventOutcome::Received,
            })
            .events
            .add(1);
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
        assert_eq!(receiver.try_iter().count(), 3);
    }
}
