// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks counting log records in an OTLP LogsData.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use otap_df_pdata::proto::opentelemetry::common::v1::*;
use otap_df_pdata::proto::opentelemetry::logs::v1::*;
use otap_df_pdata::proto::opentelemetry::resource::v1::*;

fn create_logs_data() -> LogsData {
    let mut rl: Vec<ResourceLogs> = vec![];

    for _ in 0..10 {
        let kvs = vec![
            KeyValue::new("k1", AnyValue::new_string("v1")),
            KeyValue::new("k2", AnyValue::new_string("v2")),
        ];
        let res = Resource::build().attributes(kvs.clone()).finish();
        let mut sls: Vec<ScopeLogs> = vec![];
        for _ in 0..10 {
            let is1 = InstrumentationScope::build().name("library").finish();

            let mut lrs: Vec<LogRecord> = vec![];

            for _ in 0..10 {
                let lr = LogRecord::build()
                    .time_unix_nano(2_000_000_000u64)
                    .severity_number(SeverityNumber::Info)
                    .event_name("event1")
                    .attributes(kvs.clone())
                    .finish();
                lrs.push(lr);
            }

            let sl = ScopeLogs::new(is1.clone(), lrs.clone())
                .set_schema_url("http://schema.opentelemetry.io");
            sls.push(sl);
        }
        rl.push(ResourceLogs::new(res, sls))
    }

    LogsData::new(rl)
}

fn count_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs counting");

    let logs = create_logs_data();

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
