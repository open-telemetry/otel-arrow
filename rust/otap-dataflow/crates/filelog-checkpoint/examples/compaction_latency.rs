// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Time a complete compact() call, including all syncs, without allocation profiling.

#[cfg(target_os = "linux")]
#[path = "support/compaction.rs"]
mod fixture;

#[cfg(target_os = "linux")]
fn main() -> fixture::Result<()> {
    use std::{io::Write, time::Instant};
    let fixture = fixture::Fixture::from_args()?;
    let mut store = fixture.open()?;
    let start = Instant::now();
    let receipt = store.compact()?;
    let elapsed = start.elapsed();
    assert_eq!(receipt.generation, 8);
    fixture.verify(&store)?;
    writeln!(
        std::io::stdout().lock(),
        "{},{},{},{}",
        fixture.count,
        fixture.fingerprint,
        fixture.path_bytes,
        elapsed.as_nanos()
    )?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {}
