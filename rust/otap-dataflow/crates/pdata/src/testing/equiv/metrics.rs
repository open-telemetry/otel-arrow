// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric equivalence checking.

use crate::proto::opentelemetry::metrics::v1::{
    ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram, HistogramDataPoint,
    Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics, Sum, Summary,
    SummaryDataPoint, metric,
};
use crate::testing::equiv::canonical::{
    assert_equivalent, canonicalize_any_value, canonicalize_vec,
};

/// Split a MetricsData into individual singleton MetricsData messages (one per data point).
fn metrics_split_into_singletons(metrics_data: &MetricsData) -> Vec<MetricsData> {
    let mut result = Vec::new();

    for resource_metrics in &metrics_data.resource_metrics {
        for scope_metrics in &resource_metrics.scope_metrics {
            for metric in &scope_metrics.metrics {
                // Handle the metric data oneof - each type has data_points
                let data_point_singletons = match &metric.data {
                    Some(metric::Data::Gauge(gauge)) => {
                        create_gauge_singletons(metric, gauge, &gauge.data_points)
                    }
                    Some(metric::Data::Sum(sum)) => {
                        create_sum_singletons(metric, sum, &sum.data_points)
                    }
                    Some(metric::Data::Histogram(histogram)) => {
                        create_histogram_singletons(metric, histogram, &histogram.data_points)
                    }
                    Some(metric::Data::ExponentialHistogram(exp_histogram)) => {
                        create_exp_histogram_singletons(
                            metric,
                            exp_histogram,
                            &exp_histogram.data_points,
                        )
                    }
                    Some(metric::Data::Summary(summary)) => {
                        create_summary_singletons(metric, summary, &summary.data_points)
                    }
                    None => vec![],
                };

                // Wrap each metric singleton in the full hierarchy
                for metric_singleton in data_point_singletons {
                    result.push(MetricsData {
                        resource_metrics: vec![ResourceMetrics {
                            resource: resource_metrics.resource.clone(),
                            scope_metrics: vec![ScopeMetrics {
                                scope: scope_metrics.scope.clone(),
                                metrics: vec![metric_singleton],
                                schema_url: scope_metrics.schema_url.clone(),
                            }],
                            schema_url: resource_metrics.schema_url.clone(),
                        }],
                    });
                }
            }
        }
    }

    result
}

// Helper functions to create singleton metrics for each data type
fn create_gauge_singletons(
    metric: &Metric,
    _gauge: &Gauge,
    data_points: &[NumberDataPoint],
) -> Vec<Metric> {
    data_points
        .iter()
        .map(|dp| Metric {
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            metadata: metric.metadata.clone(),
            data: Some(metric::Data::Gauge(Gauge {
                data_points: vec![dp.clone()],
            })),
        })
        .collect()
}

fn create_sum_singletons(
    metric: &Metric,
    sum: &Sum,
    data_points: &[NumberDataPoint],
) -> Vec<Metric> {
    data_points
        .iter()
        .map(|dp| Metric {
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            metadata: metric.metadata.clone(),
            data: Some(metric::Data::Sum(Sum {
                data_points: vec![dp.clone()],
                aggregation_temporality: sum.aggregation_temporality,
                is_monotonic: sum.is_monotonic,
            })),
        })
        .collect()
}

fn create_histogram_singletons(
    metric: &Metric,
    histogram: &Histogram,
    data_points: &[HistogramDataPoint],
) -> Vec<Metric> {
    data_points
        .iter()
        .map(|dp| Metric {
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            metadata: metric.metadata.clone(),
            data: Some(metric::Data::Histogram(Histogram {
                data_points: vec![dp.clone()],
                aggregation_temporality: histogram.aggregation_temporality,
            })),
        })
        .collect()
}

fn create_exp_histogram_singletons(
    metric: &Metric,
    exp_histogram: &ExponentialHistogram,
    data_points: &[ExponentialHistogramDataPoint],
) -> Vec<Metric> {
    data_points
        .iter()
        .map(|dp| Metric {
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            metadata: metric.metadata.clone(),
            data: Some(metric::Data::ExponentialHistogram(ExponentialHistogram {
                data_points: vec![dp.clone()],
                aggregation_temporality: exp_histogram.aggregation_temporality,
            })),
        })
        .collect()
}

fn create_summary_singletons(
    metric: &Metric,
    _summary: &Summary,
    data_points: &[SummaryDataPoint],
) -> Vec<Metric> {
    data_points
        .iter()
        .map(|dp| Metric {
            name: metric.name.clone(),
            description: metric.description.clone(),
            unit: metric.unit.clone(),
            metadata: metric.metadata.clone(),
            data: Some(metric::Data::Summary(Summary {
                data_points: vec![dp.clone()],
            })),
        })
        .collect()
}

/// Canonicalize a singleton MetricsData by sorting all fields that need canonical ordering.
fn metrics_canonicalize_singleton(metrics_data: &mut MetricsData) {
    // Canonicalize resource attributes
    for resource_metrics in &mut metrics_data.resource_metrics {
        if let Some(resource) = &mut resource_metrics.resource {
            canonicalize_vec(&mut resource.attributes, |attr| {
                if let Some(value) = &mut attr.value {
                    canonicalize_any_value(value);
                }
            });
        }

        // Canonicalize scope attributes
        for scope_metrics in &mut resource_metrics.scope_metrics {
            if let Some(scope) = &mut scope_metrics.scope {
                canonicalize_vec(&mut scope.attributes, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });
            }

            // Canonicalize metric fields
            for metric in &mut scope_metrics.metrics {
                // Canonicalize metric metadata
                canonicalize_vec(&mut metric.metadata, |attr| {
                    if let Some(value) = &mut attr.value {
                        canonicalize_any_value(value);
                    }
                });

                // Canonicalize data points based on metric type
                if let Some(data) = &mut metric.data {
                    match data {
                        metric::Data::Gauge(gauge) => {
                            for dp in &mut gauge.data_points {
                                canonicalize_number_data_point(dp);
                            }
                        }
                        metric::Data::Sum(sum) => {
                            for dp in &mut sum.data_points {
                                canonicalize_number_data_point(dp);
                            }
                        }
                        metric::Data::Histogram(histogram) => {
                            for dp in &mut histogram.data_points {
                                canonicalize_histogram_data_point(dp);
                            }
                        }
                        metric::Data::ExponentialHistogram(exp_histogram) => {
                            for dp in &mut exp_histogram.data_points {
                                canonicalize_exp_histogram_data_point(dp);
                            }
                        }
                        metric::Data::Summary(summary) => {
                            for dp in &mut summary.data_points {
                                canonicalize_summary_data_point(dp);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn canonicalize_number_data_point(dp: &mut NumberDataPoint) {
    canonicalize_vec(&mut dp.attributes, |attr| {
        if let Some(value) = &mut attr.value {
            canonicalize_any_value(value);
        }
    });
    canonicalize_vec(&mut dp.exemplars, |exemplar| {
        canonicalize_vec(&mut exemplar.filtered_attributes, |attr| {
            if let Some(value) = &mut attr.value {
                canonicalize_any_value(value);
            }
        });
    });
}

fn canonicalize_histogram_data_point(dp: &mut HistogramDataPoint) {
    canonicalize_vec(&mut dp.attributes, |attr| {
        if let Some(value) = &mut attr.value {
            canonicalize_any_value(value);
        }
    });
    canonicalize_vec(&mut dp.exemplars, |exemplar| {
        canonicalize_vec(&mut exemplar.filtered_attributes, |attr| {
            if let Some(value) = &mut attr.value {
                canonicalize_any_value(value);
            }
        });
    });
}

fn canonicalize_exp_histogram_data_point(dp: &mut ExponentialHistogramDataPoint) {
    canonicalize_vec(&mut dp.attributes, |attr| {
        if let Some(value) = &mut attr.value {
            canonicalize_any_value(value);
        }
    });
    canonicalize_vec(&mut dp.exemplars, |exemplar| {
        canonicalize_vec(&mut exemplar.filtered_attributes, |attr| {
            if let Some(value) = &mut attr.value {
                canonicalize_any_value(value);
            }
        });
    });
}

fn canonicalize_summary_data_point(dp: &mut SummaryDataPoint) {
    canonicalize_vec(&mut dp.attributes, |attr| {
        if let Some(value) = &mut attr.value {
            canonicalize_any_value(value);
        }
    });
    // Note: quantile_values are user data, but we canonicalize them for consistency
    canonicalize_vec(&mut dp.quantile_values, |_| {
        // ValueAtQuantile has no nested structures to canonicalize
    });
}

/// Assert that two collections of `MetricsData` instances are equivalent.
pub fn assert_metrics_equivalent(left: &[MetricsData], right: &[MetricsData]) {
    assert_equivalent(
        left,
        right,
        metrics_split_into_singletons,
        metrics_canonicalize_singleton,
        "MetricsData",
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::resource::v1::Resource;

    #[test]
    fn test_metrics_equivalent_with_unordered_attributes() {
        // Test that metrics with attributes in different orders are considered equivalent
        let request1 = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("service", AnyValue::new_string("test")),
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        attributes: vec![
                            KeyValue::new("scope.key1", AnyValue::new_string("value1")),
                            KeyValue::new("scope.key2", AnyValue::new_int(42)),
                        ],
                        ..Default::default()
                    }),
                    metrics: vec![Metric {
                        name: "test-metric".into(),
                        description: "A test metric".into(),
                        unit: "ms".into(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    KeyValue::new("key1", AnyValue::new_string("value1")),
                                    KeyValue::new("key2", AnyValue::new_int(100)),
                                ],
                                time_unix_nano: 1000,
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        // Same data but with all attributes in different order
        let request2 = MetricsData {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    attributes: vec![
                        KeyValue::new("version", AnyValue::new_string("1.0")),
                        KeyValue::new("service", AnyValue::new_string("test")),
                    ],
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    scope: Some(InstrumentationScope {
                        name: "scope1".into(),
                        attributes: vec![
                            KeyValue::new("scope.key2", AnyValue::new_int(42)),
                            KeyValue::new("scope.key1", AnyValue::new_string("value1")),
                        ],
                        ..Default::default()
                    }),
                    metrics: vec![Metric {
                        name: "test-metric".into(),
                        description: "A test metric".into(),
                        unit: "ms".into(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                attributes: vec![
                                    KeyValue::new("key2", AnyValue::new_int(100)),
                                    KeyValue::new("key1", AnyValue::new_string("value1")),
                                ],
                                time_unix_nano: 1000,
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        assert_metrics_equivalent(&[request1], &[request2]);
    }
}
