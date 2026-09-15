// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Resource Validator Processor

use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otel_arrow_dfe_telemetry::error::Error;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Reason for a batch rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum RejectReason {
    Missing,
    NotAllowed,
    InvalidType,
    ConversionError,
}

/// Batch attributes (outcome and reason)
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ValidatorBatchAttributes {
    pub outcome: Outcome,
    #[attribute_key = "reason"]
    pub reason: Option<RejectReason>,
}

#[metric_set(
    name = "processor.resource_validator.batches",
    measurement_attributes = ValidatorBatchAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ValidatorBatchMetrics {
    #[metric(unit = "{batch}")]
    pub batches: Counter<u64>,
}

#[metric_set(
    name = "processor.resource_validator.items",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ValidatorItemMetrics {
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

pub struct ResourceValidatorMetrics {
    pub batch_metrics: MeasurementMetricSet<ValidatorBatchMetrics>,
    pub item_metrics: MeasurementMetricSet<ValidatorItemMetrics>,
}

impl ResourceValidatorMetrics {
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            batch_metrics: ValidatorBatchMetrics::register(pipeline_ctx),
            item_metrics: ValidatorItemMetrics::register(pipeline_ctx),
        }
    }

    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report_measurement(&mut self.batch_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.item_metrics))
    }

    pub fn record_batch(&mut self, outcome: Outcome, reason: Option<RejectReason>) {
        self.batch_metrics
            .with(ValidatorBatchAttributes { outcome, reason })
            .batches
            .inc();
    }

    pub fn record_items(&mut self, outcome: Outcome, count: u64) {
        self.item_metrics
            .with(OutcomeAttributes { outcome })
            .items
            .add(count);
    }
}
