// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the filter processor's regex matching path.
//!
//! These exercise [`LogFilter::filter`] with small batches (notably size 8) so
//! the cost of compiling the pattern is visible relative to the search itself.
//! The filter now compiles each regex once (into [`Pattern::Regex`]) rather than
//! recompiling it per batch inside arrow's `regexp_is_match_scalar`; at batch
//! size 8 a per-batch-compile implementation is dominated by compilation.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::logs::{LogFilter, LogMatchProperties};
use otel_arrow_dfe_pdata::otap::filter::{AttrMatcher, Pattern, ValueMatcher};
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
use otel_arrow_dfe_pdata::testing::round_trip::to_otap_logs;
use regex::Regex;

criterion_group!(benches, bench_filter_regex);
criterion_main!(benches);

/// Rotating log body templates, a fixed share of which match the error regex.
const BODY_TEMPLATES: &[&str] = &[
    "request completed in 12ms",
    "user logged in successfully",
    "cache hit for key user:42",
    "connection established to upstream",
    "ERROR: failed to connect to database",
    "handler returned 200 OK",
    "scheduled job finished",
    "a panic occurred while flushing buffer",
    "metrics exported to backend",
    "config reloaded from disk",
    "unhandled exception in worker thread",
    "contact admin for access",
    "heartbeat received from node 3",
    "fatal: segmentation fault in parser",
    "warm shutdown initiated",
    "retrying request attempt 2",
];

fn generate_logs_batch(batch_size: usize) -> OtapArrowRecords {
    let log_records: Vec<LogRecord> = (0..batch_size)
        .map(|index| {
            let body = BODY_TEMPLATES[index % BODY_TEMPLATES.len()];
            LogRecord::build()
                .body(AnyValue::new_string(body))
                .attributes(vec![KeyValue::new("msg", AnyValue::new_string(body))])
                .finish()
        })
        .collect();

    to_otap_logs(log_records)
}

fn error_regex() -> Pattern {
    Pattern::Regex(Regex::new(r"(?i)(error|exception|panic|fatal)").expect("valid regex"))
}

fn bench_filter(c: &mut Criterion, group_name: &str, filter: &LogFilter, batch_sizes: &[usize]) {
    let mut group = c.benchmark_group(group_name);

    for &batch_size in batch_sizes {
        let _ = group.throughput(Throughput::Elements(batch_size as u64));
        let batch = generate_logs_batch(batch_size);
        let _ = group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, _| {
                b.iter(|| {
                    let result = filter.filter(batch.clone()).expect("filter succeeds");
                    std::hint::black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_filter_regex(c: &mut Criterion) {
    // Small batches dominate: 8 is the headline case, 128/1024 show the gain
    // shrinking as the per-batch search cost grows relative to the (now avoided)
    // per-batch compile.
    let batch_sizes = [8, 128, 1024];

    // Body path: regex applied to the log body.
    let body_filter = LogFilter::new(
        Some(LogMatchProperties::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
            vec![ValueMatcher::String(error_regex())],
        )),
        None,
    );
    bench_filter(c, "filter_regex_body", &body_filter, &batch_sizes);

    // Attribute path: regex applied to attributes["msg"].
    let attr_filter = LogFilter::new(
        Some(LogMatchProperties::new(
            Vec::new(),
            vec![AttrMatcher {
                key: "msg".to_string(),
                value: ValueMatcher::String(error_regex()),
            }],
            Vec::new(),
            None,
            Vec::new(),
        )),
        None,
    );
    bench_filter(c, "filter_regex_attr", &attr_filter, &batch_sizes);
}
