// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks executing encoding OTAP data.
//!
//! For now, this just contains a simple benchmark for encoding logs. We'll likely add more to
//! this in the future.

use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};

use otap_df_otap::encoder::encode_logs_otap_batch;
use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
use prost::Message;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn create_logs_data() -> LogsData {
    let attr_values = vec![
        AnyValue::new_string("terry"),
        AnyValue::new_bool(true),
        AnyValue::new_int(5),
        AnyValue::new_double(2.0),
        AnyValue::new_bytes(b"hi"),
        AnyValue { value: None },
        AnyValue::new_array(vec![AnyValue::new_bool(true)]),
        AnyValue::new_kvlist(vec![KeyValue::new("key1", AnyValue::new_bool(true))]),
    ];
    let mut log_attributes = attr_values
        .into_iter()
        .enumerate()
        .map(|(i, val)| KeyValue {
            key: format!("{:?}", i),
            value: Some(val),
        })
        .collect::<Vec<_>>();

    // add a 'None' attribute
    log_attributes.push(KeyValue {
        key: "noneval".to_string(),
        value: None,
    });

    LogsData::new(
        (0..5)
            .map(|_| {
                ResourceLogs::build(
                    Resource::build(vec![KeyValue::new(
                        "resource_attr1",
                        AnyValue::new_string("resource_value"),
                    )])
                    .dropped_attributes_count(1u32),
                )
                .schema_url("https://schema.opentelemetry.io/resource_schema")
                .scope_logs(
                    (0..10)
                        .map(|_| {
                            ScopeLogs::build(
                                InstrumentationScope::build("library")
                                    .version("scopev1")
                                    .attributes(vec![
                                        KeyValue::new(
                                            "scope_attr1",
                                            AnyValue::new_string("scope_val1"),
                                        ),
                                        KeyValue::new(
                                            "scope_attr2",
                                            AnyValue::new_string("scope_val2"),
                                        ),
                                        KeyValue::new(
                                            "scope_attr2",
                                            AnyValue::new_string("scope_val2"),
                                        ),
                                    ])
                                    .dropped_attributes_count(2u32)
                                    .finish(),
                            )
                            .schema_url("https://schema.opentelemetry.io/scope_schema")
                            .log_records(
                                (0..5)
                                    .map(|_| {
                                        LogRecord::build(
                                            2_000_000_000u64,
                                            SeverityNumber::Info,
                                            "event1",
                                        )
                                        .observed_time_unix_nano(3_000_000_000u64)
                                        .trace_id(vec![
                                            0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
                                        ])
                                        .span_id(vec![0, 0, 0, 0, 1, 1, 1, 1])
                                        .severity_text("Info")
                                        .attributes(log_attributes.clone())
                                        .dropped_attributes_count(3u32)
                                        .flags(LogRecordFlags::TraceFlagsMask)
                                        .body(AnyValue::new_string("log_body"))
                                        .finish()
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .finish()
                        })
                        .collect::<Vec<_>>(),
                )
                .finish()
            })
            .collect::<Vec<_>>(),
    )
}

fn bench_encode_logs(c: &mut Criterion) {
    let input = create_logs_data();
    let mut input_bytes = vec![];
    input
        .encode(&mut input_bytes)
        .expect("can encode proto bytes");

    let mut group = c.benchmark_group("encode_otap_logs_using_views");

    // 1. proto_bytes->views->OTAP
    let _ = group.bench_with_input(
        BenchmarkId::new("proto_bytes->views->OTAP", "default"),
        &input_bytes,
        |b, input| {
            b.iter_batched(
                || RawLogsData::new(input.as_ref()), // setup: create view wrapper
                |logs_data| {
                    let result = encode_logs_otap_batch(&logs_data).expect("no error");
                    black_box(result)
                },
                BatchSize::SmallInput,
            )
        },
    );

    // 2. proto_bytes->prost->views->OTAP
    let _ = group.bench_with_input(
        BenchmarkId::new("proto_bytes->prost->views->OTAP", "default"),
        &input_bytes,
        |b, input| {
            b.iter_batched(
                || input,
                |input| {
                    let logs_data =
                        LogsData::decode(input.as_ref()).expect("can decode proto bytes");
                    let result = encode_logs_otap_batch(&logs_data).expect("no error");
                    black_box(result)
                },
                BatchSize::SmallInput,
            )
        },
    );

    // 3. prost->views->OTAP
    let _ = group.bench_with_input(
        BenchmarkId::new("prost->views->OTAP", "default"),
        &input,
        |b, input| {
            b.iter_batched(
                || input, // setup: clone input (cheap clone or ref)
                |logs_data| encode_logs_otap_batch(logs_data).expect("no error"),
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();

    // Separate decode benchmark group
    let mut decode_to_prost_group = c.benchmark_group("decode_proto_bytes");

    let _ = decode_to_prost_group.bench_with_input(
        BenchmarkId::new("proto_bytes->prost", "default"),
        &input_bytes,
        |b, input| {
            b.iter_batched(
                || input.as_ref(),
                |bytes| {
                    let result = LogsData::decode(bytes).expect("can decode proto bytes");
                    black_box(result)
                },
                BatchSize::SmallInput,
            )
        },
    );

    decode_to_prost_group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(benches, bench_encode_logs);
}

criterion_main!(bench_entry::benches);
