// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoder-plus-framer throughput; excludes source I/O and receiver integration.
//!
//! 2026-10-01, Linux Xeon 6973P-C VM, Rust 1.98.1, system allocator:
//! 128-byte UTF-8 lines, 128-KiB chunks: 94.88 MiB/s preserve-raw,
//! 121.21 MiB/s replace (source bytes; median of three run means, pinned CPU).
//!
//! Run: `cargo bench --locked -p otel-arrow-dfe-core-nodes --bench filelog_framing`

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{hint::black_box, time::Duration};

mod cases;

fn bench_framing(c: &mut Criterion) {
    let mut group = c.benchmark_group("filelog_framing");
    let _ = group.sample_size(20);
    let _ = group.warm_up_time(Duration::from_millis(200));
    let _ = group.measurement_time(Duration::from_secs(1));
    for case in cases::cases() {
        for chunk_size in [17, cases::READ_CHUNK] {
            cases::validate(&case, &cases::scan(&case, chunk_size));
            let _ = group.throughput(Throughput::Bytes(case.source.len() as u64));
            let _ = group.bench_with_input(
                BenchmarkId::new(case.name, chunk_size),
                &chunk_size,
                |b, chunk_size| b.iter(|| black_box(cases::scan(black_box(&case), *chunk_size))),
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_framing);
criterion_main!(benches);
