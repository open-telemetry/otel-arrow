// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use futures::StreamExt;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use tokio::time::Duration;

use super::client::LogsIngestionClient;

pub struct CompletedExport {
    pub batch_id: u64,
    pub client: LogsIngestionClient,
    pub result: Result<Duration, String>,
    pub row_count: f64,
}

pub struct InFlightExports {
    futures: FuturesUnordered<BoxFuture<'static, CompletedExport>>,
    limit: usize,
}

impl InFlightExports {
    pub fn new(limit: usize) -> Self {
        Self {
            futures: FuturesUnordered::new(),
            limit,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.futures.len()
    }

    #[inline]
    pub async fn next_completion(&mut self) -> Option<CompletedExport> {
        if self.futures.is_empty() {
            // Stay pending forever when empty - prevents busy loop
            std::future::pending().await
        } else {
            self.futures.next().await
        }
    }

    /// Push a future. If at capacity, waits for one completion and returns it.
    #[inline]
    pub async fn push(
        &mut self,
        fut: BoxFuture<'static, CompletedExport>,
    ) -> Option<CompletedExport> {
        let completed = if self.futures.len() >= self.limit {
            self.futures.next().await
        } else {
            None
        };
        self.futures.push(fut);
        completed
    }

    /// Create and push an export. Returns any completed export due to backpressure.
    pub async fn push_export(
        &mut self,
        client: LogsIngestionClient,
        batch_id: u64,
        row_count: f64,
        body: Bytes,
    ) -> Option<CompletedExport> {
        let fut = Self::make_export_future(client, batch_id, row_count, body);
        self.push(fut).await
    }

    /// Create a boxed export future.
    pub fn make_export_future(
        mut client: LogsIngestionClient,
        batch_id: u64,
        row_count: f64,
        body: Bytes,
    ) -> BoxFuture<'static, CompletedExport> {
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
            out.push(completed);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
    use azure_core::time::OffsetDateTime;
    use reqwest::Client;
    use std::sync::{Arc, Mutex};
    use std::time::Duration as StdDuration;

    // ==================== Test Helpers ====================

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in: azure_core::time::Duration,
        call_count: Arc<Mutex<usize>>,
    }

    impl MockCredential {
        fn new(token: &str, expires_in_minutes: i64) -> (Arc<Self>, Arc<Mutex<usize>>) {
            let call_count = Arc::new(Mutex::new(0));
            let cred = Arc::new(Self {
                token: token.to_string(),
                expires_in: azure_core::time::Duration::minutes(expires_in_minutes),
                call_count: call_count.clone(),
            });
            (cred, call_count)
        }
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc() + self.expires_in,
            })
        }
    }

    fn create_test_client() -> LogsIngestionClient {
        let (credential, _) = MockCredential::new("test_token", 60);
        // Use a client that will fail fast if actually used
        let http_client = Client::builder()
            .timeout(StdDuration::from_millis(1))
            .build()
            .expect("failed to create HTTP client");

        LogsIngestionClient::from_parts(
            http_client,
            "http://localhost".to_string(),
            credential,
            "scope".to_string(),
        )
    }

    /// Create a future that completes immediately with a success result.
    /// Used to fill the InFlightExports to test backpressure without waiting for real network calls.
    fn mock_completed_export_future(batch_id: u64) -> BoxFuture<'static, CompletedExport> {
        Box::pin(async move {
            CompletedExport {
                batch_id,
                client: create_test_client(),
                result: Ok(StdDuration::from_millis(1)),
                row_count: 1.0,
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

        let pending_future: BoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());

        let _ = exports.push(pending_future).await;

        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_len_multiple_pushes() {
        let mut exports = InFlightExports::new(10);

        for _ in 0..5 {
            let pending_future: BoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            let _ = exports.push(pending_future).await;
        }

        assert_eq!(exports.len(), 5);
    }

    // ==================== push Tests ====================

    #[tokio::test]
    async fn test_push_under_limit_returns_none() {
        let mut exports = InFlightExports::new(5);

        let pending_future: BoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());

        let result = exports.push(pending_future).await;

        assert!(result.is_none());
        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_push_increments_len() {
        let mut exports = InFlightExports::new(10);

        for i in 0..5 {
            let pending_future: BoxFuture<'static, CompletedExport> =
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
            .push_export(client, 1, 10.0, Bytes::from("data"))
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
        let _ = exports.push(mock_completed_export_future(100)).await;
        assert_eq!(exports.len(), 1);

        // 2. Now call push_export. Since we are at limit (1), it must wait for a completion.
        // It should receive the completion from our dummy future immediately.
        let client = create_test_client();
        let result = exports
            .push_export(client, 2, 20.0, Bytes::from("data"))
            .await;

        // Should get the completed dummy export
        assert!(result.is_some());
        let completed = result.unwrap();
        assert_eq!(completed.batch_id, 100);

        // The new export should be in the queue
        assert_eq!(exports.len(), 1);
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
        let _ = exports.push(mock_completed_export_future(1)).await;
        let _ = exports.push(mock_completed_export_future(2)).await;
        let _ = exports.push(mock_completed_export_future(3)).await;

        assert_eq!(exports.len(), 3);

        let drained = exports.drain().await;

        assert_eq!(drained.len(), 3);
        assert_eq!(exports.len(), 0);

        let ids: Vec<u64> = drained.iter().map(|c| c.batch_id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(ids.contains(&3));
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

        let _ = exports.push(mock_completed_export_future(42)).await;

        let result = exports.next_completion().await;

        assert!(result.is_some());
        assert_eq!(result.unwrap().batch_id, 42);
        assert_eq!(exports.len(), 0);
    }

    // ==================== Capacity/Backpressure Tests ====================

    #[tokio::test]
    async fn test_push_at_limit_with_immediate_future() {
        let mut exports = InFlightExports::new(2);

        // Fill to capacity with pending futures
        let pending1: BoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());
        let pending2: BoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());

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
            let pending: BoxFuture<'static, CompletedExport> = Box::pin(std::future::pending());
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
}
