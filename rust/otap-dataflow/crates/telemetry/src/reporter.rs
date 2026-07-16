// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics reporter handle.

use crate::error::Error;
use crate::metrics::{MetricSet, MetricSetHandler, MetricSetSnapshot};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedRwLockReadGuard, RwLock, oneshot, watch};
use tokio::time::Instant as TokioInstant;

/// Finite fallback for terminal callers that do not carry a shutdown deadline.
const DEFAULT_RELIABLE_REPORT_TIMEOUT: Duration = Duration::from_secs(5);

/// Uses the existing shutdown error surface to avoid expanding the public error enum.
fn reporting_deadline_exceeded() -> Error {
    Error::ShutdownError("metrics reporting deadline exceeded".to_owned())
}

/// Ordered messages consumed by the production internal metrics collector.
#[derive(Debug)]
pub(crate) enum MetricsCollectionMessage {
    /// A metric-set snapshot to aggregate into the registry.
    Snapshot(MetricSetSnapshot),
    /// A FIFO barrier acknowledged after every preceding snapshot is aggregated.
    Flush(oneshot::Sender<()>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetricsCollectorStatus {
    NotStarted,
    Running,
    Stopped,
}

/// Handle used to establish a finite collection barrier with the metrics collector.
///
/// A successful [`Self::flush`] guarantees that every snapshot accepted before
/// the barrier marker has been aggregated into the telemetry registry. Snapshots
/// accepted later remain queued for the next collection window.
#[derive(Clone, Debug)]
pub(crate) struct MetricsFlushHandle {
    metrics_sender: flume::Sender<MetricsCollectionMessage>,
    collector_status: watch::Receiver<MetricsCollectorStatus>,
    shutdown_gate: Arc<RwLock<()>>,
}

impl MetricsFlushHandle {
    pub(crate) const fn new(
        metrics_sender: flume::Sender<MetricsCollectionMessage>,
        collector_status: watch::Receiver<MetricsCollectorStatus>,
        shutdown_gate: Arc<RwLock<()>>,
    ) -> Self {
        Self {
            metrics_sender,
            collector_status,
            shutdown_gate,
        }
    }

    fn running_status(&self) -> Result<watch::Receiver<MetricsCollectorStatus>, Error> {
        let mut collector_status = self.collector_status.clone();
        let status = *collector_status.borrow_and_update();
        match status {
            MetricsCollectorStatus::Running => Ok(collector_status),
            MetricsCollectorStatus::NotStarted | MetricsCollectorStatus::Stopped => {
                Err(Error::MetricsCollectorNotRunning)
            }
        }
    }

    async fn send_message(
        &self,
        message: MetricsCollectionMessage,
    ) -> Result<
        (
            watch::Receiver<MetricsCollectorStatus>,
            OwnedRwLockReadGuard<()>,
        ),
        Error,
    > {
        let shutdown_guard = self.shutdown_gate.clone().read_owned().await;
        let mut collector_status = self.running_status()?;
        let send_barrier = self.metrics_sender.send_async(message);
        tokio::pin!(send_barrier);

        loop {
            tokio::select! {
                result = &mut send_barrier => {
                    result.map_err(|_| Error::MetricsChannelClosed)?;
                    return Ok((collector_status, shutdown_guard));
                }
                status = collector_status.changed() => {
                    if status.is_err()
                        || *collector_status.borrow_and_update() != MetricsCollectorStatus::Running
                    {
                        return Err(Error::MetricsCollectorNotRunning);
                    }
                }
            }
        }
    }

    async fn send_snapshot(&self, snapshot: MetricSetSnapshot) -> Result<ReportOutcome, Error> {
        let status = *self.collector_status.borrow();
        match status {
            MetricsCollectorStatus::NotStarted => self.try_send_snapshot(snapshot),
            MetricsCollectorStatus::Running => {
                let (_collector_status, _shutdown_guard) = self
                    .send_message(MetricsCollectionMessage::Snapshot(snapshot))
                    .await?;
                Ok(ReportOutcome::Sent)
            }
            MetricsCollectorStatus::Stopped => Err(Error::MetricsCollectorNotRunning),
        }
    }

    fn try_send_snapshot(&self, snapshot: MetricSetSnapshot) -> Result<ReportOutcome, Error> {
        // Cancellation queues for the write side of this lock before its final
        // drain. Tokio's fair RwLock then rejects new non-blocking readers, so
        // `Sent` always means the shutdown drain will observe this snapshot.
        let _shutdown_guard = self
            .shutdown_gate
            .clone()
            .try_read_owned()
            .map_err(|_| Error::MetricsCollectorNotRunning)?;
        if *self.collector_status.borrow() == MetricsCollectorStatus::Stopped {
            return Err(Error::MetricsCollectorNotRunning);
        }
        match self
            .metrics_sender
            .try_send(MetricsCollectionMessage::Snapshot(snapshot))
        {
            Ok(()) => Ok(ReportOutcome::Sent),
            Err(flume::TrySendError::Full(_)) => Ok(ReportOutcome::Deferred),
            Err(flume::TrySendError::Disconnected(_)) => Err(Error::MetricsChannelClosed),
        }
    }

    /// Waits for the collector to process all snapshots preceding this call.
    pub(crate) async fn flush(&self) -> Result<(), Error> {
        let (ack_sender, mut ack_receiver) = oneshot::channel();
        let (mut collector_status, _shutdown_guard) = self
            .send_message(MetricsCollectionMessage::Flush(ack_sender))
            .await?;

        loop {
            tokio::select! {
                biased;

                result = &mut ack_receiver => {
                    return result.map_err(|_| Error::MetricsCollectorNotRunning);
                }
                status = collector_status.changed() => {
                    if status.is_err()
                        || *collector_status.borrow_and_update() != MetricsCollectorStatus::Running
                    {
                        return Err(Error::MetricsCollectorNotRunning);
                    }
                }
            }
        }
    }
}

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
    metrics_sender: MetricsSender,
}

#[derive(Clone, Debug)]
enum MetricsSender {
    /// Standalone channel used by component tests and embedders.
    Direct(flume::Sender<MetricSetSnapshot>),
    /// Ordered production channel shared with collector flush barriers.
    Collector { flusher: MetricsFlushHandle },
}

impl MetricsReporter {
    /// Creates a new `MetricsReporter` with the given metrics sender channel.
    ///
    /// This is primarily used for testing purposes or when you need to create
    /// a reporter independently of the `MetricsSystem`.
    #[must_use]
    pub const fn new(metrics_sender: flume::Sender<MetricSetSnapshot>) -> Self {
        Self {
            metrics_sender: MetricsSender::Direct(metrics_sender),
        }
    }

    pub(crate) const fn new_collector(flusher: MetricsFlushHandle) -> Self {
        Self {
            metrics_sender: MetricsSender::Collector { flusher },
        }
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
        match self.try_report_snapshot_with_outcome(metrics_snapshot)? {
            ReportOutcome::Sent => {
                // Successfully sent, reset the metrics
                metrics.clear_values();
                Ok(ReportOutcome::Sent)
            }
            ReportOutcome::Deferred => {
                // Channel is full, so we don't block the reporter, we don't reset the metrics
                // and we will try again later.
                // ToDo: We might have to change this behavior depending on how we apply timestamps to these snapshots otherwise we may have a thread report a larger value belonging to a longer interval than others.
                Ok(ReportOutcome::Deferred)
            }
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
        match &self.metrics_sender {
            MetricsSender::Direct(sender) => match sender.try_send(snapshot) {
                Ok(()) => Ok(ReportOutcome::Sent),
                Err(flume::TrySendError::Full(_)) => Ok(ReportOutcome::Deferred),
                Err(flume::TrySendError::Disconnected(_)) => Err(Error::MetricsChannelClosed),
            },
            MetricsSender::Collector { flusher, .. } => flusher.try_send_snapshot(snapshot),
        }
    }

    /// Waits until every previously accepted snapshot has reached the registry.
    ///
    /// Standalone reporters created with [`Self::new`] have no collector task
    /// and return [`Error::MetricsCollectorNotRunning`]. Reporters created by
    /// the internal telemetry system use an ordered collector barrier.
    pub async fn flush(&self) -> Result<(), Error> {
        self.flush_until(Instant::now() + DEFAULT_RELIABLE_REPORT_TIMEOUT)
            .await
    }

    /// Waits until every previously accepted snapshot has reached the registry,
    /// but never beyond `deadline`.
    pub async fn flush_until(&self, deadline: Instant) -> Result<(), Error> {
        let flush = async {
            match &self.metrics_sender {
                MetricsSender::Direct(_) => Err(Error::MetricsCollectorNotRunning),
                MetricsSender::Collector { flusher, .. } => flusher.flush().await,
            }
        };
        tokio::time::timeout_at(TokioInstant::from_std(deadline), flush)
            .await
            .map_err(|_| reporting_deadline_exceeded())?
    }

    /// Performs the underlying reliable send; public entry points wrap this
    /// future in either an explicit or fallback deadline.
    async fn report_snapshot_reliably_unbounded(
        &self,
        snapshot: MetricSetSnapshot,
    ) -> Result<ReportOutcome, Error> {
        match &self.metrics_sender {
            MetricsSender::Direct(_) => self.try_report_snapshot_with_outcome(snapshot),
            MetricsSender::Collector { flusher, .. } => flusher.send_snapshot(snapshot).await,
        }
    }

    /// Reliably publishes a snapshot when this reporter is backed by a running collector.
    ///
    /// Standalone reporters and collector reporters used in synchronous
    /// [`InternalCollector::collect_pending`](crate::collector::InternalCollector::collect_pending)
    /// mode retain non-blocking behavior and return [`ReportOutcome::Deferred`]
    /// when their channel is full.
    pub async fn report_snapshot_reliably(
        &self,
        snapshot: MetricSetSnapshot,
    ) -> Result<ReportOutcome, Error> {
        self.report_snapshot_reliably_until(
            snapshot,
            Instant::now() + DEFAULT_RELIABLE_REPORT_TIMEOUT,
        )
        .await
    }

    /// Reliably publishes a snapshot, but never waits beyond `deadline`.
    pub async fn report_snapshot_reliably_until(
        &self,
        snapshot: MetricSetSnapshot,
        deadline: Instant,
    ) -> Result<ReportOutcome, Error> {
        tokio::time::timeout_at(
            TokioInstant::from_std(deadline),
            self.report_snapshot_reliably_unbounded(snapshot),
        )
        .await
        .map_err(|_| reporting_deadline_exceeded())?
    }

    /// Reliably reports and clears a hot metric set when backed by the internal collector.
    pub async fn report_reliably<M: MetricSetHandler>(
        &self,
        metrics: &mut MetricSet<M>,
    ) -> Result<ReportOutcome, Error> {
        self.report_reliably_until(metrics, Instant::now() + DEFAULT_RELIABLE_REPORT_TIMEOUT)
            .await
    }

    /// Reliably reports and clears a hot metric set, but never waits beyond `deadline`.
    pub async fn report_reliably_until<M: MetricSetHandler>(
        &self,
        metrics: &mut MetricSet<M>,
        deadline: Instant,
    ) -> Result<ReportOutcome, Error> {
        if !metrics.needs_flush() {
            return Ok(ReportOutcome::Sent);
        }
        let outcome = self
            .report_snapshot_reliably_until(metrics.snapshot(), deadline)
            .await?;
        if outcome == ReportOutcome::Sent {
            metrics.clear_values();
        }
        Ok(outcome)
    }

    /// Send a raw metrics snapshot directly to the collector.
    /// This method is primarily for testing purposes.
    #[cfg(test)]
    pub(crate) async fn report_snapshot(&self, snapshot: MetricSetSnapshot) -> Result<(), Error> {
        match &self.metrics_sender {
            MetricsSender::Direct(sender) => sender
                .send_async(snapshot)
                .await
                .map_err(|_| Error::MetricsChannelClosed),
            MetricsSender::Collector { flusher } => {
                let _ = flusher.send_snapshot(snapshot).await?;
                Ok(())
            }
        }
    }
}
