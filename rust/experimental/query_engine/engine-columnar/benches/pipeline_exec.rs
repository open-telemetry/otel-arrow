// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_columnar::datagen::generate_logs_batch;
use data_engine_columnar::engine::ExecutablePipeline;
use data_engine_kql_parser::{KqlParser, Parser};

fn bench_exec_pipelines(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    let batch_sizes = [32, 1024, 8192];

    let mut group = c.benchmark_group("simple_field_filter");
    for batch_size in batch_sizes {
        let benchmark_id = BenchmarkId::new("batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, batch_size| {
            b.iter_custom(|iters| {
                let batch = generate_logs_batch(*batch_size, 0);
                let query = "logs | where severity_number == \"WARN\"";
                let pipeline = KqlParser::parse(query).expect("can parse pipeline");
                rt.block_on(async move {
                    let mut exec_pipeline = ExecutablePipeline::try_new(batch.clone(), pipeline)
                        .await
                        .unwrap();

                    let start = Instant::now();
                    for _ in 0..iters {
                        exec_pipeline.update_batch(batch.clone()).unwrap();
                        exec_pipeline.execute().await.unwrap();
                    }
                    start.elapsed()
                })
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("simple_attr_filter");
    for batch_size in batch_sizes {
        let benchmark_id = BenchmarkId::new("batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, batch_size| {
            b.iter_custom(|iters| {
                let batch = generate_logs_batch(*batch_size, 0);
                let query = "logs | where attributes[\"k8s.ns\"] == \"prod\"";
                let pipeline = KqlParser::parse(query).expect("can parse pipeline");
                rt.block_on(async move {
                    let mut exec_pipeline = ExecutablePipeline::try_new(batch.clone(), pipeline)
                        .await
                        .unwrap();

                    let start = Instant::now();
                    for _ in 0..iters {
                        exec_pipeline.update_batch(batch.clone()).unwrap();
                        exec_pipeline.execute().await.unwrap();
                    }
                    start.elapsed()
                })
            });
        });
    }
    group.finish();
}

#[allow(missing_docs)]
mod benches {
    use super::*;
    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_exec_pipelines
    );
}
criterion_main!(benches::benches);
