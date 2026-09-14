// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP metrics to ME-to-FE protocol model mapping.

mod attributes;
mod config;
mod exemplars;
mod histograms;

use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Exemplar as OtlpExemplar, ExponentialHistogramDataPoint,
    HistogramDataPoint, NumberDataPoint, ScopeMetrics, exemplar, metric, number_data_point,
};
use prost::Message;
use thiserror::Error;

use super::encoder::{
    COUNT, EXEMPLAR, HISTOGRAM, IS_RAW_DATA, MAX, METRIC_ORIGIN_OPEN_TELEMETRY,
    METRIC_TYPE_CUMULATIVE_COUNTER, METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM,
    METRIC_TYPE_CUMULATIVE_HISTOGRAM, METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER,
    METRIC_TYPE_DELTA_COUNTER, METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM,
    METRIC_TYPE_DELTA_HISTOGRAM, METRIC_TYPE_GAUGE, MIN, Metric, MetricExemplar, MetricHistogram,
    MetricValues, NumericValues, Packet, SUM,
};
#[cfg(test)]
use attributes::{ACCOUNT_ATTRIBUTE, MAX_DIMENSION_NAME_CHARS, MAX_DIMENSION_VALUE_CHARS};
use attributes::{
    NAMESPACE_ATTRIBUTE, attribute_string, overflow_diagnostic, point_context, resource_context,
    selected_scope_dimensions, string_value,
};
use exemplars::retain_exemplars_within_limits;
use histograms::{
    MAX_EXPONENTIAL_SCALE, MIN_EXPONENTIAL_SCALE, bucket_sum, downscale_if_required,
    explicit_histogram, sparse_buckets, valid_explicit_histogram,
};

pub use config::{Config, ScopeAttributes};

const DOTNET_UNIX_EPOCH_OFFSET_SECONDS: u64 = 62_135_596_800;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_DOTNET_TICK: u64 = 100;
const DOTNET_TICKS_PER_SECOND: u64 = NANOS_PER_SECOND / NANOS_PER_DOTNET_TICK;
const MAX_EXEMPLAR_AGE_NANOS: u64 = 60 * NANOS_PER_SECOND;

/// A protocol packet and the monitoring account to which it must be published.
#[derive(Clone, Debug, PartialEq)]
pub struct Publication {
    /// Monitoring account selected by OTLP attribute precedence.
    pub monitoring_account: String,
    /// ME-to-FE packet for this account.
    pub packet: Packet,
}

/// Result of mapping one OTLP export request.
#[derive(Clone, Debug, PartialEq)]
pub struct MappingOutcome {
    /// Packets grouped by monitoring account in first-seen order.
    pub publications: Vec<Publication>,
    /// Number of unsupported or invalid data points that were not mapped.
    pub rejected_data_points: usize,
    /// Cardinality-overflow diagnostics generated from reserved OTLP metadata.
    pub cardinality_overflows: Vec<CardinalityOverflow>,
}

/// Diagnostic context for an OTLP data point marked as a cardinality overflow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardinalityOverflow {
    /// Monitoring account selected for the faulting metric.
    pub monitoring_account: String,
    /// Namespace selected for the faulting metric.
    pub namespace: String,
    /// Name of the faulting metric.
    pub metric_name: String,
}

/// OTLP decoding or request-level mapping failure.
#[derive(Debug, Error)]
pub enum MappingError {
    /// The protobuf request is malformed.
    #[error("invalid OTLP metrics request: {0}")]
    Decode(#[from] prost::DecodeError),
    /// The packet timestamp exceeds the ME time representation.
    #[error("OTLP receive timestamp exceeds the ME time representation")]
    ReceiveTimeOverflow,
}

#[derive(Clone, Debug)]
struct ResourceContext {
    monitoring_account: String,
    namespace: String,
    dimensions: Vec<super::encoder::Dimension>,
}

#[derive(Clone, Debug)]
struct PointContext {
    monitoring_account: String,
    namespace: String,
    dimensions: Vec<super::encoder::Dimension>,
}

struct MapPointResult {
    metric: Option<(String, Metric)>,
    overflow: Option<CardinalityOverflow>,
}

impl MapPointResult {
    fn rejected(overflow: Option<CardinalityOverflow>) -> Self {
        Self {
            metric: None,
            overflow,
        }
    }

    fn mapped(
        monitoring_account: String,
        metric: Metric,
        overflow: Option<CardinalityOverflow>,
    ) -> Self {
        Self {
            metric: Some((monitoring_account, metric)),
            overflow,
        }
    }
}

/// Decodes an OTLP export request and maps its supported data points.
pub fn decode_and_map(
    bytes: &[u8],
    config: &Config,
    receive_time_unix_nano: u64,
) -> Result<MappingOutcome, MappingError> {
    let request = ExportMetricsServiceRequest::decode(bytes)?;
    map_request(&request, config, receive_time_unix_nano)
}

/// Maps a decoded OTLP export request into packets grouped by monitoring account.
pub fn map_request(
    request: &ExportMetricsServiceRequest,
    config: &Config,
    receive_time_unix_nano: u64,
) -> Result<MappingOutcome, MappingError> {
    let current_time_bucket = unix_nanos_to_dotnet_seconds_floor(receive_time_unix_nano)
        .ok_or(MappingError::ReceiveTimeOverflow)?;
    let mut outcome = MappingOutcome {
        publications: Vec::new(),
        rejected_data_points: 0,
        cardinality_overflows: Vec::new(),
    };

    for resource_metrics in &request.resource_metrics {
        let resource = resource_metrics.resource.as_ref().map_or_else(
            || resource_context(&[], config),
            |resource| resource_context(&resource.attributes, config),
        );

        for scope_metrics in &resource_metrics.scope_metrics {
            map_scope(
                scope_metrics,
                &resource,
                config,
                current_time_bucket,
                &mut outcome,
            );
        }
    }

    Ok(outcome)
}

fn map_scope(
    scope_metrics: &ScopeMetrics,
    resource: &ResourceContext,
    config: &Config,
    current_time_bucket: u64,
    outcome: &mut MappingOutcome,
) {
    let scope_namespace = scope_metrics
        .scope
        .as_ref()
        .filter(|scope| !scope.name.is_empty())
        .and_then(|scope| attribute_string(&scope.attributes, NAMESPACE_ATTRIBUTE))
        .unwrap_or_else(|| resource.namespace.clone());
    let scope_dimensions = selected_scope_dimensions(scope_metrics, config);

    for otlp_metric in &scope_metrics.metrics {
        let Some(data) = &otlp_metric.data else {
            outcome.rejected_data_points += 1;
            continue;
        };

        match data {
            metric::Data::Gauge(gauge) => {
                for point in &gauge.data_points {
                    record_mapped_metric(
                        outcome,
                        current_time_bucket,
                        map_number_point(
                            &otlp_metric.name,
                            point,
                            resource,
                            &scope_namespace,
                            &scope_dimensions,
                            config,
                            METRIC_TYPE_GAUGE,
                        ),
                    );
                }
            }
            metric::Data::Sum(sum) => {
                let metric_type = sum_metric_type(sum.is_monotonic, sum.aggregation_temporality);
                let Some(metric_type) = metric_type else {
                    outcome.rejected_data_points += sum.data_points.len();
                    continue;
                };
                for point in &sum.data_points {
                    record_mapped_metric(
                        outcome,
                        current_time_bucket,
                        map_number_point(
                            &otlp_metric.name,
                            point,
                            resource,
                            &scope_namespace,
                            &scope_dimensions,
                            config,
                            metric_type,
                        ),
                    );
                }
            }
            metric::Data::Histogram(histogram) => {
                let metric_type = explicit_histogram_metric_type(histogram.aggregation_temporality);
                for point in &histogram.data_points {
                    record_mapped_metric(
                        outcome,
                        current_time_bucket,
                        map_histogram_point(
                            &otlp_metric.name,
                            point,
                            resource,
                            &scope_namespace,
                            &scope_dimensions,
                            config,
                            metric_type,
                        ),
                    );
                }
            }
            metric::Data::ExponentialHistogram(histogram) => {
                let metric_type =
                    exponential_histogram_metric_type(histogram.aggregation_temporality);
                for point in &histogram.data_points {
                    record_mapped_metric(
                        outcome,
                        current_time_bucket,
                        map_exponential_histogram_point(
                            &otlp_metric.name,
                            point,
                            resource,
                            &scope_namespace,
                            &scope_dimensions,
                            config,
                            metric_type,
                        ),
                    );
                }
            }
            metric::Data::Summary(summary) => {
                outcome.rejected_data_points += summary.data_points.len();
            }
        }
    }
}

fn map_number_point(
    name: &str,
    point: &NumberDataPoint,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[super::encoder::Dimension],
    config: &Config,
    metric_type: u32,
) -> MapPointResult {
    if is_stale(point.flags) {
        return MapPointResult::rejected(None);
    }
    let overflow = overflow_diagnostic(&point.attributes, resource, scope_namespace, name);
    if name.is_empty() {
        return MapPointResult::rejected(overflow);
    }
    let Some(context) = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    ) else {
        return MapPointResult::rejected(overflow);
    };
    let mut sampling_type = SUM | COUNT | IS_RAW_DATA | metric_type | METRIC_ORIGIN_OPEN_TELEMETRY;
    let exemplars = map_point_exemplars(
        config,
        &point.exemplars,
        point.time_unix_nano,
        &mut sampling_type,
    );
    let values = match point.value.unwrap_or(number_data_point::Value::AsInt(0)) {
        number_data_point::Value::AsDouble(value) => MetricValues::Double(NumericValues {
            min: None,
            max: None,
            sum: Some(value),
            count: Some(1),
            milliseconds: None,
            histogram: None,
        }),
        number_data_point::Value::AsInt(value) => MetricValues::Unsigned(NumericValues {
            min: None,
            max: None,
            sum: Some(value as u64),
            count: Some(1),
            milliseconds: None,
            histogram: None,
        }),
    };

    let Some(time_bucket) = unix_nanos_to_dotnet_seconds_ceil(point.time_unix_nano) else {
        return MapPointResult::rejected(overflow);
    };
    MapPointResult::mapped(
        context.monitoring_account,
        Metric {
            time_bucket: time_bucket as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values,
            exemplars,
        },
        overflow,
    )
}

fn map_histogram_point(
    name: &str,
    point: &HistogramDataPoint,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[super::encoder::Dimension],
    config: &Config,
    metric_type: u32,
) -> MapPointResult {
    if is_stale(point.flags) || !valid_explicit_histogram(point) {
        return MapPointResult::rejected(None);
    }
    let overflow = overflow_diagnostic(&point.attributes, resource, scope_namespace, name);
    if name.is_empty() {
        return MapPointResult::rejected(overflow);
    }
    let Some(context) = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    ) else {
        return MapPointResult::rejected(overflow);
    };
    let histogram = explicit_histogram(point);
    let mut sampling_type = MIN
        | MAX
        | SUM
        | COUNT
        | HISTOGRAM
        | IS_RAW_DATA
        | metric_type
        | METRIC_ORIGIN_OPEN_TELEMETRY;
    let exemplars = map_point_exemplars(
        config,
        &point.exemplars,
        point.time_unix_nano,
        &mut sampling_type,
    );

    let Some(time_bucket) = unix_nanos_to_dotnet_seconds_ceil(point.time_unix_nano) else {
        return MapPointResult::rejected(overflow);
    };
    MapPointResult::mapped(
        context.monitoring_account,
        Metric {
            time_bucket: time_bucket as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values: MetricValues::Double(NumericValues {
                min: Some(point.min.unwrap_or(0.0)),
                max: Some(point.max.unwrap_or(0.0)),
                sum: Some(point.sum.unwrap_or(0.0)),
                count: Some(me_scalar_count(point.count)),
                milliseconds: None,
                histogram,
            }),
            exemplars,
        },
        overflow,
    )
}

fn map_exponential_histogram_point(
    name: &str,
    point: &ExponentialHistogramDataPoint,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[super::encoder::Dimension],
    config: &Config,
    metric_type: u32,
) -> MapPointResult {
    if is_stale(point.flags)
        || !(MIN_EXPONENTIAL_SCALE..=MAX_EXPONENTIAL_SCALE).contains(&point.scale)
    {
        return MapPointResult::rejected(None);
    }
    let Some(mut positive) = sparse_buckets(point.positive.as_ref()) else {
        return MapPointResult::rejected(None);
    };
    let Some(mut negative) = sparse_buckets(point.negative.as_ref()) else {
        return MapPointResult::rejected(None);
    };
    let Some(positive_total) = bucket_sum(&positive) else {
        return MapPointResult::rejected(None);
    };
    let Some(negative_total) = bucket_sum(&negative) else {
        return MapPointResult::rejected(None);
    };
    let Some(bucket_total) = point
        .zero_count
        .checked_add(positive_total)
        .and_then(|total| total.checked_add(negative_total))
    else {
        return MapPointResult::rejected(None);
    };
    if bucket_total != point.count {
        return MapPointResult::rejected(None);
    }
    let mut scale = point.scale;
    downscale_if_required(&mut scale, &mut positive, &mut negative);
    let Some(scale) = i8::try_from(scale).ok() else {
        return MapPointResult::rejected(None);
    };
    let overflow = overflow_diagnostic(&point.attributes, resource, scope_namespace, name);
    if name.is_empty() {
        return MapPointResult::rejected(overflow);
    }
    let Some(context) = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    ) else {
        return MapPointResult::rejected(overflow);
    };
    let mut sampling_type = MIN
        | MAX
        | SUM
        | COUNT
        | HISTOGRAM
        | IS_RAW_DATA
        | metric_type
        | METRIC_ORIGIN_OPEN_TELEMETRY;
    let exemplars = map_point_exemplars(
        config,
        &point.exemplars,
        point.time_unix_nano,
        &mut sampling_type,
    );

    let Some(time_bucket) = unix_nanos_to_dotnet_seconds_ceil(point.time_unix_nano) else {
        return MapPointResult::rejected(overflow);
    };
    MapPointResult::mapped(
        context.monitoring_account,
        Metric {
            time_bucket: time_bucket as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values: MetricValues::Double(NumericValues {
                min: Some(point.min.unwrap_or(0.0)),
                max: Some(point.max.unwrap_or(0.0)),
                sum: Some(point.sum.unwrap_or(0.0)),
                count: Some(me_scalar_count(point.count)),
                milliseconds: None,
                histogram: Some(MetricHistogram::Exponential(
                    super::encoder::ExponentialHistogram {
                        scale,
                        zero_count: point.zero_count,
                        negative,
                        positive,
                    },
                )),
            }),
            exemplars,
        },
        overflow,
    )
}

fn record_mapped_metric(
    outcome: &mut MappingOutcome,
    current_time_bucket: u64,
    result: MapPointResult,
) {
    if let Some(overflow) = result.overflow {
        outcome.cardinality_overflows.push(overflow);
    }
    let Some((monitoring_account, metric)) = result.metric else {
        outcome.rejected_data_points += 1;
        return;
    };
    if !add_metric(outcome, monitoring_account, current_time_bucket, metric) {
        outcome.rejected_data_points += 1;
    }
}

fn add_metric(
    outcome: &mut MappingOutcome,
    monitoring_account: String,
    current_time_bucket: u64,
    metric: Metric,
) -> bool {
    if super::encoder::validate_metric(&metric, current_time_bucket).is_err() {
        return false;
    }
    if let Some(publication) = outcome
        .publications
        .iter_mut()
        .find(|publication| publication.monitoring_account == monitoring_account)
    {
        publication.packet.metrics.push(metric);
        return true;
    }
    outcome.publications.push(Publication {
        monitoring_account,
        packet: Packet {
            current_time_bucket,
            metrics: vec![metric],
        },
    });
    true
}

fn sum_metric_type(is_monotonic: bool, temporality: i32) -> Option<u32> {
    let is_delta = temporality == AggregationTemporality::Delta as i32;
    match (is_monotonic, is_delta) {
        (true, true) => Some(METRIC_TYPE_DELTA_COUNTER),
        (true, false) => Some(METRIC_TYPE_CUMULATIVE_COUNTER),
        (false, true) => None,
        (false, false) => Some(METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER),
    }
}

fn explicit_histogram_metric_type(temporality: i32) -> u32 {
    if AggregationTemporality::try_from(temporality) == Ok(AggregationTemporality::Delta) {
        METRIC_TYPE_DELTA_HISTOGRAM
    } else {
        METRIC_TYPE_CUMULATIVE_HISTOGRAM
    }
}

fn exponential_histogram_metric_type(temporality: i32) -> u32 {
    if AggregationTemporality::try_from(temporality) == Ok(AggregationTemporality::Delta) {
        METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM
    } else {
        METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM
    }
}

fn map_point_exemplars(
    config: &Config,
    exemplars: &[OtlpExemplar],
    data_point_time: u64,
    sampling_type: &mut u32,
) -> Vec<MetricExemplar> {
    if config.disable_exemplars {
        return Vec::new();
    }
    let exemplars = map_exemplars(exemplars, data_point_time);
    if !exemplars.is_empty() {
        *sampling_type |= EXEMPLAR;
    }
    exemplars
}

fn map_exemplars(exemplars: &[OtlpExemplar], data_point_time: u64) -> Vec<MetricExemplar> {
    let mut mapped = exemplars
        .iter()
        .filter(|exemplar| {
            exemplar
                .time_unix_nano
                .saturating_add(MAX_EXEMPLAR_AGE_NANOS)
                >= data_point_time
        })
        .map(map_exemplar)
        .collect();
    retain_exemplars_within_limits(&mut mapped);
    mapped
}

fn map_exemplar(exemplar: &OtlpExemplar) -> MetricExemplar {
    let value = match exemplar.value.unwrap_or(exemplar::Value::AsInt(0)) {
        exemplar::Value::AsDouble(value) => value,
        exemplar::Value::AsInt(value) => value as f64,
    };
    MetricExemplar {
        value,
        time_unix_nano: Some(exemplar.time_unix_nano),
        trace_id: fixed_id::<16>(&exemplar.trace_id),
        span_id: fixed_id::<8>(&exemplar.span_id),
        sample_count: None,
        filtered_attributes: exemplar
            .filtered_attributes
            .iter()
            .map(|attribute| {
                (
                    attribute.key.clone(),
                    string_value(attribute).unwrap_or_default(),
                )
            })
            .collect(),
    }
}

fn fixed_id<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    let value = <[u8; N]>::try_from(bytes).ok()?;
    (value != [0; N]).then_some(value)
}

fn unix_nanos_to_dotnet_seconds_floor(value: u64) -> Option<u64> {
    let ticks = unix_nanos_to_dotnet_ticks(value)?;
    let seconds = ticks / DOTNET_TICKS_PER_SECOND;
    DOTNET_UNIX_EPOCH_OFFSET_SECONDS.checked_add(seconds)
}

fn unix_nanos_to_dotnet_seconds_ceil(value: u64) -> Option<u64> {
    let ticks = unix_nanos_to_dotnet_ticks(value)?;
    let seconds = ticks / DOTNET_TICKS_PER_SECOND;
    let round_up = u64::from(ticks % DOTNET_TICKS_PER_SECOND != 0);
    DOTNET_UNIX_EPOCH_OFFSET_SECONDS
        .checked_add(seconds)?
        .checked_add(round_up)
}

fn unix_nanos_to_dotnet_ticks(value: u64) -> Option<u64> {
    (value <= i64::MAX as u64).then_some(value / NANOS_PER_DOTNET_TICK)
}

fn me_scalar_count(value: u64) -> u64 {
    u64::from(value as u32)
}

fn is_stale(flags: u32) -> bool {
    flags & 1 != 0
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        ExponentialHistogram, Gauge, Histogram, Metric as OtlpMetric, ResourceMetrics, Sum,
        exponential_histogram_data_point,
    };
    use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;

    use super::*;
    use crate::exporters::geneva_metrics_exporter::encoder::{
        METRIC_TYPE_DELTA_COUNTER, METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM,
    };

    const TEST_TIME_NANOS: u64 = 1_700_000_000 * NANOS_PER_SECOND;

    fn config() -> Config {
        Config {
            monitoring_account: "default-account".to_string(),
            metric_namespace: "default-namespace".to_string(),
            resource_attributes: Vec::new(),
            honor_resource_attributes: false,
            scope_attributes: Vec::new(),
            honor_scope_attributes: false,
            disable_exemplars: false,
        }
    }

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
        }
    }

    fn int_attribute(key: &str, value: i64) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::IntValue(value)),
            }),
        }
    }

    fn bool_attribute(key: &str, value: bool) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::BoolValue(value)),
            }),
        }
    }

    fn request(
        resource_attributes: Vec<KeyValue>,
        scope: ScopeMetrics,
    ) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: resource_attributes,
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_metrics: vec![scope],
                schema_url: String::new(),
            }],
        }
    }

    fn scope_with_metrics(metrics: Vec<OtlpMetric>) -> ScopeMetrics {
        ScopeMetrics {
            metrics,
            ..Default::default()
        }
    }

    fn otlp_metric(name: impl Into<String>, data: metric::Data) -> OtlpMetric {
        OtlpMetric {
            name: name.into(),
            data: Some(data),
            ..Default::default()
        }
    }

    fn gauge_metric(name: impl Into<String>, point: NumberDataPoint) -> OtlpMetric {
        otlp_metric(
            name,
            metric::Data::Gauge(Gauge {
                data_points: vec![point],
            }),
        )
    }

    fn gauge_point(attributes: Vec<KeyValue>) -> NumberDataPoint {
        NumberDataPoint {
            attributes,
            time_unix_nano: TEST_TIME_NANOS,
            value: Some(number_data_point::Value::AsDouble(1.0)),
            ..Default::default()
        }
    }

    /// Scenario: A delta OTLP integer sum contains routing attributes and the SDK cardinality-overflow marker.
    /// Guarantees: Point routing wins, overflow metadata produces a diagnostic instead of a dimension, and raw ME counter flags are selected.
    #[test]
    fn maps_delta_sum_with_attribute_precedence() {
        let point = NumberDataPoint {
            attributes: vec![
                int_attribute("z-dimension", 7),
                string_attribute(NAMESPACE_ATTRIBUTE, "point-namespace"),
                string_attribute(ACCOUNT_ATTRIBUTE, "point-account"),
                string_attribute("a-dimension", "first"),
                string_attribute("shared", "point"),
                bool_attribute("otel.metric.overflow", true),
            ],
            start_time_unix_nano: TEST_TIME_NANOS - NANOS_PER_SECOND,
            time_unix_nano: TEST_TIME_NANOS,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(number_data_point::Value::AsInt(42)),
        };
        let scope = ScopeMetrics {
            scope: Some(InstrumentationScope {
                name: "meter".to_string(),
                version: String::new(),
                attributes: vec![
                    string_attribute(NAMESPACE_ATTRIBUTE, "scope-namespace"),
                    string_attribute("shared", "scope"),
                ],
                dropped_attributes_count: 0,
            }),
            metrics: vec![OtlpMetric {
                name: "requests".to_string(),
                description: String::new(),
                unit: String::new(),
                metadata: Vec::new(),
                data: Some(metric::Data::Sum(Sum {
                    data_points: vec![point],
                    aggregation_temporality: AggregationTemporality::Delta as i32,
                    is_monotonic: true,
                })),
            }],
            schema_url: String::new(),
        };
        let request = request(
            vec![
                string_attribute(ACCOUNT_ATTRIBUTE, "resource-account"),
                string_attribute(NAMESPACE_ATTRIBUTE, "resource-namespace"),
                string_attribute("shared", "resource"),
            ],
            scope,
        );
        let mut mapping_config = config();
        mapping_config.resource_attributes = vec!["shared".to_string()];
        mapping_config.scope_attributes = vec![ScopeAttributes {
            name: "meter".to_string(),
            keys: vec!["shared".to_string()],
        }];
        let mapped =
            map_request(&request, &mapping_config, TEST_TIME_NANOS).expect("request should map");

        assert_eq!(mapped.rejected_data_points, 0);
        assert_eq!(mapped.publications.len(), 1);
        let publication = &mapped.publications[0];
        assert_eq!(publication.monitoring_account, "point-account");
        let metric = &publication.packet.metrics[0];
        assert_eq!(metric.namespace, "point-namespace");
        assert_eq!(metric.name, "requests");
        assert_eq!(
            metric.sampling_type,
            SUM | COUNT | IS_RAW_DATA | METRIC_TYPE_DELTA_COUNTER | METRIC_ORIGIN_OPEN_TELEMETRY
        );
        assert_eq!(
            metric.dimensions,
            vec![
                super::super::encoder::Dimension {
                    name: "a-dimension".to_string(),
                    value: "first".to_string(),
                },
                super::super::encoder::Dimension {
                    name: "shared".to_string(),
                    value: "point".to_string(),
                },
                super::super::encoder::Dimension {
                    name: "z-dimension".to_string(),
                    value: "7".to_string(),
                },
            ]
        );
        assert_eq!(
            mapped.cardinality_overflows,
            vec![CardinalityOverflow {
                monitoring_account: "point-account".to_string(),
                namespace: "point-namespace".to_string(),
                metric_name: "requests".to_string(),
            }]
        );

        mapping_config.honor_resource_attributes = true;
        mapping_config.honor_scope_attributes = true;
        let honored = map_request(&request, &mapping_config, TEST_TIME_NANOS)
            .expect("request should map with honored attributes");
        assert_eq!(
            honored.publications[0].packet.metrics[0]
                .dimensions
                .iter()
                .find(|dimension| dimension.name == "shared")
                .map(|dimension| dimension.value.as_str()),
            Some("scope")
        );
    }

    /// Scenario: Monotonic and non-monotonic sums use an unrecognized aggregation temporality.
    /// Guarantees: Every non-delta value follows ME's cumulative counter and up-down-counter paths.
    #[test]
    fn maps_unknown_sum_temporalities_as_cumulative() {
        let unknown_temporality = i32::MAX;
        let sum = |is_monotonic| {
            metric::Data::Sum(Sum {
                data_points: vec![gauge_point(Vec::new())],
                aggregation_temporality: unknown_temporality,
                is_monotonic,
            })
        };
        let scope = scope_with_metrics(vec![
            otlp_metric("counter", sum(true)),
            otlp_metric("up-down-counter", sum(false)),
        ]);

        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 0);
        assert_eq!(mapped.publications.len(), 1);
        let metrics = &mapped.publications[0].packet.metrics;
        assert_eq!(metrics.len(), 2);
        assert_eq!(
            metrics[0].sampling_type & super::super::encoder::METRIC_TYPE_MASK,
            METRIC_TYPE_CUMULATIVE_COUNTER
        );
        assert_eq!(
            metrics[1].sampling_type & super::super::encoder::METRIC_TYPE_MASK,
            METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER
        );
    }

    /// Scenario: OTLP explicit and exponential histograms contain valid distributions.
    /// Guarantees: Dense OTLP buckets become the C++-compatible explicit and sparse exponential ME histogram models.
    #[test]
    fn maps_histogram_distributions() {
        let explicit_point = HistogramDataPoint {
            attributes: Vec::new(),
            start_time_unix_nano: 0,
            time_unix_nano: TEST_TIME_NANOS,
            count: 6,
            sum: Some(9.0),
            bucket_counts: vec![1, 2, 3],
            explicit_bounds: vec![1.0, 2.0],
            exemplars: Vec::new(),
            flags: 0,
            min: Some(0.5),
            max: Some(3.0),
        };
        let exponential_point = ExponentialHistogramDataPoint {
            attributes: Vec::new(),
            start_time_unix_nano: 0,
            time_unix_nano: TEST_TIME_NANOS,
            count: 6,
            sum: Some(9.0),
            scale: 2,
            zero_count: 1,
            positive: Some(exponential_histogram_data_point::Buckets {
                offset: 3,
                bucket_counts: vec![2, 0, 1],
            }),
            negative: Some(exponential_histogram_data_point::Buckets {
                offset: -2,
                bucket_counts: vec![0, 2],
            }),
            flags: 0,
            exemplars: Vec::new(),
            min: None,
            max: None,
            zero_threshold: 0.0,
        };
        let scope = ScopeMetrics {
            scope: None,
            metrics: vec![
                OtlpMetric {
                    name: "explicit".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Histogram(Histogram {
                        data_points: vec![explicit_point],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    })),
                },
                OtlpMetric {
                    name: "exponential".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                        data_points: vec![exponential_point],
                        aggregation_temporality: AggregationTemporality::Delta as i32,
                    })),
                },
            ],
            schema_url: String::new(),
        };
        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        let metrics = &mapped.publications[0].packet.metrics;
        assert_eq!(
            metrics[0].values,
            MetricValues::Double(NumericValues {
                min: Some(0.5),
                max: Some(3.0),
                sum: Some(9.0),
                count: Some(6),
                milliseconds: None,
                histogram: Some(MetricHistogram::Explicit(vec![
                    (1.0, 1),
                    (2.0, 2),
                    (3.0, 3),
                ])),
            })
        );
        assert_eq!(
            metrics[1].sampling_type & super::super::encoder::METRIC_TYPE_MASK,
            METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM
        );
        assert_eq!(
            metrics[1].values,
            MetricValues::Double(NumericValues {
                min: Some(0.0),
                max: Some(0.0),
                sum: Some(9.0),
                count: Some(6),
                milliseconds: None,
                histogram: Some(MetricHistogram::Exponential(
                    super::super::encoder::ExponentialHistogram {
                        scale: 2,
                        zero_count: 1,
                        negative: vec![(-1, 2)],
                        positive: vec![(3, 2), (5, 1)],
                    },
                )),
            })
        );
        assert_eq!(metrics[0].sampling_type & (MIN | MAX), MIN | MAX);
        assert_eq!(metrics[1].sampling_type & (MIN | MAX), MIN | MAX);
        assert_ne!(metrics[0].sampling_type & IS_RAW_DATA, 0);
        assert_ne!(metrics[1].sampling_type & IS_RAW_DATA, 0);
    }

    /// Scenario: Explicit and exponential histogram scalar counts exceed the Geneva u32 wire field.
    /// Guarantees: Counts use ME's narrowing conversion while distributions and all metrics remain encodable.
    #[test]
    fn narrows_large_histogram_scalar_counts() {
        let count = u64::from(u32::MAX) + 1;
        for (input, expected) in [
            (0, 0),
            (u64::from(u32::MAX), u64::from(u32::MAX)),
            (count, 0),
            (u64::MAX, u64::from(u32::MAX)),
        ] {
            assert_eq!(me_scalar_count(input), expected);
        }
        let scope = scope_with_metrics(vec![
            gauge_metric("valid", gauge_point(Vec::new())),
            otlp_metric(
                "explicit",
                metric::Data::Histogram(Histogram {
                    data_points: vec![HistogramDataPoint {
                        time_unix_nano: TEST_TIME_NANOS,
                        count,
                        sum: Some(1.0),
                        bucket_counts: vec![count],
                        ..Default::default()
                    }],
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                }),
            ),
            otlp_metric(
                "exponential",
                metric::Data::ExponentialHistogram(ExponentialHistogram {
                    data_points: vec![ExponentialHistogramDataPoint {
                        time_unix_nano: TEST_TIME_NANOS,
                        count,
                        sum: Some(1.0),
                        zero_count: count,
                        ..Default::default()
                    }],
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                }),
            ),
        ]);
        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 0);
        assert_eq!(mapped.publications.len(), 1);
        assert_eq!(mapped.publications[0].packet.metrics.len(), 3);
        assert_eq!(mapped.publications[0].packet.metrics[0].name, "valid");
        for metric in &mapped.publications[0].packet.metrics[1..] {
            let MetricValues::Double(values) = &metric.values else {
                panic!("histogram should use double values");
            };
            assert_eq!(values.count, Some(0));
        }
        let _ = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("all mapped metrics should encode");
    }

    /// Scenario: Receive and datapoint timestamps share a fractional whole-second boundary.
    /// Guarantees: Protocol serialization time floors while the datapoint bucket rounds up like ME.
    #[test]
    fn uses_distinct_receive_and_datapoint_timestamp_bucketing() {
        let exact = 45 * NANOS_PER_SECOND;
        let fractional = exact + 120_000_000;

        for (nanos, expected) in [
            (99, DOTNET_UNIX_EPOCH_OFFSET_SECONDS),
            (100, DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 1),
            (exact, DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 45),
            (fractional, DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 46),
        ] {
            assert_eq!(unix_nanos_to_dotnet_seconds_ceil(nanos), Some(expected));
        }
        assert_eq!(
            unix_nanos_to_dotnet_seconds_floor(fractional),
            Some(DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 45)
        );
        assert_eq!(unix_nanos_to_dotnet_seconds_ceil(i64::MAX as u64 + 1), None);

        let scope = scope_with_metrics(vec![gauge_metric(
            "fractional",
            NumberDataPoint {
                time_unix_nano: fractional,
                ..gauge_point(Vec::new())
            },
        )]);
        let mapped = map_request(&request(Vec::new(), scope), &config(), fractional)
            .expect("request should map");

        assert_eq!(
            mapped.publications[0].packet.current_time_bucket,
            DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 45
        );
        assert_eq!(
            mapped.publications[0].packet.metrics[0].time_bucket,
            (DOTNET_UNIX_EPOCH_OFFSET_SECONDS + 46) as i64
        );
    }

    /// Scenario: An explicit histogram reports only scalar count and sum without a bucket distribution.
    /// Guarantees: The mapped FE metric retains scalar fields without a histogram body and remains encodable.
    #[test]
    fn maps_distributionless_explicit_histogram_as_scalar_only() {
        let point = HistogramDataPoint {
            time_unix_nano: TEST_TIME_NANOS,
            count: 17,
            sum: Some(12.5),
            ..Default::default()
        };
        let scope = scope_with_metrics(vec![otlp_metric(
            "scalar-histogram",
            metric::Data::Histogram(Histogram {
                data_points: vec![point],
                aggregation_temporality: AggregationTemporality::Cumulative as i32,
            }),
        )]);

        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");
        let metric = &mapped.publications[0].packet.metrics[0];

        assert_eq!(
            metric.sampling_type & (MIN | MAX | HISTOGRAM | IS_RAW_DATA),
            MIN | MAX | HISTOGRAM | IS_RAW_DATA
        );
        assert_eq!(
            metric.values,
            MetricValues::Double(NumericValues {
                min: Some(0.0),
                max: Some(0.0),
                sum: Some(12.5),
                count: Some(17),
                milliseconds: None,
                histogram: None,
            })
        );
        let encoded = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("empty histogram should encode");
        assert!(!encoded.is_empty());
    }

    /// Scenario: Overflow-marked points fail dimension and metric-name validation after label processing.
    /// Guarantees: Both metrics are rejected while their ME cardinality-overflow diagnostics remain observable.
    #[test]
    fn retains_overflow_diagnostics_for_rejected_metrics() {
        let overflow_attribute = bool_attribute("otel.metric.overflow", true);
        let oversized_name = "x".repeat(513);
        let scope = scope_with_metrics(vec![
            gauge_metric(
                "invalid-dimension",
                gauge_point(vec![
                    overflow_attribute.clone(),
                    string_attribute(&"x".repeat(MAX_DIMENSION_NAME_CHARS + 1), "value"),
                ]),
            ),
            gauge_metric(
                oversized_name.clone(),
                gauge_point(vec![overflow_attribute]),
            ),
        ]);

        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 2);
        assert!(mapped.publications.is_empty());
        assert_eq!(mapped.cardinality_overflows.len(), 2);
        assert_eq!(
            mapped.cardinality_overflows[0].metric_name,
            "invalid-dimension"
        );
        assert_eq!(mapped.cardinality_overflows[1].metric_name, oversized_name);
    }

    /// Scenario: Overflow-marked gauges encounter invalid resource dimensions, invalid scope dimensions, and an empty metric name.
    /// Guarantees: Parent and name validation reject each metric after preserving its correctly routed overflow diagnostic.
    #[test]
    fn retains_overflow_diagnostics_across_parent_validation() {
        let overflow_point = || gauge_point(vec![bool_attribute("otel.metric.overflow", true)]);

        let mut resource_config = config();
        resource_config.resource_attributes = vec!["invalid-resource".to_string()];
        let resource_scope =
            scope_with_metrics(vec![gauge_metric("resource-invalid", overflow_point())]);
        let resource_mapped = map_request(
            &request(
                vec![
                    string_attribute(
                        "invalid-resource",
                        &"x".repeat(MAX_DIMENSION_VALUE_CHARS + 1),
                    ),
                    string_attribute(ACCOUNT_ATTRIBUTE, "later-account"),
                ],
                resource_scope,
            ),
            &resource_config,
            TEST_TIME_NANOS,
        )
        .expect("request should map");

        let mut scope_config = config();
        scope_config.scope_attributes = vec![ScopeAttributes {
            name: "meter".to_string(),
            keys: vec!["invalid-scope".to_string()],
        }];
        let scope = ScopeMetrics {
            scope: Some(InstrumentationScope {
                name: "meter".to_string(),
                attributes: vec![string_attribute(
                    "invalid-scope",
                    &"x".repeat(MAX_DIMENSION_VALUE_CHARS + 1),
                )],
                ..Default::default()
            }),
            metrics: vec![gauge_metric("scope-invalid", overflow_point())],
            ..Default::default()
        };
        let scope_mapped = map_request(
            &request(
                vec![string_attribute(NAMESPACE_ATTRIBUTE, "resource-namespace")],
                scope,
            ),
            &scope_config,
            TEST_TIME_NANOS,
        )
        .expect("request should map");

        let empty_name_scope = scope_with_metrics(vec![gauge_metric("", overflow_point())]);
        let empty_name_mapped = map_request(
            &request(Vec::new(), empty_name_scope),
            &config(),
            TEST_TIME_NANOS,
        )
        .expect("request should map");

        assert_eq!(resource_mapped.rejected_data_points, 1);
        assert_eq!(
            resource_mapped.cardinality_overflows[0].monitoring_account,
            "later-account"
        );
        assert_eq!(scope_mapped.rejected_data_points, 1);
        assert_eq!(
            scope_mapped.cardinality_overflows[0].namespace,
            "resource-namespace"
        );
        assert_eq!(empty_name_mapped.rejected_data_points, 1);
        assert_eq!(empty_name_mapped.cardinality_overflows[0].metric_name, "");
    }

    /// Scenario: A stale overflow-marked point is rejected before label processing.
    /// Guarantees: No cardinality-overflow diagnostic is emitted for intrinsically invalid datapoints.
    #[test]
    fn omits_overflow_diagnostic_for_stale_point() {
        let scope = scope_with_metrics(vec![gauge_metric(
            "stale",
            NumberDataPoint {
                flags: 1,
                ..gauge_point(vec![bool_attribute("otel.metric.overflow", true)])
            },
        )]);

        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 1);
        assert!(mapped.cardinality_overflows.is_empty());
    }

    /// Scenario: A minimum-scale exponential histogram exceeds the mapping bucket limit.
    /// Guarantees: Downscaling below the accepted OTLP input scale remains serializable by the Geneva encoder.
    #[test]
    fn encodes_exponential_histogram_after_downscaling_below_input_scale_limit() {
        let bucket_counts = vec![1; 503];
        let point = ExponentialHistogramDataPoint {
            time_unix_nano: TEST_TIME_NANOS,
            count: bucket_counts.len() as u64,
            sum: Some(1.0),
            scale: MIN_EXPONENTIAL_SCALE,
            zero_count: 0,
            positive: Some(exponential_histogram_data_point::Buckets {
                offset: 0,
                bucket_counts,
            }),
            ..Default::default()
        };
        let scope = scope_with_metrics(vec![otlp_metric(
            "downscaled",
            metric::Data::ExponentialHistogram(ExponentialHistogram {
                data_points: vec![point],
                aggregation_temporality: AggregationTemporality::Delta as i32,
            }),
        )]);

        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");
        let metric = &mapped.publications[0].packet.metrics[0];
        let MetricValues::Double(values) = &metric.values else {
            panic!("histogram should use double values");
        };
        let Some(MetricHistogram::Exponential(histogram)) = &values.histogram else {
            panic!("exponential histogram should be present");
        };

        assert_eq!(histogram.scale, -12);
        let _ = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("downscaled histogram should encode");
    }

    /// Scenario: A point contains one oversized exemplar and enough valid exemplars to exceed 512 bytes.
    /// Guarantees: Oversized exemplars are discarded and sampling retains the maximum fitting valid payload.
    #[test]
    fn bounds_exemplars_without_rejecting_the_metric() {
        let minimal_exemplar = OtlpExemplar {
            filtered_attributes: Vec::new(),
            time_unix_nano: TEST_TIME_NANOS,
            span_id: Vec::new(),
            trace_id: Vec::new(),
            value: Some(exemplar::Value::AsInt(1)),
        };
        let oversized_exemplar = OtlpExemplar {
            filtered_attributes: vec![string_attribute(&"k".repeat(190), "value")],
            ..minimal_exemplar.clone()
        };
        let mut exemplars = vec![oversized_exemplar];
        exemplars.extend(vec![minimal_exemplar; 45]);
        let scope = ScopeMetrics {
            scope: None,
            metrics: vec![OtlpMetric {
                name: "with-exemplars".to_string(),
                description: String::new(),
                unit: String::new(),
                metadata: Vec::new(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![NumberDataPoint {
                        attributes: Vec::new(),
                        start_time_unix_nano: 0,
                        time_unix_nano: TEST_TIME_NANOS,
                        exemplars,
                        flags: 0,
                        value: Some(number_data_point::Value::AsDouble(1.0)),
                    }],
                })),
            }],
            schema_url: String::new(),
        };
        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");
        let metric = &mapped.publications[0].packet.metrics[0];

        assert_eq!(mapped.rejected_data_points, 0);
        let exemplar_size =
            super::super::encoder::exemplar::encoded_exemplar_size(&metric.exemplars[0])
                .expect("retained exemplar should be valid");
        assert_eq!(
            metric.exemplars.len(),
            super::super::encoder::exemplar::MAX_EXEMPLAR_PAYLOAD_SIZE / exemplar_size
        );
        assert_ne!(metric.sampling_type & EXEMPLAR, 0);
        let _ = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("bounded exemplars should encode");
    }

    /// Scenario: Exemplar mapping is disabled while an OTLP point carries a valid exemplar.
    /// Guarantees: The exemplar is omitted and the Geneva exemplar sampling flag remains unset.
    #[test]
    fn omits_exemplars_when_disabled() {
        let exemplar = OtlpExemplar {
            filtered_attributes: Vec::new(),
            time_unix_nano: TEST_TIME_NANOS,
            span_id: Vec::new(),
            trace_id: Vec::new(),
            value: Some(exemplar::Value::AsInt(1)),
        };
        let mut mapping_config = config();
        mapping_config.disable_exemplars = true;
        let mut sampling_type = SUM | COUNT;

        let exemplars = map_point_exemplars(
            &mapping_config,
            &[exemplar],
            TEST_TIME_NANOS,
            &mut sampling_type,
        );

        assert!(exemplars.is_empty());
        assert_eq!(sampling_type & EXEMPLAR, 0);
    }

    /// Scenario: A request contains stale, over-limit, and unsupported-temporality data points.
    /// Guarantees: Invalid data points are counted and omitted without producing malformed metrics.
    #[test]
    fn rejects_invalid_points_individually() {
        let stale_point = NumberDataPoint {
            attributes: Vec::new(),
            start_time_unix_nano: 0,
            time_unix_nano: TEST_TIME_NANOS,
            exemplars: Vec::new(),
            flags: 1,
            value: Some(number_data_point::Value::AsDouble(1.0)),
        };
        let over_limit_point = NumberDataPoint {
            attributes: vec![string_attribute(
                &"x".repeat(MAX_DIMENSION_NAME_CHARS + 1),
                "value",
            )],
            start_time_unix_nano: 0,
            time_unix_nano: TEST_TIME_NANOS,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(number_data_point::Value::AsDouble(1.0)),
        };
        let scope = ScopeMetrics {
            scope: None,
            metrics: vec![
                OtlpMetric {
                    name: "stale".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![stale_point],
                    })),
                },
                OtlpMetric {
                    name: "over-limit".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![over_limit_point],
                    })),
                },
                OtlpMetric {
                    name: "invalid-sum".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Sum(Sum {
                        data_points: vec![NumberDataPoint {
                            attributes: Vec::new(),
                            start_time_unix_nano: 0,
                            time_unix_nano: TEST_TIME_NANOS,
                            exemplars: Vec::new(),
                            flags: 0,
                            value: Some(number_data_point::Value::AsDouble(2.0)),
                        }],
                        aggregation_temporality: AggregationTemporality::Delta as i32,
                        is_monotonic: false,
                    })),
                },
            ],
            schema_url: String::new(),
        };
        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 3);
        assert!(mapped.publications.is_empty());
    }

    /// Scenario: A serialized OTLP gauge request is decoded, mapped, and serialized as protocol v6.
    /// Guarantees: The complete local OTLP-to-ME path produces a non-empty packet with one metric.
    #[test]
    fn decodes_and_encodes_otlp_request() {
        let scope = ScopeMetrics {
            scope: None,
            metrics: vec![OtlpMetric {
                name: "temperature".to_string(),
                description: String::new(),
                unit: String::new(),
                metadata: Vec::new(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![NumberDataPoint {
                        attributes: vec![string_attribute("region", "eastus")],
                        start_time_unix_nano: 0,
                        time_unix_nano: TEST_TIME_NANOS,
                        exemplars: Vec::new(),
                        flags: 0,
                        value: Some(number_data_point::Value::AsDouble(12.5)),
                    }],
                })),
            }],
            schema_url: String::new(),
        };
        let mut bytes = Vec::new();
        request(Vec::new(), scope)
            .encode(&mut bytes)
            .expect("request should encode");

        let mapped =
            decode_and_map(&bytes, &config(), TEST_TIME_NANOS).expect("request should map");
        let encoded = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("packet should encode");

        assert_eq!(mapped.publications[0].packet.metrics.len(), 1);
        assert!(!encoded.is_empty());
    }
}
