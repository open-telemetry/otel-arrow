// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native process memory observations for the allocator selected by the binary.

use otel_arrow_dfe_engine::engine_metrics::AllocatorMemoryProvider;
#[cfg(any(feature = "mimalloc", all(not(windows), feature = "jemalloc")))]
use otel_arrow_dfe_engine::engine_metrics::{Allocator, AllocatorMemoryStats};

pub(crate) fn provider(name: &str) -> Option<AllocatorMemoryProvider> {
    match name {
        #[cfg(all(not(windows), feature = "jemalloc"))]
        "jemalloc" => Some(AllocatorMemoryProvider {
            allocator: Allocator::Jemalloc,
            read: jemalloc_memory,
        }),
        #[cfg(feature = "mimalloc")]
        "mimalloc" => Some(AllocatorMemoryProvider {
            allocator: Allocator::Mimalloc,
            read: mimalloc_memory,
        }),
        _ => None,
    }
}

#[cfg(all(not(windows), feature = "jemalloc"))]
fn jemalloc_memory() -> AllocatorMemoryStats {
    use tikv_jemalloc_ctl::{epoch, stats};

    if epoch::advance().is_err() {
        return AllocatorMemoryStats::default();
    }
    AllocatorMemoryStats {
        allocated: stats::allocated::read().ok().map(|v| v as u64),
        active: stats::active::read().ok().map(|v| v as u64),
        resident: stats::resident::read().ok().map(|v| v as u64),
        mapped: stats::mapped::read().ok().map(|v| v as u64),
        retained: stats::retained::read().ok().map(|v| v as u64),
        ..Default::default()
    }
}

#[cfg(feature = "mimalloc")]
fn mimalloc_memory() -> AllocatorMemoryStats {
    let Some(stats) = mimalloc::stats_json().and_then(|stats| {
        serde_json::from_slice::<serde_json::Value>(stats.as_c_str().to_bytes()).ok()
    }) else {
        return AllocatorMemoryStats::default();
    };
    AllocatorMemoryStats {
        // OS reservation and commit are accounted process-wide. The allocated
        // tally can omit blocks held by workers that have not merged stats.
        committed: stats["committed"]["current"].as_u64(),
        reserved: stats["reserved"]["current"].as_u64(),
        reserved_peak: stats["reserved"]["peak"].as_u64(),
        ..Default::default()
    }
}
