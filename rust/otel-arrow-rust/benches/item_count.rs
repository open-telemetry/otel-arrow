// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks counting log records in an OTLP LogsData.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use otel_arrow_rust::pdata::otlp::ItemCounter;
use otel_arrow_rust::pdata::otlp::LogsVisitor;

use otel_arrow_rust::proto::opentelemetry::common::v1::*;
use otel_arrow_rust::proto::opentelemetry::logs::v1::*;
use otel_arrow_rust::proto::opentelemetry::resource::v1::*;

fn create_logs_data() -> LogsData {
    let mut rl: Vec<ResourceLogs> = vec![];

    for _ in 0..10 {
        let kvs = vec![
            KeyValue::new("k1", AnyValue::new_string("v1")),
            KeyValue::new("k2", AnyValue::new_string("v2")),
        ];
        let res = Resource::new(kvs.clone());
        let mut sls: Vec<ScopeLogs> = vec![];
        for _ in 0..10 {
            let is1 = InstrumentationScope::new("library");

            let mut lrs: Vec<LogRecord> = vec![];

            for _ in 0..10 {
                let lr = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                    .attributes(kvs.clone())
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

fn count_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs counting");

    let logs = create_logs_data();
    assert_eq!(1000, ItemCounter::default().visit_logs(&logs));

    _ = group.bench_function("Visitor", |b| {
        b.iter(|| ItemCounter::default().visit_logs(&logs))
    });

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

criterion_group!(item_count, count_logs);
criterion_main!(item_count);
