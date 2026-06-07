// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Minimal metric counters used by the scaffolding tests.

use std::sync::atomic::{AtomicU64, Ordering};

/// Shared counters that higher layers can export through their metrics backend.
#[derive(Debug, Default)]
pub struct PersistenceMetrics {
    ingest_attempts: AtomicU64,
    flush_failures: AtomicU64,
}

impl PersistenceMetrics {
    /// Creates a metrics bundle with zeroed counters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an attempted ingest call.
    pub fn record_ingest_attempt(&self) {
        let _ = self.ingest_attempts.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the total number of ingest attempts observed.
    pub fn ingest_attempts(&self) -> u64 {
        self.ingest_attempts.load(Ordering::Relaxed)
    }

    /// Records a segment finalization (flush) failure.
    pub fn record_flush_failure(&self) {
        let _ = self.flush_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the total number of flush failures observed.
    pub fn flush_failures(&self) -> u64 {
        self.flush_failures.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_increment() {
        let metrics = PersistenceMetrics::new();
        metrics.record_ingest_attempt();
        metrics.record_ingest_attempt();
        assert_eq!(metrics.ingest_attempts(), 2);
    }
}
