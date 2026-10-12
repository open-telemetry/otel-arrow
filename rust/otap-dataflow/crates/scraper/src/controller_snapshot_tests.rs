// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn snapshot_state(now: Instant) -> ReceiverState {
    ReceiverState::new(
        CheckpointState {
            revision: 0,
            cursor: Cursor::Snapshot,
        },
        now,
        CatchUpConfig::default(),
    )
}

/// Scenario: A snapshot is acknowledged while there is still ample catch-up budget.
/// Guarantees: A full polling interval follows commit instead of immediately repeating the full query.
#[test]
fn snapshot_commit_always_ends_the_cycle() {
    let now = Instant::now();
    let interval = Duration::from_secs(60);
    let mut state = snapshot_state(now);
    state.begin_poll(now);
    assert!(!state.can_continue_cycle(now));
    state.record_sent(Cursor::Snapshot);
    assert!(!state.can_poll());
    state.commit(CheckpointState {
        revision: 1,
        cursor: Cursor::Snapshot,
    });
    state.schedule_after_commit(interval, now, false);
    assert_eq!(state.next_poll, now + interval);
    assert!(state.cycle.is_none());
    assert!(state.can_poll());
}

/// Scenario: A snapshot receives stale feedback, a retryable NACK, then a permanent rejection.
/// Guarantees: No feedback advances its revision before durable commit and permanent rejection pauses the source.
#[test]
fn snapshot_feedback_preserves_uncommitted_state() {
    let now = Instant::now();
    let mut state = snapshot_state(now);
    state.record_sent(Cursor::Snapshot);
    assert!(state.ack_candidate(2).is_none());
    assert!(!state.nack(2, now));
    assert_eq!(state.revision, 0);
    let retry_at = now + Duration::from_secs(1);
    assert!(state.nack(1, retry_at));
    assert_eq!(state.next_poll, retry_at);
    state.record_sent(Cursor::Snapshot);
    assert!(state.ack_candidate(1).is_none());
    assert_eq!(state.ack_candidate(2), Some(Cursor::Snapshot));
    assert_eq!(state.revision, 0);
    assert_eq!(state.reject(OnPermanentNack::Pause, now), None);
    assert!(!state.can_poll());
    assert_eq!(state.revision, 0);
}

/// Scenario: A snapshot adapter reports a deterministic source-local failure.
/// Guarantees: Polling pauses without pending work or checkpoint advancement.
#[test]
fn snapshot_source_pause_preserves_committed_state() {
    let now = Instant::now();
    let mut state = snapshot_state(now);
    state.begin_poll(now);
    state.pause_source();
    assert!(!state.can_poll());
    assert!(state.pending.is_none());
    assert!(state.cycle.is_none());
    assert_eq!(state.revision, 0);
    assert_eq!(state.committed, Cursor::Snapshot);
    assert!(should_pause_encoding_error(
        &crate::database::OtlpMappingError::SnapshotByteLimit { limit: 4096 }
    ));
}

/// Scenario: A snapshot page repeats its no-position marker or an adapter returns a keyset cursor.
/// Guarantees: No artificial row ordering is required, but mixing checkpoint modes fails closed.
#[test]
fn snapshot_progress_requires_only_matching_mode() {
    let page = |cursor| QueryPage {
        columns: vec![],
        rows: vec![CursorRow {
            row: Row { values: vec![] },
            cursor,
        }],
    };
    assert!(ensure_page_advanced(&Cursor::Snapshot, &page(Cursor::Snapshot)).is_ok());
    let key = Cursor::Scalar(crate::database::ScalarValue::Int64(1));
    assert!(ensure_page_advanced(&Cursor::Snapshot, &page(key.clone())).is_err());
    assert!(ensure_page_advanced(&key, &page(Cursor::Snapshot)).is_err());
}

struct SnapshotAdapter {
    observed: Rc<RefCell<Vec<Cursor>>>,
}

#[async_trait(?Send)]
impl DriverAdapter for SnapshotAdapter {
    type Error = TestCancellationError;
    type Cancellation = TestCancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        Ok(TestCancellation {
            cancelled: Rc::new(Cell::new(false)),
        })
    }

    fn is_retryable(_error: &Self::Error) -> bool {
        false
    }

    async fn reconnect(&mut self, _query: &CompiledQuery) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        assert!(matches!(
            query.watermark(),
            crate::database::CompiledWatermark::Snapshot
        ));
        Ok(vec![ColumnMetadata {
            name: "BODY".into(),
            source_type: "VARCHAR2".into(),
            nullable: true,
        }])
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &Cursor,
    ) -> Result<QueryPage, Self::Error> {
        assert_eq!(cursor, &Cursor::Snapshot);
        self.observed.borrow_mut().push(cursor.clone());
        Ok(QueryPage {
            columns: self.validate_query(query).await?,
            rows: [
                CellValue::Null,
                CellValue::String("same".into()),
                CellValue::String("same".into()),
            ]
            .into_iter()
            .map(|value| CursorRow {
                row: Row {
                    values: vec![value],
                },
                cursor: Cursor::Snapshot,
            })
            .collect(),
        })
    }
}

struct PausingSnapshotAdapter {
    executions: Rc<Cell<usize>>,
    shutdown: Rc<Cell<bool>>,
}

#[async_trait(?Send)]
impl DriverAdapter for PausingSnapshotAdapter {
    type Error = TestCancellationError;
    type Cancellation = TestCancellation;

    fn system(&self) -> DatabaseSystem {
        DatabaseSystem::Oracle
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        Ok(TestCancellation {
            cancelled: Rc::new(Cell::new(false)),
        })
    }

    fn is_retryable(_error: &Self::Error) -> bool {
        false
    }

    fn should_pause_source(_error: &Self::Error) -> bool {
        true
    }

    async fn reconnect(&mut self, _query: &CompiledQuery) -> Result<(), Self::Error> {
        panic!("source-pausing errors must not reconnect");
    }

    async fn validate_query(
        &mut self,
        _query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        Ok(vec![ColumnMetadata {
            name: "BODY".into(),
            source_type: "VARCHAR2".into(),
            nullable: true,
        }])
    }

    async fn execute(
        &mut self,
        _query: &CompiledQuery,
        _cursor: &Cursor,
    ) -> Result<QueryPage, Self::Error> {
        self.executions.set(self.executions.get() + 1);
        Err(TestCancellationError)
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.shutdown.set(true);
        Ok(())
    }
}

/// Scenario: Snapshot execution exceeds a source-local bound before producing a page.
/// Guarantees: The receiver stays alive without output, retries, or checkpoint changes until shutdown.
#[test]
fn snapshot_adapter_failure_pauses_only_the_source() {
    let directory = tempfile::tempdir_in(".").expect("state directory");
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "snapshot-pause",
        "source",
        "snapshot-config".into(),
    );
    let policy = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        on_permanent_nack: OnPermanentNack::Pause,
        nack_backoff: Duration::from_millis(1),
        max_consecutive_failures: 2,
    };
    let query = CompiledQuery::compile(
        "SELECT BODY FROM EVENTS".into(),
        PollingConfig {
            interval: Duration::from_millis(10),
            timeout: Duration::from_secs(1),
            max_rows_per_poll: 10,
            fetch_size_rows: 10,
            max_batch_bytes: 4096,
            catch_up: CatchUpConfig::default(),
        },
        &WatermarkConfig::Snapshot {},
        &policy,
        OutputConfig::default(),
    )
    .expect("snapshot query");
    let executions = Rc::new(Cell::new(0));
    let shutdown = Rc::new(Cell::new(false));
    let receiver = DatabaseReceiver::new(
        PausingSnapshotAdapter {
            executions: Rc::clone(&executions),
            shutdown: Rc::clone(&shutdown),
        },
        query,
        SourceBinding::acquire(store.clone()).expect("source lease"),
        policy.nack_backoff,
        policy.max_consecutive_failures,
        normal_admission(),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:snapshot_pause_test",
        )),
        runtime.config(),
    );
    let observed_executions = Rc::clone(&executions);
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            tokio::time::timeout(Duration::from_secs(1), async {
                while observed_executions.get() == 0 {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("snapshot attempt");
            assert_eq!(observed_executions.get(), 1);
            assert!(
                tokio::time::timeout(Duration::from_millis(30), ctx.recv())
                    .await
                    .is_err(),
                "paused source stays alive without emitting a partial snapshot"
            );
            assert_eq!(observed_executions.get(), 1, "paused source does not retry");
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "stop paused snapshot source".into(),
            })
            .await
            .expect("shutdown");
            assert!(
                tokio::time::timeout(Duration::from_secs(5), ctx.recv())
                    .await
                    .expect("receiver stops")
                    .is_err(),
                "shutdown closes the paused receiver"
            );
        });
    assert!(shutdown.get());
    assert_eq!(store.read().expect("checkpoint"), None);
    drop(SourceLease::acquire(store.lease_key()).expect("source can restart"));
}

fn snapshot_session(store: &CheckpointStore, nack_first: bool, ack_count: usize) -> usize {
    let policy = CheckpointConfig {
        directory: "./state".into(),
        on_nack: OnNack::Rewind,
        on_permanent_nack: OnPermanentNack::Pause,
        nack_backoff: Duration::from_millis(1),
        max_consecutive_failures: 2,
    };
    let query = CompiledQuery::compile(
        "SELECT BODY FROM EVENTS".into(),
        PollingConfig {
            interval: Duration::from_millis(10),
            timeout: Duration::from_secs(1),
            max_rows_per_poll: 10,
            fetch_size_rows: 10,
            max_batch_bytes: 4096,
            catch_up: CatchUpConfig::default(),
        },
        &WatermarkConfig::Snapshot {},
        &policy,
        OutputConfig::default(),
    )
    .expect("snapshot query");
    let observed = Rc::new(RefCell::new(Vec::new()));
    let receiver = DatabaseReceiver::new(
        SnapshotAdapter {
            observed: Rc::clone(&observed),
        },
        query,
        SourceBinding::acquire(store.clone()).expect("source lease"),
        policy.nack_backoff,
        policy.max_consecutive_failures,
        normal_admission(),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:database_test",
        )),
        runtime.config(),
    );
    let before = store
        .read()
        .expect("read")
        .map_or(0, |state| state.revision);
    let feedback_store = store.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            if nack_first {
                let pdata = ctx.recv().await.expect("initial snapshot");
                assert_eq!(
                    feedback_store
                        .read()
                        .expect("no speculative commit")
                        .map_or(0, |s| s.revision),
                    before
                );
                let (_, nack) =
                    next_nack(NackMsg::new("retry snapshot", pdata)).expect("NACK route");
                ctx.send_control_msg(NodeControlMsg::Nack(nack))
                    .await
                    .expect("NACK");
            }
            for _ in 0..ack_count {
                let pdata = ctx.recv().await.expect("full snapshot");
                let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK route");
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("ACK");
            }
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(2),
                reason: "snapshot verification complete".into(),
            })
            .await
            .expect("shutdown");
        });
    Rc::try_unwrap(observed)
        .expect("adapter released")
        .into_inner()
        .len()
}

/// Scenario: Full snapshots are NACKed, retried, acknowledged twice, and polled again after restart.
/// Guarantees: Durable revisions count successful snapshots while every execution remains unfiltered by a column cursor.
#[test]
fn snapshot_runtime_retries_and_restarts_without_column_tracking() {
    let directory = tempfile::tempdir_in(".").expect("state directory");
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "snapshot",
        "source",
        "snapshot-config".into(),
    );
    assert_eq!(snapshot_session(&store, true, 2), 3);
    let saved = store.read().expect("read").expect("committed snapshot");
    assert_eq!(saved.revision, 2);
    assert_eq!(saved.cursor, Cursor::Snapshot);
    assert_eq!(snapshot_session(&store, false, 1), 1);
    let saved = store.read().expect("restart read").expect("next snapshot");
    assert_eq!(saved.revision, 3);
    assert_eq!(saved.cursor, Cursor::Snapshot);
}
