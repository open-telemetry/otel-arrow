// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Memory tracking utilities.
//!
//! On non-Windows platforms, uses jemalloc statistics for accurate tracking.
//! On Windows, falls back to sysinfo process memory (less precise but functional).

/// Memory tracking utilities.
pub struct MemoryTracker;

#[cfg(not(windows))]
mod jemalloc_impl {
    use tikv_jemalloc_ctl::{epoch, stats};

    /// Gets current allocated memory from jemalloc.
    pub fn current_allocated() -> u64 {
        // Advance the epoch to get fresh stats
        if epoch::advance().is_err() {
            return 0;
        }
        stats::allocated::read().unwrap_or(0) as u64
    }
}

#[cfg(windows)]
mod sysinfo_impl {
    use sysinfo::{Pid, System};

    /// Gets current process memory from sysinfo.
    /// Note: This is less precise than jemalloc stats but works on Windows.
    pub fn current_allocated() -> u64 {
        let mut sys = System::new();
        let pid = Pid::from_u32(std::process::id());
        let _ = sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        sys.process(pid).map(|p| p.memory()).unwrap_or(0)
    }
}

impl MemoryTracker {
    /// Gets current allocated memory in bytes.
    fn current_allocated() -> u64 {
        #[cfg(not(windows))]
        {
            jemalloc_impl::current_allocated()
        }
        #[cfg(windows)]
        {
            sysinfo_impl::current_allocated()
        }
    }

    /// Gets current allocated memory in MB.
    pub fn current_allocated_mb() -> f64 {
        Self::current_allocated() as f64 / 1024.0 / 1024.0
    }
}
