// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A set of information representing the terminal state of a node after a graceful shutdown.
//!
//! This state must include all the metrics used by the node (if any exist).

use otap_df_telemetry::metrics::MetricSetSnapshot;

pub struct TerminalState {
    metrics: Vec<MetricSetSnapshot>
}

impl TerminalState {
    /// Create a new terminal state with the provided metrics.
    pub fn new<MI>(metrics: MI) -> Self
    where MI: IntoIterator, MI::Item: Into<MetricSetSnapshot>
    {
        Self {
            metrics: metrics.into_iter().map(Into::into).collect()
        }
    }
}