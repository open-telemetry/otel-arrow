// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for scaling representative metric batches through the query engine.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_contrib_data_engine_kql_parser::Parser;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
    Exemplar, Gauge, Histogram, HistogramDataPoint, Metric, NumberDataPoint, Sum, Summary,
    SummaryDataPoint, summary_data_point::ValueAtQuantile,
};
use otel_arrow_dfe_pdata::testing::round_trip::to_otap_metrics;
use otel_arrow_dfe_query_engine::parser::default_parser_options;
use otel_arrow_dfe_query_engine::pipeline::Pipeline;
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;
use tokio::runtime::Runtime;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn generate_metrics_batch(data_points_per_type: usize) -> OtapArrowRecords {
    let number_data_points = (0..data_points_per_type)
        .map(|index| {
            let builder = NumberDataPoint::build()
                .exemplars(vec![Exemplar::build().value_double(index as f64).finish()]);
            if index.is_multiple_of(2) {
                builder.value_int(index as i64).finish()
            } else {
                builder.value_double(index as f64).finish()
            }
        })
        .collect::<Vec<_>>();

    let histogram_data_points = (0..data_points_per_type)
        .map(|index| {
            HistogramDataPoint::build()
                .count(8u64)
                .sum(36.0 + index as f64)
                .min(1.0)
                .max(8.0 + index as f64)
                .bucket_counts([1, 1, 1, 1, 1, 1, 1, 1])
                .explicit_bounds([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])
                .exemplars(vec![
                    Exemplar::build().value_double(4.0 + index as f64).finish(),
                ])
                .finish()
        })
        .collect();

    let summary_data_points = (0..data_points_per_type)
        .map(|index| {
            SummaryDataPoint::build()
                .count(8u64)
                .sum(36.0 + index as f64)
                .quantile_values(vec![
                    ValueAtQuantile::new(0.5, 4.0 + index as f64),
                    ValueAtQuantile::new(0.9, 7.0 + index as f64),
                ])
                .finish()
        })
        .collect();

    to_otap_metrics(vec![
        Metric::build()
            .name("gauge")
            .unit("s")
            .data_gauge(Gauge {
                data_points: number_data_points.clone(),
            })
            .finish(),
        Metric::build()
            .name("sum")
            .unit("s")
            .data_sum(Sum {
                data_points: number_data_points,
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("histogram")
            .unit("s")
            .data_histogram(Histogram {
                data_points: histogram_data_points,
                ..Default::default()
            })
            .finish(),
        Metric::build()
            .name("summary")
            .unit("s")
            .data_summary(Summary {
                data_points: summary_data_points,
            })
            .finish(),
    ])
}

fn bench_scale_metric(c: &mut Criterion) {
    let rt = Runtime::new().expect("can build Tokio runtime");
    let mut group = c.benchmark_group("scale_metric_mixed_metrics");

    for data_points_per_type in [32, 256, 1024] {
        let _ = group.throughput(Throughput::Elements((data_points_per_type * 4) as u64));
        let benchmark_id = BenchmarkId::new("data_points_per_type", data_points_per_type);
        let _ = group.bench_with_input(
            benchmark_id,
            &data_points_per_type,
            |b, data_points_per_type| {
                b.iter_custom(|iterations| {
                    let batch = generate_metrics_batch(*data_points_per_type);
                    let parser_result = OplParser::parse_with_options(
                        r#"metrics | scale_metric 1000 "ms""#,
                        default_parser_options(),
                    )
                    .expect("can parse scale_metric pipeline");
                    let mut pipeline = Pipeline::new(parser_result.pipeline);

                    rt.block_on(async {
                        _ = pipeline
                            .execute(batch.clone())
                            .await
                            .expect("scale_metric warmup succeeds");

                        let start = Instant::now();
                        for _ in 0..iterations {
                            let result = pipeline
                                .execute(batch.clone())
                                .await
                                .expect("scale_metric succeeds");
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

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_scale_metric
    );
}

criterion_main!(benches::benches);
