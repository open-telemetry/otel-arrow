// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decode-only benchmark for the journald receiver's per-entry field decode.
//!
//! It times **only** the field-decode step ([`bench_reference_decode`] ->
//! `decode_journal_fields`) over one synthetic entry (metadata + one `MESSAGE`).
//! It is NOT an end-to-end receiver benchmark: after decode, the receiver still
//! copies each kept field into the Arrow builders, which this does not measure.
//!
//! ## Reference result — decode-only `MESSAGE`-clone removal (#3403)
//!
//! One-time local comparison (Linux, `CARGO_PROFILE_BENCH_LTO=off`) that
//! justified storing the log body as an index into `fields` instead of a second
//! owned `Vec<u8>` clone of the `MESSAGE` payload. Decode time per entry:
//!
//! ```text
//!   MESSAGE size | decode (index body) | decode + body clone (pre-#3403)
//!   -------------+---------------------+--------------------------------
//!   256 B        |             ~0.70us |  ~0.90us  (clone +28%)
//!   4 KiB        |             ~0.94us |  ~1.00us  (clone  +7%)
//!   64 KiB       |             ~2.53us |  ~4.08us  (clone +61%, -1.55us)
//! ```
//!
//! The saving is the eliminated per-entry allocation + memcpy of the `MESSAGE`
//! payload; it scales with message size (noise for small, dominant for large).
//! The `clone` arm was a validation-only comparison for this PR and is not part
//! of the long-term benchmark, which times the current decode as it ships.
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
        let _ = group.bench_with_input(BenchmarkId::from_parameter(msg_len), &raw, |b, raw| {
            b.iter(|| black_box(bench_reference_decode(black_box(raw))));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
