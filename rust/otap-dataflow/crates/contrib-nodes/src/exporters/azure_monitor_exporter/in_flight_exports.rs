// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use futures::StreamExt;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use tokio::time::Duration;

use super::client::LogsIngestionClient;
use super::error::Error;

pub struct CompletedExport {
    pub batch_id: u64,
    pub client: LogsIngestionClient,
    pub result: Result<Duration, Error>,
    pub row_count: u64,
}

pub struct InFlightExports {
    futures: FuturesUnordered<LocalBoxFuture<'static, CompletedExport>>,
    limit: usize,
    /// Running total of log records (rows) across all in-flight exports.
    ///
    /// Maintained by [`InFlightExports::push_export`] (which increments by the
    /// enqueued `row_count`) and the completion paths ([`next_completion`],
    /// [`push`], and [`drain`], which decrement by the completed export's
    /// `row_count`).
    queued_rows: u64,
}

impl InFlightExports {
    pub fn new(limit: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            limit,
            queued_rows: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.futures.len()
    }

    /// Current number of log records (rows) across all in-flight exports.
    #[inline]
    pub fn queued_rows(&self) -> u64 {
        self.queued_rows
    }

    #[inline]
    pub async fn next_completion(&mut self) -> Option<CompletedExport> {
        if self.futures.is_empty() {
            // Stay pending forever when empty - prevents busy loop
            std::future::pending().await
        } else {
            let completed = self.futures.next().await;
            if let Some(ref c) = completed {
                self.queued_rows = self.queued_rows.saturating_sub(c.row_count);
            }
            completed
        }
    }

    /// Push a future. If at capacity, waits for one completion and returns it.
    #[inline]
    async fn push(
        &mut self,
        fut: LocalBoxFuture<'static, CompletedExport>,
    ) -> Option<CompletedExport> {
        let completed = if self.futures.len() >= self.limit {
            self.futures.next().await
        } else {
            None
        };
        if let Some(ref c) = completed {
            self.queued_rows = self.queued_rows.saturating_sub(c.row_count);
        }
        self.futures.push(fut);
        completed
    }

    /// Create and push an export. Returns any completed export due to backpressure.
    pub async fn push_export(
        &mut self,
        client: LogsIngestionClient,
        batch_id: u64,
        row_count: u64,
        body: Bytes,
    ) -> Option<CompletedExport> {
        let fut = Self::make_export_future(client, batch_id, row_count, body);
        self.queued_rows = self.queued_rows.saturating_add(row_count);
        self.push(fut).await
    }

    /// Create a boxed export future.
    pub fn make_export_future(
        mut client: LogsIngestionClient,
        batch_id: u64,
        row_count: u64,
        body: Bytes,
    ) -> LocalBoxFuture<'static, CompletedExport> {
        Box::pin(async move {
            let result = client.export(body).await;
            CompletedExport {
                batch_id,
                client,
                result,
                row_count,
            }
        })
    }

    /// Drain all in-flight exports to completion.
    pub async fn drain(&mut self) -> Vec<CompletedExport> {
        let mut out = Vec::with_capacity(self.futures.len());
        while let Some(completed) = self.futures.next().await {
            self.queued_rows = self.queued_rows.saturating_sub(completed.row_count);
            out.push(completed);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::super::metrics::{
        AzureMonitorExporterMetrics, AzureMonitorExporterMetricsRc,
        AzureMonitorExporterMetricsTracker,
    };
    use super::*;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::testing::EmptyAttributes;
    use reqwest::Client;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration as StdDuration;

    // ==================== Test Helpers ====================

    fn create_test_metrics() -> AzureMonitorExporterMetricsRc {
        let registry = TelemetryRegistryHandle::new();
        let metric_set =
            registry.register_metric_set::<AzureMonitorExporterMetrics>(EmptyAttributes());
        Rc::new(RefCell::new(AzureMonitorExporterMetricsTracker::new(
            metric_set,
        )))
    }

    fn create_test_client() -> LogsIngestionClient {
        otap_df_otap::crypto::ensure_crypto_provider();
        // Use a client that will fail fast if actually used
        let http_client = Client::builder()
            .timeout(StdDuration::from_millis(1))
            .build()
            .expect("failed to create HTTP client");

        LogsIngestionClient::from_parts(
            http_client,
            "http://localhost".to_string(),
            create_test_metrics(),
        )
    }

    /// Create a future that completes immediately with the given `row_count`
    /// and result. Used to fill the InFlightExports and exercise backpressure
    /// and in-flight record accounting without waiting for real network calls.
    fn mock_completed_export_future(
        batch_id: u64,
        row_count: u64,
        success: bool,
    ) -> LocalBoxFuture<'static, CompletedExport> {
        Box::pin(async move {
            let result = if success {
                Ok(StdDuration::from_millis(1))
            } else {
                Err(Error::LogEntryTooLarge)
            };
            CompletedExport {
                batch_id,
                client: create_test_client(),
                result,
                row_count,
            }
        })
    }

    // ==================== Construction Tests ====================

    #[test]
    fn test_new_creates_empty_container() {
        let exports = InFlightExports::new(5);

        assert_eq!(exports.len(), 0);
        assert_eq!(exports.limit, 5);
    }

    #[test]
    fn test_new_various_limits() {
        for limit in [1, 4, 8, 32, 100] {
            let exports = InFlightExports::new(limit);
            assert_eq!(exports.limit, limit);
            assert_eq!(exports.len(), 0);
        }
    }

    // ==================== len Tests ====================

    #[test]
    fn test_len_empty() {
        let exports = InFlightExports::new(5);
        assert_eq!(exports.len(), 0);
    }

    #[tokio::test]
    async fn test_len_after_push() {
        let mut exports = InFlightExports::new(5);

        let pending_future: LocalBoxFuture<'static, CompletedExport> =
            Box::pin(std::future::pending());

        let _ = exports.push(pending_future).await;

        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_len_multiple_pushes() {
        let mut exports = InFlightExports::new(10);

        for _ in 0..5 {
            let pending_future: LocalBoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            let _ = exports.push(pending_future).await;
        }

        assert_eq!(exports.len(), 5);
    }

    // ==================== push Tests ====================

    #[tokio::test]
    async fn test_push_under_limit_returns_none() {
        let mut exports = InFlightExports::new(5);

        let pending_future: LocalBoxFuture<'static, CompletedExport> =
            Box::pin(std::future::pending());

        let result = exports.push(pending_future).await;

        assert!(result.is_none());
        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_push_increments_len() {
        let mut exports = InFlightExports::new(10);

        for i in 0..5 {
            let pending_future: LocalBoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            let _ = exports.push(pending_future).await;
            assert_eq!(exports.len(), i + 1);
        }
    }

    // ==================== push_export Tests ====================

    #[tokio::test]
    async fn test_push_export_adds_to_futures() {
        let mut exports = InFlightExports::new(5);
        let client = create_test_client();

        // This should not block because we are under the limit
        let result = exports
            .push_export(client, 1, 10, Bytes::from("data"))
            .await;

        assert!(result.is_none());
        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_push_export_respects_limit() {
        let mut exports = InFlightExports::new(1);

        // 1. Fill the capacity with a dummy future that completes immediately
        // We use a dummy future because if we used push_export, it would create a real
        // export future that might retry and take a long time to fail/complete.
        let _ = exports
            .push(mock_completed_export_future(100, 1, true))
            .await;
        assert_eq!(exports.len(), 1);

        // 2. Now call push_export. Since we are at limit (1), it must wait for a completion.
        // It should receive the completion from our dummy future immediately.
        let client = create_test_client();
        let result = exports
            .push_export(client, 2, 20, Bytes::from("data"))
            .await;

        // Should get the completed dummy export
        assert!(result.is_some());
        let completed = result.unwrap();
        assert_eq!(completed.batch_id, 100);

        // The new export should be in the queue
        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_push_export_increments_queued_rows() {
        let mut exports = InFlightExports::new(5);

        // Under the limit, push_export does not block and increments the
        // in-flight record tally by the enqueued row count. The real export
        // future stays pending.
        let _ = exports
            .push_export(create_test_client(), 1, 100, Bytes::from("data"))
            .await;
        assert_eq!(exports.queued_rows(), 100);

        let _ = exports
            .push_export(create_test_client(), 2, 50, Bytes::from("data"))
            .await;
        assert_eq!(exports.queued_rows(), 150);
    }

    #[tokio::test]
    async fn test_push_export_backpressure_decrements_queued_rows() {
        let mut exports = InFlightExports::new(1);
        exports.queued_rows = 10;
        // Fill capacity with a completing future worth 10 records.
        let _ = exports
            .push(mock_completed_export_future(1, 10, true))
            .await;
        assert_eq!(exports.len(), 1);

        // At capacity, push_export(+25) pops the completed future (-10) before
        // adding the new one: 10 + 25 - 10 = 25.
        let completed = exports
            .push_export(create_test_client(), 2, 25, Bytes::from("data"))
            .await;
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().row_count, 10);
        assert_eq!(exports.queued_rows(), 25);
    }

    // ==================== drain Tests ====================

    #[tokio::test]
    async fn test_drain_empty() {
        let mut exports = InFlightExports::new(5);

        let drained = exports.drain().await;

        assert!(drained.is_empty());
        assert_eq!(exports.len(), 0);
    }

    #[tokio::test]
    async fn test_drain_returns_all_futures() {
        let mut exports = InFlightExports::new(5);

        // Push 3 dummy completed futures
        let _ = exports.push(mock_completed_export_future(1, 1, true)).await;
        let _ = exports.push(mock_completed_export_future(2, 1, true)).await;
        let _ = exports.push(mock_completed_export_future(3, 1, true)).await;

        assert_eq!(exports.len(), 3);

        let drained = exports.drain().await;

        assert_eq!(drained.len(), 3);
        assert_eq!(exports.len(), 0);

        let ids: Vec<u64> = drained.iter().map(|c| c.batch_id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
    }

    #[tokio::test]
    async fn test_drain_resets_queued_rows_to_zero() {
        let mut exports = InFlightExports::new(5);
        exports.queued_rows = 60;
        let _ = exports
            .push(mock_completed_export_future(1, 20, true))
            .await;
        let _ = exports
            .push(mock_completed_export_future(2, 40, false))
            .await;

        let drained = exports.drain().await;
        assert_eq!(drained.len(), 2);
        assert_eq!(exports.queued_rows(), 0);
    }

    // ==================== next_completion Tests ====================

    #[tokio::test]
    async fn test_next_completion_empty_stays_pending() {
        let mut exports = InFlightExports::new(5);

        // next_completion on empty should stay pending forever
        // We test this with a timeout
        let result =
            tokio::time::timeout(StdDuration::from_millis(10), exports.next_completion()).await;

        // Should timeout because next_completion is pending
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_next_completion_returns_completed() {
        let mut exports = InFlightExports::new(5);

        let _ = exports
            .push(mock_completed_export_future(42, 1, true))
            .await;

        let result = exports.next_completion().await;

        assert!(result.is_some());
        assert_eq!(result.unwrap().batch_id, 42);
        assert_eq!(exports.len(), 0);
    }

    #[tokio::test]
    async fn test_next_completion_decrements_queued_rows() {
        let mut exports = InFlightExports::new(5);
        // Simulate in-flight exports totaling 100 records.
        exports.queued_rows = 100;
        let _ = exports
            .push(mock_completed_export_future(1, 30, true))
            .await;

        let completed = exports.next_completion().await.unwrap();
        assert_eq!(completed.row_count, 30);
        assert_eq!(exports.queued_rows(), 70);
    }

    #[tokio::test]
    async fn test_next_completion_failure_decrements_queued_rows() {
        let mut exports = InFlightExports::new(5);
        exports.queued_rows = 40;
        let _ = exports
            .push(mock_completed_export_future(1, 40, false))
            .await;

        let completed = exports.next_completion().await.unwrap();
        assert!(completed.result.is_err());
        assert_eq!(exports.queued_rows(), 0);
    }

    #[tokio::test]
    async fn test_next_completion_queued_rows_saturates_on_underflow() {
        let mut exports = InFlightExports::new(5);
        // queued_rows is 0 but a completion reports 5 records; saturating_sub
        // must keep it at 0 rather than wrapping around.
        let _ = exports.push(mock_completed_export_future(1, 5, true)).await;
        let _ = exports.next_completion().await.unwrap();
        assert_eq!(exports.queued_rows(), 0);
    }

    // ==================== Capacity/Backpressure Tests ====================

    #[tokio::test]
    async fn test_push_at_limit_with_immediate_future() {
        let mut exports = InFlightExports::new(2);

        // Fill to capacity with pending futures
        let pending1: LocalBoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());
        let pending2: LocalBoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());

        let _ = exports.push(pending1).await;
        let _ = exports.push(pending2).await;

        assert_eq!(exports.len(), 2);

        // Next push would need to wait for completion
        // Since we have pending futures, this would block forever
        // We test the logic by checking we're at capacity
        assert_eq!(exports.len(), exports.limit);
    }

    #[tokio::test]
    async fn test_capacity_limit_respected() {
        let limit = 3;
        let mut exports = InFlightExports::new(limit);

        // Push up to limit
        for _ in 0..limit {
            let pending: LocalBoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            let result = exports.push(pending).await;
            assert!(result.is_none(), "Should not return completion under limit");
        }

        assert_eq!(exports.len(), limit);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_new_with_zero_limit() {
        let exports = InFlightExports::new(0);
        assert_eq!(exports.limit, 0);
        assert_eq!(exports.len(), 0);
    }

    #[test]
    fn test_new_with_large_limit() {
        let exports = InFlightExports::new(1000);
        assert_eq!(exports.limit, 1000);
        assert_eq!(exports.len(), 0);
    }

    #[test]
    fn test_new_queued_rows_starts_at_zero() {
        let exports = InFlightExports::new(5);
        assert_eq!(exports.queued_rows(), 0);
    }
}
