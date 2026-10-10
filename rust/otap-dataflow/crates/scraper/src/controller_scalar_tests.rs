// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::database::{CompiledWatermark, ScalarValue};

fn sequences() -> Vec<[ScalarValue; 4]> {
    vec![
        [-2, -1, 0, i64::MAX].map(ScalarValue::Int64),
        [0, i64::MAX as u64 + 1, u64::MAX - 1, u64::MAX].map(ScalarValue::UInt64),
        ["", "alpha", "beta", "gamma"].map(|key| ScalarValue::String(key.into())),
        [
            "2026-01-01T00:00:00Z",
            "2026-01-01T00:00:00.000000001Z",
            "2026-01-01T00:00:00.000000002Z",
            "2026-01-01T00:00:00.000000003Z",
        ]
        .map(|key| ScalarValue::Timestamp(key.into())),
    ]
}

/// Scenario: A scalar batch receives stale feedback, a NACK, and then a matching ACK.
/// Guarantees: No speculative progress is visible and only the acknowledged durable candidate advances state.
#[test]
fn scalar_feedback_preserves_delivery_invariants() {
    for [initial, next, _, _] in sequences() {
        let initial = Cursor::Scalar(initial);
        let next = Cursor::Scalar(next);
        let now = Instant::now();
        let mut state = ReceiverState::new(
            CheckpointState {
                revision: 0,
                cursor: initial.clone(),
            },
            now,
            CatchUpConfig::default(),
        );
        state.record_sent(next.clone());
        assert_eq!(state.committed, initial);
        assert!(!state.can_poll());
        assert!(state.ack_candidate(2).is_none());
        assert!(!state.nack(2, now));
        assert!(state.nack(1, now));
        assert_eq!(state.committed, initial);
        state.record_sent(next.clone());
        assert!(state.ack_candidate(1).is_none());
        let candidate = state.ack_candidate(2).expect("matching ACK");
        assert_eq!(state.committed, initial);
        state.commit(CheckpointState {
            revision: 1,
            cursor: candidate,
        });
        assert_eq!(state.committed, next);
        assert_eq!(state.revision, 1);
        assert!(state.can_poll());
    }
}

/// Scenario: A page includes duplicate, backward, or differently typed scalar positions before a larger final key.
/// Guarantees: Every row is checked, preventing a valid last key from hiding unsafe intermediate progress.
#[test]
fn scalar_page_order_is_strict_for_every_row() {
    let make_page = |values: Vec<ScalarValue>| QueryPage {
        columns: vec![],
        rows: values
            .into_iter()
            .map(|value| CursorRow {
                row: Row { values: vec![] },
                cursor: Cursor::Scalar(value),
            })
            .collect(),
    };
    for [initial, first, second, _] in sequences() {
        let committed = Cursor::Scalar(initial.clone());
        assert!(
            ensure_page_advanced(&committed, &make_page(vec![first.clone(), second.clone()]))
                .is_ok()
        );
        for keys in [
            vec![first.clone(), first.clone(), second.clone()],
            vec![second.clone(), first.clone()],
            vec![initial, first],
        ] {
            assert!(ensure_page_advanced(&committed, &make_page(keys)).is_err());
        }
        assert!(ensure_page_advanced(&committed, &make_page(vec![])).is_ok());
    }
    assert!(
        ensure_page_advanced(
            &Cursor::Scalar(ScalarValue::Int64(0)),
            &make_page(vec![ScalarValue::UInt64(1), ScalarValue::Int64(2)])
        )
        .is_err()
    );
}

struct ScalarAdapter {
    available: Vec<ScalarValue>,
    bound: Rc<RefCell<Vec<Cursor>>>,
}

#[async_trait(?Send)]
impl DriverAdapter for ScalarAdapter {
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
        assert!(matches!(query.watermark(), CompiledWatermark::Scalar(_)));
        Ok(vec![ColumnMetadata {
            name: "BODY".into(),
            source_type: "VARCHAR".into(),
            nullable: false,
        }])
    }
    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &Cursor,
    ) -> Result<QueryPage, Self::Error> {
        query
            .watermark()
            .validate_cursor(cursor)
            .expect("typed parameter");
        self.bound.borrow_mut().push(cursor.clone());
        let columns = self.validate_query(query).await?;
        let rows = self
            .available
            .iter()
            .find(|value| {
                Cursor::Scalar((*value).clone())
                    .compare(cursor)
                    .expect("comparable keys")
                    == std::cmp::Ordering::Greater
            })
            .map(|value| CursorRow {
                row: Row {
                    values: vec![CellValue::String("event".into())],
                },
                cursor: Cursor::Scalar(value.clone()),
            })
            .into_iter()
            .collect();
        Ok(QueryPage { columns, rows })
    }
}

fn run_session(
    store: &CheckpointStore,
    sequence: &[ScalarValue; 4],
    replay: bool,
    pages: usize,
) -> Vec<Cursor> {
    let checkpoint = CheckpointConfig {
        directory: "./state".into(),
        on_nack: OnNack::Rewind,
        on_permanent_nack: OnPermanentNack::Pause,
        nack_backoff: Duration::from_millis(1),
        max_consecutive_failures: 3,
    };
    let query = CompiledQuery::compile(
        "SELECT KEY, BODY FROM EVENTS WHERE KEY > :last_key ORDER BY KEY ASC".into(),
        PollingConfig {
            interval: Duration::from_millis(1),
            timeout: Duration::from_secs(1),
            max_rows_per_poll: 10,
            fetch_size_rows: 10,
            max_batch_bytes: 4096,
            catch_up: CatchUpConfig::default(),
        },
        &WatermarkConfig::Scalar {
            column: "KEY".into(),
            bind: "last_key".into(),
            initial: sequence[0].clone(),
        },
        &checkpoint,
        OutputConfig::default(),
    )
    .expect("query");
    let bound = Rc::new(RefCell::new(Vec::new()));
    let receiver = DatabaseReceiver::new(
        ScalarAdapter {
            available: sequence[1..].to_vec(),
            bound: Rc::clone(&bound),
        },
        query,
        SourceBinding::acquire(store.clone()).expect("lease"),
        checkpoint.nack_backoff,
        checkpoint.max_consecutive_failures,
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
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            if replay {
                let pdata = ctx.recv().await.expect("first delivery");
                let (_, nack) =
                    next_nack(NackMsg::new("retry scalar page", pdata)).expect("NACK route");
                ctx.send_control_msg(NodeControlMsg::Nack(nack))
                    .await
                    .expect("NACK");
            }
            for _ in 0..pages {
                let pdata = ctx.recv().await.expect("scalar delivery");
                let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK route");
                ctx.send_control_msg(NodeControlMsg::Ack(ack))
                    .await
                    .expect("ACK");
            }
            ctx.send_control_msg(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(2),
                reason: "scalar test complete".into(),
            })
            .await
            .expect("shutdown");
        });
    Rc::try_unwrap(bound).expect("adapter dropped").into_inner()
}

/// Scenario: Each scalar type is polled, NACKed, replayed, ACKed, and resumed after receiver restart.
/// Guarantees: Native binds resume from the durable typed value and never skip a rejected page.
#[test]
fn scalar_runtime_replays_and_resumes_all_types() {
    for sequence in sequences() {
        let directory = tempfile::tempdir_in(".").expect("state directory");
        let store = CheckpointStore::new(
            directory.path(),
            "group",
            "pipeline",
            "scalar",
            "source",
            "scalar-fingerprint".into(),
        );
        let bound = run_session(&store, &sequence, true, 2);
        assert_eq!(
            &bound[..3],
            &[
                Cursor::Scalar(sequence[0].clone()),
                Cursor::Scalar(sequence[0].clone()),
                Cursor::Scalar(sequence[1].clone()),
            ]
        );
        let saved = store
            .read()
            .expect("read checkpoint")
            .expect("acknowledged progress");
        assert_eq!(saved.revision, 2);
        assert_eq!(saved.cursor, Cursor::Scalar(sequence[2].clone()));
        let rebound = run_session(&store, &sequence, false, 1);
        assert_eq!(rebound[0], saved.cursor);
        let saved = store
            .read()
            .expect("restart checkpoint")
            .expect("next progress");
        assert_eq!(saved.revision, 3);
        assert_eq!(saved.cursor, Cursor::Scalar(sequence[3].clone()));
    }
}
