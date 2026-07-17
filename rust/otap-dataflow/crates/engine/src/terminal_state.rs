// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A set of information representing the terminal state of a node after a graceful shutdown.
//!
//! This state must include all the metrics used by the node (if any exist).

use otap_df_telemetry::metrics::MetricSetSnapshot;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Pipeline-wide deadline shared by every terminal metrics handoff.
///
/// The runtime control manager records the real shutdown deadline as soon as
/// it accepts shutdown. Error paths that terminate without a shutdown message
/// lazily establish one finite fallback. Every final reporter then uses the
/// same absolute deadline instead of receiving a fresh timeout.
#[derive(Clone, Debug, Default)]
pub(crate) struct TerminalMetricsDeadline {
    deadline: Arc<Mutex<Option<Instant>>>,
}

impl TerminalMetricsDeadline {
    const FALLBACK: Duration = Duration::from_secs(5);

    /// Records a shutdown deadline, preserving the earliest deadline observed.
    pub(crate) fn record(&self, deadline: Instant) {
        let mut current = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Some(current.map_or(deadline, |current| current.min(deadline)));
    }

    /// Returns the shared deadline, installing a finite fallback if necessary.
    pub(crate) fn get(&self) -> Instant {
        let mut deadline = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *deadline.get_or_insert_with(|| Instant::now() + Self::FALLBACK)
    }
}

/// Captures the last metric snapshots produced by a node when it terminates gracefully.
pub struct TerminalState {
    deadline: Instant,
    metrics: Vec<MetricSetSnapshot>,
}

impl TerminalState {
    /// Create a new terminal state with the provided metrics.
    pub fn new<MI>(deadline: Instant, metrics: MI) -> Self
    where
        MI: IntoIterator,
        MI::Item: Into<MetricSetSnapshot>,
    {
        Self {
            deadline,
            metrics: metrics.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the deadline by which the node must terminate.
    #[must_use]
    pub const fn deadline(&self) -> Instant {
        self.deadline
    }

    /// Returns a slice of the metric snapshots captured in this terminal state.
    #[must_use]
    pub fn metrics(&self) -> &[MetricSetSnapshot] {
        &self.metrics
    }

    /// Consumes the terminal state and returns the contained metric snapshots.
    #[must_use]
    pub fn into_metrics(self) -> Vec<MetricSetSnapshot> {
        self.metrics
    }

    /// Returns `true` when no metrics were captured.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            deadline: Instant::now().add(Duration::from_secs(1)),
            metrics: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_metrics_deadline_preserves_the_earliest_recorded_deadline() {
        let deadline = TerminalMetricsDeadline::default();
        let now = Instant::now();
        deadline.record(now + Duration::from_secs(2));
        deadline.record(now + Duration::from_secs(1));
        deadline.record(now + Duration::from_secs(3));

        assert_eq!(deadline.get(), now + Duration::from_secs(1));
        assert_eq!(deadline.clone().get(), now + Duration::from_secs(1));
    }

    #[test]
    fn terminal_metrics_deadline_installs_only_one_fallback() {
        let deadline = TerminalMetricsDeadline::default();
        let fallback = deadline.get();

        assert_eq!(deadline.get(), fallback);
        assert_eq!(deadline.clone().get(), fallback);
    }
}
