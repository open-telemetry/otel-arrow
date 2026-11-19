// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Profile a complete compact() call in a single-threaded process without a test harness.

#[cfg(target_os = "linux")]
#[path = "support/compaction.rs"]
mod fixture;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(target_os = "linux")]
fn main() -> fixture::Result<()> {
    use std::io::Write;
    let fixture = fixture::Fixture::from_args()?;
    // Track store setup so the operation's live-heap peak includes its old table.
    let profiler = dhat::Profiler::builder().testing().build();
    let mut store = fixture.open()?;
    let before = dhat::HeapStats::get();
    let receipt = store.compact()?;
    let after = dhat::HeapStats::get();
    // Establish that the reported lifetime peak occurred during compact(), not setup.
    assert!(
        after.max_bytes > before.max_bytes,
        "setup obscured compaction peak"
    );
    assert_eq!(receipt.generation, 8);
    drop(profiler);
    fixture.verify(&store)?;
    writeln!(
        std::io::stdout().lock(),
        "{},{},{},{},{},{},{}",
        fixture.count,
        fixture.fingerprint,
        fixture.path_bytes,
        after.total_blocks - before.total_blocks,
        after.total_bytes - before.total_bytes,
        before.curr_bytes,
        after.max_bytes
    )?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {}
