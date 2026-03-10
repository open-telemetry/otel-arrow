// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for query engine attribute assignment (upsert) via the Pipeline API.
//!
//! These benchmarks exercise the full `Pipeline::execute` path for
//! `logs | extend attributes["key"] = value` statements, measuring the end-to-end cost of
//! attribute upserts through the query engine.
//!
//! Scenarios mirror those from the pdata-level `transform_attributes` upsert benchmarks
//! (PR #2024) so that the two approaches can be compared directly.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::fixtures::logs_with_varying_attributes_and_properties;
use otap_df_pdata::testing::round_trip::otlp_to_otap;
use otap_df_query_engine::pipeline::Pipeline;
use tokio::runtime::Runtime;

fn generate_logs_batch(batch_size: usize) -> OtapArrowRecords {
    let logs_data = logs_with_varying_attributes_and_properties(batch_size);
    otlp_to_otap(&OtlpProtoMessage::Logs(logs_data))
}

fn bench_log_pipeline(
    c: &mut Criterion,
    rt: &Runtime,
    batch_sizes: &[usize],
    bench_group_name: &str,
    bench_pipeline_kql: &str,
) {
    let mut group = c.benchmark_group(bench_group_name);
    for batch_size in batch_sizes {
        let benchmark_id = BenchmarkId::new("batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, batch_size| {
            b.iter_custom(|iters| {
                let batch = generate_logs_batch(**batch_size);
                let parser_result =
                    KqlParser::parse(bench_pipeline_kql).expect("can parse pipeline");
                let mut pipeline = Pipeline::new(parser_result.pipeline);
                rt.block_on(async move {
                    // execute the query once to initiate planning
                    _ = pipeline.execute(batch.clone()).await.expect("doesn't fail");

                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = pipeline.execute(batch.clone()).await.expect("doesn't fail");
                        _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                })
            });
        });
    }
    group.finish();
}

fn bench_assign_attribute_pipelines(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    let batch_sizes = [128, 1536, 8192];

    // Upsert a string key that does NOT exist on any log (pure insert path).
    // Every parent gets a new attribute row appended.
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "upsert_new_str_key",
        r#"logs | extend attributes["new_key"] = "new_val""#,
    );

    // Upsert a string key that DOES exist on every log (pure update path).
    // "code.namespace" is present on every log record in the fixture.
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "upsert_existing_str_key",
        r#"logs | extend attributes["code.namespace"] = "updated""#,
    );

    // // Upsert a new Int64 key (dict-eligible per OTAP spec, encoded as Dict(u16, Int64)).
    // bench_log_pipeline(
    //     c,
    //     &rt,
    //     &batch_sizes,
    //     "upsert_new_int_key",
    //     r#"logs | extend attributes["new_int"] = 42"#,
    // );

    // // Upsert a new Float64 key (NOT dict-eligible per OTAP spec, stays plain Float64).
    // bench_log_pipeline(
    //     c,
    //     &rt,
    //     &batch_sizes,
    //     "upsert_new_double_key",
    //     r#"logs | extend attributes["new_double"] = 3.14"#,
    // );

    // Two new string keys in a single pipeline (sequential stages, not yet fused).
    // This measures the current cost of two sequential upserts via the pipeline.
    // Once planner-level fusion is implemented, this will exercise the batched
    // `upsert_attributes` path and should show a significant improvement.
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "upsert_two_new_str_keys",
        r#"logs | extend attributes["new_key1"] = "val1", attributes["new_key2"] = "val2""#,
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "upsert_two_existing_str_keys",
        r#"logs | extend attributes["code.namespace"] = "hello", attributes["code.function.name"] = "world""#,
    );
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_assign_attribute_pipelines
    );
}

criterion_main!(benches::benches);
