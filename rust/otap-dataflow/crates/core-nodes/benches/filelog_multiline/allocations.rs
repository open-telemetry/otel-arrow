// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation qualification separated from uninstrumented timings.
#![allow(dead_code, missing_docs)]

mod cases;
#[path = "../../src/receivers/filelog_receiver/multiline_pattern.rs"]
mod pattern;
use std::io::{self, Write};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> io::Result<()> {
    let searches = std::env::var("FILELOG_BENCH_SEARCHES")
        .ok()
        .map(|s| s.parse().expect("searches"))
        .unwrap_or(3);
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "case,variant,phase,peak_live_bytes,retained_bytes,allocated_bytes,allocation_events,engine_report_bytes"
    )?;
    for case in cases::cases(
        std::env::var("FILELOG_BENCH_BYTES")
            .ok()
            .map(|s| s.parse().expect("bytes"))
            .unwrap_or(1024 * 1024),
    ) {
        let input = cases::input(&case);
        for variant in cases::variants() {
            // Fixture and output allocation are excluded from the tracked interval.
            let profiler = dhat::Profiler::builder().testing().build();
            let program = cases::compile(&case, variant);
            let stats = dhat::HeapStats::get();
            drop(profiler);
            writeln!(
                out,
                "{},{},compile,{},{},{},{},{}",
                case.name,
                variant,
                stats.max_bytes,
                stats.curr_bytes,
                stats.total_bytes,
                stats.total_blocks,
                program.program_memory_usage()
            )?;
            let profiler = dhat::Profiler::builder().testing().build();
            let mut search = program.matcher();
            for _ in 0..searches {
                assert!(!search.is_match(input).expect("nonmatch"));
            }
            let stats = dhat::HeapStats::get();
            drop(profiler);
            writeln!(
                out,
                "{},{},search,{},{},{},{},{}",
                case.name,
                variant,
                stats.max_bytes,
                stats.curr_bytes,
                stats.total_bytes,
                stats.total_blocks,
                search.cache_memory_usage()
            )?;
            assert!(
                stats.max_bytes <= program.memory_estimate().worker_heap_bound,
                "reservation must cover measured peak"
            );
        }
    }
    if std::env::var_os("FILELOG_BENCH_SKIP_COMPILE_STRESS").is_none() {
        for (name, literal_bytes) in [
            ("large_program", 200),
            ("cache_capacity_fallback", 250),
            ("program_rejection", 4000),
        ] {
            let source = format!("(?:{}){{1000}}", "a".repeat(literal_bytes));
            let profiler = dhat::Profiler::builder().testing().build();
            let result = pattern::BoundaryPattern::compile(&source, pattern::PatternMode::Raw);
            let stats = dhat::HeapStats::get();
            drop(profiler);
            if name == "cache_capacity_fallback" {
                assert!(matches!(
                    result.as_ref().expect("accepted").execution_policy(),
                    pattern::ExecutionPolicy::NoLazyDfa {
                        reason: pattern::AccelerationDisabled::CacheCapacity,
                        ..
                    }
                ));
            }
            writeln!(
                io::stderr().lock(),
                "{name}: {:?}",
                result.as_ref().map(|program| program.execution_policy())
            )?;
            writeln!(
                out,
                "{},meta,compile_{},{},{},{},{},{}",
                name,
                if result.is_ok() {
                    "accepted"
                } else {
                    "rejected"
                },
                stats.max_bytes,
                stats.curr_bytes,
                stats.total_bytes,
                stats.total_blocks,
                result.as_ref().map_or(0, |p| p.program_memory_usage())
            )?;
        }
    }
    cases::report_cgroup()?;
    Ok(())
}
