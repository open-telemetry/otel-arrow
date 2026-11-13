// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks comparing the columnar policy engine against policy-rs's
//! row-by-row engine.
//!
//! Both engines evaluate the same policies against the same logical data.
//! The columnar engine operates on a single OTAP Arrow batch while
//! policy-rs iterates records one at a time.
//!
//! Run with: cargo bench -p otel-arrow-dfe-policy-engine

use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord as OtlpLogRecord;
use otel_arrow_dfe_pdata::testing::round_trip::to_otap_logs;
use otel_arrow_dfe_policy_engine::engine::PolicyEngine;
use policy_rs::adapter::{LogRecord as AdapterLogRecord, Value};
use policy_rs::proto::tero::policy::v1::{
    AttributePath, LogField, LogMatcher, LogRename, LogTarget, LogTransform, Policy as ProtoPolicy,
    log_matcher, log_rename, policy,
};
use policy_rs::{Policy, PolicyEngine as PolicyRsEngine, PolicyRegistry, PolicySnapshot};
use tokio::runtime::Runtime;

// ---------------------------------------------------------------------------
// Data generation
// ---------------------------------------------------------------------------

const SEVERITIES: [&str; 4] = ["DEBUG", "INFO", "WARN", "ERROR"];

/// Generate an OTAP Arrow batch of log records.
///
/// Each log has:
/// - severity_text cycling through DEBUG/INFO/WARN/ERROR
/// - body = "Log message {i}"
/// - 2-5 attributes depending on the row, including:
///   - "service" = one of ["api", "web", "worker", "db"]
///   - "status_code" = cycling int (200, 404, 500, 503)
///   - optionally "old_key" = "value_to_rename" (on ~25% of rows)
///   - optionally "debug_info" = "trace data" (on ~25% of rows)
fn generate_otap_batch(batch_size: usize) -> OtapArrowRecords {
    let records: Vec<OtlpLogRecord> = (0..batch_size)
        .map(|i| {
            let severity = SEVERITIES[i % 4];
            let service = match i % 4 {
                0 => "api",
                1 => "web",
                2 => "worker",
                _ => "db",
            };
            let status_code = match i % 4 {
                0 => 200,
                1 => 404,
                2 => 500,
                _ => 503,
            };

            let mut attrs = vec![
                KeyValue::new("service", AnyValue::new_string(service)),
                KeyValue::new("status_code", AnyValue::new_int(status_code)),
            ];

            if i % 4 == 0 {
                attrs.push(KeyValue::new(
                    "old_key",
                    AnyValue::new_string("value_to_rename"),
                ));
            }
            if i % 4 == 1 {
                attrs.push(KeyValue::new(
                    "debug_info",
                    AnyValue::new_string("trace data"),
                ));
            }

            OtlpLogRecord::build()
                .severity_text(severity)
                .body(AnyValue::new_string(format!("Log message {i}")))
                .attributes(attrs)
                .finish()
        })
        .collect();

    to_otap_logs(records)
}

/// Generate adapter LogRecords for policy-rs row-by-row evaluation.
/// Same logical data as `generate_otap_batch`.
fn generate_adapter_logs(batch_size: usize) -> Vec<AdapterLogRecord> {
    (0..batch_size)
        .map(|i| {
            let severity = SEVERITIES[i % 4];
            let service = match i % 4 {
                0 => "api",
                1 => "web",
                2 => "worker",
                _ => "db",
            };
            let status_code = match i % 4 {
                0 => 200i64,
                1 => 404,
                2 => 500,
                _ => 503,
            };

            let mut log = AdapterLogRecord::new();
            log.body = Some(format!("Log message {i}"));
            log.severity_text = Some(severity.to_string());
            log.set_log_attr("service", Value::String(service.to_string()));
            log.set_log_attr("status_code", Value::Int(status_code));

            if i % 4 == 0 {
                log.set_log_attr("old_key", Value::String("value_to_rename".to_string()));
            }
            if i % 4 == 1 {
                log.set_log_attr("debug_info", Value::String("trace data".to_string()));
            }

            log
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Policy helpers
// ---------------------------------------------------------------------------

fn attr_path(key: &str) -> AttributePath {
    AttributePath {
        path: vec![key.to_string()],
    }
}

fn severity_text_exact_matcher(value: &str) -> LogMatcher {
    LogMatcher {
        field: Some(log_matcher::Field::LogField(LogField::SeverityText.into())),
        r#match: Some(log_matcher::Match::Exact(value.to_string())),
        negate: false,
        case_insensitive: false,
    }
}

fn attr_exists_matcher(key: &str) -> LogMatcher {
    LogMatcher {
        field: Some(log_matcher::Field::LogAttribute(attr_path(key))),
        r#match: Some(log_matcher::Match::Exists(true)),
        negate: false,
        case_insensitive: false,
    }
}

fn create_log_policy(
    id: &str,
    matchers: Vec<LogMatcher>,
    keep: &str,
    transform: Option<LogTransform>,
) -> Policy {
    Policy::new(ProtoPolicy {
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        target: Some(policy::Target::Log(LogTarget {
            r#match: matchers,
            keep: keep.to_string(),
            transform,
            sample_key: None,
        })),
        ..Default::default()
    })
}

/// Create the standard set of policies for throughput benchmarks:
/// - drop-debug: drop logs with severity_text == "DEBUG"
/// - rename-on-error: rename old_key -> new_key on ERROR logs
fn create_standard_policies() -> PolicySnapshot {
    let registry = PolicyRegistry::new();
    let handle = registry.register_provider();

    handle.update(vec![
        create_log_policy(
            "drop-debug",
            vec![severity_text_exact_matcher("DEBUG")],
            "none",
            None,
        ),
        create_log_policy(
            "rename-on-error",
            vec![severity_text_exact_matcher("ERROR")],
            "all",
            Some(LogTransform {
                rename: vec![LogRename {
                    from: Some(log_rename::From::FromLogAttribute(attr_path("old_key"))),
                    to: "new_key".to_string(),
                    upsert: false,
                }],
                ..Default::default()
            }),
        ),
    ]);

    registry.snapshot()
}

/// Create policies that use attribute matchers.
fn create_attribute_policies() -> PolicySnapshot {
    let registry = PolicyRegistry::new();
    let handle = registry.register_provider();

    handle.update(vec![
        create_log_policy(
            "drop-debug-info",
            vec![attr_exists_matcher("debug_info")],
            "none",
            None,
        ),
        create_log_policy(
            "keep-api",
            vec![LogMatcher {
                field: Some(log_matcher::Field::LogAttribute(attr_path("service"))),
                r#match: Some(log_matcher::Match::Exact("api".to_string())),
                negate: false,
                case_insensitive: false,
            }],
            "all",
            None,
        ),
    ]);

    registry.snapshot()
}

/// Create N policies each matching a unique severity_text regex pattern.
fn create_scaling_policies(count: usize) -> PolicySnapshot {
    let registry = PolicyRegistry::new();
    let handle = registry.register_provider();

    let policies: Vec<Policy> = (0..count)
        .map(|i| {
            create_log_policy(
                &format!("policy-{i}"),
                vec![LogMatcher {
                    field: Some(log_matcher::Field::LogField(LogField::SeverityText.into())),
                    r#match: Some(log_matcher::Match::Regex(format!("UNIQUE_PATTERN_{i}"))),
                    negate: false,
                    case_insensitive: false,
                }],
                "none",
                None,
            )
        })
        .collect();

    handle.update(policies);
    registry.snapshot()
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

/// Benchmark: full pipeline throughput with string matching + transforms.
///
/// Compares columnar engine (single batch call) against policy-rs
/// (per-record iteration).
fn bench_throughput_matching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let snapshot = create_standard_policies();

    let mut group = c.benchmark_group("throughput_matching");

    for batch_size in [100, 1000, 5000] {
        let otap_batch = generate_otap_batch(batch_size);
        let adapter_logs = generate_adapter_logs(batch_size);

        let _ = group.throughput(Throughput::Elements(batch_size as u64));

        // Columnar engine
        let _ = group.bench_with_input(
            BenchmarkId::new("columnar", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyEngine::new();
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = engine
                            .evaluate_and_transform(&snapshot, otap_batch.clone())
                            .expect("should not fail");
                        let _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                });
            },
        );

        // policy-rs row-by-row
        let _ = group.bench_with_input(
            BenchmarkId::new("policy_rs", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyRsEngine::new();
                b.to_async(&rt).iter(|| async {
                    for log in &adapter_logs {
                        let result = engine.evaluate(&snapshot, log).unwrap();
                        let _ = std::hint::black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: full pipeline with transforms.
///
/// Includes drop (keep:"none") and rename transforms.
fn bench_throughput_transforms(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let snapshot = create_standard_policies();

    let mut group = c.benchmark_group("throughput_transforms");

    for batch_size in [100, 1000, 5000] {
        let otap_batch = generate_otap_batch(batch_size);
        let adapter_logs = generate_adapter_logs(batch_size);

        let _ = group.throughput(Throughput::Elements(batch_size as u64));

        // Columnar engine
        let _ = group.bench_with_input(
            BenchmarkId::new("columnar", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyEngine::new();
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = engine
                            .evaluate_and_transform(&snapshot, otap_batch.clone())
                            .expect("should not fail");
                        let _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                });
            },
        );

        // policy-rs row-by-row with transforms
        let _ = group.bench_with_input(
            BenchmarkId::new("policy_rs", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyRsEngine::new();
                b.to_async(&rt).iter(|| async {
                    for log in &adapter_logs {
                        let mut log_clone = log.clone();
                        let result = engine
                            .evaluate_and_transform(&snapshot, &mut log_clone)
                            .unwrap();
                        let _ = std::hint::black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: attribute-based matchers.
///
/// Tests existence checks and string matching on log attributes.
fn bench_throughput_attribute_matching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let snapshot = create_attribute_policies();

    let mut group = c.benchmark_group("throughput_attribute_matching");

    for batch_size in [100, 1000, 5000] {
        let otap_batch = generate_otap_batch(batch_size);
        let adapter_logs = generate_adapter_logs(batch_size);

        let _ = group.throughput(Throughput::Elements(batch_size as u64));

        // Columnar engine
        let _ = group.bench_with_input(
            BenchmarkId::new("columnar", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyEngine::new();
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = engine
                            .evaluate_and_transform(&snapshot, otap_batch.clone())
                            .expect("should not fail");
                        let _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                });
            },
        );

        // policy-rs row-by-row
        let _ = group.bench_with_input(
            BenchmarkId::new("policy_rs", batch_size),
            &batch_size,
            |b, _| {
                let engine = PolicyRsEngine::new();
                b.to_async(&rt).iter(|| async {
                    for log in &adapter_logs {
                        let result = engine.evaluate(&snapshot, log).unwrap();
                        let _ = std::hint::black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: scaling with number of policies.
///
/// Tests how throughput changes as more policies are added. Each policy
/// has a unique regex pattern that matches nothing -- worst case for
/// Vectorscan (all patterns scanned, no matches).
fn bench_scaling_policies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let batch_size = 1000;

    let mut group = c.benchmark_group("scaling_policies");
    let _ = group.throughput(Throughput::Elements(batch_size as u64));

    for policy_count in [1, 10, 50, 100] {
        let snapshot = create_scaling_policies(policy_count);
        let otap_batch = generate_otap_batch(batch_size);
        let adapter_logs = generate_adapter_logs(batch_size);

        // Columnar engine
        let _ = group.bench_with_input(
            BenchmarkId::new("columnar", policy_count),
            &policy_count,
            |b, _| {
                let engine = PolicyEngine::new();
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _ in 0..iters {
                        let result = engine
                            .evaluate_and_transform(&snapshot, otap_batch.clone())
                            .expect("should not fail");
                        let _ = std::hint::black_box(result);
                    }
                    start.elapsed()
                });
            },
        );

        // policy-rs row-by-row
        let _ = group.bench_with_input(
            BenchmarkId::new("policy_rs", policy_count),
            &policy_count,
            |b, _| {
                let engine = PolicyRsEngine::new();
                b.to_async(&rt).iter(|| async {
                    for log in &adapter_logs {
                        let result = engine.evaluate(&snapshot, log).unwrap();
                        let _ = std::hint::black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_throughput_matching,
    bench_throughput_transforms,
    bench_throughput_attribute_matching,
    bench_scaling_policies,
);
criterion_main!(benches);
