// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests for pipelines applied to metrics data points

use otel_arrow_contrib_data_engine_kql_parser::Parser;
use otel_arrow_dfe_pdata::{
    proto::{
        OtlpProtoMessage,
        opentelemetry::{
            common::v1::{AnyValue, KeyValue},
            metrics::v1::{Gauge, Metric, NumberDataPoint},
        },
    },
    testing::round_trip::{otap_to_otlp, otlp_to_otap, to_metrics_data},
};
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

use crate::parser::default_parser_options;
use crate::pipeline::Pipeline;

#[tokio::test]
async fn test_simple_datapoint_filter() {
    let query = "metrics | apply data_points {
        where flags > 5
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

                    // TODO - set some exemplars, etc?
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

#[tokio::test]
async fn test_filter_datapoints_by_scalar() {
    let query = "metrics | apply data_points {
        where contains(\"foo\", \"f\") // should evaluate to scalar True
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
                    NumberDataPoint::build().flags(5u32).finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                    NumberDataPoint::build().flags(6u32).finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    println!("{:#?}", result_metrics);

    // OTHER TEST:

    let query = "metrics | apply data_points {
        where contains(\"foo\", \"b\") // should evaluate to scalar False
    }";

    let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
        .unwrap()
        .pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    println!("{:#?}", result_metrics);
}

#[tokio::test]
async fn test_filter_datapoints_by_propagated_nulls() {
    let query = "metrics | apply data_points {
        // should resolve to null, which we'll treat as false and we drop all the datapoints
        where flags as Boolean
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
                    NumberDataPoint::build().finish(),
                    NumberDataPoint::build().finish(),
                    NumberDataPoint::build().finish(),
                ],
            })
            .finish(),
    ];
    let input_batch = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics.clone())));

    let result = pipeline.execute(input_batch).await.unwrap();

    let OtlpProtoMessage::Metrics(result_metrics) = otap_to_otlp(&result) else {
        panic!("invalid result type")
    };

    println!("{:#?}", result_metrics);
}

// shouldn't allow this?
#[tokio::test]
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
