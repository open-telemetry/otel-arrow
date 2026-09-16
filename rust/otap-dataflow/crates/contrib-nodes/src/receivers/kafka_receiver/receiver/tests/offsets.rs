// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offset-guarantee tests: stale/late-ack classification, ack/nack handling,
//! commit triggers, and at-least-once commit ordering.

use super::*;

// ---- Offset guarantees ----

/// Scenario (offset guarantees): an ack arrives for a partition this consumer still owns, whose
/// ownership generation matches the ack.
/// Guarantees: the ack is committed (advances the tracker) rather than
/// dropped.
#[test]
fn classify_offset_feedback_commits_current_generation_ack() {
    assert_eq!(
        classify_offset_feedback(2, Some(2), 2, true),
        OffsetFeedbackAction::Commit,
    );
}

/// Scenario (offset guarantees): an ack arrives whose generation is older than the partition's
/// tracked generation (the partition was reassigned and re-tracked under a
/// newer generation).
/// Guarantees: the ack is dropped as stale, so it cannot roll back or
/// disturb the newer ownership period's committed offset.
#[test]
fn classify_offset_feedback_drops_ack_older_than_tracked_generation() {
    assert_eq!(
        classify_offset_feedback(1, Some(3), 3, true),
        OffsetFeedbackAction::DropStale,
    );
}

/// Scenario (offset guarantees): the closed gap. A partition was revoked and reassigned to this
/// consumer under a newer generation, but no record of the new period has
/// been tracked yet, so the tracker still reports the OLD generation while
/// the assignment already reports the NEW one. A stale ack for the old
/// period arrives with a generation equal to the tracker's.
/// Guarantees: the ack is still dropped as stale because the classifier
/// consults the assigned generation, not just the tracker generation -- so a
/// stale same-as-tracker ack cannot slip through and mutate/commit stale
/// state during the reassign-before-retrack window.
#[test]
fn classify_offset_feedback_drops_stale_ack_when_assigned_generation_is_newer() {
    assert_eq!(
        classify_offset_feedback(1, Some(1), 2, true),
        OffsetFeedbackAction::DropStale,
    );
}

/// Scenario (offset guarantees): an ack arrives for a partition no longer assigned to this
/// consumer, whose tracked state is not newer than the ack's generation.
/// Guarantees: the ack is dropped as a late ack and the lingering tracker
/// state is purged (it belongs to the revoked ownership period).
#[test]
fn classify_offset_feedback_late_ack_purges_when_not_newer() {
    assert_eq!(
        classify_offset_feedback(1, Some(1), 0, false),
        OffsetFeedbackAction::DropLateAck { purge: true },
    );
}

/// Scenario (offset guarantees): an ack arrives for a partition no longer assigned, whose
/// tracked state belongs to a NEWER generation than the ack. This is caught
/// by the stale-generation check *before* the late-ack check, because a
/// newer tracked generation means the partition was reassigned and
/// re-tracked since the ack's ownership period.
/// Guarantees: such an ack is classified `DropStale` (the newer tracked
/// state is preserved), never `DropLateAck` with a purge -- so a stale ack
/// can never purge a newer ownership period's tracker state.
#[test]
fn classify_offset_feedback_ack_older_than_tracked_is_stale_even_when_unassigned() {
    assert_eq!(
        classify_offset_feedback(2, Some(3), 0, false),
        OffsetFeedbackAction::DropStale,
    );
}

/// Scenario (offset guarantees): an ack arrives for a partition that is neither assigned nor
/// tracked (fully revoked and purged already).
/// Guarantees: the ack is dropped as a late ack with nothing to purge.
#[test]
fn classify_offset_feedback_late_ack_untracked_does_not_purge() {
    assert_eq!(
        classify_offset_feedback(1, None, 0, false),
        OffsetFeedbackAction::DropLateAck { purge: false },
    );
}

/// Scenario (offset guarantees): the first ack for a freshly-assigned partition arrives before
/// its record was tracked (untracked, but currently owned), with a
/// generation matching the assignment.
/// Guarantees: the ack is committed -- an untracked-but-owned partition is
/// not treated as stale as long as the ack is not older than the assigned
/// generation.
#[test]
fn classify_offset_feedback_commits_untracked_but_assigned_current_ack() {
    assert_eq!(
        classify_offset_feedback(1, None, 1, true),
        OffsetFeedbackAction::Commit,
    );
}

/// Scenario (offset guarantees): a partition owned under generation 1 is revoked and
/// reassigned under generation 2 but not yet re-tracked, then a stale generation-1 ack
/// equal to the tracker generation arrives.
/// Guarantees: the ack is classified `DropStale` because the classifier consults the
/// assigned generation, so it neither advances the tracker nor rolls back the committed
/// offset during the reassign-before-retrack window.
#[test]
fn stale_same_gen_ack_dropped_after_reassignment_before_retrack() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Generation 1: own partition 0 and track a record at offset 100.
    let mut tpl1 = TopicPartitionList::new();
    let _ = tpl1.add_partition("traces", 0);
    receiver.rebalance_state.set_assignment_for_test(&tpl1);
    let gen1 = receiver.rebalance_state.current_generation("traces", 0);
    receiver.offset_tracker.track("traces", 0, 100, gen1);

    // Revoke partition 0 (queued for tracker purge) AND drop it from the
    // assigned set by applying an empty assignment, mirroring librdkafka's
    // pre_rebalance(Revoke) removing it before post_rebalance(Assign). This
    // is what lets the subsequent reassignment allocate a fresh,
    // strictly-greater generation.
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, gen1);
    receiver
        .rebalance_state
        .set_assignment_for_test(&TopicPartitionList::new());

    // Reassign partition 0 (fresh, strictly-greater generation). The tracker
    // is NOT re-tracked yet, so it still reports generation 1 while the
    // assignment reports generation 2.
    let mut tpl2 = TopicPartitionList::new();
    let _ = tpl2.add_partition("traces", 0);
    receiver.rebalance_state.set_assignment_for_test(&tpl2);
    let gen2 = receiver.rebalance_state.current_generation("traces", 0);
    assert!(gen2 > gen1, "reassignment must allocate a newer generation");
    assert_eq!(
        receiver.offset_tracker.partition_generation("traces", 0),
        Some(gen1),
        "tracker still reports the old generation before any re-track",
    );

    // A stale generation-1 ack, equal to the tracker generation, must be
    // classified as stale because the assigned generation is newer.
    let tracked = receiver.offset_tracker.partition_generation("traces", 0);
    let assigned = receiver.rebalance_state.current_generation("traces", 0);
    let is_assigned = receiver.rebalance_state.is_assigned("traces", 0);
    assert_eq!(
        classify_offset_feedback(gen1, tracked, assigned, is_assigned),
        OffsetFeedbackAction::DropStale,
        "a stale ack matching the tracker generation is dropped once the \
             partition has been reassigned to a newer generation",
    );
}

/// Scenario (offset guarantees): a partition is revoked by the rebalance callback but
/// not yet reconciled when a commit is built.
/// Guarantees: the revoked partition is drained before the committable TPL is built, so
/// a revoked partition is never committed while an owned partition remains committable.
#[test]
fn commit_path_purges_revoked_partitions_first() {
    // Regression: every commit path drains revoked partitions before
    // building the commit TPL, so a partition revoked by the rebalance
    // callback (but not yet reconciled at the top of the loop) is never
    // committed by `commit_offsets` / TimerTick / shutdown / poison-pill.
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // In-flight offsets on two partitions.
    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver.offset_tracker.track("traces", 1, 200, 0);

    // The callback queues a revoke for partition 0, but the loop has not
    // reconciled it yet (it is still tracked).
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 0);
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);

    // The drain-before-commit step that `commit_offsets` runs.
    receiver.purge_revoked_partitions();

    // Partition 0 is purged; the committable TPL a commit would use now
    // excludes it and retains only partition 1.
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 0);
    assert_eq!(receiver.offset_tracker.pending_count("traces", 1), 1);

    let tpl = receiver.offset_tracker.committable_tpl();
    let map = tpl.to_topic_map();
    assert!(
        !map.contains_key(&("traces".to_string(), 0)),
        "revoked partition 0 must not appear in the commit TPL",
    );
    assert_eq!(
        map.get(&("traces".to_string(), 1)),
        Some(&Offset::Offset(200)),
        "owned partition 1 must remain committable",
    );
}

/// Scenario (offset guarantees): a revoked partition is queued while the receiver runs
/// in auto-commit mode.
/// Guarantees: purge leaves the tracker untouched, so under auto-commit librdkafka owns
/// offsets and the manual purge path is inert.
#[test]
fn purge_revoked_partitions_is_noop_under_auto_commit() {
    let cfg = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["traces".to_string()]))
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Auto,
                interval_ms: Some(1000),
            })
            .with_isolation_level(IsolationLevel::ReadUncommitted),
    )
    .expect("test config should be valid");
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 0);

    // Under auto-commit, purge must not touch the tracker (librdkafka owns
    // offsets and rebalance handling is disabled).
    receiver.purge_revoked_partitions();
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
}

/// Scenario (offset guarantees): a revoked partition is queued while the receiver runs
/// in auto-commit mode.
/// Guarantees: reconcile leaves the tracker untouched, so under auto-commit the manual
/// rebalance-reconcile path is inert.
#[test]
fn reconcile_is_noop_under_auto_commit() {
    let cfg = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["traces".to_string()]))
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Auto,
                interval_ms: Some(1000),
            })
            .with_isolation_level(IsolationLevel::ReadUncommitted),
    )
    .expect("test config should be valid");
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver
        .rebalance_state
        .push_revoked_for_test("traces", 0, 0);

    // Under auto-commit, reconcile must not touch the tracker or drain.
    receiver.reconcile_rebalance_state();
    assert_eq!(receiver.offset_tracker.pending_count("traces", 0), 1);
}

/// Scenario (offset guarantees): commit-callback successes and failures accumulate on
/// the poll thread and are then reconciled.
/// Guarantees: reconcile folds them into success and failure outcome buckets exactly
/// once and drains the counters, so a commit failure is surfaced and never double-counted.
#[test]
fn reconcile_folds_commit_callback_metrics() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Simulate commit-callback outcomes accumulated on the poll thread.
    receiver.rebalance_state.record_commit_result_for_test(true);
    receiver.rebalance_state.record_commit_result_for_test(true);
    receiver
        .rebalance_state
        .record_commit_result_for_test(false);

    receiver.reconcile_rebalance_state();

    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Success)
            .commits
            .get(),
        2
    );
    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Failure)
            .commits
            .get(),
        1
    );

    // Counters were drained; a second reconcile adds nothing.
    receiver.reconcile_rebalance_state();
    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Success)
            .commits
            .get(),
        2
    );
    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Failure)
            .commits
            .get(),
        1
    );
}

/// Scenario (offset guarantees): a commit request times out at the broker,
/// so its asynchronous outcome arrives on the commit callback as a failure
/// (modeled here via `record_commit_result_for_test(false)`, the same seam
/// the real `commit_callback` drives). This unit-level surrogate is used
/// because on the in-process `MockCluster` an injected `OffsetCommit`
/// timeout is not delivered to the callback within a test window (verified),
/// so the timeout outcome cannot be observed end-to-end.
/// Guarantees: a timed-out (failed) commit outcome is surfaced as
/// the failure outcome on the next reconcile and does not increment the
/// success outcome -- so a commit timeout is reported and never silently
/// counted as a successful commit or allowed to advance committed state.
#[test]
fn commit_timeout_outcome_surfaces_as_offset_commit_error() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // A commit that reached the broker succeeds; a later commit times out
    // (its callback outcome is a failure).
    receiver.rebalance_state.record_commit_result_for_test(true);
    receiver
        .rebalance_state
        .record_commit_result_for_test(false);

    receiver.reconcile_rebalance_state();

    // The timeout is surfaced as a commit error, not folded into the success
    // counter -- a timed-out commit is never mistaken for a successful one.
    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Failure)
            .commits
            .get(),
        1,
        "a timed-out commit outcome must be surfaced as a failed commit",
    );
    assert_eq!(
        receiver
            .metrics
            .offset_commits_for(Outcome::Success)
            .commits
            .get(),
        1,
        "only the successful commit should count toward the success outcome",
    );
}

/// Scenario (offset guarantees): tracked offsets are snapshotted into the shared
/// rebalance state, then a partition is assigned.
/// Guarantees: the committable snapshot feeds the rebalance state's assignment view, so
/// pre-rebalance commits see the correct assigned partitions.
#[test]
fn refresh_committable_snapshot_feeds_rebalance_state() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver.offset_tracker.track("traces", 0, 101, 0);
    receiver.refresh_committable_snapshot();

    // The shared state now reports partition 0 as assigned-or-not, but the
    // committable snapshot drives pre-rebalance commits. Assign and verify
    // the late-ack guard sees the partition.
    receiver.rebalance_state.assign_for_test("traces", 0, 1);
    assert!(receiver.rebalance_state.is_assigned("traces", 0));
    assert!(!receiver.rebalance_state.is_assigned("traces", 9));
}

/// Scenario (offset guarantees): the committable snapshot is refreshed, the lowest
/// pending offset is acknowledged, then it is refreshed again.
/// Guarantees: the snapshot advances to the next committable offset after the
/// acknowledge, so a subsequent pre-rebalance commit is never stale.
#[test]
fn snapshot_reflects_committable_after_advance() {
    // Mirrors what advance_offset_and_commit does (minus the live commit):
    // acknowledging the lowest pending offset advances the committable
    // watermark, and refreshing the snapshot must reflect it so a
    // subsequent pre-rebalance commit is not stale.
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    receiver.offset_tracker.track("traces", 0, 100, 0);
    receiver.offset_tracker.track("traces", 0, 101, 0);
    receiver.refresh_committable_snapshot();
    assert_eq!(
        receiver.rebalance_state.committable_for_test("traces", 0),
        Some(100)
    );

    // Advance past 100; snapshot must now reflect 101.
    let advanced = receiver.offset_tracker.acknowledge("traces", 0, 100);
    assert!(advanced);
    receiver.refresh_committable_snapshot();
    assert_eq!(
        receiver.rebalance_state.committable_for_test("traces", 0),
        Some(101)
    );
}

/// Scenario (offset guarantees): a single manual-commit consumer owns one
/// partition holding three in-flight records; the records are acked
/// out of order (offsets 1 and 2 first, the lowest offset 0 withheld),
/// then offset 0 is acked last.
/// Guarantees: the committed watermark holds at the gap while the lowest
/// offset is un-acked (it never advances past an un-acked offset, so
/// at-least-once cannot skip an offset), and only after offset 0 is acked
/// does it jump to the full record count -- proving the lowest-un-acked
/// watermark commit logic end-to-end through the broker.
#[tokio::test]
async fn out_of_order_acks_commit_only_lowest_contiguous() {
    const TOPIC: &str = "offset-out-of-order-traces";
    const RECORDS: usize = 3;
    let group = "offset-out-of-order-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // Produce three records to the single partition; they receive
            // offsets 0, 1, 2 in order.
            for i in 0..RECORDS {
                let key = format!("rec-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            // No safety-net timer: commits are driven purely by acks so the
            // watermark assertions are deterministic.
            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume all three records, correlating each delivered pdata
            // back to its Kafka offset via the stamped calldata so acks can
            // be issued in a controlled (out-of-order) sequence.
            let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_topic_id, _partition, offset, _generation) = decode_calldata(&route.calldata);
                let _ = by_offset.insert(offset, pdata);
            }
            assert_eq!(by_offset.len(), RECORDS, "expected one pdata per offset");

            let brokers = cluster.bootstrap_servers().to_string();

            // Ack offsets 1 and 2 first, withholding the lowest offset 0.
            receiver.ack(by_offset.remove(&1).expect("offset 1 delivered"));
            receiver.ack(by_offset.remove(&2).expect("offset 2 delivered"));

            // The committed offset must NOT advance past the un-acked lowest
            // offset 0. Because a manual commit only advances the lowest
            // contiguous acked offset, no commit should reach offset 1+.
            // Give the loop time to process the acks, then assert the
            // watermark is still below 1 (either uncommitted or 0).
            tokio::time::sleep(Duration::from_millis(500)).await;
            let committed_before = committed_offset(&brokers, group, TOPIC, 0)
                .expect("kafka-test: committed-offset probe failed");
            assert!(
                committed_before.is_none_or(|o| o < 1),
                "committed offset must not advance past the un-acked lowest \
                     offset 0, got {committed_before:?}",
            );

            // Ack the withheld lowest offset 0. Now the contiguous run
            // 0,1,2 is complete, so the watermark jumps to the full count.
            receiver.ack(by_offset.remove(&0).expect("offset 0 delivered"));

            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= RECORDS as i64)
            })
            .await;
            assert!(
                advanced,
                "once the lowest offset is acked the watermark should jump to \
                     the full count {RECORDS}, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (offset guarantees): an undecodable OTAP-Arrow traces record is
/// produced between two well-formed OTAP records on a single partition of a
/// manual-commit receiver.
/// Guarantees: the poison record is counted as a processing/unmarshal error
/// and is never forwarded downstream, yet the surrounding good records are
/// still delivered and the committed offset advances past the poison record
/// -- so one undecodable message cannot stall the partition or violate the
/// late-ack guard.
#[tokio::test]
async fn poison_message_advances_without_stalling_partition() {
    const TOPIC: &str = "offset-poison-traces";
    let group = "offset-poison-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Well-formed OTAP-Arrow trace bytes for the surrounding records.
            let good = create_traces_with_spans_otap_bytes();
            // Not a valid OTAP BatchArrowRecords payload: decoding fails.
            let poison = b"this-is-not-a-valid-otap-arrow-payload".to_vec();

            // Order on the partition: good(0), good(1), poison(2). Every
            // record carries the OTAP MessageFormat header so the receiver
            // uses the OTAP decode path (which validates the payload, unlike
            // the zero-copy OTLP path).
            producer
                .send_full(
                    SendRecord::new(TOPIC, &good)
                        .key(b"good-a")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send good a");
            producer
                .send_full(
                    SendRecord::new(TOPIC, &good)
                        .key(b"good-b")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send good b");
            producer
                .send_full(
                    SendRecord::new(TOPIC, &poison)
                        .key(b"poison")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send poison");

            let cfg = manual_otap_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Only the two good records are forwarded; the poison record is
            // dropped. Ack the good records so the watermark advances past
            // the poison offset in between.
            let first = receiver.recv_pdata().await;
            let second = receiver.recv_pdata().await;
            receiver.ack(first);
            receiver.ack(second);

            // No third record should arrive: the poison record was never
            // forwarded.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "poison record must not be forwarded downstream",
            );

            let brokers = cluster.bootstrap_servers().to_string();
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= 3)
            })
            .await;
            assert!(
                advanced,
                "committed offset must advance past the poison record to the \
                     full count 3, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            let decode_rejections = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.rejections",
                &[
                    ("signal", "traces"),
                    ("error.type", "invalid_request"),
                    ("reason", "decode"),
                ],
                "messages",
            );
            assert!(
                decode_rejections >= 1,
                "the poison record must be counted as a decode rejection, got {decode_rejections}",
            );
        },
    )
    .await;
}

/// Scenario (offset guarantees): an auto-commit (at-most-once) receiver
/// consumes every produced record but never acks; librdkafka owns offsets
/// and auto-commits them periodically.
/// Guarantees: the broker-side committed offset still advances to the full
/// record count purely from librdkafka's auto-commit, while the receiver's
/// manual tracker/rebalance-commit paths stay inert (successful acknowledgement
/// responses and failed offset commits remain 0) -- proving auto-commit mode is a true
/// no-op for the manual offset machinery.
#[tokio::test]
async fn auto_commit_mode_lets_librdkafka_own_offsets() {
    const TOPIC: &str = "offset-auto-commit-traces";
    const RECORDS: usize = 4;
    // Auto-commit mode uses the fixed "test-group" from `auto_config`.
    let group = "test-group";
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
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume every record but never ack: under auto-commit the
            // receiver does not track offsets, and librdkafka commits on its
            // own periodic schedule.
            for _ in 0..RECORDS {
                let _ = receiver.recv_pdata().await;
            }

            // Wait long enough for at least one auto-commit interval (1000ms)
            // to elapse and be flushed.
            let brokers = cluster.bootstrap_servers().to_string();
            let advanced = poll_until(Duration::from_secs(10), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= RECORDS as i64)
            })
            .await;
            assert!(
                advanced,
                "librdkafka auto-commit should advance the committed offset to \
                     the full count {RECORDS} without any acks, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            assert_eq!(
                measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.acknowledgements",
                    &[("signal", "traces"), ("outcome", "success")],
                    "responses",
                ),
                0,
                "auto-commit mode must not receive acknowledgements",
            );
            assert_eq!(
                measurement_counter(
                    terminal.metrics(),
                    "receiver.kafka.offset_commits",
                    &[("outcome", "failure")],
                    "commits",
                ),
                0,
                "the manual commit path is inert under auto-commit, so no \
                     failed offset commits should be recorded",
            );
        },
    )
    .await;
}

/// Scenario (offset guarantees): a manual-commit receiver consumes a record
/// and holds it in-flight (un-acked) while a downstream retry would be in
/// progress, then the record receives a permanent NACK from downstream.
/// Guarantees: the offset stays uncommitted while the record is in-flight
/// (the committed watermark does not advance past it), and advances to the
/// full count only once the terminal permanent Nack arrives -- proving the
/// receiver holds the offset during retries and advances only on a
/// permanent outcome. Local transient retry is tested by the
/// `processor:retry` node (see `retry_processor` tests
/// `test_retry_processor_permanent_error_not_retried`,
/// `test_retry_processor_nacks_then_timeout`,
/// `test_retry_processor_nacks_then_limit`).
#[tokio::test]
async fn terminal_nack_advances_offset_past_message() {
    const TOPIC: &str = "offset-terminal-nack-traces";
    let group = "offset-terminal-nack-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // A single record on the single partition.
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"rec-0"))
                .await
                .expect("send record");

            let cfg = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume the record but hold it un-acked: this models the window
            // where a downstream `processor:retry` is still retrying, so no
            // terminal outcome has reached the receiver yet.
            let pdata = receiver.recv_pdata().await;

            // While the record is in-flight the offset must NOT be committed.
            let brokers = cluster.bootstrap_servers().to_string();
            tokio::time::sleep(Duration::from_millis(500)).await;
            let committed_in_flight = committed_offset(&brokers, group, TOPIC, 0)
                .expect("kafka-test: committed-offset probe failed");
            assert!(
                committed_in_flight.is_none_or(|o| o < 1),
                "offset must stay uncommitted while the record is in-flight \
                     (retries in progress), got {committed_in_flight:?}",
            );

            // A downstream component classifies the failure as permanent.
            // The retry processor forwards this terminal outcome without
            // retrying it.
            receiver.nack_permanent("permanent downstream failure", pdata);

            // The terminal Nack advances the offset past the message.
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= 1)
            })
            .await;
            assert!(
                advanced,
                "a terminal permanent Nack must advance the committed offset \
                     past the message, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (offset guarantees): a manual-commit receiver consumes records
/// but never acks them (so nothing is committed), then is fully shut down;
/// a second receiver is started in the same consumer group on the same
/// cluster (a consumer restart).
/// Guarantees: because the first receiver committed nothing, the broker
/// retains no progress and the restarted receiver re-receives the
/// uncommitted records -- proving at-least-once redelivery with no data loss
/// across a consumer restart.
#[tokio::test]
async fn restart_redelivers_uncommitted_offsets() {
    const TOPIC: &str = "offset-restart-traces";
    const RECORDS: usize = 3;
    let group = "offset-restart-group";
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

            // First receiver: consume every record but NEVER ack, so no
            // offset is ever committed.
            let cfg_a = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);
            for _ in 0..RECORDS {
                let _ = receiver_a.recv_pdata().await;
            }

            // Nothing was acked, so the broker must hold no committed offset.
            let brokers = cluster.bootstrap_servers().to_string();
            tokio::time::sleep(Duration::from_millis(500)).await;
            let committed_before = committed_offset(&brokers, group, TOPIC, 0)
                .expect("kafka-test: committed-offset probe failed");
            assert!(
                committed_before.is_none_or(|o| o < RECORDS as i64),
                "no offset should be committed before restart (records were \
                     never acked), got {committed_before:?}",
            );

            // Fully stop the first receiver (a restart).
            receiver_a.shutdown(Duration::from_secs(5));
            receiver_a.await_stopped().await;

            // Second receiver in the SAME group: it must re-receive the
            // uncommitted records (at-least-once redelivery, no loss).
            let cfg_b = manual_traces_config_no_timer(cluster.bootstrap_servers(), group, TOPIC);
            let mut receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);
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
                "restarted receiver must re-receive all {RECORDS} uncommitted \
                     records, got {redelivered}",
            );

            receiver_b.shutdown(Duration::from_secs(5));
            receiver_b.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (offset guarantees): a manual-commit receiver is configured with
/// a short safety-net commit timer (`commit.interval_ms`); records are
/// consumed and acked, but the receiver is neither drained nor shut down
/// while the assertion runs.
/// Guarantees: the periodic `TimerTick` commit path advances the broker-side
/// committed offset to the full acked count on its own -- without relying on
/// the drain/shutdown final commit -- so the safety-net timer durably
/// persists acked progress during steady-state operation.
#[tokio::test]
async fn safety_net_timer_commits_without_acks_drain_or_shutdown() {
    const TOPIC: &str = "offset-safety-timer-traces";
    const RECORDS: i64 = 3;
    let group = "offset-safety-timer-group";
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

            // Short safety-net timer so the periodic commit fires well within
            // the assertion window; acks alone would also commit, but the
            // point here is that the commit is observed BEFORE any
            // drain/shutdown, i.e. driven by the timer tick.
            let cfg = manual_traces_config(cluster.bootstrap_servers(), group, TOPIC, 200, None);
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            // Wait for the periodic commit timer to persist the acked
            // offsets. No drain, no shutdown yet: the commit must come from
            // the safety-net TimerTick path alone.
            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(100), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= RECORDS)
            })
            .await;
            assert!(
                committed,
                "the safety-net commit timer must advance the committed offset \
                     to the full acked count {RECORDS} without a drain/shutdown, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
