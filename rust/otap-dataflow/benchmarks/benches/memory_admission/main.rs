// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Micro-benchmarks for the receiver memory-admission hot path.
//!
//! Receivers consult `should_shed_ingress()` on their (per-receiver)
//! `Shared`/`Local` admission state on every ingress decision. PR1 (graduated
//! `Soft` shed) adds a `soft_action` check to that predicate; this harness
//! captures the BASELINE so the change can be held to "Normal/Hard paths
//! unchanged, Soft path adds at most one comparison" with a perf gate.
//!
//! Workloads:
//!  * `should_shed/{shared,local}/{normal,soft,hard}` — single-thread micro.
//!  * `contention/shared_read_under_flips` — N readers + 1 updater flipping
//!    Normal<->Hard via `apply()`, to surface cross-core cache traffic.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_engine::memory_limiter::{
    LocalReceiverAdmissionState, MemoryPressureChanged, MemoryPressureLevel, MemoryPressureState,
    SharedReceiverAdmissionState,
};

fn changed(level: MemoryPressureLevel, generation: u64) -> MemoryPressureChanged {
    MemoryPressureChanged {
        generation,
        level,
        retry_after_secs: 1,
        usage_bytes: 0,
    }
}

fn shared_at(level: MemoryPressureLevel) -> SharedReceiverAdmissionState {
    let s = SharedReceiverAdmissionState::from_process_state(&MemoryPressureState::default());
    s.apply(changed(level, 1));
    s
}

fn local_at(level: MemoryPressureLevel) -> LocalReceiverAdmissionState {
    let s = LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default());
    s.apply(changed(level, 1));
    s
}

const LEVELS: [(&str, MemoryPressureLevel); 3] = [
    ("normal", MemoryPressureLevel::Normal),
    ("soft", MemoryPressureLevel::Soft),
    ("hard", MemoryPressureLevel::Hard),
];

fn bench_should_shed(c: &mut Criterion) {
    let mut g = c.benchmark_group("memory_admission_should_shed");
    for (name, level) in LEVELS {
        let shared = shared_at(level);
        let _ = g.bench_function(BenchmarkId::new("shared", name), |b| {
            b.iter(|| black_box(black_box(&shared).should_shed_ingress()));
        });

        let local = local_at(level);
        let _ = g.bench_function(BenchmarkId::new("local", name), |b| {
            b.iter(|| black_box(black_box(&local).should_shed_ingress()));
        });
    }
    g.finish();
}

fn bench_contention(c: &mut Criterion) {
    let mut g = c.benchmark_group("memory_admission_contention");
    for readers in [1usize, 4, 8] {
        let _ = g.bench_function(BenchmarkId::new("shared_read_under_flips", readers), |b| {
            let shared = Arc::new(shared_at(MemoryPressureLevel::Normal));
            let stop = Arc::new(AtomicBool::new(false));

            // Background readers (besides the measured thread) to create real
            // cross-core read traffic on the shared atomics.
            let mut handles = Vec::new();
            for _ in 0..readers.saturating_sub(1) {
                let s = shared.clone();
                let stop_r = stop.clone();
                handles.push(thread::spawn(move || {
                    while !stop_r.load(Ordering::Relaxed) {
                        let _ = black_box(s.should_shed_ingress());
                    }
                }));
            }
            // One updater flipping the level (the sampler's write path).
            let su = shared.clone();
            let stop_u = stop.clone();
            let updater = thread::spawn(move || {
                let mut generation = 2u64;
                while !stop_u.load(Ordering::Relaxed) {
                    let level = if generation.is_multiple_of(2) {
                        MemoryPressureLevel::Hard
                    } else {
                        MemoryPressureLevel::Normal
                    };
                    su.apply(changed(level, generation));
                    generation += 1;
                    std::hint::spin_loop();
                }
            });

            b.iter(|| black_box(shared.should_shed_ingress()));

            stop.store(true, Ordering::Relaxed);
            updater.join().expect("updater join");
            for h in handles {
                h.join().expect("reader join");
            }
            // Small settle to avoid thread teardown bleeding into the next id.
            thread::sleep(Duration::from_millis(1));
        });
    }
    g.finish();
}

criterion_group!(benches, bench_should_shed, bench_contention);
criterion_main!(benches);
