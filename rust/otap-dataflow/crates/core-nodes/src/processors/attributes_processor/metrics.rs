// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::common_attributes::{Outcome, OutcomeAttributes};
use otap_df_telemetry::error::Error;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MeasurementMetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{AttributeEnum, attribute_set, metric_set};

/// Actions performed on attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum ActionType {
    Renamed,
    Deleted,
    Inserted,
    Upserted,
    Updated,
    Hashed,
}

/// Target payload domain where transforms were applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum TargetDomain {
    Signal,
    Resource,
    Scope,
}

/// Combined action and domain dimensions for attribute modification metrics.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ActionAttributes {
    pub action: ActionType,
    pub domain: TargetDomain,
}

#[metric_set(
    name = "processor.attributes.modified",
    measurement_attributes = ActionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorModifiedMetrics {
    #[metric(unit = "{attr}")]
    pub entries: Counter<u64>,
}

/// Transform outcome metric using the common Outcome attribute.
#[metric_set(
    name = "processor.attributes",
    measurement_attributes = OutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorTransformMetrics {
    #[metric(unit = "{transform}")]
    pub transforms: Counter<u64>,
}

pub struct AttributesProcessorMetrics {
    pub transform_metrics: MeasurementMetricSet<AttributesProcessorTransformMetrics>,
    pub modified_metrics: MeasurementMetricSet<AttributesProcessorModifiedMetrics>,
}

impl AttributesProcessorMetrics {
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            transform_metrics: AttributesProcessorTransformMetrics::register(pipeline_ctx),
            modified_metrics: AttributesProcessorModifiedMetrics::register(pipeline_ctx),
        }
    }

    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report_measurement(&mut self.transform_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.modified_metrics))
    }

    pub fn record_transform_outcome(&mut self, outcome: Outcome) {
        self.transform_metrics
            .with(OutcomeAttributes { outcome })
            .transforms
            .inc();
    }

    pub fn modified_for(
        &mut self,
        action: ActionType,
        domain: TargetDomain,
    ) -> &mut AttributesProcessorModifiedMetrics {
        self.modified_metrics
            .with(ActionAttributes { action, domain })
    }
}
