// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration coverage for a Kafka receiver that is draining while a consumer
//! group rebalance is in flight.
//!
//! These regression tests exercise the drain-under-rebalance path with the
//! librdkafka `MockCluster` (the repo's in-process Kafka "integration" harness,
//! no Docker). They run on the current-thread `LocalSet` provided by
//! [`with_cluster`] and use manual-commit receivers so the rebalance-aware
//! commit path stays active. They model the reference multi-receiver test
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
/// repeated rebalances during the drain.
/// Guarantees: the receiver task reaches its terminal state well before the
/// drain deadline (drain completes on its own, not by hitting the forced
/// deadline), proving repeated rebalance churn during drain cannot wedge the
/// receive loop.
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

            // Drain with a short deadline: the drain should complete on its own
            // well before this deadline even under churn.
            let drain_deadline = Duration::from_secs(5);
            let started = tokio::time::Instant::now();
            receiver_a.drain(drain_deadline);

            // Churn the group during the drain: repeatedly start and gracefully
            // stop a second receiver in the same group so the group rebalances
            // several times while A drains. A receiver harness makes no
            // "must be assigned" assumption (unlike `RebalanceTrigger`), so the
            // churn cannot panic on A's drain timing while still forcing real
            // join+leave rebalances. Each iteration yields to the current-thread
            // executor so A's task keeps making progress.
            for _ in 0..3 {
                let churn_cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let churn = KafkaReceiverHarness::start(&cluster, churn_cfg);
                tokio::time::sleep(Duration::from_millis(200)).await;
                churn.shutdown(Duration::from_secs(1));
                let _ = churn.await_terminal_state().await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }

            // The receiver task must reach its terminal state well before the
            // drain deadline plus a small margin; a wedged loop would only
            // return at the forced deadline (or time out here).
            let terminal = tokio::time::timeout(
                drain_deadline + Duration::from_secs(2),
                receiver_a.await_terminal_state(),
            )
            .await
            .expect("draining receiver task did not complete under rebalance churn");
            let elapsed = started.elapsed();
            assert!(
                elapsed < drain_deadline + Duration::from_secs(2),
                "draining receiver task took {elapsed:?}, at or past the drain deadline; the \
                 receive loop wedged under rebalance churn",
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
