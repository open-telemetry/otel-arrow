// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for conditional pipelines with shared resource and scope parents.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_contrib_data_engine_parser_abstractions::Parser;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs,
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

fn generate_logs_batch(batch_size: usize, selectivity_percent: usize) -> OtapArrowRecords {
    let selected_count = batch_size * selectivity_percent / 100;
    let log_records: Vec<LogRecord> = (0..batch_size)
        .map(|index| {
            LogRecord::build()
                .severity_number(if index < selected_count { 20 } else { 0 })
                .finish()
        })
        .collect();

    let resource = Resource::build()
        .attributes(vec![KeyValue::new(
            "resource.id",
            AnyValue::new_string("r1"),
        )])
        .finish();
    let scope = InstrumentationScope::build()
        .attributes(vec![KeyValue::new("scope.id", AnyValue::new_string("s1"))])
        .finish();
    let logs = LogsData::new(vec![ResourceLogs::new(
        resource,
        vec![ScopeLogs::new(scope, log_records)],
    )]);

    otlp_to_otap(&OtlpProtoMessage::Logs(logs))
}

fn generate_partitioned_logs_batch(
    batch_size: usize,
    selectivity_percent: usize,
) -> OtapArrowRecords {
    const RECORDS_PER_PARENT: usize = 8;

    let selected_count = batch_size * selectivity_percent / 100;
    assert_eq!(batch_size % RECORDS_PER_PARENT, 0);
    assert_eq!(selected_count % RECORDS_PER_PARENT, 0);

    let selected_parent_count = selected_count / RECORDS_PER_PARENT;
    let resource_logs: Vec<ResourceLogs> = (0..batch_size / RECORDS_PER_PARENT)
        .map(|parent_index| {
            let severity_number = if parent_index < selected_parent_count {
                20
            } else {
                0
            };
            let log_records: Vec<LogRecord> = (0..RECORDS_PER_PARENT)
                .map(|_| LogRecord::build().severity_number(severity_number).finish())
                .collect();
            let resource = Resource::build()
                .attributes(vec![KeyValue::new(
                    "resource.id",
                    AnyValue::new_string(format!("r{parent_index}")),
                )])
                .finish();
            let scope = InstrumentationScope::build()
                .attributes(vec![KeyValue::new(
                    "scope.id",
                    AnyValue::new_string(format!("s{parent_index}")),
                )])
                .finish();

            ResourceLogs::new(resource, vec![ScopeLogs::new(scope, log_records)])
        })
        .collect();

    otlp_to_otap(&OtlpProtoMessage::Logs(LogsData::new(resource_logs)))
}

fn bench_conditional_pipeline(
    c: &mut Criterion,
    rt: &Runtime,
    group_name: &str,
    query: &str,
    batch_sizes: &[usize],
    generate_batch: fn(usize, usize) -> OtapArrowRecords,
) {
    let mut group = c.benchmark_group(group_name);

    for &batch_size in batch_sizes {
        let _ = group.throughput(Throughput::Elements(batch_size as u64));
        for selectivity_percent in [1, 50, 99] {
            let benchmark_id = BenchmarkId::new(
                format!("selectivity_{selectivity_percent}_percent"),
                batch_size,
            );
            let _ = group.bench_with_input(
                benchmark_id,
                &(batch_size, selectivity_percent),
                |b, (batch_size, selectivity_percent)| {
                    b.iter_custom(|iterations| {
                        let batch = generate_batch(*batch_size, *selectivity_percent);
                        let parser_result =
                            OplParser::parse_with_options(query, default_parser_options())
                                .expect("can parse conditional pipeline");
                        let mut pipeline = Pipeline::new(parser_result.pipeline);

                        rt.block_on(async {
                            _ = pipeline
                                .execute(batch.clone())
                                .await
                                .expect("conditional warmup succeeds");

                            let start = Instant::now();
                            for _ in 0..iterations {
                                let result = pipeline
                                    .execute(batch.clone())
                                    .await
                                    .expect("conditional pipeline succeeds");
                                _ = std::hint::black_box(result);
                            }
                            start.elapsed()
                        })
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_conditional_pipelines(c: &mut Criterion) {
    let rt = Runtime::new().expect("can build Tokio runtime");
    let shared_parent_batch_sizes = [128, 1536, 8192];

    bench_conditional_pipeline(
        c,
        &rt,
        "conditional_preserve_record_attributes",
        r#"logs | if (severity_number > 10) {
            set attributes["severe"] = "yes"
        }"#,
        &shared_parent_batch_sizes,
        generate_logs_batch,
    );
    bench_conditional_pipeline(
        c,
        &rt,
        "conditional_preserve_root_field",
        r#"logs | if (severity_number > 10) {
            set severity_text = "selected"
        }"#,
        &shared_parent_batch_sizes,
        generate_logs_batch,
    );
    bench_conditional_pipeline(
        c,
        &rt,
        "conditional_preserve_parent_read",
        r#"logs | if (severity_number > 10) {
            set severity_text = resource.attributes["resource.id"]
        }"#,
        &shared_parent_batch_sizes,
        generate_logs_batch,
    );
    bench_conditional_pipeline(
        c,
        &rt,
        "conditional_preserve_filter_parents",
        r#"logs | if (severity_number > 10) {
            where severity_number < 0
        }"#,
        &[1600],
        generate_partitioned_logs_batch,
    );
    bench_conditional_pipeline(
        c,
        &rt,
        "conditional_reindex_parents",
        r#"logs | if (severity_number > 10) {
            set resource.attributes["branch"] = "selected"
        }"#,
        &shared_parent_batch_sizes,
        generate_logs_batch,
    );
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_conditional_pipelines
    );
}

criterion_main!(benches::benches);
