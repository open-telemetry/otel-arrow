// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node-scoped metrics for completion emission from effect handlers.
//!
//! These counters are recorded from the node point of view: they count
//! completions that a node successfully routes into the shared pipeline
//! completion lane through `notify_ack` / `notify_nack`.

use crate::entity_context::NodeTelemetryHandle;
use otap_df_config::MetricLevel;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::sync::{Arc, Mutex};

/// Node-scoped completion counters emitted from effect handlers.
#[metric_set(name = "node.completion_emission")]
#[derive(Debug, Default, Clone)]
pub(crate) struct CompletionEmissionMetrics {
    /// Count of successful `notify_ack` routings into the shared completion lane.
    #[metric(name = "notify_ack.routed", unit = "{message}")]
    pub notify_ack_routed: Counter<u64>,
    /// Count of successful `notify_nack` routings into the shared completion lane.
    #[metric(name = "notify_nack.routed", unit = "{message}")]
    pub notify_nack_routed: Counter<u64>,
}

pub(crate) struct CompletionEmissionMetricsState {
    metrics: MetricSet<CompletionEmissionMetrics>,
}

impl CompletionEmissionMetricsState {
    fn new(telemetry_handle: &NodeTelemetryHandle) -> Self {
        Self {
            metrics: telemetry_handle.register_metric_set::<CompletionEmissionMetrics>(),
        }
    }

    pub(crate) fn record_notify_ack_routed(&mut self) {
        self.metrics.notify_ack_routed.inc();
    }

    pub(crate) fn record_notify_nack_routed(&mut self) {
        self.metrics.notify_nack_routed.inc();
    }

    pub(crate) fn report(
        &mut self,
        metrics_reporter: &mut MetricsReporter,
    ) -> Result<(), TelemetryError> {
        metrics_reporter.report(&mut self.metrics)
    }

    #[cfg(test)]
    pub(crate) fn counts(&self) -> (u64, u64) {
        (
            self.metrics.notify_ack_routed.get(),
            self.metrics.notify_nack_routed.get(),
        )
    }
}

pub(crate) type CompletionEmissionMetricsHandle = Arc<Mutex<CompletionEmissionMetricsState>>;

pub(crate) fn make_completion_emission_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    level: MetricLevel,
) -> Option<CompletionEmissionMetricsHandle> {
    if level >= MetricLevel::Normal {
        telemetry_handle
            .as_ref()
            .map(|telemetry| Arc::new(Mutex::new(CompletionEmissionMetricsState::new(telemetry))))
    } else {
        None
    }
}
