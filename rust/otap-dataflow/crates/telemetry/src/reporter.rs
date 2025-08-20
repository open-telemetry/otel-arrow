// SPDX-License-Identifier: Apache-2.0

//! Metrics reporter handle.

use crate::metrics::{MvMetricsSnapshot, MultivariateMetrics, MvMetrics};

/// A sharable/clonable metrics reporter sending metrics to a `MetricsCollector`.
#[derive(Clone, Debug)]
pub struct MetricsReporter {
    /// The sender for reporting metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_sender: flume::Sender<MvMetricsSnapshot>,
}

impl MetricsReporter {
    pub(crate) fn new(metrics_sender: flume::Sender<MvMetricsSnapshot>) -> Self {
        Self { metrics_sender }
    }

    /// Report multivariate metrics and reset the metrics if successful.
    pub async fn report<M: MultivariateMetrics + Send + Sync + Clone + 'static>(
        &mut self,
        metrics: &mut MvMetrics<M>,
    ) {
        if !metrics.has_non_zero() {
            // If there are no non-zero metrics, we do not send anything.
            return;
        }

        if let Err(e) = self
            .metrics_sender
            .send_async(metrics.snapshot())
            .await
        {
            // If sending fails, we do not reset the metrics to avoid losing data.
            eprintln!("Failed to send metrics: {:?}", e);
        } else {
            metrics.zero();
        }
    }
}
