// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projection of exact-counter observations directly into OTAP metric records.

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

/// Projects ready, already scaled observations using validated exact-counter configuration.
/// No-observation points are omitted; a collection without values produces no batch.
/// Errors reject the whole sample and indicate a broken collection contract, not bad counter data.
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
    let mut point_count = 0_u32;

    for point in sample.points {
        let SampleValue::Value(value) = point.value else {
            continue;
        };
        let counter = counters.get(point.counter_index).ok_or_else(|| {
            ArrowError::InvalidArgumentError(format!(
                "sample references missing configured counter {}",
                point.counter_index
            ))
        })?;
        if counter.kind == MetricKind::UpDownCounter
            && (sample.start_time_unix_nano <= 0
                || sample.start_time_unix_nano > sample.timestamp_unix_nano)
        {
            return Err(ArrowError::InvalidArgumentError(format!(
                "UpDownCounter {} ({}) has invalid start time {} for timestamp {}",
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
                MetricKind::UpDownCounter => {
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
            MetricKind::UpDownCounter => Some(sample.start_time_unix_nano),
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
mod tests {
    use super::super::config::RuntimeConfig;
    use super::super::model::{SamplePoint, scale_integer};
    use super::*;
    use arrow::array::{Array, TimestampNanosecondArray};
    use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{KeyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        AggregationTemporality, metric, number_data_point,
    };
    use otel_arrow_dfe_pdata::{OtapPayload, OtlpProtoBytes, TryIntoWithOptions};
    use prost::Message;
    use std::sync::Arc;

    fn counter(name: &str, kind: MetricKind) -> CounterConfig {
        CounterConfig {
            path: format!(r"\Object\{name}"),
            name: name.to_owned(),
            unit: "By".to_owned(),
            description: Arc::from(format!("Description for {name}.")),
            kind,
            attributes: Arc::new(BTreeMap::new()),
            scale_power10: 0,
        }
    }

    fn point(counter_index: usize, value: SampleValue) -> SamplePoint {
        SamplePoint {
            counter_index,
            value,
        }
    }

    fn sample(start: i64, timestamp: i64, points: Vec<SamplePoint>) -> Sample {
        Sample {
            start_time_unix_nano: start,
            timestamp_unix_nano: timestamp,
            points,
        }
    }

    fn decode(records: OtapArrowRecords) -> ExportMetricsServiceRequest {
        let bytes: OtlpProtoBytes = OtapPayload::from_otap(records)
            .try_into_with_default()
            .unwrap();
        ExportMetricsServiceRequest::decode(bytes.as_bytes()).unwrap()
    }

    fn attributes(attrs: &[KeyValue]) -> BTreeMap<&str, &str> {
        attrs
            .iter()
            .map(|attr| {
                let Some(any_value::Value::StringValue(value)) =
                    attr.value.as_ref().and_then(|value| value.value.as_ref())
                else {
                    panic!("expected a string attribute");
                };
                (attr.key.as_str(), value.as_str())
            })
            .collect()
    }

    /// Scenario: A collection contains integer and double Gauges and UpDownCounters.
    /// Guarantees: Decoded output retains metadata, exact values, timestamps and cumulative non-monotonic Sum shape.
    #[test]
    fn projects_metric_kinds_and_numeric_types() {
        let start = 1_788_400_000_000_000_000;
        let timestamp = start + 123_456_789;
        let counters = [
            counter("integer_gauge", MetricKind::Gauge),
            counter("double_gauge", MetricKind::Gauge),
            counter("integer_sum", MetricKind::UpDownCounter),
            counter("double_sum", MetricKind::UpDownCounter),
        ];
        let values = [
            Number::Integer(9_007_199_254_740_993),
            Number::Double(250.5),
            Number::Integer(-42),
            scale_integer(42, 3).unwrap(),
        ];
        let records = into_otap(
            &counters,
            sample(
                start,
                timestamp,
                values
                    .iter()
                    .enumerate()
                    .map(|(index, value)| point(index, SampleValue::Value(*value)))
                    .collect(),
            ),
        )
        .unwrap()
        .unwrap();
        let starts = records
            .get(ArrowPayloadType::NumberDataPoints)
            .unwrap()
            .column_by_name("start_time_unix_nano")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .unwrap();
        assert!(starts.is_null(0));
        assert!(starts.is_null(1));
        assert_eq!(starts.value(2), start);
        assert_eq!(starts.value(3), start);

        let decoded = decode(records);
        assert_eq!(decoded.resource_metrics.len(), 1);
        let resource = &decoded.resource_metrics[0];
        assert_eq!(
            attributes(&resource.resource.as_ref().unwrap().attributes),
            BTreeMap::from([("os.type", "windows")])
        );
        assert_eq!(resource.scope_metrics.len(), 1);
        let scope = &resource.scope_metrics[0];
        let metadata = scope.scope.as_ref().unwrap();
        assert_eq!(
            metadata.name,
            "otel-arrow-dfe-contrib-nodes/windows_perf_counters"
        );
        assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(scope.metrics.len(), counters.len());
        for ((metric, counter), value) in scope.metrics.iter().zip(&counters).zip(values) {
            assert_eq!(metric.name, counter.name);
            assert_eq!(metric.description, counter.description.as_ref());
            assert_eq!(metric.unit, counter.unit);
            let points = match (counter.kind, metric.data.as_ref().unwrap()) {
                (MetricKind::Gauge, metric::Data::Gauge(gauge)) => &gauge.data_points,
                (MetricKind::UpDownCounter, metric::Data::Sum(sum)) => {
                    assert_eq!(
                        sum.aggregation_temporality,
                        AggregationTemporality::Cumulative as i32
                    );
                    assert!(!sum.is_monotonic);
                    &sum.data_points
                }
                _ => panic!("incorrect metric shape"),
            };
            assert_eq!(points.len(), 1);
            let point = &points[0];
            assert_eq!(point.time_unix_nano, timestamp as u64);
            assert_eq!(
                point.start_time_unix_nano,
                if counter.kind == MetricKind::Gauge {
                    0
                } else {
                    start as u64
                }
            );
            assert_eq!(point.flags, 0);
            assert_eq!(
                point.value,
                Some(match value {
                    Number::Integer(value) => number_data_point::Value::AsInt(value),
                    Number::Double(value) => number_data_point::Value::AsDouble(value),
                })
            );
            assert_eq!(
                attributes(&point.attributes),
                BTreeMap::from([("windows.perf_counter.path", counter.path.as_str())])
            );
        }
    }

    /// Scenario: Validated configuration maps exact counters to a shared metric with static attributes.
    /// Guarantees: Observed points share one metric and retain correct path/attribute joins despite omissions and reordering.
    #[test]
    fn projects_shared_metric_with_attributes_and_omissions() {
        let config = RuntimeConfig::from_json(&serde_json::json!({
            "metrics": {
                "windows.processor.time": {
                    "description": "Processor time", "unit": "%", "gauge": {}
                },
                "windows.process.private": {
                    "description": "Private bytes", "unit": "By", "up_down_counter": {}
                }
            },
            "perfcounters": [
                {"object": "Processor", "instances": "_Total", "counters": [
                    {"name": "% Processor Time", "metric": "windows.processor.time",
                     "attributes": {"state": "active"}},
                    {"name": "% Idle Time", "metric": "windows.processor.time",
                     "attributes": {"state": "idle"}},
                    {"name": "% User Time", "metric": "windows.processor.time",
                     "attributes": {"state": "user"}}
                ]},
                {"object": "Process", "instances": "worker#01", "counters": [
                    {"name": "Private Bytes", "metric": "windows.process.private"}
                ]}
            ]
        }))
        .unwrap();
        let records = into_otap(
            &config.counters,
            sample(
                0,
                10,
                vec![
                    point(3, SampleValue::NoObservation),
                    point(1, SampleValue::Value(Number::Double(87.5))),
                    point(2, SampleValue::NoObservation),
                    point(0, SampleValue::Value(Number::Double(12.5))),
                ],
            ),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            records
                .get(ArrowPayloadType::UnivariateMetrics)
                .unwrap()
                .num_rows(),
            1
        );
        assert_eq!(
            records
                .get(ArrowPayloadType::NumberDataPoints)
                .unwrap()
                .num_rows(),
            2
        );
        let decoded = decode(records);
        let metrics = &decoded.resource_metrics[0].scope_metrics[0].metrics;
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "windows.processor.time");
        let Some(metric::Data::Gauge(gauge)) = &metrics[0].data else {
            panic!("expected a Gauge");
        };
        assert_eq!(gauge.data_points.len(), 2);
        for (point, state, path, value) in [
            (
                &gauge.data_points[0],
                "idle",
                r"\Processor(_Total)\% Idle Time",
                87.5,
            ),
            (
                &gauge.data_points[1],
                "active",
                r"\Processor(_Total)\% Processor Time",
                12.5,
            ),
        ] {
            assert_eq!(
                attributes(&point.attributes),
                BTreeMap::from([("state", state), ("windows.perf_counter.path", path)])
            );
            assert_eq!(point.value, Some(number_data_point::Value::AsDouble(value)));
        }
    }

    /// Scenario: A Gauge carries zero, negative or future cumulative start-time state.
    /// Guarantees: Projection ignores that state and never emits a Gauge start time.
    #[test]
    fn gauges_ignore_cumulative_start_time() {
        let counters = [counter("gauge", MetricKind::Gauge)];
        for start in [i64::MIN, -1, 0, 11, i64::MAX] {
            let records = into_otap(
                &counters,
                sample(
                    start,
                    10,
                    vec![point(0, SampleValue::Value(Number::Integer(7)))],
                ),
            )
            .unwrap()
            .unwrap();
            let points = records.get(ArrowPayloadType::NumberDataPoints).unwrap();
            assert!(
                points
                    .column_by_name("start_time_unix_nano")
                    .unwrap()
                    .is_null(0)
            );
        }
    }

    /// Scenario: An emitted cumulative Sum carries an invalid or boundary start timestamp.
    /// Guarantees: Only 0 < start <= observation time is accepted; errors identify the counter and both times.
    #[test]
    fn validates_emitted_sum_start_time() {
        let counters = [counter("sum", MetricKind::UpDownCounter)];
        for start in [i64::MIN, -1, 0, 11, i64::MAX] {
            assert!(matches!(
                into_otap(
                    &counters,
                    sample(
                        start,
                        10,
                        vec![point(0, SampleValue::Value(Number::Integer(1)))]
                    )
                ),
                Err(ArrowError::InvalidArgumentError(message))
                    if message == format!(
                        "UpDownCounter sum (\\Object\\sum) has invalid start time {start} for timestamp 10"
                    )
            ));
        }
        for start in [1, 10] {
            let decoded = decode(
                into_otap(
                    &counters,
                    sample(
                        start,
                        10,
                        vec![point(0, SampleValue::Value(Number::Integer(1)))],
                    ),
                )
                .unwrap()
                .unwrap(),
            );
            let Some(metric::Data::Sum(sum)) =
                &decoded.resource_metrics[0].scope_metrics[0].metrics[0].data
            else {
                panic!("expected a Sum");
            };
            assert_eq!(sum.data_points[0].start_time_unix_nano, start as u64);
        }
    }

    /// Scenario: Observation time is zero or negative, with ready, omitted or empty points.
    /// Guarantees: All samples require a positive observation timestamp; errors retain the invalid value.
    #[test]
    fn rejects_nonpositive_observation_time() {
        let counters = [counter("gauge", MetricKind::Gauge)];
        for timestamp in [i64::MIN, -1, 0] {
            for points in [
                vec![],
                vec![point(0, SampleValue::NoObservation)],
                vec![point(0, SampleValue::Value(Number::Integer(1)))],
            ] {
                assert!(matches!(
                    into_otap(&counters, sample(1, timestamp, points)),
                    Err(ArrowError::InvalidArgumentError(message))
                        if message == format!("sample has non-positive timestamp {timestamp}")
                ));
            }
        }
    }

    /// Scenario: A collection has no points or only no-observation points, including warming Sums.
    /// Guarantees: No batch is emitted and unused cumulative start time is not validated.
    #[test]
    fn omits_empty_and_unobserved_samples() {
        let counters = [counter("sum", MetricKind::UpDownCounter)];
        for start in [-1, 0, 11] {
            for points in [vec![], vec![point(0, SampleValue::NoObservation)]] {
                assert!(
                    into_otap(&counters, sample(start, 10, points))
                        .unwrap()
                        .is_none()
                );
            }
        }
        assert!(into_otap(&[], sample(0, 1, vec![])).unwrap().is_none());
    }

    /// Scenario: An emit-ready point references a missing counter or carries a non-finite double.
    /// Guarantees: Invalid observations are rejected; non-finite errors retain the counter path and value.
    #[test]
    fn rejects_invalid_ready_values() {
        let counters = [counter("gauge", MetricKind::Gauge)];
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                into_otap(
                    &counters,
                    sample(
                        0,
                        1,
                        vec![point(0, SampleValue::Value(Number::Double(value)))]
                    )
                ),
                Err(ArrowError::InvalidArgumentError(message))
                    if message == format!("counter \\Object\\gauge has non-finite double value {value}")
            ));
        }
        assert!(matches!(
            into_otap(
                &counters,
                sample(0, 1, vec![point(1, SampleValue::Value(Number::Integer(1)))])
            ),
            Err(ArrowError::InvalidArgumentError(_))
        ));
    }
}
