// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for pipelines applied to metrics data points

use otel_arrow_contrib_data_engine_kql_parser::Parser;
use otel_arrow_dfe_pdata::{
    proto::{
        OtlpProtoMessage,
        opentelemetry::{
            arrow::v1::ArrowPayloadType,
            common::v1::{AnyValue, KeyValue},
            metrics::v1::{
                Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
                HistogramDataPoint, Metric, MetricsData, NumberDataPoint, Sum, Summary,
                SummaryDataPoint, exponential_histogram_data_point::Buckets, metric::Data,
            },
        },
    },
    schema::consts,
    testing::round_trip::{otap_to_otlp, otlp_to_otap, to_metrics_data},
};
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

use crate::{parser::default_parser_options, pipeline::Pipeline};

/// Helper function to compare the metrics & their data points on left right data.
///
/// This is used for convenience because, during construction for test data we may not wish to
/// create empty placeholder data such as resources/scopes that get set when we roundtrip from
/// OTAP, and comparing these placeholder is not relevant anyway to what this module is testing
fn assert_metrics_eq(left: MetricsData, right: MetricsData) {
    assert_eq!(left.resource_metrics.len(), right.resource_metrics.len());

    for rm_idx in 0..left.resource_metrics.len() {
        let left_rm = &left.resource_metrics[rm_idx];
        let right_rm = &right.resource_metrics[rm_idx];
        assert_eq!(left_rm.scope_metrics.len(), right_rm.scope_metrics.len());
        for sm_idx in 0..left_rm.scope_metrics.len() {
            let left_sm = &left_rm.scope_metrics[sm_idx];
            let right_sm = &right_rm.scope_metrics[sm_idx];
            pretty_assertions::assert_eq!(left_sm.metrics, right_sm.metrics)
        }
    }
}

/// Scenario: invoke a simple filter on all metric data points where predicate involves only
/// a column on the metrics data point record batch.
/// Guarantees: All types of metric data points, for all metrics, are filtered by the predicate.
/// Attributes, exemplars and exemplar attributes are preserved only for kept rows.
#[tokio::test]
async fn test_simple_data_point_filter() {
    let query = "metrics | apply data_points {
        where flags > 5
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    // these are not actually valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .flags(5u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(1u64)
                                .filtered_attributes(vec![
                                    KeyValue::new("ea1", AnyValue::new_string("b")),
                                    KeyValue::new("ea2", AnyValue::new_string("b")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(4))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(2u64)
                                .filtered_attributes(vec![
                                    KeyValue::new("ea1", AnyValue::new_string("b")),
                                    KeyValue::new("ea2", AnyValue::new_string("b")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(5)),
                            KeyValue::new("z", AnyValue::new_int(5)),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("sum_metric")
            .data_sum(Sum {
                aggregation_temporality: 2,
                is_monotonic: false,
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(2u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(5)),
                            KeyValue::new("z", AnyValue::new_int(5)),
                        ])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(7u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(5)),
                            KeyValue::new("z", AnyValue::new_int(5)),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("histogram_metric")
            .data_histogram(Histogram {
                aggregation_temporality: 0,
                data_points: vec![
                    HistogramDataPoint::build()
                        .flags(7u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_int(4))])
                        .finish(),
                    HistogramDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_int(9))])
                        .flags(3u32)
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(2u64)
                                .filtered_attributes(vec![KeyValue::new("b", AnyValue::new_int(4))])
                                .finish(),
                        ])
                        .finish(),
                    HistogramDataPoint::build()
                        .flags(8u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_int(1))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(8u64)
                                .filtered_attributes(vec![KeyValue::new("b", AnyValue::new_int(8))])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("expr_hist_metric")
            .data_exponential_histogram(ExponentialHistogram {
                aggregation_temporality: 5,
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .flags(2u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("adsf"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(8u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "c",
                                    AnyValue::new_string("d"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                    ExponentialHistogramDataPoint::build()
                        .flags(9u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("qwer"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(9u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "c",
                                    AnyValue::new_string("e"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("summary_metric")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .flags(1u32)
                        .attributes(vec![
                            KeyValue::new("a", AnyValue::new_string("x")),
                            KeyValue::new("b", AnyValue::new_string("y")),
                        ])
                        .finish(),
                    SummaryDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![
                            KeyValue::new("a", AnyValue::new_string("x")),
                            KeyValue::new("b", AnyValue::new_string("y")),
                            KeyValue::new("c", AnyValue::new_string("z")),
                        ])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
    let result = pipeline.execute(input_batch).await.unwrap();

    // check that we've removed extraneous rows from the child record batches
    let number_dp_attrs_rb = result.get(ArrowPayloadType::NumberDpAttrs).unwrap();
    assert_eq!(number_dp_attrs_rb.num_rows(), 5);
    let number_exemplars_rb = result.get(ArrowPayloadType::NumberDpExemplars).unwrap();
    assert_eq!(number_exemplars_rb.num_rows(), 1);
    let number_exemplars_dp_attrs = result.get(ArrowPayloadType::NumberDpExemplarAttrs).unwrap();
    assert_eq!(number_exemplars_dp_attrs.num_rows(), 2);

    let hist_dp_attrs_rb = result.get(ArrowPayloadType::HistogramDpAttrs).unwrap();
    assert_eq!(hist_dp_attrs_rb.num_rows(), 2);
    let hist_dp_exemplars_rb = result.get(ArrowPayloadType::HistogramDpExemplars).unwrap();
    assert_eq!(hist_dp_exemplars_rb.num_rows(), 1);
    let hist_dp_exemplar_attrs = result
        .get(ArrowPayloadType::HistogramDpExemplarAttrs)
        .unwrap();
    assert_eq!(hist_dp_exemplar_attrs.num_rows(), 1);

    let exp_hist_dp_attrs_rb = result.get(ArrowPayloadType::ExpHistogramDpAttrs).unwrap();
    assert_eq!(exp_hist_dp_attrs_rb.num_rows(), 1);
    let exp_hist_dp_exemplars_rb = result
        .get(ArrowPayloadType::ExpHistogramDpExemplars)
        .unwrap();
    assert_eq!(exp_hist_dp_exemplars_rb.num_rows(), 1);
    let exp_hist_dp_exemplar_attrs = result
        .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
        .unwrap();
    assert_eq!(exp_hist_dp_exemplar_attrs.num_rows(), 1);

    let summary_dp_attrs = result.get(ArrowPayloadType::SummaryDpAttrs).unwrap();
    assert_eq!(summary_dp_attrs.num_rows(), 3);

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    // assert on data contained in result
    let expected = to_metrics_data(vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(4))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(2u64)
                                .filtered_attributes(vec![
                                    KeyValue::new("ea1", AnyValue::new_string("b")),
                                    KeyValue::new("ea2", AnyValue::new_string("b")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(5)),
                            KeyValue::new("z", AnyValue::new_int(5)),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("sum_metric")
            .data_sum(Sum {
                aggregation_temporality: 2,
                is_monotonic: false,
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(7u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(5)),
                            KeyValue::new("z", AnyValue::new_int(5)),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("histogram_metric")
            .data_histogram(Histogram {
                aggregation_temporality: 0,
                data_points: vec![
                    HistogramDataPoint::build()
                        .flags(7u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_int(4))])
                        .finish(),
                    HistogramDataPoint::build()
                        .flags(8u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_int(1))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(8u64)
                                .filtered_attributes(vec![KeyValue::new("b", AnyValue::new_int(8))])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("expr_hist_metric")
            .data_exponential_histogram(ExponentialHistogram {
                aggregation_temporality: 0,
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .flags(9u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("qwer"))])
                        .positive(Buckets::default())
                        .negative(Buckets::default())
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(9u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "c",
                                    AnyValue::new_string("e"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("summary_metric")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![
                            KeyValue::new("a", AnyValue::new_string("x")),
                            KeyValue::new("b", AnyValue::new_string("y")),
                            KeyValue::new("c", AnyValue::new_string("z")),
                        ])
                        .finish(),
                ],
            })
            .finish(),
    ]);

    assert_metrics_eq(result_metrics, expected)
}

/// Scenario: filter metrics data points where the predicate evals to a scalar value of "true"
/// Guarantees: the filter pipeline stage properly handles this by keeping all metric data points
#[tokio::test]
async fn test_filter_data_points_by_scalar_true() {
    let query = "metrics | apply data_points {
        where contains(\"foo\", \"f\") // evaluates to scalar true
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    // flags are not valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .flags(5u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("histogram_metric")
            .data_histogram(Histogram {
                aggregation_temporality: 0,
                data_points: vec![
                    HistogramDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("exp_histogram_metric")
            .data_exponential_histogram(ExponentialHistogram {
                aggregation_temporality: 0,
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .positive(Buckets::default())
                        .negative(Buckets::default())
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("summary_metric")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    assert_metrics_eq(result_metrics, to_metrics_data(metrics.clone()));
}

/// Scenario: filter metrics data points where the predicate evals to a scalar value of "false"
/// Guarantees: the filter pipeline stage properly handles this by keeping all metric data points
#[tokio::test]
async fn test_filter_data_points_by_scalar_false() {
    let query = "metrics | apply data_points {
        where contains(\"foo\", \"b\") // evaluates to scalar false
    }";

    run_all_data_points_dropped_test(query).await;
}

/// Scenario: the "drop" operator call is used in a nested pipeline of metric data points
/// Guarantees: this is supported and the result is that all the metric data points are dropped
#[tokio::test]
async fn test_drop_all_metric_data_points() {
    let query = "metrics | apply data_points { drop }";

    run_all_data_points_dropped_test(query).await;
}

/// helper which runs a test that evaluates the given query on a batch that contains data points
/// for all types of metrics and child record batches for each type of data point (like exemplars,
/// attributes, and exemplar attributes) and ensures that after execution all the data points have
/// been dropped.
async fn run_all_data_points_dropped_test(query: &'static str) {
    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);
    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    // flags are not valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .flags(5u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("histogram_metric")
            .data_histogram(Histogram {
                aggregation_temporality: 0,
                data_points: vec![
                    HistogramDataPoint::build()
                        .flags(4u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("exp_histogram_metric")
            .data_exponential_histogram(ExponentialHistogram {
                aggregation_temporality: 0,
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .positive(Buckets::default())
                        .negative(Buckets::default())
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("summary_metric")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .flags(4u32)
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));
    let result = pipeline.execute(input_batch).await.unwrap();

    // assert that the data points and their child batches have been completely removed
    assert!(result.get(ArrowPayloadType::NumberDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::NumberDpAttrs).is_none());
    assert!(result.get(ArrowPayloadType::NumberDpExemplars).is_none());
    assert!(
        result
            .get(ArrowPayloadType::NumberDpExemplarAttrs)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::HistogramDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::HistogramDpAttrs).is_none());
    assert!(result.get(ArrowPayloadType::HistogramDpExemplars).is_none());
    assert!(
        result
            .get(ArrowPayloadType::HistogramDpExemplarAttrs)
            .is_none()
    );
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDataPoints)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::ExpHistogramDpAttrs).is_none());
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDpExemplars)
            .is_none()
    );
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::SummaryDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::SummaryDpAttrs).is_none());

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    assert_metrics_eq(
        result_metrics,
        to_metrics_data(vec![
            Metric::build()
                .name("gauge_metric")
                .data_gauge(Gauge {
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("histogram_metric")
                .data_histogram(Histogram {
                    aggregation_temporality: 0,
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("exp_histogram_metric")
                .data_exponential_histogram(ExponentialHistogram {
                    aggregation_temporality: 0,
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("summary_metric")
                .data_summary(Summary {
                    data_points: vec![],
                })
                .finish(),
        ]),
    );
}

/// Scenario: filter metric data points by a field that is not present on the batch containing
/// the metric data point
/// Guarantees: the missing field is treated as evaluating to null, which is treated as false
/// and all the metric data points are removed.
#[tokio::test]
async fn test_filter_data_points_null_predicate_result() {
    let query = "metrics | apply data_points {
        // should resolve to null, which we'll treat as false and we drop all the data points
        where flags as Boolean
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);
    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    // flags are not valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("histogram_metric")
            .data_histogram(Histogram {
                aggregation_temporality: 0,
                data_points: vec![
                    HistogramDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("exp_histogram_metric")
            .data_exponential_histogram(ExponentialHistogram {
                aggregation_temporality: 0,
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("d"))])
                        .positive(Buckets::default())
                        .negative(Buckets::default())
                        .exemplars(vec![
                            Exemplar::build()
                                .time_unix_nano(5u64)
                                .filtered_attributes(vec![KeyValue::new(
                                    "b",
                                    AnyValue::new_string("c"),
                                )])
                                .finish(),
                        ])
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("summary_metric")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .attributes(vec![KeyValue::new("a", AnyValue::new_string("b"))])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));

    // assert field used in predicate is not present
    for payload_type in [
        ArrowPayloadType::SummaryDataPoints,
        ArrowPayloadType::NumberDataPoints,
        ArrowPayloadType::HistogramDataPoints,
        ArrowPayloadType::ExpHistogramDataPoints,
    ] {
        let dp_rb = input_batch.get(payload_type).unwrap();
        assert!(dp_rb.column_by_name(consts::FLAGS).is_none());
    }

    let result = pipeline.execute(input_batch).await.unwrap();

    // assert that the data points and their child batches have been completely removed
    assert!(result.get(ArrowPayloadType::NumberDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::NumberDpAttrs).is_none());
    assert!(result.get(ArrowPayloadType::NumberDpExemplars).is_none());
    assert!(
        result
            .get(ArrowPayloadType::NumberDpExemplarAttrs)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::HistogramDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::HistogramDpAttrs).is_none());
    assert!(result.get(ArrowPayloadType::HistogramDpExemplars).is_none());
    assert!(
        result
            .get(ArrowPayloadType::HistogramDpExemplarAttrs)
            .is_none()
    );
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDataPoints)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::ExpHistogramDpAttrs).is_none());
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDpExemplars)
            .is_none()
    );
    assert!(
        result
            .get(ArrowPayloadType::ExpHistogramDpExemplarAttrs)
            .is_none()
    );
    assert!(result.get(ArrowPayloadType::SummaryDataPoints).is_none());
    assert!(result.get(ArrowPayloadType::SummaryDpAttrs).is_none());

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    assert_metrics_eq(
        result_metrics,
        to_metrics_data(vec![
            Metric::build()
                .name("gauge_metric")
                .data_gauge(Gauge {
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("histogram_metric")
                .data_histogram(Histogram {
                    aggregation_temporality: 0,
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("exp_histogram_metric")
                .data_exponential_histogram(ExponentialHistogram {
                    aggregation_temporality: 0,
                    data_points: vec![],
                })
                .finish(),
            Metric::build()
                .name("summary_metric")
                .data_summary(Summary {
                    data_points: vec![],
                })
                .finish(),
        ]),
    );
}

/// In a handful of tests below, we want to ensure that some filtering is applied to all data
/// point types and we correctly retain only the data points selected by the predicate. This
/// helper sets up data points of all types, ensures the filter is applied to all correctly. The
/// motivation for this is to reduce the boilerplate of creating fixtures and asserting the result
/// of each test case.
async fn run_filter_all_data_point_types_test(
    query: &str,
    flags_and_attrs: Vec<(u32, Option<Vec<KeyValue>>)>,
    expected_retained: Vec<usize>,
) {
    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let mut number_dps = Vec::new();
    let mut hist_dps = Vec::new();
    let mut exp_hist_dps = Vec::new();
    let mut summary_dps = Vec::new();

    for (flag, attrs) in flags_and_attrs.clone() {
        number_dps.push(
            NumberDataPoint::build()
                .flags(flag)
                .attributes(attrs.clone().unwrap_or_default())
                .finish(),
        );
        hist_dps.push(
            HistogramDataPoint::build()
                .flags(flag)
                .attributes(attrs.clone().unwrap_or_default())
                .finish(),
        );
        exp_hist_dps.push(
            ExponentialHistogramDataPoint::build()
                .flags(flag)
                .positive(Buckets::default())
                .negative(Buckets::default())
                .attributes(attrs.clone().unwrap_or_default())
                .finish(),
        );
        summary_dps.push(
            SummaryDataPoint::build()
                .flags(flag)
                .attributes(attrs.clone().unwrap_or_default())
                .finish(),
        );
    }

    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: number_dps.clone(),
            })
            .finish(),
        Metric::build()
            .name("sum")
            .data_sum(Sum {
                data_points: number_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("histogram")
            .data_histogram(Histogram {
                data_points: hist_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("exp_histogram")
            .data_exponential_histogram(ExponentialHistogram {
                data_points: exp_hist_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("summary")
            .data_summary(Summary {
                data_points: summary_dps.clone(),
            })
            .finish(),
    ];

    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(metrics_result) = otap_to_otlp(&result) else {
        panic!("invalid signal type")
    };

    let mut expected_number_dps = Vec::new();
    let mut expected_hist_dps = Vec::new();
    let mut expected_exp_hist_dps = Vec::new();
    let mut expected_summary_dps = Vec::new();

    for i in expected_retained {
        expected_number_dps.push(number_dps[i].clone());
        expected_hist_dps.push(hist_dps[i].clone());
        expected_exp_hist_dps.push(exp_hist_dps[i].clone());
        expected_summary_dps.push(summary_dps[i].clone());
    }

    let expected = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: expected_number_dps.clone(),
            })
            .finish(),
        Metric::build()
            .name("sum")
            .data_sum(Sum {
                data_points: expected_number_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("histogram")
            .data_histogram(Histogram {
                data_points: expected_hist_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("exp_histogram")
            .data_exponential_histogram(ExponentialHistogram {
                data_points: expected_exp_hist_dps.clone(),
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("summary")
            .data_summary(Summary {
                data_points: expected_summary_dps.clone(),
            })
            .finish(),
    ];

    assert_metrics_eq(metrics_result, to_metrics_data(expected))
}

/// Scenario: Filter metric data points having an attribute value equal to some scalar
/// Guarantees: the engine can filter metric data points by this type of predicate
#[tokio::test]
async fn test_filter_data_point_by_attribute_value() {
    let query = "metrics | apply data_points {
        where attributes[\"x\"] == 5
    }";

    run_filter_all_data_point_types_test(
        query,
        vec![
            (
                1u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(5)),
                ]),
            ),
            (
                2u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(6)),
                ]),
            ),
            (
                3u32,
                Some(vec![KeyValue::new("a", AnyValue::new_string("b"))]),
            ),
            (4u32, None),
        ],
        vec![0],
    )
    .await
}

/// Scenario: Filter metric data points by a predicate that will involve a bitmap join for the
/// two binary expressions AND'd together
/// Guarantees: the engine can filter metric data points by this type of predicate
#[tokio::test]
async fn test_filter_data_point_by_attribute_and() {
    let query = "metrics | apply data_points {
        where attributes[\"x\"] == 5 and attributes[\"y\"] == 6
    }";

    run_filter_all_data_point_types_test(
        query,
        vec![
            (
                2u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(6)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                1u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(5)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                3u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (4u32, None),
        ],
        vec![1],
    )
    .await;
}

/// Scenario: Filter metric data points by a predicate that will involve a bitmap join for the
/// two binary expressions OR'd together
/// Guarantees: the engine can filter metric data points by this type of predicate
#[tokio::test]
async fn test_filter_data_point_by_attribute_or() {
    let query = "metrics | apply data_points {
        where attributes[\"x\"] == 5 or attributes[\"y\"] == 6
    }";
    run_filter_all_data_point_types_test(
        query,
        vec![
            (
                2u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(6)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                1u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(5)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                3u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (4u32, None),
        ],
        vec![0, 1, 2],
    )
    .await;
}

/// Scenario: Filter metric data points by a predicate that will involve an ID bitmap inversion
/// for the NOT expression
/// Guarantees: the engine can filter metric data points by this type of predicate
#[tokio::test]
async fn test_filter_data_point_by_attribute_logical_binary_inverted() {
    let query = "metrics | apply data_points {
        where not(attributes[\"x\"] == 5)
    }";

    run_filter_all_data_point_types_test(
        query,
        vec![
            (
                2u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(6)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                1u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(5)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                3u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (4u32, None),
        ],
        vec![0, 2, 3],
    )
    .await;
}

/// Scenario: Filter metric data points by a predicate that will check if the metric data point
/// does not have some attribute value
/// Guarantees: the engine can filter metric data points by this type of predicate
#[tokio::test]
async fn test_filter_data_point_by_attribute_is_null() {
    let query = "metrics | apply data_points {
        where attributes[\"x\"] == null
    }";

    run_filter_all_data_point_types_test(
        query,
        vec![
            (
                2u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(6)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                1u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("x", AnyValue::new_int(5)),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (
                3u32,
                Some(vec![
                    KeyValue::new("a", AnyValue::new_string("b")),
                    KeyValue::new("y", AnyValue::new_int(6)),
                ]),
            ),
            (4u32, None),
        ],
        vec![2, 3],
    )
    .await;
}

/// In a handful of tests below, we want to ensure that values are assigned to all data point types
/// with the correct key / value from sources involving various types of expressions. This helper
/// simply populates each type of data point, evaluates the expression, and ensures the correct
/// attribute was assigned. The motivation for this is to reduce the boilerplate of creating
/// fixtures and asserting the result of each test case.
async fn run_assign_to_all_data_point_type_test(
    query: &str,
    flags: u32,
    attributes: Option<Vec<KeyValue>>,
    expected_key: &str,
    expected_value: AnyValue,
) {
    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(flags)
                        .attributes(attributes.clone().unwrap_or_default())
                        .finish(),
                ],
            })
            .finish(),
        Metric::build()
            .name("sum")
            .data_sum(Sum {
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(flags)
                        .attributes(attributes.clone().unwrap_or_default())
                        .finish(),
                ],
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("histogram")
            .data_histogram(Histogram {
                data_points: vec![
                    HistogramDataPoint::build()
                        .flags(flags)
                        .attributes(attributes.clone().unwrap_or_default())
                        .finish(),
                ],
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("exp_histogram")
            .data_exponential_histogram(ExponentialHistogram {
                data_points: vec![
                    ExponentialHistogramDataPoint::build()
                        .flags(flags)
                        .attributes(attributes.clone().unwrap_or_default())
                        .finish(),
                ],
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("summary")
            .data_summary(Summary {
                data_points: vec![
                    SummaryDataPoint::build()
                        .flags(flags)
                        .attributes(attributes.clone().unwrap_or_default())
                        .finish(),
                ],
            })
            .finish(),
    ];

    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(metrics_result) = otap_to_otlp(&result) else {
        panic!("invalid signal type")
    };

    assert_eq!(metrics_result.resource_metrics.len(), 1);
    assert_eq!(metrics_result.resource_metrics[0].scope_metrics.len(), 1);
    assert_eq!(
        metrics_result.resource_metrics[0].scope_metrics[0]
            .metrics
            .len(),
        5
    );
    for metric in metrics_result.resource_metrics[0].scope_metrics[0]
        .metrics
        .iter()
    {
        let attrs = match metric.data.as_ref().unwrap() {
            Data::Gauge(g) => {
                assert_eq!(g.data_points.len(), 1);
                &g.data_points[0].attributes
            }
            Data::Sum(s) => {
                assert_eq!(s.data_points.len(), 1);
                &s.data_points[0].attributes
            }
            Data::Histogram(h) => {
                assert_eq!(h.data_points.len(), 1);
                &h.data_points[0].attributes
            }
            Data::ExponentialHistogram(h) => {
                assert_eq!(h.data_points.len(), 1);
                &h.data_points[0].attributes
            }
            Data::Summary(s) => {
                assert_eq!(s.data_points.len(), 1);
                &s.data_points[0].attributes
            }
        };

        let expected_attr = attrs.iter().find(|kv| kv.key == expected_key).unwrap();
        assert_eq!(expected_attr.value.as_ref().unwrap(), &expected_value);
    }
}

/// Scenario: test creating a new attribute from a scalar value
/// Guarantees: a new attribute can be created from a scalar value
#[tokio::test]
async fn test_assign_to_data_point_attributes() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = 5
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![KeyValue::new("a", AnyValue::new_int(5))]),
        "x",
        AnyValue::new_int(5),
    )
    .await;
}

/// Scenario: test replacing the value of an existing attribute
/// Guarantees: the attribute value is replaced with the new value
#[tokio::test]
async fn test_data_point_replace_existing_attr_value() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = 6
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![KeyValue::new("x", AnyValue::new_int(5))]),
        "x",
        AnyValue::new_int(6),
    )
    .await;
}

/// Scenario: creating a new attribute when the existing telemetry batch had no existing
/// attributes. In this case, there is not an existing attribute record batch to update so the
/// engine will need to synthesize a new one.
/// Guarantees: the new attributes will be created.
#[tokio::test]
async fn test_assign_to_data_point_attributes_no_existing_attrs() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = 5
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        None, // no existing attrs
        "x",
        AnyValue::new_int(5),
    )
    .await;
}

/// Scenario: assigning the value of an attribute from an existing attribute
/// Guarantees: new attribute is created having the same value as the existing attribtue
#[tokio::test]
async fn test_assign_to_data_point_attributes_copy_attribute() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = attributes[\"y\"]
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![KeyValue::new("y", AnyValue::new_int(5))]),
        "x",
        AnyValue::new_int(5),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must join two existing
/// attributes on their parent_id column
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attributes_requiring_join_attrs() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = attributes[\"y\"] + attributes[\"z\"]
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![
            KeyValue::new("y", AnyValue::new_int(3)),
            KeyValue::new("z", AnyValue::new_int(4)),
        ]),
        "x",
        AnyValue::new_int(7),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must join an existing
/// attribute and a field from the data point record batch on left.parent_id = right.id columns
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attributes_requiring_join_attrs_and_record_right() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = attributes[\"y\"] - flags
    }";

    run_assign_to_all_data_point_type_test(
        query,
        2,
        Some(vec![KeyValue::new("y", AnyValue::new_int(5))]),
        "x",
        AnyValue::new_int(3),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must join an existing
/// attribute and a field from the data point record batch on left.id = right.parent_id columns
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attributes_requiring_join_attrs_and_record_left() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = flags - attributes[\"y\"]
    }";

    run_assign_to_all_data_point_type_test(
        query,
        5,
        Some(vec![KeyValue::new("y", AnyValue::new_int(3))]),
        "x",
        AnyValue::new_int(2),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must do a join of
/// multiple scopes, in this case being scalar -> attributes.parent_id -> data point.id
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attrs_requiring_multi_join_attrs_and_scalar_and_record_right() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = join(\".\", attributes[\"y\"], flags as String)
    }";

    run_assign_to_all_data_point_type_test(
        query,
        5,
        Some(vec![KeyValue::new("y", AnyValue::new_string("b"))]),
        "x",
        AnyValue::new_string("b.5"),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must do a join of
/// multiple scopes, in this case being scalar -> data point.id -> attributes.parent_id
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attrs_requiring_multi_join_attrs_and_scalar_and_record_left() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = join(\".\", flags as String, attributes[\"y\"])
    }";

    run_assign_to_all_data_point_type_test(
        query,
        5,
        Some(vec![KeyValue::new("y", AnyValue::new_string("b"))]),
        "x",
        AnyValue::new_string("5.b"),
    )
    .await;
}

/// Scenario: assign a new attribute computed from a binary expression that must do a join of
/// multiple scopes, in this case being scalar -> attrs.parent_id -> attributes.parent_id
/// Guarantees: expression evaluates to produce the correct result and the result is assigned to
/// the new attribute value
#[tokio::test]
async fn test_assign_to_data_point_attributes_requiring_multi_join_attrs_and_scalar() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = join(\".\", attributes[\"y\"], attributes[\"z\"])
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![
            KeyValue::new("y", AnyValue::new_string("b")),
            KeyValue::new("z", AnyValue::new_string("c")),
        ]),
        "x",
        AnyValue::new_string("b.c"),
    )
    .await;
}

/// Scenario: assign a new attribute computed from an expression that produces a boolean value
/// Guarantees: The new boolean valued attribute is created
#[tokio::test]
async fn test_assign_to_data_point_attributes_from_logical_binary_expr() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = attributes[\"y\"] > 2
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![KeyValue::new("y", AnyValue::new_int(3))]),
        "x",
        AnyValue::new_bool(true),
    )
    .await;
}

/// Scenario: assign a new attribute computed from an expression that produces a boolean value
/// using a join executed as a bitmap join of ID columns.
/// Guarantees: The new boolean valued attribute is created
#[tokio::test]
async fn test_assign_to_data_point_attributes_requiring_bitmap_join_attrs() {
    let query = "metrics | apply data_points {
        set attributes[\"x\"] = attributes[\"y\"] == 3 and attributes[\"z\"] == 4
    }";

    run_assign_to_all_data_point_type_test(
        query,
        Default::default(),
        Some(vec![
            KeyValue::new("y", AnyValue::new_int(3)),
            KeyValue::new("z", AnyValue::new_int(4)),
        ]),
        "x",
        AnyValue::new_bool(true),
    )
    .await;
}

/// Scenario: filter metric data points by attribute predicates when the data point attributes
/// batch has more than 256 distinct parent_ids, forcing the parent_id column to use
/// Dict<UInt16, UInt32> dictionary encoding instead of the default Dict<UInt8, UInt32>
/// Guarantees: the bitmap join correctly handles Dict<UInt16, UInt32> parent_id columns
#[tokio::test]
async fn test_filter_data_point_by_attribute_with_dict_u16_parent_ids() {
    let query = "metrics | apply data_points {
        where attributes[\"x\"] == 999 and attributes[\"z\"] == 999
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    // Create 300 data points, each with unique attribute values. This exceeds the 256
    // distinct value threshold for Dict<UInt8> keys, forcing the encoder to upgrade the
    // parent_id column in the dp attributes batch to Dict<UInt16, UInt32>.
    let num_data_points = 300;
    let matching_index = 150;

    let data_points: Vec<NumberDataPoint> = (0..num_data_points)
        .map(|i| {
            let x_val = if i == matching_index { 999 } else { i };
            let z_val = if i == matching_index { 999 } else { i + 1000 };
            NumberDataPoint::build()
                .flags(i as u32)
                .attributes(vec![
                    KeyValue::new("x", AnyValue::new_int(x_val as i64)),
                    KeyValue::new("z", AnyValue::new_int(z_val as i64)),
                ])
                .finish()
        })
        .collect();

    let metrics = vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge { data_points })
            .finish(),
    ];

    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));

    // verify that the dp attrs parent_id column is indeed Dict<UInt16, _> encoded
    let dp_attrs = input_batch
        .get(ArrowPayloadType::NumberDpAttrs)
        .expect("dp attrs should be present");
    let parent_id_col = dp_attrs
        .column_by_name("parent_id")
        .expect("parent_id column should be present");
    assert!(
        matches!(
            parent_id_col.data_type(),
            arrow::datatypes::DataType::Dictionary(k, _) if k.as_ref() == &arrow::datatypes::DataType::UInt16
        ),
        "expected Dict<UInt16, _> parent_id but got {:?}",
        parent_id_col.data_type()
    );

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(metrics_result) = otap_to_otlp(&result) else {
        panic!("invalid signal type")
    };

    // only one data point should survive: the one at matching_index
    let expected = to_metrics_data(vec![
        Metric::build()
            .name("gauge_metric")
            .data_gauge(Gauge {
                data_points: vec![
                    NumberDataPoint::build()
                        .flags(matching_index as u32)
                        .attributes(vec![
                            KeyValue::new("x", AnyValue::new_int(999)),
                            KeyValue::new("z", AnyValue::new_int(999)),
                        ])
                        .finish(),
                ],
            })
            .finish(),
    ]);

    assert_metrics_eq(metrics_result, expected);
}

/// Scenario: try to execute some queries that have valid syntax, but define operations that are
/// not supported by this query engine (although most will be supported in future)
/// Guarantees: that the operation returns an expected error instead of inadvertently evaluating
/// and producing invalid results
#[tokio::test]
async fn test_not_supported_queries_return_error() {
    struct TestCase {
        query: &'static str,
    }

    let test_cases = [
        TestCase {
            query: "metrics | apply data_points {
                where resource.attributes[\"x\"] > 0
            }",
        },
        // filtering by checking the type of metric data point is not yet supported
        TestCase {
            query: "metrics | apply data_points {
                where is Log
            }",
        },
        // conditional operator call (if/else) is not yet supported for metric data points
        TestCase {
            query: "metrics | apply data_points {
                if (flags > 0) {
                    drop
                }
            }",
        },
        // assignment to metric data point fields or attributes is not yet supported
        TestCase {
            query: "metrics | apply data_points {
                set flags = 0
            }",
        },
        // nested apply pipeline to modify data point attributes is not yet supported
        TestCase {
            query: "metrics | apply data_points {
                    where flags > 5 |
                    apply attributes {
                        set value = 5
                    }
                }",
        },
        // assert that special attribute operations are not yet supported
        TestCase {
            query: "metrics | apply data_points {
                rename attributes \"x\" as \"y\"
            }",
        },
        TestCase {
            query: "metrics | apply data_points {
                remove attributes[\"x\"]
            }",
        },
        // the following two cases, where we're accessing resource attributes for some data point
        // should probably never be supported (instead, renaming attributes should be supported at
        // the level metric itself).
        TestCase {
            query: "metrics | apply data_points {
                rename resource.attributes \"x\" as \"y\"
            }",
        },
        TestCase {
            query: "metrics | apply data_points {
                remove resource.attributes[\"x\"]
            }",
        },
    ];

    for test_case in test_cases {
        let metrics = vec![
            Metric::build()
                .data_gauge(Gauge {
                    data_points: vec![
                        NumberDataPoint::build()
                            .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                            .finish(),
                    ],
                })
                .finish(),
        ];

        let pipeline_expr =
            OplParser::parse_with_options(test_case.query, default_parser_options())
                .unwrap()
                .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
        if pipeline.execute(input_batch).await.is_ok() {
            panic!(
                "unexpectedly did not produce error for query {:?}",
                test_case.query
            );
        }
    }
}
