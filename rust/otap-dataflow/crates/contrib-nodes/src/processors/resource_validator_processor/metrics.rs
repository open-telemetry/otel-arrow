// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Resource Validator Processor

use super::ValidationFailure;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{attribute_set, metric_set};

/// Bounded failure classification for resource validation.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ValidationFailureAttributes {
    /// Reason that resource validation failed.
    #[attribute_key = "error.type"]
    pub error_type: ValidationFailure,
}

/// Resource validation failures grouped by reason.
#[metric_set(
    name = "processor.resource_validator",
    measurement_attributes = ValidationFailureAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ResourceValidatorFailureMetrics {
    /// Number of batches that failed resource validation.
    #[metric(unit = "{batch}")]
    pub failures: Counter<u64>,
}

/// Metrics collected by the Resource Validator Processor.
pub struct ResourceValidatorMetrics {
    validation_failures: MeasurementMetricSet<ResourceValidatorFailureMetrics>,
}

impl ResourceValidatorMetrics {
    /// Registers resource validator metrics.
    #[must_use]
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            validation_failures: ResourceValidatorFailureMetrics::register(pipeline_ctx),
        }
    }

    /// Records one failed batch for the bounded validation failure reason.
    pub fn record_failure(&mut self, failure: ValidationFailure) {
        self.validation_failures
            .with(ValidationFailureAttributes {
                error_type: failure,
            })
            .failures
            .inc();
    }

    /// Reports resource validator metrics.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter.report_measurement(&mut self.validation_failures)
    }
}
