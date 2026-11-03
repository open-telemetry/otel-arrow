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
//!     + receiver.etw.drops.events{reason="forward_failed"}        (events in batches that failed to build/send)
//!     + receiver.etw.drops.events{reason="slow_worker"}           (events dropped by internal backpressure)
//!     + receiver.etw.drops.events{reason="memory_pressure"}       (reserved; not currently recorded)
//!     + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
//! ```
//!
//! Operational notes:
//! * `<scope>.<instrument>` is used as shorthand for metric names.
//! * `receiver.etw.events.received`: Ingress denominator, counted in the ETW event callback before channel enqueue.
//! * `receiver.etw.events.forwarded`: Events handed off to the next pipeline stage in batches.
//! * `receiver.etw.decode.errors`: TDH decoding failures. Events that fail decoding are still forwarded with empty fields where possible.
//! * `receiver.etw.session.events.lost`: Kernel buffer overrun before the receiver saw the events.
//! * `receiver.etw.session.buffers.written`: Kernel buffers written by the session.
//! * `receiver.etw.session.buffers.lost`: Kernel buffers lost, split by `delivery="real_time"` vs `delivery="log_file"`.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::SignalRegistrationAttributes;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{
    MeasurementMetricSet, MetricSet, MetricSetHandler, MetricSetSnapshot,
};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Event metrics for the ETW receiver tracking delivery accounting.
#[metric_set(
    name = "receiver.etw.events",
    registration_attributes = SignalRegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct EtwEventMetrics {
    /// Total events produced/received by the session before channel enqueue.
    /// This is the ingress denominator and authoritative total.
    #[metric(unit = "{event}")]
    pub(super) received: Counter<u64>,

    /// Events successfully forwarded downstream in batches.
    #[metric(unit = "{event}")]
    pub(super) forwarded: Counter<u64>,
}

/// Decode quality metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.decode",
    registration_attributes = SignalRegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct EtwDecodeMetrics {
    /// Number of ETW events that failed TDH decode/parsing.
    /// Note that these events remain eligible for forwarding downstream with empty decoded fields.
    #[metric(unit = "{event}")]
    pub(super) errors: Counter<u64>,
}

/// Reason an ETW event was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum DropReason {
    /// Dropped because the consuming worker could not drain the internal channel fast enough.
    SlowWorker,
    /// Dropped because building or forwarding the batch downstream failed.
    ForwardFailed,
    /// Reserved; not currently recorded as the receiver does not implement memory-pressure shedding.
    #[allow(dead_code)]
    MemoryPressure,
}

/// Attributes for ETW dropped event metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct DropAttributes {
    /// Reason the event was dropped.
    pub(super) reason: DropReason,
}

/// Dropped event metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.drops",
    registration_attributes = SignalRegistrationAttributes,
    measurement_attributes = DropAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct EtwDropMetrics {
    /// Number of dropped ETW events.
    #[metric(unit = "{event}")]
    pub(super) events: Counter<u64>,
}

/// Session-level statistics for the ETW receiver (Windows ETW session properties).
#[metric_set(
    name = "receiver.etw.session",
    registration_attributes = SignalRegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct EtwSessionMetrics {
    /// Number of events lost inside kernel ETW buffers before delivery (buffer overrun).
    /// Distinct from receiver-level drops and not included in received total.
    #[metric(name = "events.lost", unit = "{event}")]
    pub(super) events_lost: Counter<u64>,

    /// Number of kernel ETW buffers written by the session.
    #[metric(name = "buffers.written", unit = "{buffer}")]
    pub(super) buffers_written: Counter<u64>,
}

/// Delivery mode where kernel buffer loss occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum BufferLossDelivery {
    /// Real-time consumer delivery loss.
    RealTime,
    /// Trace log file buffer loss.
    LogFile,
}

/// Attributes for session buffer loss metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub(super) struct BufferLossAttributes {
    /// Delivery mode where buffer loss occurred.
    pub(super) delivery: BufferLossDelivery,
}

/// Session buffer loss metrics for the ETW receiver.
#[metric_set(
    name = "receiver.etw.session",
    registration_attributes = SignalRegistrationAttributes,
    measurement_attributes = BufferLossAttributes
)]
#[derive(Debug, Default, Clone)]
pub(super) struct EtwSessionBufferLossMetrics {
    /// Number of kernel ETW buffers lost.
    #[metric(name = "buffers.lost", unit = "{buffer}")]
    pub(super) buffers_lost: Counter<u64>,
}

/// ETW receiver metrics collection.
pub(super) struct EtwReceiverMetrics {
    /// Delivery accounting event metrics (received, forwarded).
    pub(super) events: MetricSet<EtwEventMetrics>,
    /// Dropped event metrics partitioned by drop reason.
    pub(super) drops: MeasurementMetricSet<EtwDropMetrics>,
    /// Decode quality metrics (TDH decode failures).
    pub(super) decode: MetricSet<EtwDecodeMetrics>,
    /// Session statistics (events.lost, buffers.written).
    pub(super) session: MetricSet<EtwSessionMetrics>,
    /// Session buffer loss metrics partitioned by delivery mode.
    pub(super) session_buffer_losses: MeasurementMetricSet<EtwSessionBufferLossMetrics>,
}

impl EtwReceiverMetrics {
    /// Registers all metric sets with the pipeline context.
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        let signal_attrs = SignalRegistrationAttributes {
            signal: SignalType::Logs,
        };
        Self {
            events: EtwEventMetrics::register(pipeline_ctx, &signal_attrs),
            drops: EtwDropMetrics::register(pipeline_ctx, &signal_attrs),
            decode: EtwDecodeMetrics::register(pipeline_ctx, &signal_attrs),
            session: EtwSessionMetrics::register(pipeline_ctx, &signal_attrs),
            session_buffer_losses: EtwSessionBufferLossMetrics::register(
                pipeline_ctx,
                &signal_attrs,
            ),
        }
    }

    /// Snapshots all metric sets and returns their descriptors.
    pub(super) fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = Vec::new();
        if self.events.needs_flush() {
            snapshots.extend(self.events.terminal_snapshots());
        }
        snapshots.extend(self.drops.terminal_snapshots());
        if self.decode.needs_flush() {
            snapshots.extend(self.decode.terminal_snapshots());
        }
        if self.session.needs_flush() {
            snapshots.extend(self.session.terminal_snapshots());
        }
        snapshots.extend(self.session_buffer_losses.terminal_snapshots());
        snapshots
    }

    /// Reports touched metric buckets to the given reporter.
    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report(&mut self.events)?;
        reporter.report_measurement(&mut self.drops)?;
        reporter.report(&mut self.decode)?;
        reporter.report(&mut self.session)?;
        reporter.report_measurement(&mut self.session_buffer_losses)?;
        Ok(())
    }

    /// Constructs an `EtwReceiverMetrics` instance for testing using a mock pipeline context.
    #[cfg(test)]
    #[must_use]
    pub(super) fn new_test() -> Self {
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

    /// Scenario: Delivery, decode, drop, and session metrics are recorded.
    /// Guarantees: Each metric set correctly isolates its values without overlap.
    #[test]
    fn test_etw_metrics_separation() {
        let mut metrics = new_test_metrics();

        metrics.events.received.add(10);
        metrics.events.forwarded.add(7);
        metrics.decode.errors.add(2);

        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .events
            .add(3);
        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::ForwardFailed,
            })
            .events
            .add(4);
        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .events
            .add(5);

        metrics.session.events_lost.add(1);
        metrics.session.buffers_written.add(12);

        metrics
            .session_buffer_losses
            .with(BufferLossAttributes {
                delivery: BufferLossDelivery::RealTime,
            })
            .buffers_lost
            .add(6);
        metrics
            .session_buffer_losses
            .with(BufferLossAttributes {
                delivery: BufferLossDelivery::LogFile,
            })
            .buffers_lost
            .add(8);

        assert_eq!(metrics.events.received.get(), 10);
        assert_eq!(metrics.events.forwarded.get(), 7);
        assert_eq!(metrics.decode.errors.get(), 2);

        assert_eq!(
            metrics
                .drops
                .get(DropAttributes {
                    reason: DropReason::SlowWorker,
                })
                .events
                .get(),
            3
        );
        assert_eq!(
            metrics
                .drops
                .get(DropAttributes {
                    reason: DropReason::ForwardFailed,
                })
                .events
                .get(),
            4
        );
        assert_eq!(
            metrics
                .drops
                .get(DropAttributes {
                    reason: DropReason::MemoryPressure,
                })
                .events
                .get(),
            5
        );

        assert_eq!(metrics.session.events_lost.get(), 1);
        assert_eq!(metrics.session.buffers_written.get(), 12);
        assert_eq!(
            metrics
                .session_buffer_losses
                .get(BufferLossAttributes {
                    delivery: BufferLossDelivery::RealTime,
                })
                .buffers_lost
                .get(),
            6
        );
        assert_eq!(
            metrics
                .session_buffer_losses
                .get(BufferLossAttributes {
                    delivery: BufferLossDelivery::LogFile,
                })
                .buffers_lost
                .get(),
            8
        );
    }

    /// Scenario: Terminal snapshots are collected from ETW receiver metrics.
    /// Guarantees: Table-driven assertions verify exact numeric values, descriptors, units,
    /// and attributes covering all drop reasons and buffer loss delivery modes.
    #[test]
    fn test_etw_terminal_snapshots_capture_and_clear() {
        let mut metrics = new_test_metrics();

        metrics.events.received.add(100);
        metrics.events.forwarded.add(90);
        metrics.decode.errors.add(5);

        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .events
            .add(11);
        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::ForwardFailed,
            })
            .events
            .add(12);
        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::MemoryPressure,
            })
            .events
            .add(13);

        metrics.session.events_lost.add(21);
        metrics.session.buffers_written.add(42);

        metrics
            .session_buffer_losses
            .with(BufferLossAttributes {
                delivery: BufferLossDelivery::RealTime,
            })
            .buffers_lost
            .add(31);
        metrics
            .session_buffer_losses
            .with(BufferLossAttributes {
                delivery: BufferLossDelivery::LogFile,
            })
            .buffers_lost
            .add(32);

        let snapshots = metrics.terminal_snapshots();

        struct Expected {
            scope: &'static str,
            instrument: &'static str,
            unit: &'static str,
            attr: Option<(&'static str, &'static str)>,
            value: u64,
        }

        let cases = [
            Expected {
                scope: "receiver.etw.events",
                instrument: "received",
                unit: "{event}",
                attr: None,
                value: 100,
            },
            Expected {
                scope: "receiver.etw.events",
                instrument: "forwarded",
                unit: "{event}",
                attr: None,
                value: 90,
            },
            Expected {
                scope: "receiver.etw.decode",
                instrument: "errors",
                unit: "{event}",
                attr: None,
                value: 5,
            },
            Expected {
                scope: "receiver.etw.drops",
                instrument: "events",
                unit: "{event}",
                attr: Some(("reason", "slow_worker")),
                value: 11,
            },
            Expected {
                scope: "receiver.etw.drops",
                instrument: "events",
                unit: "{event}",
                attr: Some(("reason", "forward_failed")),
                value: 12,
            },
            Expected {
                scope: "receiver.etw.drops",
                instrument: "events",
                unit: "{event}",
                attr: Some(("reason", "memory_pressure")),
                value: 13,
            },
            Expected {
                scope: "receiver.etw.session",
                instrument: "events.lost",
                unit: "{event}",
                attr: None,
                value: 21,
            },
            Expected {
                scope: "receiver.etw.session",
                instrument: "buffers.written",
                unit: "{buffer}",
                attr: None,
                value: 42,
            },
            Expected {
                scope: "receiver.etw.session",
                instrument: "buffers.lost",
                unit: "{buffer}",
                attr: Some(("delivery", "real_time")),
                value: 31,
            },
            Expected {
                scope: "receiver.etw.session",
                instrument: "buffers.lost",
                unit: "{buffer}",
                attr: Some(("delivery", "log_file")),
                value: 32,
            },
        ];

        for c in cases {
            let snap = snapshots
                .iter()
                .find(|s| {
                    s.descriptor().name == c.scope
                        && match c.attr {
                            Some((k, v)) => s.measurement_attribute_value(k) == Some(v),
                            None => true,
                        }
                        && s.descriptor()
                            .metrics
                            .iter()
                            .any(|m| m.name == c.instrument)
                })
                .unwrap_or_else(|| panic!("snapshot `{}` ({}) not found", c.scope, c.instrument));

            let idx = snap
                .descriptor()
                .metrics
                .iter()
                .position(|m| m.name == c.instrument)
                .unwrap();

            assert_eq!(snap.descriptor().metrics[idx].unit, c.unit);
            assert_eq!(snap.get_metrics()[idx].to_u64_lossy(), c.value);
        }

        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: A bounded telemetry channel fills up during reporting.
    /// Guarantees: Partial report handles backpressure without panic; subsequent collection
    /// or retry delivers each recorded value exactly once across collections.
    #[test]
    fn test_etw_partial_report_with_full_channel_retries_and_delivers_once() {
        let mut metrics = new_test_metrics();
        let (rx, mut reporter) = MetricsReporter::create_new_and_receiver(2);

        metrics.events.received.add(10);
        metrics
            .drops
            .with(DropAttributes {
                reason: DropReason::SlowWorker,
            })
            .events
            .add(3);
        metrics.decode.errors.add(2);
        metrics.session.events_lost.add(5);
        metrics
            .session_buffer_losses
            .with(BufferLossAttributes {
                delivery: BufferLossDelivery::RealTime,
            })
            .buffers_lost
            .add(4);

        // Channel capacity is 2; initial report succeeds and defers overflow items
        assert!(metrics.report(&mut reporter).is_ok());

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct KeyedObservation {
            scope: &'static str,
            instrument: &'static str,
            attr: Option<(&'static str, &'static str)>,
            value: u64,
        }

        let mut initial_snapshots = Vec::new();
        while let Ok(snap) = rx.try_recv() {
            initial_snapshots.push(snap);
        }
        assert_eq!(
            initial_snapshots.len(),
            2,
            "initial report should fill channel capacity"
        );

        let terminal_snapshots = metrics.terminal_snapshots();
        assert!(
            !terminal_snapshots.is_empty(),
            "remaining items should be deferred"
        );

        let mut delivered = Vec::new();
        for snap in initial_snapshots.into_iter().chain(terminal_snapshots) {
            let attr = snap.measurement_attributes().next();
            for (field, val) in snap.descriptor().metrics.iter().zip(snap.get_metrics()) {
                delivered.push(KeyedObservation {
                    scope: snap.descriptor().name,
                    instrument: field.name,
                    attr,
                    value: val.to_u64_lossy(),
                });
            }
        }
        delivered.sort();

        let mut expected = vec![
            KeyedObservation {
                scope: "receiver.etw.events",
                instrument: "received",
                attr: None,
                value: 10,
            },
            KeyedObservation {
                scope: "receiver.etw.events",
                instrument: "forwarded",
                attr: None,
                value: 0,
            },
            KeyedObservation {
                scope: "receiver.etw.drops",
                instrument: "events",
                attr: Some(("reason", "slow_worker")),
                value: 3,
            },
            KeyedObservation {
                scope: "receiver.etw.decode",
                instrument: "errors",
                attr: None,
                value: 2,
            },
            KeyedObservation {
                scope: "receiver.etw.session",
                instrument: "events.lost",
                attr: None,
                value: 5,
            },
            KeyedObservation {
                scope: "receiver.etw.session",
                instrument: "buffers.written",
                attr: None,
                value: 0,
            },
            KeyedObservation {
                scope: "receiver.etw.session",
                instrument: "buffers.lost",
                attr: Some(("delivery", "real_time")),
                value: 4,
            },
        ];
        expected.sort();

        assert_eq!(delivered, expected);
        assert!(metrics.terminal_snapshots().is_empty());
    }
}
