// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for PayloadDefinition column lookup strategies.
//!
//! Compares binary search lookup (the current implementation) against
//! sequential scan to validate that binary search is adequate for the
//! definition sizes we use in practice.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::payload_definitions::{self, ColumnDef, PayloadDefinition};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

criterion_group!(benches, bench_lookup);
criterion_main!(benches);

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_lookup");

    // Small definition: U16_ATTRS (9 columns)
    let small_def = payload_definitions::get_definition(ArrowPayloadType::LogAttrs);
    // Large definition: LOGS (28 columns)
    let large_def = payload_definitions::get_definition(ArrowPayloadType::Logs);

    // Columns to look up (hit cases)
    let small_hit_names = ["str", "key", "bytes", "type", "parent_id"];
    let large_hit_names = [
        "severity_text",
        "trace_id",
        "body_ser",
        "schema_url",
        "flags",
    ];

    // Miss cases
    let miss_names = ["nonexistent", "foo", "zzzzz"];

    // Nested lookups (LOGS body.str, resource.schema_url)
    let nested_cases = [("body", "str"), ("resource", "schema_url"), ("scope", "name")];

    // --- Binary search (current implementation) ---

    let _ = group.bench_function(BenchmarkId::new("binary_search_hit", "small_9col"), |b| {
        b.iter(|| {
            for name in &small_hit_names {
                let _ = std::hint::black_box(small_def.get(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("binary_search_hit", "large_28col"), |b| {
        b.iter(|| {
            for name in &large_hit_names {
                let _ = std::hint::black_box(large_def.get(name));
            }
        });
    });

    let _ = group.bench_function(
        BenchmarkId::new("binary_search_miss", "small_9col"),
        |b| {
            b.iter(|| {
                for name in &miss_names {
                    let _ = std::hint::black_box(small_def.get(name));
                }
            });
        },
    );

    let _ = group.bench_function(
        BenchmarkId::new("binary_search_miss", "large_28col"),
        |b| {
            b.iter(|| {
                for name in &miss_names {
                    let _ = std::hint::black_box(large_def.get(name));
                }
            });
        },
    );

    let _ = group.bench_function(
        BenchmarkId::new("binary_search_nested", "large_28col"),
        |b| {
            b.iter(|| {
                for (parent, child) in &nested_cases {
                    let _ = std::hint::black_box(large_def.get_nested(parent, child));
                }
            });
        },
    );

    // --- Match statement ---

    let _ = group.bench_function(BenchmarkId::new("match_hit", "small_9col"), |b| {
        b.iter(|| {
            for name in &small_hit_names {
                let _ = std::hint::black_box(match_get_u16_attrs(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("match_hit", "large_28col"), |b| {
        b.iter(|| {
            for name in &large_hit_names {
                let _ = std::hint::black_box(match_get_logs(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("match_miss", "small_9col"), |b| {
        b.iter(|| {
            for name in &miss_names {
                let _ = std::hint::black_box(match_get_u16_attrs(name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("match_miss", "large_28col"), |b| {
        b.iter(|| {
            for name in &miss_names {
                let _ = std::hint::black_box(match_get_logs(name));
            }
        });
    });

    let _ = group.bench_function(
        BenchmarkId::new("match_nested", "large_28col"),
        |b| {
            b.iter(|| {
                for (parent, child) in &nested_cases {
                    let _ = std::hint::black_box(match_get_logs_nested(parent, child));
                }
            });
        },
    );

    // --- Linear scan ---

    let _ = group.bench_function(BenchmarkId::new("linear_scan_hit", "small_9col"), |b| {
        b.iter(|| {
            for name in &small_hit_names {
                let _ = std::hint::black_box(linear_get(small_def, name));
            }
        });
    });

    let _ = group.bench_function(BenchmarkId::new("linear_scan_hit", "large_28col"), |b| {
        b.iter(|| {
            for name in &large_hit_names {
                let _ = std::hint::black_box(linear_get(large_def, name));
            }
        });
    });

    let _ = group.bench_function(
        BenchmarkId::new("linear_scan_miss", "small_9col"),
        |b| {
            b.iter(|| {
                for name in &miss_names {
                    let _ = std::hint::black_box(linear_get(small_def, name));
                }
            });
        },
    );

    let _ = group.bench_function(
        BenchmarkId::new("linear_scan_miss", "large_28col"),
        |b| {
            b.iter(|| {
                for name in &miss_names {
                    let _ = std::hint::black_box(linear_get(large_def, name));
                }
            });
        },
    );

    let _ = group.bench_function(
        BenchmarkId::new("linear_scan_nested", "large_28col"),
        |b| {
            b.iter(|| {
                for (parent, child) in &nested_cases {
                    let _ = std::hint::black_box(linear_get_nested(large_def, parent, child));
                }
            });
        },
    );

    group.finish();
}

/// Linear scan alternative for comparison.
fn linear_get<'a>(def: &'a PayloadDefinition, name: &str) -> Option<&'a ColumnDef> {
    def.columns()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, def)| def)
}

/// Linear scan nested lookup. Checks each entry against "parent.child" by
/// comparing prefix, dot, and suffix without allocating.
fn linear_get_nested<'a>(
    def: &'a PayloadDefinition,
    parent: &str,
    child: &str,
) -> Option<&'a ColumnDef> {
    def.columns()
        .iter()
        .find(|(n, _)| {
            n.len() == parent.len() + 1 + child.len()
                && n.as_bytes().starts_with(parent.as_bytes())
                && n.as_bytes()[parent.len()] == b'.'
                && n.as_bytes()[parent.len() + 1..] == *child.as_bytes()
        })
        .map(|(_, def)| def)
}

// ---------------------------------------------------------------------------
// Match statement implementations
// ---------------------------------------------------------------------------

use otap_df_pdata::otap::payload_definitions::{MinDictKeySize, NativeType};

const fn col(native_type: NativeType) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: None,
    }
}

const fn dict(native_type: NativeType, min: MinDictKeySize) -> ColumnDef {
    ColumnDef {
        native_type,
        min_dict_key_size: Some(min),
    }
}

const U8: MinDictKeySize = MinDictKeySize::U8;
const U16: MinDictKeySize = MinDictKeySize::U16;

/// Match-based lookup for U16_ATTRS (9 columns).
fn match_get_u16_attrs(name: &str) -> Option<ColumnDef> {
    match name {
        "bool" => Some(col(NativeType::Boolean)),
        "bytes" => Some(dict(NativeType::Binary, U16)),
        "double" => Some(col(NativeType::Float64)),
        "int" => Some(dict(NativeType::Int64, U16)),
        "key" => Some(dict(NativeType::Utf8, U8)),
        "parent_id" => Some(col(NativeType::UInt16)),
        "ser" => Some(dict(NativeType::Binary, U16)),
        "str" => Some(dict(NativeType::Utf8, U16)),
        "type" => Some(col(NativeType::UInt8)),
        _ => None,
    }
}

/// Match-based nested lookup for LOGS. Matches on (parent, child) pairs.
fn match_get_logs_nested(parent: &str, child: &str) -> Option<ColumnDef> {
    match (parent, child) {
        ("body", "bool") => Some(col(NativeType::Boolean)),
        ("body", "bytes") => Some(dict(NativeType::Binary, U16)),
        ("body", "double") => Some(col(NativeType::Float64)),
        ("body", "int") => Some(dict(NativeType::Int64, U16)),
        ("body", "str") => Some(dict(NativeType::Utf8, U16)),
        ("body", "type") => Some(col(NativeType::UInt8)),
        ("resource", "dropped_attributes_count") => Some(col(NativeType::UInt32)),
        ("resource", "id") => Some(col(NativeType::UInt16)),
        ("resource", "schema_url") => Some(dict(NativeType::Utf8, U8)),
        ("scope", "dropped_attributes_count") => Some(col(NativeType::UInt32)),
        ("scope", "id") => Some(col(NativeType::UInt16)),
        ("scope", "name") => Some(dict(NativeType::Utf8, U8)),
        ("scope", "version") => Some(dict(NativeType::Utf8, U8)),
        _ => None,
    }
}

/// Match-based lookup for LOGS (28 columns, top-level names only).
fn match_get_logs(name: &str) -> Option<ColumnDef> {
    match name {
        "body" => Some(col(NativeType::Struct)),
        "body.bool" => Some(col(NativeType::Boolean)),
        "body.bytes" => Some(dict(NativeType::Binary, U16)),
        "body.double" => Some(col(NativeType::Float64)),
        "body.int" => Some(dict(NativeType::Int64, U16)),
        "body.str" => Some(dict(NativeType::Utf8, U16)),
        "body.type" => Some(col(NativeType::UInt8)),
        "body_ser" => Some(dict(NativeType::Binary, U16)),
        "dropped_attributes_count" => Some(col(NativeType::UInt32)),
        "event_name" => Some(dict(NativeType::Utf8, U8)),
        "flags" => Some(col(NativeType::UInt32)),
        "id" => Some(col(NativeType::UInt16)),
        "observed_time_unix_nano" => Some(col(NativeType::TimestampNs)),
        "resource" => Some(col(NativeType::Struct)),
        "resource.dropped_attributes_count" => Some(col(NativeType::UInt32)),
        "resource.id" => Some(col(NativeType::UInt16)),
        "resource.schema_url" => Some(dict(NativeType::Utf8, U8)),
        "schema_url" => Some(dict(NativeType::Utf8, U8)),
        "scope" => Some(col(NativeType::Struct)),
        "scope.dropped_attributes_count" => Some(col(NativeType::UInt32)),
        "scope.id" => Some(col(NativeType::UInt16)),
        "scope.name" => Some(dict(NativeType::Utf8, U8)),
        "scope.version" => Some(dict(NativeType::Utf8, U8)),
        "severity_number" => Some(dict(NativeType::Int32, U8)),
        "severity_text" => Some(dict(NativeType::Utf8, U8)),
        "span_id" => Some(dict(NativeType::FixedSizeBinary(8), U8)),
        "time_unix_nano" => Some(col(NativeType::TimestampNs)),
        "trace_id" => Some(dict(NativeType::FixedSizeBinary(16), U8)),
        _ => None,
    }
}
