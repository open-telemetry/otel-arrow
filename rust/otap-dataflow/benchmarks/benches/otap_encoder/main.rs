// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks executing encoding OTAP data.
//!
//! For now, this just contains a simple benchmark for encoding logs. We'll likely add more to
//! this in the future.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use otap_df_otap::encoder::encode_logs_otap_batch;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;

fn create_logs_data() -> LogsData {
    let mut rl: Vec<ResourceLogs> = vec![];

    for _ in 0..1 {
        let mut kvs = vec![
            KeyValue::new("k1", AnyValue::new_string("v1")),
            KeyValue::new("k2", AnyValue::new_string("v2")),
        ];
        let res = Resource::new(kvs.clone());
        let mut sls: Vec<ScopeLogs> = vec![];
        for _ in 0..1 {
            let is1 = InstrumentationScope::new("library");

            let mut lrs: Vec<LogRecord> = vec![];

            for k in 0..7 {
                if k % 4 == 0 {
                    kvs.push(KeyValue::new("k3", AnyValue::new_int(1)));
                }

                if k % 5 == 0 {
                    kvs.push(KeyValue::new("k4", AnyValue::new_bool(true)));
                }

                if k % 3 == 0 {
                    kvs.push(KeyValue::new("k5", AnyValue::new_bytes(b"12341234241234")));
                }

                let lr_builder = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                    .trace_id(vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3])
                    .span_id(vec![0, 0, 1, 1, 2, 2, 3, 3]);
                let lr = if k % 2 == 0 {
                    lr_builder.attributes(kvs.clone())
                } else {
                    lr_builder
                }
                .finish();

                lrs.push(lr);
            }

            let sl = ScopeLogs::build(is1.clone())
                .log_records(lrs.clone())
                .schema_url("http://schema.opentelemetry.io")
                .finish();
            sls.push(sl);
        }
        rl.push(ResourceLogs::build(res).scope_logs(sls).finish())
    }

    LogsData::new(rl)
}

fn bench_encode_logs(c: &mut Criterion) {
    let input = create_logs_data();
    let mut group = c.benchmark_group("encode_otap_logs_using_views");
    let _ = group.bench_with_input(
        BenchmarkId::new("encode_otap_logs", "default"),
        &input,
        |b, input| {
            b.iter(|| encode_logs_otap_batch(input).expect("function no errors"));
        },
    );

    group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(benches, bench_encode_logs);
}

criterion_main!(bench_entry::benches);
