// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::oracle_receiver::{OracleReceiverConfig, tests};
use std::time::Duration;

struct DelayedProvider;

#[async_trait(?Send)]
impl BasicAuthProvider for DelayedProvider {
    async fn get_credential(
        &self,
    ) -> Result<BasicAuthCredential, otel_arrow_dfe_engine::capability::CapabilityError> {
        tokio::time::sleep(Duration::from_secs(20)).await;
        Ok(BasicAuthCredential::new("user", "password").expect("credential"))
    }

    fn credential_stream(
        &self,
    ) -> otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthCredentialStream
    {
        Box::pin(futures::stream::pending())
    }
}

fn adapter() -> OracleAdapter {
    OracleAdapter::new(
        OracleAdapterConfig {
            connect_string: String::new(),
            instant_client_dir: String::new(),
        },
        tests::test_provider(),
    )
}

fn query() -> CompiledQuery {
    let config: OracleReceiverConfig =
        serde_json::from_value(tests::documented_config()).expect("config");
    config.query()
}

// The fake native job holds its operation-owned resource until explicitly
// released. Its Drop signal is independent of test process/global state.
struct DropSignal(Option<tokio::sync::oneshot::Sender<()>>);

impl Drop for DropSignal {
    fn drop(&mut self) {
        if let Some(dropped) = self.0.take() {
            _ = dropped.send(());
        }
    }
}

struct BlockedWorker {
    release: Option<std::sync::mpsc::SyncSender<()>>,
}

impl BlockedWorker {
    fn release(&mut self) {
        self.release
            .take()
            .expect("unreleased worker")
            .send(())
            .expect("release");
    }
}

impl Drop for BlockedWorker {
    fn drop(&mut self) {
        // Also release the native thread on assertion failure.
        if let Some(release) = self.release.take() {
            _ = release.send(());
        }
    }
}

async fn block_worker(
    adapter: &mut OracleAdapter,
) -> (BlockedWorker, tokio::sync::oneshot::Receiver<()>) {
    let worker = NativeWorker::new("oracle-deadline-test").expect("worker");
    let (release, gate) = std::sync::mpsc::sync_channel(1);
    let (started, ready) = tokio::sync::oneshot::channel();
    let (dropped, drop_confirmed) = tokio::sync::oneshot::channel();
    let completion = worker
        .run(move |_| {
            let _resource = DropSignal(Some(dropped));
            _ = started.send(());
            gate.recv().expect("release native work");
        })
        .expect("native job");
    // Paused-clock tests resume real time before waiting for OS-thread work.
    tokio::time::timeout(Duration::from_secs(10), receive(ready))
        .await
        .expect("native readiness deadline")
        .expect("native readiness");
    drop(completion);
    adapter.worker = Some(worker);
    (
        BlockedWorker {
            release: Some(release),
        },
        drop_confirmed,
    )
}

async fn invoke(
    adapter: &mut OracleAdapter,
    query: &CompiledQuery,
    mode: u8,
) -> Result<(), OracleAdapterError> {
    match mode {
        0 => adapter.validate_query(query).await.map(|_| ()),
        1 => adapter.reconnect(query).await,
        _ => adapter
            .execute(query, &query.watermark().initial())
            .await
            .map(|_| ()),
    }
}

/// Scenario: Credentials consume part of each operation's budget and its native worker is still busy at expiry.
/// Guarantees: Validation, reconnect and execute share one deadline, cancel queued work before OCI, and join cleanup before retry.
#[tokio::test]
async fn query_deadline_includes_credentials_and_native_work() {
    for mode in 0..3 {
        let mut adapter = adapter();
        adapter.credentials = Box::new(DelayedProvider);
        let query = query();
        assert_eq!(query.timeout(), Duration::from_secs(30));
        let cancellation = adapter.begin_operation().expect("operation");
        let (mut blocked, dropped) = block_worker(&mut adapter).await;
        tokio::time::pause();
        {
            let operation = invoke(&mut adapter, &query, mode);
            tokio::pin!(operation);
            assert!(futures::poll!(&mut operation).is_pending());
            tokio::time::advance(Duration::from_secs(20)).await;
            assert!(futures::poll!(&mut operation).is_pending());
            tokio::time::advance(Duration::from_secs(9)).await;
            assert!(futures::poll!(&mut operation).is_pending());
            cancellation
                .ensure_not_requested()
                .expect("before deadline");
            let cancelled = cancellation.requested.notified();
            tokio::pin!(cancelled);
            assert!(futures::poll!(&mut cancelled).is_pending());
            tokio::time::advance(Duration::from_secs(1)).await;
            // Drive the timer and wait for its observable cancellation signal;
            // advancing time alone does not await timer-driver processing.
            tokio::select! {
                _ = cancelled => {}
                result = &mut operation => panic!("cleanup completed before native release: {result:?}"),
            }
            assert!(futures::poll!(&mut operation).is_pending());
            assert!(matches!(
                cancellation.ensure_not_requested(),
                Err(OracleAdapterError::Cancelled)
            ));
            // Do not auto-advance a virtual cleanup guard while OS work runs.
            tokio::time::resume();
            blocked.release();
            let error = tokio::time::timeout(Duration::from_secs(10), operation)
                .await
                .expect("cleanup deadline")
                .expect_err("query deadline");
            assert!(matches!(error, OracleAdapterError::QueryTimeout));
            assert!(OracleAdapter::is_retryable(&error));
        }
        dropped.await.expect("native resource dropped");
        let current = adapter.begin_operation().expect("safe next generation");
        cancellation.cancel().await.expect("stale handle ignored");
        current
            .ensure_not_requested()
            .expect("new generation not cancelled");
        adapter
            .shutdown()
            .await
            .expect("all native workers stopped");
    }
}

/// Scenario: A native operation does not stop during the separate cancellation/cleanup budget.
/// Guarantees: Timeout is non-retryable, new work is rejected, and shutdown cannot confirm cleanup until the worker releases its resources.
#[tokio::test]
async fn query_deadline_fails_closed_on_unconfirmed_cleanup() {
    let mut adapter = adapter();
    let query = query();
    _ = adapter.begin_operation().expect("operation");
    let (mut blocked, mut dropped) = block_worker(&mut adapter).await;
    tokio::time::pause();
    {
        let operation = adapter.validate_query(&query);
        tokio::pin!(operation);
        assert!(futures::poll!(&mut operation).is_pending());
        tokio::time::advance(query.timeout()).await;
        assert!(futures::poll!(&mut operation).is_pending());
        tokio::time::advance(QUERY_CLEANUP_TIMEOUT).await;
        let error = operation.await.expect_err("cleanup deadline");
        assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
        assert!(!OracleAdapter::is_retryable(&error));
    }
    assert!(matches!(
        adapter.begin_operation(),
        Err(OracleAdapterError::Cancelled)
    ));
    assert!(matches!(
        dropped.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));
    {
        let shutdown = adapter.shutdown();
        tokio::pin!(shutdown);
        assert!(futures::poll!(&mut shutdown).is_pending());
        tokio::time::resume();
        blocked.release();
        tokio::time::timeout(Duration::from_secs(10), shutdown)
            .await
            .expect("shutdown deadline")
            .expect("confirmed cleanup");
    }
    dropped.await.expect("native resource dropped");
}

/// Scenario: A credential provider alone outlasts the whole operation budget.
/// Guarantees: Deadline cancellation is retryable without creating native work or retaining credential acquisition.
#[tokio::test(start_paused = true)]
async fn query_deadline_bounds_credential_acquisition() {
    let mut adapter = adapter();
    adapter.credentials = Box::new(DelayedProvider);
    let mut config = tests::documented_config();
    config["query"]["timeout"] = serde_json::json!("1s");
    let query: OracleReceiverConfig = serde_json::from_value(config).expect("config");
    _ = adapter.begin_operation().expect("operation");
    let error = adapter
        .validate_query(&query.query())
        .await
        .expect_err("deadline");
    assert!(matches!(error, OracleAdapterError::QueryTimeout));
    assert!(OracleAdapter::is_retryable(&error));
    assert!(adapter.worker.is_none());
    _ = adapter.begin_operation().expect("safe next generation");
    adapter.shutdown().await.expect("shutdown");
}
