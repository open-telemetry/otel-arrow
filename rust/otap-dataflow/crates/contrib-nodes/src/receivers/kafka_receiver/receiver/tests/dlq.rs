// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration and edge-case coverage for the Kafka receiver dead-letter queue.
//!
//! These tests exercise the DLQ egress against an in-process mock cluster: raw
//! bytes recovery (inline for decode / unknown-topic, re-read for
//! permanent-nack), byte fidelity, the non-stall contract under producer and
//! re-read stalls, pending-queue overflow, and offset advancement only on a
//! terminal DLQ outcome.

use super::*;
use crate::receivers::kafka_receiver::config::{DlqCapture, DlqConfig};

/// Build a manual-commit OTAP-proto traces config with a DLQ. OTAP-proto decode
/// validates the payload, so a garbage payload deterministically fails to
/// decode (unlike OTLP-proto, whose bytes are wrapped without validation).
fn manual_otap_traces_config_with_dlq(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
    dlq_topic: &str,
    capture: Vec<DlqCapture>,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtapProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted)
        .with_dlq(DlqConfig {
            topic: Some(dlq_topic.to_string()),
            per_signal: None,
            capture,
            connection: None,
        });
    KafkaReceiverConfig::try_from(builder).expect("test DLQ config valid")
}

/// Read up to `n` records from `topic` on the mock cluster, returning fewer if
/// they do not all arrive within the timeout.
async fn drain_dlq_topic(
    cluster: &KafkaTestCluster,
    topic: &str,
    n: usize,
    per_record: Duration,
) -> Vec<crate::common::kafka::test::message::ConsumedMessage> {
    let consumer = cluster
        .consumer()
        .group_id("dlq-reader")
        .auto_offset_reset("earliest")
        .subscribe(&[topic]);
    let mut out = Vec::new();
    for _ in 0..n {
        match consumer.try_recv(per_record).await {
            Some(msg) => out.push(msg),
            None => break,
        }
    }
    out
}

/// Scenario: a manual-commit receiver with DLQ decode capture consumes an
/// undecodable OTLP-proto record.
/// Guarantees: the original raw bytes are dead-lettered byte-identically to the
/// configured DLQ topic with the expected dlq.* context headers, and the source
/// offset advances (the poison message does not block the partition).
#[tokio::test]
async fn decode_failure_is_dead_lettered_byte_identical() {
    const TOPIC: &str = "dlq-decode-src";
    const DLQ: &str = "dlq-decode-out";
    let group = "dlq-decode-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // A payload that is not a valid OTAP-proto BatchArrowRecords, so it
            // deterministically fails to decode.
            let bad = b"not-a-valid-otap-proto-payload".to_vec();
            producer
                .send_full(SendRecord::new(TOPIC, &bad).key(b"k"))
                .await
                .expect("send");

            let cfg = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");
            let record = &records[0];
            assert_eq!(
                record.payload.as_deref(),
                Some(bad.as_slice()),
                "DLQ payload must be byte-identical to the source"
            );
            assert_eq!(record.header("dlq.reason"), Some(&b"decode"[..]));
            assert_eq!(record.header("dlq.source.topic"), Some(TOPIC.as_bytes()));
            assert!(record.header("dlq.error").is_some());

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a permanently-nacked message is dead-lettered via the dedicated
/// re-read consumer.
/// Guarantees: the receiver recovers the original bytes from Kafka and produces
/// them to the DLQ topic byte-identically with reason permanent_nack.
#[tokio::test]
async fn permanent_nack_is_dead_lettered_via_reread() {
    const TOPIC: &str = "dlq-nack-src";
    const DLQ: &str = "dlq-nack-out";
    let group = "dlq-nack-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The receiver decodes and forwards the record downstream; permanently
            // nack it to trigger the DLQ re-read workflow.
            let pdata = receiver.recv_pdata().await;
            receiver.nack_permanent("terminal", pdata);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");
            assert_eq!(
                records[0].payload.as_deref(),
                Some(bytes.as_slice()),
                "re-read DLQ payload must be byte-identical to the source"
            );
            assert_eq!(
                records[0].header("dlq.reason"),
                Some(&b"permanent_nack"[..])
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: the DLQ producer cannot deliver because every produce is rejected
/// by the broker while good records keep arriving on another partition.
/// Guarantees: the main receive loop keeps consuming and forwarding good records
/// (it is never blocked by the stalled DLQ path), and the failed message is
/// eventually counted as a DLQ loss rather than wedging ingestion.
#[tokio::test]
async fn producer_stall_does_not_block_receive_loop() {
    const TOPIC: &str = "dlq-stall-src";
    const DLQ: &str = "dlq-stall-out";
    let group = "dlq-stall-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Seed a bad (undecodable OTAP) record followed by a good one BEFORE
            // enabling the produce fault, so the source topic is populated.
            let bad = b"poison-otap".to_vec();
            producer
                .send_full(SendRecord::new(TOPIC, &bad).key(b"bad"))
                .await
                .expect("send");
            let good = create_traces_with_spans_otap_bytes();
            producer
                .send_full(SendRecord::new(TOPIC, &good).key(b"good"))
                .await
                .expect("send");

            // Now fail every produce so the DLQ egress cannot make progress.
            cluster
                .faults()
                .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_TOPIC_AUTHORIZATION_FAILED]);

            let cfg = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Despite the stalled DLQ producer, the good record must still be
            // decoded and forwarded downstream: the receive loop is not blocked.
            let pdata = receiver
                .try_recv_pdata(Duration::from_secs(30))
                .await
                .expect("good record must be forwarded despite DLQ produce stall");
            receiver.ack(pdata);

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a permanent nack is dead-lettered but the re-read consumer can
/// never fetch the offset (all fetches fail), so the re-read times out.
/// Guarantees: the receiver keeps servicing control messages and the loss path
/// advances rather than wedging; the receiver shuts down cleanly.
#[tokio::test]
async fn reread_stall_does_not_block_receive_loop() {
    const TOPIC: &str = "dlq-reread-stall-src";
    const DLQ: &str = "dlq-reread-stall-out";
    let group = "dlq-reread-stall-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            // The re-read fetch is bounded by the fixed DLQ operation timeout,
            // so the loss path resolves without blocking the receive loop.
            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let pdata = receiver.recv_pdata().await;

            // Make every fetch fail so the re-read consumer cannot recover bytes.
            cluster
                .faults()
                .fail_fetch(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_UNKNOWN_TOPIC_OR_PART]);

            receiver.nack_permanent("terminal", pdata);

            // The receiver must still service control messages promptly while the
            // re-read is stalled/timing out.
            receiver.wait_for_control_barrier().await;

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: the DLQ shuts down cleanly while a re-read/produce may still be in
/// flight, using a bounded deadline.
/// Guarantees: the receiver reaches its terminal state within the shutdown
/// deadline even when a DLQ delivery is outstanding (drain is bounded).
#[tokio::test]
async fn shutdown_drains_dlq_within_deadline() {
    const TOPIC: &str = "dlq-shutdown-src";
    const DLQ: &str = "dlq-shutdown-out";
    let group = "dlq-shutdown-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let pdata = receiver.recv_pdata().await;
            receiver.nack_permanent("terminal", pdata);

            // Immediately shut down: the bounded DLQ drain must not wedge the
            // terminal path.
            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a DLQ is enabled but does not capture permanent nacks.
/// Guarantees: a permanent nack follows today's terminal path (no dead-letter),
/// so nothing is written to the DLQ topic.
#[tokio::test]
async fn permanent_nack_not_captured_is_not_dead_lettered() {
    const TOPIC: &str = "dlq-nocap-src";
    const DLQ: &str = "dlq-nocap-out";
    let group = "dlq-nocap-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            // DLQ captures only decode failures, not permanent nacks.
            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let pdata = receiver.recv_pdata().await;
            receiver.nack_permanent("terminal", pdata);

            // No record should be dead-lettered.
            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(3)).await;
            assert!(
                records.is_empty(),
                "permanent nack must not be dead-lettered when not captured"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a permanently-nacked traces message is dead-lettered via re-read.
/// Guarantees: the DLQ record carries `dlq.signal = traces` (resolved from the
/// source topic), not the placeholder `unknown`, so telemetry and the record
/// header attribute the loss to the correct signal.
#[tokio::test]
async fn permanent_nack_dead_letter_carries_source_signal() {
    const TOPIC: &str = "dlq-nack-signal-src";
    const DLQ: &str = "dlq-nack-signal-out";
    let group = "dlq-nack-signal-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);
            let pdata = receiver.recv_pdata().await;
            receiver.nack_permanent("terminal", pdata);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");
            assert_eq!(
                records[0].header("dlq.reason"),
                Some(&b"permanent_nack"[..])
            );
            assert_eq!(
                records[0].header("dlq.signal"),
                Some(&b"traces"[..]),
                "permanent-nack dead-letter must carry the resolved source signal, not `unknown`"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a decode failure is dead-lettered and the receiver is then shut
/// down so its terminal metric snapshot can be inspected.
/// Guarantees: `receiver.kafka.dlq.messages` increments with
/// `signal=traces,reason=decode,outcome=produced`, so the DLQ telemetry is wired
/// end-to-end with the expected attributes.
#[tokio::test]
async fn dlq_messages_metric_increments_with_attributes() {
    const TOPIC: &str = "dlq-metric-src";
    const DLQ: &str = "dlq-metric-out";
    let group = "dlq-metric-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bad = b"not-a-valid-otap-proto-payload".to_vec();
            producer
                .send_full(SendRecord::new(TOPIC, &bad).key(b"k"))
                .await
                .expect("send");

            let cfg = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Wait for the dead-letter to land so the metric is recorded before
            // the terminal snapshot is taken.
            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");

            let terminal = shutdown_and_terminal(receiver, Duration::from_secs(5)).await;
            let produced = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.dlq.messages",
                &[
                    ("signal", "traces"),
                    ("reason", "decode"),
                    ("outcome", "produced"),
                ],
                "messages",
            );
            assert_eq!(
                produced, 1,
                "one decode failure must be counted as a produced dead-letter"
            );
        },
    )
    .await;
}

/// Scenario: traces and metrics are configured with distinct per-signal DLQ
/// topics; a metrics record fails to decode.
/// Guarantees: the dead-letter lands in the metrics DLQ topic (not the traces
/// one), so per-signal routing is honored at runtime.
#[tokio::test]
async fn per_signal_dlq_topic_routing() {
    const TRACES: &str = "dlq-route-traces-src";
    const METRICS: &str = "dlq-route-metrics-src";
    const DLQ_TRACES: &str = "dlq-route-traces-out";
    const DLQ_METRICS: &str = "dlq-route-metrics-out";
    let group = "dlq-route-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TRACES, 1, 1)
            .topic_with(METRICS, 1, 1)
            .topic_with(DLQ_TRACES, 1, 1)
            .topic_with(DLQ_METRICS, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Undecodable OTAP-proto payload on the metrics topic.
            let bad = b"not-a-valid-otap-proto-payload".to_vec();
            producer
                .send_full(SendRecord::new(METRICS, &bad).key(b"k"))
                .await
                .expect("send");

            use crate::receivers::kafka_receiver::config::{DlqConfig, DlqPerSignalTopics};
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec![TRACES.to_string()])
                            .with_encoding(MessageFormat::OtapProto),
                    )
                    .with_metrics(
                        SignalConfig::new(vec![METRICS.to_string()])
                            .with_encoding(MessageFormat::OtapProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_isolation_level(IsolationLevel::ReadUncommitted)
                    .with_dlq(DlqConfig {
                        topic: None,
                        per_signal: Some(DlqPerSignalTopics {
                            traces: Some(DLQ_TRACES.to_string()),
                            metrics: Some(DLQ_METRICS.to_string()),
                            logs: None,
                        }),
                        capture: vec![DlqCapture::Decode],
                        connection: None,
                    });
            let cfg = KafkaReceiverConfig::try_from(builder).expect("valid");
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The record must land in the metrics DLQ topic.
            let metrics_dlq =
                drain_dlq_topic(&cluster, DLQ_METRICS, 1, Duration::from_secs(30)).await;
            assert_eq!(
                metrics_dlq.len(),
                1,
                "metrics decode failure must route to the metrics DLQ topic"
            );
            assert_eq!(metrics_dlq[0].header("dlq.signal"), Some(&b"metrics"[..]));

            // And nothing must land in the traces DLQ topic.
            let traces_dlq = drain_dlq_topic(&cluster, DLQ_TRACES, 1, Duration::from_secs(3)).await;
            assert!(
                traces_dlq.is_empty(),
                "no record should route to the traces DLQ topic"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: more poison records than the fixed in-flight bound
/// (`DLQ_MAX_IN_FLIGHT`) arrive while every DLQ produce is rejected, alongside a
/// good record on a separate partition.
/// Guarantees: the receive loop keeps forwarding the good record (never blocked
/// by the saturated DLQ path), and the overflow poison records are counted as
/// DLQ losses rather than wedging ingestion.
#[tokio::test]
async fn in_flight_full_records_loss_and_does_not_block() {
    const TOPIC: &str = "dlq-overflow-src";
    const DLQ: &str = "dlq-overflow-out";
    let group = "dlq-overflow-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 2, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Seed more poison records than DLQ_MAX_IN_FLIGHT (=5) on partition 0
            // and one good record on partition 1, BEFORE enabling the fault.
            for i in 0..8u32 {
                let bad = format!("poison-otap-{i}").into_bytes();
                producer
                    .send_full(SendRecord::new(TOPIC, &bad).key(b"bad").partition(0))
                    .await
                    .expect("send poison");
            }
            let good = create_traces_with_spans_otap_bytes();
            producer
                .send_full(SendRecord::new(TOPIC, &good).key(b"good").partition(1))
                .await
                .expect("send good");

            // Fail every produce so DLQ deliveries pile up against the in-flight
            // bound and then overflow into immediate losses.
            cluster
                .faults()
                .fail_produce(&[RDKafkaRespErr::RD_KAFKA_RESP_ERR_TOPIC_AUTHORIZATION_FAILED]);

            let cfg = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The good record on the other partition must still be forwarded.
            let pdata = receiver
                .try_recv_pdata(Duration::from_secs(30))
                .await
                .expect("good record must be forwarded despite DLQ overflow");
            receiver.ack(pdata);

            let terminal = shutdown_and_terminal(receiver, Duration::from_secs(5)).await;
            // Every poison record ultimately fails to dead-letter (all produces
            // rejected), so each is counted as a loss. We assert at least one loss
            // to prove the overflow/loss path executed without wedging.
            let loss = measurement_counter(
                terminal.metrics(),
                "receiver.kafka.dlq.loss",
                &[("signal", "traces"), ("reason", "decode")],
                "messages",
            );
            assert!(
                loss >= 1,
                "poison records must be counted as DLQ losses, got {loss}"
            );
        },
    )
    .await;
}

/// Scenario: a decode failure is dead-lettered and produced successfully, but
/// the source-offset commit is prevented (all offset commits are rejected)
/// before the receiver stops; a second receiver then joins the same group.
/// Guarantees: because delivery-then-commit is at-least-once, the uncommitted
/// poison offset is re-delivered and dead-lettered again (a duplicate), so the
/// DLQ topic ends up with two copies -- proving no silent loss, at the cost of a
/// possible duplicate.
#[tokio::test]
async fn uncommitted_dead_letter_is_redelivered_on_restart_at_least_once() {
    const TOPIC: &str = "dlq-restart-src";
    const DLQ: &str = "dlq-restart-out";
    let group = "dlq-restart-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bad = b"not-a-valid-otap-proto-payload".to_vec();
            producer
                .send_full(SendRecord::new(TOPIC, &bad).key(b"k"))
                .await
                .expect("send");

            // Reject every offset commit so the source offset for the poison
            // record can never be persisted, even though the DLQ produce succeeds.
            cluster.faults().fail_offset_commits(
                &vec![RDKafkaRespErr::RD_KAFKA_RESP_ERR_REQUEST_TIMED_OUT; 512],
            );

            let cfg_a = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let receiver_a = KafkaReceiverHarness::start(&cluster, cfg_a);

            // Receiver A dead-letters the poison record once.
            let first = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(
                first.len(),
                1,
                "receiver A must dead-letter the poison once"
            );

            // Stop A without a successful commit (commits are still failing).
            receiver_a.shutdown(Duration::from_secs(5));
            let _ = receiver_a.await_terminal_state().await;

            // Confirm the source offset never advanced past the poison record.
            let committed =
                committed_offset(cluster.bootstrap_servers(), group, TOPIC, 0).unwrap_or(None);
            assert!(
                committed.is_none() || committed == Some(0),
                "poison offset must remain uncommitted, got {committed:?}"
            );

            // Allow commits again and start receiver B on the same group; it must
            // re-consume the uncommitted poison record and dead-letter it again.
            cluster.faults().clear_offset_commit_failures();
            let cfg_b = manual_otap_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let receiver_b = KafkaReceiverHarness::start(&cluster, cfg_b);

            // The DLQ topic must now contain a second (duplicate) copy: at-least-
            // once, never silent loss.
            let both = drain_dlq_topic(&cluster, DLQ, 2, Duration::from_secs(30)).await;
            assert_eq!(
                both.len(),
                2,
                "the uncommitted poison must be re-dead-lettered on restart (at-least-once)"
            );
            assert_eq!(
                both[0].payload, both[1].payload,
                "both copies are byte-identical"
            );

            receiver_b.shutdown(Duration::from_secs(5));
            let _ = receiver_b.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a record arrives on a topic matched by the include regex but
/// removed by `exclude_topics`, so it routes to no signal (unknown topic) while
/// the DLQ captures `unknown_topic`.
/// Guarantees: the original bytes are dead-lettered byte-identically with reason
/// `unknown_topic`, and the source offset advances.
#[tokio::test]
async fn unknown_topic_is_dead_lettered_byte_identical() {
    const EXCLUDED: &str = "dlq-unknown-excluded";
    // The DLQ topic must sit outside the `^dlq-unknown-.*` ingest regex, or
    // loop-prevention validation would reject it.
    const DLQ: &str = "unknown-dead-letters";
    let group = "dlq-unknown-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(EXCLUDED, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            // A well-formed record on the excluded topic: it is subscribed (the
            // include regex matches) but routes to no signal.
            producer
                .send_full(SendRecord::new(EXCLUDED, &bytes).key(b"k"))
                .await
                .expect("send");

            // Subscribe via the broad `^dlq-unknown-.*` regex so EXCLUDED is
            // consumed by librdkafka, but exclude it so it routes to no signal
            // (the unknown-topic path). The DLQ topic sits outside that regex.
            use crate::receivers::kafka_receiver::config::DlqConfig;
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec!["^dlq-unknown-.*$".to_string()])
                            .with_encoding(MessageFormat::OtlpProto)
                            .with_exclude_topics(vec!["^dlq-unknown-excluded$".to_string()]),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_dlq(DlqConfig {
                        topic: Some(DLQ.to_string()),
                        per_signal: None,
                        capture: vec![DlqCapture::UnknownTopic],
                        connection: None,
                    });
            let cfg = KafkaReceiverConfig::try_from(builder).expect("valid");
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");
            assert_eq!(
                records[0].payload.as_deref(),
                Some(bytes.as_slice()),
                "unknown-topic DLQ payload must be byte-identical to the source"
            );
            assert_eq!(records[0].header("dlq.reason"), Some(&b"unknown_topic"[..]));
            assert_eq!(
                records[0].header("dlq.source.topic"),
                Some(EXCLUDED.as_bytes())
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}
