// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for syslog RFC3164, RFC5424, and CEF message parsing.

#![allow(missing_docs)]

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use otap_df_syslog_cef::arrow_records_encoder::ArrowRecordsBuilder;
use otap_df_syslog_cef::parser::cef::parse_cef;
use otap_df_syslog_cef::parser::parsed_message::ParsedSyslogMessage;
use otap_df_syslog_cef::parser::rfc3164::parse_rfc3164;
use otap_df_syslog_cef::parser::rfc5424::parse_rfc5424;
use std::hint::black_box;

// Static test messages for benchmarking
static RFC3164_MSG: &[u8] =
    b"<34>Oct 11 22:14:15 mymachine su: 'su root' failed for lonvick on /dev/pts/8";
static RFC5424_MSG: &[u8] =
    b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - Message";
static CEF_MSG: &[u8] =
    b"CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1";

/// Benchmark comparing all three parsers
fn bench_parser_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_comparison");

    // RFC3164 benchmark with its specific throughput
    _ = group.throughput(Throughput::Bytes(RFC3164_MSG.len() as u64));
    let _ = group.bench_function("rfc3164", |b| {
        b.iter(|| {
            let result = parse_rfc3164(black_box(RFC3164_MSG));
            black_box(result)
        });
    });

    // RFC5424 benchmark with its specific throughput
    _ = group.throughput(Throughput::Bytes(RFC5424_MSG.len() as u64));
    let _ = group.bench_function("rfc5424", |b| {
        b.iter(|| {
            let result = parse_rfc5424(black_box(RFC5424_MSG));
            black_box(result)
        });
    });

    // CEF benchmark with its specific throughput
    _ = group.throughput(Throughput::Bytes(CEF_MSG.len() as u64));
    let _ = group.bench_function("cef", |b| {
        b.iter(|| {
            let result = parse_cef(black_box(CEF_MSG));
            black_box(result)
        });
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
                let parsed = parse_rfc3164(black_box(RFC3164_MSG)).unwrap();
                let parsed_msg = ParsedSyslogMessage::Rfc3164(parsed);
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().unwrap();
            black_box(arrow_records)
        });
    });

    // Benchmark RFC5424 Arrow batch creation with 100 messages
    let _ = group.bench_function("rfc5424_arrow_batch_100_msgs", |b| {
        b.iter(|| {
            let mut builder = ArrowRecordsBuilder::new();
            for _ in 0..100 {
                let parsed = parse_rfc5424(black_box(RFC5424_MSG)).unwrap();
                let parsed_msg = ParsedSyslogMessage::Rfc5424(parsed);
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().unwrap();
            black_box(arrow_records)
        });
    });

    // Benchmark CEF Arrow batch creation with 100 messages
    let _ = group.bench_function("cef_arrow_batch_100_msgs", |b| {
        b.iter(|| {
            let mut builder = ArrowRecordsBuilder::new();
            for _ in 0..100 {
                let parsed = parse_cef(black_box(CEF_MSG)).unwrap();
                let parsed_msg = ParsedSyslogMessage::Cef(parsed);
                builder.append_syslog(parsed_msg);
            }
            let arrow_records = builder.build().unwrap();
            black_box(arrow_records)
        });
    });

    group.finish();
}
criterion_group!(benches, bench_parser_comparison, bench_arrow_batch_creation);
criterion_main!(benches);
