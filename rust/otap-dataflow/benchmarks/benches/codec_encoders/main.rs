// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compare native OTAP -> owned bytes at the codec-service boundary.
//!
//! Weaver fixture construction, OTLP -> OTAP conversion, input cloning, and codec
//! startup are outside timing. Encoding includes output allocation and framing,
//! but not trafficgen, network transport, or receiving-engine work. OTLP preserves
//! all input fields; syslog/CEF intentionally serialize only their mapped fields.
//! Output is detached for ownership suitable for async exporters.
//! Throughput counts input logs, not equivalent wire bytes.
//!
//! Add encoder configurations in `cases.rs`; the timing harness is codec-agnostic.
//! Run: cargo bench -p benchmarks --bench codec_encoders

#![allow(missing_docs)]

mod cases;
mod fixtures;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use otel_arrow_dfe_config::codecs::CodecsConfig;
use otel_arrow_dfe_pdata_codec::{CodecServiceBuilder, EncodePolicy, EncodingPlan};
use std::hint::black_box;

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn bench_encoder(c: &mut Criterion) {
    let encoders = cases::encoders();
    let mut group = c.benchmark_group("native_logs_encode");
    for count in [10, 100, 1000] {
        _ = group.throughput(Throughput::Elements(count as u64));
        for case in &encoders {
            let native = (case.fixture)(count);
            let service = CodecServiceBuilder::from_global_registry()
                .expect("codec registry")
                .with_configuration(CodecsConfig::resolve([&case.configuration]))
                .build()
                .expect("codec service");
            let plan =
                EncodingPlan::resolve(service.registry(), &case.encoding, EncodePolicy::default())
                    .expect("encoding plan");
            _ = group.bench_with_input(BenchmarkId::new(case.name, count), &native, |b, input| {
                let output = service
                    .encode_bytes(&mut input.clone(), &plan)
                    .expect("warm encoder");
                assert!(!output.is_empty());
                b.iter_batched_ref(
                    || input.clone(),
                    |records| {
                        black_box(
                            service
                                .encode_bytes(black_box(records), &plan)
                                .expect("encode native logs"),
                        )
                    },
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_encoder);
criterion_main!(benches);
