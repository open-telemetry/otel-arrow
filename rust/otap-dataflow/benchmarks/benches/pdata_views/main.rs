// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for implementations of pdata view implementations

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata_views::views::common::{
    AnyValueView, AttributeView, InstrumentationScopeView, ValueType,
};
use otap_df_pdata_views::views::logs::{
    LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView,
};
use otap_df_pdata_views::views::resource::ResourceView;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
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

fn visit_any_value<'a, T>(any_value_view_impl: T)
where
    T: AnyValueView<'a>,
{
    match any_value_view_impl.value_type() {
        ValueType::Array => {
            for val in any_value_view_impl.as_array().expect("value to be array") {
                visit_any_value(val);
            }
        }
        ValueType::Bool => {
            let _ = black_box(any_value_view_impl.as_bool().expect("value to be bool"));
        }
        ValueType::Bytes => {
            let _ = black_box(any_value_view_impl.as_bytes().expect("value to be bytes"));
        }
        ValueType::Double => {
            let _ = black_box(any_value_view_impl.as_double().expect("value to be double"));
        }
        ValueType::Int64 => {
            let _ = black_box(any_value_view_impl.as_int64().expect("value to be int"));
        }
        ValueType::KeyValueList => {
            for kv in any_value_view_impl.as_kvlist().expect("value to be kvlist") {
                visit_attribute(kv);
            }
        }
        ValueType::String => {
            let _ = black_box(any_value_view_impl.as_string().expect("value ot be string"));
        }
        ValueType::Empty => {}
    }
}

fn visit_attribute<T>(attribute_view_impl: T)
where
    T: AttributeView,
{
    let _ = black_box(attribute_view_impl.key());
    if let Some(val) = attribute_view_impl.value() {
        visit_any_value(val);
    }
}

fn visit_logs_record_ordered<T>(log_record: &T)
where
    T: LogRecordView,
{
    let _ = black_box(log_record.time_unix_nano());
    let _ = black_box(log_record.severity_number());
    let _ = black_box(log_record.severity_text());
    let _ = black_box(log_record.body().map(|b| b.value_type()));
    for kv in log_record.attributes() {
        visit_attribute(kv);
    }
    let _ = black_box(log_record.dropped_attributes_count());
    let _ = black_box(log_record.flags());
    let _ = black_box(log_record.trace_id());
    let _ = black_box(log_record.span_id());
    let _ = black_box(log_record.observed_time_unix_nano());
}

/// visit every field in the logs data
fn visit_logs_data_ordered<T>(logs_view_impl: &T)
where
    T: LogsDataView,
{
    for resource_logs in logs_view_impl.resources() {
        if let Some(resource) = resource_logs.resource() {
            for kv in resource.attributes() {
                visit_attribute(kv);
            }

            let _ = black_box(resource.dropped_attributes_count());
        }

        for scope_logs in resource_logs.scopes() {
            if let Some(scope) = scope_logs.scope() {
                let _ = black_box(scope.name());
                let _ = black_box(scope.version());
                for kv in scope.attributes() {
                    visit_attribute(kv);
                }
                let _ = black_box(scope.dropped_attributes_count());
            }

            for log_record in scope_logs.log_records() {
                visit_logs_record_ordered(&log_record);
            }

            let _ = black_box(scope_logs.schema_url());
        }
        let _ = black_box(resource_logs.schema_url());
    }
}

fn visit_logs_record_unordered<T>(log_record: &T)
where
    T: LogRecordView,
{
    let _ = black_box(log_record.time_unix_nano());
    let _ = black_box(log_record.observed_time_unix_nano());
    let _ = black_box(log_record.severity_number());
    let _ = black_box(log_record.severity_text());

    let _ = black_box(log_record.trace_id());
    let _ = black_box(log_record.span_id());
    let _ = black_box(log_record.body().map(|b| b.value_type()));
    for kv in log_record.attributes() {
        visit_attribute(kv);
    }

    let _ = black_box(log_record.dropped_attributes_count());
    let _ = black_box(log_record.flags());
}

/// visit every field in the logs data, but in an order that is not the
/// order of the field messages
fn visit_logs_data_unordered<T>(logs_view_impl: &T)
where
    T: LogsDataView,
{
    for resource_logs in logs_view_impl.resources() {
        for scope_logs in resource_logs.scopes() {
            for log_record in scope_logs.log_records() {
                visit_logs_record_unordered(&log_record);
            }

            if let Some(scope) = scope_logs.scope() {
                let _ = black_box(scope.name());
                let _ = black_box(scope.version());
                for kv in scope.attributes() {
                    visit_attribute(kv);
                }
                let _ = black_box(scope.dropped_attributes_count());
            }
            let _ = black_box(scope_logs.schema_url());
        }
        let _ = black_box(resource_logs.schema_url());
        if let Some(resource) = resource_logs.resource() {
            for kv in resource.attributes() {
                visit_attribute(kv);
            }

            let _ = black_box(resource.dropped_attributes_count());
        }
    }
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
                visit_logs_data_unordered(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_structs_decode", "default"),
        &input_bytes,
        |b, input| {
            b.iter(|| {
                let logs_data = LogsData::decode(input.as_ref()).expect("can decode proto bytes");
                visit_logs_data_unordered(&logs_data);
            })
        },
    );

    let _ = group.bench_with_input(
        BenchmarkId::new("proto_struct_no_decode", "default"),
        &input,
        |b, input| {
            b.iter(|| {
                visit_logs_data_unordered(input);
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
