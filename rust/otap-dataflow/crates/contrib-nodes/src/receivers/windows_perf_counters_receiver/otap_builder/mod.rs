// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projects one scrape's values into OTAP metric records.

use super::config::{CounterConfig, MetricKind, Temporality};
use super::pdh::Point;
use arrow::error::ArrowError;
use otel_arrow_dfe_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::record::metrics::{
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::collections::BTreeMap;
use std::sync::Arc;

const AGGREGATION_TEMPORALITY_DELTA: i32 = 1;
const AGGREGATION_TEMPORALITY_CUMULATIVE: i32 = 2;
/// Go's instance label; a static attribute with the same key replaces it.
pub(super) const INSTANCE_ATTRIBUTE: &str = "instance";

struct ProjectionBuilder {
    metrics: MetricsRecordBatchBuilder,
    points: NumberDataPointsRecordBatchBuilder,
    attributes: StrKeysAttributesRecordBatchBuilder<u32>,
    metric_ids: BTreeMap<Arc<str>, u16>,
}

impl ProjectionBuilder {
    fn metric_id(&mut self, counter: &CounterConfig) -> Result<u16, ArrowError> {
        if let Some(metric_id) = self.metric_ids.get(counter.name.as_ref()) {
            return Ok(*metric_id);
        }
        let metric_id = u16::try_from(self.metric_ids.len()).map_err(|_| {
            ArrowError::InvalidArgumentError("too many configured metrics".to_owned())
        })?;
        self.metrics.append_id(metric_id);
        self.metrics.append_name(counter.name.as_bytes());
        self.metrics
            .append_description(counter.description.as_bytes());
        self.metrics.append_unit(counter.unit.as_bytes());
        match counter.metric_kind {
            MetricKind::Gauge => {
                self.metrics.append_metric_type(MetricType::Gauge as u8);
                self.metrics.append_aggregation_temporality(None);
                self.metrics.append_is_monotonic(None);
            }
            MetricKind::Sum {
                temporality,
                monotonic,
            } => {
                self.metrics.append_metric_type(MetricType::Sum as u8);
                self.metrics
                    .append_aggregation_temporality(Some(match temporality {
                        Temporality::Cumulative => AGGREGATION_TEMPORALITY_CUMULATIVE,
                        Temporality::Delta => AGGREGATION_TEMPORALITY_DELTA,
                    }));
                self.metrics.append_is_monotonic(Some(monotonic));
            }
        }
        let _ = self.metric_ids.insert(Arc::clone(&counter.name), metric_id);
        Ok(metric_id)
    }

    fn finish(mut self) -> Result<OtapArrowRecords, ArrowError> {
        let metric_count = self.metric_ids.len();
        self.metrics.resource.append_id_n(0, metric_count);
        self.metrics
            .resource
            .append_schema_url_n(None, metric_count);
        self.metrics
            .resource
            .append_dropped_attributes_count_n(0, metric_count);
        self.metrics.scope.append_id_n(0, metric_count);
        self.metrics.scope.append_name_n(
            Some(b"otel-arrow-dfe-contrib-nodes/windows_perf_counters"),
            metric_count,
        );
        self.metrics
            .scope
            .append_version_n(Some(env!("CARGO_PKG_VERSION").as_bytes()), metric_count);
        self.metrics
            .scope
            .append_dropped_attributes_count_n(0, metric_count);
        self.metrics.append_scope_schema_url_n(b"", metric_count);

        let mut resource = StrKeysAttributesRecordBatchBuilder::<u16>::new();
        resource.append_parent_id(&0);
        resource.append_key("os.type");
        resource.any_values_builder.append_str(b"windows");

        let mut records = OtapArrowRecords::Metrics(Metrics::default());
        for (kind, batch) in [
            (ArrowPayloadType::UnivariateMetrics, self.metrics.finish()?),
            (ArrowPayloadType::NumberDataPoints, self.points.finish()?),
            (ArrowPayloadType::NumberDpAttrs, self.attributes.finish()?),
            (ArrowPayloadType::ResourceAttrs, resource.finish()?),
        ] {
            records
                .set(kind, batch)
                .map_err(|err| ArrowError::ExternalError(Box::new(err)))?;
        }
        Ok(records)
    }
}

/// Projects one scrape's points into an OTAP metrics batch, or `None` when there are none.
pub(super) fn project_points(
    counters: &[CounterConfig],
    points: &[Point],
) -> Result<Option<OtapArrowRecords>, ArrowError> {
    if points.is_empty() {
        return Ok(None);
    }
    let mut builder = ProjectionBuilder {
        metrics: MetricsRecordBatchBuilder::new(),
        points: NumberDataPointsRecordBatchBuilder::new(),
        attributes: StrKeysAttributesRecordBatchBuilder::<u32>::new(),
        metric_ids: BTreeMap::new(),
    };
    for (point_id, point) in points.iter().enumerate() {
        let point_id = u32::try_from(point_id)
            .map_err(|_| ArrowError::InvalidArgumentError("too many counter values".to_owned()))?;
        let counter = &counters[point.counter];
        let metric_id = builder.metric_id(counter)?;
        let points = &mut builder.points;
        points.append_id(point_id);
        points.append_parent_id(metric_id);
        // Only cumulative Sums carry a start time; Go sets none for delta.
        points.append_start_time_unix_nano(
            matches!(
                counter.metric_kind,
                MetricKind::Sum {
                    temporality: Temporality::Cumulative,
                    ..
                }
            )
            .then_some(point.start_time_unix_nano),
        );
        points.append_time_unix_nano(point.time_unix_nano);
        points.append_int_value(None);
        points.append_double_value(Some(point.value));
        points.append_flags(0);

        let attributes = &mut builder.attributes;
        let instance = point
            .instance
            .as_deref()
            .filter(|_| !counter.attributes.contains_key(INSTANCE_ATTRIBUTE))
            .map(|instance| (INSTANCE_ATTRIBUTE, instance));
        for (key, value) in instance.into_iter().chain(
            counter
                .attributes
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        ) {
            attributes.append_parent_id(&point_id);
            attributes.append_key(key);
            attributes.any_values_builder.append_str(value.as_bytes());
        }
    }
    builder.finish().map(Some)
}

#[cfg(test)]
mod tests;
