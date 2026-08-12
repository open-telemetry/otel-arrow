// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks item-counting overhead for OTLP and OTAP log payloads.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

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

fn count_payload_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData item-count overhead");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_payload: OtapPayload = otlp_message_to_bytes(&message).into();
        let otap_payload: OtapPayload = otlp_to_otap(&message).into();

        for (format, payload) in [("OTLP", otlp_payload), ("OTAP", otap_payload)] {
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/disabled"), record_count),
                &payload,
                |b, payload| b.iter(|| black_box(payload.signal_type())),
            );
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/enabled"), record_count),
                &payload,
                |b, payload| {
                    b.iter(|| {
                        _ = black_box(payload.signal_type());
                        black_box(payload.num_items())
                    })
                },
            );
        }
    }

    group.finish();
}

criterion_group!(item_count, count_logs, count_payload_items);
criterion_main!(item_count);
