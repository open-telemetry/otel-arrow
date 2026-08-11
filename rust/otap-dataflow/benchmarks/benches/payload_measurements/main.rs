// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks cached PData measurements for OTLP and OTAP log payloads.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::OtapPayload;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::common::v1::*;
use otap_df_pdata::proto::opentelemetry::logs::v1::*;
use otap_df_pdata::proto::opentelemetry::resource::v1::*;
use otap_df_pdata::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn create_logs_data(record_count: usize) -> LogsData {
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ];
    let resource = Resource::build().attributes(kvs.clone()).finish();
    let scope = InstrumentationScope::build().name("library").finish();
    let record = LogRecord::build()
        .time_unix_nano(2_000_000_000u64)
        .severity_number(SeverityNumber::Info)
        .event_name("event1")
        .attributes(kvs)
        .finish();
    let scope_logs = ScopeLogs::new(scope, vec![record; record_count])
        .set_schema_url("http://schema.opentelemetry.io");

    LogsData::new(vec![ResourceLogs::new(resource, vec![scope_logs])])
}

fn count_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs counting");

    let logs = create_logs_data(1_000);

    _ = group.bench_function("Manual", |b| {
        b.iter(|| {
            let mut count = 0;
            for rl in &logs.resource_logs {
                for sl in &rl.scope_logs {
                    // Note! This is an optimization not available to the visitor.
                    count += sl.log_records.len();
                }
            }
            black_box(count)
        })
    });

    _ = group.bench_function("FlatMap", |b| {
        b.iter(|| {
            logs.resource_logs
                .iter()
                .flat_map(|rl| &rl.scope_logs)
                .flat_map(|sl| &sl.log_records)
                .count()
        })
    });

    group.finish();
}

fn measure_payloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData measurement overhead");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_payload: OtapPayload = otlp_message_to_bytes(&message).into();
        let otap_payload: OtapPayload = otlp_to_otap(&message).into();

        for (format, payload) in [("OTLP", otlp_payload), ("OTAP", otap_payload)] {
            let disabled = OtapPdata::new(Context::default(), payload.clone());
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/baseline/disabled"), record_count),
                &disabled,
                |b, pdata| b.iter(|| black_box(pdata.signal_type())),
            );
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/clone/cold"), record_count),
                &payload,
                |b, payload| {
                    b.iter_batched(
                        || OtapPdata::new(Context::default(), black_box(payload.clone())),
                        |pdata| black_box(pdata.clone()),
                        BatchSize::SmallInput,
                    )
                },
            );

            let clone_cached = OtapPdata::new(Context::default(), payload.clone());
            _ = black_box(clone_cached.clone());
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/clone/cached"), record_count),
                &clone_cached,
                |b, pdata| b.iter(|| black_box(pdata.clone())),
            );
            if format == "OTLP" {
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/count/cold"), record_count),
                    &payload,
                    |b, payload| {
                        b.iter_batched_ref(
                            || OtapPdata::new(Context::default(), black_box(payload.clone())),
                            |pdata| black_box(pdata.num_items()),
                            BatchSize::SmallInput,
                        )
                    },
                );
                let item_count_cached = OtapPdata::new(Context::default(), payload.clone());
                _ = black_box(item_count_cached.num_items());
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/count/cached"), record_count),
                    &item_count_cached,
                    |b, pdata| b.iter(|| black_box(pdata.num_items())),
                );
            } else {
                let item_count_direct = OtapPdata::new(Context::default(), payload.clone());
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/count/direct"), record_count),
                    &item_count_direct,
                    |b, pdata| b.iter(|| black_box(pdata.num_items())),
                );
            }

            if format == "OTAP" {
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/size/cold"), record_count),
                    &payload,
                    |b, payload| {
                        b.iter_batched_ref(
                            || OtapPdata::new(Context::default(), black_box(payload.clone())),
                            |pdata| black_box(pdata.canonical_size()),
                            BatchSize::SmallInput,
                        )
                    },
                );
                let canonical_size_cached = OtapPdata::new(Context::default(), payload.clone());
                _ = black_box(canonical_size_cached.canonical_size());
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/size/cached"), record_count),
                    &canonical_size_cached,
                    |b, pdata| b.iter(|| black_box(pdata.canonical_size())),
                );
            } else {
                let canonical_size_direct = OtapPdata::new(Context::default(), payload.clone());
                _ = group.bench_with_input(
                    BenchmarkId::new(format!("{format}/size/direct"), record_count),
                    &canonical_size_direct,
                    |b, pdata| b.iter(|| black_box(pdata.canonical_size())),
                );
            }
        }
    }

    group.finish();
}

criterion_group!(payload_measurements, count_logs, measure_payloads);
criterion_main!(payload_measurements);
