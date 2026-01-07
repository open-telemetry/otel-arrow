// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Memory tracking using jemalloc statistics.

use tikv_jemalloc_ctl::{epoch, stats};
use tracing::info;

/// Tracks memory usage at various checkpoints.
pub struct MemoryTracker {
    /// Baseline memory at start of tracking (after test data generation)
    baseline_bytes: u64,
    /// Peak memory observed since baseline
    peak_bytes: u64,
    /// Named checkpoints with memory values
    checkpoints: Vec<(String, u64)>,
}

impl MemoryTracker {
    /// Creates a new memory tracker.
    pub fn new() -> Self {
        Self {
            baseline_bytes: 0,
            peak_bytes: 0,
            checkpoints: Vec::new(),
        }
    }

    /// Gets current allocated memory from jemalloc.
    fn current_allocated() -> u64 {
        // Advance the epoch to get fresh stats
        if epoch::advance().is_err() {
            return 0;
        }
        stats::allocated::read().unwrap_or(0) as u64
    }

    /// Gets current allocated memory in MB (public for stress testing).
    pub fn current_allocated_mb() -> f64 {
        Self::current_allocated() as f64 / 1024.0 / 1024.0
    }

    /// Resets the baseline to current memory usage.
    ///
    /// Call this after generating test data so the baseline excludes
    /// the memory used by pre-generated Arrow batches.
    pub fn reset_baseline(&mut self) {
        self.baseline_bytes = Self::current_allocated();
        self.peak_bytes = 0;
        self.checkpoints.clear();
    }

    /// Records a named checkpoint with current memory usage.
    pub fn checkpoint(&mut self, name: &str) {
        let current = Self::current_allocated();
        let delta = current.saturating_sub(self.baseline_bytes);

        if delta > self.peak_bytes {
            self.peak_bytes = delta;
        }

        self.checkpoints.push((name.to_string(), delta));

        info!(
            checkpoint = name,
            memory_mb = format!("{:.2}", delta as f64 / 1024.0 / 1024.0),
            "Memory checkpoint"
        );
    }

    /// Records a checkpoint without logging (for high-frequency checkpoints).
    pub fn checkpoint_silent(&mut self, name: &str) {
        let current = Self::current_allocated();
        let delta = current.saturating_sub(self.baseline_bytes);

        if delta > self.peak_bytes {
            self.peak_bytes = delta;
        }

        self.checkpoints.push((name.to_string(), delta));
    }

    /// Returns peak memory usage in MB since baseline.
    pub fn peak_mb(&self) -> f64 {
        self.peak_bytes as f64 / 1024.0 / 1024.0
    }

    /// Prints a summary of memory usage across checkpoints.
    pub fn print_summary(&self) {
        info!("Memory usage summary (relative to baseline):");
        info!(
            "  Baseline: {:.2} MB",
            self.baseline_bytes as f64 / 1024.0 / 1024.0
        );
        info!("  Peak delta: {:.2} MB", self.peak_mb());
        info!("");
        info!("Key checkpoints:");

        // Find notable checkpoints to display
        let notable = [
            "engine_created",
            "after_ingestion",
            "after_shutdown",
            "segments_loaded",
            "after_consumption",
        ];

        for name in notable {
            if let Some((_, bytes)) = self.checkpoints.iter().find(|(n, _)| n == name) {
                info!("  {}: {:.2} MB", name, *bytes as f64 / 1024.0 / 1024.0);
            }
        }

        // Show memory growth during ingestion
        let ingest_checkpoints: Vec<_> = self
            .checkpoints
            .iter()
            .filter(|(n, _)| n.starts_with("ingest_"))
            .collect();

        if ingest_checkpoints.len() >= 2 {
            let first = ingest_checkpoints.first().map(|(_, b)| *b).unwrap_or(0);
            let last = ingest_checkpoints.last().map(|(_, b)| *b).unwrap_or(0);
            let growth = last.saturating_sub(first);
            info!("");
            info!(
                "Ingestion memory growth: {:.2} MB",
                growth as f64 / 1024.0 / 1024.0
            );
        }
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}
