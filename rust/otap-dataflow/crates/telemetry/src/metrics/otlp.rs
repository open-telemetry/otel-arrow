// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Projects registry-backed metric sets into standard OTLP metrics.
//!
//! # Why this bridge exists
//!
//! The internal telemetry SDK records a *metric set*: one multivariate value
//! containing several metric fields that share an entity, attributes, and a
//! collection time. The current OTLP and OTAP export paths operate on standard
//! univariate metrics instead. This module is the boundary that expands each
//! field of a metric set into one OTLP [`Metric`], allowing the resulting pdata
//! to flow through either:
//!
//! - an OTLP exporter, which forwards the encoded request; or
//! - an OTAP exporter, which converts the same OTLP request into OTAP Arrow
//!   records before export.
//!
//! The registry first drains its export accumulator into an owned
//! [`MetricExportBatch`]. Consequently, this module never holds the registry
//! mutex while allocating protobuf values, encoding bytes, or waiting for a
//! downstream pipeline.
//!
//! # Input contract
//!
//! [`MetricExportBatch`] is produced only by the registry's atomic export
//! drain. Each [`MetricSetExport`] contains a static descriptor, an owned view
//! of its entity attributes, and a value vector with exactly the same length
//! and ordering as the descriptor's field vector. The metric-set macro and
//! registry maintain the value-kind/instrument pairing. Sum-like descriptors
//! must also declare a [`Temporality`]; a missing temporality is reported as
//! [`Error::MissingTemporality`] instead of emitting an ambiguous OTLP sum.
//!
//! # Output hierarchy
//!
//! Each batch is projected into the following protobuf hierarchy:
//!
//! ```text
//! ExportMetricsServiceRequest
//! `-- ResourceMetrics                         process resource
//!     `-- ScopeMetrics                        one per metric set + entity
//!         |-- InstrumentationScope
//!         |   |-- name                        metric-set descriptor name
//!         |   `-- attributes                  entity attributes
//!         `-- Metric                          one per metric-set field
//!             `-- NumberDataPoint or HistogramDataPoint
//! ```
//!
//! A [`MetricSetExport`] keeps values in the same order as
//! [`MetricsDescriptor::metrics`](crate::descriptor::MetricsDescriptor::metrics).
//! `encode_metric_set` zips those two slices, so the descriptor supplies each
//! OTLP metric's name, description, unit, instrument kind, and temporality while
//! the corresponding [`MetricValue`] supplies its data-point value.
//!
//! Entity attributes are placed on `InstrumentationScope` rather than repeated
//! on every data point. Resource attributes come from the process-level
//! `ResourceMetrics` prototype retained by [`MetricsOtlpEncoder`]. Data-point
//! attributes are therefore empty.
//!
//! # Instrument mapping
//!
//! | Internal instrument | OTLP representation | Start time and semantics |
//! | --- | --- | --- |
//! | `Counter` | monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `UpDownCounter` | non-monotonic `Sum` | Descriptor temporality; delta-window or registration start |
//! | `Gauge` | `Gauge` | Start time is zero, as required for an instantaneous value |
//! | scalar `Histogram` | delta `Histogram` with one observation | Delta-window start; uses the SDK-compatible default explicit bounds |
//! | `Mmsc` | delta, bucketless `Histogram` | Delta-window start; preserves exact min, max, sum, and count |
//!
//! Every point uses [`MetricExportBatch::time_unix_nano`] as its end time. A
//! dirty scalar field is emitted even when its value is zero, because zero may
//! be a meaningful gauge or cumulative transition. An empty `Mmsc` is omitted
//! because its min and max are internal sentinel values, not observations. An
//! otherwise empty batch produces no pdata.
//!
//! Aggregation and reset policy intentionally remain in the registry rather
//! than in this encoder. During an atomic drain, delta sums and histograms are
//! reset, cumulative sums and gauges retain their latest values, and the next
//! delta window begins. The encoder only performs the owned, lock-free
//! projection described above.
//!
//! OTLP integer data points are signed `i64`, whereas internal counters can be
//! `u64`. Values above `i64::MAX` are saturated instead of wrapping. Attribute
//! values retain their native OTLP type; unsigned attribute values follow the
//! same saturation rule, and map attributes become OTLP key-value lists.
//!
//! # Transitional design
//!
//! This univariate projection is a compatibility bridge, not the intended
//! long-term representation of internal metric sets. We plan to investigate
//! native multivariate metric-set support in OTAP so the shared structure does
//! not need to be expanded at this boundary. We may also investigate a native
//! metric-set representation in OTLP if the protocol gains suitable standard
//! support, or if an interoperable extension can be defined. Keeping the
//! projection isolated in this module makes either future path replaceable
//! without changing the hot-path metric-set API or registry aggregation model.

use crate::attributes::{AttributeSetHandler, AttributeValue};
use crate::descriptor::{Instrument, MetricsField, Temporality};
use crate::metrics::{MetricExportBatch, MetricSetExport, MetricValue};
use bytes::Bytes;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, NumberDataPoint,
    ResourceMetrics, ScopeMetrics, Sum,
};
use prost::Message;

/// Errors produced while encoding registry metrics as OTLP.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The pre-encoded resource fragment was invalid.
    #[error("invalid internal telemetry resource: {0}")]
    InvalidResource(#[from] prost::DecodeError),

    /// A sum-like metric did not declare its aggregation temporality.
    #[error("sum metric '{metric}' is missing aggregation temporality")]
    MissingTemporality {
        /// Metric name from the descriptor.
        metric: &'static str,
    },
}

/// Reusable OTLP encoder holding the process resource prototype.
#[derive(Debug, Clone)]
pub struct MetricsOtlpEncoder {
    resource_metrics: ResourceMetrics,
}

impl MetricsOtlpEncoder {
    /// Creates an encoder from the resource fragment shared with internal logs.
    ///
    /// `ResourceLogs` and `ResourceMetrics` use the same field numbers for
    /// `resource` and `schema_url`, so the pre-encoded fragment is valid for
    /// either message type.
    pub fn new(resource_fragment: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            resource_metrics: ResourceMetrics::decode(resource_fragment)?,
        })
    }

    /// Encodes a registry export batch. Empty batches produce no pdata.
    pub fn encode(&self, batch: &MetricExportBatch) -> Result<Option<OtlpProtoBytes>, Error> {
        let mut scope_metrics = Vec::with_capacity(batch.metric_sets.len());
        for metric_set in &batch.metric_sets {
            if let Some(scope) = encode_metric_set(metric_set, batch.time_unix_nano)? {
                scope_metrics.push(scope);
            }
        }

        if scope_metrics.is_empty() {
            return Ok(None);
        }

        let mut resource_metrics = self.resource_metrics.clone();
        resource_metrics.scope_metrics = scope_metrics;
        let request = ExportMetricsServiceRequest::new(vec![resource_metrics]);
        Ok(Some(OtlpProtoBytes::ExportMetricsRequest(Bytes::from(
            request.encode_to_vec(),
        ))))
    }
}

/// Expands one metric set into a scope containing one OTLP metric per field.
///
/// `None` is returned when every field is omitted, which currently occurs for
/// a metric set containing only empty `Mmsc` summaries.
fn encode_metric_set(
    metric_set: &MetricSetExport,
    time_unix_nano: u64,
) -> Result<Option<ScopeMetrics>, Error> {
    let mut metrics = Vec::with_capacity(metric_set.values.len());
    for (field, value) in metric_set.descriptor.metrics.iter().zip(&metric_set.values) {
        if let Some(metric) = encode_metric(field, *value, metric_set, time_unix_nano)? {
            metrics.push(metric);
        }
    }

    if metrics.is_empty() {
        return Ok(None);
    }

    let attributes: Vec<KeyValue> = metric_set
        .attributes
        .iter_attributes()
        .map(|(key, value)| KeyValue::new(key, encode_attribute_value(value)))
        .collect();
    let scope = InstrumentationScope::build()
        .name(metric_set.descriptor.name)
        .attributes(attributes)
        .finish();
    Ok(Some(ScopeMetrics::new(scope, metrics)))
}

/// Projects one multivariate metric field into its univariate OTLP data type.
fn encode_metric(
    field: &'static MetricsField,
    value: MetricValue,
    metric_set: &MetricSetExport,
    time_unix_nano: u64,
) -> Result<Option<Metric>, Error> {
    let data = match field.instrument {
        Instrument::Counter | Instrument::UpDownCounter => {
            let temporality = field
                .temporality
                .ok_or(Error::MissingTemporality { metric: field.name })?;
            let start_time = match temporality {
                Temporality::Delta => metric_set.delta_start_time_unix_nano,
                Temporality::Cumulative => metric_set.cumulative_start_time_unix_nano,
            };
            let point = number_data_point(value, start_time, time_unix_nano);
            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Sum(Sum::new(
                encode_temporality(temporality),
                matches!(field.instrument, Instrument::Counter),
                vec![point],
            ))
        }
        Instrument::Gauge => {
            let point = number_data_point(value, 0, time_unix_nano);
            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Gauge(Gauge::new(vec![
                point,
            ]))
        }
        Instrument::Histogram => {
            let point = scalar_histogram_data_point(
                value,
                metric_set.delta_start_time_unix_nano,
                time_unix_nano,
            );
            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Histogram(
                Histogram::new(AggregationTemporality::Delta, vec![point]),
            )
        }
        Instrument::Mmsc => {
            let MetricValue::Mmsc(snapshot) = value else {
                debug_assert!(false, "MMSC descriptor must have an MMSC value");
                return Ok(None);
            };
            if snapshot.count == 0 {
                return Ok(None);
            }
            let point = HistogramDataPoint::build()
                .start_time_unix_nano(metric_set.delta_start_time_unix_nano)
                .time_unix_nano(time_unix_nano)
                .count(snapshot.count)
                .sum(snapshot.sum)
                .min(snapshot.min)
                .max(snapshot.max)
                .bucket_counts(Vec::<u64>::new())
                .explicit_bounds(Vec::<f64>::new())
                .finish();
            otap_df_pdata::proto::opentelemetry::metrics::v1::metric::Data::Histogram(
                Histogram::new(AggregationTemporality::Delta, vec![point]),
            )
        }
    };

    Ok(Some(Metric {
        name: field.name.to_owned(),
        description: field.brief.to_owned(),
        unit: field.unit.to_owned(),
        metadata: Vec::new(),
        data: Some(data),
    }))
}

/// Creates a scalar OTLP point, saturating unsigned values to OTLP's signed range.
fn number_data_point(
    value: MetricValue,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
) -> NumberDataPoint {
    let builder = NumberDataPoint::build()
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano);
    match value {
        MetricValue::U64(value) => builder.value_int(saturating_i64(value)).finish(),
        MetricValue::F64(value) => builder.value_double(value).finish(),
        MetricValue::Mmsc(_) => {
            debug_assert!(false, "number data point cannot contain MMSC data");
            builder.value_double(0.0).finish()
        }
    }
}

/// Encodes the legacy scalar histogram form as exactly one observation.
///
/// Scalar histograms are not pre-aggregated. Retaining the OpenTelemetry SDK's
/// default explicit bounds keeps this bridge compatible with the previous SDK
/// export path. [`Instrument::Mmsc`] follows a separate, bucketless path above.
fn scalar_histogram_data_point(
    value: MetricValue,
    start_time_unix_nano: u64,
    time_unix_nano: u64,
) -> HistogramDataPoint {
    const DEFAULT_BOUNDS: [f64; 15] = [
        0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0, 5000.0,
        7500.0, 10000.0,
    ];

    let value = match value {
        MetricValue::U64(value) => value as f64,
        MetricValue::F64(value) => value,
        MetricValue::Mmsc(_) => {
            debug_assert!(false, "scalar histogram cannot contain MMSC data");
            0.0
        }
    };
    let bucket = DEFAULT_BOUNDS
        .iter()
        .position(|bound| value <= *bound)
        .unwrap_or(DEFAULT_BOUNDS.len());
    let mut bucket_counts = vec![0; DEFAULT_BOUNDS.len() + 1];
    bucket_counts[bucket] = 1;

    HistogramDataPoint::build()
        .start_time_unix_nano(start_time_unix_nano)
        .time_unix_nano(time_unix_nano)
        .count(1u64)
        .sum(value)
        .min(value)
        .max(value)
        .bucket_counts(bucket_counts)
        .explicit_bounds(DEFAULT_BOUNDS.to_vec())
        .finish()
}

const fn encode_temporality(temporality: Temporality) -> AggregationTemporality {
    match temporality {
        Temporality::Delta => AggregationTemporality::Delta,
        Temporality::Cumulative => AggregationTemporality::Cumulative,
    }
}

/// Preserves internal attribute types in their corresponding OTLP value forms.
fn encode_attribute_value(value: &AttributeValue) -> AnyValue {
    match value {
        AttributeValue::String(value) => AnyValue::new_string(value.clone()),
        AttributeValue::Int(value) => AnyValue::new_int(*value),
        AttributeValue::UInt(value) => AnyValue::new_int(saturating_i64(*value)),
        AttributeValue::Double(value) => AnyValue::new_double(*value),
        AttributeValue::Boolean(value) => AnyValue::new_bool(*value),
        AttributeValue::Map(values) => AnyValue::new_kvlist(
            values
                .iter()
                .map(|(key, value)| KeyValue::new(key, encode_attribute_value(value)))
                .collect::<Vec<_>>(),
        ),
    }
}

/// Converts an unsigned internal value without wrapping OTLP's signed integer.
const fn saturating_i64(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::AttributeSetHandler;
    use crate::descriptor::{
        AttributeField, AttributeValueType, AttributesDescriptor, MetricValueType,
        MetricsDescriptor,
    };
    use crate::entity::{EntityAttributeSet, EntityRegistry};
    use crate::instrument::MmscSnapshot;
    use otap_df_pdata::proto::opentelemetry::common::v1::{KeyValueList, any_value};
    use otap_df_pdata::proto::opentelemetry::logs::v1::ResourceLogs;
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{metric, number_data_point};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::{OtapArrowRecords, OtapPayload, TryIntoWithOptions};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    const DELTA_START: u64 = 10;
    const CUMULATIVE_START: u64 = 5;
    const COLLECTION_TIME: u64 = 20;

    static ALL_METRICS_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.scope",
        metrics: &[
            MetricsField {
                name: "counter.delta",
                unit: "{request}",
                brief: "Delta counter",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "counter.cumulative",
                unit: "By",
                brief: "Cumulative counter",
                instrument: Instrument::Counter,
                temporality: Some(Temporality::Cumulative),
                value_type: MetricValueType::U64,
            },
            MetricsField {
                name: "up_down.delta",
                unit: "1",
                brief: "Delta up/down counter",
                instrument: Instrument::UpDownCounter,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "gauge",
                unit: "Cel",
                brief: "Current gauge",
                instrument: Instrument::Gauge,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "histogram.scalar",
                unit: "ms",
                brief: "Scalar histogram",
                instrument: Instrument::Histogram,
                temporality: None,
                value_type: MetricValueType::F64,
            },
            MetricsField {
                name: "histogram.mmsc",
                unit: "ms",
                brief: "Pre-aggregated histogram",
                instrument: Instrument::Mmsc,
                temporality: Some(Temporality::Delta),
                value_type: MetricValueType::F64,
            },
        ],
    };

    static MMSC_ONLY_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.empty_mmsc",
        metrics: &[MetricsField {
            name: "histogram.empty",
            unit: "ms",
            brief: "Empty pre-aggregated histogram",
            instrument: Instrument::Mmsc,
            temporality: Some(Temporality::Delta),
            value_type: MetricValueType::F64,
        }],
    };

    static INVALID_SUM_DESCRIPTOR: MetricsDescriptor = MetricsDescriptor {
        name: "test.invalid",
        metrics: &[MetricsField {
            name: "invalid.sum",
            unit: "1",
            brief: "Sum without temporality",
            instrument: Instrument::Counter,
            temporality: None,
            value_type: MetricValueType::U64,
        }],
    };

    static EMPTY_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test.empty_attributes",
        fields: &[],
    };

    static FULL_ATTRIBUTES_DESCRIPTOR: AttributesDescriptor = AttributesDescriptor {
        name: "test.full_attributes",
        fields: &[
            AttributeField {
                key: "worker.name",
                brief: "Worker name",
                r#type: AttributeValueType::String,
            },
            AttributeField {
                key: "worker.delta",
                brief: "Signed delta",
                r#type: AttributeValueType::Int,
            },
            AttributeField {
                key: "worker.sequence",
                brief: "Unsigned sequence",
                r#type: AttributeValueType::Int,
            },
            AttributeField {
                key: "worker.load",
                brief: "Worker load",
                r#type: AttributeValueType::Double,
            },
            AttributeField {
                key: "worker.ready",
                brief: "Readiness",
                r#type: AttributeValueType::Boolean,
            },
            AttributeField {
                key: "worker.labels",
                brief: "Labels",
                r#type: AttributeValueType::Map,
            },
        ],
    };

    #[derive(Debug)]
    struct TestAttributeSet {
        descriptor: &'static AttributesDescriptor,
        values: Vec<AttributeValue>,
    }

    impl AttributeSetHandler for TestAttributeSet {
        fn descriptor(&self) -> &'static AttributesDescriptor {
            self.descriptor
        }

        fn attribute_values(&self) -> &[AttributeValue] {
            &self.values
        }
    }

    fn shared_attributes(
        descriptor: &'static AttributesDescriptor,
        values: Vec<AttributeValue>,
    ) -> Arc<EntityAttributeSet> {
        let mut entities = EntityRegistry::default();
        let key = entities
            .register(TestAttributeSet { descriptor, values })
            .key();
        entities.get_shared(key).expect("registered entity")
    }

    fn empty_attributes() -> Arc<EntityAttributeSet> {
        shared_attributes(&EMPTY_ATTRIBUTES_DESCRIPTOR, Vec::new())
    }

    fn metric_set(
        descriptor: &'static MetricsDescriptor,
        attributes: Arc<EntityAttributeSet>,
        values: Vec<MetricValue>,
    ) -> MetricSetExport {
        MetricSetExport {
            descriptor,
            attributes,
            values,
            delta_start_time_unix_nano: DELTA_START,
            cumulative_start_time_unix_nano: CUMULATIVE_START,
        }
    }

    fn empty_resource_encoder() -> MetricsOtlpEncoder {
        MetricsOtlpEncoder::new(&ResourceLogs::default().encode_to_vec())
            .expect("valid resource fragment")
    }

    fn decode_request(encoded: OtlpProtoBytes) -> ExportMetricsServiceRequest {
        let OtlpProtoBytes::ExportMetricsRequest(bytes) = encoded else {
            panic!("encoder returned the wrong OTLP signal")
        };
        ExportMetricsServiceRequest::decode(bytes).expect("valid metrics request")
    }

    fn only_scope(request: &ExportMetricsServiceRequest) -> &ScopeMetrics {
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        let [scope_metrics] = resource_metrics.scope_metrics.as_slice() else {
            panic!("expected one scope metrics message")
        };
        scope_metrics
    }

    fn metric_named<'a>(scope: &'a ScopeMetrics, name: &str) -> &'a Metric {
        scope
            .metrics
            .iter()
            .find(|metric| metric.name == name)
            .unwrap_or_else(|| panic!("missing metric {name}"))
    }

    fn number_point(metric: &Metric) -> (&NumberDataPoint, &Sum) {
        let Some(metric::Data::Sum(sum)) = metric.data.as_ref() else {
            panic!("expected sum metric")
        };
        let [point] = sum.data_points.as_slice() else {
            panic!("expected one number data point")
        };
        (point, sum)
    }

    #[test]
    fn encodes_all_instrument_kinds_with_otlp_semantics() {
        let encoder = empty_resource_encoder();
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &ALL_METRICS_DESCRIPTOR,
                empty_attributes(),
                vec![
                    MetricValue::U64(7),
                    MetricValue::U64(u64::MAX),
                    MetricValue::F64(-2.5),
                    MetricValue::F64(18.25),
                    MetricValue::F64(42.0),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 2.0,
                        max: 9.0,
                        sum: 20.0,
                        count: 4,
                    }),
                ],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let scope = only_scope(&request);
        assert_eq!(scope.scope.as_ref().expect("scope").name, "test.scope");
        assert_eq!(scope.metrics.len(), 6);

        let delta_counter = metric_named(scope, "counter.delta");
        assert_eq!(delta_counter.description, "Delta counter");
        assert_eq!(delta_counter.unit, "{request}");
        let (point, sum) = number_point(delta_counter);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        assert!(sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(7)));

        let cumulative_counter = metric_named(scope, "counter.cumulative");
        let (point, sum) = number_point(cumulative_counter);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Cumulative as i32
        );
        assert!(sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, CUMULATIVE_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsInt(i64::MAX)));

        let up_down = metric_named(scope, "up_down.delta");
        let (point, sum) = number_point(up_down);
        assert_eq!(
            sum.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        assert!(!sum.is_monotonic);
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(-2.5)));

        let gauge = metric_named(scope, "gauge");
        let Some(metric::Data::Gauge(gauge)) = gauge.data.as_ref() else {
            panic!("expected gauge metric")
        };
        let [point] = gauge.data_points.as_slice() else {
            panic!("expected one gauge point")
        };
        assert_eq!(point.start_time_unix_nano, 0);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.value, Some(number_data_point::Value::AsDouble(18.25)));

        let scalar = metric_named(scope, "histogram.scalar");
        let Some(metric::Data::Histogram(histogram)) = scalar.data.as_ref() else {
            panic!("expected histogram metric")
        };
        assert_eq!(
            histogram.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one histogram point")
        };
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.count, 1);
        assert_eq!(point.sum, Some(42.0));
        assert_eq!(point.min, Some(42.0));
        assert_eq!(point.max, Some(42.0));
        assert_eq!(
            point.explicit_bounds,
            vec![
                0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0, 2500.0,
                5000.0, 7500.0, 10000.0,
            ]
        );
        let mut expected_buckets = vec![0; 16];
        expected_buckets[4] = 1;
        assert_eq!(point.bucket_counts, expected_buckets);

        let mmsc = metric_named(scope, "histogram.mmsc");
        let Some(metric::Data::Histogram(histogram)) = mmsc.data.as_ref() else {
            panic!("expected MMSC histogram metric")
        };
        assert_eq!(
            histogram.aggregation_temporality,
            AggregationTemporality::Delta as i32
        );
        let [point] = histogram.data_points.as_slice() else {
            panic!("expected one MMSC histogram point")
        };
        assert_eq!(point.start_time_unix_nano, DELTA_START);
        assert_eq!(point.time_unix_nano, COLLECTION_TIME);
        assert_eq!(point.count, 4);
        assert_eq!(point.sum, Some(20.0));
        assert_eq!(point.min, Some(2.0));
        assert_eq!(point.max, Some(9.0));
        assert!(point.bucket_counts.is_empty());
        assert!(point.explicit_bounds.is_empty());
    }

    #[test]
    fn omits_empty_mmsc_and_empty_batches() {
        let encoder = empty_resource_encoder();
        let empty_mmsc = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::Mmsc(MmscSnapshot {
                    min: f64::MAX,
                    max: f64::MIN,
                    sum: 0.0,
                    count: 0,
                })],
            )],
        };
        assert!(
            encoder
                .encode(&empty_mmsc)
                .expect("encode succeeds")
                .is_none()
        );

        let empty_batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: Vec::new(),
        };
        assert!(
            encoder
                .encode(&empty_batch)
                .expect("encode succeeds")
                .is_none()
        );
    }

    #[test]
    fn attaches_native_attributes_to_scope_and_preserves_resource() {
        let resource = Resource {
            attributes: vec![KeyValue::new(
                "service.name",
                AnyValue::new_string("telemetry-test"),
            )],
            dropped_attributes_count: 2,
            entity_refs: Vec::new(),
        };
        let fragment = ResourceLogs {
            resource: Some(resource.clone()),
            scope_logs: Vec::new(),
            schema_url: "https://resource.example/schema".to_owned(),
        }
        .encode_to_vec();
        let encoder = MetricsOtlpEncoder::new(&fragment).expect("valid log resource fragment");

        let mut labels = BTreeMap::new();
        let _ = labels.insert("overflow".to_owned(), AttributeValue::UInt(u64::MAX));
        let _ = labels.insert(
            "region".to_owned(),
            AttributeValue::String("west".to_owned()),
        );
        let attributes = shared_attributes(
            &FULL_ATTRIBUTES_DESCRIPTOR,
            vec![
                AttributeValue::String("worker-a".to_owned()),
                AttributeValue::Int(-4),
                AttributeValue::UInt(u64::MAX),
                AttributeValue::Double(0.75),
                AttributeValue::Boolean(true),
                AttributeValue::Map(labels),
            ],
        );
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &MMSC_ONLY_DESCRIPTOR,
                attributes,
                vec![MetricValue::Mmsc(MmscSnapshot {
                    min: 1.0,
                    max: 1.0,
                    sum: 1.0,
                    count: 1,
                })],
            )],
        };

        let request = decode_request(
            encoder
                .encode(&batch)
                .expect("encode succeeds")
                .expect("non-empty request"),
        );
        let [resource_metrics] = request.resource_metrics.as_slice() else {
            panic!("expected one resource metrics message")
        };
        assert_eq!(resource_metrics.resource, Some(resource));
        assert_eq!(
            resource_metrics.schema_url,
            "https://resource.example/schema"
        );

        let scope = resource_metrics.scope_metrics[0]
            .scope
            .as_ref()
            .expect("instrumentation scope");
        assert_eq!(scope.name, "test.empty_mmsc");
        assert_eq!(scope.attributes.len(), 6);
        assert_eq!(
            scope.attributes[0]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::StringValue("worker-a".to_owned()))
        );
        assert_eq!(
            scope.attributes[1]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::IntValue(-4))
        );
        assert_eq!(
            scope.attributes[2]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::IntValue(i64::MAX))
        );
        assert_eq!(
            scope.attributes[3]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::DoubleValue(0.75))
        );
        assert_eq!(
            scope.attributes[4]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::BoolValue(true))
        );
        assert_eq!(scope.attributes[5].key, "worker.labels");
        assert_eq!(
            scope.attributes[5]
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::KvlistValue(KeyValueList {
                values: vec![
                    KeyValue::new("overflow", AnyValue::new_int(i64::MAX)),
                    KeyValue::new("region", AnyValue::new_string("west")),
                ],
            }))
        );
    }

    #[test]
    fn rejects_sum_without_temporality() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &INVALID_SUM_DESCRIPTOR,
                empty_attributes(),
                vec![MetricValue::U64(1)],
            )],
        };
        let error = empty_resource_encoder()
            .encode(&batch)
            .expect_err("missing temporality must fail");
        assert!(matches!(
            error,
            Error::MissingTemporality {
                metric: "invalid.sum"
            }
        ));
    }

    #[test]
    fn encoded_metrics_are_consumable_by_the_otap_export_path() {
        let batch = MetricExportBatch {
            time_unix_nano: COLLECTION_TIME,
            metric_sets: vec![metric_set(
                &ALL_METRICS_DESCRIPTOR,
                empty_attributes(),
                vec![
                    MetricValue::U64(7),
                    MetricValue::U64(11),
                    MetricValue::F64(-2.5),
                    MetricValue::F64(18.25),
                    MetricValue::F64(42.0),
                    MetricValue::Mmsc(MmscSnapshot {
                        min: 2.0,
                        max: 9.0,
                        sum: 20.0,
                        count: 4,
                    }),
                ],
            )],
        };
        let encoded = empty_resource_encoder()
            .encode(&batch)
            .expect("OTLP encoding succeeds")
            .expect("batch is non-empty");
        let payload: OtapPayload = encoded.into();

        let records: OtapArrowRecords = payload
            .try_into_with_default()
            .expect("OTAP exporter can convert bridge output to Arrow records");
        assert!(matches!(records, OtapArrowRecords::Metrics(_)));
    }
}
