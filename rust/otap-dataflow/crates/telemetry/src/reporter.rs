// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics reporter handle.

use crate::error::Error;
use crate::metrics::{MetricSet, MetricSetHandler, MetricSetSnapshot};

/// A sharable/clonable metrics reporter sending metrics to a `MetricsCollector`.
#[derive(Clone, Debug)]
pub struct MetricsReporter {
    /// The sender for reporting metrics.
    /// The message is a tuple of (MetricsKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_sender: flume::Sender<MetricSetSnapshot>,
}

impl MetricsReporter {
    /// Creates a new `MetricsReporter` with the given metrics sender channel.
    ///
    /// This is primarily used for testing purposes or when you need to create
    /// a reporter independently of the `MetricsSystem`.
    #[must_use]
    pub fn new(metrics_sender: flume::Sender<MetricSetSnapshot>) -> Self {
        Self { metrics_sender }
    }

    /// Report multivariate metrics and reset the metrics if successful.
    pub async fn report<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), Error> {
        if !metrics.needs_flush() {
            return Ok(());
        }
        match self.metrics_sender.try_send(metrics.snapshot()) {
            Ok(_) => {
                // Successfully sent, reset the metrics
                metrics.clear_values();
                Ok(())
            }
            Err(flume::TrySendError::Full(_)) => {
                // Channel is full, so we don't block the reporter, we don't reset the metrics
                // and we will try again later.
                Ok(())
            }
            Err(flume::TrySendError::Disconnected(_)) => Err(Error::MetricsChannelClosed),
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
