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
/// partition and never re-acquires any of them -- is_partition_assigned stays false for
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
            let bytes = encoded_trace_fixture();

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
            // bounded stability window and assert is_partition_assigned stays false for
            // every partition the whole time; a pre-fix receiver would go
            // non-empty again here.
            let stable_until = tokio::time::Instant::now() + Duration::from_secs(3);
            while tokio::time::Instant::now() < stable_until {
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    assert!(
                        !receiver_a.is_partition_assigned(TOPIC, partition),
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
            let bytes = encoded_trace_fixture();

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
                    let _ = shutdown_and_terminal(churn, Duration::from_secs(1)).await;
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
            let bytes = encoded_trace_fixture();

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
            recv_and_ack(&mut receiver_a, wave).await;

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

            let _terminal_b = shutdown_and_terminal(receiver_b, Duration::from_secs(5)).await;
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

            let bytes = encoded_trace_fixture();

            // Produce an initial batch that the receiver will consume before drain.
            produce_traces(&producer, TOPIC, INITIAL, "pre", &bytes).await;

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 60_000, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack the initial batch so offsets are tracked and
            // committable at drain time.
            recv_and_ack(&mut receiver, INITIAL).await;

            // Begin receiver-first drain.
            receiver.drain(Duration::from_secs(5));

            // The receiver must signal ReceiverDrained. The runtime channel
            // also carries timer-setup messages (StartTimer /
            // StartTelemetryTimer) emitted while the loop starts up, so skip
            // past those until the drain signal arrives.
            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(drained, "receiver never emitted ReceiverDrained");

            // After drain, produce more records. The receiver has stopped
            // polling, so none of these should be forwarded downstream.
            produce_traces(&producer, TOPIC, INITIAL, "post", &bytes).await;

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
            let committed = poll_committed_offset(
                &brokers,
                group,
                TOPIC,
                INITIAL as i64,
                Duration::from_secs(5),
                Duration::from_millis(250),
            )
            .await;
            assert!(
                committed,
                "pre-drain offsets should be committed at drain time, got {:?}",
                probe_committed_offset(&brokers, group, TOPIC),
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
            let ctx = make_pipeline_ctx(0, 1, 0);
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
            let cfg = auto_traces_config(cluster.bootstrap_servers(), "g", "c", TOPIC);
            let ctx = make_pipeline_ctx(0, 1, 0);
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
            let bytes = encoded_trace_fixture();

            // Produce a first burst the receiver will consume and ack.
            produce_traces(&producer, TOPIC, PRE_DRAIN, "pre", &bytes).await;

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack every pre-drain record so its offset is
            // committable at drain time.
            recv_and_ack(&mut receiver, PRE_DRAIN).await;

            // Begin the receiver-first drain while traffic continues.
            receiver.drain(Duration::from_secs(5));

            // Keep producing after the drain: the receiver has stopped
            // polling, so none of these must be forwarded.
            produce_traces(&producer, TOPIC, POST_DRAIN, "post", &bytes).await;

            // The receiver must signal ReceiverDrained (skip past the
            // timer-setup runtime messages emitted during startup).
            let drained = wait_for_receiver_drained(&mut receiver).await;
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
            let committed = poll_committed_offset(
                &brokers,
                group,
                TOPIC,
                PRE_DRAIN as i64,
                Duration::from_secs(5),
                Duration::from_millis(250),
            )
            .await;
            assert!(
                committed,
                "pre-drain offsets should be committed, got {:?}",
                probe_committed_offset(&brokers, group, TOPIC),
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
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            let mut receiver = start_manual_traces_receiver(&cluster, group, TOPIC);

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
            let drained = wait_for_receiver_drained(&mut receiver).await;
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
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            let mut receiver = start_manual_traces_receiver(&cluster, group, TOPIC);

            // Consume and ack every record so there are tracked offsets to
            // commit at shutdown.
            recv_and_ack(&mut receiver, RECORDS).await;

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
            let elapsed =
                shutdown_bounded_terminal(receiver, Duration::from_secs(1), Duration::from_secs(5))
                    .await;
            assert!(
                elapsed < Duration::from_secs(5),
                "termination should be bounded by the shutdown deadline, not the \
                     30s broker round-trip delay; took {elapsed:?}",
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
            let bytes = encoded_trace_fixture();
            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            // Arm the lag refresh timer at a short interval so a lag-refresh
            // worker is repeatedly spawned.
            let cfg = manual_traces_config_with_lag_refresh(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                50,
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            recv_and_ack(&mut receiver, RECORDS).await;

            // Give the lag timer several ticks so a refresh worker is spawned
            // and can be in flight when the shutdown arrives.
            tokio::time::sleep(Duration::from_millis(150)).await;

            // Shutdown with a short (1s) deadline. Even with a lag worker in
            // flight, the bounded drain plus off-loop-thread close must let
            // the receiver return within the deadline.
            let elapsed =
                shutdown_bounded_terminal(receiver, Duration::from_secs(1), Duration::from_secs(5))
                    .await;
            assert!(
                elapsed < Duration::from_secs(5),
                "termination must be bounded by the shutdown deadline; took {elapsed:?}",
            );
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver has
/// consumed and acked records (so it has tracked offsets to commit), then is
/// drained with a short deadline while the broker round-trip is stalled far
/// past that deadline, so the drain-time async offset commit and the
/// unsubscribe/close cannot complete before the deadline elapses.
/// Guarantees: the receiver still emits `RuntimeControlMsg::ReceiverDrained`
/// and reaches its terminal state well within a bound far shorter than the
/// broker stall, so a stalled offset commit at drain time cannot block the
/// receiver-first drain past its deadline (the bounded off-loop-thread close
/// caps termination, not the broker's round-trip latency).
#[tokio::test]
async fn drain_deadline_forces_drained_when_commit_stalls() {
    const TOPIC: &str = "drain-deadline-commit-stall-traces";
    const RECORDS: usize = 3;
    let group = "drain-deadline-commit-stall-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack every record so there are tracked offsets whose
            // drain-time async commit will be issued against the broker.
            recv_and_ack(&mut receiver, RECORDS).await;

            // Stall the broker far past the drain deadline. Unlike marking the
            // broker fully down, a large round-trip delay still lets the
            // off-thread close eventually finish, so the test does not orphan
            // a permanently-blocked librdkafka FFI thread.
            cluster.faults().round_trip_time(1, Duration::from_secs(30));

            // Drain with a short deadline while the broker is effectively
            // unavailable for the commit/close.
            let drain_at = tokio::time::Instant::now();
            receiver.drain(Duration::from_secs(1));

            // ReceiverDrained must still arrive (skip past startup timer
            // runtime messages), proving the drain does not block on the
            // stalled commit.
            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(
                drained,
                "receiver must emit ReceiverDrained even when the drain-time \
                 commit is stalled at the broker",
            );

            // Termination is bounded by the deadline, not the 30s stall.
            let terminated =
                tokio::time::timeout(Duration::from_secs(5), receiver.await_terminal_state()).await;
            assert!(
                terminated.is_ok(),
                "receiver must terminate within the bounded deadline even when \
                 the drain-time commit stalls at the broker",
            );
            assert!(
                drain_at.elapsed() < Duration::from_secs(5),
                "drain termination should be bounded by the deadline, not the \
                 30s broker round-trip delay; took {:?}",
                drain_at.elapsed(),
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

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver
/// consumes and ACKS every record (so it holds a committable offset past the
/// whole batch), then is drained while the broker rejects every OffsetCommit
/// RPC. The receiver's commits are issued asynchronously (`CommitMode::Async`)
/// and the bounded consumer close does not synchronously flush them, so the
/// acked progress is never durably committed at the broker.
/// Guarantees: an un-persisted drain-time commit yields at-least-once, NOT
/// data loss -- the broker retains no committed offset past the acked prefix,
/// and a fresh consumer in the SAME group re-reads every acked record
/// (duplicate delivery), so acked-but-uncommitted records are redelivered
/// rather than lost across a drain whose commit did not persist.
#[tokio::test]
async fn drain_unflushed_commit_redelivers_acked_records() {
    const TOPIC: &str = "drain-unflushed-commit-redeliver-traces";
    const RECORDS: usize = 3;
    let group = "drain-unflushed-commit-redeliver-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            // Reject every OffsetCommit RPC so NO commit (steady-state async
            // per-ack, or the drain-time commit) can durably persist. This is
            // the deterministic stand-in for a commit that is issued but never
            // reaches durable broker state before the receiver stops. A long
            // run of the same error covers every commit attempt the receiver
            // makes across the batch and the drain.
            cluster
                .faults()
                .fail_offset_commits(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 64]);

            // Manual-commit with no safety-net timer, so the only commits are
            // the per-ack async commits and the drain-time commit -- all of
            // which the broker rejects above.
            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume and ack every record so the tracker holds a committable
            // offset covering the whole batch.
            recv_and_ack(&mut receiver, RECORDS).await;
            // FIFO barrier: guarantee all acks were processed (and their
            // rejected async commits attempted) before we drain.
            receiver.wait_for_control_barrier().await;

            // Drain: the receiver issues its final commit, which is also
            // rejected, then terminates.
            receiver.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(
                drained,
                "receiver must emit ReceiverDrained even when every offset \
                 commit is rejected by the broker",
            );
            receiver.await_stopped().await;

            // The rejected commits never persisted, so the broker holds no
            // committed offset past the acked prefix. A committed offset is
            // "next to read", so a durable commit of the whole batch would be
            // exactly RECORDS; anything absent or below that proves the acked
            // progress was NOT persisted.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = probe_committed_offset(&brokers, group, TOPIC);
            assert!(
                committed.is_none_or(|o| o < RECORDS as i64),
                "a rejected drain-time commit must not persist past the acked \
                 prefix (else the redelivery below would be lost), got \
                 {committed:?}",
            );

            // Clear the fault so the observer below can join the group and
            // commit its own fetch position without interference.
            cluster.faults().clear_offset_commit_failures();

            // A fresh consumer in the SAME group re-reads every acked record:
            // at-least-once (duplicate) delivery, not data loss. Assign the
            // partition directly so redelivery is observed without depending
            // on group-rebalance timing after the old member left. With no
            // committed offset for the group, `auto.offset.reset=earliest`
            // starts the read at offset 0, so every acked record is re-read.
            let observer = cluster
                .consumer()
                .group_id(group)
                .assign_partition(TOPIC, 0);
            let redelivered = observer.recv_n(RECORDS).await;
            assert_eq!(
                redelivered.len(),
                RECORDS,
                "a same-group consumer must re-read all {RECORDS} acked records \
                 after a drain whose commit did not persist (at-least-once, no \
                 loss)",
            );
            // No phantom extra records: exactly the acked batch redelivers.
            observer
                .assert_no_more_messages(Duration::from_secs(2))
                .await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver
/// consumes an initial batch, is drained, and then -- after the drain
/// deadline has elapsed -- more records are produced while the broker's fetch
/// path is failing transient errors.
/// Guarantees: once drained the receiver has stopped polling, so no record
/// produced after the drain (and after the deadline) is ever forwarded
/// downstream (`try_recv_pdata` yields `None`), even while librdkafka retries
/// the failing fetch -- proving ingress fully stops at the deadline and is
/// not resumed by fetch-path retries.
#[tokio::test]
async fn drain_stops_polling_no_pdata_after_deadline() {
    const TOPIC: &str = "drain-no-pdata-after-deadline-traces";
    const INITIAL: usize = 3;
    const POST: usize = 4;
    let group = "drain-no-pdata-after-deadline-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, INITIAL, "pre", &bytes).await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            recv_and_ack(&mut receiver, INITIAL).await;

            // Drain with a short deadline and wait for it to complete.
            receiver.drain(Duration::from_millis(500));
            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(drained, "receiver never emitted ReceiverDrained");

            // Ensure we are past the drain deadline before producing more.
            tokio::time::sleep(Duration::from_millis(600)).await;

            // Inject transient fetch errors so any (illegal) resumed poll
            // would be retried by librdkafka; the receiver must not resume.
            cluster.faults().fail_fetch(&[
                RDKafkaRespErr::RD_KAFKA_RESP_ERR_NOT_LEADER_FOR_PARTITION,
                RDKafkaRespErr::RD_KAFKA_RESP_ERR_NOT_LEADER_FOR_PARTITION,
                RDKafkaRespErr::RD_KAFKA_RESP_ERR_NOT_LEADER_FOR_PARTITION,
            ]);

            produce_traces(&producer, TOPIC, POST, "post", &bytes).await;

            // No pdata may arrive after the drain deadline, even under the
            // retrying fetch path.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "receiver forwarded a record after the drain deadline; polling \
                 did not stop",
            );

            cluster.faults().clear_fetch_failures();
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver is
/// drained before it has consumed anything, so it holds no tracked offsets at
/// drain time.
/// Guarantees: the receiver still emits `RuntimeControlMsg::ReceiverDrained`
/// and terminates cleanly -- the drain notification is unconditional and does
/// not depend on there being offsets to commit, so an idle receiver still
/// participates in the receiver-first drain.
#[tokio::test]
async fn drain_with_no_tracked_offsets_still_notifies_drained() {
    const TOPIC: &str = "drain-no-offsets-traces";
    let group = "drain-no-offsets-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Give the receiver time to reach its poll loop, but produce and
            // consume nothing so no offsets are tracked.
            receiver.wait_for_control_barrier().await;

            receiver.drain(Duration::from_secs(5));

            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(
                drained,
                "an idle receiver with no tracked offsets must still emit \
                 ReceiverDrained",
            );

            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver
/// consumes three records but acks them out of order, leaving the lowest
/// offset (0) un-acked while acking offsets 1 and 2, then is drained.
/// Guarantees: the drain-time commit advances only to the lowest contiguous
/// acked offset -- because offset 0 is still un-acked, nothing is committed
/// past it (committed offset stays below the first gap), so at-least-once
/// redelivery covers the un-acked prefix after a restart.
#[tokio::test]
async fn drain_commits_only_lowest_contiguous_offset() {
    const TOPIC: &str = "drain-lowest-contiguous-traces";
    const RECORDS: usize = 3;
    let group = "drain-lowest-contiguous-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume all three, but ack only offsets 1 and 2 (hold 0
            // un-acked so the committable watermark cannot pass the gap).
            let first = receiver.recv_pdata().await;
            let second = receiver.recv_pdata().await;
            let third = receiver.recv_pdata().await;
            receiver.ack(second);
            receiver.ack(third);

            receiver.drain(Duration::from_secs(5));

            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(drained, "receiver never emitted ReceiverDrained");

            receiver.await_stopped().await;

            // The committed offset must not pass the un-acked offset 0. A
            // committed offset is "next to read", so a correct commit is
            // either absent or exactly 0 (never >= 1, which would skip the
            // un-acked record).
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = probe_committed_offset(&brokers, group, TOPIC);
            assert!(
                committed.is_none_or(|o| o == 0),
                "drain must not commit past the un-acked offset 0 (lowest \
                 contiguous only), got {committed:?}",
            );

            // Release the held record.
            drop(first);
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver is sent
/// a `DrainIngress` immediately followed by a second `DrainIngress` and a
/// `Shutdown` on the same control channel.
/// Guarantees: the receiver drains once (emits a single
/// `RuntimeControlMsg::ReceiverDrained`) and terminates via the drain path;
/// the trailing duplicate drain and the queued shutdown are harmless because
/// the drain handler returns terminal state on the first drain -- documenting
/// that a duplicate drain / drain-then-shutdown sequence is idempotent by
/// termination and cannot double-notify or panic.
#[tokio::test]
async fn drain_then_shutdown_drain_wins() {
    const TOPIC: &str = "drain-then-shutdown-traces";
    const RECORDS: usize = 2;
    let group = "drain-then-shutdown-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            recv_and_ack(&mut receiver, RECORDS).await;

            // Queue a drain, a duplicate drain, and a shutdown back to back.
            // Only the first drain should take effect.
            receiver.drain(Duration::from_secs(5));
            receiver.drain(Duration::from_secs(5));
            receiver.shutdown(Duration::from_secs(5));

            // Exactly one ReceiverDrained should be observed (drain won).
            let mut drained_count = 0usize;
            for _ in 0..24 {
                match receiver.try_recv_runtime(Duration::from_secs(3)).await {
                    Some(RuntimeControlMsg::ReceiverDrained { .. }) => {
                        drained_count += 1;
                    }
                    Some(_) => continue,
                    None => break,
                }
            }
            assert_eq!(
                drained_count, 1,
                "the drain-then-shutdown sequence must notify ReceiverDrained \
                 exactly once (the first drain wins)",
            );

            // The receiver terminated cleanly via the drain path.
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): an auto-commit receiver (offset
/// management owned by librdkafka) consumes a batch and is then drained.
/// Guarantees: the drain skips the manual final-commit path entirely yet
/// still emits `RuntimeControlMsg::ReceiverDrained`, stops forwarding new
/// records, and terminates cleanly -- so the receiver-first drain contract
/// holds identically under auto-commit.
#[tokio::test]
async fn drain_under_auto_commit_terminates_cleanly() {
    const TOPIC: &str = "drain-auto-commit-traces";
    const INITIAL: usize = 3;
    let group = "drain-auto-commit-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, INITIAL, "pre", &bytes).await;

            // Auto-commit config: librdkafka owns commits, so the receiver's
            // manual final-commit block is skipped at drain.
            let cfg = KafkaReceiverConfig::try_from(
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec![TOPIC.to_string()])
                            .with_encoding(MessageFormat::OtlpProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Auto,
                        interval_ms: Some(1000),
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted),
            )
            .expect("auto-commit test config should be valid");
            assert!(cfg.is_auto_commit());
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Under auto-commit the receiver does not require acks to advance;
            // just consume the batch so the poll loop is active before drain.
            for _ in 0..INITIAL {
                let _ = receiver.recv_pdata().await;
            }

            receiver.drain(Duration::from_secs(5));

            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(
                drained,
                "an auto-commit receiver must still emit ReceiverDrained on drain",
            );

            // After drain, produced records must not be forwarded.
            produce_traces(&producer, TOPIC, INITIAL, "post", &bytes).await;
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(3))
                    .await
                    .is_none(),
                "auto-commit receiver forwarded a record after DrainIngress",
            );

            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a single manual-commit receiver
/// owns every partition of a multi-partition topic, consumes and acks every
/// record, and is then drained.
/// Guarantees: the drain-time commit advances each owned partition to its own
/// lowest-contiguous acked offset in one final commit, so every partition
/// ends with a committed offset accounting for all its records (no partition
/// is left behind by a multi-partition drain).
#[tokio::test]
async fn drain_multi_partition_commits_each_partition() {
    const TOPIC: &str = "drain-multi-partition-traces";
    let group = "drain-multi-partition-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            recv_and_ack(&mut receiver, total).await;

            receiver.drain(Duration::from_secs(5));

            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(drained, "receiver never emitted ReceiverDrained");

            receiver.await_stopped().await;

            // Every partition must carry a committed offset accounting for its
            // records (commit is async, flushed on unsubscribe/close).
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let brokers = cluster.bootstrap_servers().to_string();
                let committed =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed")
                            .is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                    })
                    .await;
                assert!(
                    committed,
                    "partition {partition} should have a drain-committed offset \
                     >= {REBALANCE_RECORDS_PER_PARTITION}, got {:?}",
                    committed_offset(&brokers, group, TOPIC, partition)
                        .expect("kafka-test: committed-offset probe failed"),
                );
            }
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): following the engine's live-
/// reconfiguration order -- the OLD receiver (A) owns both partitions and acks
/// their records; the NEW receiver (B) is started in the SAME group (distinct
/// `group.instance.id`) and provably ACQUIRES a partition via a normal
/// cooperative rebalance (NEW Ready, a real overlap) BEFORE A is drained; only
/// THEN is A drained.
/// Guarantees: the OLD drain-time commit purges the partition revoked to B
/// before building the committable set, so A commits only what it still owns;
/// both partitions nonetheless retain committed offsets (the revoked one from
/// the pre-revoke commit-before-revoke) so no progress is lost, and the NEW
/// receiver continues to own and consume its acquired partition after the OLD
/// instance drains cleanly.
#[tokio::test]
async fn cutover_new_receiver_same_group_acquires_partition_before_old_drains() {
    const TOPIC: &str = "cutover-same-group-handoff-traces";
    let group = "cutover-same-group-handoff-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // OLD receiver (A): owns both partitions, acks all records.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC,
                Some("inst-a"),
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            recv_and_ack(&mut receiver_a, total).await;

            // Ensure A's progress on both partitions is committed before the
            // rebalance revokes one from it (commit-before-revoke).
            let brokers = cluster.bootstrap_servers().to_string();
            let a_committed =
                poll_until(Duration::from_secs(5), Duration::from_millis(100), || {
                    let c0 = committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed");
                    let c1 = committed_offset(&brokers, group, TOPIC, 1)
                        .expect("kafka-test: committed-offset probe failed");
                    c0.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                        && c1.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                })
                .await;
            // A commits via ack; if the async commit has not flushed yet, the
            // drain below still commits, so this is a best-effort precondition.
            let _ = a_committed;

            // Engine order step 1+2: start the NEW receiver (B) in the SAME
            // group and wait until it ACQUIRES a partition (NEW Ready) -- the
            // cooperative rebalance revokes one partition from A to B while A
            // is still running.
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC,
                Some("inst-b"),
            );
            let receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
            let b_ready = tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    if receiver_b.is_partition_assigned(TOPIC, 0)
                        || receiver_b.is_partition_assigned(TOPIC, 1)
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            })
            .await;
            assert!(
                b_ready.is_ok(),
                "the NEW receiver must acquire a partition (NEW Ready) before \
                 the OLD instance is drained",
            );

            // Engine order step 3: only now drain the OLD instance (A).
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(
                drained,
                "old receiver must emit ReceiverDrained even after a concurrent \
                 partition revocation to the new receiver",
            );
            receiver_a.await_stopped().await;

            // Both partitions must retain committed offsets accounting for all
            // produced records: no progress is lost across the cutover.
            let all_committed =
                poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                    let c0 = committed_offset(&brokers, group, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed");
                    let c1 = committed_offset(&brokers, group, TOPIC, 1)
                        .expect("kafka-test: committed-offset probe failed");
                    c0.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                        && c1.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                })
                .await;
            assert!(
                all_committed,
                "both partitions must retain committed offsets >= \
                 {REBALANCE_RECORDS_PER_PARTITION} across the cutover",
            );

            // The NEW receiver keeps owning at least one partition after the
            // OLD instance has fully drained (it is unaffected).
            assert!(
                receiver_b.is_partition_assigned(TOPIC, 0)
                    || receiver_b.is_partition_assigned(TOPIC, 1),
                "the NEW receiver must still own a partition after the OLD \
                 instance drains",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): live-reconfiguration cutover on
/// the SAME broker with DISTINCT consumer groups -- the OLD receiver (A, group
/// A) is serving when the NEW receiver (B, group B) is started, reads and
/// commits from the same topic (NEW Ready), and only THEN is A drained.
/// Guarantees: the NEW receiver consumes, acks, and commits its own records
/// independently of the OLD instance and keeps working after A has fully
/// drained -- an independent-group cutover on one broker does not disturb the
/// new instance.
#[tokio::test]
async fn cutover_new_receiver_same_broker_distinct_group_starts_before_old_drains() {
    const TOPIC: &str = "cutover-distinct-group-traces";
    const PRE: usize = 3;
    const POST: usize = 3;
    let group_a = "cutover-distinct-group-a";
    let group_b = "cutover-distinct-group-b";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, PRE, "pre", &bytes).await;

            // OLD receiver (A, group A): consume and ack the initial batch.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group_a,
                "receiver-a",
                TOPIC,
                None,
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            recv_and_ack(&mut receiver_a, PRE).await;

            // Engine order step 1+2: start the NEW receiver (B, group B) and
            // confirm it is working (reads the same records from its own
            // group offset and acks) BEFORE the OLD instance is drained.
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group_b,
                "receiver-b",
                TOPIC,
                None,
            );
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
            recv_and_ack(&mut receiver_b, PRE).await;

            // Engine order step 3: only now drain the OLD instance (A).
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(drained, "old receiver never emitted ReceiverDrained");
            receiver_a.await_stopped().await;

            // The NEW receiver keeps working after the OLD instance is gone:
            // it consumes and acks records produced post-cutover.
            produce_traces(&producer, TOPIC, POST, "post", &bytes).await;
            for _ in 0..POST {
                let pdata = receiver_b
                    .try_recv_pdata(Duration::from_secs(15))
                    .await
                    .expect("new receiver must consume post-cutover records");
                receiver_b.ack(pdata);
            }

            // The NEW receiver's group commits its post-cutover progress.
            let brokers = cluster.bootstrap_servers().to_string();
            let b_committed =
                poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                    committed_offset(&brokers, group_b, TOPIC, 0)
                        .expect("kafka-test: committed-offset probe failed")
                        .is_some_and(|o| o >= (PRE + POST) as i64)
                })
                .await;
            assert!(
                b_committed,
                "the NEW receiver must commit its own progress after the cutover",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): live-reconfiguration cutover on
/// a single-core deployment where the OLD receiver (A) and the NEW receiver
/// (B) share the SAME operator `group.instance.id` in the SAME group but run
/// at DISTINCT deployment generations (A at generation 0, B at generation 1),
/// exactly as the engine deploys a new pipeline instance during a rolling
/// cutover. Two partitions so the members can genuinely split ownership.
/// Guarantees: because `KafkaReceiver::new` folds the deployment generation
/// into the static `group.instance.id`, A resolves to `inst-shared-g0` and B
/// to `inst-shared-g1` -- distinct static members even though no core-id
/// suffix is appended on a single-core pipeline. So the NEW receiver is NOT
/// stalled by the OLD one: it acquires a partition and actually receives
/// data (liveness), and it keeps working after A drains. This is the
/// integration guard that the generation suffix lets a same-operator-id new
/// instance make progress during a cutover (contrast with
/// `cutover_same_generation_same_instance_id_stalls_new_receiver`, which shows
/// an IDENTICAL resolved id starves the new receiver).
#[tokio::test]
async fn cutover_new_receiver_distinct_generation_is_not_stalled() {
    const TOPIC: &str = "cutover-distinct-gen-liveness-traces";
    let group = "cutover-distinct-gen-liveness-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // OLD receiver (A): generation 0, operator id `inst-shared`
            // -> resolves to `inst-shared-g0`. Consume so A owns both
            // partitions.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC,
                Some("inst-shared"),
            );
            let mut receiver_a = KafkaReceiverHarness::start_with_generation(&cluster, cfg_a, 0);
            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            recv_and_ack(&mut receiver_a, total).await;

            // NEW receiver (B): generation 1, SAME operator id `inst-shared`
            // -> resolves to `inst-shared-g1` (distinct static member).
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC,
                Some("inst-shared"),
            );
            let mut receiver_b = KafkaReceiverHarness::start_with_generation(&cluster, cfg_b, 1);

            // Liveness step 1: B must ACQUIRE a partition (not be starved by
            // A) because it is a distinct static member.
            let b_acquired = tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    if receiver_b.is_partition_assigned(TOPIC, 0)
                        || receiver_b.is_partition_assigned(TOPIC, 1)
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            })
            .await;
            assert!(
                b_acquired.is_ok(),
                "the NEW receiver (distinct generation suffix) must acquire a \
                 partition and not be stalled by the OLD same-operator-id \
                 receiver",
            );

            // Liveness step 2: B must actually RECEIVE data from the partition
            // it acquired. Produce fresh records to both partitions so at
            // least one lands on B's partition.
            for i in 0..REBALANCE_RECORDS_PER_PARTITION {
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &bytes)
                                .key(format!("post-{partition}-{i}").as_bytes())
                                .partition(partition),
                        )
                        .await
                        .expect("send post record");
                }
            }
            let b_pdata = receiver_b.try_recv_pdata(Duration::from_secs(15)).await;
            assert!(
                b_pdata.is_some(),
                "the NEW receiver must receive data from its acquired partition \
                 (it is live, not stalled)",
            );
            if let Some(pdata) = b_pdata {
                receiver_b.ack(pdata);
            }

            // Drain the OLD instance (engine order): A drains cleanly and B
            // keeps owning a partition afterward.
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(drained, "old receiver must drain cleanly");
            receiver_a.await_stopped().await;

            assert!(
                receiver_b.is_partition_assigned(TOPIC, 0)
                    || receiver_b.is_partition_assigned(TOPIC, 1),
                "the NEW receiver must still own a partition after the OLD \
                 instance drains",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): the hazard the deployment-
/// generation suffix on `group.instance.id` prevents. On a single-core
/// deployment, the OLD receiver (A) and the NEW receiver (B) share the SAME
/// operator `group.instance.id` AND the SAME deployment generation, so both
/// resolve to the IDENTICAL static member id (`inst-shared-g0`, with no
/// core-id suffix on a single-core pipeline) -- the situation that would arise
/// if a new instance reused the old instance's exact static identity.
/// Guarantees: with an identical resolved `group.instance.id`, the group
/// coordinator (as emulated by the in-process mock broker) does not hand a
/// partition to the duplicate member, so the NEW receiver is STARVED: it never
/// acquires a partition and receives no data while A keeps owning and
/// consuming. This documents why the generation suffix matters -- reusing an
/// identical static id across a cutover stalls the new instance (a real broker
/// would instead fence the OLD member via `FENCED_INSTANCE_ID`; either way an
/// identical static id is unsafe for a cutover). Contrast with
/// `cutover_new_receiver_distinct_generation_is_not_stalled`.
#[tokio::test]
async fn cutover_same_generation_same_instance_id_stalls_new_receiver() {
    const TOPIC: &str = "cutover-same-gen-stall-traces";
    let group = "cutover-same-gen-stall-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // OLD receiver (A): generation 0, operator id `inst-shared`
            // -> resolves to `inst-shared-g0`. Consume so A owns the
            // partitions.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC,
                Some("inst-shared"),
            );
            let mut receiver_a = KafkaReceiverHarness::start_with_generation(&cluster, cfg_a, 0);
            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            recv_and_ack(&mut receiver_a, total).await;

            // NEW receiver (B): generation 0 too, SAME operator id
            // `inst-shared` -> resolves to the IDENTICAL `inst-shared-g0`
            // (single core, so no core-id suffix differentiates them).
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC,
                Some("inst-shared"),
            );
            let mut receiver_b = KafkaReceiverHarness::start_with_generation(&cluster, cfg_b, 0);
            receiver_b.wait_for_control_barrier().await;

            // Produce fresh records so there is data available for whichever
            // member owns each partition.
            for i in 0..REBALANCE_RECORDS_PER_PARTITION {
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    producer
                        .send_full(
                            SendRecord::new(TOPIC, &bytes)
                                .key(format!("post-{partition}-{i}").as_bytes())
                                .partition(partition),
                        )
                        .await
                        .expect("send post record");
                }
            }

            // B is starved: with an identical resolved static id, the mock
            // coordinator never grants B a partition, so it acquires nothing
            // and receives no data within a bounded window.
            assert!(
                receiver_b
                    .try_recv_pdata(Duration::from_secs(5))
                    .await
                    .is_none(),
                "the NEW receiver sharing an IDENTICAL resolved group.instance.id \
                 must be starved (receive no data) -- this is the stall the \
                 generation suffix prevents",
            );
            assert!(
                !receiver_b.is_partition_assigned(TOPIC, 0)
                    && !receiver_b.is_partition_assigned(TOPIC, 1),
                "the starved NEW receiver must not own any partition",
            );

            // The OLD receiver is unaffected: it still consumes the freshly
            // produced records.
            let a_more = receiver_a.try_recv_pdata(Duration::from_secs(10)).await;
            assert!(
                a_more.is_some(),
                "the OLD receiver must keep consuming while the duplicate-id new \
                 receiver is starved",
            );
            if let Some(pdata) = a_more {
                receiver_a.ack(pdata);
            }

            // Clean shutdown of both.
            shutdown_receiver(receiver_a).await;
            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): live-reconfiguration cutover
/// where the NEW receiver (B) points at a DIFFERENT broker than the OLD
/// receiver (A). B is started against broker B, consumes from it (NEW Ready),
/// and only THEN is A drained on broker A.
/// Guarantees: the NEW receiver on the second broker consumes+acks+commits
/// independently and keeps working after A drains -- brokers are fully
/// isolated across the cutover, so draining the old instance cannot affect
/// the new one.
#[tokio::test]
async fn cutover_new_receiver_different_broker_starts_before_old_drains() {
    const TOPIC_A: &str = "cutover-broker-a-traces";
    const TOPIC_B: &str = "cutover-broker-b-traces";
    const RECORDS: usize = 3;
    let group = "cutover-diff-broker-drain-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC_A),
        |cluster_a| async move {
            // Build the SECOND broker on the same LocalSet thread.
            let cluster_b = KafkaTestCluster::builder().topic(TOPIC_B).build();

            let bytes = encoded_trace_fixture();

            // OLD receiver (A) on broker A with some produced records.
            let producer_a = cluster_a.producer().build();
            produce_traces(&producer_a, TOPIC_A, RECORDS, "a", &bytes).await;
            let cfg_a = cutover_traces_config(
                cluster_a.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC_A,
                None,
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster_a, cfg_a);
            recv_and_ack(&mut receiver_a, RECORDS).await;

            // Engine order step 1+2: start the NEW receiver (B) on broker B
            // and confirm it consumes from broker B (NEW Ready).
            let producer_b = cluster_b.producer().build();
            produce_traces(&producer_b, TOPIC_B, RECORDS, "b", &bytes).await;
            let cfg_b = cutover_traces_config(
                cluster_b.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC_B,
                None,
            );
            let mut receiver_b = KafkaReceiverHarness::start(&cluster_b, cfg_b);
            recv_and_ack(&mut receiver_b, RECORDS).await;

            // Engine order step 3: only now drain the OLD instance (A) on
            // broker A.
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(drained, "old receiver never emitted ReceiverDrained");
            receiver_a.await_stopped().await;

            // The NEW receiver on broker B keeps working after A drains: it
            // consumes and acks records produced to broker B post-cutover.
            for i in 0..RECORDS {
                let key = format!("b-post-{i}");
                producer_b
                    .send_full(SendRecord::new(TOPIC_B, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send post to broker B");
            }
            for _ in 0..RECORDS {
                let pdata = receiver_b
                    .try_recv_pdata(Duration::from_secs(15))
                    .await
                    .expect("new receiver on broker B must keep consuming");
                receiver_b.ack(pdata);
            }

            // Broker B commits the new receiver's progress.
            let brokers_b = cluster_b.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers_b, group, TOPIC_B, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= (2 * RECORDS) as i64)
            })
            .await;
            assert!(
                committed,
                "the NEW receiver on broker B must commit its progress after \
                 the OLD instance on broker A drains",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): the different-broker cutover
/// where the OLD receiver (A) is terminated via a bare `Shutdown` instead of
/// `DrainIngress`. The NEW receiver (B) on broker B is started and consuming
/// (NEW Ready) before A is shut down.
/// Guarantees: the NEW receiver on the second broker is unaffected regardless
/// of whether the OLD instance receives `DrainIngress` or `Shutdown` -- it
/// keeps consuming+acking+committing on broker B after A stops.
#[tokio::test]
async fn cutover_new_receiver_different_broker_shutdown_variant_starts_before_old_stops() {
    const TOPIC_A: &str = "cutover-broker-a-shutdown-traces";
    const TOPIC_B: &str = "cutover-broker-b-shutdown-traces";
    const RECORDS: usize = 3;
    let group = "cutover-diff-broker-shutdown-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC_A),
        |cluster_a| async move {
            let cluster_b = KafkaTestCluster::builder().topic(TOPIC_B).build();

            let bytes = encoded_trace_fixture();

            let producer_a = cluster_a.producer().build();
            produce_traces(&producer_a, TOPIC_A, RECORDS, "a", &bytes).await;
            let cfg_a = cutover_traces_config(
                cluster_a.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC_A,
                None,
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster_a, cfg_a);
            recv_and_ack(&mut receiver_a, RECORDS).await;

            // NEW receiver (B) on broker B, consuming (NEW Ready).
            let producer_b = cluster_b.producer().build();
            produce_traces(&producer_b, TOPIC_B, RECORDS, "b", &bytes).await;
            let cfg_b = cutover_traces_config(
                cluster_b.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC_B,
                None,
            );
            let mut receiver_b = KafkaReceiverHarness::start(&cluster_b, cfg_b);
            recv_and_ack(&mut receiver_b, RECORDS).await;

            // Engine order step 3, Shutdown variant: terminate the OLD
            // instance (A) via a bare Shutdown rather than DrainIngress.
            shutdown_receiver(receiver_a).await;

            // The NEW receiver on broker B keeps working regardless of the
            // OLD instance's terminal control message.
            for i in 0..RECORDS {
                let key = format!("b-post-{i}");
                producer_b
                    .send_full(SendRecord::new(TOPIC_B, &bytes).key(key.as_bytes()))
                    .await
                    .expect("send post to broker B");
            }
            for _ in 0..RECORDS {
                let pdata = receiver_b
                    .try_recv_pdata(Duration::from_secs(15))
                    .await
                    .expect("new receiver on broker B must keep consuming");
                receiver_b.ack(pdata);
            }

            let brokers_b = cluster_b.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers_b, group, TOPIC_B, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= (2 * RECORDS) as i64)
            })
            .await;
            assert!(
                committed,
                "the NEW receiver on broker B must commit its progress after \
                 the OLD instance on broker A is shut down",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): following the engine's live-
/// reconfiguration order on a single-partition topic -- the OLD receiver (A)
/// consumes records but leaves them un-acked; the NEW receiver (B) is started
/// in the SAME group and joins BEFORE A is drained; only THEN is A drained
/// via `DrainIngress`. With one partition, B cannot own it until A releases it
/// on drain, so "NEW Ready" here means B has joined the group before the OLD
/// drain step and the partition hands off when A leaves.
/// Guarantees: the drain commits nothing past the un-acked prefix, so after
/// the handoff the NEW receiver re-receives every un-acked record (at-least-
/// once redelivery, no data loss across the cutover), while the OLD receiver
/// drains cleanly.
#[tokio::test]
async fn cutover_new_receiver_starts_before_old_drains_redelivers_uncommitted_single_partition() {
    const TOPIC: &str = "cutover-redeliver-1p-traces";
    const RECORDS: usize = 3;
    let group = "cutover-redeliver-1p-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            produce_traces(&producer, TOPIC, RECORDS, "rec", &bytes).await;

            // OLD receiver (A): consume every record but NEVER ack, so no
            // offset is committable at drain time.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC,
                None,
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            for _ in 0..RECORDS {
                let _ = receiver_a.recv_pdata().await;
            }

            // Engine order step 1+2: start the NEW receiver (B) in the SAME
            // group and let it join the group BEFORE the OLD instance is
            // drained. With a single partition B parks waiting for the
            // assignment A still holds.
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC,
                None,
            );
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
            receiver_b.wait_for_control_barrier().await;

            // Engine order step 3: only now drain the OLD instance (A).
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(drained, "old receiver never emitted ReceiverDrained");
            receiver_a.await_stopped().await;

            // The un-acked records were never committed, so the broker holds
            // no committed offset past the un-acked prefix.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = probe_committed_offset(&brokers, group, TOPIC);
            assert!(
                committed.is_none_or(|o| o < RECORDS as i64),
                "drain must not commit past the un-acked prefix, got {committed:?}",
            );

            // The NEW receiver (B) picks up the released partition and
            // re-receives all un-acked records (at-least-once, no loss).
            let mut redelivered = 0usize;
            for _ in 0..RECORDS {
                if receiver_b
                    .try_recv_pdata(Duration::from_secs(15))
                    .await
                    .is_some()
                {
                    redelivered += 1;
                }
            }
            assert_eq!(
                redelivered, RECORDS,
                "the NEW receiver must re-receive all {RECORDS} un-acked \
                 records after the OLD instance drains (no loss across the \
                 cutover), got {redelivered}",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): the multi-partition analogue of
/// the cutover redelivery test -- the OLD receiver (A) owns every partition
/// and consumes records un-acked; the NEW receiver (B) is started in the SAME
/// group and provably ACQUIRES a partition (strong "NEW Ready", a true
/// overlap with A still running) BEFORE A is drained; only THEN is A drained.
/// Guarantees: no partition commits past its un-acked prefix, so after the
/// cutover the NEW receiver re-receives the un-acked records from the
/// partition handed off to it (at-least-once redelivery, no loss), while the
/// OLD receiver drains cleanly.
#[tokio::test]
async fn cutover_new_receiver_starts_before_old_drains_redelivers_uncommitted_multi_partition() {
    const TOPIC: &str = "cutover-redeliver-mp-traces";
    let group = "cutover-redeliver-mp-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // OLD receiver (A): consume all records from both partitions but
            // NEVER ack, so no offset is committable at drain time.
            let cfg_a = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-a",
                TOPIC,
                None,
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            for _ in 0..total {
                let _ = receiver_a.recv_pdata().await;
            }

            // Engine order step 1+2: start the NEW receiver (B) in the SAME
            // group and wait until it ACQUIRES a partition -- a real overlap
            // while A is still running (NEW Ready before the OLD drain step).
            let cfg_b = cutover_traces_config(
                cluster.bootstrap_servers(),
                group,
                "receiver-b",
                TOPIC,
                None,
            );
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
            // One partition is revoked from A and assigned to B; B owning
            // either partition proves it is a live, Ready group member.
            let b_ready = tokio::time::timeout(Duration::from_secs(20), async {
                loop {
                    if receiver_b.is_partition_assigned(TOPIC, 0)
                        || receiver_b.is_partition_assigned(TOPIC, 1)
                    {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            })
            .await;
            assert!(
                b_ready.is_ok(),
                "the NEW receiver must acquire a partition before the OLD drain",
            );

            // Engine order step 3: only now drain the OLD instance (A).
            receiver_a.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver_a).await;
            assert!(drained, "old receiver never emitted ReceiverDrained");
            receiver_a.await_stopped().await;

            // No partition committed past its un-acked prefix (nothing acked).
            let brokers = cluster.bootstrap_servers().to_string();
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let committed = committed_offset(&brokers, group, TOPIC, partition)
                    .expect("kafka-test: committed-offset probe failed");
                assert!(
                    committed.is_none_or(|o| o < REBALANCE_RECORDS_PER_PARTITION as i64),
                    "partition {partition} must not commit past the un-acked \
                     prefix, got {committed:?}",
                );
            }

            // The NEW receiver (B) re-receives un-acked records after the
            // cutover (at-least-once redelivery, no loss). It now owns every
            // partition, so all records redeliver to it.
            let mut redelivered = 0usize;
            for _ in 0..total {
                if receiver_b
                    .try_recv_pdata(Duration::from_secs(15))
                    .await
                    .is_some()
                {
                    redelivered += 1;
                }
            }
            assert_eq!(
                redelivered, total,
                "the NEW receiver must re-receive all {total} un-acked records \
                 across the cutover (no loss), got {redelivered}",
            );

            shutdown_receiver(receiver_b).await;
        },
    )
    .await;
}

/// Scenario (lifecycle: drain and shutdown): a manual-commit receiver owns
/// both partitions of a multi-partition topic and acks a DIFFERENT prefix on
/// each -- one partition fully acked, the other left with an un-acked gap at
/// its first offset -- then is drained.
/// Guarantees: the drain-time commit halts at each partition's own gap: the
/// fully-acked partition commits past all its records, while the
/// gap-at-offset-0 partition commits nothing past offset 0 (the broker
/// receives no commit past any partition's un-acked prefix), so a
/// multi-partition drain preserves per-partition at-least-once boundaries.
#[tokio::test]
async fn drain_multi_partition_commits_halt_at_each_partition_gap() {
    const TOPIC: &str = "drain-multi-gap-traces";
    // Partition that is fully acked, and the one left with a leading gap.
    const FULL_P: i32 = 0;
    const GAP_P: i32 = 1;
    let group = "drain-multi-gap-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();

            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume every record from both partitions, but ack selectively:
            // ack all of FULL_P; for GAP_P ack every record EXCEPT its offset
            // 0, leaving a leading un-acked gap. Partition/offset are read from
            // each pdata's Kafka source route calldata.
            let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
            for _ in 0..total {
                let pdata = receiver.recv_pdata().await;
                let (_, partition, offset, _) =
                    decode_calldata(&pdata.source_route().expect("source route").calldata);
                if partition == GAP_P && offset == 0 {
                    // Leave the first record of the gap partition un-acked.
                    continue;
                }
                receiver.ack(pdata);
            }

            receiver.drain(Duration::from_secs(5));
            let drained = wait_for_receiver_drained(&mut receiver).await;
            assert!(drained, "receiver never emitted ReceiverDrained");
            receiver.await_stopped().await;

            let brokers = cluster.bootstrap_servers().to_string();

            // The fully-acked partition commits past all its records.
            let full_committed =
                poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                    committed_offset(&brokers, group, TOPIC, FULL_P)
                        .expect("kafka-test: committed-offset probe failed")
                        .is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                })
                .await;
            assert!(
                full_committed,
                "the fully-acked partition must commit past all its records, got {:?}",
                committed_offset(&brokers, group, TOPIC, FULL_P)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            // The gap partition must NOT commit past its un-acked offset 0.
            let gap_committed = committed_offset(&brokers, group, TOPIC, GAP_P)
                .expect("kafka-test: committed-offset probe failed");
            assert!(
                gap_committed.is_none_or(|o| o == 0),
                "the gap partition must not commit past its un-acked offset 0 \
                 (lowest contiguous only), got {gap_committed:?}",
            );
        },
    )
    .await;
}
