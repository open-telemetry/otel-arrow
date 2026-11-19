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
    block_worker_result(adapter, false).await
}

async fn block_worker_result(
    adapter: &mut OracleAdapter,
    fail: bool,
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
            assert!(!fail, "injected native worker failure");
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

fn native_break_failure(
    adapter: &mut OracleAdapter,
) -> (OracleCancellation, tokio::sync::oneshot::Receiver<()>) {
    adapter.credentials = Box::new(DelayedProvider);
    let cancellation = adapter.begin_operation().expect("operation");
    let mut state = cancellation.state.lock().expect("state");
    // An expired weak reference exercises the real cancellation worker without OCI.
    state.connection = Some(Weak::new());
    let (completed, completion) = tokio::sync::oneshot::channel();
    state.native_break_error = Some((
        OracleErrorCodes {
            oci: Some(1013),
            dpi: None,
        },
        completed,
    ));
    drop(state);
    (cancellation, completion)
}

fn short_query() -> CompiledQuery {
    let mut config = tests::documented_config();
    config["query"]["timeout"] = serde_json::json!("1s");
    let query: OracleReceiverConfig = serde_json::from_value(config).expect("config");
    query.query()
}

async fn expire<T>(
    operation: &mut (impl Future<Output = Result<T, OracleAdapterError>> + Unpin),
    cancellation: &OracleCancellation,
    timeout: Duration,
) {
    assert!(futures::poll!(&mut *operation).is_pending());
    let requested = cancellation.requested.notified();
    tokio::pin!(requested);
    assert!(futures::poll!(&mut requested).is_pending());
    tokio::time::advance(timeout).await;
    tokio::select! {
        _ = requested => {}
        _ = &mut *operation => panic!("cleanup completed before discard"),
    }
}

async fn confirm_native_break(
    cancellation: &OracleCancellation,
    completed: tokio::sync::oneshot::Receiver<()>,
) {
    // A barrier on the same native worker confirms cancellation has returned.
    // Resume real time while waiting for the OS thread, not virtual timers.
    tokio::time::resume();
    tokio::time::timeout(Duration::from_secs(10), receive(completed))
        .await
        .expect("native break completion deadline")
        .expect("native break finished");
    let barrier = cancellation
        .state
        .lock()
        .expect("state")
        .worker
        .as_ref()
        .expect("cancellation worker")
        .run(|_| ())
        .expect("barrier");
    tokio::time::timeout(Duration::from_secs(10), receive(barrier))
        .await
        .expect("cancellation completion deadline")
        .expect("cancellation joined");
    assert!(!cancellation.state.lock().expect("state").cancelling);
    tokio::time::pause();
}

/// Scenario: Native cancellation reports an error, but its worker and the operation both finish.
/// Guarantees: Expiry waits for successful session discard before becoming retryable, and stale cancellation cannot affect a new generation.
#[tokio::test]
async fn query_deadline_native_break_error_waits_for_discard() {
    let mut adapter = adapter();
    let (cancellation, completed) = native_break_failure(&mut adapter);
    let (discarded, discard) = tokio::sync::oneshot::channel();
    adapter.discard_gate = Some(discard);
    let query = short_query();
    tokio::time::pause();
    {
        let operation = adapter.validate_query(&query);
        tokio::pin!(operation);
        expire(&mut operation, &cancellation, query.timeout()).await;
        confirm_native_break(&cancellation, completed).await;
        assert!(
            futures::poll!(&mut operation).is_pending(),
            "completed native break failure must still wait for session discard"
        );
        discarded.send(Ok(())).expect("successful discard");
        let error = operation.await.expect_err("query timeout");
        assert!(matches!(error, OracleAdapterError::QueryTimeout));
        assert!(OracleAdapter::is_retryable(&error));
    }
    let current = adapter.begin_operation().expect("safe next generation");
    cancellation.cancel().await.expect("stale handle ignored");
    current.ensure_not_requested().expect("generation fenced");
    tokio::time::resume();
    adapter.shutdown().await.expect("workers stopped");
}

/// Scenario: A completed native break error is followed by hung, failed, unconfirmed, or late session discard.
/// Guarantees: Discard shares the original five-second cleanup deadline and none of these outcomes permits retry or a new operation.
#[tokio::test]
async fn query_deadline_native_break_error_requires_confirmed_discard() {
    for mode in 0..4 {
        let mut adapter = adapter();
        let (cancellation, completed) = native_break_failure(&mut adapter);
        let (discarded, discard) = tokio::sync::oneshot::channel();
        adapter.discard_gate = Some(discard);
        let query = short_query();
        tokio::time::pause();
        {
            let operation = adapter.validate_query(&query);
            tokio::pin!(operation);
            expire(&mut operation, &cancellation, query.timeout()).await;
            // Spend four seconds joining, leaving at most one for discard.
            tokio::time::advance(Duration::from_secs(4)).await;
            confirm_native_break(&cancellation, completed).await;
            assert!(futures::poll!(&mut operation).is_pending());
            match mode {
                0 => tokio::time::advance(Duration::from_secs(1)).await,
                1 => discarded
                    .send(Err(OracleAdapterError::Worker(
                        std::io::Error::from(std::io::ErrorKind::BrokenPipe).into(),
                    )))
                    .expect("failed discard"),
                2 => drop(discarded),
                _ => {
                    tokio::time::advance(Duration::from_secs(1)).await;
                    discarded.send(Ok(())).expect("late discard");
                }
            }
            let error = operation.await.expect_err("unconfirmed cleanup");
            assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
            assert!(!OracleAdapter::is_retryable(&error));
        }
        assert!(matches!(
            adapter.begin_operation(),
            Err(OracleAdapterError::Cancelled)
        ));
        tokio::time::resume();
        adapter.shutdown().await.expect("workers stopped");
    }
}

/// Scenario: Native cancellation fails but joins while the query worker is still blocked.
/// Guarantees: A break error alone never proves query completion, permits discard, or releases ownership before shutdown joins the work.
#[tokio::test]
async fn query_deadline_native_break_error_requires_joined_work() {
    let mut adapter = adapter();
    let (cancellation, completed) = native_break_failure(&mut adapter);
    adapter.credentials = tests::test_provider();
    let (mut blocked, mut dropped) = block_worker(&mut adapter).await;
    let query = short_query();
    tokio::time::pause();
    {
        let operation = adapter.validate_query(&query);
        tokio::pin!(operation);
        expire(&mut operation, &cancellation, query.timeout()).await;
        confirm_native_break(&cancellation, completed).await;
        assert!(futures::poll!(&mut operation).is_pending());
        tokio::time::advance(QUERY_CLEANUP_TIMEOUT).await;
        let error = operation.await.expect_err("query worker still blocked");
        assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
        assert!(!OracleAdapter::is_retryable(&error));
    }
    assert!(matches!(
        dropped.try_recv(),
        Err(tokio::sync::oneshot::error::TryRecvError::Empty)
    ));
    assert!(matches!(
        adapter.begin_operation(),
        Err(OracleAdapterError::Cancelled)
    ));
    tokio::time::resume();
    blocked.release();
    tokio::time::timeout(Duration::from_secs(10), adapter.shutdown())
        .await
        .expect("shutdown deadline")
        .expect("workers stopped");
    dropped.await.expect("query resource dropped");
}

/// Scenario: Session discard completes after cancellation state becomes stopped, busy, stale, or poisoned.
/// Guarantees: Successful discard cannot turn unknown or unsafe cancellation state into a retryable timeout.
#[tokio::test]
async fn query_deadline_rechecks_cancellation_state_after_discard() {
    for mode in 0..4 {
        let mut adapter = adapter();
        let (cancellation, completed) = native_break_failure(&mut adapter);
        let (discarded, discard) = tokio::sync::oneshot::channel();
        adapter.discard_gate = Some(discard);
        let query = short_query();
        tokio::time::pause();
        {
            let operation = adapter.validate_query(&query);
            tokio::pin!(operation);
            expire(&mut operation, &cancellation, query.timeout()).await;
            confirm_native_break(&cancellation, completed).await;
            assert!(futures::poll!(&mut operation).is_pending());
            let changed = std::panic::catch_unwind(|| {
                let mut state = cancellation.state.lock().expect("state");
                match mode {
                    0 => state.stopped = true,
                    1 => state.cancelling = true,
                    2 => state.generation += 1,
                    _ => panic!("injected poisoned cancellation state"),
                }
            });
            assert_eq!(changed.is_err(), mode == 3);
            discarded.send(Ok(())).expect("discard completed");
            let error = operation.await.expect_err("unsafe cancellation state");
            assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
            assert!(!OracleAdapter::is_retryable(&error));
        }
        assert!(adapter.begin_operation().is_err());
        // Restore only the test-injected state so teardown can join its worker.
        cancellation.state.clear_poison();
        cancellation.state.lock().expect("state").cancelling = false;
        tokio::time::resume();
        adapter.shutdown().await.expect("workers stopped");
    }
}

/// Scenario: The cancellation worker cannot accept a job, or another cancellation remains in progress at expiry.
/// Guarantees: Worker failure and incomplete cancellation are terminal even when the query future has returned.
#[tokio::test]
async fn query_deadline_rejects_unconfirmed_cancellation() {
    for busy in [false, true] {
        let mut adapter = adapter();
        adapter.credentials = Box::new(DelayedProvider);
        let cancellation = adapter.begin_operation().expect("operation");
        let mut worker = NativeWorker::new("oracle-stopped-cancel-test").expect("worker");
        worker.stop().await.expect("worker stopped");
        {
            let mut state = cancellation.state.lock().expect("state");
            state.connection = Some(Weak::new());
            state.worker = Some(worker);
            state.cancelling = busy;
        }
        let (discarded, discard) = tokio::sync::oneshot::channel();
        adapter.discard_gate = Some(discard);
        let query = short_query();
        tokio::time::pause();
        let error = adapter
            .validate_query(&query)
            .await
            .expect_err("unsafe cancellation");
        assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
        assert!(!OracleAdapter::is_retryable(&error));
        assert!(adapter.discard_gate.is_some(), "discard must not start");
        assert!(adapter.begin_operation().is_err());
        drop(discarded);
        cancellation.state.lock().expect("state").cancelling = false;
        tokio::time::resume();
        adapter.shutdown().await.expect("workers stopped");
    }
}

/// Scenario: The query worker fails after expiry while native cancellation itself completes successfully.
/// Guarantees: Lost query-worker completion is not ignored or downgraded to retry, and session discard is not attempted.
#[tokio::test]
async fn query_deadline_preserves_query_worker_failure() {
    let mut adapter = adapter();
    let cancellation = adapter.begin_operation().expect("operation");
    let (mut blocked, dropped) = block_worker_result(&mut adapter, true).await;
    let (discarded, discard) = tokio::sync::oneshot::channel();
    adapter.discard_gate = Some(discard);
    let query = short_query();
    tokio::time::pause();
    {
        let operation = adapter.validate_query(&query);
        tokio::pin!(operation);
        expire(&mut operation, &cancellation, query.timeout()).await;
        tokio::time::resume();
        blocked.release();
        let error = tokio::time::timeout(Duration::from_secs(10), operation)
            .await
            .expect("worker failure deadline")
            .expect_err("query worker failed");
        assert!(matches!(error, OracleAdapterError::QueryTimeoutCleanup));
        assert!(!OracleAdapter::is_retryable(&error));
    }
    dropped.await.expect("query resource dropped on failure");
    assert!(adapter.discard_gate.is_some(), "discard must not start");
    assert!(adapter.begin_operation().is_err());
    drop(discarded);
    assert!(matches!(
        adapter.shutdown().await,
        Err(OracleAdapterError::Worker(_))
    ));
}
