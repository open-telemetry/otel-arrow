//! Bounded operation and record-level metrics for declarative parsing.

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::{
    common_attributes::{Outcome, SignalOutcomeAttributes},
    error::Error as TelemetryError,
    instrument::Counter,
    metrics::MeasurementMetricSet,
    reporter::MetricsReporter,
};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub(super) enum ParserErrorType {
    PayloadConversion,
    IdDecode,
    OutputSend,
    Internal,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct FailureAttributes {
    signal: SignalType,
    #[attribute_key = "error.type"]
    error_type: ParserErrorType,
}

#[metric_set(name = "processor.log_parser", measurement_attributes = SignalOutcomeAttributes)]
#[derive(Debug, Default, Clone)]
struct OperationMetrics {
    /// Matching input messages whose local parsing operation terminated.
    #[metric(unit = "{operation}")]
    operations: Counter<u64>,
}

#[metric_set(name = "processor.log_parser", measurement_attributes = FailureAttributes)]
#[derive(Debug, Default, Clone)]
struct FailureMetrics {
    /// Failed local parsing operations.
    #[metric(unit = "{operation}")]
    failures: Counter<u64>,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
struct ParsingAttributes {
    format: super::parse_logs::Format,
    reason: super::parse_logs::DataError,
}

#[metric_set(name = "processor.log_parser", measurement_attributes = ParsingAttributes)]
#[derive(Debug, Default, Clone)]
struct ParsingMetrics {
    /// Preserved malformed records and committed observed-time fallbacks.
    #[metric(unit = "{record}")]
    records: Counter<u64>,
}

pub(super) struct ParserMetrics {
    operations: MeasurementMetricSet<OperationMetrics>,
    failures: MeasurementMetricSet<FailureMetrics>,
    parsing: MeasurementMetricSet<ParsingMetrics>,
}

impl ParserMetrics {
    pub(super) fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operations: OperationMetrics::register(pipeline_ctx),
            failures: FailureMetrics::register(pipeline_ctx),
            parsing: ParsingMetrics::register(pipeline_ctx),
        }
    }

    pub(super) fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter
            .report_measurement(&mut self.operations)
            .and_then(|()| reporter.report_measurement(&mut self.failures))
            .and_then(|()| reporter.report_measurement(&mut self.parsing))
    }

    pub(super) fn record_parsing(
        &mut self,
        format: super::parse_logs::Format,
        counts: super::parse_logs::Counts,
    ) {
        for (reason, count) in counts.values() {
            if count != 0 {
                self.parsing
                    .with(ParsingAttributes { format, reason })
                    .records
                    .add(count);
            }
        }
    }

    pub(super) fn record_success(&mut self) {
        self.operations
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .operations
            .inc();
    }

    pub(super) fn record_failure(&mut self, error_type: ParserErrorType) {
        self.operations
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Failure,
            })
            .operations
            .inc();
        self.failures
            .with(FailureAttributes {
                signal: SignalType::Logs,
                error_type,
            })
            .failures
            .inc();
    }
}
