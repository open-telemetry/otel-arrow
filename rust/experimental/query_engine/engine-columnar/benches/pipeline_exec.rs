// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::hint::black_box;
use std::sync::Arc;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use data_engine_columnar::engine::OtapBatchEngine;
use data_engine_kql_parser::{KqlParser, Parser};
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::execution::TaskContext;
use datafusion::physical_plan::displayable;
use datafusion::prelude::{SessionConfig, SessionContext};
use otap_df_otap::encoder::encode_logs_otap_batch;
use otap_df_otap::proto::opentelemetry::logs::v1::SeverityNumber;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
use prost::Message;

fn generate_logs_batch(batch_size: usize) -> OtapArrowRecords {
    let logs = (0..batch_size)
        .map(|i| {
            let severity_number = SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).unwrap();
            let severity_text = severity_number
                .as_str_name()
                .split("_")
                .skip(2)
                .next()
                .unwrap();
            let event_name = format!("{} happen", severity_text.to_lowercase());

            let attrs = vec![
                KeyValue::new("k8s.pod", AnyValue::new_string(format!("my-app-{}", i % 4))),
                KeyValue::new(
                    "k8s.ns",
                    AnyValue::new_string(format!(
                        "{}",
                        match i % 3 {
                            0 => "dev",
                            1 => "staging",
                            _ => "prod",
                        }
                    )),
                ),
                KeyValue::new(
                    "region",
                    AnyValue::new_string(if i > batch_size / 2 {
                        "us-east-1"
                    } else {
                        "us-west-1"
                    }),
                ),
            ];

            LogRecord::build(i as u64, severity_number, event_name)
                .severity_text(severity_text)
                .attributes(attrs)
                .finish()
        })
        .collect::<Vec<_>>();

    let log_req = ExportLogsServiceRequest::new(vec![
        ResourceLogs::build(Resource::default())
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::default())
                    .log_records(logs)
                    .finish(),
            ])
            .finish(),
    ]);

    let mut bytes = vec![];
    log_req.encode(&mut bytes).expect("can encode to vec");
    let logs_view = RawLogsData::new(&bytes);
    let otap_batch = encode_logs_otap_batch(&logs_view).expect("can convert to OTAP");

    otap_batch
}

fn bench_exec_pipelines(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("can build tokio single threaded runtime");

    // double check correct thing is happening ...
    if false {
        rt.block_on(async {
            let batch = generate_logs_batch(30);

            println!("logs:");
            let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
            arrow::util::pretty::print_batches(&[logs_rb.clone()]).unwrap();

            println!("log attrs:");
            let logs_attrs_rb = batch.get(ArrowPayloadType::LogAttrs).unwrap();
            arrow::util::pretty::print_batches(&[logs_attrs_rb.clone()]).unwrap();

            // let filter = "severity_text == \"WARN\"";
            let filter = "attributes[\"k8s.ns\"] == \"prod\"";
            let pipeline =
                KqlParser::parse(&format!("logs | where {}", filter)).expect("can parse pipeline");
            let engine = OtapBatchEngine::new();
            let result = engine
                .execute(&pipeline, &batch)
                .await
                .expect("can process result");

            println!("result logs:");
            let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();
            arrow::util::pretty::print_batches(&[logs_rb.clone()]).unwrap();

            println!("result log attrs:");
            let logs_attrs_rb = result.get(ArrowPayloadType::LogAttrs).unwrap();
            arrow::util::pretty::print_batches(&[logs_attrs_rb.clone()]).unwrap();
        });
    }


    let batch_sizes = [32, 1024, 8192];

    let mut group = c.benchmark_group("simple_field_filter");
    for batch_size in batch_sizes {
        let batch = generate_logs_batch(batch_size);
        let pipeline =
            KqlParser::parse("logs | where severity_text == \"WARN\"").expect("can parse pipeline");

        let benchmark_id = BenchmarkId::new("batch_size=", batch_size);
        let _ = group.bench_with_input(benchmark_id, &(batch, pipeline), |b, input| {
            b.to_async(&rt).iter_batched(
                || input,
                |input| async move {
                    let engine = OtapBatchEngine::new();
                    let (batch, pipeline) = &input;
                    let result = engine
                        .execute(pipeline, batch)
                        .await
                        .expect("can process result");
                    black_box(result)
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();

    let mut group = c.benchmark_group("simple_attrs_filter");
    for batch_size in batch_sizes {
        let batch = generate_logs_batch(batch_size);
        let pipeline =
            KqlParser::parse("logs | where attributes[\"k8s.ns\"] == \"prod\"").expect("can parse pipeline");

        let benchmark_id = BenchmarkId::new("batch_size=", batch_size);
        let _ = group.bench_with_input(benchmark_id, &(batch, pipeline), |b, input| {
            b.to_async(&rt).iter_batched(
                || input,
                |input| async move {
                    let engine = OtapBatchEngine::new();
                    let (batch, pipeline) = &input;
                    let result = engine
                        .execute(pipeline, batch)
                        .await
                        .expect("can process result");
                    black_box(result)
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();

    let mut group = c.benchmark_group("attrs_filter_exec_only");
    for batch_size in batch_sizes {

        let (physical_plan1, physical_plan2, task_context) = rt.block_on(async move {
            use datafusion::prelude::{lit, col};
            let session_config = SessionConfig::new()
                // .with_batch_size(8192 * 4)
                .with_target_partitions(1);
            let ctx = SessionContext::new_with_config(session_config);
            let batch1 = generate_logs_batch(batch_size);
            let (logs, log_attrs) = (batch1.get(ArrowPayloadType::Logs).unwrap(), batch1.get(ArrowPayloadType::LogAttrs).unwrap());

            let logs_table = MemTable::try_new(logs.schema(), vec![vec![logs.clone()]]).unwrap();
            let log_attrs_table = MemTable::try_new(log_attrs.schema(), vec![vec![log_attrs.clone()]]).unwrap();

            ctx.register_table("logs", Arc::new(logs_table)).unwrap();
            ctx.register_table("logattrs", Arc::new(log_attrs_table)).unwrap();

            let df = ctx.table("logs").await.unwrap()
                .join(
                    ctx.table("logattrs").await.unwrap().filter(col("key").eq(lit("k8s.ns")).and(col("str").eq(lit("prod")))).unwrap(),
                    JoinType::LeftSemi,
                    &["id"],
                    &["parent_id"],
                    None
                )
                .unwrap();

            let state = ctx.state();
            let task_ctx = Arc::new(TaskContext::from(&state));

            let logical_plan = state.optimize(df.logical_plan()).unwrap();
            let physical_plan1 = state.create_physical_plan(&logical_plan).await.unwrap();


            let df2 = ctx.table("logattrs").await.unwrap()
                .join(
                    df,
                    JoinType::LeftSemi,
                    &["parent_id"],
                    &["id"],
                    None
                ).unwrap();
            let logical_plan = state.optimize(df2.logical_plan()).unwrap();
            let physical_plan2 = state.create_physical_plan(&logical_plan).await.unwrap();

            (physical_plan1, physical_plan2, task_ctx)
        });

        let dis = displayable(physical_plan2.as_ref());
        println!("phy plan 2 {}", dis.indent(true));

        let benchmark_id = BenchmarkId::new("batch_size=", batch_size);
        let _ = group.bench_with_input(benchmark_id, &(physical_plan1, physical_plan2, task_context), |b, input| {
            b.to_async(&rt).iter_batched(
                || input,
                |input| async move {
                    let (physical_plan1, physical_plan2, task_context) = input;
                    let stream = physical_plan1.execute(0, task_context.clone()).unwrap();
                    let result = datafusion::physical_plan::common::collect(stream).await.unwrap();
                    black_box(result);

                    let stream = physical_plan2.execute(0, task_context.clone()).unwrap();
                    let result = datafusion::physical_plan::common::collect(stream).await.unwrap();
                    black_box(result)
                },
                BatchSize::SmallInput,
            );
        });


    }

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
