// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded tracking for concurrent ClickHouse insert requests.

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
    pub result: Result<WrittenRows, ClickhouseExporterError>,
}

/// Tracks insert futures and enforces the configured concurrency bound.
pub(super) struct InFlightWrites {
    futures: FuturesUnordered<LocalBoxFuture<'static, CompletedWrite>>,
    limit: usize,
}

impl InFlightWrites {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            limit: limit.max(1),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    pub(super) fn is_at_capacity(&self) -> bool {
        self.futures.len() >= self.limit
    }

    pub(super) async fn push(
        &mut self,
        future: LocalBoxFuture<'static, CompletedWrite>,
    ) -> Option<CompletedWrite> {
        let completed = if self.is_at_capacity() {
            self.futures.next().await
        } else {
            None
        };
        self.futures.push(future);
        completed
    }

    pub(super) async fn next_completion(&mut self) -> Option<CompletedWrite> {
        self.futures.next().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn completed_write(rows: u64) -> CompletedWrite {
        CompletedWrite {
            signal_type: SignalType::Logs,
            result: Ok(vec![(ArrowPayloadType::Logs, rows)]),
        }
    }

    /// Scenario: a third insert is submitted to a queue limited to two concurrent requests.
    /// Guarantees: one request completes before the third is admitted and the queue stays bounded.
    #[tokio::test]
    async fn push_at_capacity_waits_for_a_completion() {
        let mut writes = InFlightWrites::new(2);
        assert!(
            writes
                .push(Box::pin(async { completed_write(1) }))
                .await
                .is_none()
        );
        assert!(
            writes
                .push(Box::pin(async { completed_write(2) }))
                .await
                .is_none()
        );
        assert!(writes.is_at_capacity());

        let completed = writes
            .push(Box::pin(async { completed_write(3) }))
            .await
            .expect("one insert should finish before admitting another");

        assert!(matches!(completed.result.unwrap()[0].1, 1 | 2));
        assert!(writes.is_at_capacity());
    }

    /// Scenario: an internal caller constructs a queue with a zero concurrency limit.
    /// Guarantees: the defensive lower bound still permits exactly one in-flight request.
    #[tokio::test]
    async fn zero_limit_is_clamped_to_one() {
        let mut writes = InFlightWrites::new(0);
        assert!(
            writes
                .push(Box::pin(async { completed_write(1) }))
                .await
                .is_none()
        );
        assert!(writes.is_at_capacity());

        let completed = writes
            .push(Box::pin(async { completed_write(2) }))
            .await
            .expect("the first insert should complete before admitting the second");

        assert_eq!(completed.result.unwrap()[0].1, 1);
        assert!(writes.is_at_capacity());
    }

    /// Scenario: shutdown begins while two accepted insert requests are still in flight.
    /// Guarantees: callers can drain every accepted request and observe all written row counts.
    #[tokio::test]
    async fn accepted_writes_can_be_drained() {
        let mut writes = InFlightWrites::new(2);
        assert!(
            writes
                .push(Box::pin(async { completed_write(3) }))
                .await
                .is_none()
        );
        assert!(
            writes
                .push(Box::pin(async { completed_write(5) }))
                .await
                .is_none()
        );

        let mut rows = Vec::new();
        while let Some(completed) = writes.next_completion().await {
            rows.push(completed.result.unwrap()[0].1);
        }
        rows.sort_unstable();

        assert_eq!(rows, vec![3, 5]);
        assert!(writes.is_empty());
    }
}
