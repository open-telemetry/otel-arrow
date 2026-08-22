// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks cached PData measurements for OTLP and OTAP payloads.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::OtapPayload;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::*;
use otel_arrow_dfe_pdata::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};

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
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);

        for format in ["OTLP", "OTAP"] {
            let fresh_payload = |format: &str| -> OtapPayload {
                match format {
                    "OTLP" => otlp_bytes.clone().into(),
                    _ => otap_records.clone().into(),
                }
            };

            let disabled = OtapPdata::new(Context::default(), fresh_payload(format));
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/disabled"), record_count),
                &disabled,
                |b, pdata| b.iter(|| black_box(pdata.signal_type())),
            );

            _ = group.bench_function(
                BenchmarkId::new(format!("{format}/clone/uncached"), record_count),
                |b| {
                    let pdata =
                        OtapPdata::new(Context::default(), black_box(fresh_payload(format)));
                    b.iter(|| black_box(pdata.clone()))
                },
            );

            let mut cached = OtapPdata::new(Context::default(), fresh_payload(format));
            _ = black_box(cached.num_items());
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/clone/cached"), record_count),
                &cached,
                |b, pdata| b.iter(|| black_box(pdata.clone())),
            );

            if format == "OTLP" {
                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/uncached"), record_count),
                    |b| {
                        b.iter_batched_ref(
                            || OtapPdata::new(Context::default(), black_box(fresh_payload(format))),
                            |pdata| black_box(pdata.num_items()),
                            BatchSize::SmallInput,
                        )
                    },
                );

                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/cached"), record_count),
                    |b| {
                        let mut pdata = cached.clone();
                        b.iter(|| black_box(pdata.num_items()))
                    },
                );
            } else {
                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/direct"), record_count),
                    |b| {
                        let mut pdata = cached.clone();
                        b.iter(|| black_box(pdata.num_items()))
                    },
                );
            }
        }
    }

    group.finish();
}

fn measure_payload_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData size overhead");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);

        _ = group.bench_function(BenchmarkId::new("OTLP/size/direct", record_count), |b| {
            let mut pdata =
                OtapPdata::new(Context::default(), black_box(otlp_bytes.clone().into()));
            b.iter(|| black_box(pdata.num_bytes()))
        });

        _ = group.bench_function(BenchmarkId::new("OTAP/size/uncached", record_count), |b| {
            b.iter_batched_ref(
                || OtapPdata::new(Context::default(), black_box(otap_records.clone().into())),
                |pdata| black_box(pdata.num_bytes()),
                BatchSize::SmallInput,
            )
        });

        let mut cached = OtapPdata::new(Context::default(), black_box(otap_records.clone().into()));
        _ = black_box(cached.num_bytes());
        _ = group.bench_function(BenchmarkId::new("OTAP/size/cached", record_count), |b| {
            b.iter(|| black_box(cached.num_bytes()))
        });
    }

    group.finish();
}

criterion_group!(
    payload_measurements,
    count_logs,
    count_payload_items,
    measure_payload_size
);
criterion_main!(payload_measurements);
