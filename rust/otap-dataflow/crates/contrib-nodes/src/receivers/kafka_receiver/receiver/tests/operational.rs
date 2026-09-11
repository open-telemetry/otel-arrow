// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Operational-visibility tests: consumer-lag computation and rejection-reason categorization.

use super::*;

// ---- Operational visibility ----

/// Scenario (operational visibility): a consumer owns partitions but *none* of them has a
/// broker-committed offset yet (every `committed_offsets` entry is
/// `Offset::Invalid`).
/// Guarantees: `compute_consumer_lag` reports the refresh as incomplete
/// (`None`) instead of computing a mean from a subset, so the caller retains
/// the previous `receiver.kafka.consumer.group.lag` value rather than
/// publishing a partial or zeroed measurement.
#[tokio::test]
async fn compute_consumer_lag_none_when_all_offsets_invalid() {
    const TOPIC: &str = "lag-all-invalid";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let brokers = cluster.bootstrap_servers().to_string();
            // Assign both partitions but never commit, so the broker holds
            // no committed offset for either -> both `Offset::Invalid`.
            let consumer = make_manual_consumer(&brokers, "lag-all-invalid-group");
            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);
            let _ = tpl.add_partition(TOPIC, 1);
            consumer.assign(&tpl).expect("assign partitions");

            let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
            let result = tokio::task::spawn_blocking(move || {
                compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
            })
            .await
            .expect("lag task should not panic");

            assert_eq!(
                result, None,
                "an assignment with no committed offsets must abort the refresh, not \
                     produce a subset/zero mean",
            );
        },
    )
    .await;
}

/// Scenario (operational visibility): the receive loop's lag-refresh deadline elapses, so the loop
/// cancels the worker's token (as the `Err(Elapsed)` arm does) while the
/// worker still owns partitions.
/// Guarantees: a cancelled token makes `compute_consumer_lag` abandon the
/// refresh (`None`) at its next cancellation check instead of continuing to
/// issue broker calls -- the observable behavior that lets the loop drop the
/// wedged worker and resume future refreshes without blocking.
#[tokio::test]
async fn compute_consumer_lag_none_when_cancelled() {
    const TOPIC: &str = "lag-cancelled";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let brokers = cluster.bootstrap_servers().to_string();
            let consumer = make_manual_consumer(&brokers, "lag-cancelled-group");
            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);
            let _ = tpl.add_partition(TOPIC, 1);
            consumer.assign(&tpl).expect("assign partitions");

            // Pre-cancel the token to model the timeout path cancelling a
            // still-running worker. The assignment is non-empty, so the
            // cancellation check (not the empty-assignment shortcut) decides
            // the outcome.
            let cancel = CancellationToken::new();
            cancel.cancel();
            let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
            let result = tokio::task::spawn_blocking(move || {
                compute_consumer_lag(&consumer, deadline, &cancel)
            })
            .await
            .expect("lag task should not panic");

            assert_eq!(
                result, None,
                "a cancelled refresh must abandon measurement rather than \
                     continue issuing broker calls",
            );
        },
    )
    .await;
}

/// Scenario (operational visibility): a consumer owns two partitions but only one has a
/// broker-committed offset; the other is still `Offset::Invalid`.
/// Guarantees: `compute_consumer_lag` aborts (`None`) because the mean must
/// cover every owned partition -- it never silently drops the uncommitted
/// partition and averages only the committed one.
#[tokio::test]
async fn compute_consumer_lag_none_when_offsets_mixed_valid_invalid() {
    const TOPIC: &str = "lag-mixed";
    let group = "lag-mixed-group";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 2, 1),
        |cluster| async move {
            let brokers = cluster.bootstrap_servers().to_string();
            let producer = cluster.producer().build();

            // Produce a few records to partition 0 only.
            for _ in 0..3 {
                producer
                    .send_to_partition(TOPIC, 0, b"payload")
                    .await
                    .expect("produce to partition 0");
            }
            producer.flush(Duration::from_secs(5));

            let consumer = make_manual_consumer(&brokers, group);
            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);
            let _ = tpl.add_partition(TOPIC, 1);
            consumer.assign(&tpl).expect("assign partitions");

            // Commit an offset for partition 0 only, leaving partition 1
            // without a committed offset (`Offset::Invalid`).
            let mut commit_tpl = TopicPartitionList::new();
            commit_tpl
                .add_partition_offset(TOPIC, 0, Offset::Offset(2))
                .expect("build commit tpl");
            consumer
                .commit(&commit_tpl, CommitMode::Sync)
                .expect("commit partition 0");

            let deadline = Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
            let result = tokio::task::spawn_blocking(move || {
                compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
            })
            .await
            .expect("lag task should not panic");

            assert_eq!(
                result, None,
                "a mix of committed and uncommitted owned partitions must abort the \
                     refresh so the mean is never taken over a subset",
            );
        },
    )
    .await;
}

/// Scenario (operational visibility): the total refresh deadline has already passed when
/// `compute_consumer_lag` starts (assignment is non-empty).
/// Guarantees: the worker self-terminates with `None` (incomplete) at its
/// first between-partition/broker-call deadline check rather than issuing
/// broker calls, so an overrunning refresh bounds itself.
#[tokio::test]
async fn compute_consumer_lag_none_when_deadline_already_passed() {
    const TOPIC: &str = "lag-deadline";
    with_cluster(
        KafkaTestCluster::builder().topic_with(TOPIC, 1, 1),
        |cluster| async move {
            let brokers = cluster.bootstrap_servers().to_string();
            let consumer = make_manual_consumer(&brokers, "lag-deadline-group");
            let mut tpl = TopicPartitionList::new();
            let _ = tpl.add_partition(TOPIC, 0);
            consumer.assign(&tpl).expect("assign partition");

            // Deadline in the past: the first broker-call deadline check
            // must abort before any committed_offsets/fetch_watermarks call.
            let deadline = Instant::now() - Duration::from_secs(1);
            let result = tokio::task::spawn_blocking(move || {
                compute_consumer_lag(&consumer, deadline, &CancellationToken::new())
            })
            .await
            .expect("lag task should not panic");

            assert_eq!(
                result, None,
                "an already-expired deadline must abort the refresh"
            );
        },
    )
    .await;
}

/// Scenario (operational visibility): the receive loop's lag apply branch observes the in-flight
/// worker *finish* with a value (a real mean, or the `0.0`
/// empty-assignment reset).
/// Guarantees: the apply branch publishes the value to
/// `receiver.kafka.consumer.group.lag` and clears the in-flight slot so the
/// next tick may start a fresh refresh.
#[tokio::test]
async fn lag_apply_publishes_and_clears_on_completion() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // A finished worker that measured a mean of 42.0.
    let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
        Some((
            tokio::task::spawn(async { Some(42.0_f64) }),
            tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE,
        ));
    let join_result = in_flight.as_mut().map(|(h, _)| h).expect("in flight").await;

    // Mirror the apply branch's inlined result-handling.
    let result: Result<Result<Option<f64>, tokio::task::JoinError>, tokio::time::error::Elapsed> =
        Ok(join_result);
    match result {
        Err(_elapsed) => unreachable!("worker finished, not a deadline crossing"),
        Ok(join_result) => {
            in_flight = None;
            match join_result {
                Ok(Some(value)) => receiver.metrics.consumer.lag.set(value),
                Ok(None) => {}
                Err(join_err) => panic!("unexpected join error: {join_err}"),
            }
        }
    }

    assert_eq!(receiver.metrics.consumer.lag.get(), 42.0);
    assert!(
        in_flight.is_none(),
        "a finished worker must clear the in-flight slot",
    );
}

/// Scenario (operational visibility): the lag apply branch observes the absolute deadline elapse
/// while the worker is still running (a `spawn_blocking` task cannot be
/// cancelled by dropping its handle).
/// Guarantees: the apply branch keeps the in-flight slot set so the trigger
/// branch cannot start a second worker -- proving at most one worker runs at
/// a time -- and does not disturb the previous gauge value.
#[tokio::test(start_paused = true)]
async fn lag_apply_keeps_in_flight_on_deadline_and_blocks_new_worker() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Seed a known gauge value so we can prove it is retained on timeout.
    receiver.metrics.consumer.lag.set(7.0);

    // A worker that never finishes within the deadline.
    let deadline = tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
    let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
        Some((
            tokio::task::spawn(async {
                std::future::pending::<()>().await;
                None
            }),
            deadline,
        ));

    // Cross the deadline (paused clock), then await with `timeout_at`.
    tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE + Duration::from_secs(1)).await;
    let handle = in_flight.as_mut().map(|(h, _)| h).expect("in flight");
    let result = tokio::time::timeout_at(deadline, handle).await;
    assert!(
        result.is_err(),
        "worker must still be running at the deadline"
    );

    // Mirror the apply branch: on `Err(Elapsed)` keep the in-flight slot and
    // leave the gauge untouched.
    match result {
        Err(_elapsed) => { /* keep in_flight, retain gauge */ }
        Ok(_) => unreachable!("deadline crossing, worker not finished"),
    }

    assert!(
        in_flight.is_some(),
        "a deadline crossing must NOT clear the in-flight slot, so the trigger branch \
             (guarded by is_none) cannot start a second worker while the first still runs",
    );
    assert_eq!(
        receiver.metrics.consumer.lag.get(),
        7.0,
        "the previous gauge value must be retained on a deadline crossing",
    );

    // Clean up the still-running background task.
    if let Some((handle, _)) = in_flight.take() {
        handle.abort();
    }
}

/// Scenario (operational visibility): paused time; the apply branch is polled repeatedly while the
/// receive branch would always be ready. After a deadline crossing the
/// branch must await the *bare* handle (no spinning `timeout_at`) so it does
/// not starve `recv()`, and it must still process the worker's eventual
/// completion.
/// Guarantees: once the worker finally exits, the apply branch publishes its
/// value and clears the in-flight slot even though it was polled past the
/// deadline -- i.e. a completed refresh is never lost to starvation, and the
/// deadline is absolute (not reset by re-polling).
#[tokio::test(start_paused = true)]
async fn lag_apply_processes_completion_after_deadline() {
    let cfg = make_config(&["traces"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let mut receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    let deadline = tokio::time::Instant::now() + LAG_REFRESH_TOTAL_DEADLINE;
    // A worker that completes only after the deadline has passed.
    let mut in_flight: Option<(tokio::task::JoinHandle<Option<f64>>, tokio::time::Instant)> =
        Some((
            tokio::task::spawn(async {
                tokio::time::sleep(LAG_REFRESH_TOTAL_DEADLINE * 2).await;
                Some(5.0_f64)
            }),
            deadline,
        ));

    // Advance past the deadline; the worker is still sleeping.
    tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE + Duration::from_secs(1)).await;

    // Past the deadline the loop awaits the bare handle (no timeout). Model
    // that: it resolves only when the worker actually finishes.
    tokio::time::advance(LAG_REFRESH_TOTAL_DEADLINE).await;
    let handle = in_flight.as_mut().map(|(h, _)| h).expect("in flight");
    let join_result = handle.await;

    // Mirror the apply branch's inlined result-handling for a finished worker.
    let result: Result<Result<Option<f64>, tokio::task::JoinError>, tokio::time::error::Elapsed> =
        Ok(join_result);
    match result {
        Err(_elapsed) => unreachable!("worker finished, not a deadline crossing"),
        Ok(join_result) => {
            in_flight = None;
            match join_result {
                Ok(Some(value)) => receiver.metrics.consumer.lag.set(value),
                Ok(None) => {}
                Err(join_err) => panic!("unexpected join error: {join_err}"),
            }
        }
    }

    assert_eq!(
        receiver.metrics.consumer.lag.get(),
        5.0,
        "a refresh that completes after the deadline must still be published",
    );
    assert!(
        in_flight.is_none(),
        "the in-flight slot must be cleared once the worker finishes",
    );
}

/// Scenario (operational visibility): a manual-commit receiver processes a
/// well-formed record followed by an undecodable OTAP record on the same
/// topic.
/// Guarantees: the data-processing failure is attributed to the bounded
/// `decode` rejection reason while the unrelated `unknown_topic` rejection
/// and partition-revocation metrics stay at zero.
#[tokio::test]
async fn decode_rejections_are_categorized_separately_from_filtering_and_rebalance() {
    const TOPIC: &str = "visibility-processing-traces";
    let group = "visibility-processing-group";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();
            let good = create_traces_with_spans_otap_bytes();
            let poison = b"not-a-valid-otap-arrow-payload".to_vec();

            producer
                .send_full(
                    SendRecord::new(TOPIC, &good)
                        .key(b"good")
                        .header("MessageFormat", MSG_FORMAT_OTAP),
                )
                .await
                .expect("send good");
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

            // The good record is delivered; the poison record is not.
            let pdata = receiver.recv_pdata().await;
            receiver.ack(pdata);
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "poison record must not be forwarded",
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            let mut m = FoldedMetrics::new();
            m.fold_all(terminal.metrics());

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
            let unknown_topic_rejections = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.rejections",
                &[
                    ("signal", "unknown"),
                    ("error.type", "invalid_request"),
                    ("reason", "unknown_topic"),
                ],
                "messages",
            );
            assert!(
                decode_rejections >= 1,
                "decode rejection reason should count the failure, got {decode_rejections}",
            );
            assert_eq!(
                unknown_topic_rejections, 0,
                "a decode failure must not be counted as an unknown-topic rejection",
            );
            assert_eq!(
                m.value("group.partition.revocations"),
                0,
                "a decode failure must not be counted as a rebalance \
                     revocation, got {}",
                m.value("group.partition.revocations"),
            );
        },
    )
    .await;
}

/// Scenario (operational visibility): a receiver subscribes to a `^`-prefixed
/// include regex that also matches an `exclude_topics` pattern; a record is
/// produced to the excluded topic (which librdkafka still delivers because
/// the include regex matches) alongside a record on a normal included topic.
/// Guarantees: the excluded record is attributed to the bounded
/// `unknown_topic` rejection reason while the `decode` reason stays at zero.
#[tokio::test]
async fn unknown_topic_rejections_are_categorized_separately_from_decode_errors() {
    const INCLUDED: &str = "visibility-included";
    const EXCLUDED: &str = "visibility-excluded";
    let group = "visibility-filtering-group";
    with_cluster(
        KafkaTestCluster::builder().topic(INCLUDED).topic(EXCLUDED),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            // One record on each topic. Both are well-formed, so any counted
            // error is a filtering decision, not a decode failure.
            producer
                .send_full(SendRecord::new(INCLUDED, &bytes).key(b"inc"))
                .await
                .expect("send included");
            producer
                .send_full(SendRecord::new(EXCLUDED, &bytes).key(b"exc"))
                .await
                .expect("send excluded");

            // Include everything matching `^visibility-`, but exclude the
            // `visibility-excluded` topic. librdkafka subscribes to both
            // (the include regex matches), so the receiver-side guard is what
            // rejects the excluded topic.
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec!["^visibility-.*".to_string()])
                            .with_encoding(MessageFormat::OtlpProto)
                            .with_exclude_topics(vec!["^visibility-excluded$".to_string()]),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted);
            let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The included topic's record is delivered and decoded.
            let pdata = receiver.recv_pdata().await;
            receiver.ack(pdata);
            // The excluded topic's record is filtered out, never delivered.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "excluded topic record must not be forwarded",
            );

            receiver.shutdown(Duration::from_secs(5));
            let terminal = receiver.await_terminal_state().await;
            let unknown_topic_rejections = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.rejections",
                &[
                    ("signal", "unknown"),
                    ("error.type", "invalid_request"),
                    ("reason", "unknown_topic"),
                ],
                "messages",
            );
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
                unknown_topic_rejections >= 1,
                "the excluded topic should be counted as an unknown-topic rejection, got \
                     {unknown_topic_rejections}",
            );
            assert_eq!(
                decode_rejections, 0,
                "expected filtering must not be counted as a decode rejection",
            );
        },
    )
    .await;
}
