// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for transient-NACK Kafka replay.

use super::*;
use std::cell::RefCell;

struct FailingReplayConsumer {
    failed_operation: ReplayOperation,
    calls: RefCell<Vec<ReplayOperation>>,
}

impl FailingReplayConsumer {
    fn new(failed_operation: ReplayOperation) -> Self {
        Self {
            failed_operation,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn execute(&self, operation: ReplayOperation) -> Result<(), KafkaError> {
        self.calls.borrow_mut().push(operation);
        if operation != self.failed_operation {
            return Ok(());
        }
        match operation {
            ReplayOperation::Pause | ReplayOperation::Resume => Err(KafkaError::PauseResume(
                format!("injected {} failure", operation.as_str()),
            )),
            ReplayOperation::Seek => Err(KafkaError::Seek("injected seek failure".to_string())),
        }
    }
}

impl ReplayConsumerOperations for FailingReplayConsumer {
    fn pause_partition(&self, _topic: &str, _partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Pause)
    }

    fn seek_partition(
        &self,
        _topic: &str,
        _partition: i32,
        _offset: i64,
    ) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Seek)
    }

    fn resume_partition(&self, _topic: &str, _partition: i32) -> Result<(), KafkaError> {
        self.execute(ReplayOperation::Resume)
    }
}

/// Scenario: A partition is revoked while its failed offset is in transient-NACK backoff.
/// Guarantees: Reconciliation drops tracking and retry state without advancing the failed offset.
#[test]
fn reconcile_revocation_drops_active_retry_without_advancing_offset() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
    let mut assignment = TopicPartitionList::new();
    let _ = assignment.add_partition("traces", 0);
    receiver
        .rebalance_state
        .set_assignment_for_test(&assignment);
    let ownership_raw = receiver.rebalance_state.current_generation("traces", 0);
    let ownership_generation = OwnershipGeneration::from_raw(ownership_raw);
    receiver
        .offset_tracker
        .track("traces", 0, 12, ownership_raw);
    let original_delivery =
        receiver
            .retry_manager
            .delivery_generation("traces", 0, ownership_generation);
    let rewind = receiver
        .offset_tracker
        .prepare_replay("traces", 0, 12, ownership_raw)
        .expect("failed offset remains pending");
    let retry_config = receiver
        .config
        .replay_backoff()
        .expect("replay config")
        .clone();
    let replay_delivery = receiver.retry_manager.begin_retry(
        BeginRetry {
            topic: "traces",
            partition: 0,
            ownership_generation,
            failed_offset: 12,
            rewind_offset: rewind,
            paused: true,
            now: Instant::now(),
        },
        &retry_config,
    );
    assert_ne!(original_delivery, replay_delivery);

    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, ownership_raw);
    receiver.reconcile_rebalance_state();

    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
    assert!(
        !receiver
            .offset_tracker
            .committable_snapshot()
            .contains_key(&("traces".to_string(), 0))
    );
    assert_eq!(receiver.retry_manager.paused_count(), 0);
    assert!(receiver.retry_manager.next_deadline().is_none());
    assert_eq!(
        receiver
            .retry_manager
            .feedback_ownership_generation("traces", 0, replay_delivery),
        Some(ownership_generation),
    );
}

/// Scenario: Pause, seek, and resume failures occur in the production replay handler.
/// Guarantees: Every failure preserves offset 12 and schedules another bounded operation attempt.
#[test]
fn replay_operation_failures_preserve_failed_offset_watermark() {
    let scenarios = [
        (
            ReplayOperation::Pause,
            false,
            vec![ReplayOperation::Pause],
            0,
        ),
        (ReplayOperation::Seek, true, vec![ReplayOperation::Seek], 1),
        (
            ReplayOperation::Resume,
            true,
            vec![ReplayOperation::Seek, ReplayOperation::Resume],
            1,
        ),
    ];

    for (failed_operation, initially_paused, expected_calls, expected_paused) in scenarios {
        let cfg = manual_traces_config_with_replay_backoff(
            "localhost:9092",
            "operation-failure-group",
            "traces",
            1,
            4,
        );
        let ctx = make_pipeline_ctx();
        let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
        let mut assignment = TopicPartitionList::new();
        let _ = assignment.add_partition("traces", 0);
        receiver
            .rebalance_state
            .set_assignment_for_test(&assignment);
        let ownership_raw = receiver.rebalance_state.current_generation("traces", 0);
        let ownership_generation = OwnershipGeneration::from_raw(ownership_raw);
        receiver
            .offset_tracker
            .track("traces", 0, 12, ownership_raw);
        receiver
            .offset_tracker
            .track("traces", 0, 13, ownership_raw);
        assert!(!receiver.offset_tracker.acknowledge("traces", 0, 13));
        let _ = receiver
            .retry_manager
            .delivery_generation("traces", 0, ownership_generation);
        let rewind = receiver
            .offset_tracker
            .prepare_replay("traces", 0, 12, ownership_raw)
            .expect("failed offset remains pending");
        let retry_config = receiver
            .config
            .replay_backoff()
            .expect("replay config")
            .clone();
        let _ = receiver.retry_manager.begin_retry(
            BeginRetry {
                topic: "traces",
                partition: 0,
                ownership_generation,
                failed_offset: 12,
                rewind_offset: rewind,
                paused: initially_paused,
                now: Instant::now() - Duration::from_secs(1),
            },
            &retry_config,
        );
        let consumer = FailingReplayConsumer::new(failed_operation);

        receiver.process_due_replays(&consumer);

        assert_eq!(
            receiver
                .offset_tracker
                .committable_snapshot()
                .get(&("traces".to_string(), 0)),
            Some(&12),
        );
        assert!(receiver.retry_manager.blocks_delivery("traces", 0));
        assert!(receiver.retry_manager.next_deadline().is_some());
        assert_eq!(receiver.retry_manager.paused_count(), expected_paused);
        assert_eq!(*consumer.calls.borrow(), expected_calls);
    }
}

fn manual_traces_config_with_commit_and_skip(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::CommitAndSkip,
            ..Default::default()
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test commit-and-skip config valid")
}

fn manual_traces_config_with_replay(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
) -> KafkaReceiverConfig {
    manual_traces_config_with_replay_backoff(brokers, group_id, traces_topic, 10, 40)
}

fn manual_traces_config_with_replay_backoff(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::Replay,
            initial_backoff_ms,
            max_backoff_ms,
        })
        .with_enable_idempotency(true)
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test replay config valid")
}

/// Scenario: Explicit commit-and-skip handles transient and permanent NACKs as terminal outcomes.
/// Guarantees: Both records advance the committed offset despite replay being the manual default.
#[tokio::test]
async fn explicit_commit_and_skip_advances_transient_and_permanent_nacks() {
    const TOPIC: &str = "offset-nack-parity-traces";
    const RECORDS: usize = 2;
    let group = "offset-nack-parity-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            let cfg = manual_traces_config_with_commit_and_skip(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_, _, offset, _) = decode_calldata(&route.calldata);
                let _ = by_offset.insert(offset, pdata);
            }

            receiver.nack_transient(
                "transient failure",
                by_offset.remove(&0).expect("offset 0 delivered"),
            );
            receiver.nack_permanent(
                "permanent failure",
                by_offset.remove(&1).expect("offset 1 delivered"),
            );

            let brokers = cluster.bootstrap_servers().to_string();
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|offset| offset >= RECORDS as i64)
            })
            .await;
            assert!(advanced, "both terminal outcomes must advance the offset");

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario: Offset 0 is transiently NACKed while obsolete feedback ACKs offset 1.
/// Guarantees: Both offsets replay under a fresh generation and stale feedback cannot commit them.
#[tokio::test]
async fn transient_nack_replays_without_committing_past_failure() {
    const TOPIC: &str = "offset-transient-replay-traces";
    const RECORDS: usize = 2;
    let group = "offset-transient-replay-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send record");
            }

            let cfg = manual_traces_config_with_replay(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut original = HashMap::new();
            let mut original_delivery_generation = None;
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata.source_route().expect("source route");
                let (_, partition, offset, delivery_generation) = decode_calldata(&route.calldata);
                assert_eq!(partition, 0);
                if let Some(expected) = original_delivery_generation {
                    assert_eq!(delivery_generation, expected);
                } else {
                    original_delivery_generation = Some(delivery_generation);
                }
                let _ = original.insert(offset, pdata);
            }

            receiver.nack_transient(
                "retry from Kafka",
                original.remove(&0).expect("offset 0 delivered"),
            );
            receiver.ack(original.remove(&1).expect("offset 1 delivered"));

            let mut replayed = HashMap::new();
            let mut replay_delivery_generation = None;
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata.source_route().expect("replay source route");
                let (_, partition, offset, delivery_generation) = decode_calldata(&route.calldata);
                assert_eq!(partition, 0);
                assert_ne!(Some(delivery_generation), original_delivery_generation);
                if let Some(expected) = replay_delivery_generation {
                    assert_eq!(delivery_generation, expected);
                } else {
                    replay_delivery_generation = Some(delivery_generation);
                }
                let _ = replayed.insert(offset, pdata);
            }

            let brokers = cluster.bootstrap_servers().to_string();
            let committed_before_replay_ack =
                committed_offset(&brokers, group, TOPIC, 0).expect("committed-offset probe");
            assert!(committed_before_replay_ack.is_none_or(|offset| offset <= 0));

            receiver.ack(replayed.remove(&1).expect("replayed offset 1"));
            receiver.wait_for_control_barrier().await;
            let committed_out_of_order =
                committed_offset(&brokers, group, TOPIC, 0).expect("committed-offset probe");
            assert!(committed_out_of_order.is_none_or(|offset| offset <= 0));

            receiver.ack(replayed.remove(&0).expect("replayed offset 0"));
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(25), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_some_and(|offset| offset >= RECORDS as i64)
            })
            .await;
            assert!(advanced, "ACKing the rewind record must release progress");

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario: One partition backs off while a sibling continues and the receiver then shuts down.
/// Guarantees: Replay blocks only the failed partition and shutdown does not commit its offset.
#[tokio::test]
async fn transient_nack_pause_is_partition_local_and_shutdown_safe() {
    const TOPIC: &str = "offset-transient-partition-local-traces";
    let group = "offset-transient-partition-local-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                5_000,
                5_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let mut first_by_partition = HashMap::new();
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                let (_, partition, offset, _) =
                    decode_calldata(&pdata.source_route().expect("source route").calldata);
                assert_eq!(offset, 0);
                let _ = first_by_partition.insert(partition, pdata);
            }

            receiver.nack_transient(
                "pause partition zero",
                first_by_partition
                    .remove(&0)
                    .expect("partition 0 delivered"),
            );
            receiver.ack(
                first_by_partition
                    .remove(&1)
                    .expect("partition 1 delivered"),
            );
            producer
                .send_to_partition(TOPIC, 1, &bytes)
                .await
                .expect("send sibling record");

            let sibling = receiver
                .try_recv_pdata(Duration::from_secs(2))
                .await
                .expect("healthy partition continues during replay backoff");
            let (_, partition, offset, _) =
                decode_calldata(&sibling.source_route().expect("source route").calldata);
            assert_eq!((partition, offset), (1, 1));
            receiver.ack(sibling);

            let brokers = cluster.bootstrap_servers().to_string();
            let sibling_advanced =
                poll_until(Duration::from_secs(5), Duration::from_millis(25), || {
                    committed_offset(&brokers, group, TOPIC, 1)
                        .expect("committed-offset probe")
                        .is_some_and(|committed| committed >= 2)
                })
                .await;
            assert!(
                sibling_advanced,
                "healthy partition must commit independently"
            );
            assert!(
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_none_or(|committed| committed <= 0),
                "failed partition must remain at offset 0",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
            assert!(
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("committed-offset probe")
                    .is_none_or(|committed| committed <= 0),
                "shutdown during retry must not commit past the failed offset",
            );
        },
    )
    .await;
}

/// Scenario: A second group member acquires a partition held in transient-NACK backoff.
/// Guarantees: The new owner receives the uncommitted failed offset from Kafka.
#[tokio::test]
async fn transient_nack_rebalance_replays_failed_offset_on_new_owner() {
    const TOPIC: &str = "offset-transient-rebalance-traces";
    let group = "offset-transient-rebalance-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                30_000,
                30_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                receiver.nack_transient("hold for rebalance", pdata);
            }
            receiver.wait_for_control_barrier().await;

            let new_owner = cluster
                .consumer()
                .group_id(group)
                .enable_auto_commit(false)
                .auto_offset_reset("earliest")
                .subscribe(&[TOPIC]);
            assert!(
                new_owner
                    .wait_for_assignment(1, Duration::from_secs(30))
                    .await,
                "second consumer never acquired a partition",
            );
            let assigned_partition = new_owner
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("second consumer owns a test partition");
            receiver
                .wait_for_partition_revocation(TOPIC, assigned_partition, Duration::from_secs(10))
                .await;

            let replayed = new_owner
                .try_recv(Duration::from_secs(10))
                .await
                .expect("new owner should receive the failed offset");
            assert_eq!(replayed.topic, TOPIC);
            assert_eq!(replayed.partition, assigned_partition);
            assert_eq!(replayed.offset, 0);
            assert!(
                new_owner
                    .committed_offset(TOPIC, assigned_partition)
                    .expect("committed-offset probe")
                    .is_none_or(|offset| offset <= 0),
                "rebalance must not commit past the failed offset",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
            drop(new_owner);
        },
    )
    .await;
}

/// Scenario: A paused partition is revoked and then reassigned to the original consumer.
/// Guarantees: Rebalance clears persistent pause state and redelivers the failed offset.
#[tokio::test]
async fn transient_nack_reassignment_to_same_consumer_redelivers_failed_offset() {
    const TOPIC: &str = "offset-transient-reassign-same-consumer-traces";
    let group = "offset-transient-reassign-same-consumer-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer.produce_per_partition(TOPIC, 2, 1, &bytes).await;

            let cfg = manual_traces_config_with_replay_backoff(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                30_000,
                30_000,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            for _ in 0..2 {
                let pdata = receiver.recv_pdata().await;
                receiver.nack_transient("hold through reassignment", pdata);
            }
            receiver.wait_for_control_barrier().await;

            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            let revoked_partition = trigger
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("temporary member should own a test partition");
            receiver
                .wait_for_partition_revocation(TOPIC, revoked_partition, Duration::from_secs(10))
                .await;

            drop(trigger);
            receiver
                .wait_for_partition_assignment(TOPIC, revoked_partition, Duration::from_secs(30))
                .await;

            let replayed = receiver
                .try_recv_pdata(Duration::from_secs(10))
                .await
                .expect("reacquired partition should redeliver its failed offset");
            let route = replayed
                .source_route()
                .expect("redelivered pdata carries source calldata");
            let (_, partition, offset, _) = decode_calldata(&route.calldata);
            assert_eq!(partition, revoked_partition);
            assert_eq!(offset, 0);

            receiver.ack(replayed);
            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
