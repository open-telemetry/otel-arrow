// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projects exact-counter samples into OTAP metric records.

use super::config::{CounterConfig, MetricKind};
use super::model::{Number, Sample, SampleValue};
use arrow::error::ArrowError;
use otel_arrow_dfe_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otel_arrow_dfe_pdata::encode::record::metrics::{
    MetricsRecordBatchBuilder, NumberDataPointsRecordBatchBuilder,
};
use otel_arrow_dfe_pdata::otap::{Metrics, OtapArrowRecords};
use otel_arrow_dfe_pdata::otlp::metrics::MetricType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::collections::BTreeMap;

const AGGREGATION_TEMPORALITY_CUMULATIVE: i32 = 2;

/// Projects already-scaled counter samples into OTAP metrics.
/// Each input point must reference a distinct configured counter.
/// Only numeric values are emitted; returns `Ok(None)` when there are none.
/// Contract violations reject the whole sample and indicate a collection bug, not bad counter data.
pub(super) fn into_otap(
    counters: &[CounterConfig],
    sample: Sample,
) -> Result<Option<OtapArrowRecords>, ArrowError> {
    if sample.timestamp_unix_nano <= 0 {
        return Err(ArrowError::InvalidArgumentError(format!(
            "sample has non-positive timestamp {}",
            sample.timestamp_unix_nano
        )));
    }
    let mut metrics = MetricsRecordBatchBuilder::new();
    let mut points = NumberDataPointsRecordBatchBuilder::new();
    let mut attrs = StrKeysAttributesRecordBatchBuilder::<u32>::new();
    let mut metric_ids = BTreeMap::new();
    let mut seen_counters = vec![false; counters.len()];
    let mut point_count = 0_u32;

    for point in sample.points {
        let counter = counters.get(point.counter_index).ok_or_else(|| {
            ArrowError::InvalidArgumentError(format!(
                "sample references missing configured counter {}",
                point.counter_index
            ))
        })?;
        if seen_counters[point.counter_index] {
            return Err(ArrowError::InvalidArgumentError(format!(
                "sample repeats counter {} (metric {}) at index {}",
                counter.path, counter.name, point.counter_index
            )));
        }
        seen_counters[point.counter_index] = true;
        let SampleValue::Value(value) = point.value else {
            continue;
        };
        if counter.kind == MetricKind::Sum
            && (sample.start_time_unix_nano <= 0
                || sample.start_time_unix_nano > sample.timestamp_unix_nano)
        {
            return Err(ArrowError::InvalidArgumentError(format!(
                "Sum {} ({}) has invalid start time {} for timestamp {}",
                counter.name, counter.path, sample.start_time_unix_nano, sample.timestamp_unix_nano
            )));
        }
        let metric_id = if let Some(metric_id) = metric_ids.get(&counter.name) {
            *metric_id
        } else {
            let metric_id = u16::try_from(metric_ids.len()).map_err(|_| {
                ArrowError::InvalidArgumentError("too many configured metrics".to_owned())
            })?;
            metrics.append_id(metric_id);
            match counter.kind {
                MetricKind::Gauge => {
                    metrics.append_metric_type(MetricType::Gauge as u8);
                    metrics.append_aggregation_temporality(None);
                    metrics.append_is_monotonic(None);
                }
                MetricKind::Sum => {
                    metrics.append_metric_type(MetricType::Sum as u8);
                    metrics
                        .append_aggregation_temporality(Some(AGGREGATION_TEMPORALITY_CUMULATIVE));
                    metrics.append_is_monotonic(Some(false));
                }
            }
            metrics.append_name(counter.name.as_bytes());
            metrics.append_description(counter.description.as_bytes());
            metrics.append_unit(counter.unit.as_bytes());
            let _ = metric_ids.insert(counter.name.clone(), metric_id);
            metric_id
        };

        points.append_id(point_count);
        points.append_parent_id(metric_id);
        points.append_start_time_unix_nano(match counter.kind {
            MetricKind::Gauge => None,
            MetricKind::Sum => Some(sample.start_time_unix_nano),
        });
        points.append_time_unix_nano(sample.timestamp_unix_nano);
        match value {
            Number::Integer(value) => {
                points.append_int_value(Some(value));
                points.append_double_value(None);
            }
            Number::Double(value) if value.is_finite() => {
                points.append_int_value(None);
                points.append_double_value(Some(value));
            }
            Number::Double(value) => {
                return Err(ArrowError::InvalidArgumentError(format!(
                    "counter {} has non-finite double value {value}",
                    counter.path
                )));
            }
        }
        points.append_flags(0);

        for (key, value) in counter.attributes.iter() {
            attrs.append_parent_id(&point_count);
            attrs.append_key(key);
            attrs.any_values_builder.append_str(value.as_bytes());
        }
        attrs.append_parent_id(&point_count);
        attrs.append_key("windows.perf_counter.path");
        attrs.any_values_builder.append_str(counter.path.as_bytes());
        point_count = point_count.checked_add(1).ok_or_else(|| {
            ArrowError::InvalidArgumentError("too many counter values".to_owned())
        })?;
    }
    if point_count == 0 {
        return Ok(None);
    }

    let metric_count = metric_ids.len();
    metrics.resource.append_id_n(0, metric_count);
    metrics.resource.append_schema_url_n(None, metric_count);
    metrics
        .resource
        .append_dropped_attributes_count_n(0, metric_count);
    metrics.scope.append_id_n(0, metric_count);
    metrics.scope.append_name_n(
        Some(b"otel-arrow-dfe-contrib-nodes/windows_perf_counters"),
        metric_count,
    );
    metrics
        .scope
        .append_version_n(Some(env!("CARGO_PKG_VERSION").as_bytes()), metric_count);
    metrics
        .scope
        .append_dropped_attributes_count_n(0, metric_count);
    metrics.append_scope_schema_url_n(b"", metric_count);

    let mut resource = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    resource.append_parent_id(&0);
    resource.append_key("os.type");
    resource.any_values_builder.append_str(b"windows");

    let mut records = OtapArrowRecords::Metrics(Metrics::default());
    for (kind, batch) in [
        (ArrowPayloadType::UnivariateMetrics, metrics.finish()?),
        (ArrowPayloadType::NumberDataPoints, points.finish()?),
        (ArrowPayloadType::NumberDpAttrs, attrs.finish()?),
        (ArrowPayloadType::ResourceAttrs, resource.finish()?),
    ] {
        records
            .set(kind, batch)
            .map_err(|err| ArrowError::ExternalError(Box::new(err)))?;
    }
    Ok(Some(records))
}

#[cfg(test)]
mod tests;
