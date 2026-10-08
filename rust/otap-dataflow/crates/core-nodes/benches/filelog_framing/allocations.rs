// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Separate heap measurements: never compare instrumented timings with Criterion.

#![allow(missing_docs)]

use std::io::{self, Write};

mod cases;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() -> io::Result<()> {
    let mut output = io::stdout().lock();
    let fixtures = cases::cases();
    writeln!(
        output,
        "case,source_bytes,frames,body_bytes,allocation_events,allocated_bytes,peak_live_bytes,allocations_per_frame"
    )?;
    for case in fixtures {
        // Corpus allocation and CSV output are outside the measured interval.
        let profiler = dhat::Profiler::builder().testing().build();
        let result = cases::scan(&case, cases::READ_CHUNK);
        let stats = dhat::HeapStats::get();
        drop(profiler);
        cases::validate(&case, &result);
        writeln!(
            output,
            "{},{},{},{},{},{},{},{:.4}",
            case.name,
            case.source.len(),
            result.frames,
            result.body_bytes,
            stats.total_blocks,
            stats.total_bytes,
            stats.max_bytes,
            stats.total_blocks as f64 / result.frames as f64
        )?;
    }
    Ok(())
}
