// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A set of information representing the terminal state of a node after a graceful shutdown.
//!
//! This state must include all the metrics used by the node (if any exist).

use otap_df_telemetry::metrics::MetricSetSnapshot;
use std::ops::Add;
use std::time::{Duration, Instant};

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
    pub fn deadline(&self) -> Instant {
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
    pub fn is_empty(&self) -> bool {
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
