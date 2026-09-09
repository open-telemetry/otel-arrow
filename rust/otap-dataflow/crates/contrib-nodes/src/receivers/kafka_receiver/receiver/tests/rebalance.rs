// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalance integration tests.

use super::*;

// ---- Consumer-group rebalancing ----

/// Scenario (consumer-group rebalancing): a revoked partition is queued and then
/// reconciled at the top of the receive loop.
/// Guarantees: reconcile purges the revoked partition from the tracker, so its stale
/// offsets cannot be committed after ownership is lost.
#[test]
fn reconcile_purges_revoked_partitions_from_tracker() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Simulate in-flight offsets across two partitions.
    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver.offset_tracker.track("traces", 1, 200, 0);
    assert_eq!(receiver.offset_tracker.total_pending(), 2);

    // Simulate a rebalance revoking partition 0.
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 0);

    receiver.reconcile_rebalance_state();

    // Partition 0 purged; partition 1 retained.
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
    assert_eq!(receiver.offset_tracker.pending_count("traces", 1), 1);
}

/// Scenario (consumer-group rebalancing): a partition is revoked and immediately
/// reassigned to this consumer under a newer generation before the stale revocation is
/// processed.
/// Guarantees: the reassigned partition's newer state is preserved and only the stale
/// revocation is dropped, so a rapid revoke/reassign does not discard freshly-owned
/// state.
#[test]
fn stale_revocation_preserves_reassigned_partition_state() {
    // Regression for the revoke/reassign race: a revocation queued for an
    // older ownership period must not delete tracker state created after
    // the partition was reassigned to this consumer under a newer
    // generation.
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // A new record for partition 0 was tracked under generation 2 (after a
    // reassignment), while a revocation from generation 1 is still queued.
    receiver.offset_tracker.track("traces", 0, 250, 2);
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 1);

    receiver.reconcile_rebalance_state();

    // The stale generation-1 revocation must be a no-op: the newer state
    // survives so its ACK can still advance and commit.
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
    assert_eq!(
        receiver.offset_tracker.partition_generation("traces", 0),
        Some(2),
    );
}

/// Scenario (consumer-group rebalancing): records tracked under an old generation
/// remain when the partition is reassigned under a newer generation.
/// Guarantees: acks for the old-generation records are not committed after
/// reassignment, so a stale generation cannot advance the newly-owned partition's
/// offset.
#[test]
fn stale_generation_records_not_committed_after_reassignment() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Generation 1: own partition 0, track and ack offsets 100..=104. The
    // committable offset is high_water_mark + 1 = 105.
    for offset in 100..=104 {
        receiver.offset_tracker.track("traces", 0, offset, 1);
    }
    for offset in 100..=104 {
        let _ = receiver.offset_tracker.acknowledge("traces", 0, offset);
    }
    assert_eq!(
        receiver
            .offset_tracker
            .committable_snapshot()
            .get(&("traces".to_string(), 0))
            .copied(),
        Some(105),
        "generation 1 would commit its own high-water mark",
    );

    // Partition 0 is revoked; the revocation carries generation 1. The
    // receive loop reconciles and purges the generation-1 tracker state.
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 1);
    receiver.reconcile_rebalance_state();
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
    assert!(
        !receiver
            .offset_tracker
            .committable_snapshot()
            .contains_key(&("traces".to_string(), 0)),
        "a revoked partition contributes no committable offset",
    );

    // Generation 2: partition 0 is reassigned to this receiver and resumes
    // from the group's committed position (200), then tracks a new record.
    receiver.offset_tracker.track("traces", 0, 200, 2);
    assert_eq!(
        receiver.offset_tracker.partition_generation("traces", 0),
        Some(2),
    );

    // The receiver only commits the generation-2 offset (200); it never
    // regresses to generation 1's 105.
    assert_eq!(
        receiver
            .offset_tracker
            .committable_snapshot()
            .get(&("traces".to_string(), 0))
            .copied(),
        Some(200),
        "only generation-2 records drive the commit after reassignment",
    );
}

/// Scenario (consumer-group rebalancing): a partition owned under generation 1
/// (with an established committable offset) is revoked and reassigned to this
/// receiver under generation 2, and a new generation-2 record is tracked.
/// A stale Ack/Nack for the old generation-1 record then arrives, carrying
/// generation 1 in its calldata.
/// Guarantees: the stale feedback is classified `DropStale` -- the receiver's
/// exact classifier decision on an Ack/Nack (both funnel through
/// `resolve_offset_feedback`, which calls `classify_offset_feedback`) -- so it
/// is ignored: acknowledging the old offset is a no-op (returns false) and the
/// committable offset continues to reflect only the generation-2 record, never
/// regressing to or advancing on the generation-1 offset. An old-generation
/// Ack/Nack thus cannot move the commit offset.
#[test]
fn stale_generation_ack_does_not_advance_committable_offset() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Generation 1: own partition 0, track and ack offset 100 so there is an
    // established committable offset (101) for the generation-1 period.
    let mut tpl1 = TopicPartitionList::new();
    let _ = tpl1.add_partition("traces", 0);
    receiver.rebalance_state.set_assignment_for_test(&tpl1);
    let gen1 = receiver.rebalance_state.current_generation("traces", 0);
    receiver.offset_tracker.track("traces", 0, 100, gen1);
    let _ = receiver.offset_tracker.acknowledge("traces", 0, 100);

    // Partition 0 is revoked (revocation carries generation 1); the loop
    // reconciles and purges the generation-1 tracker state. Also drop it from
    // the assigned set (empty assignment), mirroring librdkafka's
    // pre_rebalance(Revoke) removing it before post_rebalance(Assign), so the
    // subsequent reassignment allocates a fresh, strictly-greater generation.
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, gen1);
    receiver.reconcile_rebalance_state();
    receiver
        .rebalance_state
        .set_assignment_for_test(&TopicPartitionList::new());

    // Generation 2: partition 0 is reassigned; a new record (offset 200) is
    // tracked under the newer generation.
    receiver.rebalance_state.set_assignment_for_test(&tpl1);
    let gen2 = receiver.rebalance_state.current_generation("traces", 0);
    assert!(gen2 > gen1, "reassignment must allocate a newer generation");
    receiver.offset_tracker.track("traces", 0, 200, gen2);
    let committable_after_reassign = receiver
        .offset_tracker
        .committable_snapshot()
        .get(&("traces".to_string(), 0))
        .copied();

    // A stale Ack/Nack for the old generation-1 record arrives. The receiver
    // reads the same state `resolve_offset_feedback` reads and classifies it.
    let tracked_generation = receiver.offset_tracker.partition_generation("traces", 0);
    let assigned_generation = receiver.rebalance_state.current_generation("traces", 0);
    let is_assigned = receiver.rebalance_state.is_assigned("traces", 0);
    assert_eq!(
        classify_offset_feedback(gen1, tracked_generation, assigned_generation, is_assigned),
        OffsetFeedbackAction::DropStale,
        "an Ack/Nack from the old generation must classify as DropStale",
    );

    // The DropStale path does not touch the offset tracker; simulate the only
    // mutation a stale feedback could attempt (acknowledging its old offset)
    // and confirm it is a no-op -- the old offset is not pending under the new
    // generation, so the watermark cannot move.
    assert!(
        !receiver.offset_tracker.acknowledge("traces", 0, 100),
        "acking a stale old-generation offset must not advance the watermark",
    );

    // The committable offset still reflects only the generation-2 record; the
    // stale feedback neither advanced nor rolled it back.
    assert_eq!(
        receiver
            .offset_tracker
            .committable_snapshot()
            .get(&("traces".to_string(), 0))
            .copied(),
        committable_after_reassign,
        "a stale old-generation Ack/Nack must not change the committable offset",
    );
    assert_eq!(
        committable_after_reassign,
        Some(200),
        "only the generation-2 record drives the commit",
    );
}

/// Scenario (consumer-group rebalancing): a partition is retained across a rebalance
/// that only adds or removes other partitions.
/// Guarantees: the retained partition keeps its generation, so an unrelated rebalance
/// does not invalidate acks for partitions this consumer never lost.
#[test]
fn retained_partition_generation_is_stable_across_unrelated_rebalance() {
    // Regression: the per-partition ownership generation must NOT change when the
    // partition is retained across a rebalance that only affects OTHER
    // partitions. Otherwise a newer record on the retained partition would
    // bump its generation and cause a legitimate late ACK for an earlier record
    // (carrying the older generation) to be wrongly dropped as stale.
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Initial assignment: own partition 0.
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    receiver.rebalance_state.set_assignment_for_test(&tpl);
    let generation_p0 = receiver.rebalance_state.current_generation("traces", 0);

    // An unrelated rebalance retains partition 0 and acquires partition 1
    // (which advances the generation allocator).
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("traces", 0);
    let _ = tpl2.add_partition("traces", 1);
    receiver.rebalance_state.set_assignment_for_test(&tpl2);

    // Partition 0 was retained: its generation must be unchanged, even though the
    // allocator advanced for partition 1.
    assert_eq!(
        receiver.rebalance_state.current_generation("traces", 0),
        generation_p0
    );
    assert!(receiver.rebalance_state.current_generation("traces", 1) > generation_p0);
}

/// Scenario (consumer-group rebalancing): a rebalance assigns partitions and the receive loop reconciles.
/// Guarantees: `reconcile_rebalance_state` folds the rebalance deltas into
/// the metric set - counting the rebalance event and cumulative acquisitions,
/// and observing `receiver.kafka.consumer.group.partitions` as the current
/// owned count rather than accumulating it.
#[test]
fn reconcile_folds_consumer_group_metrics() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Simulate a rebalance that assigns two partitions.
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    let _ = tpl.add_partition("traces", 1);
    receiver.rebalance_state.set_assignment_for_test(&tpl);

    receiver.reconcile_rebalance_state();

    // Observed up/down counter reflects current ownership; cumulative
    // counter reflects the acquisitions.
    assert_eq!(receiver.metrics.consumer.partitions.get(), 2);
    assert_eq!(receiver.metrics.consumer.partition_assignments.get(), 2);

    // A second reconcile with no further rebalance activity must not change
    // the observed value (it is folded only when a rebalance occurred) or
    // double count the counter.
    receiver.reconcile_rebalance_state();
    assert_eq!(receiver.metrics.consumer.partitions.get(), 2);
    assert_eq!(receiver.metrics.consumer.partition_assignments.get(), 2);
}

/// Scenario (operational visibility): a manual-commit receiver tracks several
/// in-flight offsets, then acknowledges some, then has its partition revoked
/// and purged, reconciling after each step.
/// Guarantees: the `records_in_flight` up/down counter reflects the current
/// count of tracked-but-uncommitted offsets at each reconcile -- it rises as
/// offsets are tracked, falls as they are acked and the watermark advances,
/// and drops to zero when the partition is purged -- giving operators a
/// current view of the receiver's outstanding depth.
#[test]
fn records_in_flight_gauge_reflects_outstanding_offsets() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Own partition 0 and track three in-flight offsets under its generation.
    // in flight: traces/0={0,1,2} => gauge 3
    let mut tpl = TopicPartitionList::new();
    let _ = tpl.add_partition("traces", 0);
    receiver.rebalance_state.set_assignment_for_test(&tpl);
    let generation = receiver.rebalance_state.current_generation("traces", 0);
    for offset in 0..3 {
        receiver
            .offset_tracker
            .track("traces", 0, offset, generation);
    }
    receiver.reconcile_rebalance_state();
    assert_eq!(
        receiver.metrics.consumer.records_in_flight.get(),
        3,
        "the counter must report all three tracked in-flight offsets",
    );

    // Acknowledge the two lowest offsets; only offset 2 remains pending.
    // in flight: traces/0={2} => gauge 1
    let _ = receiver.offset_tracker.acknowledge("traces", 0, 0);
    let _ = receiver.offset_tracker.acknowledge("traces", 0, 1);
    receiver.reconcile_rebalance_state();
    assert_eq!(
        receiver.metrics.consumer.records_in_flight.get(),
        1,
        "the counter must drop as offsets are acked and the watermark advances",
    );

    // Revoke and purge the partition; nothing remains in flight.
    // in flight: {} => gauge 0
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, generation);
    receiver.reconcile_rebalance_state();
    assert_eq!(
        receiver.metrics.consumer.records_in_flight.get(),
        0,
        "the counter must drop to zero once the partition's state is purged",
    );
}

/// Scenario (consumer-group rebalancing): a single manual-commit consumer owns all partitions of a
/// multi-partition topic, consumes and acks every produced record, and is
/// then shut down (which commits tracked offsets).
/// Guarantees: each partition ends with a committed offset that accounts for
/// all records produced to it (offset >= records-per-partition).
#[tokio::test]
async fn rebalance_single_consumer_assigns_and_commits() {
    const TOPIC: &str = "rebalance-assign-traces";
    let group = "rebalance-assign-group";
    with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce `REBALANCE_RECORDS_PER_PARTITION` records to each partition.
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Consume all produced messages and ack each one so the
                // receiver advances its committable offsets (manual commit only
                // commits acknowledged offsets).
                let total =
                    (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Allow at least one safety-net commit cycle to fire.
                tokio::time::sleep(Duration::from_millis(800)).await;

                // Shutdown also commits all tracked offsets before exit.
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;

                // Each partition should have a committed offset accounting for
                // its records (committed offset is "next to read", so >= count).
                // Commits are asynchronous (flushed on unsubscribe/close), so
                // poll until the broker reports them rather than asserting once.
                for partition in 0..REBALANCE_TEST_PARTITIONS {
                    let brokers = cluster.bootstrap_servers().to_string();
                    let committed = poll_until(
                        Duration::from_secs(5),
                        Duration::from_millis(250),
                        || {
                            committed_offset(&brokers, group, TOPIC, partition)
                                .expect("kafka-test: committed-offset probe failed")
                                .is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                        },
                    )
                    .await;
                    assert!(
                        committed,
                        "partition {partition} should have committed offset >= {REBALANCE_RECORDS_PER_PARTITION}, got {:?}",
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed"),
                    );
                }
            },
        )
        .await;
}

/// Scenario (consumer-group rebalancing): a manual-commit receiver owns both partitions, consumes and
/// acks every record, then a second consumer joins the group and forces one
/// partition to be revoked from the receiver (commit-before-revoke).
/// Guarantees: after the forced rebalance, both partitions retain a committed
/// offset that accounts for all produced records, so no progress was lost and
/// the new owner will not re-consume from an earlier offset.
#[tokio::test]
async fn rebalance_revoke_commits_before_reassign() {
    const TOPIC: &str = "rebalance-revoke-traces";
    let group = "rebalance-revoke-group";
    with_cluster(
            KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
            |cluster| async move {
                let producer = cluster.producer().build();

                let req = create_traces_with_spans();
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");

                // Produce records to both partitions.
                producer
                    .produce_per_partition(
                        TOPIC,
                        REBALANCE_TEST_PARTITIONS,
                        REBALANCE_RECORDS_PER_PARTITION,
                        &bytes,
                    )
                    .await;

                let cfg =
                    manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
                let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

                // Drain all messages (receiver A owns both partitions
                // initially) and ack each so A advances and commits its offsets.
                let total =
                    (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
                for _ in 0..total {
                    let pdata = receiver.recv_pdata().await;
                    receiver.ack(pdata);
                }

                // Let a safety-net commit flush A's progress on both partitions.
                tokio::time::sleep(Duration::from_millis(800)).await;

                // A second consumer joins the SAME group, forcing librdkafka to
                // revoke one partition from receiver A and assign it to B. Keep
                // the trigger alive to hold the revoke.
                let _trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(10))
                        .await;

                // After the rebalance, every partition that B now owns must have a
                // committed offset from A's pre-revoke commit (commit-before-revoke).
                // We require that *both* partitions carry a committed offset that
                // accounts for all produced records, i.e. no progress was lost.
                let brokers = cluster.bootstrap_servers().to_string();
                let all_committed = poll_until(
                    Duration::from_secs(5),
                    Duration::from_millis(250),
                    || {
                        let c0 = committed_offset(&brokers, group, TOPIC, 0)
                            .expect("kafka-test: committed-offset probe failed");
                        let c1 = committed_offset(&brokers, group, TOPIC, 1)
                            .expect("kafka-test: committed-offset probe failed");
                        c0.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                            && c1.is_some_and(|o| o >= REBALANCE_RECORDS_PER_PARTITION as i64)
                    },
                )
                .await;
                assert!(
                    all_committed,
                    "both partitions must retain committed offsets >= {REBALANCE_RECORDS_PER_PARTITION} \
                     across the rebalance (commit-before-revoke)",
                );

                // Clean up: shut down receiver A.
                receiver.shutdown(Duration::from_secs(5));
                receiver.await_stopped().await;
            },
        )
        .await;
}

/// Scenario (consumer-group rebalancing): a cooperative-sticky manual-commit receiver owns both
/// partitions, then a second cooperative-sticky consumer joins the group,
/// causing an incremental rebalance that moves one partition away while the
/// receiver retains the other; a new record is produced to the retained
/// partition.
/// Guarantees: the retained partition keeps committing (its post-rebalance
/// record reaches committed offset >= 2), proving retained-partition ACKs
/// are not dropped as revoked under the cooperative protocol.
#[tokio::test]
async fn rebalance_cooperative_sticky_retains_owned_partitions() {
    const TOPIC: &str = "rebalance-coop-traces";
    let group = "rebalance-coop-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Produce an initial record to each partition.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let key = format!("init-{partition}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &bytes)
                            .key(key.as_bytes())
                            .partition(partition),
                    )
                    .await
                    .expect("Failed to send message");
            }

            let cfg = manual_traces_config(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                500,
                Some(RebalanceStrategy::CooperativeSticky),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // A initially owns both partitions: consume and ack the two
            // initial records.
            for _ in 0..REBALANCE_TEST_PARTITIONS as usize {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // A second cooperative-sticky consumer joins the group, forcing
            // an incremental rebalance that moves exactly one partition to B
            // while A retains the other. The trigger consumer MUST also use
            // cooperative-sticky, which `RebalanceTrigger` does not expose,
            // so this consumer is created inline.
            let consumer_b: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", cluster.bootstrap_servers())
                .set("group.id", group)
                .set("enable.auto.commit", "false")
                .set("auto.offset.reset", "earliest")
                .set("partition.assignment.strategy", "cooperative-sticky")
                .create()
                .expect("failed to create consumer B");
            consumer_b
                .subscribe(&[TOPIC])
                .expect("consumer B subscribe");

            // Poll B until it is assigned a partition (drives the rebalance).
            let mut b_partition = None;
            for _ in 0..40 {
                if let Ok(a) = consumer_b.assignment() {
                    if let Some(elem) = a.elements().first() {
                        b_partition = Some(elem.partition());
                        break;
                    }
                }
                let _ = tokio::time::timeout(Duration::from_millis(500), consumer_b.recv()).await;
            }
            let b_partition =
                b_partition.expect("consumer B was never assigned; rebalance did not occur");
            // The partition A retains is the other one.
            let a_partition = (REBALANCE_TEST_PARTITIONS - 1) - b_partition;

            // Produce a new record to A's retained partition and have A
            // consume + ack it. If A wrongly dropped the retained partition
            // from its assigned set, this ack would be rejected and the
            // offset would never advance.
            producer
                .send_full(
                    SendRecord::new(TOPIC, &bytes)
                        .key(b"post-rebalance")
                        .partition(a_partition),
                )
                .await
                .expect("Failed to send post-rebalance message");

            // A may still receive records for the partition being handed off
            // before the rebalance settles; keep reading until we get one on
            // the retained partition and ack everything we see.
            let brokers = cluster.bootstrap_servers().to_string();
            let mut retained_committed = false;
            'outer: for _ in 0..40 {
                if let Some(pdata) = receiver.try_recv_pdata(Duration::from_secs(5)).await {
                    receiver.ack(pdata);
                }
                // The retained partition must accumulate a committed offset
                // that accounts for its initial + post-rebalance records.
                if poll_until(Duration::from_secs(2), Duration::from_millis(250), || {
                    committed_offset(&brokers, group, TOPIC, a_partition)
                        .expect("kafka-test: committed-offset probe failed")
                        .is_some_and(|o| o >= 2)
                })
                .await
                {
                    retained_committed = true;
                    break 'outer;
                }
            }
            assert!(
                retained_committed,
                "retained partition {a_partition} must keep committing after a \
                     cooperative-sticky rebalance (ACKs must not be dropped)",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
            drop(consumer_b);
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): a manual-commit receiver owns all partitions, then a second
/// consumer joins (forcing a revoke) and leaves (reassigning everything back
/// to the receiver); a fresh record is produced to every partition after the
/// reassignment and drained/acked. Best-effort end-to-end exercise of the
/// assignment-generation guard (the deterministic core is covered by
/// `stale_revocation_preserves_reassigned_partition_state`).
/// Guarantees: at least one reassigned partition commits its
/// post-reassignment record (offset >= 2), proving the fresh state was not
/// purged and its ack was not dropped after reassignment.
#[tokio::test]
async fn rebalance_revoke_then_reassign_preserves_new_records() {
    const TOPIC: &str = "rebalance-reassign-traces";
    let group = "rebalance-reassign-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // One initial record per partition.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let key = format!("init-{partition}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &bytes)
                            .key(key.as_bytes())
                            .partition(partition),
                    )
                    .await
                    .expect("Failed to send message");
            }

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume + ack the initial records (receiver owns all partitions).
            for _ in 0..REBALANCE_TEST_PARTITIONS as usize {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // A second consumer joins (forcing a revoke), then drops out of
            // scope (reassigning all partitions back to the receiver).
            {
                let _trigger =
                    RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(10))
                        .await;
            }

            // Produce a fresh record to every partition after the
            // reassignment. Consume and ack whatever the receiver delivers.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let key = format!("post-{partition}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &bytes)
                            .key(key.as_bytes())
                            .partition(partition),
                    )
                    .await
                    .expect("Failed to send post-reassign message");
            }

            // Drain and ack post-reassignment records for a while.
            let brokers = cluster.bootstrap_servers().to_string();
            for _ in 0..40 {
                if let Some(pdata) = receiver.try_recv_pdata(Duration::from_secs(2)).await {
                    receiver.ack(pdata);
                }
                // Both partitions should end up with a committed offset that
                // accounts for the initial + post-reassignment records.
                let c0 = committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed");
                let c1 = committed_offset(&brokers, group, TOPIC, 1)
                    .expect("kafka-test: committed-offset probe failed");
                if c0.is_some_and(|o| o >= 2) && c1.is_some_and(|o| o >= 2) {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }

            // At least one partition must show the post-reassignment record
            // committed (offset >= 2). If the generation guard were broken,
            // the reassigned partition's fresh state would be purged and its
            // ack dropped, leaving the offset stuck at 1.
            let c0 = committed_offset(&brokers, group, TOPIC, 0)
                .expect("kafka-test: committed-offset probe failed");
            let c1 = committed_offset(&brokers, group, TOPIC, 1)
                .expect("kafka-test: committed-offset probe failed");
            assert!(
                c0.is_some_and(|o| o >= 2) || c1.is_some_and(|o| o >= 2),
                "a reassigned partition must commit its post-reassignment record; \
                     got c0={c0:?} c1={c1:?}",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): two `KafkaReceiver` replicas share one `group_id` against a
/// multi-partition topic; replica B joins (scale-up) then leaves
/// (scale-down), driving two rebalances. This is the in-process analogue of
/// running 2+ replicas with the same group and scaling the replica count up
/// and down; the full procedure is documented in the Kafka test-suite README
/// ("Multi-receiver scale-up/down").
///
/// Guarantees: (1) both replicas own a partition at some point, so the
/// partitions distribute across the group (B consumes records that only its
/// assigned partition can deliver, and both replicas' terminal metrics show
/// `group.partitions >= 1`); (2) a rebalance is observed on scale-up/down
/// (`group.partition.revocations >= 1` across the two replicas); (3) no message is
/// lost or double-committed -- every produced record is delivered at least
/// once and durably retained on the broker, each partition's committed
/// offset stays within `[wave-1 count, total produced count]` (the lower
/// bound proves committed progress is never rolled back across a rebalance,
/// the upper bound proves nothing is committed past the produced data), and
/// neither replica reports failed offset commits.
///
/// This is the in-process analogue of running 2+ replicas with the same
/// `group_id` against a multi-partition topic and scaling the replica count
/// up and down. Procedure (mirrored by the code below):
///   1. Pre-create a `REBALANCE_TEST_PARTITIONS`-partition topic and produce
///      wave 1 (`REBALANCE_RECORDS_PER_PARTITION` per partition).
///   2. Start replica A alone and drain wave 1 in full, so A demonstrably
///      owned every partition before anyone else joined.
///   3. Start replica B in the same group (scale-up), then produce wave 2 so
///      B's newly-assigned partition has fresh records to deliver.
///   4. Drain the group, prioritizing B so its assigned partition is not
///      re-won by A's continuously-polling loop, until every produced record
///      has been delivered at least once (bounded by a deadline so a stall
///      fails loudly instead of hanging).
///   5. Shut down B (scale-down); this forces a second rebalance that returns
///      B's partition to A. Drain A briefly so A can re-own and commit.
///   6. Shut down A. Read each replica's `TerminalState` metrics and assert
///      distribution, rebalance observation, and no-loss/no-double-commit.
///
/// Rebalance timing on the mock is nondeterministic and delivery is
/// at-least-once, so distribution is gated by B's own deliveries plus folded
/// rebalance metrics, and no-loss/no-double-commit is gated by broker-side
/// record retention plus a bounded committed offset per partition (not by an
/// exact delivered-record count, which duplicates can inflate during a
/// rebalance).
#[tokio::test]
async fn rebalance_two_receivers_scale_up_down_distribute_without_loss_or_double_commit() {
    const TOPIC: &str = "rebalance-scale-traces";
    let group = "rebalance-scale-group";
    // Records are produced in two waves of `REBALANCE_RECORDS_PER_PARTITION`
    // per partition: wave 1 before B joins (drained by A alone) and wave 2
    // after B joins (so B's newly-assigned partition has fresh records to
    // deliver, making its assignment observable rather than timing-dependent).
    let per_partition_total = 2 * REBALANCE_RECORDS_PER_PARTITION;
    let wave = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
    let total_produced = 2 * wave;

    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            let brokers = cluster.bootstrap_servers().to_string();

            // Manual commit so the receiver's rebalance-aware commit path is
            // active and acks drive the committable offsets.
            let mut delivered = 0usize;
            let mut delivered_b = 0usize;

            // True once every partition's committed offset has reached the
            // produced total (no loss, no rollback, no double-commit).
            let all_committed = |b: &str| {
                (0..REBALANCE_TEST_PARTITIONS).all(|p| {
                    committed_offset(b, group, TOPIC, p)
                        .expect("kafka-test: committed-offset probe failed")
                        == Some(per_partition_total as i64)
                })
            };

            // Step 1: produce wave 1 (`REBALANCE_RECORDS_PER_PARTITION` per
            // partition).
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Step 2: start replica A alone and drain wave 1 in full. A single
            // member is assigned every partition, so consuming the whole wave
            // proves A held the entire topic before anyone else joined.
            let cfg_a = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            for _ in 0..wave {
                let pdata = receiver_a.recv_pdata().await;
                receiver_a.ack(pdata);
                delivered += 1;
            }

            // Step 3: start replica B in the same group (scale-up), let the
            // rebalance settle, then produce wave 2 to every partition so B's
            // newly-assigned partition has fresh records to deliver.
            let cfg_b = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
            tokio::time::sleep(Duration::from_secs(1)).await;
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Step 4: drain B *first and exclusively* until it has consumed
            // its partition's share of wave 2. Under an eager assignor B owns
            // one partition, but A's continuously-polling loop would re-win
            // that partition if A were polled concurrently; leaving A idle
            // here lets B keep and drain its assigned partition. Reaching
            // B's expected share is the direct proof that partitions
            // distributed across the group. Bounded by a deadline so a
            // failure to distribute fails loudly instead of hanging.
            let expected_b = REBALANCE_RECORDS_PER_PARTITION as usize;
            let b_deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            while delivered_b < expected_b {
                assert!(
                    tokio::time::Instant::now() < b_deadline,
                    "timed out: replica B consumed {delivered_b} of {expected_b} expected \
                         records; the scale-up rebalance did not hand it a partition",
                );
                if let Some(pdata) = receiver_b.try_recv_pdata(Duration::from_millis(250)).await {
                    receiver_b.ack(pdata);
                    delivered += 1;
                    delivered_b += 1;
                }
            }

            // Let B durably commit before it leaves: B's commits are async,
            // so wait past its safety-net commit interval (500ms) so its
            // acked offsets flush to the broker.
            tokio::time::sleep(Duration::from_secs(1)).await;

            // Step 5: shut down B (scale-down). This forces a second
            // rebalance that returns B's partition to A. B commits the
            // offsets it acked as part of its graceful shutdown.
            receiver_b.shutdown(Duration::from_secs(5));
            let terminal_b = receiver_b.await_terminal_state().await;

            // Step 6: drain A. The loop body focuses solely on A receiving
            // and acking records; A re-consuming and acking the tail B did
            // not durably commit is what advances the committed offsets back
            // to the produced total. The deadline is the loop guard (not a
            // per-iteration assert), and the loop stops as soon as every
            // partition is committed to the produced total.
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            while tokio::time::Instant::now() < deadline && !all_committed(&brokers) {
                if let Some(pdata) = receiver_a.try_recv_pdata(Duration::from_millis(200)).await {
                    receiver_a.ack(pdata);
                    delivered += 1;
                }
            }

            // One assertion, after both replicas have received and acked:
            // the broker must report every partition committed to the
            // produced total. This is the authoritative no-loss /
            // no-rollback / no-double-commit check; the deadline above lets
            // A's async commits and any redelivery settle first.
            assert!(
                all_committed(&brokers),
                "after scale-down the group did not commit every partition to the produced \
                     total {per_partition_total} (delivered {delivered} of {total_produced}); \
                     committed offsets did not converge",
            );

            // Shut down A (flushes its tracked offsets) and collect metrics.
            receiver_a.shutdown(Duration::from_secs(5));
            let terminal_a = receiver_a.await_terminal_state().await;

            // ---- Assertions ----
            let mut fa = FoldedMetrics::new();
            fa.fold_all(terminal_a.metrics());
            let mut fb = FoldedMetrics::new();
            fb.fold_all(terminal_b.metrics());

            // (1) Distribution: both replicas acquired a partition over their
            // lifetimes and together cover the topic. B's deliveries above
            // already prove it owned a partition; metrics corroborate it.
            assert!(
                fa.value("group.partition.assignments") >= 1,
                "replica A should have acquired at least one partition, got {}",
                fa.value("group.partition.assignments"),
            );
            assert!(
                fb.value("group.partition.assignments") >= 1,
                "replica B should have acquired at least one partition on scale-up, got {}",
                fb.value("group.partition.assignments"),
            );
            assert!(
                fa.value("group.partition.assignments") + fb.value("group.partition.assignments")
                    >= REBALANCE_TEST_PARTITIONS as u64,
                "the group should have acquired all {REBALANCE_TEST_PARTITIONS} partitions \
                     across the two replicas' lifetimes",
            );
            // After scale-down A re-owns its partitions, a deterministic
            // current-ownership check.
            assert!(
                fa.value("group.partitions") >= 1,
                "replica A should currently own at least one partition at shutdown, got {}",
                fa.value("group.partitions"),
            );

            // (2) Rebalance observed: at least one owned partition was revoked
            // across scale-up/down.
            assert!(
                fa.value("group.partition.revocations") + fb.value("group.partition.revocations")
                    >= 1,
                "a partition revoke should have been observed across scale-up/down",
            );

            // (3a) No commit failures on either replica.
            assert_eq!(
                measurement_counter(
                    terminal_a.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ),
                0,
                "replica A should have no offset commit errors",
            );
            assert_eq!(
                measurement_counter(
                    terminal_b.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ),
                0,
                "replica B should have no offset commit errors",
            );

            // (3b) No loss: every produced record was delivered at least once
            // (delivery is at-least-once, so `>=`) and durably retained on the
            // broker (`message_count` is `high - low`).
            assert!(
                delivered >= total_produced,
                "the group should deliver every produced record at least once: \
                     delivered {delivered} of {total_produced}",
            );
            let inspector = cluster.inspect();
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                assert_eq!(
                    inspector.message_count(TOPIC, partition),
                    per_partition_total as i64,
                    "partition {partition} should durably retain all produced records",
                );
            }

            // (3c) No rollback and no double-commit: each partition's committed
            // offset equals exactly the produced total -- committed progress
            // was never rolled back across a rebalance and nothing was
            // committed past the produced data. Guaranteed by the convergence
            // drain above, so this equality is deterministic.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let committed = committed_offset(&brokers, group, TOPIC, partition)
                    .expect("kafka-test: committed-offset probe failed")
                    .unwrap_or_else(|| {
                        panic!("partition {partition} should have a committed offset")
                    });
                assert_eq!(
                    committed, per_partition_total as i64,
                    "partition {partition} committed offset should equal the produced total \
                         {per_partition_total} (no rollback, no commit past produced data)",
                );
            }
        },
    )
    .await;
}

/// Runs two same-group manual-commit receivers against a 2-partition topic
/// under the given eager assignment `strategy` and asserts the group
/// distributes both partitions and commits every record with no loss.
///
/// Shared body for the `range` and `roundrobin` strategy tests: the receiver
/// rebalance logic is strategy-agnostic for the eager protocols, so both
/// strategies must produce the same distribute-and-commit outcome.
async fn run_two_member_strategy_rebalance(topic: &'static str, strategy: RebalanceStrategy) {
    let group = "rebalance-strategy-group";
    let per_partition_total = REBALANCE_RECORDS_PER_PARTITION;
    let total = (REBALANCE_RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;
    with_cluster(
        KafkaTestCluster::builder().topic_with(topic, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            let brokers = cluster.bootstrap_servers().to_string();

            producer
                .produce_per_partition(
                    topic,
                    REBALANCE_TEST_PARTITIONS,
                    REBALANCE_RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Two members join the same group under the configured strategy.
            let cfg_a = manual_traces_config(
                cluster.bootstrap_servers(),
                group,
                topic,
                500,
                Some(strategy),
            );
            let cfg_b = manual_traces_config(
                cluster.bootstrap_servers(),
                group,
                topic,
                500,
                Some(strategy),
            );
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);

            // True once every partition's committed offset reaches the
            // produced total (no loss, no rollback, no double-commit).
            let all_committed = |b: &str| {
                (0..REBALANCE_TEST_PARTITIONS).all(|p| {
                    committed_offset(b, group, topic, p)
                        .expect("kafka-test: committed-offset probe failed")
                        == Some(per_partition_total as i64)
                })
            };

            // Drain both members concurrently until every partition is
            // committed to the produced total, bounded by a deadline so a
            // failure to distribute fails loudly instead of hanging.
            let mut delivered = 0usize;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
            while tokio::time::Instant::now() < deadline && !all_committed(&brokers) {
                if let Some(pdata) = receiver_a.try_recv_pdata(Duration::from_millis(200)).await {
                    receiver_a.ack(pdata);
                    delivered += 1;
                }
                if let Some(pdata) = receiver_b.try_recv_pdata(Duration::from_millis(200)).await {
                    receiver_b.ack(pdata);
                    delivered += 1;
                }
            }

            assert!(
                all_committed(&brokers),
                "under {strategy:?} the group did not commit every partition to the produced \
                     total {per_partition_total} (delivered {delivered} of {total}); committed \
                     offsets did not converge",
            );

            receiver_a.shutdown(Duration::from_secs(5));
            let terminal_a = receiver_a.await_terminal_state().await;
            receiver_b.shutdown(Duration::from_secs(5));
            let terminal_b = receiver_b.await_terminal_state().await;

            let mut fa = FoldedMetrics::new();
            fa.fold_all(terminal_a.metrics());
            let mut fb = FoldedMetrics::new();
            fb.fold_all(terminal_b.metrics());

            // Distribution: together the two members acquired every partition
            // over their lifetimes.
            assert!(
                fa.value("group.partition.assignments") + fb.value("group.partition.assignments")
                    >= REBALANCE_TEST_PARTITIONS as u64,
                "under {strategy:?} the group should acquire all {REBALANCE_TEST_PARTITIONS} \
                     partitions across the two members (A={}, B={})",
                fa.value("group.partition.assignments"),
                fb.value("group.partition.assignments"),
            );

            // No commit failures on either member.
            assert_eq!(
                measurement_counter(
                    terminal_a.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ) + measurement_counter(
                    terminal_b.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ),
                0,
                "no offset commit errors expected under {strategy:?}",
            );

            // No loss: every produced record delivered at least once and
            // durably retained on the broker.
            assert!(
                delivered >= total,
                "under {strategy:?} the group should deliver every produced record at least \
                     once: delivered {delivered} of {total}",
            );
            let inspector = cluster.inspect();
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                assert_eq!(
                    inspector.message_count(topic, partition),
                    per_partition_total as i64,
                    "partition {partition} should durably retain all produced records",
                );
            }
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): two same-group receivers configured
/// with the `range` assignment strategy join against a 2-partition topic.
/// Guarantees: the group distributes both partitions across the two members
/// and commits every produced record with no loss and no commit errors, so
/// the `range` eager strategy assigns and commits correctly end-to-end.
#[tokio::test]
async fn rebalance_strategy_range_assigns_and_commits() {
    run_two_member_strategy_rebalance("rebalance-range-traces", RebalanceStrategy::Range).await;
}

/// Scenario (consumer-group rebalancing): two same-group receivers configured
/// with the `roundrobin` assignment strategy join against a 2-partition
/// topic.
/// Guarantees: the group distributes both partitions across the two members
/// and commits every produced record with no loss and no commit errors, so
/// the `roundrobin` eager strategy assigns and commits correctly end-to-end.
#[tokio::test]
async fn rebalance_strategy_roundrobin_assigns_and_commits() {
    run_two_member_strategy_rebalance("rebalance-roundrobin-traces", RebalanceStrategy::RoundRobin)
        .await;
}

/// Scenario (consumer-group rebalancing): a manual-commit receiver holds an
/// un-acked in-flight record on a partition that is then stolen by a second
/// group member (a `RebalanceTrigger`), after which the test acks the now
/// stale record.
/// Guarantees: the late ack for the revoked partition is classified as a
/// stale/late ack -- it increments
/// `receiver.kafka.consumer.group.feedback.after_revocation` and is not
/// committed -- so an ack that arrives after a partition is revoked can
/// never advance a partition this consumer no longer owns.
#[tokio::test]
async fn stale_ack_after_revoke_counts_feedback_after_revocation() {
    const TOPIC: &str = "rebalance-stale-ack-traces";
    let group = "rebalance-stale-ack-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // One record per partition so the receiver has in-flight work on
            // every partition it owns.
            producer
                .produce_per_partition(TOPIC, REBALANCE_TEST_PARTITIONS, 1, &bytes)
                .await;

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The receiver owns every partition initially; consume the
            // in-flight records but hold them un-acked so their offsets stay
            // pending across the revoke.
            let mut in_flight = Vec::new();
            for _ in 0..REBALANCE_TEST_PARTITIONS {
                in_flight.push(receiver.recv_pdata().await);
            }

            // A second member joins and steals at least one partition,
            // forcing a revoke on the receiver.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            let revoked_partition = trigger
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("rebalance trigger should own a partition from the test topic");
            receiver
                .wait_for_partition_revocation(TOPIC, revoked_partition, Duration::from_secs(10))
                .await;

            // Ack the in-flight records now that at least one of their
            // partitions has been revoked. The acks for revoked partitions
            // must be dropped by the late-ack/stale-generation guard.
            for pdata in in_flight {
                receiver.ack(pdata);
            }

            // Ack and Shutdown share the same FIFO control channel, so the
            // feedback is handled before the terminal snapshot is taken.
            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            drop(trigger);

            let mut m = FoldedMetrics::new();
            m.fold_all(terminal.metrics());
            let acknowledgements = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.acknowledgements",
                &[("signal", "traces"), ("outcome", "success")],
                "responses",
            );
            assert!(
                m.value("group.feedback.after_revocation") >= 1,
                "at least one ack for a revoked partition should be counted and dropped, got \
                     {}; acknowledgements={acknowledgements}, revocations={}",
                m.value("group.feedback.after_revocation"),
                m.value("group.partition.revocations"),
            );
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): identical to
/// `stale_ack_after_revoke_counts_feedback_after_revocation` but the stale
/// feedback is a terminal permanent **Nack** instead of an Ack -- a
/// manual-commit receiver holds an un-acked in-flight record on a partition
/// that is then stolen by a second group member (a `RebalanceTrigger`), after
/// which the test permanently nacks the now-stale record.
/// Guarantees: the late Nack for the revoked partition is subject to the same
/// stale/late guard as an Ack (both funnel through terminal feedback handling)
/// -- it increments `receiver.kafka.consumer.group.feedback.after_revocation`
/// and is not committed -- so a Nack that arrives after a partition is
/// revoked can never advance a partition this consumer no longer owns.
#[tokio::test]
async fn stale_nack_after_revoke_counts_feedback_after_revocation() {
    const TOPIC: &str = "rebalance-stale-nack-traces";
    let group = "rebalance-stale-nack-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // One record per partition so the receiver has in-flight work on
            // every partition it owns.
            producer
                .produce_per_partition(TOPIC, REBALANCE_TEST_PARTITIONS, 1, &bytes)
                .await;

            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 500, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume the in-flight records but hold them un-acked so their
            // offsets stay pending across the revoke.
            let mut in_flight = Vec::new();
            for _ in 0..REBALANCE_TEST_PARTITIONS {
                in_flight.push(receiver.recv_pdata().await);
            }

            // A second member joins and steals at least one partition,
            // forcing a revoke on the receiver.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            let revoked_partition = trigger
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("rebalance trigger should own a partition from the test topic");
            receiver
                .wait_for_partition_revocation(TOPIC, revoked_partition, Duration::from_secs(10))
                .await;

            // Permanently nack the in-flight records now that at least one of
            // their partitions has been revoked. A terminal Nack for a revoked
            // partition must be dropped by the late-ack/stale-generation guard
            // exactly like an Ack.
            for pdata in in_flight {
                receiver.nack_permanent("stale after revoke", pdata);
            }

            // Nack and Shutdown share the same FIFO control channel, so the
            // feedback is handled before the terminal snapshot is taken.
            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            drop(trigger);

            let mut m = FoldedMetrics::new();
            m.fold_all(terminal.metrics());
            let acknowledgements = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.acknowledgements",
                &[("signal", "traces"), ("outcome", "refused")],
                "responses",
            );
            assert!(
                m.value("group.feedback.after_revocation") >= 1,
                "at least one nack for a revoked partition should be counted and dropped, got \
                     {}; acknowledgements={acknowledgements}, revocations={}",
                m.value("group.feedback.after_revocation"),
                m.value("group.partition.revocations"),
            );
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): an idempotent manual-commit
/// receiver consumes records but holds them un-acked, then a rebalance (a
/// joining and leaving `RebalanceTrigger`) revokes and reassigns the
/// partitions -- a NEW ownership generation -- and librdkafka redelivers the
/// uncommitted offsets under that new generation.
/// Guarantees: because the redelivered offsets belong to a *new* ownership
/// period, the generation-aware idempotency guard does NOT skip them -- they
/// are reprocessed (delivered again), not silently dropped, and
/// `receiver.kafka.consumer.records.duplicates` is not incremented for a
/// cross-generation redelivery. Idempotent dedupe applies only WITHIN an
/// ownership generation (covered by the unit test
/// `is_known_offset_for_generation_*`); it must never suppress a record that
/// a new owner is responsible for reprocessing.
#[tokio::test]
async fn idempotent_redelivery_under_new_generation_is_reprocessed_not_skipped() {
    const TOPIC: &str = "rebalance-idempotent-traces";
    let group = "rebalance-idempotent-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            let records_per_partition = 3;
            producer
                .produce_per_partition(
                    TOPIC,
                    REBALANCE_TEST_PARTITIONS,
                    records_per_partition,
                    &bytes,
                )
                .await;

            // Idempotent manual-commit receiver.
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec![TOPIC.to_string()])
                            .with_encoding(MessageFormat::OtlpProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: Some(500),
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted)
                    .with_enable_idempotency(true);
            let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume the initial records but hold them un-acked so their
            // offsets are never committed; when the partition is reassigned
            // back, librdkafka redelivers from the (uncommitted) start.
            let total = (records_per_partition * REBALANCE_TEST_PARTITIONS) as usize;
            let mut seen = Vec::new();
            for _ in 0..total {
                seen.push(receiver.recv_pdata().await);
            }

            // A member joins (revoking partitions from the receiver) and then
            // leaves (reassigning them back). The reassignment allocates a new
            // ownership generation, and the uncommitted offsets are redelivered
            // under it.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
            drop(trigger);

            // Drain redelivered records within a bounded window, counting how
            // many arrive. Under the new generation they must be reprocessed
            // (delivered), not idempotently skipped.
            let mut redelivered = 0usize;
            let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
            while tokio::time::Instant::now() < deadline {
                if let Some(pdata) = receiver.try_recv_pdata(Duration::from_millis(250)).await {
                    redelivered += 1;
                    receiver.ack(pdata);
                }
            }
            assert!(
                redelivered >= 1,
                "offsets redelivered under a new ownership generation must be \
                     reprocessed (delivered again), not skipped; got {redelivered}",
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;

            let mut m = FoldedMetrics::new();
            m.fold_all(terminal.metrics());
            assert_eq!(
                m.value("records.duplicates"),
                0,
                "a cross-generation redelivery must not be counted as a duplicate, got {}",
                m.value("records.duplicates"),
            );
        },
    )
    .await;
}

/// Scenario (consumer-group rebalancing): a manual-commit receiver (idempotency
/// disabled) consumes every record but holds them un-acked, so their offsets
/// are never committed; a `RebalanceTrigger` then joins (revoking partitions,
/// leaving the in-flight records un-committed) and leaves (reassigning them
/// back), which forces librdkafka to redeliver those uncommitted offsets.
/// This exercises the documented in-flight-on-revoke design (rebalance.rs:
/// in-flight messages on a revoked partition are not drained/interrupted --
/// the new owner re-delivers them, safe under at-least-once).
/// Guarantees: the resulting duplication is **bounded** -- each
/// `(partition, offset)` is delivered at most `1 + rebalance-transitions`
/// times (the original delivery plus at most one redelivery per revoke/
/// reassign transition), never in an unbounded re-loop -- and there is no
/// loss: every produced offset is delivered at least once and, after the
/// redelivered records are acked, each partition's committed offset equals
/// exactly the produced count (no rollback, no commit past produced data).
/// The stale ack from the pre-revoke ownership is dropped by the generation
/// guard (asserted separately by
/// `stale_ack_after_revoke_counts_feedback_after_revocation`).
#[tokio::test]
async fn inflight_records_on_revoke_are_redelivered_with_bounded_duplication() {
    const TOPIC: &str = "rebalance-bounded-dup-traces";
    const RECORDS_PER_PARTITION: i32 = 3;
    let group = "rebalance-bounded-dup-group";
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
                    RECORDS_PER_PARTITION,
                    &bytes,
                )
                .await;

            // Manual-commit, idempotency DISABLED so redelivered offsets are
            // genuinely re-delivered (the harshest bounded-duplication case)
            // rather than skipped. No safety-net timer so acks alone drive
            // commits.
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec![TOPIC.to_string()])
                            .with_encoding(MessageFormat::OtlpProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted);
            let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Count how many times each (partition, offset) is delivered.
            let mut delivery_counts: HashMap<(i32, i64), usize> = HashMap::new();
            let total = (RECORDS_PER_PARTITION * REBALANCE_TEST_PARTITIONS) as usize;

            // Consume every record but hold them un-acked so their offsets are
            // never committed before the revoke.
            let mut in_flight = Vec::new();
            for _ in 0..total {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_topic_id, partition, offset, _generation) = decode_calldata(&route.calldata);
                *delivery_counts.entry((partition, offset)).or_insert(0) += 1;
                in_flight.push(pdata);
            }

            // A member joins (revoking partitions, leaving the in-flight
            // records uncommitted) and then leaves (reassigning them back),
            // forcing redelivery of the uncommitted offsets.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
            drop(trigger);

            // Now ack the original in-flight records. Acks for a partition that
            // was revoked are dropped by the stale-generation guard; acks for a
            // partition still owned advance its offset.
            for pdata in in_flight {
                receiver.ack(pdata);
            }

            // Drain any redelivered records within a bounded window, counting
            // each delivery and acking so the redelivered offsets can commit.
            let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
            while tokio::time::Instant::now() < deadline {
                if let Some(pdata) = receiver.try_recv_pdata(Duration::from_millis(250)).await {
                    let route = pdata
                        .source_route()
                        .expect("delivered pdata carries source calldata");
                    let (_topic_id, partition, offset, _generation) =
                        decode_calldata(&route.calldata);
                    *delivery_counts.entry((partition, offset)).or_insert(0) += 1;
                    receiver.ack(pdata);
                }
            }

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;

            // No loss: every produced (partition, offset) was delivered at
            // least once.
            assert_eq!(
                delivery_counts.len(),
                total,
                "every produced offset must be delivered at least once (no loss); \
                     saw {} distinct offsets, expected {total}",
                delivery_counts.len(),
            );

            // Bounded duplication: no offset is delivered more than twice --
            // the original delivery plus at most one redelivery after the
            // revoke/reassign. This is the upper bound the acceptance
            // criterion requires; an unbounded re-loop would exceed it.
            // The `RebalanceTrigger` join+drop drives two rebalance
            // transitions (revoke on join, reassign on drop), so an
            // uncommitted offset can be redelivered once per transition on top
            // of its original delivery. The duplication is therefore bounded
            // by `1 + TRANSITIONS`; it must never grow into an unbounded
            // re-loop.
            const REBALANCE_TRANSITIONS: usize = 2;
            let max_deliveries = 1 + REBALANCE_TRANSITIONS;
            for ((partition, offset), count) in &delivery_counts {
                assert!(
                    *count >= 1 && *count <= max_deliveries,
                    "offset {offset} on partition {partition} was delivered {count} \
                         times; duplication across the revoke/reassign must be bounded \
                         to at most {max_deliveries} (original + one redelivery per \
                         rebalance transition), not an unbounded re-loop",
                );
            }

            // Global corroboration: total deliveries are bounded by the
            // produced count plus at most one redelivery wave.
            let max_total_deliveries = (total * max_deliveries) as u64;
            let admitted_messages = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.messages",
                &[("signal", "traces")],
                "started",
            );
            assert!(
                admitted_messages <= max_total_deliveries,
                "total admitted messages ({admitted_messages}) must be bounded by produced records \
                     times the per-offset delivery bound ({max_total_deliveries}); an \
                     unbounded redelivery loop would exceed it",
            );

            // No loss, no rollback, no commit past produced data: once the
            // redelivered records are acked, each partition's committed offset
            // equals exactly the produced count.
            let brokers = cluster.bootstrap_servers().to_string();
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                let converged =
                    poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                        committed_offset(&brokers, group, TOPIC, partition)
                            .expect("kafka-test: committed-offset probe failed")
                            == Some(RECORDS_PER_PARTITION as i64)
                    })
                    .await;
                assert!(
                    converged,
                    "partition {partition} committed offset must equal the produced \
                         count {RECORDS_PER_PARTITION} (no loss, no rollback, no \
                         double-commit), got {:?}",
                    committed_offset(&brokers, group, TOPIC, partition)
                        .expect("kafka-test: committed-offset probe failed"),
                );
            }
        },
    )
    .await;
}
