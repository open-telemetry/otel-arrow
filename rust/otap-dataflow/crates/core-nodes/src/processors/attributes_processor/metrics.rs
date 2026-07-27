// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::error::Error;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::{attribute_set, metric_set, AttributeEnum};

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

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ActionAttributes {
    pub action: ActionType,
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

/// Target payload domain where transforms were applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum TargetDomain {
    Signal,
    Resource,
    Scope,
}

#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct DomainAttributes {
    pub domain: TargetDomain,
}

#[metric_set(
    name = "processor.attributes.domains",
    measurement_attributes = DomainAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorDomainMetrics {
    #[metric(unit = "{apply}")]
    pub applied: Counter<u64>,
}

/// Metrics that do not have a bounded item dimension.
#[metric_set(name = "processor.attributes")]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorOperationalMetrics {
    #[metric(unit = "{op}")]
    pub transform_failed: Counter<u64>,
}

pub struct AttributesProcessorMetrics {
    pub operational_metrics: MetricSet<AttributesProcessorOperationalMetrics>,
    pub modified_metrics: MeasurementMetricSet<AttributesProcessorModifiedMetrics>,
    pub domain_metrics: MeasurementMetricSet<AttributesProcessorDomainMetrics>,
}

impl AttributesProcessorMetrics {
    pub fn new(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            operational_metrics: AttributesProcessorOperationalMetrics::register(pipeline_ctx),
            modified_metrics: AttributesProcessorModifiedMetrics::register(pipeline_ctx),
            domain_metrics: AttributesProcessorDomainMetrics::register(pipeline_ctx),
        }
    }

    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), Error> {
        reporter
            .report(&mut self.operational_metrics)
            .and_then(|()| reporter.report_measurement(&mut self.modified_metrics))
            .and_then(|()| reporter.report_measurement(&mut self.domain_metrics))
    }

    pub fn modified_for(&mut self, action: ActionType) -> &mut AttributesProcessorModifiedMetrics {
        self.modified_metrics.with(ActionAttributes { action })
    }

    pub fn domains_for(&mut self, domain: TargetDomain) -> &mut AttributesProcessorDomainMetrics {
        self.domain_metrics.with(DomainAttributes { domain })
    }
}
