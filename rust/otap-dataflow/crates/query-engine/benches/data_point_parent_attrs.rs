// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for assigning metric data point attributes from resource and scope attributes.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_contrib_data_engine_kql_parser::Parser;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
use otel_arrow_dfe_query_engine::parser::default_parser_options;
use otel_arrow_dfe_query_engine::pipeline::Pipeline;
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;
use tokio::runtime::Runtime;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const NUM_SCOPES: usize = 16;
const METRICS_PER_SCOPE: usize = 4;

fn generate_metrics_batch(data_points_per_metric: usize) -> OtapArrowRecords {
    let scope_metrics = (0..NUM_SCOPES)
        .map(|scope| {
            let metrics = (0..METRICS_PER_SCOPE)
                .map(|metric| {
                    let data_points = (0..data_points_per_metric)
                        .map(|index| {
                            NumberDataPoint::build()
                                .value_double(index as f64)
                                .attributes(vec![KeyValue::new(
                                    "y",
                                    AnyValue::new_string(format!("value-{index}")),
                                )])
                                .finish()
                        })
                        .collect();
                    Metric::build()
                        .name(format!("metric-{metric}"))
                        .data_gauge(Gauge { data_points })
                        .finish()
                })
                .collect::<Vec<_>>();

            ScopeMetrics::new(
                InstrumentationScope::build()
                    .name(format!("scope-{scope}"))
                    .attributes(vec![
                        KeyValue::new(
                            "pipeline.id",
                            AnyValue::new_string(format!("pipeline-{scope}")),
                        ),
                        KeyValue::new(
                            "custom",
                            AnyValue::new_kvlist(vec![KeyValue::new(
                                "componentName",
                                AnyValue::new_string(format!("component-{scope}")),
                            )]),
                        ),
                    ])
                    .finish(),
                metrics,
            )
        })
        .collect::<Vec<_>>();

    otlp_to_otap(&OtlpProtoMessage::Metrics(MetricsData::new(vec![
        ResourceMetrics::new(
            Resource::build()
                .attributes(vec![KeyValue::new(
                    "service.instance.id",
                    AnyValue::new_string("instance-0"),
                )])
                .finish(),
            scope_metrics,
        ),
    ])))
}

fn bench_data_point_parent_attrs(c: &mut Criterion) {
    let rt = Runtime::new().expect("can build Tokio runtime");
    let queries = [
        (
            "data_point_attr",
            r#"metrics | apply data_points { set attributes["x"] = attributes["y"] }"#,
        ),
        (
            "scope_attr",
            r#"metrics | apply data_points { set attributes["x"] = instrumentation_scope.attributes["pipeline.id"] }"#,
        ),
        (
            "nested_scope_attr",
            r#"metrics | apply data_points { set attributes["x"] = instrumentation_scope.attributes["custom"]["componentName"] }"#,
        ),
        (
            "resource_attr",
            r#"metrics | apply data_points { set attributes["x"] = resource.attributes["service.instance.id"] }"#,
        ),
    ];

    for (name, query) in queries {
        let mut group = c.benchmark_group(format!("data_point_parent_attrs/{name}"));
        for data_points_per_metric in [8, 64, 256] {
            let num_data_points = NUM_SCOPES * METRICS_PER_SCOPE * data_points_per_metric;
            let _ = group.throughput(Throughput::Elements(num_data_points as u64));
            let _ = group.bench_with_input(
                BenchmarkId::new("data_points", num_data_points),
                &data_points_per_metric,
                |b, data_points_per_metric| {
                    b.iter_custom(|iterations| {
                        let batch = generate_metrics_batch(*data_points_per_metric);
                        let parser_result =
                            OplParser::parse_with_options(query, default_parser_options())
                                .expect("can parse pipeline");
                        let mut pipeline =
                            Pipeline::try_new(parser_result.pipeline).expect("pipeline");

                        rt.block_on(async {
                            _ = pipeline
                                .execute(batch.clone())
                                .await
                                .expect("warmup succeeds");

                            let start = Instant::now();
                            for _ in 0..iterations {
                                let result = pipeline
                                    .execute(batch.clone())
                                    .await
                                    .expect("pipeline succeeds");
                                _ = std::hint::black_box(result);
                            }
                            start.elapsed()
                        })
                    });
                },
            );
        }
        group.finish();
    }
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_data_point_parent_attrs
    );
}

criterion_main!(benches::benches);
