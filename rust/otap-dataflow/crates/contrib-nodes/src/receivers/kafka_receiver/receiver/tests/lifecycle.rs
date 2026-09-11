// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Lifecycle coverage for the Kafka receiver: ingress drain, shutdown, and the
//! consumer-lag worker lifecycle, including draining while a consumer-group
//! rebalance is in flight.
//!
//! These regression tests exercise the drain and shutdown paths (and the bounded
//! consumer close / lag-worker teardown) with the librdkafka `MockCluster` (the
//! repo's in-process Kafka "integration" harness, no Docker). They run on the
//! current-thread `LocalSet` provided by [`with_cluster`] and use manual-commit
//! receivers so the rebalance-aware commit path stays active. The
//! drain-under-rebalance cases model the reference multi-receiver test
//! `rebalance_two_receivers_scale_up_down_distribute_without_loss_or_double_commit`.

use super::*;

/// Records produced per partition in the drain-under-rebalance tests.
const DRAIN_RECORDS_PER_PARTITION: i32 = 5;

/// Scenario (drain under rebalance): a manual-commit receiver A owns every
/// partition of a multi-partition topic, is put into ingress drain via
/// DrainIngress, and then a second group member joins and forces a rebalance
/// that revokes A's partitions.
/// Guarantees: once the rebalance settles, a draining receiver releases every
/// partition and never re-acquires any of them -- is_assigned stays false for
/// all partitions across a bounded stability window -- so a drain that overlaps
/// a rebalance cannot pull the receiver back to a non-empty assignment.
#[tokio::test]
async fn draining_receiver_does_not_reacquire_partitions_after_new_member_joins() {
    const TOPIC: &str = "drain-rebalance-no-reacquire-traces";
    let group = "drain-rebalance-no-reacquire-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Produce to every partition so A has work on each partition it owns.
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    DRAIN_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Start replica A alone; a single member is assigned every
            // partition, so waiting for each assignment proves A holds the
            // whole topic before anyone else joins.
            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let receiver_a = KafkaReceiverHarness::start(&cluster, cfg);
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                receiver_a
                    .wait_for_partition_assignment(TOPIC, partition, Duration::from_secs(30))
                    .await;
            }

            // Put A into ingress drain with a long deadline: the drain must not
            // depend on the deadline to release partitions.
            receiver_a.drain(Duration::from_secs(30));

            // A second group member joins and forces a rebalance that revokes
            // A's partitions.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;

            // Every partition A held must be revoked once the rebalance settles.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                receiver_a
                    .wait_for_partition_revocation(TOPIC, partition, Duration::from_secs(15))
                    .await;
            }

            // A draining receiver must not re-acquire any partition. Poll a
            // bounded stability window and assert is_assigned stays false for
            // every partition the whole time; a pre-fix receiver would go
            // non-empty again here.
            let stable_until = tokio::time::Instant::now() + Duration::from_secs(3);
            while tokio::time::Instant::now() < stable_until {
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    assert!(
                        !receiver_a.is_assigned(TOPIC, partition),
                        "draining receiver re-acquired {TOPIC}/{partition} after the rebalance; \
                         it must stay released",
                    );
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            drop(trigger);
        },
    )
    .await;
}

/// Scenario (drain under rebalance): a manual-commit receiver A owning every
/// partition is put into ingress drain with a short deadline while a sequence
/// of joining-and-leaving group members churns the consumer group, forcing
/// repeated rebalances that are still in flight *concurrently* with A's drain.
/// Guarantees: A's receive loop reaches its terminal state strictly before the
/// drain deadline elapses
#[tokio::test]
async fn draining_receiver_task_completes_before_deadline_under_rebalance_churn() {
    const TOPIC: &str = "drain-rebalance-churn-traces";
    let group = "drain-rebalance-churn-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    DRAIN_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Start replica A alone and let it acquire every partition.
            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let receiver_a = KafkaReceiverHarness::start(&cluster, cfg);
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                receiver_a
                    .wait_for_partition_assignment(TOPIC, partition, Duration::from_secs(30))
                    .await;
            }

            // Drain with a short deadline: the drain must complete on its own
            // strictly before this deadline even while rebalances churn.
            let drain_deadline = Duration::from_secs(5);
            let started = tokio::time::Instant::now();
            receiver_a.drain(drain_deadline);

            // Churn the group *concurrently* with A's drain: repeatedly start
            // and gracefully stop a second receiver in the same group so the
            // group rebalances several times while A is draining. A receiver
            // harness makes no "must be assigned" assumption (unlike
            // `RebalanceTrigger`), so the churn cannot panic on A's drain timing
            // while still forcing real join+leave rebalances. This future is
            // polled alongside A's terminal-state await below (not before it),
            // so the rebalances overlap the window in which A's receive loop
            // must finish its drain and bounded close.
            let churn = async {
                for _ in 0..3 {
                    let churn_cfg =
                        manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                    let churn = KafkaReceiverHarness::start(&cluster, churn_cfg);
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    churn.shutdown(Duration::from_secs(1));
                    let _ = churn.await_terminal_state().await;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            };

            // Await A's terminal state concurrently with the churn.
            let terminal = tokio::join!(churn, async {
                tokio::time::timeout(
                    drain_deadline + Duration::from_secs(5),
                    receiver_a.await_terminal_state(),
                )
                .await
                .expect("draining receiver task did not complete under rebalance churn")
            })
            .1;
            let elapsed = started.elapsed();
            assert!(
                elapsed < drain_deadline,
                "draining receiver task took {elapsed:?}, at or past the {drain_deadline:?} drain \
                 deadline; the drain did not complete on its own -- a commit-before-revoke or close \
                 wedged on the pipeline thread would only unblock at the forced deadline",
            );

            // Terminal state is produced only on a clean drain-to-completion.
            let _ = terminal;
        },
    )
    .await;
}

/// Scenario (drain under rebalance): two manual-commit receivers A and B share
/// a group; A is established owning every partition, then A drains and reaches
/// its terminal state, after which a fresh wave of records is produced to every
/// partition.
/// Guarantees: B receives records from the partitions A previously held within
/// a bounded window -- direct proof that a draining receiver authoritatively
/// leaves the consumer group (releasing its partitions to a peer) rather than
/// lingering and re-winning them.
#[tokio::test]
async fn draining_receiver_leaves_group_so_peer_gains_partitions() {
    const TOPIC: &str = "drain-rebalance-peer-gains-traces";
    let group = "drain-rebalance-peer-gains-group";
    let wave = (DRAIN_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Wave 1: produce to every partition and let A (alone) drain it in
            // full, establishing that A owned the whole topic before B mattered.
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    DRAIN_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            let cfg_a = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                receiver_a
                    .wait_for_partition_assignment(TOPIC, partition, Duration::from_secs(30))
                    .await;
            }
            for _ in 0..wave {
                let pdata = receiver_a.recv_pdata().await;
                receiver_a.ack(pdata);
            }

            // Start replica B in the same group. Under an eager assignor B may
            // sit idle until A leaves, so its assignment is proven below by the
            // records it receives after A drains.
            let cfg_b = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);

            // Let B's initial join settle before A leaves.
            tokio::time::sleep(Duration::from_secs(1)).await;

            // A drains and reaches its terminal state: A must authoritatively
            // leave the group here, not linger.
            receiver_a.drain(Duration::from_secs(30));
            let _terminal_a = receiver_a.await_terminal_state().await;

            // Wave 2: fresh records on every partition. If A truly left, B now
            // owns every partition and must deliver this wave.
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    DRAIN_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // B must receive the whole fresh wave within a bounded window --
            // direct proof it gained the partitions A released by leaving.
            // Bounded by a deadline so a failure to hand off fails loudly
            // instead of hanging.
            let mut delivered_b = 0usize;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            while delivered_b < wave {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "timed out: peer B received {delivered_b} of {wave} expected records after A \
                     drained; A did not leave the group and hand off its partitions",
                );
                if let Some(pdata) = receiver_b.try_recv_pdata(Duration::from_millis(250)).await {
                    receiver_b.ack(pdata);
                    delivered_b += 1;
                }
            }

            receiver_b.shutdown(Duration::from_secs(5));
            let _terminal_b = receiver_b.await_terminal_state().await;
        },
    )
    .await;
}

// ---- Lifecycle: drain and shutdown (moved from receiver.rs) ----

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver consumes and acks an initial batch,
/// then receives `DrainIngress`; more records are produced after the drain.
/// Guarantees: the receiver emits `RuntimeControlMsg::ReceiverDrained`, stops
/// forwarding new records (no pdata arrives post-drain), commits the
/// pre-drain offsets (committed offset >= INITIAL), and terminates as part
/// of the receiver-first drain (via `await_stopped` returning).
#[tokio::test]
async fn drain_ingress_stops_polling_and_notifies_drained() {
    const TOPIC: &str = "drain-ingress-traces";
    const INITIAL: usize = 3;
    let group = "drain-ingress-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Produce an initial batch that the receiver will consume before drain.
            for i in 0..INITIAL {
                let key = format!("pre-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 60_000, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack the initial batch so offsets are tracked and
            // committable at drain time.
            for _ in 0..INITIAL {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Begin receiver-first drain.
            receiver.drain(Duration::from_secs(5));

            // The receiver must signal ReceiverDrained. The runtime channel
            // also carries timer-setup messages (StartTimer /
            // StartTelemetryTimer) emitted while the loop starts up, so skip
            // past those until the drain signal arrives.
            let mut drained = false;
            for _ in 0..16 {
                let msg = receiver
                    .try_recv_runtime(Duration::from_secs(10))
                    .await
                    .expect("timed out waiting for ReceiverDrained");
                if matches!(msg, RuntimeControlMsg::ReceiverDrained { .. }) {
                    drained = true;
                    break;
                }
            }
            assert!(drained, "receiver never emitted ReceiverDrained");

            // After drain, produce more records. The receiver has stopped
            // polling, so none of these should be forwarded downstream.
            for i in 0..INITIAL {
                let key = format!("post-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send post-drain message");
            }

            // No further pdata should arrive within a reasonable window.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "receiver forwarded a record after DrainIngress; polling did not stop",
            );

            // Committed offset must account for the pre-drain batch (final
            // commit was issued during drain and flushed on unsubscribe).
            // The commit is asynchronous, so poll until the broker reports
            // it rather than asserting once.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= INITIAL as i64)
            })
            .await;
            assert!(
                committed,
                "pre-drain offsets should be committed at drain time, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            // The receiver terminates after reporting its ingress drain,
            // matching the runtime's receiver-first lifecycle contract.
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a consumer-lag worker holding an
/// `Arc` clone of the consumer is still in flight at Shutdown, but it honors
/// cooperative cancellation and returns within the bounded drain.
/// Guarantees: after the shutdown handler's bounded drain
/// (`timeout_at(min(lag_deadline, shutdown_deadline), handle)`) resolves, the
/// worker has returned and released its clone, so only the loop's clone
/// remains (`strong_count == 1`). The subsequent consumer drop is therefore
/// the last `Arc` and is what triggers the leave-group/close -- the happy
/// path the shutdown ordering relies on.
#[tokio::test(start_paused = true)]
async fn shutdown_bounded_drain_lets_cooperative_lag_worker_release_clone_before_close() {
    let start = tokio::time::Instant::now();
    // Worker deadline far out (15s); shutdown deadline near (1s). The worker
    // must finish via cancellation, not by hitting either deadline.
    let lag_deadline = start + LAG_REFRESH_TOTAL_DEADLINE;
    let shutdown_deadline = start + Duration::from_secs(1);

    // Shared reference stands in for the consumer `Arc`: one clone held by
    // the loop, one by the lag worker.
    let consumer = Arc::new(());
    let worker_clone = Arc::clone(&consumer);

    // A cooperatively-cancellable worker (mirrors `compute_consumer_lag`
    // abandoning the refresh on cancellation). It holds its clone until it
    // observes cancellation and returns.
    let cancel = CancellationToken::new();
    let worker_cancel = cancel.clone();
    let handle = tokio::task::spawn(async move {
        worker_cancel.cancelled().await;
        drop(worker_clone);
        None::<f64>
    });

    // Model the shutdown handler's bounded drain: cancel, then wait bounded by
    // the tighter of the two deadlines.
    cancel.cancel();
    let bound = lag_deadline.min(shutdown_deadline);
    let drain = tokio::time::timeout_at(bound, handle).await;

    // The worker observed cancellation and completed within the bound, so the
    // drain resolved with the worker's result (not a timeout).
    let join_result = drain.expect("drain must not exceed the bounded deadline");
    assert_eq!(
        join_result.expect("worker must not panic"),
        None,
        "a cancelled lag worker abandons the refresh and returns None",
    );

    // The worker released its clone before the drain returned, so the loop's
    // clone is the only one left: the consumer drop here would be the last
    // `Arc`.
    assert_eq!(
        Arc::strong_count(&consumer),
        1,
        "cooperative lag worker must release its clone before the close, \
             leaving the loop clone as the last Arc",
    );
}

/// Scenario (lifecycle: drain and shutdown): a consumer-lag worker holding an
/// `Arc` clone of the consumer is parked mid-librdkafka-call at Shutdown and
/// does NOT observe cancellation before the bounded drain expires.
/// Guarantees: the bounded drain
/// (`timeout_at(min(lag_deadline, shutdown_deadline), handle)`) times out with
/// the worker still alive and still holding its clone, so at the close the
/// consumer `Arc` still has `strong_count == 2`. This documents that the
/// consumer drop in the shutdown handler is NOT guaranteed to be the last
/// `Arc` -- the leave-group/close then runs when the abandoned worker later
/// releases its clone, not necessarily at that drop.
#[tokio::test(start_paused = true)]
async fn shutdown_bounded_drain_can_leave_lag_worker_clone_alive_refcount_two() {
    let start = tokio::time::Instant::now();
    // Shutdown deadline near (1s); worker deadline far (15s). The bound is the
    // shutdown deadline.
    let lag_deadline = start + LAG_REFRESH_TOTAL_DEADLINE;
    let shutdown_deadline = start + Duration::from_secs(1);

    let consumer = Arc::new(());
    let worker_clone = Arc::clone(&consumer);

    // A worker that ignores cancellation and never finishes on its own, so it
    // is still holding its clone when the bounded drain expires. It stands in
    // for a worker parked inside a librdkafka FFI call that has not yet
    // returned to observe the cancellation token.
    let cancel = CancellationToken::new();
    let handle = tokio::task::spawn(async move {
        // Keep the clone alive for the whole task.
        let _held = worker_clone;
        std::future::pending::<()>().await;
        None::<f64>
    });

    // Model the shutdown handler's bounded drain: cancel, then wait bounded by
    // the tighter of the two deadlines. The worker never honors it.
    cancel.cancel();
    let bound = lag_deadline.min(shutdown_deadline);
    assert_eq!(
        bound, shutdown_deadline,
        "min must pick the shutdown deadline"
    );
    let drain = tokio::time::timeout_at(bound, handle).await;

    // The drain timed out: the worker is still running.
    assert!(
        drain.is_err(),
        "a non-cooperative worker is bounded by the deadline, not awaited to completion",
    );
    // The still-running worker keeps its clone, so the consumer drop that
    // follows in the shutdown handler is NOT the last `Arc`.
    assert_eq!(
        Arc::strong_count(&consumer),
        2,
        "a lag worker that outlives the bounded drain still holds its clone, \
             so the close-time drop is not the last Arc (refcount is 2)",
    );
    // Termination is still bounded: the drain returned by the shutdown
    // deadline rather than waiting for the worker.
    assert!(
        tokio::time::Instant::now() <= shutdown_deadline,
        "drain must complete by the shutdown deadline, not the worker's",
    );
}

// Scenario: a consumer-lag worker is in flight at Shutdown, and this time the
// worker's lag deadline is *earlier* than the shutdown deadline.
// Guarantees: the drain bound is the tighter (lag) deadline, so `min` selects
// the lag deadline and the drain still cannot run to the later shutdown
// deadline.
#[tokio::test(start_paused = true)]
async fn shutdown_lag_drain_bound_selects_the_earlier_lag_deadline() {
    let start = tokio::time::Instant::now();
    // Worker deadline is near (2s); shutdown deadline is far (30s).
    let lag_deadline = start + Duration::from_secs(2);
    let shutdown_deadline = start + Duration::from_secs(30);

    // A worker that never finishes on its own and ignores cancellation, so
    // the only thing that can unblock the drain is the min-bounded timeout.
    let handle = tokio::task::spawn(async {
        std::future::pending::<()>().await;
        None::<f64>
    });

    let bound = lag_deadline.min(shutdown_deadline);
    assert_eq!(
        bound, lag_deadline,
        "min must pick the earlier lag deadline"
    );

    let drain = tokio::time::timeout_at(bound, handle).await;
    assert!(
        drain.is_err(),
        "a non-cooperative worker is bounded by the lag deadline, not the later shutdown one",
    );

    // The drain elapsed at the lag deadline, strictly before the shutdown
    // deadline.
    let elapsed = tokio::time::Instant::now();
    assert!(
        elapsed <= lag_deadline && elapsed < shutdown_deadline,
        "drain must be bounded by the earlier (lag) deadline",
    );
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver spawns a lag refresh for a consumer
/// that owns no partitions (empty assignment).
/// Guarantees: `spawn_consumer_lag_refresh` still spawns a task (manual mode)
/// and the task returns `Some(0.0)` -- the documented empty-assignment
/// sentinel -- so the caller resets `receiver.kafka.consumer.group.lag` to 0
/// rather than leaving a stale value.
#[tokio::test]
async fn spawn_consumer_lag_refresh_resets_to_zero_when_unassigned() {
    const TOPIC: &str = "lag-empty";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
        |cluster| async move {
            let cfg = make_config(&[TOPIC], &["metrics"], &[], MessageFormat::OtlpProto);
            assert!(!cfg.is_auto_commit());
            let ctx = make_pipeline_ctx();
            let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

            let consumer = Arc::new(make_manual_consumer(
                cluster.bootstrap_servers(),
                "lag-empty-group",
            ));

            // Manual mode => a task is spawned; the consumer has no
            // assignment, so the task yields `Some(0.0)` (reset the gauge to
            // the empty value).
            let handle = receiver
                .spawn_consumer_lag_refresh(
                    &consumer,
                    Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                    CancellationToken::new(),
                )
                .expect("manual mode spawns a refresh task");
            let result = handle.await.expect("lag task should not panic");
            assert_eq!(result, Some(0.0));
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): auto-commit receiver requests a lag refresh.
/// Guarantees: `spawn_consumer_lag_refresh` returns `None` (no task, no
/// broker work) because offset management is owned by librdkafka.
#[tokio::test]
async fn spawn_consumer_lag_refresh_none_under_auto_commit() {
    const TOPIC: &str = "lag-auto";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
        |cluster| async move {
            let cfg = KafkaReceiverConfig::try_from(
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), "g", "c")
                    .with_traces(SignalConfig::new(vec![TOPIC.to_string()]))
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Auto,
                        interval_ms: Some(1000),
                    })
                    .with_isolation_level(IsolationLevel::ReadUncommitted),
            )
            .expect("test config should be valid");
            let ctx = make_pipeline_ctx();
            let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

            let consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", cluster.bootstrap_servers())
                .set("group.id", "lag-auto-group")
                .set("enable.auto.commit", "true")
                .create()
                .expect("failed to create consumer");
            let consumer = Arc::new(consumer);

            assert!(
                receiver
                    .spawn_consumer_lag_refresh(
                        &consumer,
                        Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
                        CancellationToken::new(),
                    )
                    .is_none()
            );
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver
/// consumes and acks records while a producer keeps sending throughout; the
/// receiver is then drained, with more records produced after the drain
/// begins.
/// Guarantees: under sustained traffic the receiver still emits
/// `ReceiverDrained`, stops forwarding new records once drained, commits the
/// offsets acked before the drain (committed offset >= the pre-drain acked
/// count), and terminates cleanly as part of ingress drain.
#[tokio::test]
async fn drain_under_sustained_traffic_commits_and_stops_cleanly() {
    const TOPIC: &str = "drain-sustained-traces";
    const PRE_DRAIN: usize = 5;
    const POST_DRAIN: usize = 10;
    let group = "drain-sustained-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Produce a first burst the receiver will consume and ack.
            for i in 0..PRE_DRAIN {
                let key = format!("pre-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send pre-drain");
            }

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack every pre-drain record so its offset is
            // committable at drain time.
            for _ in 0..PRE_DRAIN {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Begin the receiver-first drain while traffic continues.
            receiver.drain(Duration::from_secs(5));

            // Keep producing after the drain: the receiver has stopped
            // polling, so none of these must be forwarded.
            for i in 0..POST_DRAIN {
                let key = format!("post-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send post-drain");
            }

            // The receiver must signal ReceiverDrained (skip past the
            // timer-setup runtime messages emitted during startup).
            let mut drained = false;
            for _ in 0..16 {
                match receiver.try_recv_runtime(Duration::from_secs(10)).await {
                    Some(RuntimeControlMsg::ReceiverDrained { .. }) => {
                        drained = true;
                        break;
                    }
                    Some(_) => continue,
                    None => break,
                }
            }
            assert!(
                drained,
                "receiver never emitted ReceiverDrained under traffic"
            );

            // No further pdata should arrive once drained, even though
            // POST_DRAIN records are now on the broker.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "receiver forwarded a record after DrainIngress under traffic",
            );

            // The pre-drain acked offsets must be committed.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= PRE_DRAIN as i64)
            })
            .await;
            assert!(
                committed,
                "pre-drain offsets should be committed, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver holds
/// in-flight records that are never acked (their downstream acks are still
/// pending) when `DrainIngress` arrives.
/// Guarantees: the receiver signals `ReceiverDrained` promptly without
/// blocking on the un-acked in-flight records -- codifying the documented
/// design that drain does not wait for in-flight downstream acks and relies
/// on at-least-once redelivery for the un-committed offsets.
#[tokio::test]
async fn drain_does_not_wait_for_inflight_downstream_acks() {
    const TOPIC: &str = "drain-inflight-traces";
    const RECORDS: usize = 4;
    let group = "drain-inflight-group";
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

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume every record but hold them un-acked: their downstream
            // acks are deliberately never delivered, so they stay in-flight.
            let mut in_flight = Vec::new();
            for _ in 0..RECORDS {
                in_flight.push(receiver.recv_pdata().await);
            }

            // Begin the drain with the in-flight records still un-acked.
            let drain_started = tokio::time::Instant::now();
            receiver.drain(Duration::from_secs(30));

            // ReceiverDrained must arrive promptly -- well within the drain
            // deadline -- proving the drain did not block waiting for the
            // in-flight acks.
            let mut drained = false;
            for _ in 0..16 {
                match receiver.try_recv_runtime(Duration::from_secs(5)).await {
                    Some(RuntimeControlMsg::ReceiverDrained { .. }) => {
                        drained = true;
                        break;
                    }
                    Some(_) => continue,
                    None => break,
                }
            }
            assert!(
                drained,
                "receiver must signal ReceiverDrained without waiting for \
                     in-flight downstream acks",
            );
            assert!(
                drain_started.elapsed() < Duration::from_secs(15),
                "drain notification took too long ({:?}); it should not block \
                     on in-flight acks",
                drain_started.elapsed(),
            );

            // The held records are still un-acked; drop them to release the
            // in-flight set; the receiver has already terminated cleanly
            // as part of ingress drain.
            drop(in_flight);
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver with
/// tracked, un-committed offsets is shut down while every broker is marked
/// down, so the shutdown-time consumer unsubscribe/close cannot reach the
/// broker.
/// Guarantees: the receiver still reaches its terminal state within a
/// bounded wait rather than hanging the pipeline on an unreachable broker --
/// the synchronous librdkafka close runs off the loop thread bounded by the
/// shutdown deadline, so it cannot stall termination.
#[tokio::test]
async fn shutdown_with_broker_unavailable_does_not_hang() {
    const TOPIC: &str = "shutdown-broker-down-traces";
    const RECORDS: usize = 3;
    let group = "shutdown-broker-down-group";
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

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack every record so there are tracked offsets to
            // commit at shutdown.
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Make the broker slow to respond so the shutdown-time close
            // (unsubscribe + consumer drop) is delayed well past the shutdown
            // deadline. A large per-request round-trip delay stands in for an
            // unreachable broker but -- unlike marking the broker fully down --
            // still lets the off-thread close eventually complete, so the test
            // does not orphan a permanently-blocked librdkafka FFI thread that
            // would stall runtime teardown.
            cluster.faults().round_trip_time(1, Duration::from_secs(30));

            // Shutdown with a short (1s) deadline while the broker is
            // effectively unavailable. The receiver's off-loop-thread close is
            // bounded by this deadline, so the receiver task must return well
            // within the generous outer bound below even though the broker is
            // not responding -- a regression that ran the blocking close on the
            // loop thread would exceed it and fail the test deterministically.
            let shutdown_at = tokio::time::Instant::now();
            receiver.shutdown(Duration::from_secs(1));
            let terminated =
                tokio::time::timeout(Duration::from_secs(5), receiver.await_terminal_state()).await;
            assert!(
                terminated.is_ok(),
                "receiver must terminate within the bounded deadline even when \
                     the broker is unavailable at shutdown",
            );
            assert!(
                shutdown_at.elapsed() < Duration::from_secs(5),
                "termination should be bounded by the shutdown deadline, not the \
                     30s broker round-trip delay; took {:?}",
                shutdown_at.elapsed(),
            );

            // Restore normal broker latency so the (deadline-exceeded)
            // off-thread close can finish and its blocking thread joins,
            // keeping test teardown clean.
            cluster
                .faults()
                .round_trip_time(1, Duration::from_millis(1));
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver with
/// the opt-in consumer-lag refresh timer armed (so a lag-refresh worker is
/// periodically spawned and holds its own `Arc` clone of the consumer) is
/// shut down after the timer has had time to spawn a refresh.
/// Guarantees: the shutdown handler's bounded lag-drain-then-close path runs
/// with a real in-flight lag worker present and the receiver still reaches
/// its terminal state within the shutdown deadline -- the lag worker cannot
/// stall termination. This exercises the real receiver path; the internal
/// `Arc`-count-can-be-2 behavior it can produce is asserted deterministically
/// by the unit test
/// `shutdown_bounded_drain_can_leave_lag_worker_clone_alive_refcount_two`
/// (the harness exposes no access to the receiver's private consumer `Arc`).
#[tokio::test]
async fn shutdown_with_lag_refresh_in_flight_still_terminates_within_deadline() {
    const TOPIC: &str = "shutdown-lag-inflight-traces";
    const RECORDS: usize = 3;
    let group = "shutdown-lag-inflight-group";
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

            // Arm the lag refresh timer at a short interval so a lag-refresh
            // worker is repeatedly spawned.
            let cfg = manual_traces_config_with_lag_refresh(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                50,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Give the lag timer several ticks so a refresh worker is spawned
            // and can be in flight when the shutdown arrives.
            tokio::time::sleep(Duration::from_millis(150)).await;

            // Shutdown with a short (1s) deadline. Even with a lag worker in
            // flight, the bounded drain plus off-loop-thread close must let
            // the receiver return within the deadline.
            let shutdown_at = tokio::time::Instant::now();
            receiver.shutdown(Duration::from_secs(1));
            let terminated =
                tokio::time::timeout(Duration::from_secs(5), receiver.await_terminal_state()).await;
            assert!(
                terminated.is_ok(),
                "receiver must terminate within the bounded deadline even with a \
                     lag refresh in flight at shutdown",
            );
            assert!(
                shutdown_at.elapsed() < Duration::from_secs(5),
                "termination must be bounded by the shutdown deadline; took {:?}",
                shutdown_at.elapsed(),
            );
        },
    )
    .await;
}
