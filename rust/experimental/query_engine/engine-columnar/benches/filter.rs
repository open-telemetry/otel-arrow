// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_columnar::pipeline::Pipeline;
use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::fixtures::logs_with_varying_attributes_and_properties;
use otap_df_pdata::testing::round_trip::otlp_to_otap;
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
                    pipeline.execute(batch.clone()).await.unwrap();

                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = pipeline.execute(batch.clone()).await.unwrap();
                        std::hint::black_box(result);
                    }
                    start.elapsed()
                })
            });
        });
    }
    group.finish();
}

fn bench_filter_pipelines(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    let batch_sizes = [32, 1024, 8192];
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "simple_field_filter",
        "logs | where severity_text == \"WARN\"",
    );
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "simple_attr_filter",
        "logs | where attributes[\"code.namespace\"] == \"main\"",
    );
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "attr_or_attr_filter",
        "logs | where attributes[\"code.namespace\"] == \"main\" or attributes[\"code.line.number\"] == 2",
    );
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "attr_and_prop_filter",
        "logs | where attributes[\"code.namespace\"] == \"main\" and severity_text == \"WARN\"",
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "attr_and_attr_filter",
        "logs | where attributes[\"code.namespace\"] == \"main\" and attributes[\"code.line\"] == 2",
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "attr_and_or_together_filter", 
        "logs | where 
            (attributes[\"code.namespace\"] == \"main\" and attributes[\"code.line\"] == 2) 
            or 
            (attributes[\"code.namespace\"] == \"otap_dataflow_engine\" and attributes[\"code.line.number\"] == 3)",
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "and_attrs_short_circuit",
        // left expr of "and" should always return false for all rows
        "logs | where attributes[\"code.line.number\"] > 1000 and attributes[\"code.line.number\"] == 2",
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "and_short_circuit",
        // left expr of "and" should be false for all rows
        //
        // this is different from the case above in that the "and" here is currently something that
        // won't get optimized into a Composite<AttributeFilterExec> so we can test the fast path
        // in Composite<FilterExec>
        "logs | where severity_text == \"invalid value\" and attributes[\"code.line.number\"] == 2",
    );

    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "or_short_circuit",
        // left expr of "or" should be true for all rows
        //
        // this is different from the case above in that the "and" here is currently something that
        // won't get optimized into a Composite<AttributeFilterExec> so we can test the fast path
        // in Composite<FilterExec>
        "logs | where attributes[\"code.line.number\"] >= 0 or not(attributes[\"some.attr\"] >= 0 and severity_text == \"WARN\")",
    );
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_filter_pipelines
    );
}

criterion_main!(benches::benches);
