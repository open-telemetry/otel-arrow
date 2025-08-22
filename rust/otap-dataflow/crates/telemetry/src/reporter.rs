// SPDX-License-Identifier: Apache-2.0

//! Metrics reporter handle.

use crate::metrics::{MetricSetSnapshot, MetricSetHandler, MetricSet};

/// A sharable/clonable metrics reporter sending metrics to a `MetricsCollector`.
#[derive(Clone, Debug)]
pub struct MetricsReporter {
    /// The sender for reporting metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_sender: flume::Sender<MetricSetSnapshot>,
}

impl MetricsReporter {
    pub(crate) fn new(metrics_sender: flume::Sender<MetricSetSnapshot>) -> Self {
        Self { metrics_sender }
    }

    /// Report multivariate metrics and reset the metrics if successful.
    pub async fn report<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) {
        if !metrics.needs_flush() {
            return;
        }
        // ToDo (LQ) Use a try send async to avoid blocking the reporter
        if let Err(e) = self.metrics_sender.send_async(metrics.snapshot()).await {
            eprintln!("Failed to send metrics: {:?}", e);
        } else {
            metrics.clear_values();
        }
    }

    /// Send a raw metrics snapshot directly to the collector.
    /// This method is primarily for testing purposes.
    #[cfg(test)]
    pub async fn report_snapshot(
        &self,
        snapshot: MetricSetSnapshot,
    ) -> Result<(), flume::SendError<MetricSetSnapshot>> {
        self.metrics_sender.send_async(snapshot).await
    }
}
