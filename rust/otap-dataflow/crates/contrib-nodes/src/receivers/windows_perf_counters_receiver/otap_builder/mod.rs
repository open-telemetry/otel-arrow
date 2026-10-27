// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projects concrete counter observations into OTAP metric records.

use super::config::{CounterConfig, MetricKind};
use super::model::{InstanceIdentity, Number, Sample, SamplePoint, SampleValue};
use arrow::error::ArrowError;
use otel_arrow_dfe_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::record::metrics::{
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const AGGREGATION_TEMPORALITY_CUMULATIVE: i32 = 2;

fn instance_attribute_value(instance: &InstanceIdentity) -> String {
    if instance.index == 0 {
        instance.name.clone()
    } else {
        format!("{}#{}", instance.name, instance.index)
    }
}

struct ProjectionBuilder {
    metrics: MetricsRecordBatchBuilder,
    points: NumberDataPointsRecordBatchBuilder,
    attributes: StrKeysAttributesRecordBatchBuilder<u32>,
    metric_ids: BTreeMap<Arc<str>, u16>,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub(super) struct ProjectionInvariantError(ArrowError);

struct PreparedPoint<'a> {
    counter: &'a CounterConfig,
    point: &'a SamplePoint,
    value: Number,
    start_time_unix_nano: Option<i64>,
}

impl ProjectionBuilder {
    fn new() -> Self {
        Self {
            metrics: MetricsRecordBatchBuilder::new(),
            points: NumberDataPointsRecordBatchBuilder::new(),
            attributes: StrKeysAttributesRecordBatchBuilder::<u32>::new(),
            metric_ids: BTreeMap::new(),
        }
    }

    fn append_point(
        &mut self,
        point_id: u32,
        prepared: PreparedPoint<'_>,
    ) -> Result<(), ProjectionInvariantError> {
        let metric_id = self.metric_id(prepared.counter)?;
        self.points.append_id(point_id);
        self.points.append_parent_id(metric_id);
        self.points
            .append_start_time_unix_nano(prepared.start_time_unix_nano);
        self.points
            .append_time_unix_nano(prepared.point.timestamp_unix_nano);
        append_number(&mut self.points, prepared.value);
        self.points.append_flags(0);
        append_attributes(
            &mut self.attributes,
            point_id,
            prepared.counter,
            prepared.point,
        );
        Ok(())
    }

    fn metric_id(&mut self, counter: &CounterConfig) -> Result<u16, ProjectionInvariantError> {
        if let Some(metric_id) = self.metric_ids.get(counter.name.as_ref()) {
            return Ok(*metric_id);
        }
        let metric_id = u16::try_from(self.metric_ids.len()).map_err(|_| {
            ProjectionInvariantError(ArrowError::InvalidArgumentError(
                "too many configured metrics".to_owned(),
            ))
        })?;
        self.metrics.append_id(metric_id);
        self.metrics.append_metric_type(match counter.metric_kind {
            MetricKind::Gauge => MetricType::Gauge as u8,
            MetricKind::Sum => MetricType::Sum as u8,
        });
        self.metrics.append_name(counter.name.as_bytes());
        self.metrics
            .append_description(counter.description.as_bytes());
        self.metrics.append_unit(counter.unit.as_bytes());
        match counter.metric_kind {
            MetricKind::Gauge => {
                self.metrics.append_aggregation_temporality(None);
                self.metrics.append_is_monotonic(None);
            }
            MetricKind::Sum => {
                self.metrics
                    .append_aggregation_temporality(Some(AGGREGATION_TEMPORALITY_CUMULATIVE));
                self.metrics.append_is_monotonic(Some(false));
            }
        }
        let _ = self.metric_ids.insert(counter.name.clone(), metric_id);
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

fn append_number(points: &mut NumberDataPointsRecordBatchBuilder, value: Number) {
    match value {
        Number::Integer(value) => {
            points.append_int_value(Some(value));
            points.append_double_value(None);
        }
        Number::Double(value) => {
            points.append_int_value(None);
            points.append_double_value(Some(value));
        }
    }
}

fn append_attributes(
    attributes: &mut StrKeysAttributesRecordBatchBuilder<u32>,
    point_id: u32,
    counter: &CounterConfig,
    point: &SamplePoint,
) {
    let mut append = |key: &str, value: &str| {
        attributes.append_parent_id(&point_id);
        attributes.append_key(key);
        attributes.any_values_builder.append_str(value.as_bytes());
    };
    for (key, value) in counter.attributes.iter() {
        append(key, value);
    }
    append("windows.perf_counter.path", &point.path);
    if point.path.as_ref() != counter.path {
        append("windows.perf_counter.path_template", &counter.path);
    }
    if let Some(instance) = &point.instance {
        if point.emit_instance_attribute && !counter.attributes.contains_key("instance") {
            append("instance", &instance_attribute_value(instance));
        }
        append("windows.perf_counter.instance", &instance.name);
        if let Some(parent) = &instance.parent {
            append("windows.perf_counter.parent_instance", parent);
        }
        append(
            "windows.perf_counter.instance_index",
            &instance.index.to_string(),
        );
    }
}

fn prepare_point<'a>(
    counter: &'a CounterConfig,
    point: &'a SamplePoint,
    value: Number,
) -> Result<PreparedPoint<'a>, ProjectionInvariantError> {
    if point.timestamp_unix_nano <= 0 {
        return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
            format!(
                "counter {} has non-positive timestamp {}",
                point.path, point.timestamp_unix_nano
            ),
        )));
    }
    if let Number::Double(number) = value
        && !number.is_finite()
    {
        return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
            format!(
                "counter {} has non-finite double value {number}",
                point.path
            ),
        )));
    }
    let start_time_unix_nano = match counter.metric_kind {
        MetricKind::Gauge => None,
        MetricKind::Sum => {
            let start = point.start_time_unix_nano.ok_or_else(|| {
                ProjectionInvariantError(ArrowError::InvalidArgumentError(format!(
                    "Sum {} ({}) has no start time",
                    counter.name, point.path
                )))
            })?;
            if start <= 0 || start > point.timestamp_unix_nano {
                return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
                    format!(
                        "Sum {} ({}) has invalid start time {} for timestamp {}",
                        counter.name, point.path, start, point.timestamp_unix_nano
                    ),
                )));
            }
            Some(start)
        }
    };
    Ok(PreparedPoint {
        counter,
        point,
        value,
        start_time_unix_nano,
    })
}

/// Project concrete observations using their own timestamps and required Sum starts.
/// Invalid envelopes, duplicate mapping/path identities, malformed observations and
/// missing/invalid Sum starts are batch-level receiver invariant errors.
pub(super) fn project_sample(
    counters: &[CounterConfig],
    sample: &Sample,
) -> Result<Option<OtapArrowRecords>, ProjectionInvariantError> {
    if sample.timestamp_unix_nano <= 0 {
        return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
            format!(
                "sample has non-positive timestamp {}",
                sample.timestamp_unix_nano
            ),
        )));
    }
    let mut builder = ProjectionBuilder::new();
    let mut membership = BTreeSet::new();
    for member in &sample.members {
        if member.counter_index >= counters.len() {
            return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
                format!(
                    "sample membership references missing configured counter {}",
                    member.counter_index
                ),
            )));
        }
        let _ = membership.insert((member.counter_index, Arc::clone(&member.path)));
    }
    let mut point_count = 0usize;
    let mut seen_counters = BTreeSet::new();
    for point in &sample.points {
        let counter = counters.get(point.counter_index).ok_or_else(|| {
            ProjectionInvariantError(ArrowError::InvalidArgumentError(format!(
                "sample references missing configured counter {}",
                point.counter_index
            )))
        })?;
        if !seen_counters.insert((point.counter_index, point.path.as_ref())) {
            return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
                format!(
                    "sample repeats counter {} (metric {}) at index {}",
                    point.path, counter.name, point.counter_index
                ),
            )));
        }
        if !membership.is_empty()
            && !membership.contains(&(point.counter_index, Arc::clone(&point.path)))
        {
            return Err(ProjectionInvariantError(ArrowError::InvalidArgumentError(
                format!(
                    "sample point {} ({}) is absent from admitted membership",
                    point.counter_index, point.path
                ),
            )));
        }
        if let SampleValue::Value(value) = point.value {
            let prepared = prepare_point(counter, point, value)?;
            let point_id = u32::try_from(point_count).map_err(|_| {
                ProjectionInvariantError(ArrowError::InvalidArgumentError(
                    "too many counter values".to_owned(),
                ))
            })?;
            builder.append_point(point_id, prepared)?;
            point_count += 1;
        }
    }
    if builder.metric_ids.is_empty() {
        return Ok(None);
    }
    // Projection supplies valid UTF-8 and keeps every builder column aligned, so a finish error
    // indicates a violated internal invariant rather than a data-dependent scrape failure.
    let records = builder.finish().map_err(ProjectionInvariantError)?;
    Ok(Some(records))
}

#[cfg(test)]
mod tests;
