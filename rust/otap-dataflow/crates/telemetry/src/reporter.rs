// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics reporter handle.

use crate::error::Error;
use crate::metrics::{MetricSet, MetricSetHandler, MetricSetSnapshot};

/// Outcome of attempting to publish a metrics snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportOutcome {
    /// The snapshot was accepted by the reporting channel.
    Sent,
    /// The reporting channel was full, so the caller should retry later.
    Deferred,
}

/// A sharable/clonable metrics reporter sending metrics to a `MetricsCollector`.
#[derive(Clone, Debug)]
pub struct MetricsReporter {
    /// The sender for reporting metrics.
    /// The message is a tuple of (MetricSetKey, MultivariateMetrics).
    /// The metrics key is the aggregation key for the metrics,
    metrics_sender: flume::Sender<MetricSetSnapshot>,
}

impl MetricsReporter {
    /// Creates a new `MetricsReporter` with the given metrics sender channel.
    ///
    /// This is primarily used for testing purposes or when you need to create
    /// a reporter independently of the `MetricsSystem`.
    #[must_use]
    pub const fn new(metrics_sender: flume::Sender<MetricSetSnapshot>) -> Self {
        Self { metrics_sender }
    }

    /// Create a test instance of this metrics reporter. It returns the reporter and the receiver
    /// that the reporter will send the metrics to when report is called
    #[must_use]
    pub fn create_new_and_receiver(
        channel_size: usize,
    ) -> (flume::Receiver<MetricSetSnapshot>, Self) {
        let (tx, rx) = flume::bounded(channel_size);
        (rx, Self::new(tx))
    }

    /// Report multivariate metrics and reset the metrics if successful.
    pub fn report<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<(), Error> {
        let _ = self.report_with_outcome(metrics)?;
        Ok(())
    }

    /// Report multivariate metrics and return whether the snapshot was sent or deferred.
    pub fn report_with_outcome<M: MetricSetHandler + 'static>(
        &mut self,
        metrics: &mut MetricSet<M>,
    ) -> Result<ReportOutcome, Error> {
        if !metrics.needs_flush() {
            return Ok(ReportOutcome::Sent);
        }
        let metrics_snapshot = metrics.snapshot();
        match self.metrics_sender.try_send(metrics_snapshot) {
            Ok(_) => {
                // Successfully sent, reset the metrics
                metrics.clear_values();
                Ok(ReportOutcome::Sent)
            }
            Err(flume::TrySendError::Full(_)) => {
                // Channel is full, so we don't block the reporter, we don't reset the metrics
                // and we will try again later.
                // ToDo: We might have to change this behavior depending on how we apply timestamps to these snapshots otherwise we may have a thread report a larger value belonging to a longer interval than others.
                Ok(ReportOutcome::Deferred)
            }
            Err(flume::TrySendError::Disconnected(_)) => Err(Error::MetricsChannelClosed),
        }
    }

    /// Report an already materialized snapshot.
    pub fn try_report_snapshot(&self, snapshot: MetricSetSnapshot) -> Result<(), Error> {
        let _ = self.try_report_snapshot_with_outcome(snapshot)?;
        Ok(())
    }

    /// Report an already materialized snapshot and indicate whether it was sent or deferred.
    pub fn try_report_snapshot_with_outcome(
        &self,
        snapshot: MetricSetSnapshot,
    ) -> Result<ReportOutcome, Error> {
        match self.metrics_sender.try_send(snapshot) {
            Ok(_) => Ok(ReportOutcome::Sent),
            Err(flume::TrySendError::Full(_)) => Ok(ReportOutcome::Deferred),
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
