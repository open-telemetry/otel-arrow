// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::VecDeque;

/// Scenario: A database remains unavailable through many retries, then recovers.
/// Guarantees: Delays start at one second, double to a thirty-second cap without overflow,
/// and only successful polling resets the schedule.
#[test]
fn database_backoff_is_capped_and_resets_after_recovery() {
    let now = Instant::now();
    let mut retry = DatabaseRetry::default();
    for seconds in [1, 2, 4, 8, 16, 30, 30, 30] {
        assert_eq!(retry.schedule(now), Duration::from_secs(seconds));
        assert_eq!(retry.retry_at, Some(now + Duration::from_secs(seconds)));
    }
    retry.failures = u64::MAX;
    assert_eq!(retry.schedule(now), DATABASE_RETRY_MAX);
    assert_eq!(retry.failures, u64::MAX);
    retry.recovered("test-source");
    assert_eq!(retry.failures, 0);
    assert!(retry.retry_at.is_none());
    assert_eq!(retry.schedule(now), DATABASE_RETRY_INITIAL);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Begin,
    Validate,
    Execute,
    Reconnect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Success,
    Transient,
    Permanent,
    InvalidMapping,
    WaitForCancellation,
}

#[derive(Debug, thiserror::Error)]
enum RecoveryError {
    #[error("test transient database failure")]
    Transient,
    #[error("test terminal database failure")]
    Permanent,
}

#[derive(Clone)]
struct RecoveryCancellation(Rc<Cell<bool>>);

#[async_trait(?Send)]
impl DriverCancellation for RecoveryCancellation {
    type Error = RecoveryError;

    async fn cancel(&self) -> Result<(), Self::Error> {
        self.0.set(true);
        Ok(())
    }
}

struct RecoveryAdapter {
    inner: FakeAdapter,
    steps: Rc<RefCell<VecDeque<(Phase, Action)>>>,
    calls: Rc<RefCell<Vec<(Phase, Instant)>>>,
    cursors: Rc<RefCell<Vec<CompositeCursor>>>,
    cancelled: Rc<Cell<bool>>,
}

impl RecoveryAdapter {
    fn step(&mut self, phase: Phase) -> Action {
        let (expected, action) = self
            .steps
            .borrow_mut()
            .pop_front()
            .expect("expected operation");
        assert_eq!(
            phase, expected,
            "reconnect and revalidation precede execution"
        );
        self.calls.borrow_mut().push((phase, Instant::now()));
        action
    }

    fn result(action: Action) -> Result<(), RecoveryError> {
        match action {
            Action::Success => Ok(()),
            Action::Transient => Err(RecoveryError::Transient),
            Action::Permanent => Err(RecoveryError::Permanent),
            _ => panic!("action requires its phase-specific handler"),
        }
    }
}

#[async_trait(?Send)]
impl DriverAdapter for RecoveryAdapter {
    type Error = RecoveryError;
    type Cancellation = RecoveryCancellation;

    fn system(&self) -> DatabaseSystem {
        self.inner.system()
    }

    fn begin_operation(&mut self) -> Result<Self::Cancellation, Self::Error> {
        self.cancelled.set(false);
        if self
            .steps
            .borrow()
            .front()
            .is_some_and(|(phase, _)| *phase == Phase::Begin)
        {
            Self::result(self.step(Phase::Begin))?;
        }
        Ok(RecoveryCancellation(Rc::clone(&self.cancelled)))
    }

    fn is_retryable(error: &Self::Error) -> bool {
        matches!(error, RecoveryError::Transient)
    }

    async fn reconnect(&mut self, _query: &CompiledQuery) -> Result<(), Self::Error> {
        let action = self.step(Phase::Reconnect);
        if action == Action::WaitForCancellation {
            while !self.cancelled.get() {
                tokio::task::yield_now().await;
            }
            return Err(RecoveryError::Transient);
        }
        Self::result(action)
    }

    async fn validate_query(
        &mut self,
        _query: &CompiledQuery,
    ) -> Result<Vec<ColumnMetadata>, Self::Error> {
        let action = self.step(Phase::Validate);
        if action == Action::InvalidMapping {
            return Ok(Vec::new());
        }
        Self::result(action)?;
        Ok(fake_columns())
    }

    async fn execute(
        &mut self,
        query: &CompiledQuery,
        cursor: &CompositeCursor,
    ) -> Result<QueryPage, Self::Error> {
        self.cursors.borrow_mut().push(cursor.clone());
        Self::result(self.step(Phase::Execute))?;
        self.inner
            .execute(query, cursor)
            .await
            .map_err(|_| RecoveryError::Permanent)
    }

    async fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.inner
            .shutdown()
            .await
            .map_err(|_| RecoveryError::Permanent)
    }

    fn classify_error(error: &Self::Error) -> ReceiverErrorKind {
        match error {
            RecoveryError::Transient => ReceiverErrorKind::Transport,
            RecoveryError::Permanent => ReceiverErrorKind::Configuration,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    Startup,
    Execute,
    Begin,
    ExecuteSetup,
    Revalidate,
    Recurrent,
    PermanentValidate,
    PermanentExecute,
    PermanentReconnect,
    InvalidMapping,
    DrainBackoff,
    ShutdownReconnect,
    MemoryPressure,
}

impl Case {
    fn script(self) -> VecDeque<(Phase, Action)> {
        use Action::{Permanent as Fail, Success as Ok, Transient as Retry};
        use Phase::{Begin, Execute, Reconnect, Validate};
        match self {
            Self::Startup => vec![
                (Validate, Retry),
                (Reconnect, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::Execute => vec![
                (Validate, Ok),
                (Execute, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::Begin => vec![
                (Begin, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::ExecuteSetup => vec![
                (Validate, Ok),
                (Begin, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::Revalidate => vec![
                (Validate, Ok),
                (Execute, Retry),
                (Reconnect, Ok),
                (Validate, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::Recurrent => vec![
                (Validate, Ok),
                (Execute, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
                (Execute, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
            Self::PermanentValidate => vec![(Validate, Fail)],
            Self::PermanentExecute => vec![(Validate, Ok), (Execute, Fail)],
            Self::PermanentReconnect => vec![(Validate, Retry), (Reconnect, Fail)],
            Self::InvalidMapping => vec![
                (Validate, Ok),
                (Execute, Retry),
                (Reconnect, Ok),
                (Validate, Action::InvalidMapping),
            ],
            Self::DrainBackoff => vec![(Validate, Retry)],
            Self::ShutdownReconnect => {
                vec![(Validate, Retry), (Reconnect, Action::WaitForCancellation)]
            }
            Self::MemoryPressure => vec![
                (Validate, Retry),
                (Reconnect, Ok),
                (Validate, Ok),
                (Execute, Ok),
            ],
        }
        .into()
    }

    fn terminal(self) -> bool {
        matches!(
            self,
            Self::PermanentValidate
                | Self::PermanentExecute
                | Self::PermanentReconnect
                | Self::InvalidMapping
        )
    }

    fn stopped(self) -> bool {
        matches!(self, Self::DrainBackoff | Self::ShutdownReconnect)
    }
}

struct RecoveryProbe {
    receiver: DatabaseReceiver<RecoveryAdapter>,
    terminal: bool,
    finished: Rc<Cell<bool>>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for RecoveryProbe {
    async fn start(
        self: Box<Self>,
        controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let result = tokio::time::timeout(
            Duration::from_secs(12),
            local::Receiver::start(Box::new(self.receiver), controls, effects),
        )
        .await
        .expect("bounded recovery test");
        self.finished.set(true);
        if self.terminal {
            assert!(matches!(
                result,
                Err(Error::ReceiverError {
                    kind: ReceiverErrorKind::Configuration,
                    ..
                })
            ));
            Ok(TerminalState::default())
        } else {
            result
        }
    }
}

fn run_recovery(case: Case) {
    let directory = tempfile::tempdir_in(".").expect("checkpoint directory");
    let config = CheckpointConfig {
        directory: directory.path().to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_millis(10),
        // Database recovery must not consume the checkpoint failure budget.
        max_consecutive_failures: 1,
    };
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "recovery",
        "source",
        "fingerprint".to_owned(),
    );
    let source = SourceBinding::acquire(store.clone()).expect("source binding");
    let (previous, _) = store
        .write(0, &checkpoint(0, 41).cursor)
        .expect("saved progress");
    let steps = Rc::new(RefCell::new(case.script()));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let cursors = Rc::new(RefCell::new(Vec::new()));
    let joined = Rc::new(Cell::new(false));
    let cancelled = Rc::new(Cell::new(false));
    let finished = Rc::new(Cell::new(false));
    let pipeline = create_test_pipeline_context();
    let receiver = DatabaseReceiver::new(
        RecoveryAdapter {
            inner: FakeAdapter {
                shutdown_joined: Rc::clone(&joined),
                lease_key: store.lease_key().to_path_buf(),
            },
            steps: Rc::clone(&steps),
            calls: Rc::clone(&calls),
            cursors: Rc::clone(&cursors),
            cancelled: Rc::clone(&cancelled),
        },
        query_with_polling(
            &config,
            if case == Case::Recurrent {
                Duration::from_millis(1)
            } else {
                Duration::from_secs(60)
            },
            Some(CatchUpConfig {
                max_pages: 1,
                max_duration: Duration::from_secs(10),
            }),
        ),
        source,
        config.nack_backoff,
        config.max_consecutive_failures,
        normal_admission(),
        Some(DatabaseReceiverMetrics::register(&pipeline)),
    );
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        RecoveryProbe {
            receiver,
            terminal: case.terminal(),
            finished: Rc::clone(&finished),
        },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:recovery_test",
        )),
        runtime.config(),
    );
    let observed = Rc::clone(&calls);
    let active = Rc::clone(&finished);
    let saved = previous.clone();
    let stored = store.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            tokio::time::timeout(Duration::from_secs(12), async {
                if case.terminal() {
                    assert!(
                        ctx.recv().await.is_err(),
                        "terminal failures never emit a page"
                    );
                    return;
                }
                while observed.borrow().is_empty() {
                    tokio::task::yield_now().await;
                }
                // The real lease and checkpoint remain unchanged while the receiver is recovering.
                let worker = ScraperWorker::new().expect("checkpoint probe worker");
                let key = stored.lease_key().to_path_buf();
                assert!(
                    worker
                        .run(move || SourceLease::acquire(&key))
                        .expect("lease probe")
                        .await
                        .expect("lease probe joined")
                        .is_err()
                );
                assert_eq!(stored_checkpoint(&worker, &stored).await, Some(saved));
                worker
                    .stop(Instant::now() + Duration::from_secs(1))
                    .await
                    .expect("probe cleanup");
                if case == Case::MemoryPressure {
                    ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                        .await
                        .expect("hard pressure");
                    ctx.send_control_msg(pressure(1, MemoryPressureLevel::Normal))
                        .await
                        .expect("stale normal");
                    tokio::time::sleep(Duration::from_millis(1200)).await;
                    assert_eq!(
                        observed.borrow().len(),
                        1,
                        "pressure blocks reconnect after backoff elapses"
                    );
                    assert!(!active.get(), "outage must not end the receiver");
                    ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                        .await
                        .expect("pressure recovery");
                }
                if case == Case::ShutdownReconnect {
                    while !observed
                        .borrow()
                        .iter()
                        .any(|(phase, _)| *phase == Phase::Reconnect)
                    {
                        tokio::task::yield_now().await;
                    }
                }
                if !case.stopped() {
                    let page_count = if case == Case::Recurrent { 2 } else { 1 };
                    for page in 0..page_count {
                        let pdata = ctx.recv().await.expect("recovered page");
                        assert_page_id(&pdata, 42 + page);
                        assert!(!active.get(), "recovery stays inside the original receiver");
                        if page + 1 < page_count {
                            let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK first page");
                            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                                .await
                                .expect("commit before next outage");
                            continue;
                        }
                        let reconnect_count = observed
                            .borrow()
                            .iter()
                            .filter(|(phase, _)| *phase == Phase::Reconnect)
                            .count();
                        let (reports, reporter) = MetricsReporter::create_new_and_receiver(1);
                        ctx.send_control_msg(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter,
                        })
                        .await
                        .expect("metrics");
                        let snapshot = reports.recv_async().await.expect("snapshot");
                        assert_eq!(snapshot_counter(&snapshot, "starts"), 1);
                        assert_eq!(
                            snapshot_counter(&snapshot, "reconnects"),
                            reconnect_count as u64
                        );
                        let polls = observed
                            .borrow()
                            .iter()
                            .filter(|(phase, _)| *phase == Phase::Execute)
                            .count()
                            + usize::from(case == Case::ExecuteSetup);
                        assert_eq!(snapshot_counter(&snapshot, "polls"), polls as u64);
                        assert_eq!(
                            snapshot_counter(&snapshot, "query.failures"),
                            polls as u64 - page_count,
                        );
                        let (_, ack) = next_ack(AckMsg::new(pdata)).expect("ACK route");
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("ACK recovered page");
                    }
                }
                let started = Instant::now();
                let deadline = started + Duration::from_secs(1);
                let reason = "recovery test complete".to_owned();
                let message = if case == Case::DrainBackoff {
                    NodeControlMsg::DrainIngress { deadline, reason }
                } else {
                    NodeControlMsg::Shutdown { deadline, reason }
                };
                ctx.send_control_msg(message).await.expect("stop");
                assert!(ctx.recv().await.is_err(), "no extra page during stop");
                assert!(
                    started.elapsed() < Duration::from_secs(1),
                    "backoff and reconnect remain stop-responsive"
                );
            })
            .await
            .expect("recovery remains responsive");
        });
    assert!(
        steps.borrow().is_empty(),
        "{case:?}: exact operation sequence"
    );
    assert!(joined.get(), "{case:?}: cleanup held source ownership");
    assert!(finished.get());
    if case == Case::ShutdownReconnect {
        assert!(cancelled.get(), "reconnect was cancelled and joined");
    }
    let expected_cursors: Vec<_> = if case == Case::Recurrent {
        vec![
            previous.cursor.clone(),
            previous.cursor.clone(),
            checkpoint(0, 42).cursor,
            checkpoint(0, 42).cursor,
        ]
    } else {
        vec![previous.cursor.clone(); cursors.borrow().len()]
    };
    assert_eq!(
        *cursors.borrow(),
        expected_cursors,
        "only ACKed progress changes retry cursors"
    );
    let expected = if case.terminal() || case.stopped() {
        previous
    } else {
        let count = if case == Case::Recurrent { 2 } else { 1 };
        checkpoint(previous.revision + count, 41 + count as i64)
    };
    assert_eq!(store.read().expect("checkpoint"), Some(expected));
    drop(SourceBinding::acquire(store).expect("lease released after confirmed cleanup"));
    let calls = calls.borrow();
    let mut delay = DATABASE_RETRY_INITIAL;
    let script = case.script();
    for index in 0..calls.len() {
        if calls[index].0 == Phase::Reconnect {
            assert!(
                calls[index].1.duration_since(calls[index - 1].1) >= delay,
                "{case:?}: backoff is not skipped"
            );
            delay = delay.saturating_mul(2).min(DATABASE_RETRY_MAX);
        }
        if script[index] == (Phase::Execute, Action::Success) {
            delay = DATABASE_RETRY_INITIAL;
        }
    }
}

/// Scenario: Startup, operation setup, execution, reconnect, and revalidation suffer transient failures.
/// Guarantees: Recovery stays in one receiver, reconnects and revalidates before retrying, preserves the
/// checkpoint and lease, and commits the recovered page only after its ACK.
#[test]
fn transient_database_failures_recover_without_pipeline_restart() {
    for case in [
        Case::Startup,
        Case::Execute,
        Case::Begin,
        Case::ExecuteSetup,
        Case::Revalidate,
        Case::Recurrent,
    ] {
        run_recovery(case);
    }
}

/// Scenario: Validation, execution, or reconnect fails permanently, or reconnection reveals invalid mapping.
/// Guarantees: Non-retryable failures remain explicit, never emit rows, never advance the checkpoint,
/// and clean up ownership rather than entering an infinite retry loop.
#[test]
fn terminal_database_failures_are_not_retried() {
    for case in [
        Case::PermanentValidate,
        Case::PermanentExecute,
        Case::PermanentReconnect,
        Case::InvalidMapping,
    ] {
        run_recovery(case);
    }
}

/// Scenario: Drain arrives during backoff or shutdown arrives during an active reconnect.
/// Guarantees: Backoff is interruptible, reconnect cancellation joins, and ownership is released only after cleanup.
#[test]
fn database_recovery_remains_stoppable() {
    for case in [Case::DrainBackoff, Case::ShutdownReconnect] {
        run_recovery(case);
    }
}

/// Scenario: Hard memory pressure and a stale Normal update arrive while retry backoff is pending.
/// Guarantees: No reconnect starts under pressure even after backoff expires; a fresh Normal update resumes recovery.
#[test]
fn database_recovery_honors_memory_pressure() {
    run_recovery(Case::MemoryPressure);
}
