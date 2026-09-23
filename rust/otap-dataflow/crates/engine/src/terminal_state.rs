// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A set of information representing the terminal state of a node after a graceful shutdown.
//!
//! This state must include all the metrics used by the node (if any exist).

use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Pipeline-wide deadline shared by every terminal metrics handoff.
///
/// The runtime control manager records the real shutdown deadline as soon as
/// it accepts shutdown. Error paths that terminate without a shutdown message
/// lazily establish one finite fallback. Every final reporter then uses the
/// same absolute deadline instead of receiving a fresh timeout.
#[derive(Clone, Debug)]
pub(crate) struct TerminalMetricsDeadline {
    deadline: Arc<Mutex<Option<Instant>>>,
    shutdown_deadline: watch::Sender<Option<Instant>>,
}

impl Default for TerminalMetricsDeadline {
    fn default() -> Self {
        Self {
            deadline: Arc::default(),
            shutdown_deadline: watch::channel(None).0,
        }
    }
}

impl TerminalMetricsDeadline {
    const FALLBACK: Duration = Duration::from_secs(5);

    /// Records a terminal metrics deadline, preserving the earliest deadline observed.
    pub(crate) fn record(&self, deadline: Instant) {
        let mut current = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *current = Some(current.map_or(deadline, |current| current.min(deadline)));
    }

    /// Records a pipeline shutdown deadline for both cancellation and terminal metrics.
    pub(crate) fn record_shutdown(&self, deadline: Instant) {
        self.record(deadline);
        self.shutdown_deadline.send_modify(|current| {
            *current = Some(current.map_or(deadline, |current| current.min(deadline)));
        });
    }

    /// Returns the shared deadline, installing a finite fallback if necessary.
    pub(crate) fn get(&self) -> Instant {
        let mut deadline = self
            .deadline
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *deadline.get_or_insert_with(|| Instant::now() + Self::FALLBACK)
    }

    /// Waits for an explicit shutdown deadline, ignoring terminal metrics fallback deadlines.
    pub(crate) async fn expired(&self) {
        let mut updates = self.shutdown_deadline.subscribe();
        loop {
            let deadline = *updates.borrow_and_update();
            if let Some(deadline) = deadline {
                tokio::select! {
                    biased;
                    _ = crate::clock::sleep_until(deadline) => return,
                    _ = updates.changed() => {}
                }
            } else {
                let _ = updates.changed().await;
            }
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

    /// Scenario: Metric fallback and node terminal deadlines precede a pipeline shutdown request.
    /// Guarantees: Only the explicit pipeline shutdown deadline wakes cancellation waiters.
    #[tokio::test(start_paused = true)]
    async fn deadline_waiters_observe_shutdown() {
        let deadline = TerminalMetricsDeadline::default();
        let _ = deadline.get();
        deadline.record(tokio::time::Instant::now().into_std() + Duration::from_secs(1));
        let first = deadline.clone();
        let second = deadline.clone();
        let first = tokio::spawn(async move { first.expired().await });
        let second = tokio::spawn(async move { second.expired().await });
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(60)).await;
        assert!(!first.is_finished());
        assert!(!second.is_finished());
        deadline.record_shutdown(tokio::time::Instant::now().into_std() + Duration::from_secs(2));
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(2)).await;
        first.await.expect("first waiter completes");
        second.await.expect("second waiter completes");
    }

    /// Scenario: An existing shutdown deadline is shortened while a waiter is sleeping.
    /// Guarantees: The waiter observes the earlier deadline instead of the original timeout.
    #[tokio::test(start_paused = true)]
    async fn deadline_waiter_observes_shortened_deadline() {
        let deadline = TerminalMetricsDeadline::default();
        let now = tokio::time::Instant::now();
        deadline.record_shutdown((now + Duration::from_secs(60)).into_std());
        let waiter = deadline.clone();
        let waiter = tokio::spawn(async move { waiter.expired().await });
        tokio::task::yield_now().await;
        deadline.record_shutdown((now + Duration::from_secs(2)).into_std());
        tokio::time::timeout(Duration::from_secs(3), waiter)
            .await
            .expect("shortened deadline is honored")
            .expect("waiter completes");
        assert!(tokio::time::Instant::now() < now + Duration::from_secs(3));
    }

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
