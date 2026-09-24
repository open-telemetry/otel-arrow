// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: A candidate predates the committed timestamp but has a larger tie-breaker.
/// Guarantees: The durable composite watermark cannot move backwards.
#[test]
fn rejects_older_timestamp_with_larger_tie_breaker() {
    assert!(
        ensure_cursor_advanced(
            &CompositeCursor::new("2026-01-02 00:00:00".into(), 1),
            &CompositeCursor::new("2026-01-01 00:00:00".into(), 100),
        )
        .is_err()
    );
}
use crate::checkpoint::WriteControl;
use crate::database::{
    CellValue, CheckpointConfig, ColumnMetadata, CursorRow, DatabaseSystem, OnNack, OutputConfig,
    PollingConfig, QueryPage, Row, TieBreakerCursorConfig, TimestampCursorConfig, WatermarkConfig,
};
use otel_arrow_dfe_channel::mpsc::Channel;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_config::policy::MemoryLimiterMode;
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg};
use otel_arrow_dfe_engine::local::message::LocalReceiver;
use otel_arrow_dfe_engine::memory_limiter::{
    MemoryPressureBehaviorConfig, MemoryPressureLevel, MemoryPressureState,
};
use otel_arrow_dfe_engine::message::Receiver;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::exporter::create_test_pipeline_context;
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
use otel_arrow_dfe_otap::testing::{next_ack, next_nack};
use otel_arrow_dfe_pdata::PayloadData;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogsData;
use prost::Message;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn control_channel(message: NodeControlMsg<OtapPdata>) -> local::ControlChannel<OtapPdata> {
    let (sender, receiver) = Channel::new(1);
    sender
        .send(message)
        .expect("control channel should have capacity");
    local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)))
}

fn closed_control_channel() -> local::ControlChannel<OtapPdata> {
    let (sender, receiver) = Channel::new(1);
    drop(sender);
    local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)))
}

fn checkpoint(revision: u64, tie_breaker: i64) -> CheckpointState {
    CheckpointState {
        revision,
        cursor: CompositeCursor::new("2026-01-01 00:00:00".to_owned(), tie_breaker),
    }
}

fn normal_admission() -> LocalReceiverAdmissionState {
    LocalReceiverAdmissionState::from_process_state(&MemoryPressureState::default())
}

fn poll_admission() -> PollAdmission {
    PollAdmission {
        state: normal_admission(),
        cycle_interrupted: Cell::new(false),
    }
}

fn pressure(generation: u64, level: MemoryPressureLevel) -> NodeControlMsg<OtapPdata> {
    NodeControlMsg::MemoryPressureChanged {
        update: MemoryPressureChanged {
            generation,
            level,
            ..MemoryPressureChanged::initial()
        },
    }
}

#[derive(Clone)]
struct TestCancellation {
    cancelled: Rc<Cell<bool>>,
}

#[derive(Debug, thiserror::Error)]
#[error("test cancellation failed")]
struct TestCancellationError;

#[async_trait(?Send)]
impl DriverCancellation for TestCancellation {
    type Error = TestCancellationError;

    async fn cancel(&self) -> Result<(), Self::Error> {
        self.cancelled.set(true);
        Ok(())
    }
}

struct FakeAdapter {
    shutdown_joined: Rc<Cell<bool>>,
    lease_key: std::path::PathBuf,
}

fn fake_columns() -> Vec<ColumnMetadata> {
    vec![
        ColumnMetadata {
            name: "EVENT_ID".to_owned(),
            source_type: "NUMBER".to_owned(),
            nullable: false,
        },
        ColumnMetadata {
            name: "EVENT_TS".to_owned(),
            source_type: "TIMESTAMP".to_owned(),
            nullable: false,
        },
    ]
}

#[async_trait(?Send)]
impl DriverAdapter for FakeAdapter {
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

    async fn validate_query(
        &mut self,
        _query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        Ok(fake_columns())
    }

    async fn execute(
        &mut self,
        _query: &CompiledQuery,
        committed: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        let cursor =
            CompositeCursor::new("2026-01-01 00:00:00".to_owned(), committed.tie_breaker + 1);
        Ok(QueryPage {
            columns: fake_columns(),
            rows: vec![CursorRow {
                row: Row {
                    values: vec![
                        CellValue::Decimal(cursor.tie_breaker.to_string()),
                        CellValue::Timestamp("2026-01-01T00:00:00".to_owned()),
                    ],
                },
                cursor,
            }],
        })
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        let key = self.lease_key.clone();
        // Lease acquisition touches disk, even when used as a test assertion.
        let worker = ScraperWorker::new().expect("lease probe worker");
        let competing_lease = worker
            .run(move || SourceLease::acquire(&key))
            .expect("lease probe accepted")
            .await
            .expect("lease probe worker");
        assert!(
            competing_lease.is_err(),
            "cleanup still owns the storage lease"
        );
        worker
            .stop(Instant::now() + Duration::from_secs(1))
            .await
            .expect("lease probe stopped");
        self.shutdown_joined.set(true);
        Ok(())
    }
}

fn fake_query(checkpoint: &CheckpointConfig) -> CompiledQuery {
    query_with_polling(checkpoint, Duration::from_millis(1), None)
}

fn query_with_polling(
    checkpoint: &CheckpointConfig,
    interval: Duration,
    budget_override: Option<CatchUpConfig>,
) -> CompiledQuery {
    let watermark = WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            column: "EVENT_TS".to_owned(),
            bind: "last_timestamp".to_owned(),
            initial: "1970-01-01 00:00:00".to_owned(),
            timezone: "UTC".to_owned(),
        },
        tie_breaker: TieBreakerCursorConfig {
            column: "EVENT_ID".to_owned(),
            bind: "last_tie_breaker".to_owned(),
            initial: 0,
        },
    };
    CompiledQuery::compile(
        "SELECT EVENT_ID, EVENT_TS FROM EVENTS".to_owned(),
        PollingConfig {
            interval,
            catch_up: budget_override.unwrap_or_default(),
            timeout: Duration::from_secs(1),
            fetch_size_rows: 10,
            max_rows_per_poll: 10,
            max_batch_bytes: 1024 * 1024,
        },
        &watermark,
        checkpoint,
        OutputConfig {
            timestamp_column: Some("EVENT_TS".to_owned()),
            validation_columns: vec!["EVENT_ID".to_owned()],
        },
    )
    .expect("fake query should compile")
}

/// Scenario: Shutdown arrives after a native database operation has started.
/// Guarantees: Cancellation is requested and the operation is joined before termination is
/// reported, so a replacement receiver cannot overlap an in-flight database call.
#[tokio::test]
async fn stop_cancels_and_joins_active_operation() {
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut control = control_channel(NodeControlMsg::Shutdown {
        deadline,
        reason: "test".to_owned(),
    });
    let cancelled = Rc::new(Cell::new(false));
    let completed = Rc::new(Cell::new(false));
    let operation_cancelled = Rc::clone(&cancelled);
    let operation_completed = Rc::clone(&completed);
    let operation = async move {
        while !operation_cancelled.get() {
            tokio::task::yield_now().await;
        }
        operation_completed.set(true);
    };

    let outcome = await_database_operation_or_stop(
        operation,
        TestCancellation {
            cancelled: Rc::clone(&cancelled),
        },
        &mut control,
        &mut None,
        &Cell::new(false),
        &poll_admission(),
    )
    .await
    .expect("controlled operation should finish");

    assert!(matches!(
        outcome,
        OperationOutcome::Stopped(StopRequest::Shutdown(value)) if value == deadline
    ));
    assert!(cancelled.get());
    assert!(completed.get());
}

/// Scenario: The control channel closes while a native database operation is active.
/// Guarantees: Channel failure still cancels and joins the operation before returning an error,
/// so no orphaned blocking worker keeps a database connection open.
#[tokio::test]
async fn closed_control_channel_cancels_and_joins_active_operation() {
    let mut control = closed_control_channel();
    let cancelled = Rc::new(Cell::new(false));
    let completed = Rc::new(Cell::new(false));
    let operation_cancelled = Rc::clone(&cancelled);
    let operation_completed = Rc::clone(&completed);
    let operation = async move {
        while !operation_cancelled.get() {
            tokio::task::yield_now().await;
        }
        operation_completed.set(true);
    };

    let result = await_database_operation_or_stop(
        operation,
        TestCancellation {
            cancelled: Rc::clone(&cancelled),
        },
        &mut control,
        &mut None,
        &Cell::new(false),
        &poll_admission(),
    )
    .await;

    assert!(matches!(result, Err(Error::ChannelRecvError(_))));
    assert!(cancelled.get());
    assert!(completed.get());
}

/// Scenario: a matching ACK is durably committed after a page is sent downstream.
/// Guarantees: the in-memory cursor advances to the last emitted row only through a committed
/// checkpoint, and the in-flight slot is cleared so the next poll can start.
#[test]
fn ack_commit_advances_cursor_and_clears_pending() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(3, 10), now, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 20).cursor);
    let candidate = state.ack_candidate(1).expect("matching candidate");

    state.commit(CheckpointState {
        revision: 4,
        cursor: candidate,
    });

    assert_eq!(state.revision, 4);
    assert_eq!(state.committed.tie_breaker, 20);
    assert!(state.pending.is_none());
    assert!(state.can_poll());
}

/// Scenario: a page is negatively acknowledged while one batch is in flight.
/// Guarantees: the durable cursor is retained and the next poll is deferred to the configured
/// replay instant, so the same page is re-queried rather than skipped.
#[test]
fn nack_retains_cursor_and_schedules_replay() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(3, 10), now, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 20).cursor);
    let replay_at = now + Duration::from_secs(2);

    assert!(state.nack(1, replay_at));
    assert_eq!(state.revision, 3);
    assert_eq!(state.committed.tie_breaker, 10);
    assert_eq!(state.next_poll, replay_at);
    assert!(state.pending.is_none());
}

/// Scenario: exactly one page is emitted and awaits feedback.
/// Guarantees: no second query starts while a page is in flight, bounding both in-memory rows
/// and the number of unacknowledged rows to a single page.
#[test]
fn only_one_page_is_in_flight_per_source() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(0, 0), now, CatchUpConfig::default());

    assert!(state.can_poll());
    state.record_sent(checkpoint(0, 1).cursor);
    assert!(!state.can_poll());
}

/// Scenario: a receiver begins draining while a page is still awaiting ACK/NACK.
/// Guarantees: no new query is started during drain, so the receiver stops producing work and
/// only waits for the outstanding page to resolve.
#[test]
fn drain_stops_new_polls_until_pending_resolves() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(0, 0), now, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 1).cursor);
    state.begin_drain();

    assert!(!state.can_poll());
    assert!(state.pending.is_some());
}

/// Scenario: delayed or duplicate feedback arrives for a page that is no longer in flight.
/// Guarantees: stale feedback cannot commit, clear, or reschedule state, so a late ACK for an
/// older page can never advance the checkpoint past unacknowledged rows.
#[test]
fn stale_feedback_does_not_change_state() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(2, 10), now, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 20).cursor);
    let original_next_poll = state.next_poll;

    assert!(state.ack_candidate(2).is_none());
    assert!(!state.nack(2, now + Duration::from_secs(5)));
    assert_eq!(state.revision, 2);
    assert_eq!(state.committed.tie_breaker, 10);
    assert_eq!(state.next_poll, original_next_poll);
    assert_eq!(state.pending.as_ref().map(|pending| pending.id), Some(1));
}

/// Scenario: successive pages are emitted after each ACK commits.
/// Guarantees: batch IDs increase monotonically, so feedback for an earlier page can always be
/// distinguished from feedback for the current one.
#[test]
fn batch_ids_increase_monotonically() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(0, 0), now, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 1).cursor);
    let first = state.pending.as_ref().map(|pending| pending.id);
    state.commit(checkpoint(1, 1));
    state.record_sent(checkpoint(0, 2).cursor);
    let second = state.pending.as_ref().map(|pending| pending.id);

    assert_eq!(first, Some(1));
    assert_eq!(second, Some(2));
}

/// Scenario: a batch ID is stamped into ACK/NACK call data and read back on feedback.
/// Guarantees: the correlation identity round-trips exactly, so feedback is matched to the page
/// that produced it rather than to an arbitrary in-flight page.
#[test]
fn batch_id_round_trips_through_call_data() {
    let mut call_data = CallData::new();
    call_data.push(Context8u8::from(7_u64));
    call_data.push(Context8u8::from(42_u64));
    assert_eq!(batch_id_from_call_data(&call_data, 42), Some(7));
    assert_eq!(batch_id_from_call_data(&call_data, 43), None);
    assert_eq!(batch_id_from_call_data(&CallData::new(), 42), None);
}

/// Scenario: A query returns a candidate equal to the last durably committed cursor.
/// Guarantees: The receiver fails fast instead of ACKing and checkpointing the same page forever.
#[test]
fn equal_candidate_is_rejected_as_non_advancing() {
    let committed = CompositeCursor::new("2026-01-01 00:00:00".to_owned(), 7);

    assert!(matches!(
        ensure_cursor_advanced(&committed, &committed),
        Err(ProgressError::NonAdvancingCursor)
    ));
    assert!(matches!(
        ensure_cursor_advanced(
            &committed,
            &CompositeCursor::new("2026-01-01 00:00:00".to_owned(), 6)
        ),
        Err(ProgressError::NonAdvancingCursor)
    ));
}

/// Scenario: Successive receiver pages use the same encoder across separate blocking jobs and matching ACKs.
/// Guarantees: Both pages commit in order, cached encoding state survives the handoff, and cleanup precedes lease release.
#[test]
fn matching_acks_reuse_encoder_and_commit_pages_through_the_receiver_loop() {
    let directory = tempfile::tempdir_in(".").expect("checkpoint test directory");
    let checkpoint = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_millis(10),
        max_consecutive_failures: 3,
    };
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "fake",
        "fake-source",
        "fingerprint".to_owned(),
    );
    let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
    let shutdown_joined = Rc::new(Cell::new(false));
    let receiver = DatabaseReceiver::new(
        FakeAdapter {
            shutdown_joined: Rc::clone(&shutdown_joined),
            lease_key: store.lease_key().to_path_buf(),
        },
        fake_query(&checkpoint),
        store.clone(),
        lease,
        checkpoint.nack_backoff,
        checkpoint.max_consecutive_failures,
        "fake-source".to_owned(),
        normal_admission(),
        None,
    );
    let test_runtime = TestRuntime::<OtapPdata>::new();
    let node_config = Arc::new(NodeUserConfig::new_receiver_config(
        "urn:otel:receiver:database_test",
    ));
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(test_runtime.config().name.clone()),
        node_config,
        test_runtime.config(),
    );

    test_runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(|mut ctx| async move {
            for _ in 0..2 {
                let pdata = ctx
                    .recv()
                    .await
                    .expect("receiver should emit the next page");
                let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK subscription frame");
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("ACK should enqueue");
            }
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "checkpoint committed".to_owned(),
            })
            .await
            .expect("shutdown should enqueue");
        });

    let committed = store
        .read()
        .expect("checkpoint should be readable")
        .expect("ACK should install a checkpoint");
    assert_eq!(committed.revision, 2);
    assert_eq!(committed.cursor.tie_breaker, 2);
    assert!(shutdown_joined.get());
    drop(SourceLease::acquire(store.lease_key()).expect("lease released after receiver shutdown"));
}

struct StartupFailureProbe<A: DriverAdapter>(DatabaseReceiver<A>);

struct ClosedControlCleanupProbe {
    receiver: DatabaseReceiver<FakeAdapter>,
    write: Arc<WriteControl>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for ClosedControlCleanupProbe {
    async fn start(
        self: Box<Self>,
        mut controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let Self { receiver, write } = *self;
        let (sender, channel) = Channel::new(1);
        let replacement = local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(channel)));
        let (result, ()) = tokio::time::timeout(Duration::from_secs(3), async {
            tokio::join!(
                local::Receiver::start(Box::new(receiver), replacement, effects),
                async {
                    let ack = controls.recv().await.expect("matching ACK");
                    assert!(matches!(&ack, NodeControlMsg::Ack(_)));
                    sender.send(ack).expect("forward ACK");
                    while write.attempts.load(Ordering::SeqCst) == 0 {
                        tokio::task::yield_now().await;
                    }
                    assert_eq!(write.completed.load(Ordering::SeqCst), 0);
                    // Close while the write is unjoined, setting provisional abandonment.
                    drop(sender);
                }
            )
        })
        .await
        .expect("channel closure and worker cleanup complete");
        assert!(
            matches!(result, Err(Error::ChannelRecvError(_))),
            "successful cleanup must preserve the original channel error"
        );
        assert_eq!(write.completed.load(Ordering::SeqCst), 1);
        Ok(TerminalState::default())
    }
}

/// Scenario: The control channel closes during a checkpoint write, then both workers finish cleanup successfully.
/// Guarantees: The original channel error is preserved, failed progress is not committed, and the same lease can be reacquired without restarting.
#[test]
fn closed_control_channel_releases_lease_after_confirmed_cleanup() {
    let directory = tempfile::tempdir_in(".").expect("checkpoint directory");
    let config = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_millis(10),
        max_consecutive_failures: 3,
    };
    let mut store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "closed-control",
        "source",
        "fingerprint".to_owned(),
    );
    let lease = SourceLease::acquire(store.lease_key()).expect("initial lease");
    let (previous, _) = store
        .write(0, &checkpoint(0, 41).cursor)
        .expect("previous acknowledged progress");
    // Coordinates the receiver core with a delayed, failing filesystem worker.
    let write = Arc::new(WriteControl {
        delay: Duration::from_millis(300),
        attempts: AtomicUsize::new(0),
        completed: AtomicUsize::new(0),
    });
    store.write_control = Some(Arc::clone(&write));
    let shutdown_joined = Rc::new(Cell::new(false));
    let receiver = DatabaseReceiver::new(
        FakeAdapter {
            shutdown_joined: Rc::clone(&shutdown_joined),
            lease_key: store.lease_key().to_path_buf(),
        },
        fake_query(&config),
        store.clone(),
        lease,
        config.nack_backoff,
        config.max_consecutive_failures,
        "source".to_owned(),
        normal_admission(),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        ClosedControlCleanupProbe { receiver, write },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:closed_control_probe",
        )),
        runtime.config(),
    );
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(|mut ctx| async move {
            let pdata = ctx.recv().await.expect("next page");
            let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK subscription");
            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                .await
                .expect("ACK");
            assert!(
                tokio::time::timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("receiver exits after cleanup")
                    .is_err(),
                "no additional page after channel closure"
            );
        });
    assert!(shutdown_joined.get(), "adapter cleanup was confirmed");
    assert_eq!(store.read().expect("checkpoint read"), Some(previous));
    drop(SourceLease::acquire(store.lease_key()).expect("same source can restart in-process"));
}

#[async_trait(?Send)]
impl<A: DriverAdapter + 'static> local::Receiver<OtapPdata> for StartupFailureProbe<A> {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            local::Receiver::start(Box::new(self.0), controls, effects),
        )
        .await
        .expect("configuration failure must terminate the receiver");
        assert!(matches!(
            result,
            Err(Error::ReceiverError {
                kind: ReceiverErrorKind::Configuration,
                ..
            })
        ));
        Ok(TerminalState::default())
    }
}

/// Scenario: Startup checkpoint reading rejects a stored configuration fingerprint before any query runs.
/// Guarantees: The original read error survives cleanup, adapter shutdown still runs, and both cleanup paths permit lease release.
#[test]
fn checkpoint_read_failure_still_cleans_up_adapter_and_worker() {
    let directory = tempfile::tempdir_in(".").expect("startup checkpoint directory");
    let config = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_millis(10),
        max_consecutive_failures: 3,
    };
    let make_store = |fingerprint: &str| {
        CheckpointStore::new(
            directory.path(),
            "group",
            "pipeline",
            "startup",
            "source",
            fingerprint.to_owned(),
        )
    };
    _ = make_store("old")
        .write(0, &checkpoint(0, 1).cursor)
        .expect("prior configuration checkpoint");
    let store = make_store("new");
    let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
    let shutdown_joined = Rc::new(Cell::new(false));
    let receiver = StartupFailureProbe(DatabaseReceiver::new(
        FakeAdapter {
            shutdown_joined: Rc::clone(&shutdown_joined),
            lease_key: store.lease_key().to_path_buf(),
        },
        fake_query(&config),
        store.clone(),
        lease,
        config.nack_backoff,
        config.max_consecutive_failures,
        "source".to_owned(),
        normal_admission(),
        None,
    ));
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:startup_probe",
        )),
        runtime.config(),
    );
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation(|_| async {});
    assert!(
        shutdown_joined.get(),
        "read failure must still attempt adapter cleanup"
    );
    drop(SourceLease::acquire(store.lease_key()).expect("startup cleanup releases lease"));
}

struct EmptyMetadataAdapter {
    inner: FakeAdapter,
    columns: Vec<ColumnMetadata>,
    executions: Rc<Cell<usize>>,
}

#[async_trait(?Send)]
impl DriverAdapter for EmptyMetadataAdapter {
    type Error = TestCancellationError;
    type Cancellation = TestCancellation;

    fn system(&self) -> DatabaseSystem {
        self.inner.system()
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        self.inner.begin_operation()
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        self.inner.validate_query(query).await
    }

    async fn execute(
        &mut self,
        _query: &CompiledQuery,
        _committed: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.executions.set(self.executions.get() + 1);
        Ok(QueryPage {
            columns: self.columns.clone(),
            rows: vec![],
        })
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.inner.shutdown().await
    }
}

/// Scenario: Valid startup metadata changes to invalid execution metadata on a zero-row page.
/// Guarantees: Missing, ambiguous, or unsupported columns fail as Configuration without data or progress, and cleanup releases the lease.
#[test]
fn invalid_empty_execution_metadata_fails_and_releases_lease() {
    for case in [
        "missing_timestamp",
        "missing_validation",
        "duplicate",
        "number",
    ] {
        let mut columns = fake_columns();
        match case {
            "missing_timestamp" => columns.retain(|column| column.name != "EVENT_TS"),
            "missing_validation" => columns.retain(|column| column.name != "EVENT_ID"),
            "duplicate" => {
                let mut duplicate = columns[1].clone();
                duplicate.name = "event_ts".to_owned();
                columns.push(duplicate);
            }
            "number" => columns[1].source_type = "NUMBER".to_owned(),
            _ => unreachable!(),
        }
        let directory = tempfile::tempdir_in(".").expect("empty metadata directory");
        let config = CheckpointConfig {
            directory: directory.path().to_string_lossy().into_owned(),
            on_nack: OnNack::Rewind,
            nack_backoff: Duration::from_millis(10),
            max_consecutive_failures: 3,
        };
        let store = CheckpointStore::new(
            directory.path(),
            "group",
            "pipeline",
            "empty",
            case,
            "fingerprint".to_owned(),
        );
        let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
        let shutdown_joined = Rc::new(Cell::new(false));
        let executions = Rc::new(Cell::new(0));
        let receiver = StartupFailureProbe(DatabaseReceiver::new(
            EmptyMetadataAdapter {
                inner: FakeAdapter {
                    shutdown_joined: Rc::clone(&shutdown_joined),
                    lease_key: store.lease_key().to_path_buf(),
                },
                columns,
                executions: Rc::clone(&executions),
            },
            fake_query(&config),
            store.clone(),
            lease,
            config.nack_backoff,
            config.max_consecutive_failures,
            case.to_owned(),
            normal_admission(),
            None,
        ));
        let runtime = TestRuntime::<OtapPdata>::new();
        let wrapper = ReceiverWrapper::local(
            receiver,
            test_node(runtime.config().name.clone()),
            Arc::new(NodeUserConfig::new_receiver_config(
                "urn:otel:receiver:empty_metadata_probe",
            )),
            runtime.config(),
        );
        runtime
            .set_receiver(wrapper)
            .run_test(|_| async {})
            .run_validation(|mut ctx| async move {
                assert!(
                    tokio::time::timeout(Duration::from_secs(1), ctx.recv())
                        .await
                        .expect("receiver output closes after failure")
                        .is_err(),
                    "invalid empty page emitted data"
                );
            });
        assert_eq!(
            executions.get(),
            1,
            "{case}: startup succeeds before execution fails"
        );
        assert!(
            store.read().expect("checkpoint readable").is_none(),
            "{case}"
        );
        assert!(shutdown_joined.get(), "{case}: adapter cleanup must join");
        drop(SourceLease::acquire(store.lease_key()).expect("cleanup releases real lease"));
    }
}

/// Scenario: A worker ignores cancellation past an already-expired stop deadline.
/// Guarantees: The controller returns promptly and marks ownership for process-lifetime quarantine.
#[tokio::test]
async fn stuck_worker_quarantines_ownership_at_deadline() {
    let mut control = control_channel(NodeControlMsg::Shutdown {
        deadline: Instant::now(),
        reason: "stuck worker".to_owned(),
    });
    let abandoned = Cell::new(false);
    let outcome = await_database_operation_or_stop(
        std::future::pending::<()>(),
        NonInterruptible,
        &mut control,
        &mut None,
        &abandoned,
        &poll_admission(),
    )
    .await
    .expect("bounded shutdown");
    assert!(matches!(outcome, OperationOutcome::Stopped(_)));
    assert!(abandoned.get());
}

/// Scenario: A real OS job never returns while shutdown and Tokio runtime drop run in a child process.
/// Guarantees: Both deadlines return without joining the job, and the real storage lease stays quarantined until process exit.
#[test]
fn scraper_worker_deadline_does_not_hold_runtime_or_release_lease() {
    const CHILD: &str = "OTEL_SCRAPER_WORKER_RUNTIME_CHILD";
    if let Some(directory) = std::env::var_os(CHILD) {
        let key = std::path::PathBuf::from(directory).join("blocked-source");
        let ownership = HeldOwnership {
            lease: Some(SourceLease::acquire(&key).expect("source lease")),
            abandoned: Cell::new(false),
            cleanup_joined: Cell::new(false),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current-thread runtime");
        let worker = ScraperWorker::new().expect("scraper worker");
        // The subprocess watchdog bounds this deliberately nonreturning OS job.
        let (started, ready) = oneshot::channel();
        let operation = worker
            .run::<()>(move || {
                let _ = started.send(());
                loop {
                    std::thread::park();
                }
            })
            .expect("job accepted");
        let started_at = Instant::now();
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), ready)
                .await
                .expect("worker started promptly")
                .expect("worker started");
            let deadline = Instant::now() + Duration::from_millis(20);
            let mut control = control_channel(NodeControlMsg::Shutdown {
                deadline,
                reason: "nonreturning filesystem operation".to_owned(),
            });
            let outcome = await_database_operation_or_stop(
                operation,
                NonInterruptible,
                &mut control,
                &mut None,
                &ownership.abandoned,
                &poll_admission(),
            )
            .await
            .expect("operation stop returns");
            assert!(matches!(outcome, OperationOutcome::Stopped(_)));
            assert!(ownership.abandoned.get());
            assert!(matches!(
                worker.stop(deadline).await,
                Err(ScraperWorkerError::Deadline)
            ));
        });
        drop(runtime);
        assert!(started_at.elapsed() < Duration::from_secs(2));
        drop(ownership);
        assert!(
            SourceLease::acquire(&key).is_err(),
            "a live job must keep the real lease"
        );
        return;
    }

    let directory = tempfile::tempdir_in(".").expect("subprocess lease directory");
    let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "controller::tests::scraper_worker_deadline_does_not_hold_runtime_or_release_lease",
            "--nocapture",
        ])
        .env(
            CHILD,
            directory
                .path()
                .canonicalize()
                .expect("absolute lease directory"),
        )
        .spawn()
        .expect("runtime regression subprocess");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().expect("subprocess status") {
            assert!(status.success(), "runtime regression subprocess failed");
            break;
        }
        if Instant::now() >= deadline {
            child
                .kill()
                .expect("kill hung runtime regression subprocess");
            let _ = child.wait();
            panic!("Tokio runtime drop waited for the scraper OS job");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let key = directory.path().join("blocked-source");
    drop(SourceLease::acquire(&key).expect("process exit releases quarantine"));
}

/// Scenario: One dedicated worker job is blocked and a second job fills its only queue slot.
/// Guarantees: A third submission fails fast with Busy; accepted jobs and explicit worker exit complete after release.
#[tokio::test]
async fn scraper_worker_bounds_active_and_queued_jobs() {
    let worker = ScraperWorker::new().expect("scraper worker");
    // The gate synchronizes this local test with its one real blocking worker.
    let (release, gate) = sync_channel(1);
    let (started, ready) = oneshot::channel();
    let first = worker
        .run(move || {
            let _ = started.send(());
            gate.recv_timeout(Duration::from_secs(2))
                .expect("release active job");
            1
        })
        .expect("first job accepted");
    ready.await.expect("active job started");
    let second = worker.run(|| 2).expect("one queued job accepted");
    assert!(matches!(worker.run(|| 3), Err(ScraperWorkerError::Busy)));
    release.send(()).expect("release worker");
    assert_eq!(first.await.expect("first job completed"), 1);
    assert_eq!(second.await.expect("queued job completed"), 2);
    worker
        .stop(Instant::now() + Duration::from_secs(1))
        .await
        .expect("worker exit confirmed");
}

/// Scenario: A submitted job panics on the dedicated OS thread.
/// Guarantees: Both the job result and worker-exit acknowledgement fail instead of reporting successful cleanup.
#[tokio::test]
async fn scraper_worker_panic_is_not_successful_cleanup() {
    let worker = ScraperWorker::new().expect("scraper worker");
    let result = worker
        .run::<()>(|| panic!("injected scraper worker panic"))
        .expect("job accepted");
    assert!(result.await.is_err());
    assert!(matches!(
        worker.stop(Instant::now() + Duration::from_secs(1)).await,
        Err(ScraperWorkerError::Stopped)
    ));
}

/// Scenario: A worker's receiving endpoint has already disconnected before submission.
/// Guarantees: Submission fails explicitly with Stopped and cannot create hidden queued work.
#[test]
fn scraper_worker_disconnected_submission_fails() {
    let (jobs, requests) = sync_channel::<ScraperJob>(1);
    let (exit, exited) = oneshot::channel();
    drop(requests);
    drop(exit);
    let worker = ScraperWorker {
        jobs: Some(jobs),
        exited,
    };
    assert!(matches!(
        worker.run(|| ()),
        Err(ScraperWorkerError::Stopped)
    ));
}

/// Scenario: A real storage operation completes and the worker confirms that its job loop exited.
/// Guarantees: Ownership is held during work and released only after confirmed successful cleanup.
#[test]
fn scraper_worker_confirmed_cleanup_releases_real_lease() {
    let directory = tempfile::tempdir_in(".").expect("worker lease directory");
    let key = directory.path().join("completed-source");
    let ownership = HeldOwnership {
        lease: Some(SourceLease::acquire(&key).expect("source lease")),
        abandoned: Cell::new(false),
        cleanup_joined: Cell::new(false),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("current-thread runtime");
    runtime.block_on(async {
        let worker = ScraperWorker::new().expect("scraper worker");
        let probe = key.clone();
        assert!(
            worker
                .run(move || SourceLease::acquire(&probe))
                .expect("lease probe accepted")
                .await
                .expect("lease probe completed")
                .is_err()
        );
        worker
            .stop(Instant::now() + Duration::from_secs(1))
            .await
            .expect("worker exited");
        ownership.cleanup_joined.set(true);
    });
    drop(runtime);
    drop(ownership);
    drop(SourceLease::acquire(&key).expect("confirmed cleanup releases source"));
}

/// Scenario: A gated real encoding job runs off-core while hard and stale Normal pressure controls arrive.
/// Guarantees: Controls update admission before encoding completes, stale recovery stays rejected, and the encoded page is retained.
#[tokio::test]
async fn scraper_encoding_job_keeps_pressure_controls_responsive() {
    let worker = ScraperWorker::new().expect("scraper worker");
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "encoding-probe".to_owned(),
        OutputConfig {
            timestamp_column: Some("EVENT_TS".to_owned()),
            validation_columns: vec!["EVENT_ID".to_owned()],
        },
        fake_columns(),
    )
    .expect("encoder");
    let page = QueryPage {
        columns: fake_columns(),
        rows: vec![CursorRow {
            row: Row {
                values: vec![
                    CellValue::Decimal("1".to_owned()),
                    CellValue::Timestamp("2026-01-01T00:00:00".to_owned()),
                ],
            },
            cursor: checkpoint(0, 1).cursor,
        }],
    };
    // Keep the real encoding job active until the local core processes controls.
    let (release, gate) = sync_channel(1);
    let (started, ready) = oneshot::channel();
    let encoding = worker
        .run(move || {
            let _ = started.send(());
            gate.recv_timeout(Duration::from_secs(2))
                .expect("encoding release");
            let encoded = encoder.encode_page(page, 1, 1024 * 1024);
            (encoder, encoded)
        })
        .expect("encoding accepted");
    ready.await.expect("encoding worker active");
    let (sender, receiver) = Channel::new(2);
    sender
        .send(pressure(2, MemoryPressureLevel::Hard))
        .expect("hard pressure");
    sender
        .send(pressure(1, MemoryPressureLevel::Normal))
        .expect("stale recovery");
    let mut control = local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)));
    let admission = poll_admission();
    let abandoned = Cell::new(false);
    let mut metrics = None;
    let (outcome, ()) = tokio::join!(
        await_database_operation_or_stop(
            encoding,
            NonInterruptible,
            &mut control,
            &mut metrics,
            &abandoned,
            &admission,
        ),
        async {
            tokio::time::timeout(Duration::from_secs(1), async {
                while !admission.state.should_shed_ingress() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("core processes controls while OS job blocks");
            assert!(admission.cycle_interrupted.get());
            release.send(()).expect("release encoding");
        }
    );
    let OperationOutcome::Completed(result) = outcome.expect("encoding wait") else {
        panic!("pressure must not discard encoding");
    };
    let (_, encoded) = result.expect("encoding result");
    assert_eq!(
        encoded
            .expect("valid encoding")
            .expect("one page")
            .row_count,
        1
    );
    assert!(admission.state.should_shed_ingress());
    assert!(!abandoned.get());
    worker
        .stop(Instant::now() + Duration::from_secs(1))
        .await
        .expect("encoder worker stopped");
    drop(sender);
}

struct DropProbe(Rc<Cell<bool>>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

/// Scenario: A receiver exits with unjoined work after its deadline.
/// Guarantees: Its ownership guard is not released for an overlapping replacement.
#[test]
fn quarantined_ownership_is_retained() {
    let dropped = Rc::new(Cell::new(false));
    drop(HeldOwnership {
        lease: Some(DropProbe(Rc::clone(&dropped))),
        abandoned: Cell::new(true),
        cleanup_joined: Cell::new(true),
    });
    assert!(!dropped.get());
}

/// Scenario: A sibling node failure drops the receiver future without a stop message.
/// Guarantees: Abnormal cancellation cannot release ownership before worker cleanup joins.
#[tokio::test]
async fn dropped_receiver_future_quarantines_ownership() {
    let dropped = Rc::new(Cell::new(false));
    let lease = DropProbe(Rc::clone(&dropped));
    let mut receiver = Box::pin(async move {
        let _ownership = HeldOwnership {
            lease: Some(lease),
            abandoned: Cell::new(false),
            cleanup_joined: Cell::new(false),
        };
        std::future::pending::<()>().await;
    });
    tokio::select! {
        biased;
        () = &mut receiver => unreachable!(),
        () = tokio::task::yield_now() => {}
    }
    drop(receiver);
    assert!(!dropped.get());
}

/// Scenario: Equivalent UTC instants use different separators and fractional precision.
/// Guarantees: Advancement follows time and tie-breaker order, never string spelling.
#[test]
fn cursor_ordering_uses_utc_instants() {
    assert!(
        ensure_cursor_advanced(
            &CompositeCursor::new("2026-01-02 10:00:00".into(), 1),
            &CompositeCursor::new("2026-01-01T11:00:00Z".into(), 2),
        )
        .is_err()
    );
    assert!(
        ensure_cursor_advanced(
            &CompositeCursor::new("2026-01-01 10:00:00.0".into(), 1),
            &CompositeCursor::new("2026-01-01T10:00:00Z".into(), 1),
        )
        .is_err()
    );
    assert!(
        ensure_cursor_advanced(
            &CompositeCursor::new("2026-01-01T10:00:00.0Z".into(), 1),
            &CompositeCursor::new("2026-01-01 10:00:00".into(), 2),
        )
        .is_ok()
    );
}

/// Scenario: Either the committed or candidate timestamp cannot be parsed.
/// Guarantees: Invalid timestamp text fails closed rather than advancing by lexical order.
#[test]
fn invalid_cursor_timestamp_is_rejected() {
    let valid = CompositeCursor::new("2026-01-01 00:00:00".into(), 1);
    let invalid = CompositeCursor::new("not a timestamp".into(), 2);
    assert!(matches!(
        ensure_cursor_advanced(&valid, &invalid),
        Err(ProgressError::InvalidTimestamp)
    ));
    assert!(matches!(
        ensure_cursor_advanced(&invalid, &valid),
        Err(ProgressError::InvalidTimestamp)
    ));
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckpointStopCase {
    WriteDrain,
    WriteShutdown,
    RetryDrain,
    RetryShutdown,
    InheritedDrain,
}

struct CheckpointProbe {
    store: CheckpointStore,
    control: Arc<WriteControl>,
    case: CheckpointStopCase,
    stop_deadline: Rc<Cell<Option<Instant>>>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for CheckpointProbe {
    async fn start(
        self: Box<Self>,
        mut controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let worker = ScraperWorker::new().expect("checkpoint worker");
        let abandoned = Cell::new(false);
        let already_draining = self.case == CheckpointStopCase::InheritedDrain;
        let during_write = matches!(
            self.case,
            CheckpointStopCase::WriteDrain | CheckpointStopCase::WriteShutdown
        );
        let shutdown = matches!(
            self.case,
            CheckpointStopCase::WriteShutdown | CheckpointStopCase::RetryShutdown
        );
        let pipeline = create_test_pipeline_context();
        let mut metrics = Some(DatabaseReceiverMetrics::register(&pipeline));
        if already_draining {
            // Lifecycle counters count observed messages, not successful cleanup.
            metrics.as_mut().expect("registered metrics").drains.add(1);
            self.stop_deadline
                .set(Some(Instant::now() + Duration::from_millis(20)));
        }
        let mut failures = 0;
        let outcome = tokio::time::timeout(
            Duration::from_secs(2),
            commit_checkpoint(
                &worker,
                &self.store,
                0,
                &checkpoint(0, 1).cursor,
                1000,
                if already_draining {
                    Duration::from_millis(1)
                } else {
                    Duration::from_secs(60)
                },
                &mut failures,
                "test",
                1,
                &effects,
                &mut metrics,
                &abandoned,
                &mut controls,
                self.stop_deadline.get(),
                &poll_admission(),
            ),
        )
        .await;
        let attempts = self.control.attempts.load(Ordering::SeqCst);
        let completed = self.control.completed.load(Ordering::SeqCst);
        let retained = read_checkpoint(&worker, &self.store, &effects).await;
        worker
            .stop(Instant::now() + Duration::from_secs(1))
            .await
            .expect("checkpoint worker stopped");
        let outcome = outcome.expect("checkpoint stop must be bounded")?;
        let expected_deadline = self.stop_deadline.get().expect("stop was sent");
        assert!(
            match outcome {
                CommitOutcome::Stopped(StopRequest::Shutdown(deadline)) => {
                    shutdown && deadline == expected_deadline
                }
                CommitOutcome::Stopped(StopRequest::Drain(deadline)) => {
                    !shutdown && deadline == expected_deadline
                }
                CommitOutcome::Committed(_) => false,
            },
            "stop kind and deadline must survive checkpoint handling"
        );
        let metrics = metrics.as_ref().expect("registered metrics");
        assert_eq!(metrics.drains.get(), u64::from(!shutdown));
        assert_eq!(metrics.shutdowns.get(), u64::from(shutdown));
        assert_eq!(metrics.checkpoint_commits.get(), 0);
        assert_eq!(metrics.checkpoint_failures.get(), u64::from(failures));
        if during_write {
            assert!(abandoned.get());
            assert_eq!(completed, 0);
            assert_eq!(failures, 0);
        } else if already_draining {
            assert!(
                failures >= 2,
                "exercise repeated retries without recounting drain"
            );
            assert!(attempts < 1000);
        } else {
            assert!(!abandoned.get());
            assert_eq!(failures, 1, "stop must arrive in the first retry backoff");
            assert_eq!(attempts, 1);
            assert_eq!(completed, 1);
        }
        assert!(retained.expect("read retained checkpoint").is_none());
        Ok(TerminalState::default())
    }
}

fn run_checkpoint_probe(case: CheckpointStopCase) {
    let directory = tempfile::tempdir_in(".").expect("checkpoint test directory");
    let during_write = matches!(
        case,
        CheckpointStopCase::WriteDrain | CheckpointStopCase::WriteShutdown
    );
    // Coordinates the local test driver with the actual blocking store writer.
    let control = Arc::new(WriteControl {
        delay: if during_write {
            Duration::from_millis(200)
        } else {
            Duration::ZERO
        },
        attempts: AtomicUsize::new(0),
        completed: AtomicUsize::new(0),
    });
    let mut store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "probe",
        "source",
        "fingerprint".to_owned(),
    );
    let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
    let lease_key = store.lease_key().to_path_buf();
    store.write_control = Some(Arc::clone(&control));
    let started = Arc::clone(&control);
    let stop_deadline = Rc::new(Cell::new(None));
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        CheckpointProbe {
            store,
            control,
            case,
            stop_deadline: Rc::clone(&stop_deadline),
        },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:checkpoint_test",
        )),
        runtime.config(),
    );
    runtime
        .set_receiver(wrapper)
        .run_test(move |ctx| async move {
            if case != CheckpointStopCase::InheritedDrain {
                tokio::time::timeout(Duration::from_secs(1), async {
                    while if during_write {
                        started.attempts.load(Ordering::SeqCst) == 0
                    } else {
                        started.completed.load(Ordering::SeqCst) == 0
                    } {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("checkpoint reached requested phase");
                assert_eq!(started.attempts.load(Ordering::SeqCst), 1);
                assert_eq!(
                    started.completed.load(Ordering::SeqCst),
                    usize::from(!during_write)
                );
                let deadline = Instant::now() + Duration::from_millis(10);
                stop_deadline.set(Some(deadline));
                let reason = "checkpoint lifecycle probe".to_owned();
                let message = if matches!(
                    case,
                    CheckpointStopCase::WriteShutdown | CheckpointStopCase::RetryShutdown
                ) {
                    NodeControlMsg::Shutdown { deadline, reason }
                } else {
                    NodeControlMsg::DrainIngress { deadline, reason }
                };
                ctx.send_control_msg(message).await.expect("stop");
            }
        })
        .run_validation(|_| async {});
    // The probe explicitly confirmed its dedicated worker exited.
    drop(lease);
    drop(SourceLease::acquire(&lease_key).expect("joined checkpoint worker releases lease"));
}

/// Scenario: Drain arrives while a checkpoint write is blocked on filesystem work.
/// Guarantees: Control stays responsive, the cursor is retained, and unjoined work is identified.
#[test]
fn slow_checkpoint_write_remains_drainable() {
    run_checkpoint_probe(CheckpointStopCase::WriteDrain);
}

/// Scenario: Repeated checkpoint failures occur after an earlier drain request.
/// Guarantees: Retries stop at the active deadline, retain durable progress, and count the inherited drain only once.
#[test]
fn checkpoint_retries_honor_existing_drain_deadline() {
    run_checkpoint_probe(CheckpointStopCase::InheritedDrain);
}

/// Scenario: Drain and Shutdown are received during an active checkpoint write or its first retry backoff.
/// Guarantees: Each message counts once, preserves its stop kind/deadline, leaves no checkpoint, and joins the worker before lease release.
#[test]
fn checkpoint_write_and_retry_count_lifecycle_messages_once() {
    for case in [
        CheckpointStopCase::WriteDrain,
        CheckpointStopCase::WriteShutdown,
        CheckpointStopCase::RetryDrain,
        CheckpointStopCase::RetryShutdown,
    ] {
        run_checkpoint_probe(case);
    }
}

/// Scenario: Catch-up budgets allow one or two pages, each awaiting its matching ACK.
/// Guarantees: Only durable commits permit immediate continuation, and the last allowed page ends the cycle.
#[test]
fn catch_up_page_budget_is_ack_gated_and_exact() {
    let now = Instant::now();
    let interval = Duration::from_secs(60);
    for max_pages in [1, 2] {
        let mut state = ReceiverState::new(
            checkpoint(0, 0),
            now,
            CatchUpConfig {
                max_pages,
                max_duration: interval,
            },
        );
        for page in 1..=max_pages {
            state.begin_poll(now);
            state.record_sent(checkpoint(0, page as i64).cursor);
            assert!(!state.can_poll(), "no fetch before ACK");
            assert_eq!(state.committed.tie_breaker, (page - 1) as i64);
            assert_eq!(
                state.next_poll, now,
                "sending alone does not start an interval"
            );
            assert!(state.ack_candidate(page as u64).is_some());
            assert!(
                !state.can_poll(),
                "matching ACK still needs a durable write"
            );
            state.commit(checkpoint(page as u64, page as i64));
            state.schedule_after_commit(interval, now, false);
            assert_eq!(
                state.next_poll,
                if page < max_pages {
                    now
                } else {
                    now + interval
                }
            );
        }
        assert!(state.cycle.is_none());
    }
}

/// Scenario: An ACK completes immediately before or exactly at the elapsed catch-up budget.
/// Guarantees: ACK wait counts toward elapsed time and the exact boundary disallows another fetch.
#[test]
fn catch_up_elapsed_budget_includes_ack_wait() {
    let now = Instant::now();
    let budget = Duration::from_secs(1);
    let interval = Duration::from_secs(60);
    for elapsed in [budget - Duration::from_nanos(1), budget, budget * 2] {
        let mut state = ReceiverState::new(
            checkpoint(0, 0),
            now,
            CatchUpConfig {
                max_pages: 10,
                max_duration: budget,
            },
        );
        state.begin_poll(now);
        state.record_sent(checkpoint(0, 1).cursor);
        assert!(!state.can_poll());
        assert_eq!(state.can_continue_cycle(now + elapsed), elapsed < budget);
        state.commit(checkpoint(1, 1));
        state.schedule_after_commit(interval, now + elapsed, false);
        assert_eq!(
            state.next_poll,
            now + elapsed
                + if elapsed < budget {
                    Duration::ZERO
                } else {
                    interval
                }
        );
        assert_eq!(state.committed.tie_breaker, 1);
    }
}

/// Scenario: An empty page, interrupted send/commit, or NACK terminates a partly used catch-up cycle.
/// Guarantees: Normal delay or NACK backoff replaces immediate continuation and the next cycle gets a fresh budget.
#[test]
fn catch_up_empty_interruption_and_nack_reset_cycle() {
    let now = Instant::now();
    let interval = Duration::from_secs(60);
    for ending in ["empty", "interrupted", "nack"] {
        let mut state = ReceiverState::new(
            checkpoint(0, 0),
            now,
            CatchUpConfig {
                max_pages: 2,
                max_duration: Duration::from_secs(1),
            },
        );
        state.begin_poll(now);
        let delay = if ending == "nack" {
            Duration::from_millis(30)
        } else {
            interval
        };
        match ending {
            "empty" => state.finish_cycle(interval, now),
            "interrupted" => {
                state.record_sent(checkpoint(0, 1).cursor);
                state.commit(checkpoint(1, 1));
                state.schedule_after_commit(interval, now, true);
            }
            _ => {
                state.record_sent(checkpoint(0, 1).cursor);
                assert!(state.nack(1, now + delay));
            }
        }
        assert!(state.cycle.is_none());
        assert_eq!(state.next_poll, now + delay);
        assert_eq!(
            state.committed.tie_breaker,
            i64::from(ending == "interrupted")
        );
        state.begin_poll(now + delay);
        let cycle = state.cycle.as_ref().expect("fresh cycle");
        assert_eq!(cycle.started, now + delay);
        assert_eq!(cycle.pages_started, 1);
        assert!(state.can_continue_cycle(now + delay));
    }
}

/// Scenario: A single-page cycle receives its ACK after a long downstream wait.
/// Guarantees: The next cycle's interval starts after the durable commit, not after page send.
#[test]
fn single_page_cycle_waits_interval_after_commit() {
    let now = Instant::now();
    let interval = Duration::from_secs(60);
    let mut state = ReceiverState::new(
        checkpoint(0, 0),
        now,
        CatchUpConfig {
            max_pages: 1,
            ..CatchUpConfig::default()
        },
    );
    state.begin_poll(now);
    state.record_sent(checkpoint(0, 1).cursor);
    state.commit(checkpoint(1, 1));
    state.schedule_after_commit(interval, now + interval * 2, false);
    assert_eq!(state.next_poll, now + interval * 3);
    assert!(state.cycle.is_none());
}

/// Scenario: Hard pressure arrives during the idle interval and clears at its deadline.
/// Guarantees: Pressure pauses admission without postponing an already-scheduled next cycle.
#[test]
fn pressure_pause_preserves_idle_cycle_deadline() {
    let start = Instant::now();
    let interval = Duration::from_secs(60 * 60);
    let mut state = ReceiverState::new(checkpoint(0, 0), start, CatchUpConfig::default());
    state.record_sent(checkpoint(0, 1).cursor);
    state.commit(checkpoint(1, 1));
    state.finish_cycle(interval, start);
    let original_deadline = state.next_poll;
    let admission = poll_admission();
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "source".to_owned(),
        OutputConfig::default(),
        fake_columns(),
    )
    .expect("encoder");
    admission.apply(MemoryPressureChanged {
        generation: 1,
        level: MemoryPressureLevel::Hard,
        retry_after_secs: 1,
        usage_bytes: 0,
    });
    assert!(pause_for_pressure(
        &admission,
        &mut state,
        &mut encoder,
        interval,
        start + Duration::from_secs(59 * 60),
    ));
    assert_eq!(state.next_poll, original_deadline);
    admission.apply(MemoryPressureChanged {
        generation: 2,
        level: MemoryPressureLevel::Normal,
        retry_after_secs: 1,
        usage_bytes: 0,
    });
    assert!(!pause_for_pressure(
        &admission,
        &mut state,
        &mut encoder,
        interval,
        original_deadline,
    ));
    assert!(state.can_poll());
    assert_eq!(state.next_poll, original_deadline);
}

/// Scenario: A warmed encoder is idle while a query consumes Hard pressure and returns no rows.
/// Guarantees: The main-loop pressure gate releases scratch storage despite the empty-query early return.
#[tokio::test]
async fn pressure_during_empty_query_releases_encoder_scratch() {
    let mut encoder = OtlpPageEncoder::new(
        DatabaseSystem::Oracle,
        "source".to_owned(),
        OutputConfig::default(),
        fake_columns(),
    )
    .expect("encoder");
    _ = encoder
        .encode_page(
            QueryPage {
                columns: fake_columns(),
                rows: vec![CursorRow {
                    row: Row {
                        values: vec![
                            CellValue::Int64(1),
                            CellValue::Timestamp("2026-01-01T00:00:00Z".to_owned()),
                        ],
                    },
                    cursor: checkpoint(0, 1).cursor,
                }],
            },
            1,
            4096,
        )
        .expect("warm encoder");
    assert!(encoder.retained_record_bytes() > 0);
    let (sender, receiver) = Channel::new(1);
    sender
        .send(pressure(1, MemoryPressureLevel::Hard))
        .expect("queued pressure");
    let mut controls = local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)));
    let admission = poll_admission();
    let result = await_database_operation_or_stop(
        async {
            QueryPage {
                columns: fake_columns(),
                rows: Vec::new(),
            }
        },
        TestCancellation {
            cancelled: Rc::new(Cell::new(false)),
        },
        &mut controls,
        &mut None,
        &Cell::new(false),
        &admission,
    )
    .await
    .expect("empty query completes");
    assert!(matches!(result, OperationOutcome::Completed(page) if page.is_empty()));
    assert!(admission.state.should_shed_ingress());
    let now = Instant::now();
    let interval = Duration::from_secs(60);
    let mut state = ReceiverState::new(
        checkpoint(1, 1),
        now,
        CatchUpConfig {
            max_pages: 3,
            max_duration: Duration::from_secs(5),
        },
    );
    state.finish_cycle(interval, now);
    assert!(pause_for_pressure(
        &admission,
        &mut state,
        &mut encoder,
        interval,
        now,
    ));
    assert_eq!(encoder.retained_record_bytes(), 0);
    drop(sender);
}

/// Scenario: Pressure changes arrive while a database, checkpoint-read, or encoding operation is waiting.
/// Guarantees: Waiters apply updates without cancelling work; stale recovery cannot reopen admission and hard interruption is sticky.
#[tokio::test]
async fn operation_wait_applies_pressure_and_retains_interruption() {
    for levels in [
        vec![
            (2, MemoryPressureLevel::Hard),
            (1, MemoryPressureLevel::Normal),
        ],
        vec![
            (2, MemoryPressureLevel::Hard),
            (3, MemoryPressureLevel::Normal),
        ],
        vec![(1, MemoryPressureLevel::Soft)],
    ] {
        let (sender, receiver) = Channel::new(4);
        for &(generation, level) in &levels {
            sender
                .send(pressure(generation, level))
                .expect("queued update");
        }
        let mut control =
            local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(receiver)));
        let admission = poll_admission();
        let cancelled = Rc::new(Cell::new(false));
        let outcome = await_database_operation_or_stop(
            tokio::time::sleep(Duration::from_millis(10)),
            TestCancellation {
                cancelled: Rc::clone(&cancelled),
            },
            &mut control,
            &mut None,
            &Cell::new(false),
            &admission,
        )
        .await
        .expect("operation completes");
        assert!(matches!(outcome, OperationOutcome::Completed(())));
        assert!(!cancelled.get());
        assert_eq!(admission.cycle_interrupted.get(), levels.len() == 2);
        assert_eq!(
            admission.state.should_shed_ingress(),
            levels[levels.len() - 1].0 == 1 && levels.len() == 2
        );
        drop(sender);
    }
}

struct BacklogAdapter {
    inner: FakeAdapter,
    fetched: Rc<RefCell<Vec<i64>>>,
    delay: Duration,
    max_id: i64,
    release_query: Option<Rc<Cell<bool>>>,
}

#[async_trait(?Send)]
impl DriverAdapter for BacklogAdapter {
    type Error = TestCancellationError;
    type Cancellation = TestCancellation;

    fn system(&self) -> DatabaseSystem {
        self.inner.system()
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        self.inner.begin_operation()
    }

    async fn validate_query(
        &mut self,
        query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        self.inner.validate_query(query).await
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        committed: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.fetched.borrow_mut().push(committed.tie_breaker);
        if let Some(release) = &self.release_query {
            while !release.get() {
                tokio::task::yield_now().await;
            }
        }
        tokio::time::sleep(self.delay).await;
        if committed.tie_breaker >= self.max_id {
            Ok(QueryPage {
                columns: fake_columns(),
                rows: Vec::new(),
            })
        } else {
            self.inner.execute(query, committed).await
        }
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.inner.shutdown().await
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoopCase {
    DefaultBudgets,
    OnePage,
    EmptyTail,
    PageLimit,
    SlowPage,
    NackReplay,
    InitialHard,
    DefaultInitialHard,
    ObserveOnly,
    HardPending,
    HardDuringQuery,
}

async fn stored_checkpoint(
    worker: &ScraperWorker,
    store: &CheckpointStore,
) -> Option<CheckpointState> {
    let store = store.clone();
    worker
        .run(move || store.read())
        .expect("checkpoint read accepted")
        .await
        .expect("checkpoint read joins")
        .expect("checkpoint read succeeds")
}

fn assert_page_id(pdata: &OtapPdata, expected: u64) {
    let PayloadData::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(bytes)) =
        pdata.clone().payload().into_data()
    else {
        panic!("expected unchanged OTLP logs protobuf transport");
    };
    let logs = LogsData::decode(bytes).expect("valid OTLP logs");
    let records: Vec<_> = logs
        .resource_logs
        .iter()
        .flat_map(|resource| &resource.scope_logs)
        .flat_map(|scope| &scope.log_records)
        .collect();
    assert_eq!(records.len(), 1, "short page remains one OTLP record");
    let Some(any_value::Value::KvlistValue(body)) = records[0]
        .body
        .as_ref()
        .and_then(|value| value.value.as_ref())
    else {
        panic!("structured database body");
    };
    let id = body
        .values
        .iter()
        .find(|value| value.key == "EVENT_ID")
        .and_then(|value| value.value.as_ref())
        .and_then(|value| value.value.as_ref());
    assert_eq!(
        id,
        Some(&any_value::Value::StringValue(expected.to_string()))
    );
}

fn run_catch_up_loop(case: LoopCase) {
    let directory = tempfile::tempdir_in(".").expect("checkpoint test directory");
    let config = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_millis(50),
        max_consecutive_failures: 3,
    };
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "catch-up",
        "source",
        "fingerprint".to_owned(),
    );
    let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
    let fetched = Rc::new(RefCell::new(Vec::new()));
    let shutdown_joined = Rc::new(Cell::new(false));
    let process = MemoryPressureState::default();
    if case == LoopCase::ObserveOnly {
        process.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 1,
            fail_readiness_on_hard: false,
            mode: MemoryLimiterMode::ObserveOnly,
        });
    }
    if matches!(
        case,
        LoopCase::InitialHard | LoopCase::DefaultInitialHard | LoopCase::ObserveOnly
    ) {
        process.set_level_for_tests(MemoryPressureLevel::Hard);
    }
    let admission = LocalReceiverAdmissionState::from_process_state(&process);
    let release_query = Rc::new(Cell::new(false));
    let receiver = DatabaseReceiver::new(
        BacklogAdapter {
            inner: FakeAdapter {
                shutdown_joined: Rc::clone(&shutdown_joined),
                lease_key: store.lease_key().to_path_buf(),
            },
            fetched: Rc::clone(&fetched),
            delay: if case == LoopCase::SlowPage {
                Duration::from_millis(60)
            } else {
                Duration::ZERO
            },
            max_id: 3,
            release_query: (case == LoopCase::HardDuringQuery).then(|| Rc::clone(&release_query)),
        },
        query_with_polling(
            &config,
            Duration::from_secs(60),
            (!matches!(
                case,
                LoopCase::DefaultInitialHard | LoopCase::DefaultBudgets
            ))
            .then_some(CatchUpConfig {
                max_pages: match case {
                    LoopCase::PageLimit => 2,
                    LoopCase::OnePage => 1,
                    _ => 8,
                },
                max_duration: if case == LoopCase::SlowPage {
                    Duration::from_millis(20)
                } else {
                    Duration::from_secs(5)
                },
            }),
        ),
        store.clone(),
        lease,
        config.nack_backoff,
        config.max_consecutive_failures,
        "source".to_owned(),
        admission.clone(),
        None,
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:catch_up_test",
        )),
        runtime.config(),
    );
    let expected_pages = match case {
        LoopCase::PageLimit => 2,
        LoopCase::SlowPage
        | LoopCase::HardPending
        | LoopCase::HardDuringQuery
        | LoopCase::OnePage => 1,
        _ => 3,
    };
    let validation_store = store.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            let reader = ScraperWorker::new().expect("validation reader");
            tokio::time::timeout(Duration::from_secs(2), async {
                if matches!(case, LoopCase::InitialHard | LoopCase::DefaultInitialHard) {
                    assert!(
                        tokio::time::timeout(Duration::from_millis(30), ctx.recv())
                            .await
                            .is_err()
                    );
                    assert!(
                        fetched.borrow().is_empty(),
                        "startup hard pressure must prevent execute"
                    );
                    ctx.send_control_msg(pressure(1, MemoryPressureLevel::Normal))
                        .await
                        .expect("resume");
                }
                if case == LoopCase::HardDuringQuery {
                    while fetched.borrow().is_empty() {
                        tokio::task::yield_now().await;
                    }
                    ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                        .await
                        .expect("hard");
                    while !admission.should_shed_ingress() {
                        tokio::task::yield_now().await;
                    }
                    ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                        .await
                        .expect("recovery");
                    while admission.should_shed_ingress() {
                        tokio::task::yield_now().await;
                    }
                    release_query.set(true);
                }
                for page in 1..=expected_pages {
                    let mut pdata = ctx.recv().await.expect("next catch-up page");
                    assert_page_id(&pdata, page);
                    if page == 1 {
                        assert!(
                            stored_checkpoint(&reader, &validation_store)
                                .await
                                .is_none()
                        );
                        assert!(
                            tokio::time::timeout(Duration::from_millis(20), ctx.recv())
                                .await
                                .is_err()
                        );
                        assert_eq!(&*fetched.borrow(), &[0], "no fetch before matching ACK");
                        if case == LoopCase::NackReplay {
                            let (_, nack) = next_nack(NackMsg::new("replay", pdata))
                                .expect("NACK subscription");
                            ctx.send_control_msg(NodeControlMsg::Nack(nack))
                                .await
                                .expect("NACK");
                            assert!(
                                tokio::time::timeout(Duration::from_millis(20), ctx.recv())
                                    .await
                                    .is_err()
                            );
                            assert!(
                                stored_checkpoint(&reader, &validation_store)
                                    .await
                                    .is_none()
                            );
                            pdata = ctx.recv().await.expect("replayed page");
                            assert_page_id(&pdata, page);
                            assert_eq!(&*fetched.borrow(), &[0, 0], "NACK reuses committed cursor");
                        }
                        if case == LoopCase::HardPending {
                            ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                                .await
                                .expect("hard");
                            ctx.send_control_msg(pressure(1, MemoryPressureLevel::Normal))
                                .await
                                .expect("stale recovery");
                        }
                    }
                    let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK subscription");
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("ACK");
                }
                loop {
                    if stored_checkpoint(&reader, &validation_store)
                        .await
                        .is_some_and(|value| value.revision == expected_pages)
                    {
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                if case == LoopCase::HardPending {
                    assert!(
                        tokio::time::timeout(Duration::from_millis(20), ctx.recv())
                            .await
                            .is_err()
                    );
                    assert_eq!(
                        &*fetched.borrow(),
                        &[0],
                        "stale Normal cannot reopen admission"
                    );
                    ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                        .await
                        .expect("recovery");
                }
                let expected_fetches = match case {
                    LoopCase::PageLimit => vec![0, 1],
                    LoopCase::SlowPage
                    | LoopCase::HardPending
                    | LoopCase::HardDuringQuery
                    | LoopCase::OnePage => {
                        vec![0]
                    }
                    LoopCase::NackReplay => vec![0, 0, 1, 2, 3],
                    _ => vec![0, 1, 2, 3],
                };
                while fetched.borrow().len() < expected_fetches.len() {
                    tokio::task::yield_now().await;
                }
                assert!(
                    tokio::time::timeout(Duration::from_millis(30), ctx.recv())
                        .await
                        .is_err()
                );
                assert_eq!(
                    *fetched.borrow(),
                    expected_fetches,
                    "no extra fetch after cycle ends: {case:?}"
                );
                ctx.send_control_msg(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "catch-up validated".to_owned(),
                })
                .await
                .expect("shutdown");
            })
            .await
            .expect("catch-up completes without waiting for the 60-second interval");
            reader
                .stop(Instant::now() + Duration::from_secs(1))
                .await
                .expect("validation reader stopped");
        });
    let committed = store.read().expect("checkpoint read").expect("durable ACK");
    assert_eq!(committed.revision, expected_pages, "{case:?}");
    assert_eq!(
        committed.cursor.tie_breaker, expected_pages as i64,
        "{case:?}"
    );
    assert!(shutdown_joined.get());
    drop(SourceLease::acquire(store.lease_key()).expect("lease released after cleanup"));
}

/// Scenario: Bounded catch-up drains short pages, exhausts budgets, replays NACKs, and encounters memory pressure.
/// Guarantees: Real ACK checkpoints advance in order, empty tails are fetched once, interrupted cycles do not restart early, and cleanup holds the lease.
#[test]
fn catch_up_loop_enforces_budgets_feedback_and_memory_admission() {
    for case in [
        LoopCase::DefaultBudgets,
        LoopCase::OnePage,
        LoopCase::EmptyTail,
        LoopCase::PageLimit,
        LoopCase::SlowPage,
        LoopCase::NackReplay,
        LoopCase::InitialHard,
        LoopCase::DefaultInitialHard,
        LoopCase::ObserveOnly,
        LoopCase::HardPending,
        LoopCase::HardDuringQuery,
    ] {
        run_catch_up_loop(case);
    }
}

struct BlockedSendProbe {
    admission: Rc<PollAdmission>,
    started: Rc<Cell<bool>>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for BlockedSendProbe {
    async fn start(
        self: Box<Self>,
        mut controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let pdata = OtapPdata::new_todo_context(
            OtlpProtoBytes::ExportLogsRequest(Vec::new().into()).into(),
        );
        effects.send_message(pdata.clone()).await?;
        let now = Instant::now();
        let interval = Duration::from_secs(60);
        let mut state = ReceiverState::new(
            checkpoint(0, 0),
            now,
            CatchUpConfig {
                max_pages: 3,
                max_duration: interval,
            },
        );
        state.begin_poll(now);
        state.record_sent(checkpoint(0, 1).cursor);
        self.started.set(true);
        let outcome = send_or_stop(
            pdata,
            &mut controls,
            &effects,
            &mut state,
            &mut None,
            &mut None,
            &mut None,
            1,
            &self.admission,
        )
        .await?;
        assert!(matches!(outcome, SendOutcome::Sent));
        assert!(self.admission.cycle_interrupted.get());
        assert!(!self.admission.state.should_shed_ingress());
        state.commit(checkpoint(1, 1));
        state.schedule_after_commit(interval, now, self.admission.cycle_interrupted.get());
        assert_eq!(state.next_poll, now + interval);
        assert!(state.cycle.is_none());
        Ok(TerminalState::default())
    }
}

/// Scenario: A capacity-one output queue blocks a page while hard pressure rises and recovers.
/// Guarantees: The send finishes without dropping pdata, applies both controls, and backpressure ends immediate catch-up.
#[test]
fn blocked_send_applies_pressure_and_ends_catch_up() {
    let admission = Rc::new(poll_admission());
    let started = Rc::new(Cell::new(false));
    let runtime = TestRuntime::<OtapPdata>::new();
    let mut config = runtime.config().clone();
    config.output_pdata_channel.capacity = 1;
    let wrapper = ReceiverWrapper::local(
        BlockedSendProbe {
            admission: Rc::clone(&admission),
            started: Rc::clone(&started),
        },
        test_node(config.name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:send_probe",
        )),
        &config,
    );
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            tokio::time::timeout(Duration::from_secs(2), async {
                while !started.get() || !admission.cycle_interrupted.get() {
                    tokio::task::yield_now().await;
                }
                ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                    .await
                    .expect("hard");
                while !admission.state.should_shed_ingress() {
                    tokio::task::yield_now().await;
                }
                ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                    .await
                    .expect("normal");
                while admission.state.should_shed_ingress() {
                    tokio::task::yield_now().await;
                }
                let _ = ctx.recv().await.expect("queued page");
                let _ = ctx.recv().await.expect("blocked page is retained");
            })
            .await
            .expect("blocked sender processes controls");
        });
}

struct PressureCheckpointProbe {
    store: CheckpointStore,
    admission: Rc<PollAdmission>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for PressureCheckpointProbe {
    async fn start(
        self: Box<Self>,
        mut controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let worker = ScraperWorker::new().expect("checkpoint worker");
        let abandoned = Cell::new(false);
        let outcome = commit_checkpoint(
            &worker,
            &self.store,
            0,
            &checkpoint(0, 1).cursor,
            1000,
            Duration::from_millis(100),
            &mut 0,
            "pressure-probe",
            1,
            &effects,
            &mut None,
            &abandoned,
            &mut controls,
            None,
            &self.admission,
        )
        .await?;
        assert!(matches!(
            outcome,
            CommitOutcome::Stopped(StopRequest::Shutdown(_))
        ));
        assert!(self.admission.cycle_interrupted.get());
        assert!(!self.admission.state.should_shed_ingress());
        assert!(!abandoned.get(), "checkpoint worker must join");
        assert!(stored_checkpoint(&worker, &self.store).await.is_none());
        worker
            .stop(Instant::now() + Duration::from_secs(1))
            .await
            .expect("checkpoint worker stopped");
        Ok(TerminalState::default())
    }
}

/// Scenario: Hard pressure and recovery arrive during a delayed checkpoint write or its retry backoff.
/// Guarantees: Both wait phases apply admission updates, retain sticky interruption, and never checkpoint a failed write.
#[test]
fn checkpoint_write_and_retry_wait_apply_pressure() {
    for during_write in [true, false] {
        let directory = tempfile::tempdir_in(".").expect("checkpoint test directory");
        // The blocking store worker shares only its existing atomic fault-injection counters.
        let write = Arc::new(WriteControl {
            delay: if during_write {
                Duration::from_millis(100)
            } else {
                Duration::ZERO
            },
            attempts: AtomicUsize::new(0),
            completed: AtomicUsize::new(0),
        });
        let mut store = CheckpointStore::new(
            directory.path(),
            "group",
            "pipeline",
            "pressure-probe",
            "source",
            "fingerprint".to_owned(),
        );
        let lease = SourceLease::acquire(store.lease_key()).expect("source lease");
        store.write_control = Some(Arc::clone(&write));
        let admission = Rc::new(poll_admission());
        let runtime = TestRuntime::<OtapPdata>::new();
        let wrapper = ReceiverWrapper::local(
            PressureCheckpointProbe {
                store,
                admission: Rc::clone(&admission),
            },
            test_node(runtime.config().name.clone()),
            Arc::new(NodeUserConfig::new_receiver_config(
                "urn:otel:receiver:pressure_probe",
            )),
            runtime.config(),
        );
        runtime
            .set_receiver(wrapper)
            .run_test(|_| async {})
            .run_validation_concurrent(move |ctx| async move {
                tokio::time::timeout(Duration::from_secs(2), async {
                    while if during_write {
                        write.attempts.load(Ordering::SeqCst) == 0
                    } else {
                        write.completed.load(Ordering::SeqCst) == 0
                    } {
                        tokio::task::yield_now().await;
                    }
                    ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                        .await
                        .expect("hard");
                    while !admission.state.should_shed_ingress() {
                        tokio::task::yield_now().await;
                    }
                    if during_write {
                        assert_eq!(write.completed.load(Ordering::SeqCst), 0);
                    } else {
                        assert_eq!(write.attempts.load(Ordering::SeqCst), 1);
                    }
                    ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                        .await
                        .expect("normal");
                    while admission.state.should_shed_ingress() {
                        tokio::task::yield_now().await;
                    }
                    assert!(admission.cycle_interrupted.get());
                    ctx.send_control_msg(NodeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "pressure wait verified".to_owned(),
                    })
                    .await
                    .expect("shutdown");
                })
                .await
                .expect("checkpoint wait processes pressure");
            });
        drop(lease);
    }
}
