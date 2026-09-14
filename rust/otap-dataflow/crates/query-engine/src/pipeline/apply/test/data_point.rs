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
                SummaryDataPoint, exponential_histogram_data_point::Buckets,
            },
        },
    }, schema::consts, testing::round_trip::{otap_to_otlp, otlp_to_otap, to_metrics_data},
};
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

use crate::parser::default_parser_options;
use crate::pipeline::Pipeline;

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
                    NumberDataPoint::build().finish(),
                    NumberDataPoint::build().finish(),
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

// shouldn't allow this?
#[tokio::test]
#[ignore]
async fn test_apply_to_metric_data_points() {
    // this is currently a planning error!
    let query = "metrics | apply data_points {
        where flags > 5 |
        apply attributes {
            set value = 5
        }
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let metrics = vec![
        Metric::build()
            .data_gauge(Gauge {
                data_points: vec![
                    // flags are not valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .flags(5u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    println!("{:#?}", result_metrics)

    // TODO assert the result
}

// TODO function call
// TODO type check (e.g. is NumberDataPoint)

// TODO - shouldn't allow this ...
#[tokio::test]
#[ignore]
async fn test_apply_to_metric_set() {
    // this is currently a planning error!
    let query = "metrics | apply data_points {
        set resource.attributes[\"x\"] = flags as String
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let metrics = vec![
        Metric::build()
            .data_gauge(Gauge {
                data_points: vec![
                    // flags are not valid flag values but, just need to set some primitive field
                    // for testing engine behaviour
                    NumberDataPoint::build()
                        .flags(5u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                    NumberDataPoint::build()
                        .flags(6u32)
                        .attributes(vec![KeyValue::new("x", AnyValue::new_int(3))])
                        .finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    println!("{:#?}", result_metrics)

    // TODO assert the result
}
