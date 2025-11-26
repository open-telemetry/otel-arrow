// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_columnar::pipeline::Pipeline;
use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_pdata::proto::opentelemetry::common::v1::{KeyValue, AnyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::round_trip::otlp_to_otap;
use otap_df_pdata::OtapArrowRecords;
use tokio::runtime::Runtime;

fn generate_logs_batch(batch_size: usize) -> OtapArrowRecords {
    let log_records = (0..batch_size).map(|i| {
        // generate some log attributes that somewhat follow semantic conventions
        let attrs = vec![
            KeyValue::new(
                "code.namespace", 
                AnyValue::new_string(match i % 3 {
                    0 => "main",
                    1 => "otap_dataflow_engine",
                    _ => "arrow::array"
                })
            ),
            KeyValue::new(
                "code.line.number",
                AnyValue::new_int((i % 5) as i64)
            )
        ];

        // cycle through severity numbers
        // 5 = DEBUG, 9 = INFO, 13 = WARN, 17 = ERROR 
        let severity_number = SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).unwrap();
        let severity_text = severity_number.as_str_name().split("_").nth(2).unwrap();
        let event_name = format!("event {}", i);
        let time_unix_nano = i as u64;

        LogRecord::build()
            .attributes(attrs)
            .event_name(event_name)
            .severity_number(severity_number)
            .severity_text(severity_text)
            .time_unix_nano(time_unix_nano)
            .finish()
    }).collect::<Vec<_>>();

    otlp_to_otap(&OtlpProtoMessage::Logs(LogsData {
        resource_logs: vec![
            ResourceLogs {
                scope_logs: vec![
                    ScopeLogs {
                        log_records,
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
        ]
    }))
}

fn bench_pipeline(
    c: &mut Criterion,
    rt: &Runtime,
    batch_sizes: &[usize],
    bench_group_name: &str,
    bench_pipeline_kql: &str,
) {
    rt.block_on(async {
        preview_result(bench_pipeline_kql).await;
    });

    let mut group = c.benchmark_group(bench_group_name);
    for batch_size in batch_sizes {
        let benchmark_id = BenchmarkId::new("batch_size", batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, batch_size| {
            b.iter_custom(|iters| {
                let batch = generate_logs_batch(**batch_size);
                let query = KqlParser::parse(bench_pipeline_kql).expect("can parse pipeline");
                let mut pipeline = Pipeline::new(query);
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

// used for debugging to make sure we're not just filtering empty batches
async fn preview_result(pipeline_kql: &str) {
    let batch = generate_logs_batch(20);
    let pipeline_expr = KqlParser::parse(pipeline_kql).unwrap();
    let mut pipeline = Pipeline::new(pipeline_expr);
    let result = pipeline.execute(batch).await.unwrap();

    println!("Testing output of pipeline: {}", pipeline_kql);
    for payload_type in result.allowed_payload_types() {
        println!("{:?}", payload_type);
        match result.get(*payload_type) {
            Some(rb) => arrow::util::pretty::print_batches(&[rb.clone()]).unwrap(),
            None => println!("None")
        }
    }
}

fn bench_filter_pipelines(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    let batch_sizes = [32, 1024, 8192];
    bench_pipeline(c, &rt, &batch_sizes, "simple_field_filter", "logs | where severity_text == \"WARN\"");
    bench_pipeline(c, &rt, &batch_sizes, "simple_attr_filter", "logs | where attributes[\"code.namespace\"] == \"main\"");
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