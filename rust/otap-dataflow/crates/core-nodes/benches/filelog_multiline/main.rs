// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pattern-only qualification: synchronous per-line latency, not receiver throughput.
#![allow(dead_code, missing_docs)]

mod cases;
#[path = "../../src/receivers/filelog_receiver/multiline_pattern.rs"]
mod pattern;

use std::{
    hint::black_box,
    io::{self, Write},
    time::Instant,
};

fn main() -> io::Result<()> {
    let bytes = std::env::var("FILELOG_BENCH_BYTES")
        .ok()
        .map(|s| s.parse().expect("byte count"))
        .unwrap_or(1024 * 1024);
    let samples = std::env::var("FILELOG_BENCH_SAMPLES")
        .ok()
        .map(|s| s.parse().expect("sample count"))
        .unwrap_or(7usize);
    assert!(samples > 0);
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "case,variant,cache,body_bytes,samples,median_ns,max_ns,program_bytes,max_reported_cache_bytes,worker_heap_bound"
    )?;
    for case in cases::cases(bytes) {
        let input = cases::input(&case); // Validate text once, outside every timed interval.
        for variant in cases::variants() {
            let program = cases::compile(&case, variant);
            for warm in [false, true] {
                let mut search = program.matcher();
                if warm {
                    assert!(!search.is_match(input).expect("warmup"));
                }
                let mut times = Vec::with_capacity(samples);
                let mut maximum_cache = search.cache_memory_usage();
                for _ in 0..samples {
                    if !warm {
                        search = program.matcher();
                    }
                    let begin = Instant::now();
                    let matched = black_box(search.is_match(black_box(input)).expect("search"));
                    times.push(begin.elapsed().as_nanos());
                    assert!(
                        !matched,
                        "fixture must be a full-line nonmatch or explicitly prefiltered case"
                    );
                    maximum_cache = maximum_cache.max(search.cache_memory_usage());
                }
                times.sort_unstable();
                writeln!(
                    out,
                    "{},{},{},{},{},{},{},{},{},{}",
                    case.name,
                    variant,
                    if warm { "warm" } else { "cold" },
                    case.body.len(),
                    samples,
                    times[samples / 2],
                    times[samples - 1],
                    program.program_memory_usage(),
                    maximum_cache,
                    program.memory_estimate().worker_heap_bound
                )?;
            }
        }
    }
    cases::report_cgroup()?;
    Ok(())
}
