// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer lifecycle and consumer-group lag.
//!
//! Bounds the two potentially blocking librdkafka operations off the
//! single-threaded runtime: the final consumer close on drain/shutdown and the
//! periodic mean consumer-group lag refresh (run on a `spawn_blocking` worker
//! bounded by a deadline and a cancellation token).

use super::KafkaReceiver;
use otel_arrow_dfe_engine::effect_handler::TelemetryTimerCancelHandle;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, ConsumerContext};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

/// Bounded broker timeout for a single per-partition consumer-lag watermark
/// lookup. This bounds *each* `fetch_watermarks` call, not the whole refresh:
/// a refresh queries every owned partition sequentially, so the worst-case time
/// spent in one refresh scales with the number of owned partitions
/// (`partitions * timeout`).
pub(super) const LAG_FETCH_PARTITION_TIMEOUT: Duration = Duration::from_secs(5);

/// Total deadline for a single off-loop consumer-lag refresh.
pub(super) const LAG_REFRESH_TOTAL_DEADLINE: Duration = Duration::from_secs(15);

pub(super) type LagRefreshTask = (
    tokio::task::JoinHandle<Option<f64>>,
    tokio::time::Instant,
    CancellationToken,
);

/// Stops receiver-owned background work and closes the Kafka consumer without
/// blocking the single-threaded pipeline runtime past `deadline`.
pub(super) async fn close_consumer_bounded<C: ConsumerContext + 'static>(
    consumer: Arc<StreamConsumer<C>>,
    lag_refresh_in_flight: &mut Option<LagRefreshTask>,
    telemetry_cancel_handle: TelemetryTimerCancelHandle<OtapPdata>,
    deadline: Instant,
) {
    if let Some((handle, lag_deadline, lag_cancel)) = lag_refresh_in_flight.take() {
        lag_cancel.cancel();
        let bound = lag_deadline.min(tokio::time::Instant::from_std(deadline));
        let _ = tokio::time::timeout_at(bound, handle).await;
    }

    let _ = telemetry_cancel_handle.cancel().await;
    let close_handle = tokio::task::spawn_blocking(move || {
        consumer.unsubscribe();
        drop(consumer);
    });
    let _ = tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), close_handle).await;
}

/// Compute the mean consumer-group lag across all owned partitions, bounded by
/// an absolute `deadline`. The `deadline` is checked before each partition
///
/// Return contract (see [`KafkaReceiver::spawn_consumer_lag_refresh`]):
/// - `Some(mean)` -- every owned partition was measured; the mean covers the
///   whole assignment.
/// - `Some(0.0)` -- the assignment is empty (nothing owned); the caller resets
///   the gauge to the documented empty-assignment value.
/// - `None` -- the refresh is incomplete (an owned partition has no committed
///   offset yet, a broker read failed, or the `deadline` was exceeded); the
///   caller retains the previous gauge value.
pub(super) fn compute_consumer_lag<C: ConsumerContext>(
    consumer: &StreamConsumer<C>,
    deadline: Instant,
    cancel: &CancellationToken,
) -> Option<f64> {
    // Remaining time until `deadline`
    let remaining_call_timeout = || -> Option<Duration> {
        if cancel.is_cancelled() {
            return None;
        }
        let remaining = deadline.checked_duration_since(Instant::now())?;
        if remaining.is_zero() {
            return None;
        }
        Some(remaining.min(LAG_FETCH_PARTITION_TIMEOUT))
    };

    // Owned partitions. `assignment()` is a local (non-RPC) query.
    let assignment = match consumer.assignment() {
        Ok(tpl) => tpl,
        Err(e) => {
            otel_error!("kafka.lag.assignment_failed", error = %e);
            return None;
        }
    };
    if assignment.count() == 0 {
        // Nothing owned: reset the gauge to the documented empty value (0).
        return Some(0.0);
    }

    // Deadline / cancellation check before the first (committed_offsets) broker
    // call.
    let Some(committed_timeout) = remaining_call_timeout() else {
        let reason = if cancel.is_cancelled() {
            "cancelled"
        } else {
            "deadline_exceeded"
        };
        otel_warn!("kafka.lag.refresh_incomplete", reason = reason);
        return None;
    };

    // Broker-acknowledged committed offsets for the owned partitions.
    let committed = match consumer.committed_offsets(assignment, committed_timeout) {
        Ok(tpl) => tpl,
        Err(e) => {
            otel_error!("kafka.lag.committed_offsets_failed", error = %e);
            return None;
        }
    };

    // Per-partition consumer-group lag for *every* owned partition. The mean
    // must cover the whole assignment: any partition we cannot measure -- a
    // missing committed offset, a failed broker read, or the deadline expiring
    // -- abandons the whole refresh (returns `None`) so a mean is never computed
    // from a subset of partitions.
    let elements = committed.elements();
    let mut sum: i64 = 0;
    for elem in &elements {
        // Bound this partition's watermark lookup by the remaining time (or
        // abandon on cancellation).
        let Some(watermark_timeout) = remaining_call_timeout() else {
            let reason = if cancel.is_cancelled() {
                "cancelled"
            } else {
                "deadline_exceeded"
            };
            otel_warn!("kafka.lag.refresh_incomplete", reason = reason);
            return None;
        };

        let topic = elem.topic();
        let partition = elem.partition();

        // An owned partition with no broker-committed offset yet
        // (`Offset::Invalid`) cannot be measured. Abort rather than exclude it,
        // so the mean always covers the whole assignment.
        let committed_offset = match elem.offset() {
            rdkafka::Offset::Offset(o) => o,
            _ => {
                otel_warn!(
                    "kafka.lag.refresh_incomplete",
                    reason = "uncommitted_partition",
                    topic = %topic,
                    partition = partition,
                );
                return None;
            }
        };

        match consumer.fetch_watermarks(topic, partition, watermark_timeout) {
            Ok((_low, high)) => {
                // Both the high watermark and the committed offset are
                // "one past" positions, so their difference is the number
                // of records the group has not yet consumed on this
                // partition (consumer-group lag).
                sum = sum.saturating_add(high.saturating_sub(committed_offset).max(0));
            }
            Err(e) => {
                // Fail fast: one failed lookup means the mean would be
                // incomplete, so abandon this refresh and retain the
                // previous gauge value.
                otel_error!(
                    "kafka.lag.fetch_watermarks_failed",
                    topic = %topic,
                    partition = partition,
                    error = %e,
                );
                return None;
            }
        }
    }

    // `elements` is non-empty here (assignment count was > 0), so the divisor is
    // never zero.
    Some(sum as f64 / elements.len() as f64)
}

impl KafkaReceiver {
    /// Spawn an off-loop consumer-lag refresh, returning its join handle.
    ///
    /// Moves an `Arc` clone of the consumer into a blocking task
    /// ([`tokio::task::spawn_blocking`]) that runs [`compute_consumer_lag`] off
    /// the receive loop.
    ///
    /// The task returns:
    /// - `Some(mean_lag)` when the high-watermark lookup succeeds for *every*
    ///   owned partition (the mean covers the whole assignment, never a subset);
    /// - `Some(0.0)` when the assignment is empty (nothing owned), the caller's
    ///   signal to reset the gauge to the documented empty-assignment value;
    /// - `None` when the refresh is incomplete -- any owned partition lacks a
    ///   committed offset, a broker read failed, or the deadline was exceeded --
    ///   the caller's signal to retain the previous gauge value. Instantly returns
    ///   when in auto-commit mode.
    pub(super) fn spawn_consumer_lag_refresh<C: ConsumerContext + 'static>(
        &self,
        consumer: &Arc<StreamConsumer<C>>,
        deadline: Instant,
        cancel: CancellationToken,
    ) -> Option<tokio::task::JoinHandle<Option<f64>>> {
        if self.config.is_auto_commit() {
            return None;
        }

        let consumer = Arc::clone(consumer);
        Some(tokio::task::spawn_blocking(move || {
            compute_consumer_lag(consumer.as_ref(), deadline, &cancel)
        }))
    }
}
