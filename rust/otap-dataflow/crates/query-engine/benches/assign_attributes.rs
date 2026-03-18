// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for query engine attribute assignment (upsert) via the Pipeline API.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otap_df_pdata::testing::round_trip::otlp_to_otap;
use otap_df_query_engine::pipeline::Pipeline;
use tokio::runtime::Runtime;

#[must_use]
fn generate_logs_batch(batch_size: usize) -> OtapArrowRecords {
    let log_records = (0..batch_size)
        .map(|i| {
            // generate some log attributes that somewhat follow semantic conventions
            let attrs = vec![
                KeyValue::new(
                    "code.namespace",
                    AnyValue::new_string(match i % 3 {
                        0 => "main",
                        1 => "otap_dataflow_engine",
                        _ => "arrow::array",
                    }),
                ),
                KeyValue::new(
                    "code.function.name",
                    AnyValue::new_string(match i % 3 {
                        0 => "main",
                        1 => "try_recv",
                        _ => "concat",
                    }),
                ),
                KeyValue::new("code.line.number", AnyValue::new_int((i % 5) as i64)),
                KeyValue::new("code.column.number", AnyValue::new_int(40i64)),
            ];

            // cycle through severity numbers
            // 5 = DEBUG, 9 = INFO, 13 = WARN, 17 = ERROR
            let severity_number =
                SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).expect("valid severity_number");
            let severity_text = severity_number
                .as_str_name()
                .split("_") // Note: this splitting something like SEVERITY_NUMBER_INFO
                .nth(2)
                .expect("can parse severity_text");
            let event_name = format!("event {}", i);
            let time_unix_nano = i as u64;

            LogRecord::build()
                .attributes(attrs)
                .event_name(event_name)
                .severity_number(severity_number)
                .severity_text(severity_text)
                .time_unix_nano(time_unix_nano)
                .finish()
        })
        .collect::<Vec<_>>();

    otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }))
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

    // Execute two attribute assignments at once. The planner should fuse these operations into
    // a single assignment pipeline stage, which should run faster than two sequential stages.
    // the time of these benchmark cases should be less than twice the previous two
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

    // mix of insert and upsert
    bench_log_pipeline(
        c,
        &rt,
        &batch_sizes,
        "upsert_two_existing_one_new_str_keys",
        r#"logs | extend attributes["code.namespace"] = "hello", attributes["code.function.name"] = "world", attributes["new_key2"] = "val2""#,
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
