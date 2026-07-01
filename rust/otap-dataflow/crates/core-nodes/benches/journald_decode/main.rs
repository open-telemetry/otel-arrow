// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark proving the per-entry `MESSAGE`-clone elimination in the journald
//! receiver's field decode.
//!
//! It compares two shapes of the *real* decode over an identical synthetic
//! entry (see [`bench_reference_decode`]):
//!   * `index` — the current path: the log body is an index into the decoded
//!     fields, so the `MESSAGE` payload is stored once per entry.
//!   * `clone` — the pre-change path, reproduced by additionally cloning the
//!     `MESSAGE` body value per entry (`message_body = Some(value.clone())`).
//!
//! The delta between the two groups is exactly the eliminated clone; it scales
//! with `MESSAGE` size (negligible for small messages, large for big ones).
//!
//! Run: `cargo bench -p otap-df-core-nodes --features bench`

#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otap_df_core_nodes::receivers::journald_receiver::bench_reference_decode;

/// A realistic entry: the metadata journald always attaches plus one `MESSAGE`
/// whose size is the workload variable.
fn build_entry(message_len: usize) -> Vec<Vec<u8>> {
    let mut v: Vec<Vec<u8>> = vec![
        b"_PID=4321".to_vec(),
        b"_UID=1000".to_vec(),
        b"_GID=1000".to_vec(),
        b"_COMM=nginx".to_vec(),
        b"_HOSTNAME=web-01".to_vec(),
        b"_SYSTEMD_UNIT=nginx.service".to_vec(),
        b"SYSLOG_IDENTIFIER=nginx".to_vec(),
        b"PRIORITY=6".to_vec(),
        b"_BOOT_ID=1e2d3c4b5a69788796a5b4c3d2e1f000".to_vec(),
    ];
    let mut msg = b"MESSAGE=".to_vec();
    msg.extend(std::iter::repeat_n(b'x', message_len));
    v.push(msg);
    v
}

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("journald_decode");
    for msg_len in [256usize, 4 * 1024, 64 * 1024] {
        let owned = build_entry(msg_len);
        let raw: Vec<&[u8]> = owned.iter().map(Vec::as_slice).collect();
        let _ = group.throughput(Throughput::Bytes(msg_len as u64));
        let _ = group.bench_with_input(BenchmarkId::new("index", msg_len), &raw, |b, raw| {
            b.iter(|| black_box(bench_reference_decode(black_box(raw), false)));
        });
        let _ = group.bench_with_input(BenchmarkId::new("clone", msg_len), &raw, |b, raw| {
            b.iter(|| black_box(bench_reference_decode(black_box(raw), true)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
