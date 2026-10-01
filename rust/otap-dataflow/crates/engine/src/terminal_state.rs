// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A set of information representing the terminal state of a node after a graceful shutdown.
//!
//! This state must include all the metrics used by the node (if any exist).

use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default)]
enum DeadlineMode {
    #[default]
    Earliest,
    HostOwned,
}

/// Shutdown deadline policy shared by terminal metrics reporters.
///
/// Pipelines retain the earliest recorded deadline and share one error fallback.
/// Scope hosts own their phase deadline; individual terminal deadlines and
/// pre-shutdown error fallbacks never change that shared phase budget.
#[derive(Clone, Debug, Default)]
pub(crate) struct TerminalMetricsDeadline {
    deadline: Arc<Mutex<Option<Instant>>>,
    mode: DeadlineMode,
}

impl TerminalMetricsDeadline {
    const FALLBACK: Duration = Duration::from_secs(5);

    /// Creates a deadline that only explicit host shutdown can establish.
    pub(crate) fn host_owned() -> Self {
        Self {
            mode: DeadlineMode::HostOwned,
            ..Self::default()
        }
    }

    /// Records an explicit shutdown deadline.
    ///
    /// Pipelines preserve the earliest value; hosts fix the first phase deadline.
    pub(crate) fn record(&self, deadline: Instant) {
        let mut current = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Some(match self.mode {
            DeadlineMode::Earliest => current.map_or(deadline, |current| current.min(deadline)),
            DeadlineMode::HostOwned => current.unwrap_or(deadline),
        });
    }

    /// Resolves one terminal state's reporting deadline.
    ///
    /// A host-owned deadline is only read here: a peer may limit its own reporting,
    /// but cannot change the budget for other peers or the host's final flush.
    pub(crate) fn for_report(&self, terminal_deadline: Instant) -> Instant {
        match self.mode {
            DeadlineMode::Earliest => {
                self.record(terminal_deadline);
                self.get()
            }
            DeadlineMode::HostOwned => self.get().min(terminal_deadline),
        }
    }

    /// Returns the phase deadline or a finite error-reporting fallback.
    ///
    /// Host fallbacks stay local to the reporting operation: early failures
    /// must not start the shutdown clock for still-running peer providers.
    pub(crate) fn get(&self) -> Instant {
        let mut deadline = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match self.mode {
            DeadlineMode::Earliest => {
                *deadline.get_or_insert_with(|| Instant::now() + Self::FALLBACK)
            }
            DeadlineMode::HostOwned => deadline.unwrap_or_else(|| Instant::now() + Self::FALLBACK),
        }
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

    /// Scenario: a pipeline receives several explicit shutdown deadlines.
    /// Guarantees: every clone retains the earliest deadline rather than extending cleanup.
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

    /// Scenario: pipeline error reporting starts without an explicit shutdown deadline.
    /// Guarantees: all pipeline reporters share a single finite fallback.
    #[test]
    fn terminal_metrics_deadline_installs_only_one_fallback() {
        let deadline = TerminalMetricsDeadline::default();
        let fallback = deadline.get();

        assert_eq!(deadline.get(), fallback);
        assert_eq!(deadline.clone().get(), fallback);
    }

    /// Scenario: pipeline terminal states return different deadlines after shutdown starts.
    /// Guarantees: terminal reports retain the existing pipeline-wide earliest-deadline behavior.
    #[test]
    fn pipeline_terminal_reports_preserve_the_earliest_deadline() {
        let deadline = TerminalMetricsDeadline::default();
        let now = Instant::now();
        deadline.record(now + Duration::from_secs(5));
        let earlier = now + Duration::from_secs(1);
        assert_eq!(deadline.for_report(earlier), earlier);
        assert_eq!(deadline.get(), earlier);
        assert_eq!(deadline.for_report(now + Duration::from_secs(10)), earlier);
    }

    /// Scenario: hosted extensions return earlier and later terminal deadlines within one phase.
    /// Guarantees: each report is bounded locally while peers and repeated signals preserve the host deadline.
    #[test]
    fn host_terminal_reports_cannot_change_the_phase_deadline() {
        let deadline = TerminalMetricsDeadline::host_owned();
        let now = Instant::now();
        let phase_deadline = now + Duration::from_secs(5);
        deadline.record(phase_deadline);
        let earlier = now + Duration::from_secs(1);
        assert_eq!(deadline.for_report(earlier), earlier);
        assert_eq!(deadline.get(), phase_deadline);
        assert_eq!(deadline.clone().for_report(phase_deadline), phase_deadline);
        assert_eq!(
            deadline.for_report(phase_deadline + Duration::from_secs(5)),
            phase_deadline
        );
        deadline.record(earlier);
        deadline.record(phase_deadline + Duration::from_secs(5));
        assert_eq!(deadline.get(), phase_deadline);
    }

    /// Scenario: a hosted provider fails before the host has entered its shutdown phase.
    /// Guarantees: error reporting stays bounded without installing a deadline for peers or later shutdown.
    #[test]
    fn host_error_fallback_does_not_establish_the_phase_deadline() {
        let deadline = TerminalMetricsDeadline::host_owned();
        let now = Instant::now();
        let expired = now - Duration::from_secs(1);
        assert_eq!(deadline.for_report(expired), expired);
        let fallback = deadline.get();
        assert!(fallback >= now + TerminalMetricsDeadline::FALLBACK);
        assert!(fallback <= Instant::now() + TerminalMetricsDeadline::FALLBACK);
        assert!(
            deadline
                .deadline
                .lock()
                .expect("deadline mutex should not be poisoned")
                .is_none()
        );
        let phase_deadline = fallback + Duration::from_secs(5);
        deadline.record(phase_deadline);
        assert_eq!(deadline.get(), phase_deadline);
    }
}
