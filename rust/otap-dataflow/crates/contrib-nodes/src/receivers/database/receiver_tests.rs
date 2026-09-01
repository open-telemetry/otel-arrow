// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::receivers::database::{
    CellValue, CheckpointConfig, ColumnMetadata, CursorRow, OnNack, OutputConfig, PollingConfig,
    QueryPage, Row, SourceLease, TieBreakerCursorConfig, TimestampCursorConfig, WatermarkConfig,
};
use otel_arrow_dfe_channel::mpsc::Channel;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::control::AckMsg;
use otel_arrow_dfe_engine::local::message::LocalReceiver;
use otel_arrow_dfe_engine::message::Receiver;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::testing::{receiver::TestRuntime, test_node};
use otel_arrow_dfe_otap::testing::next_ack;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

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

struct FakeAdapter;

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

    fn system(&self) -> super::super::DatabaseSystem {
        super::super::DatabaseSystem::Oracle
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
        _cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        let cursor = CompositeCursor::new("2026-01-01 00:00:00".to_owned(), 1);
        Ok(QueryPage {
            columns: fake_columns(),
            rows: vec![CursorRow {
                row: Row {
                    values: vec![
                        CellValue::Decimal("1".to_owned()),
                        CellValue::Timestamp("2026-01-01T00:00:00".to_owned()),
                    ],
                },
                cursor,
            }],
        })
    }
}

fn fake_query(checkpoint: &CheckpointConfig) -> CompiledQuery {
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
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(1),
            fetch_size: 10,
            max_rows_per_poll: 10,
            max_batch_bytes: 1024 * 1024,
            max_normalized_bytes: 1024 * 1024,
        },
        &watermark,
        checkpoint,
        OutputConfig {
            timestamp_column: Some("EVENT_TS".to_owned()),
            validation_columns: vec!["EVENT_ID".to_owned()],
            ..OutputConfig::default()
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
    let mut state = ReceiverState::new(checkpoint(3, 10), now);
    state.record_sent(checkpoint(0, 20).cursor, Duration::from_secs(1), now);
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
    let mut state = ReceiverState::new(checkpoint(3, 10), now);
    state.record_sent(checkpoint(0, 20).cursor, Duration::from_secs(1), now);
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
    let mut state = ReceiverState::new(checkpoint(0, 0), now);

    assert!(state.can_poll());
    state.record_sent(checkpoint(0, 1).cursor, Duration::from_secs(1), now);
    assert!(!state.can_poll());
}

/// Scenario: a receiver begins draining while a page is still awaiting ACK/NACK.
/// Guarantees: no new query is started during drain, so the receiver stops producing work and
/// only waits for the outstanding page to resolve.
#[test]
fn drain_stops_new_polls_until_pending_resolves() {
    let now = Instant::now();
    let mut state = ReceiverState::new(checkpoint(0, 0), now);
    state.record_sent(checkpoint(0, 1).cursor, Duration::from_secs(1), now);
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
    let mut state = ReceiverState::new(checkpoint(2, 10), now);
    state.record_sent(checkpoint(0, 20).cursor, Duration::from_secs(1), now);
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
    let mut state = ReceiverState::new(checkpoint(0, 0), now);
    state.record_sent(checkpoint(0, 1).cursor, Duration::from_secs(1), now);
    let first = state.pending.as_ref().map(|pending| pending.id);
    state.commit(checkpoint(1, 1));
    state.record_sent(checkpoint(0, 2).cursor, Duration::from_secs(1), now);
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

    assert_eq!(batch_id_from_call_data(&call_data), Some(7));
    assert_eq!(batch_id_from_call_data(&CallData::new()), None);
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

/// Scenario: A complete receiver page reaches downstream and receives a matching ACK.
/// Guarantees: The production receiver loop persists the last emitted cursor before processing
/// shutdown, proving that subscription call data, ACK correlation, and durable commit are wired
/// together rather than only tested as isolated state helpers.
#[test]
fn matching_ack_commits_the_page_through_the_receiver_loop() {
    let directory = tempfile::tempdir().expect("temporary directory");
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
    let lease = SourceLease::acquire(&store.lease_key()).expect("source lease");
    let receiver = DatabaseReceiver::new(
        FakeAdapter,
        fake_query(&checkpoint),
        store.clone(),
        lease,
        checkpoint.nack_backoff,
        checkpoint.max_consecutive_failures,
        "fake-source".to_owned(),
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
            let pdata = ctx.recv().await.expect("receiver should emit one page");
            let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK subscription frame");
            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                .await
                .expect("ACK should enqueue");
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "checkpoint committed".to_owned(),
            })
            .await
            .expect("shutdown should enqueue");
        });

    let committed = store
        .read()
        .expect("checkpoint should be readable")
        .expect("ACK should install a checkpoint");
    assert_eq!(committed.revision, 1);
    assert_eq!(committed.cursor.tie_breaker, 1);
}
