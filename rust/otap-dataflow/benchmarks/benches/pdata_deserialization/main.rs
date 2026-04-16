// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for OtapPdata deserialization: `OtlpProtoBytes → OtapArrowRecords`.
//!
//! This measures the cost of `payload.try_into::<OtapArrowRecords>()` — the
//! deserialization step that occurs inside every processor before the timed
//! transform work.  This cost is excluded from per-processor
//! `compute.success.duration` metrics but included in the processor chain's
//! composite duration timer.
//!
//! The conversion only does real work when the payload is `OtlpBytes` (the
//! first processor to touch a batch from an OTLP receiver).  Subsequent
//! processors receive `OtapArrowRecords` already, so their `try_into()` is
//! a no-op enum unwrap.
//!
//! ## Results (100-log batch)
//!
//! | Batch size | Time |
//! |---|---|
//! | 10 logs | ~56µs |
//! | 100 logs | ~115µs |
//! | 1000 logs | ~1.0ms |
//!
//! The 100-log result (~115µs) explains the ~190µs gap between the processor
//! chain's composite duration and its sub-processor sum: one deserialization
//! (~120µs) plus send + bookkeeping (~70µs).

#![allow(missing_docs)]

use std::hint::black_box;

use bytes::BytesMut;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_pdata::OtapPayload;
use otap_df_pdata::OtlpProtoBytes;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use prost::Message;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn create_logs_data(num_logs: usize) -> LogsData {
    let log_attributes: Vec<KeyValue> = (0..8)
        .map(|i| {
            KeyValue::new(
                format!("attr_{i}"),
                AnyValue::new_string(format!("val_{i}")),
            )
        })
        .collect();

    LogsData::new(vec![ResourceLogs::new(
        Resource::build()
            .attributes(vec![KeyValue::new(
                "service.name",
                AnyValue::new_string("bench-svc"),
            )])
            .dropped_attributes_count(0u32),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name("bench-lib")
                .version("1.0")
                .finish(),
            (0..num_logs)
                .map(|_| {
                    LogRecord::build()
                        .time_unix_nano(1_000_000_000u64)
                        .observed_time_unix_nano(1_000_000_000u64)
                        .severity_number(SeverityNumber::Info)
                        .severity_text("Info")
                        .body(AnyValue::new_string("benchmark log message body"))
                        .attributes(log_attributes.clone())
                        .trace_id(vec![1u8; 16])
                        .span_id(vec![2u8; 8])
                        .flags(LogRecordFlags::TraceFlagsMask)
                        .finish()
                })
                .collect::<Vec<LogRecord>>(),
        )],
    )])
}

fn encode_to_proto_bytes(logs: &LogsData) -> bytes::Bytes {
    let mut buf = BytesMut::new();
    logs.encode(&mut buf).unwrap();
    buf.freeze()
}

fn bench_pdata_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdata_deserialization");

    for num_logs in [10, 100, 1000] {
        let logs = create_logs_data(num_logs);
        let proto_bytes = encode_to_proto_bytes(&logs);

        let _ = group.throughput(Throughput::Elements(num_logs as u64));

        // Benchmark: OtlpProtoBytes → OtapArrowRecords (the full deserialization path)
        let _ = group.bench_with_input(
            BenchmarkId::new("otlp_to_otap", num_logs),
            &proto_bytes,
            |b, proto_bytes| {
                b.iter_batched(
                    || {
                        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
                            proto_bytes.clone(),
                        ))
                    },
                    |payload| {
                        let records: OtapArrowRecords = black_box(payload).try_into().unwrap();
                        black_box(records)
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_pdata_deserialization);
criterion_main!(benches);
