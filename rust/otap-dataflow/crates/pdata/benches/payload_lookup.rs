// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for PayloadDefinition column lookup.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::payload_definitions;

criterion_group!(benches, bench_lookup);
criterion_main!(benches);

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_lookup");

    // Small definition: U16_ATTRS (9 columns)
    let small_def = payload_definitions::get(ArrowPayloadType::LogAttrs);
    // Large definition: LOGS (15 top-level + 14 nested columns)
    let large_def = payload_definitions::get(ArrowPayloadType::Logs);

    // Columns to look up (hit cases)
    let small_hit_names = ["str", "key", "bytes", "type", "parent_id"];
    let large_hit_names = ["severity_text", "trace_id", "schema_url", "flags", "body"];

    // Miss cases
    let miss_names = ["nonexistent", "foo", "zzzzz"];

    // Nested lookups (LOGS body.str, resource.schema_url)
    let nested_cases = [
        ("body", "str"),
        ("resource", "schema_url"),
        ("scope", "name"),
    ];

    // --- Hit benchmarks ---

    let _ = group.bench_function(BenchmarkId::new("hit", "small_9col"), |b| {
        b.iter(|| {
            for name in &small_hit_names {
                let _ = std::hint::black_box(small_def.get(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("hit", "large_logs"), |b| {
        b.iter(|| {
            for name in &large_hit_names {
                let _ = std::hint::black_box(large_def.get(name));
            }
        });
    });

    // --- Miss benchmarks ---

    let _ = group.bench_function(BenchmarkId::new("miss", "small_9col"), |b| {
        b.iter(|| {
            for name in &miss_names {
                let _ = std::hint::black_box(small_def.get(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("miss", "large_logs"), |b| {
        b.iter(|| {
            for name in &miss_names {
                let _ = std::hint::black_box(large_def.get(name));
            }
        });
    });

    // --- Nested benchmarks ---

    let _ = group.bench_function(BenchmarkId::new("nested", "large_logs"), |b| {
        b.iter(|| {
            for (parent, child) in &nested_cases {
                let _ = std::hint::black_box(large_def.get_nested(parent, child));
            }
        });
    });

    group.finish();
}
