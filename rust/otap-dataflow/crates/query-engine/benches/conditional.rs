// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark for query engine conditional pipeline stage
use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_opl::parser::OplParser;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::OtapBatchStore;
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
                    OplParser::parse(bench_pipeline_kql).expect("can parse pipeline");
                let mut pipeline = Pipeline::new(parser_result.pipeline);
                rt.block_on(async move {
                    // execute the query once to initiate planning
                    _ = pipeline.execute(batch.clone()).await.expect("doesn't fail");

                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = pipeline.execute(batch.clone()).await.expect("doesn't fail");
                        // let OtapArrowRecords::Logs(logs) = result else {
                        //     todo!();
                        // };

                        // for batch in logs.into_batches() {
                        //     if let Some(batch) = batch {
                        //         arrow::util::pretty::print_batches(&[batch]).unwrap()
                        //     }
                        // }
                        // todo!()
                        // // println!("{:?}", result);
                        _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                })
            });
        });
    }
    group.finish();
}

fn bench_conditional_pipeline(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    let batch_sizes = [32, 1536, 8192];

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "conditional_assign",
        r#"logs |
            if (severity_text == "WARN") {
                set event_name = "warn happen"
            } else {
                set event_name = "no warn happen"
            }
        "#,
    );
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_conditional_pipeline
    );
}

criterion_main!(benches::benches);
