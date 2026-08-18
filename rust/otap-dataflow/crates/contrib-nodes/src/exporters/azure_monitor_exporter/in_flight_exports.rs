// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use futures::StreamExt;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use http::HeaderValue;
use tokio::time::Duration;

use super::client::LogsIngestionClient;
use super::error::Error;

pub struct CompletedExport {
    pub batch_id: u64,
    pub client: LogsIngestionClient,
    pub result: Result<Duration, Error>,
    pub row_count: u64,
    pub body_size_bytes: u64,
    pub token_generation: u64,
}

pub struct InFlightExports {
    futures: FuturesUnordered<LocalBoxFuture<'static, CompletedExport>>,
    limit: usize,
    /// Running total of log records (rows) across all in-flight exports.
    ///
    /// Maintained by [`InFlightExports::push_export`] (which increments by the
    /// enqueued `row_count`) and the completion paths ([`next_completion`],
    /// [`reap_if_at_capacity`], and [`drain`], which decrement by the completed
    /// export's `row_count`).
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

    /// Free a slot when the set is full, returning the export that completed.
    ///
    /// Callers must reap before [`push_export`](Self::push_export) and settle the
    /// completion first: it may reject the token the next export would otherwise
    /// already have been stamped with.
    pub async fn reap_if_at_capacity(&mut self) -> Option<CompletedExport> {
        if self.futures.len() < self.limit {
            return None;
        }
        let completed = self.futures.next().await;
        if let Some(ref c) = completed {
            self.queued_rows = self.queued_rows.saturating_sub(c.row_count);
        }
        completed
    }

    /// Stamp a batch with `auth_header` and add it to the in-flight set.
    ///
    /// Does not enforce the limit; call [`reap_if_at_capacity`](Self::reap_if_at_capacity) first.
    pub fn push_export(
        &mut self,
        client: LogsIngestionClient,
        batch_id: u64,
        row_count: u64,
        body: Bytes,
        auth_header: HeaderValue,
        token_generation: u64,
    ) {
        let fut = Self::make_export_future(
            client,
            batch_id,
            row_count,
            body,
            auth_header,
            token_generation,
        );
        self.queued_rows = self.queued_rows.saturating_add(row_count);
        self.push(fut);
    }

    #[inline]
    fn push(&mut self, fut: LocalBoxFuture<'static, CompletedExport>) {
        self.futures.push(fut);
    }

    /// Create a boxed export future.
    fn make_export_future(
        mut client: LogsIngestionClient,
        batch_id: u64,
        row_count: u64,
        body: Bytes,
        auth_header: HeaderValue,
        token_generation: u64,
    ) -> LocalBoxFuture<'static, CompletedExport> {
        Box::pin(async move {
            let body_size_bytes = body.len() as u64;
            let result = client.export(body, &auth_header).await;
            CompletedExport {
                batch_id,
                client,
                result,
                row_count,
                body_size_bytes,
                token_generation,
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
        AzureMonitorExporterMetricsRc, AzureMonitorExporterMetricsTracker,
    };
    use super::*;
    use otap_df_engine::context::{ControllerContext, PipelineContext};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use reqwest::Client;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration as StdDuration;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // ==================== Test Helpers ====================

    fn create_test_metrics() -> AzureMonitorExporterMetricsRc {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx: PipelineContext =
            controller.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        Rc::new(RefCell::new(AzureMonitorExporterMetricsTracker::register(
            &pipeline_ctx,
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
                body_size_bytes: 0,
                token_generation: 1,
            }
        })
    }

    fn test_auth() -> (HeaderValue, u64) {
        (HeaderValue::from_static("Bearer test-token"), 1)
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

        exports.push(pending_future);

        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_len_multiple_pushes() {
        let mut exports = InFlightExports::new(10);

        for _ in 0..5 {
            let pending_future: LocalBoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            exports.push(pending_future);
        }

        assert_eq!(exports.len(), 5);
    }

    // ==================== push Tests ====================

    #[tokio::test]
    async fn test_push_increments_len() {
        let mut exports = InFlightExports::new(10);

        for i in 0..5 {
            let pending_future: LocalBoxFuture<'static, CompletedExport> =
                Box::pin(std::future::pending());
            exports.push(pending_future);
            assert_eq!(exports.len(), i + 1);
        }
    }

    // ==================== reap_if_at_capacity Tests ====================

    /// Scenario: a caller asks to free a slot while the in-flight set is below
    /// its limit.
    /// Guarantees: reaping returns nothing without awaiting, so queueing a batch
    /// never stalls while slots remain.
    #[tokio::test]
    async fn reap_under_the_limit_returns_none_without_waiting() {
        let mut exports = InFlightExports::new(5);
        exports.push(Box::pin(std::future::pending()));

        let reaped =
            tokio::time::timeout(StdDuration::from_millis(50), exports.reap_if_at_capacity())
                .await
                .expect("reaping below the limit must not wait");

        assert!(reaped.is_none());
        assert_eq!(exports.len(), 1);
    }

    /// Scenario: a caller asks to free a slot while the in-flight set is full.
    /// Guarantees: exactly one export is awaited and handed back, freeing a slot
    /// and decrementing the in-flight record tally, so the caller can settle that
    /// completion before stamping the next export.
    #[tokio::test]
    async fn reap_at_capacity_returns_the_completed_export_and_frees_a_slot() {
        let mut exports = InFlightExports::new(1);
        exports.queued_rows = 10;
        exports.push(mock_completed_export_future(100, 10, true));

        let reaped = exports
            .reap_if_at_capacity()
            .await
            .expect("at capacity, one export must be reaped");

        assert_eq!(reaped.batch_id, 100);
        assert_eq!(exports.len(), 0);
        assert_eq!(exports.queued_rows(), 0);
    }

    // ==================== push_export Tests ====================

    #[tokio::test]
    async fn test_push_export_adds_to_futures() {
        let mut exports = InFlightExports::new(5);
        let client = create_test_client();

        let (auth_header, token_generation) = test_auth();
        exports.push_export(
            client,
            1,
            10,
            Bytes::from("data"),
            auth_header,
            token_generation,
        );

        assert_eq!(exports.len(), 1);
    }

    #[tokio::test]
    async fn test_push_export_increments_queued_rows() {
        let mut exports = InFlightExports::new(5);

        // push_export increments the in-flight record tally by the enqueued row
        // count. The real export future stays pending.
        let (auth_header, token_generation) = test_auth();
        exports.push_export(
            create_test_client(),
            1,
            100,
            Bytes::from("data"),
            auth_header,
            token_generation,
        );
        assert_eq!(exports.queued_rows(), 100);

        let (auth_header, token_generation) = test_auth();
        exports.push_export(
            create_test_client(),
            2,
            50,
            Bytes::from("data"),
            auth_header,
            token_generation,
        );
        assert_eq!(exports.queued_rows(), 150);
    }

    #[tokio::test]
    async fn test_backpressure_decrements_queued_rows() {
        let mut exports = InFlightExports::new(1);
        exports.queued_rows = 10;
        // Fill capacity with a completing future worth 10 records.
        exports.push(mock_completed_export_future(1, 10, true));
        assert_eq!(exports.len(), 1);

        // Reaping pops the completed future (-10), then push_export adds the new
        // one (+25): 10 - 10 + 25 = 25.
        let completed = exports.reap_if_at_capacity().await;
        assert!(completed.is_some());
        assert_eq!(completed.unwrap().row_count, 10);

        let (auth_header, token_generation) = test_auth();
        exports.push_export(
            create_test_client(),
            2,
            25,
            Bytes::from("data"),
            auth_header,
            token_generation,
        );
        assert_eq!(exports.queued_rows(), 25);
    }

    /// Scenario: an export is enqueued with a bearer header and the generation
    /// of the token that header was built from.
    /// Guarantees: the dispatched request carries that header, and the completion
    /// reports the same generation, so a rejection invalidates exactly the token
    /// the request used.
    #[tokio::test]
    async fn push_export_sends_the_stamped_header_and_reports_its_generation() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header("authorization", "Bearer gen-7"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&mock_server)
            .await;

        otap_df_otap::crypto::ensure_crypto_provider();
        let client = LogsIngestionClient::from_parts(
            Client::new(),
            mock_server.uri(),
            create_test_metrics(),
        );

        let mut exports = InFlightExports::new(5);
        exports.push_export(
            client,
            7,
            3,
            Bytes::from_static(b"payload"),
            HeaderValue::from_static("Bearer gen-7"),
            42,
        );
        assert_eq!(exports.queued_rows(), 3);

        let completed = exports.next_completion().await.expect("export completes");

        assert_eq!(completed.batch_id, 7);
        assert_eq!(completed.token_generation, 42);
        assert_eq!(completed.row_count, 3);
        assert_eq!(completed.body_size_bytes, 7);
        assert!(
            completed.result.is_ok(),
            "expected success, got {:?}",
            completed.result
        );
        assert_eq!(exports.queued_rows(), 0);
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
        exports.push(mock_completed_export_future(1, 1, true));
        exports.push(mock_completed_export_future(2, 1, true));
        exports.push(mock_completed_export_future(3, 1, true));

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
        exports.push(mock_completed_export_future(1, 20, true));
        exports.push(mock_completed_export_future(2, 40, false));

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

        exports.push(mock_completed_export_future(42, 1, true));

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
        exports.push(mock_completed_export_future(1, 30, true));

        let completed = exports.next_completion().await.unwrap();
        assert_eq!(completed.row_count, 30);
        assert_eq!(exports.queued_rows(), 70);
    }

    #[tokio::test]
    async fn test_next_completion_failure_decrements_queued_rows() {
        let mut exports = InFlightExports::new(5);
        exports.queued_rows = 40;
        exports.push(mock_completed_export_future(1, 40, false));

        let completed = exports.next_completion().await.unwrap();
        assert!(completed.result.is_err());
        assert_eq!(exports.queued_rows(), 0);
    }

    #[tokio::test]
    async fn test_next_completion_queued_rows_saturates_on_underflow() {
        let mut exports = InFlightExports::new(5);
        // queued_rows is 0 but a completion reports 5 records; saturating_sub
        // must keep it at 0 rather than wrapping around.
        exports.push(mock_completed_export_future(1, 5, true));
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

        exports.push(pending1);
        exports.push(pending2);

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
            exports.push(pending);
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
