// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded tracking for concurrent ClickHouse insert requests.

use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::time::Instant;

use futures::StreamExt;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use otap_df_config::SignalType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::error::ClickhouseExporterError;

pub(super) type WrittenRows = Vec<(ArrowPayloadType, u64)>;

/// Result of one fully transformed pdata message sent to ClickHouse.
pub(super) struct CompletedWrite {
    pub signal_type: SignalType,
    pub export_started_at: Instant,
    pub result: Result<WrittenRows, ClickhouseExporterError>,
}

/// Tracks insert futures and enforces the configured concurrency bound.
pub(super) struct InFlightWrites {
    futures: FuturesUnordered<LocalBoxFuture<'static, CompletedWrite>>,
    queued: VecDeque<LocalBoxFuture<'static, CompletedWrite>>,
    limit: NonZeroUsize,
}

impl InFlightWrites {
    pub(super) fn new(limit: NonZeroUsize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            queued: VecDeque::new(),
            limit,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.futures.is_empty() && self.queued.is_empty()
    }

    pub(super) fn is_at_capacity(&self) -> bool {
        self.futures.len() >= self.limit.get()
    }

    pub(super) fn len(&self) -> usize {
        self.futures.len() + self.queued.len()
    }

    /// Adds a write without blocking the exporter's control-message loop.
    ///
    /// Normal pdata admission stops when the active set reaches `limit`. The
    /// queue is used only when `ExporterInbox` force-drains buffered pdata
    /// during shutdown. Queued futures remain unpolled until an active slot is
    /// available, so the configured concurrency bound still applies.
    pub(super) fn push(&mut self, future: LocalBoxFuture<'static, CompletedWrite>) {
        self.queued.push_back(future);
        self.fill_capacity();
    }

    pub(super) async fn next_completion(&mut self) -> Option<CompletedWrite> {
        self.fill_capacity();
        let completed = self.futures.next().await;
        if completed.is_some() {
            self.fill_capacity();
        }
        completed
    }

    /// Drains accepted writes until they complete or the shutdown deadline expires.
    ///
    /// Returns the number of active and queued writes left when the deadline
    /// expires. Those futures are cancelled when this tracker is dropped.
    pub(super) async fn drain_until(
        &mut self,
        deadline: tokio::time::Instant,
        mut on_completion: impl FnMut(CompletedWrite),
    ) -> usize {
        while !self.is_empty() {
            match tokio::time::timeout_at(deadline, self.next_completion()).await {
                Ok(Some(completed)) => on_completion(completed),
                Ok(None) => break,
                Err(_) => return self.len(),
            }
        }
        0
    }

    fn fill_capacity(&mut self) {
        while !self.is_at_capacity() {
            let Some(future) = self.queued.pop_front() else {
                break;
            };
            self.futures.push(future);
        }
    }

    #[cfg(test)]
    fn active_len(&self) -> usize {
        self.futures.len()
    }

    #[cfg(test)]
    fn queued_len(&self) -> usize {
        self.queued.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(value: usize) -> NonZeroUsize {
        NonZeroUsize::new(value).expect("test limit must be non-zero")
    }

    fn completed_write(rows: u64) -> CompletedWrite {
        CompletedWrite {
            signal_type: SignalType::Logs,
            export_started_at: Instant::now(),
            result: Ok(vec![(ArrowPayloadType::Logs, rows)]),
        }
    }

    /// Scenario: forced shutdown draining submits a third write while two writes are active.
    /// Guarantees: submission does not block and the third future remains unpolled until a slot opens.
    #[tokio::test]
    async fn push_at_capacity_queues_without_blocking() {
        let mut writes = InFlightWrites::new(limit(2));
        writes.push(Box::pin(async { completed_write(1) }));
        writes.push(Box::pin(futures::future::pending()));
        assert!(writes.is_at_capacity());

        writes.push(Box::pin(async { completed_write(3) }));
        assert_eq!(writes.active_len(), 2);
        assert_eq!(writes.queued_len(), 1);
        assert_eq!(writes.len(), 3);

        let completed = writes
            .next_completion()
            .await
            .expect("the ready active write should complete");
        assert_eq!(completed.result.unwrap()[0].1, 1);
        assert!(writes.is_at_capacity());
        assert_eq!(writes.active_len(), 2);
        assert_eq!(writes.queued_len(), 0);
    }

    /// Scenario: shutdown begins while two accepted insert requests are still in flight.
    /// Guarantees: callers can drain every accepted request and observe all written row counts.
    #[tokio::test]
    async fn accepted_writes_can_be_drained() {
        let mut writes = InFlightWrites::new(limit(2));
        writes.push(Box::pin(async { completed_write(3) }));
        writes.push(Box::pin(async { completed_write(5) }));

        let mut rows = Vec::new();
        let abandoned = writes
            .drain_until(
                tokio::time::Instant::now() + std::time::Duration::from_secs(1),
                |completed| {
                    rows.push(completed.result.unwrap()[0].1);
                },
            )
            .await;
        rows.sort_unstable();

        assert_eq!(abandoned, 0);
        assert_eq!(rows, vec![3, 5]);
        assert!(writes.is_empty());
    }

    /// Scenario: shutdown reaches its deadline with one active and one queued write stalled.
    /// Guarantees: draining returns at the deadline and reports every write that will be abandoned.
    #[tokio::test(start_paused = true)]
    async fn drain_stops_at_deadline() {
        let mut writes = InFlightWrites::new(limit(1));
        writes.push(Box::pin(futures::future::pending()));
        writes.push(Box::pin(async { completed_write(2) }));

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        let abandoned = writes
            .drain_until(deadline, |_| panic!("stalled writes must not complete"))
            .await;

        assert_eq!(abandoned, 2);
        assert_eq!(writes.active_len(), 1);
        assert_eq!(writes.queued_len(), 1);
    }
}
