// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks OTLP.

use criterion::{Criterion, criterion_group, criterion_main};
use prost::Message;

use otap_df_pdata::proto::opentelemetry::common::v1::*;
use otap_df_pdata::proto::opentelemetry::logs::v1::*;
use otap_df_pdata::proto::opentelemetry::resource::v1::*;

fn create_logs_data() -> LogsData {
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ];
    let res = Resource::build().attributes(kvs.clone()).finish();

    let is1 = InstrumentationScope::build().name("library").finish();

    let lr1 = LogRecord::build()
        .time_unix_nano(2_000_000_000u64)
        .severity_number(SeverityNumber::Info)
        .event_name("event1")
        .attributes(kvs.clone())
        .finish();
    let lr2 = LogRecord::build()
        .time_unix_nano(3_000_000_000u64)
        .severity_number(SeverityNumber::Info2)
        .event_name("event2")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("message text"))
        .severity_text("not on fire")
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let lr3 = LogRecord::build()
        .time_unix_nano(3_000_000_000u64)
        .severity_number(SeverityNumber::Info2)
        .event_name("event3")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("here we go to 2us"))
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let mut lrs = Vec::new();
    for _ in 0..1000 {
        lrs.extend(vec![lr1.clone(), lr2.clone(), lr3.clone()]);
    }

    let sl1 =
        ScopeLogs::new(is1.clone(), lrs.clone()).set_schema_url("http://schema.opentelemetry.io");
    let sl2 = sl1.clone();
    let sls = vec![sl1, sl2];

    LogsData::new(vec![ResourceLogs::new(res, sls)])
}

fn otlp_pdata_to_bytes_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs Serialization");

    let logs = create_logs_data();

    _ = group.bench_function("LogsData Prost encode", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            let encoded = logs.encode(&mut buf);
            encoded.expect("encoding success")
        })
    });

    _ = group.bench_function("LogsData Prost decode", |b| {
        let mut enc_buf = Vec::new();
        logs.encode(&mut enc_buf).expect("expect can encode logs");

        b.iter(|| LogsData::decode(enc_buf.as_slice()).expect("expect can decode"))
    });

    _ = group.bench_function("LogsData Prost encoded_len", |b| {
        b.iter(|| logs.encoded_len())
    });

    group.finish();
}

criterion_group!(otlp_bytes, otlp_pdata_to_bytes_logs);
criterion_main!(otlp_bytes);
