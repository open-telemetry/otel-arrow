// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Scenario: A rejected source pauses or repeatedly schedules explicitly enabled retry.
/// Guarantees: Pause admits no work, stale ACKs cannot commit rejected attempts, retry delay caps
/// at thirty seconds, and only a committed ACK resets rejection state.
#[test]
fn rejection_state_preserves_progress_and_bounds_retry() {
    let now = Instant::now();
    let previous = checkpoint(1, 41);
    let mut paused = ReceiverState::new(previous.clone(), now, CatchUpConfig::default());
    paused.record_sent(checkpoint(0, 42).cursor);
    assert_eq!(paused.reject(OnPermanentNack::Pause, now), None);
    assert!(paused.rejection_paused());
    assert!(!paused.can_poll());
    assert!(paused.ack_candidate(1).is_none());
    assert_eq!(paused.committed, previous.cursor);
    assert_eq!(paused.revision, previous.revision);

    let mut retrying = ReceiverState::new(previous.clone(), now, CatchUpConfig::default());
    for seconds in [1, 2, 4, 8, 16, 30, 30] {
        retrying.record_sent(checkpoint(0, 42).cursor);
        let rejected_id = retrying.pending.as_ref().expect("pending").id;
        let delay = Duration::from_secs(seconds);
        assert_eq!(retrying.reject(OnPermanentNack::Retry, now), Some(delay));
        assert_eq!(retrying.next_poll, now + delay);
        assert!(retrying.ack_candidate(rejected_id).is_none());
        assert!(retrying.can_poll());
        assert_eq!(retrying.committed, previous.cursor);
    }
    retrying.record_sent(checkpoint(0, 42).cursor);
    assert!(
        retrying.rejection.is_some(),
        "a successful query/send does not reset rejection"
    );
    retrying.commit(checkpoint(2, 42));
    assert!(retrying.rejection.is_none());
    retrying.record_sent(checkpoint(0, 43).cursor);
    assert_eq!(
        retrying.reject(OnPermanentNack::Retry, now),
        Some(RETRY_INITIAL)
    );
}

fn rejection_config(directory: &std::path::Path, policy: OnPermanentNack) -> CheckpointConfig {
    CheckpointConfig {
        directory: directory.to_string_lossy().into_owned(),
        on_nack: OnNack::Rewind,
        on_permanent_nack: policy,
        nack_backoff: Duration::from_millis(1),
        max_consecutive_failures: 3,
    }
}

fn source_store(directory: &std::path::Path, source: &str) -> CheckpointStore {
    CheckpointStore::new(
        directory,
        "group",
        "pipeline",
        "rejection",
        source,
        "fingerprint".to_owned(),
    )
}

fn receiver_for(
    store: &CheckpointStore,
    config: &CheckpointConfig,
    source: SourceBinding,
    joined: Rc<Cell<bool>>,
) -> DatabaseReceiver<FakeAdapter> {
    let pipeline = create_test_pipeline_context();
    DatabaseReceiver::new(
        FakeAdapter {
            shutdown_joined: joined,
            lease_key: store.lease_key().to_path_buf(),
        },
        fake_query(config),
        source,
        config.nack_backoff,
        config.max_consecutive_failures,
        normal_admission(),
        Some(DatabaseReceiverMetrics::register(&pipeline)),
    )
}

fn decoded_source(pdata: &OtapPdata) -> String {
    let payload = pdata.payload_ref();
    assert_eq!(payload.encoding(), Some(&PdataEncoding::OTLP));
    assert_eq!(
        payload.signal_type(),
        otel_arrow_dfe_config::SignalType::Logs
    );
    let bytes = payload.encoded_bytes().expect("OTLP logs");
    let logs = LogsData::decode(bytes.as_ref()).expect("logs");
    let resource = logs.resource_logs[0].resource.as_ref().expect("resource");
    let Some(any_value::Value::StringValue(id)) = resource
        .attributes
        .iter()
        .find(|attribute| attribute.key == "receiver.database.source_id")
        .and_then(|attribute| attribute.value.as_ref())
        .and_then(|value| value.value.as_ref())
    else {
        panic!("source identity")
    };
    id.clone()
}

struct IsolatedSourcesProbe {
    rejected: DatabaseReceiver<FakeAdapter>,
    healthy: DatabaseReceiver<FakeAdapter>,
    rejected_controls: local::ControlChannel<OtapPdata>,
    healthy_controls: local::ControlChannel<OtapPdata>,
    replacement_controls: local::ControlChannel<OtapPdata>,
    store: CheckpointStore,
    config: CheckpointConfig,
    replacement_joined: Rc<Cell<bool>>,
}

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for IsolatedSourcesProbe {
    async fn start(
        self: Box<Self>,
        _controls: local::ControlChannel<OtapPdata>,
        effects: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let Self {
            rejected,
            healthy,
            rejected_controls,
            healthy_controls,
            replacement_controls,
            store,
            config,
            replacement_joined,
        } = *self;
        let healthy_effects = effects.clone();
        let (result, _) = tokio::try_join!(
            async move {
                _ = local::Receiver::start(Box::new(rejected), rejected_controls, effects.clone())
                    .await?;
                // Simulate factory construction for a source-only restart off the pipeline core.
                let worker = ScraperWorker::new().expect("factory worker");
                let replacement_store = store.clone();
                let source = worker
                    .run(move || SourceBinding::acquire(replacement_store))
                    .expect("factory job")
                    .await
                    .expect("factory joined")
                    .expect("lease reacquired");
                worker
                    .stop(Instant::now() + Duration::from_secs(1))
                    .await
                    .expect("factory worker stopped");
                let receiver = receiver_for(&store, &config, source, replacement_joined);
                local::Receiver::start(Box::new(receiver), replacement_controls, effects).await
            },
            local::Receiver::start(Box::new(healthy), healthy_controls, healthy_effects),
        )?;
        Ok(result)
    }
}

/// Scenario: One source is permanently rejected while a second receiver continues; repair restarts only the paused source.
/// Guarantees: The paused receiver stays alive and observable with unchanged progress and held ownership,
/// the healthy receiver keeps delivering without cleanup/restart, and repaired delivery advances only after ACK.
#[test]
fn paused_source_repair_does_not_restart_healthy_receiver() {
    let directory = tempfile::tempdir_in(".").expect("checkpoint directory");
    let config = rejection_config(directory.path(), OnPermanentNack::Pause);
    let rejected_store = source_store(directory.path(), "rejected");
    let healthy_store = source_store(directory.path(), "healthy");
    let source = SourceBinding::acquire(rejected_store.clone()).expect("rejected source binding");
    let (previous, _) = rejected_store
        .write(0, &checkpoint(0, 41).cursor)
        .expect("saved cursor");
    let joined = Rc::new(Cell::new(false));
    let healthy_joined = Rc::new(Cell::new(false));
    let replacement_joined = Rc::new(Cell::new(false));
    let rejected = receiver_for(&rejected_store, &config, source, Rc::clone(&joined));
    let healthy = receiver_for(
        &healthy_store,
        &config,
        SourceBinding::acquire(healthy_store.clone()).expect("healthy binding"),
        Rc::clone(&healthy_joined),
    );
    let (rejected_tx, rejected_rx) = Channel::new(16);
    let (healthy_tx, healthy_rx) = Channel::new(16);
    let (replacement_tx, replacement_rx) = Channel::new(16);
    let channel = |rx| local::ControlChannel::new(Receiver::Local(LocalReceiver::mpsc(rx)));
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        IsolatedSourcesProbe {
            rejected,
            healthy,
            rejected_controls: channel(rejected_rx),
            healthy_controls: channel(healthy_rx),
            replacement_controls: channel(replacement_rx),
            store: rejected_store.clone(),
            config,
            replacement_joined: Rc::clone(&replacement_joined),
        },
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:source_isolation_test",
        )),
        runtime.config(),
    );
    let store = rejected_store.clone();
    let saved = previous.clone();
    let healthy_stopped = Rc::clone(&healthy_joined);
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            tokio::time::timeout(Duration::from_secs(5), async {
                let mut rejected_page = None;
                let mut healthy_page = None;
                for _ in 0..2 {
                    let pdata = ctx.recv().await.expect("source page");
                    match decoded_source(&pdata).as_str() {
                        "rejected" => rejected_page = Some(pdata),
                        "healthy" => healthy_page = Some(pdata),
                        _ => panic!("unexpected source"),
                    }
                }
                let pdata = rejected_page.expect("rejected page");
                assert_page_id(&pdata, 42);
                let (_, stale_ack) = next_ack(AckMsg::new(pdata.clone())).expect("old ACK");
                let (_, nack) =
                    next_nack(NackMsg::new_permanent("private reason", pdata)).expect("NACK");
                rejected_tx
                    .send(NodeControlMsg::Nack(nack))
                    .expect("reject");
                rejected_tx
                    .send(NodeControlMsg::Ack(stale_ack))
                    .expect("stale ACK");
                // Pressure transitions cannot implicitly resume a rejection pause.
                rejected_tx
                    .send(pressure(1, MemoryPressureLevel::Hard))
                    .expect("hard");
                rejected_tx
                    .send(pressure(2, MemoryPressureLevel::Normal))
                    .expect("normal");
                for _ in 0..2 {
                    let (reports, reporter) = MetricsReporter::create_new_and_receiver(1);
                    rejected_tx
                        .send(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: reporter,
                        })
                        .expect("metrics");
                    let snapshot = reports.recv_async().await.expect("paused status");
                    assert_eq!(
                        snapshot_counter(&snapshot, "rejection.paused"),
                        1,
                        "pause stays observable across scrapes"
                    );
                }
                let worker = ScraperWorker::new().expect("checkpoint probe");
                assert_eq!(stored_checkpoint(&worker, &store).await, Some(saved));
                let key = store.lease_key().to_path_buf();
                assert!(
                    worker
                        .run(move || SourceLease::acquire(&key))
                        .expect("lease check")
                        .await
                        .expect("lease check joined")
                        .is_err()
                );
                worker
                    .stop(Instant::now() + Duration::from_secs(1))
                    .await
                    .expect("probe stopped");
                let (_, ack) = next_ack(AckMsg::new(healthy_page.expect("healthy page")))
                    .expect("healthy ACK");
                healthy_tx
                    .send(NodeControlMsg::Ack(ack))
                    .expect("healthy progress");
                let healthy_next = ctx
                    .recv()
                    .await
                    .expect("healthy continues while peer paused");
                assert_eq!(decoded_source(&healthy_next), "healthy");
                assert_page_id(&healthy_next, 2);
                assert!(!healthy_stopped.get());
                // Repair the rejected destination, then request only that source's restart.
                rejected_tx
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "source-only repair".to_owned(),
                    })
                    .expect("restart rejected source");
                let repaired = ctx
                    .recv()
                    .await
                    .expect("replacement replays unchanged cursor");
                assert_eq!(decoded_source(&repaired), "rejected");
                assert_page_id(&repaired, 42);
                let (_, ack) = next_ack(AckMsg::new(repaired)).expect("repaired ACK");
                replacement_tx
                    .send(NodeControlMsg::Ack(ack))
                    .expect("accept repaired page");
                replacement_tx
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "repaired progress committed".to_owned(),
                    })
                    .expect("stop replacement");
                let (_, ack) = next_ack(AckMsg::new(healthy_next)).expect("healthy second ACK");
                healthy_tx
                    .send(NodeControlMsg::Ack(ack))
                    .expect("healthy still progressing");
                let healthy_third = ctx.recv().await.expect("healthy survives peer replacement");
                assert_eq!(decoded_source(&healthy_third), "healthy");
                assert_page_id(&healthy_third, 3);
                assert!(!healthy_stopped.get(), "no unrelated receiver restart");
                healthy_tx
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test done".to_owned(),
                    })
                    .expect("stop healthy");
                assert!(ctx.recv().await.is_err());
            })
            .await
            .expect("source-local repair remains responsive");
        });
    assert!(joined.get() && replacement_joined.get() && healthy_joined.get());
    assert_eq!(
        rejected_store.read().expect("repaired checkpoint"),
        Some(checkpoint(previous.revision + 1, 42))
    );
    drop(SourceBinding::acquire(rejected_store).expect("repaired source lease released"));
}

fn run_rejection_retry(repair: bool) {
    let directory = tempfile::tempdir_in(".").expect("checkpoint directory");
    let config = rejection_config(directory.path(), OnPermanentNack::Retry);
    let store = source_store(directory.path(), "retry");
    let source = SourceBinding::acquire(store.clone()).expect("binding");
    let (previous, _) = store
        .write(0, &checkpoint(0, 41).cursor)
        .expect("saved checkpoint");
    let joined = Rc::new(Cell::new(false));
    let receiver = receiver_for(&store, &config, source, Rc::clone(&joined));
    let runtime = TestRuntime::<OtapPdata>::new();
    let wrapper = ReceiverWrapper::local(
        receiver,
        test_node(runtime.config().name.clone()),
        Arc::new(NodeUserConfig::new_receiver_config(
            "urn:otel:receiver:rejection_retry_test",
        )),
        runtime.config(),
    );
    let stored = store.clone();
    let saved = previous.clone();
    runtime
        .set_receiver(wrapper)
        .run_test(|_| async {})
        .run_validation_concurrent(move |mut ctx| async move {
            tokio::time::timeout(Duration::from_secs(7), async {
                let first = ctx.recv().await.expect("first page");
                let (_, old_ack) = next_ack(AckMsg::new(first.clone())).expect("old ACK");
                let (_, nack) =
                    next_nack(NackMsg::new_permanent("private rejection", first)).expect("NACK");
                let rejected_at = Instant::now();
                ctx.send_control_msg(NodeControlMsg::Nack(nack))
                    .await
                    .expect("permanent reject");
                if repair {
                    ctx.send_control_msg(pressure(2, MemoryPressureLevel::Hard))
                        .await
                        .expect("pressure");
                    ctx.send_control_msg(pressure(1, MemoryPressureLevel::Normal))
                        .await
                        .expect("stale pressure");
                    assert!(
                        tokio::time::timeout(Duration::from_millis(1200), ctx.recv())
                            .await
                            .is_err(),
                        "pressure gates retry after delay expires"
                    );
                    ctx.send_control_msg(pressure(3, MemoryPressureLevel::Normal))
                        .await
                        .expect("resume");
                    let second = ctx.recv().await.expect("controlled replay");
                    assert!(rejected_at.elapsed() >= RETRY_INITIAL);
                    assert_page_id(&second, 42);
                    ctx.send_control_msg(NodeControlMsg::Ack(old_ack))
                        .await
                        .expect("stale ACK");
                    let (reports, reporter) = MetricsReporter::create_new_and_receiver(1);
                    ctx.send_control_msg(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: reporter,
                    })
                    .await
                    .expect("metrics");
                    let snapshot = reports.recv_async().await.expect("status");
                    assert_eq!(snapshot_counter(&snapshot, "starts"), 1);
                    assert_eq!(snapshot_counter(&snapshot, "rejection.paused"), 0);
                    assert_eq!(snapshot_counter(&snapshot, "stale.feedback"), 1);
                    let worker = ScraperWorker::new().expect("checkpoint probe");
                    assert_eq!(stored_checkpoint(&worker, &stored).await, Some(saved));
                    worker
                        .stop(Instant::now() + Duration::from_secs(1))
                        .await
                        .expect("probe stopped");
                    let (_, nack) =
                        next_nack(NackMsg::new("now transient", second)).expect("retryable NACK");
                    let rejected_at = Instant::now();
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("reject again");
                    let repaired = ctx.recv().await.expect("repair accepted");
                    assert!(
                        rejected_at.elapsed() >= Duration::from_secs(2),
                        "retryable feedback cannot reset permanent-rejection backoff"
                    );
                    assert_page_id(&repaired, 42);
                    let (_, ack) = next_ack(AckMsg::new(repaired)).expect("repaired ACK");
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("accept");
                }
                let stopped_at = Instant::now();
                ctx.send_control_msg(NodeControlMsg::DrainIngress {
                    deadline: stopped_at + Duration::from_secs(1),
                    reason: "finish rejection test".to_owned(),
                })
                .await
                .expect("drain");
                assert!(ctx.recv().await.is_err());
                assert!(stopped_at.elapsed() < Duration::from_secs(1));
            })
            .await
            .expect("controlled retry and drain complete");
        });
    assert!(joined.get());
    let expected = if repair {
        checkpoint(previous.revision + 1, 42)
    } else {
        previous
    };
    assert_eq!(store.read().expect("checkpoint"), Some(expected));
    drop(SourceBinding::acquire(store).expect("lease released"));
}

/// Scenario: Explicit retry receives a permanent NACK, memory pressure, stale ACK, and another NACK before repair.
/// Guarantees: One receiver retries unchanged committed progress with increasing delays and fresh IDs,
/// honors pressure, ignores old ACKs, and advances only after repaired delivery is acknowledged.
#[test]
fn permanent_rejection_retry_recovers_after_repair() {
    run_rejection_retry(true);
}

/// Scenario: Drain arrives while explicit permanent-rejection retry is in backoff.
/// Guarantees: The receiver stops without another query or checkpoint advancement and releases ownership after cleanup.
#[test]
fn permanent_rejection_backoff_is_drainable() {
    run_rejection_retry(false);
}
