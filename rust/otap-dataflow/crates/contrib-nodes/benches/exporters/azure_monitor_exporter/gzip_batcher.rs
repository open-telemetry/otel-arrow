// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark for GzipBatcher: measures time to fill a complete 1MB batch
//! across compression levels and data profiles.

#![allow(unused_results)]
#![allow(clippy::print_stderr)]

use bytes::Bytes;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_contrib_nodes::exporters::azure_monitor_exporter::{GzipBatcher, PushResult};
use rand::RngExt;

/// Pre-generate unique JSON entries of `size` bytes with the given data profile.
/// All profiles produce valid JSON to match the batcher's design envelope.
fn generate_entries(kind: &str, size: usize, count: usize) -> Vec<Vec<u8>> {
    let mut rng = rand::rng();
    (0..count)
        .map(|_| match kind {
            "json_log" => {
                // Realistic structured log: repeating keys, random values, random message.
                let id = rng.random_range(100000..999999);
                let ts = rng.random_range(1700000000u64..1700100000);
                let sev = ["DEBUG", "INFO", "WARN", "ERROR"][rng.random_range(0..4usize)];
                let base = format!(
                    r#"{{"id":{id},"timestamp":{ts},"severity":"{sev}","service":"my-service","host":"host-01","msg":""#
                );
                let closing = r#""}"#;
                let msg_len = size.saturating_sub(base.len() + closing.len());
                let msg: String = (0..msg_len)
                    .map(|_| rng.random_range(b'a'..=b'z') as char)
                    .collect();
                format!("{base}{msg}{closing}").into_bytes()
            }
            "json_hex" => {
                // Minimal JSON with random hex value.
                let base = r#"{"v":""#;
                let closing = r#""}"#;
                let hex = b"0123456789abcdef";
                let val_len = size.saturating_sub(base.len() + closing.len());
                let val: String = (0..val_len)
                    .map(|_| hex[rng.random_range(0..16usize)] as char)
                    .collect();
                format!("{base}{val}{closing}").into_bytes()
            }
            "json_ascii" => {
                // Minimal JSON with random printable ASCII value.
                let base = r#"{"v":""#;
                let closing = r#""}"#;
                let val_len = size.saturating_sub(base.len() + closing.len());
                let val: String = (0..val_len)
                    .map(|_| {
                        // Printable ASCII excluding quotes and backslashes to keep valid JSON.
                        loop {
                            let c = rng.random_range(b' '..=b'~');
                            if c != b'"' && c != b'\\' {
                                return c as char;
                            }
                        }
                    })
                    .collect();
                format!("{base}{val}{closing}").into_bytes()
            }
            _ => unreachable!(),
        })
        .collect()
}

fn bench_fill_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("gzip_batcher/fill_batch");

    for kind in ["json_log", "json_hex", "json_ascii"] {
        for entry_size in [256, 512, 1024] {
            for level in [1u32, 6, 9] {
                // Dry-run with a large pool to find how many rows fill a batch
                let pool: Vec<Bytes> = generate_entries(kind, entry_size, 10000)
                    .into_iter()
                    .map(Bytes::from)
                    .collect();
                let (rows_per_batch, compressed_size) = {
                    let mut batcher = GzipBatcher::new(level);
                    for entry in &pool {
                        match batcher.push(entry.clone()).expect("push failed") {
                            PushResult::Ok(_) => continue,
                            PushResult::BatchReady(_) => break,
                            PushResult::TooLarge => panic!("entry should fit"),
                        }
                    }
                    let batch = batcher
                        .take_pending_batch()
                        .expect("no batch after dry-run");
                    (batch.row_count as usize, batch.compressed_data.len())
                };

                // Pre-generate exactly the right amount of unique entries
                let entries: Vec<Bytes> = generate_entries(kind, entry_size, rows_per_batch)
                    .into_iter()
                    .map(Bytes::from)
                    .collect();
                let bytes_per_batch = rows_per_batch as u64 * entry_size as u64;
                let ratio = compressed_size as f64 / bytes_per_batch as f64 * 100.0;

                eprintln!(
                    "{kind}/level_{level}/{entry_size}B: {rows_per_batch} rows, \
                     uncompressed={bytes_per_batch}B, compressed={compressed_size}B, \
                     ratio={ratio:.1}%"
                );

                group.throughput(criterion::Throughput::Bytes(bytes_per_batch));
                group.bench_with_input(
                    BenchmarkId::new(format!("{kind}/level_{level}"), entry_size),
                    &entries,
                    |b, entries| {
                        b.iter_with_setup(
                            || GzipBatcher::new(level),
                            |mut batcher| {
                                for entry in entries {
                                    match batcher.push(entry.clone()).expect("push failed") {
                                        PushResult::Ok(_) => continue,
                                        PushResult::BatchReady(_) => {
                                            let batch = batcher
                                                .take_pending_batch()
                                                .expect("no pending batch");
                                            assert!(batch.row_count > 0);
                                            return;
                                        }
                                        PushResult::TooLarge => panic!("entry should fit"),
                                    }
                                }
                                // If we didn't hit BatchReady, finalize what we have
                                batcher.finalize().expect("finalize failed");
                                let batch = batcher.take_pending_batch().expect("no pending batch");
                                assert!(batch.row_count > 0);
                            },
                        );
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_fill_batch);
criterion_main!(benches);
