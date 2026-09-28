// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoder-plus-framer throughput; excludes source I/O and receiver integration.
//!
//! ## Reference results -- testvm, 2026-09-28
//!
//! Initial baseline for decoder/framer revision `cfa9342d8`, not a comparison
//! with an older implementation. Linux x86-64, AMD EPYC 7763 VM with four logical
//! CPUs; timing pinned to guest CPU 0. Rust 1.98.1 / LLVM 22.1.8, Criterion 0.8.2.
//! Workspace bench profile: opt-level 3, fat LTO, one codegen unit.
//!
//! Median of three run means with 128-KiB input chunks; each run uses 20 samples,
//! 200-ms warm-up and a one-second measurement target. No concurrent builds or
//! tests; host isolation and CPU frequency were not controlled.
//!
//! ```text
//! Workload                    | Source MiB/s | Frames/s | Alloc/frame | Peak heap B
//! ----------------------------+--------------+----------+-------------+------------
//! utf8_short_preserve          |         51.0 |  414,536 |      16.000 |         256
//! utf8_short_replace           |         68.6 |  557,666 |       8.000 |         128
//! utf8_multibyte_preserve       |         47.3 |  341,931 |      14.000 |         320
//! utf16le_preserve              |         80.8 |  651,338 |      14.000 |         240
//! utf8_malformed_preserve       |         55.1 |  447,975 |       8.004 |         384
//! utf8_oversize_split           |         59.1 |   60,470 |      11.043 |       2,048
//! utf8_oversize_truncate        |        110.4 |    7,064 |      22.000 |       2,048
//! ```
//!
//! Short lines have 128-byte bodies. Oversized cases use 16-KiB lines and a
//! 1-KiB body limit. MiB/s counts source bytes processed, including discarded
//! truncate tails; split frames are fragments rather than whole input lines.
//! See `README.md` for the complete synthetic workload definitions.
//!
//! Alloc/frame includes reallocations, measured separately with DHAT 0.3.3.
//! Each output and its source shadow are dropped before the next framing step.
//! Peak live heap excludes the prebuilt input and profiler bookkeeping; it is
//! not RSS, a worst-case configured bound, or full receiver memory usage.
//! The short preserve-raw case's 16 allocation events/frame provide a baseline
//! for future buffer-reuse work, not a claim of a measured optimization.
//!
//! `results.csv` includes both chunk sizes and run-mean ranges. `RESULTS.md`
//! records validation limits and measured-source hashes. Rerun after changes
//! affecting implementation or benchmark behavior.
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
