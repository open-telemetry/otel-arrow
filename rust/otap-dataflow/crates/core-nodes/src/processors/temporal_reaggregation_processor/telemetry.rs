// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry definitions for the temporal reaggregation processor.

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Emitted when creating a view fails so we cannot process the data
pub const VIEW_CREATION_FAILED_EVENT: &str = "temporal_reaggregation.view.creation_failed";

/// Emitted when encoding one or more attributes fails. This is mostly a concern for CBOR
/// encoded data.
pub const ATTRIBUTE_ENCODE_FAILED_EVENT: &str = "temporal_reaggregation.attribute.encode_failed";

/// Emitted when calldata returned to this processor is invalid in some way
pub const INVALID_CALLDATA_EVENT: &str = "temporal_reaggregation.calldata.invalid";

/// Emitted when there is an erroneous ack/nack event
pub const ERRONEOUS_ACK_EVENT: &str = "temporal_reaggregation.ack.erroneous";

/// Actionable cause for a failed input operation.
/// These map directly to the `error.type` attribute on the `failures` metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ErrorType {
    /// A view could not be created over the input records.
    ViewCreation,
    /// The batch was too large even after a flush; ID counter would overflow.
    IdOverflow,
    /// The batch was too large even after a flush; stream cardinality limit hit.
    StreamCardinalityExceeded,
    /// Failed to send output downstream.
    OutputSend,
    /// Internal scheduling or other fatal error.
    Internal,
}

/// What triggered a flush. Used as the `reason` attribute on the `flushes` metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum FlushReason {
    /// Regular collection-period timer fired.
    Timer,
    /// ID overflow caused an early flush before the timer.
    IdOverflow,
    /// Stream cardinality limit caused an early flush before the timer.
    StreamCardinalityExceeded,
    /// Processor is shutting down; flush remaining data.
    Shutdown,
}

/// Attributes carried on each `failures` data point.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct FailureAttributes {
    /// The actionable cause of the failure.
    #[attribute_key = "error.type"]
    pub error_type: ErrorType,
}

/// Attributes carried on each `flushes` data point.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct FlushAttributes {
    /// Whether the flush itself succeeded or failed.
    pub outcome: Outcome,
    /// What triggered this flush.
    pub reason: FlushReason,
}

/// Counts one terminal outcome per metrics PData input.
///
/// Incremented exactly once per input, regardless of how many overflow flushes
/// happen internally while processing it.
#[metric_set(
    name = "processor.temporal_reaggregation",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OperationMetrics {
    #[metric(unit = "{operation}")]
    pub operations: Counter<u64>,
}

/// Counts failed input operations broken down by actionable cause.
///
/// Always incremented alongside a corresponding `operations` failure so that
/// `sum(failures)` == number of failed `operations`.
#[metric_set(
    name = "processor.temporal_reaggregation",
    measurement_attributes = FailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FailureMetrics {
    #[metric(unit = "{operation}")]
    pub failures: Counter<u64>,
}

/// Counts non-empty flush attempts.
///
/// Empty flushes (no data accumulated) are not recorded here.
#[metric_set(
    name = "processor.temporal_reaggregation",
    measurement_attributes = FlushAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct FlushMetrics {
    #[metric(unit = "{flush}")]
    pub flushes: Counter<u64>,
}

/// All metrics for the temporal reaggregation processor.
pub struct TemporalReaggregationMetrics {
    operations: MeasurementMetricSet<OperationMetrics>,
    failures: MeasurementMetricSet<FailureMetrics>,
    flushes: MeasurementMetricSet<FlushMetrics>,
}

impl TemporalReaggregationMetrics {
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operations: OperationMetrics::register(pipeline_ctx),
            failures: FailureMetrics::register(pipeline_ctx),
            flushes: FlushMetrics::register(pipeline_ctx),
        }
    }

    /// Record one successful input operation.
    pub fn record_success(&mut self) {
        self.operations
            .with(OutcomeAttributes {
                outcome: Outcome::Success,
            })
            .operations
            .inc();
    }

    /// Record one failed input operation with the actionable cause.
    pub fn record_failure(&mut self, error_type: ErrorType) {
        self.operations
            .with(OutcomeAttributes {
                outcome: Outcome::Failure,
            })
            .operations
            .inc();
        self.failures
            .with(FailureAttributes { error_type })
            .failures
            .inc();
    }

    /// Record a non-empty flush attempt.
    ///
    /// Do not call this for empty flushes (when `builder.finish()` returns
    /// nothing) - those are intentionally silent.
    pub fn record_flush(&mut self, outcome: Outcome, reason: FlushReason) {
        self.flushes
            .with(FlushAttributes { outcome, reason })
            .flushes
            .inc();
    }

    pub fn report(
        &mut self,
        reporter: &mut MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        reporter.report_measurement(&mut self.operations)?;
        reporter.report_measurement(&mut self.failures)?;
        reporter.report_measurement(&mut self.flushes)
    }
}
