// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for `matches(...)` regex pipelines.
//!
//! These exercise the regex planning path with small batches (notably size 8) so the
//! cost of compiling the pattern is visible relative to the search itself: a naive
//! implementation that recompiles the pattern per batch is dominated by compilation at
//! this size, whereas compiling once at planning time is not.

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_contrib_data_engine_parser_abstractions::Parser;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
use otel_arrow_dfe_pdata::testing::round_trip::to_otap_logs;
use otel_arrow_dfe_query_engine::parser::default_parser_options;
use otel_arrow_dfe_query_engine::pipeline::Pipeline;
use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;
use tokio::runtime::Runtime;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Rotating log body templates. A fixed share match the error regex (templates with
/// error/exception/panic/fatal) or the email regex, mirroring the demo pipeline where
/// about 6% of records are errors and about 4% carry an email.
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
    "contact admin at alice@example.com for access",
    "heartbeat received from node 3",
    "fatal: segmentation fault in parser",
    "warm shutdown initiated",
    "retrying request attempt 2",
    "payload validated",
    "session expired for bob@corp.example",
    "queue depth is 17",
    "background sync ok",
];

/// Build `batch_size` log records whose body (and an attribute `msg`) cycles through
/// [`BODY_TEMPLATES`], so both the body path and the fused attribute path see the same
/// distribution of matching and non-matching strings.
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

fn bench_regex_pipeline(
    c: &mut Criterion,
    rt: &Runtime,
    group_name: &str,
    query: &str,
    batch_sizes: &[usize],
) {
    let mut group = c.benchmark_group(group_name);

    for &batch_size in batch_sizes {
        let _ = group.throughput(Throughput::Elements(batch_size as u64));
        let benchmark_id = BenchmarkId::from_parameter(batch_size);
        let _ = group.bench_with_input(benchmark_id, &batch_size, |b, &batch_size| {
            b.iter_custom(|iterations| {
                let batch = generate_logs_batch(batch_size);
                let parser_result = OplParser::parse_with_options(query, default_parser_options())
                    .expect("can parse regex pipeline");
                let mut pipeline =
                    Pipeline::try_new(parser_result.pipeline).expect("pipeline builds");

                rt.block_on(async {
                    // Warm up so query planning is excluded from the measurement.
                    _ = pipeline
                        .execute(batch.clone())
                        .await
                        .expect("regex warmup succeeds");

                    let start = Instant::now();
                    for _ in 0..iterations {
                        let result = pipeline
                            .execute(batch.clone())
                            .await
                            .expect("regex pipeline succeeds");
                        _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                })
            });
        });
    }

    group.finish();
}

fn bench_regex_pipelines(c: &mut Criterion) {
    let rt = Runtime::new().expect("can build Tokio runtime");
    // Small batches dominate: 8 is the headline case, 128/1024 show the gain shrinking as
    // the per-batch search cost grows relative to the (now avoided) per-batch compile.
    let batch_sizes = [8, 128, 1024];

    // Body path: regex applied to the log body.
    bench_regex_pipeline(
        c,
        &rt,
        "regex_filter_body",
        r#"logs | where matches(body, r"(?i)(error|exception|panic|fatal)")"#,
        &batch_sizes,
    );

    // Fused attribute path: regex applied to attributes["msg"].
    bench_regex_pipeline(
        c,
        &rt,
        "regex_filter_attr",
        r#"logs | where matches(attributes["msg"], r"(?i)(error|exception|panic|fatal)")"#,
        &batch_sizes,
    );

    // Conditional set driven by a regex match on the body (the demo transform).
    bench_regex_pipeline(
        c,
        &rt,
        "regex_conditional_set",
        r#"logs | if (matches(body, r"(?i)(error|exception|panic|fatal)")) {
            set severity_text = "ERROR"
        }"#,
        &batch_sizes,
    );
}

criterion_group!(benches, bench_regex_pipelines);
criterion_main!(benches);
