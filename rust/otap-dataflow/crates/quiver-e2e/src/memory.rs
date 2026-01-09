// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Memory tracking using jemalloc statistics.

use tikv_jemalloc_ctl::{epoch, stats};

/// Memory tracking utilities.
pub struct MemoryTracker;

impl MemoryTracker {
    /// Gets current allocated memory from jemalloc.
    fn current_allocated() -> u64 {
        // Advance the epoch to get fresh stats
        if epoch::advance().is_err() {
            return 0;
        }
        stats::allocated::read().unwrap_or(0) as u64
    }

    /// Gets current allocated memory in MB.
    pub fn current_allocated_mb() -> f64 {
        Self::current_allocated() as f64 / 1024.0 / 1024.0
    }
}
