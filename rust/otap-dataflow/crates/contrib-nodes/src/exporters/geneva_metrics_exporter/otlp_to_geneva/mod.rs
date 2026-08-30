// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP metrics to ME-to-FE protocol model mapping.

mod attributes;
mod config;
mod histograms;

use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    AggregationTemporality, Exemplar as OtlpExemplar, ExponentialHistogramDataPoint,
    HistogramDataPoint, NumberDataPoint, ScopeMetrics, exemplar, metric, number_data_point,
};
use prost::Message;
use thiserror::Error;

use super::encoder::{
    COUNT, EXEMPLAR, HISTOGRAM, MAX, METRIC_ORIGIN_OPEN_TELEMETRY, METRIC_TYPE_CUMULATIVE_COUNTER,
    METRIC_TYPE_CUMULATIVE_EXPONENTIAL_HISTOGRAM, METRIC_TYPE_CUMULATIVE_HISTOGRAM,
    METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER, METRIC_TYPE_DELTA_COUNTER,
    METRIC_TYPE_DELTA_EXPONENTIAL_HISTOGRAM, METRIC_TYPE_DELTA_HISTOGRAM, METRIC_TYPE_GAUGE, MIN,
    Metric, MetricExemplar, MetricHistogram, MetricValues, NumericValues, Packet, SUM,
};
#[cfg(test)]
use attributes::{ACCOUNT_ATTRIBUTE, MAX_DIMENSION_NAME_CHARS};
use attributes::{
    NAMESPACE_ATTRIBUTE, attribute_string, point_context, resource_context,
    selected_scope_dimensions, string_value,
};
use histograms::{
    MAX_EXPONENTIAL_SCALE, MIN_EXPONENTIAL_SCALE, bucket_sum, downscale_if_required,
    explicit_histogram, sparse_buckets, valid_explicit_histogram,
};

pub use config::{Config, ScopeAttributes};

const DOTNET_UNIX_EPOCH_OFFSET_SECONDS: u64 = 62_135_596_800;
const NANOS_PER_SECOND: u64 = 1_000_000_000;
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
    dimensions_valid: bool,
}

#[derive(Clone, Debug)]
struct PointContext {
    monitoring_account: String,
    namespace: String,
    dimensions: Vec<super::encoder::Dimension>,
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
    let current_time_bucket = unix_nanos_to_dotnet_seconds(receive_time_unix_nano)
        .ok_or(MappingError::ReceiveTimeOverflow)?;
    let mut outcome = MappingOutcome {
        publications: Vec::new(),
        rejected_data_points: 0,
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
    if !resource.dimensions_valid {
        outcome.rejected_data_points += scope_metrics
            .metrics
            .iter()
            .filter_map(|metric| metric.data.as_ref())
            .map(data_point_count)
            .sum::<usize>();
        return;
    }
    let scope_namespace = scope_metrics
        .scope
        .as_ref()
        .filter(|scope| !scope.name.is_empty())
        .and_then(|scope| attribute_string(&scope.attributes, NAMESPACE_ATTRIBUTE))
        .unwrap_or_else(|| resource.namespace.clone());
    let Some(scope_dimensions) = selected_scope_dimensions(scope_metrics, config) else {
        outcome.rejected_data_points += scope_metrics
            .metrics
            .iter()
            .filter_map(|metric| metric.data.as_ref())
            .map(data_point_count)
            .sum::<usize>();
        return;
    };

    for otlp_metric in &scope_metrics.metrics {
        let Some(data) = &otlp_metric.data else {
            outcome.rejected_data_points += 1;
            continue;
        };
        if otlp_metric.name.is_empty() {
            outcome.rejected_data_points += data_point_count(data).max(1);
            continue;
        }

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
) -> Option<(String, Metric)> {
    if is_stale(point.flags) {
        return None;
    }
    let context = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    )?;
    let mut sampling_type = SUM | COUNT | metric_type | METRIC_ORIGIN_OPEN_TELEMETRY;
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

    Some((
        context.monitoring_account,
        Metric {
            time_bucket: unix_nanos_to_dotnet_seconds(point.time_unix_nano)? as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values,
            exemplars,
        },
    ))
}

fn map_histogram_point(
    name: &str,
    point: &HistogramDataPoint,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[super::encoder::Dimension],
    config: &Config,
    metric_type: u32,
) -> Option<(String, Metric)> {
    if is_stale(point.flags) || !valid_explicit_histogram(point) {
        return None;
    }
    let context = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    )?;
    let histogram = explicit_histogram(point);
    let mut sampling_type = SUM | COUNT | metric_type | METRIC_ORIGIN_OPEN_TELEMETRY;
    if point.min.is_some() {
        sampling_type |= MIN;
    }
    if point.max.is_some() {
        sampling_type |= MAX;
    }
    if histogram.is_some() {
        sampling_type |= HISTOGRAM;
    }
    let exemplars = map_point_exemplars(
        config,
        &point.exemplars,
        point.time_unix_nano,
        &mut sampling_type,
    );

    Some((
        context.monitoring_account,
        Metric {
            time_bucket: unix_nanos_to_dotnet_seconds(point.time_unix_nano)? as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values: MetricValues::Double(NumericValues {
                min: point.min,
                max: point.max,
                sum: Some(point.sum.unwrap_or(0.0)),
                count: Some(point.count),
                milliseconds: None,
                histogram,
            }),
            exemplars,
        },
    ))
}

fn map_exponential_histogram_point(
    name: &str,
    point: &ExponentialHistogramDataPoint,
    resource: &ResourceContext,
    scope_namespace: &str,
    scope_dimensions: &[super::encoder::Dimension],
    config: &Config,
    metric_type: u32,
) -> Option<(String, Metric)> {
    if is_stale(point.flags)
        || !(MIN_EXPONENTIAL_SCALE..=MAX_EXPONENTIAL_SCALE).contains(&point.scale)
    {
        return None;
    }
    let mut positive = sparse_buckets(point.positive.as_ref())?;
    let mut negative = sparse_buckets(point.negative.as_ref())?;
    let bucket_total = point
        .zero_count
        .checked_add(bucket_sum(&positive)?)
        .and_then(|total| total.checked_add(bucket_sum(&negative)?))?;
    if bucket_total != point.count {
        return None;
    }
    let mut scale = point.scale;
    downscale_if_required(&mut scale, &mut positive, &mut negative);
    let scale = i8::try_from(scale).ok()?;
    let context = point_context(
        &point.attributes,
        resource,
        scope_namespace,
        scope_dimensions,
        config,
    )?;
    let mut sampling_type = SUM | COUNT | HISTOGRAM | metric_type | METRIC_ORIGIN_OPEN_TELEMETRY;
    if point.min.is_some() {
        sampling_type |= MIN;
    }
    if point.max.is_some() {
        sampling_type |= MAX;
    }
    let exemplars = map_point_exemplars(
        config,
        &point.exemplars,
        point.time_unix_nano,
        &mut sampling_type,
    );

    Some((
        context.monitoring_account,
        Metric {
            time_bucket: unix_nanos_to_dotnet_seconds(point.time_unix_nano)? as i64,
            namespace: context.namespace,
            name: name.to_string(),
            dimensions: context.dimensions,
            sampling_type,
            values: MetricValues::Double(NumericValues {
                min: point.min,
                max: point.max,
                sum: Some(point.sum.unwrap_or(0.0)),
                count: Some(point.count),
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
    ))
}

fn record_mapped_metric(
    outcome: &mut MappingOutcome,
    current_time_bucket: u64,
    mapped_metric: Option<(String, Metric)>,
) {
    let Some((monitoring_account, metric)) = mapped_metric else {
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
    let temporality = AggregationTemporality::try_from(temporality).ok()?;
    match (is_monotonic, temporality) {
        (true, AggregationTemporality::Delta) => Some(METRIC_TYPE_DELTA_COUNTER),
        (true, AggregationTemporality::Cumulative | AggregationTemporality::Unspecified) => {
            Some(METRIC_TYPE_CUMULATIVE_COUNTER)
        }
        (false, AggregationTemporality::Delta) => None,
        (false, AggregationTemporality::Cumulative | AggregationTemporality::Unspecified) => {
            Some(METRIC_TYPE_CUMULATIVE_UP_DOWN_COUNTER)
        }
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
    super::encoder::retain_exemplars_within_limits(&mut mapped);
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

fn unix_nanos_to_dotnet_seconds(value: u64) -> Option<u64> {
    DOTNET_UNIX_EPOCH_OFFSET_SECONDS.checked_add(value / NANOS_PER_SECOND)
}

fn is_stale(flags: u32) -> bool {
    flags & 1 != 0
}

fn data_point_count(data: &metric::Data) -> usize {
    match data {
        metric::Data::Gauge(value) => value.data_points.len(),
        metric::Data::Sum(value) => value.data_points.len(),
        metric::Data::Histogram(value) => value.data_points.len(),
        metric::Data::ExponentialHistogram(value) => value.data_points.len(),
        metric::Data::Summary(value) => value.data_points.len(),
    }
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

    /// Scenario: A delta OTLP integer sum contains routing attributes and the SDK cardinality-overflow marker.
    /// Guarantees: Point routing wins, OTLP dimensions including the overflow marker are preserved and sorted, and the ME counter flags are selected.
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
            SUM | COUNT | METRIC_TYPE_DELTA_COUNTER | METRIC_ORIGIN_OPEN_TELEMETRY
        );
        assert_eq!(
            metric.dimensions,
            vec![
                super::super::encoder::Dimension {
                    name: "a-dimension".to_string(),
                    value: "first".to_string(),
                },
                super::super::encoder::Dimension {
                    name: "otel.metric.overflow".to_string(),
                    value: "True".to_string(),
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
                min: None,
                max: None,
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
    }

    /// Scenario: Valid gauge data is mixed with explicit and exponential histogram counts above ME's limit.
    /// Guarantees: Invalid points are counted and omitted while the valid metric remains encodable.
    #[test]
    fn rejects_unencodable_points_without_dropping_valid_metrics() {
        let count = u64::from(u32::MAX) + 1;
        let scope = ScopeMetrics {
            scope: None,
            metrics: vec![
                OtlpMetric {
                    name: "valid".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            attributes: Vec::new(),
                            start_time_unix_nano: 0,
                            time_unix_nano: TEST_TIME_NANOS,
                            exemplars: Vec::new(),
                            flags: 0,
                            value: Some(number_data_point::Value::AsDouble(1.0)),
                        }],
                    })),
                },
                OtlpMetric {
                    name: "explicit".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            attributes: Vec::new(),
                            start_time_unix_nano: 0,
                            time_unix_nano: TEST_TIME_NANOS,
                            count,
                            sum: Some(1.0),
                            bucket_counts: vec![count],
                            explicit_bounds: Vec::new(),
                            exemplars: Vec::new(),
                            flags: 0,
                            min: None,
                            max: None,
                        }],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    })),
                },
                OtlpMetric {
                    name: "exponential".to_string(),
                    description: String::new(),
                    unit: String::new(),
                    metadata: Vec::new(),
                    data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                        data_points: vec![ExponentialHistogramDataPoint {
                            attributes: Vec::new(),
                            start_time_unix_nano: 0,
                            time_unix_nano: TEST_TIME_NANOS,
                            count,
                            sum: Some(1.0),
                            scale: 0,
                            zero_count: count,
                            positive: None,
                            negative: None,
                            flags: 0,
                            exemplars: Vec::new(),
                            min: None,
                            max: None,
                            zero_threshold: 0.0,
                        }],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    })),
                },
            ],
            schema_url: String::new(),
        };
        let mapped = map_request(&request(Vec::new(), scope), &config(), TEST_TIME_NANOS)
            .expect("request should map");

        assert_eq!(mapped.rejected_data_points, 2);
        assert_eq!(mapped.publications.len(), 1);
        assert_eq!(mapped.publications[0].packet.metrics.len(), 1);
        assert_eq!(mapped.publications[0].packet.metrics[0].name, "valid");
        let _ = super::super::encoder::encode(&mapped.publications[0].packet)
            .expect("remaining valid metric should encode");
    }

    /// Scenario: A point contains one oversized exemplar and enough valid exemplars to exceed 512 bytes.
    /// Guarantees: Oversized and excess exemplars are discarded while the metric and bounded exemplar list encode.
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
        assert_eq!(metric.exemplars.len(), 39);
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
