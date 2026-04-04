// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for syslog RFC3164, RFC5424, and CEF message parsing.

/*
    The benchmark results:
    criterion = "0.5.1"

    Hardware: Apple M1 Max
    Total Number of Cores: 10 (8 performance and 2 efficiency)
    RAM: 32.0 GB
    | Test                                              | Average time |
    |---------------------------------------------------|--------------|
    | parse_auto_detect/rfc3164                         | 22.298 ns    |
    | parse_auto_detect/rfc5424                         | 34.430 ns    |
    | parse_auto_detect/cef                             | 36.681 ns    |
    | parse_auto_detect/cef_with_rfc3164                | 50.547 ns    |
    | parse_auto_detect/cef_with_rfc5424                | 61.485 ns    |
    | timestamp_extraction/rfc3164                      | 525.84 ns    |
    | timestamp_extraction/rfc5424                      | 22.971 ns    |
    | cef_extensions/one_extension                      | 20.282 ns    |
    | cef_extensions/ten_extensions                     | 188.09 ns    |
    | cef_extensions/ten_extensions_with_escape         | 207.15 ns    |
    | arrow_batch_creation/rfc3164_arrow_batch_100_msgs | 95.808 µs    |
    | arrow_batch_creation/rfc5424_arrow_batch_100_msgs | 47.463 µs    |
    | arrow_batch_creation/cef_arrow_batch_100_msgs     | 43.892 µs    |
*/

#![allow(missing_docs)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use otap_df_core_nodes::receivers::syslog_cef_receiver::arrow_records_encoder::ArrowRecordsBuilder;
use otap_df_core_nodes::receivers::syslog_cef_receiver::parser::bench_support;
use otap_df_core_nodes::receivers::syslog_cef_receiver::parser::cef::parse_cef;
use std::hint::black_box;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Static test messages for benchmarking
static RFC3164_MSG: &[u8] =
    b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
static RFC5424_MSG: &[u8] =
    b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - Message";
static CEF_MSG: &[u8] =
    b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1";
static CEF_WITH_RFC3164_MSG: &[u8] =
    b"<34>Oct 11 22:14:15 mymachine CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1";
static CEF_WITH_RFC5424_MSG: &[u8] =
    b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com app - - - CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1";
static CEF_MSG_TEN_EXT: &[u8] =
    b"CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=1233 proto=TCP act=blocked app=HTTP rt=1234567890 msg=Worm stopped cn1Label=score";
static CEF_MSG_TEN_EXT_WITH_ESCAPE: &[u8] =
    b"CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=1233 proto=TCP act=blocked app=HTTP rt=1234567890 msg=Worm\\=stopped cn1Label=score";

/// Benchmark the top-level auto-detect `parse()` function across all formats
fn bench_parse_auto_detect(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_auto_detect");

    _ = group.throughput(Throughput::Bytes(RFC3164_MSG.len() as u64));
    let _ = group.bench_function("rfc3164", |b| {
        b.iter(|| black_box(bench_support::parse(black_box(RFC3164_MSG))))
    });

    _ = group.throughput(Throughput::Bytes(RFC5424_MSG.len() as u64));
    let _ = group.bench_function("rfc5424", |b| {
        b.iter(|| black_box(bench_support::parse(black_box(RFC5424_MSG))))
    });

    _ = group.throughput(Throughput::Bytes(CEF_MSG.len() as u64));
    let _ = group.bench_function("cef", |b| {
        b.iter(|| black_box(bench_support::parse(black_box(CEF_MSG))))
    });

    _ = group.throughput(Throughput::Bytes(CEF_WITH_RFC3164_MSG.len() as u64));
    let _ = group.bench_function("cef_with_rfc3164", |b| {
        b.iter(|| black_box(bench_support::parse(black_box(CEF_WITH_RFC3164_MSG))))
    });

    _ = group.throughput(Throughput::Bytes(CEF_WITH_RFC5424_MSG.len() as u64));
    let _ = group.bench_function("cef_with_rfc5424", |b| {
        b.iter(|| black_box(bench_support::parse(black_box(CEF_WITH_RFC5424_MSG))))
    });

    group.finish();
}

/// Benchmark timestamp extraction
fn bench_timestamp_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_extraction");

    let rfc3164_parsed = bench_support::parse(RFC3164_MSG).expect("parse RFC3164");
    let _ = group.bench_function("rfc3164", |b| {
        b.iter(|| black_box(bench_support::timestamp(black_box(&rfc3164_parsed))))
    });

    let rfc5424_parsed = bench_support::parse(RFC5424_MSG).expect("parse RFC5424");
    let _ = group.bench_function("rfc5424", |b| {
        b.iter(|| black_box(bench_support::timestamp(black_box(&rfc5424_parsed))))
    });

    group.finish();
}

/// Benchmark CEF extension iteration
fn bench_cef_extensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("cef_extensions");

    let cef_parsed = parse_cef(CEF_MSG).expect("parse CEF 1 ext");
    let _ = group.bench_function("one_extension", |b| {
        b.iter(|| {
            let mut iter = bench_support::parse_extensions(&cef_parsed);
            while let Some(kv) = iter.next_extension() {
                let _ = black_box(kv);
            }
        })
    });

    let cef_ten = parse_cef(CEF_MSG_TEN_EXT).expect("parse CEF 10 ext");
    let _ = group.bench_function("ten_extensions", |b| {
        b.iter(|| {
            let mut iter = bench_support::parse_extensions(&cef_ten);
            while let Some(kv) = iter.next_extension() {
                let _ = black_box(kv);
            }
        })
    });

    let cef_ten_with_escape =
        parse_cef(CEF_MSG_TEN_EXT_WITH_ESCAPE).expect("parse CEF 10 ext with escape");
    let _ = group.bench_function("ten_extensions_with_escape", |b| {
        b.iter(|| {
            let mut iter = bench_support::parse_extensions(&cef_ten_with_escape);
            while let Some(kv) = iter.next_extension() {
                let _ = black_box(kv);
            }
        })
    });

    group.finish();
}

fn bench_arrow_batch_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arrow_batch_creation");

    // Benchmark RFC3164 Arrow batch creation with 100 messages
    let _ = group.bench_function("rfc3164_arrow_batch_100_msgs", |b| {
        b.iter(|| {
            let mut builder = ArrowRecordsBuilder::new();
            for _ in 0..100 {
                let parsed_msg = bench_support::parse(black_box(RFC3164_MSG))
                    .expect("Failed to parse RFC3164 message");
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().expect("Failed to build Arrow records");
            black_box(arrow_records)
        });
    });

    // Benchmark RFC5424 Arrow batch creation with 100 messages
    let _ = group.bench_function("rfc5424_arrow_batch_100_msgs", |b| {
        b.iter(|| {
            let mut builder = ArrowRecordsBuilder::new();
            for _ in 0..100 {
                let parsed_msg = bench_support::parse(black_box(RFC5424_MSG))
                    .expect("Failed to parse RFC5424 message");
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().expect("Failed to build Arrow records");
            black_box(arrow_records)
        });
    });

    // Benchmark CEF Arrow batch creation with 100 messages
    let _ = group.bench_function("cef_arrow_batch_100_msgs", |b| {
        b.iter(|| {
            let mut builder = ArrowRecordsBuilder::new();
            for _ in 0..100 {
                let parsed_msg =
                    bench_support::parse(black_box(CEF_MSG)).expect("Failed to parse CEF message");
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().expect("Failed to build Arrow records");
            black_box(arrow_records)
        });
    });

    group.finish();
}
criterion_group!(
    benches,
    bench_parse_auto_detect,
    bench_timestamp_extraction,
    bench_cef_extensions,
    bench_arrow_batch_creation
);
criterion_main!(benches);
