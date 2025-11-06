// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for implementations of pdata view implementations

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::views::bench_helpers::{visit_logs_data, visit_logs_data_ordered};
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use prost::Message;

/// creates a log data with every field present in the proto message
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
            key: format!("{i:?}"),
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
                ResourceLogs::new(
                    Resource::build()
                        .attributes(vec![KeyValue::new(
                            "resource_attr1",
                            AnyValue::new_string("resource_value"),
                        )])
                        .dropped_attributes_count(1u32),
                    (0..10)
                        .map(|_| {
                            ScopeLogs::new(
                                InstrumentationScope::build()
                                    .name("library")
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
                                (0..5)
                                    .map(|_| {
                                        LogRecord::build()
                                            .time_unix_nano(2_000_000_000u64)
                                            .severity_number(SeverityNumber::Info)
                                            .event_name("event1")
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
                            .set_schema_url("https://schema.opentelemetry.io/scope_schema")
                        })
                        .collect::<Vec<_>>(),
                )
                .set_schema_url("https://schema.opentelemetry.io/resource_schema")
            })
            .collect::<Vec<_>>(),
    )
}

fn bench_logs_view_impl_ordered(c: &mut Criterion) {
    let input = create_logs_data();
    let mut input_bytes = vec![];
    input
        .encode(&mut input_bytes)
        .expect("can decode proto bytes");

    let mut group = c.benchmark_group("bench_logs_view_impl_ordered");

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_bytes", "default"),
        &input_bytes,
        |b, input| {
            b.iter(|| {
                let logs_data = RawLogsData::new(input);
                visit_logs_data_ordered(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_structs_decode", "default"),
        &input_bytes,
        |b, input| {
            b.iter(|| {
                let logs_data = LogsData::decode(input.as_ref()).expect("can decode proto bytes");
                visit_logs_data_ordered(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_struct_no_decode", "default"),
        &input,
        |b, input| {
            b.iter(|| {
                visit_logs_data_ordered(input);
            })
        },
    );

    group.finish();
}

fn bench_logs_view_impl_unordered(c: &mut Criterion) {
    let input = create_logs_data();
    let mut input_bytes = vec![];
    input
        .encode(&mut input_bytes)
        .expect("can encode proto bytes");

    let mut group = c.benchmark_group("bench_logs_view_impl_unordered");

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_bytes", "default"),
        &input_bytes,
        |b, input| {
            b.iter(|| {
                let logs_data = RawLogsData::new(input);
                visit_logs_data(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_structs_decode", "default"),
        &input_bytes,
        |b, input| {
            b.iter(|| {
                let logs_data = LogsData::decode(input.as_ref()).expect("can decode proto bytes");
                visit_logs_data(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_struct_no_decode", "default"),
        &input,
        |b, input| {
            b.iter(|| {
                visit_logs_data(input);
            })
        },
    );

    group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_logs_view_impl_ordered, bench_logs_view_impl_unordered,
    );
}

criterion_main!(bench_entry::benches);
