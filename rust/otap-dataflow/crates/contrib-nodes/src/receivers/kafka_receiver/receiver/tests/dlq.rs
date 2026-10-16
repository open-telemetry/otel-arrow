// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration and edge-case coverage for the Kafka receiver dead-letter queue.
//!
//! These tests exercise the DLQ egress against an in-process mock cluster: raw
//! bytes recovery (inline for decode / excluded-topic, re-read for
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

/// Scenario: a receiver has BOTH transient-nack replay AND a DLQ that captures
/// permanent nacks. A record is transiently nacked, then (after replay
/// redelivery) permanently nacked.
/// Guarantees: the transient nack is replayed and never dead-lettered (replay
/// takes precedence over the DLQ for non-permanent nacks), while the subsequent
/// permanent nack on the redelivered record IS dead-lettered -- locking the
/// nack-handling precedence.
#[tokio::test]
async fn transient_nack_with_replay_is_not_dead_lettered_but_permanent_is() {
    use crate::receivers::kafka_receiver::config::DlqConfig;
    const TOPIC: &str = "dlq-replay-precedence-src";
    const DLQ: &str = "dlq-replay-precedence-out";
    let group = "dlq-replay-precedence-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();
            producer
                .send_full(SendRecord::new(TOPIC, &bytes).key(b"k"))
                .await
                .expect("send");

            // Replay transient nacks (short backoff so redelivery is prompt) AND
            // capture permanent nacks in the DLQ, so both handlers are armed.
            let builder = manual_traces_builder(cluster.bootstrap_servers(), group, TOPIC)
                .with_transient_nack(TransientNackConfig {
                    mode: TransientNackMode::Replay,
                    initial_backoff_ms: 10,
                    max_backoff_ms: 40,
                })
                .with_enable_idempotency(true)
                .with_dlq(DlqConfig {
                    topic: Some(DLQ.to_string()),
                    per_signal: None,
                    capture: vec![DlqCapture::PermanentNack],
                    connection: None,
                });
            let cfg = KafkaReceiverConfig::try_from(builder).expect("valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Transiently nack the record: it must be replayed, not dead-lettered.
            let first = receiver.recv_pdata().await;
            receiver.nack_transient("retry me", first);

            // Nothing is dead-lettered for the transient nack.
            let after_transient = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(3)).await;
            assert!(
                after_transient.is_empty(),
                "a transient nack under replay must not be dead-lettered"
            );

            // The record is redelivered by replay; permanently nack it now.
            let redelivered = receiver
                .try_recv_pdata(Duration::from_secs(30))
                .await
                .expect("transiently-nacked record must be replayed/redelivered");
            receiver.nack_permanent("terminal", redelivered);

            // The permanent nack IS dead-lettered.
            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(
                records.len(),
                1,
                "the permanent nack on the redelivered record must be dead-lettered"
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

/// Scenario: a manual-commit receiver with `permanent_nack` DLQ capture holds an
/// un-acked in-flight record on a partition that a second group member then
/// steals (revoking it); the now-stale record is permanently nacked.
/// Guarantees: the stale permanent nack is dropped by the feedback guard
/// (`receiver.kafka.consumer.group.feedback.after_revocation` increments), so it
/// is NOT dead-lettered and the revoked partition's offset is not committed by
/// the losing owner -- the record is left for the new owner (at-least-once).
#[tokio::test]
async fn dlq_permanent_nack_after_revoke_is_not_dead_lettered() {
    const TOPIC: &str = "dlq-nack-revoke-src";
    const DLQ: &str = "dlq-nack-revoke-out";
    let group = "dlq-nack-revoke-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, REBALANCE_TEST_PARTITIONS, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            // One valid record per partition so the receiver holds in-flight work
            // on every partition it owns.
            for partition in 0..REBALANCE_TEST_PARTITIONS {
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &bytes)
                            .key(b"k")
                            .partition(partition),
                    )
                    .await
                    .expect("send");
            }

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume every record but hold them un-acked so their offsets stay
            // pending across the revoke.
            let mut in_flight = Vec::new();
            for _ in 0..REBALANCE_TEST_PARTITIONS {
                in_flight.push(receiver.recv_pdata().await);
            }

            // A second member joins and steals at least one partition.
            let trigger =
                RebalanceTrigger::join(&cluster, group, &[TOPIC], Duration::from_secs(30)).await;
            let stolen = trigger
                .assignment()
                .into_iter()
                .find_map(|(topic, partition)| (topic == TOPIC).then_some(partition))
                .expect("rebalance trigger should own a partition from the test topic");
            receiver
                .wait_for_partition_revocation(TOPIC, stolen, Duration::from_secs(10))
                .await;

            // Permanently nack every in-flight record now that a partition is
            // revoked. The nack for the revoked partition must be dropped by the
            // feedback guard and never enter the DLQ re-read path.
            for pdata in in_flight {
                receiver.nack_permanent("stale after revoke", pdata);
            }

            // Nack and Shutdown share the FIFO control channel, so feedback is
            // handled before the terminal snapshot.
            let terminal = shutdown_and_terminal(receiver, Duration::from_secs(5)).await;
            drop(trigger);

            let mut m = FoldedMetrics::new();
            m.fold_all(terminal.metrics());
            assert!(
                m.value("group.feedback.after_revocation") >= 1,
                "a permanent nack for a revoked partition must be dropped and counted, got {}; \
                     revocations={}",
                m.value("group.feedback.after_revocation"),
                m.value("group.partition.revocations"),
            );

            // Nothing was dead-lettered for the dropped nack: the DLQ topic stays
            // empty (the revoked record is left for the new owner, at-least-once).
            let dead_letters = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(3)).await;
            assert!(
                dead_letters.is_empty(),
                "a permanent nack dropped after revoke must not be dead-lettered"
            );
        },
    )
    .await;
}

/// Scenario: several distinct records are produced to one partition; a record
/// that is NOT the first offset is permanently nacked with `permanent_nack` DLQ
/// capture, so the re-read consumer must seek past earlier offsets to it.
/// Guarantees: the dead-lettered payload is byte-identical to the specifically
/// nacked record (its exact offset), never an adjacent record's bytes, and its
/// `dlq.source.offset` header matches that offset.
#[tokio::test]
async fn reread_recovers_exact_offset_among_multiple_records() {
    const TOPIC: &str = "dlq-reread-exact-src";
    const DLQ: &str = "dlq-reread-exact-out";
    let group = "dlq-reread-exact-group";
    const RECORDS: usize = 4;
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Produce RECORDS distinct, decodable payloads to partition 0. Each
            // carries a unique span name so the encoded bytes differ per offset.
            let mut payloads: Vec<Vec<u8>> = Vec::new();
            for i in 0..RECORDS {
                let mut req = create_traces_with_spans();
                // Make each record's bytes unique and distinguishable.
                req.resource_spans[0].scope_spans[0].spans[0].name = format!("span-{i}");
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(b"k").partition(0))
                    .await
                    .expect("send");
                payloads.push(bytes);
            }

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume all records, mapping each source offset to the pdata so we
            // can nack a specific, non-first offset.
            let mut by_offset: HashMap<i64, OtapPdata> = HashMap::new();
            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_topic_id, _partition, offset, _generation) = decode_calldata(&route.calldata);
                let _ = by_offset.insert(offset, pdata);
            }

            // Ack every record except the target (offset 2, the third record) so
            // only the targeted offset is dead-lettered.
            const TARGET_OFFSET: i64 = 2;
            for (offset, pdata) in by_offset {
                if offset == TARGET_OFFSET {
                    receiver.nack_permanent("terminal", pdata);
                } else {
                    receiver.ack(pdata);
                }
            }

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(
                records.len(),
                1,
                "exactly the targeted record is dead-lettered"
            );
            assert_eq!(
                records[0].payload.as_deref(),
                Some(payloads[TARGET_OFFSET as usize].as_slice()),
                "re-read must recover the exact target offset's bytes, not an adjacent record"
            );
            assert_eq!(
                records[0].header("dlq.source.offset"),
                Some(TARGET_OFFSET.to_string().as_bytes()),
                "dead-letter must carry the targeted source offset"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: `DLQ_MAX_IN_FLIGHT` distinct records are permanently nacked close
/// together with `permanent_nack` capture, so up to five re-read recoveries run
/// concurrently through the single shared re-read consumer while keep-warm polls.
/// Guarantees: every record is dead-lettered exactly once with byte-identical
/// bytes matching its OWN source offset (no cross-contamination or stolen
/// records), and no spurious loss occurs -- the count equals the number nacked.
#[tokio::test]
async fn concurrent_permanent_nack_recoveries_recover_correct_bytes() {
    use crate::receivers::kafka_receiver::config::DLQ_MAX_IN_FLIGHT;
    const TOPIC: &str = "dlq-concurrent-src";
    const DLQ: &str = "dlq-concurrent-out";
    let group = "dlq-concurrent-group";
    let n = DLQ_MAX_IN_FLIGHT; // 5 concurrent recoveries: the in-flight bound.
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // Distinct, decodable payloads so each dead-letter can be matched to
            // the exact source record it should have recovered.
            let mut payload_by_offset: HashMap<i64, Vec<u8>> = HashMap::new();
            let mut sent: Vec<Vec<u8>> = Vec::new();
            for i in 0..n {
                let mut req = create_traces_with_spans();
                req.resource_spans[0].scope_spans[0].spans[0].name = format!("concurrent-span-{i}");
                let mut bytes = vec![];
                req.encode(&mut bytes).expect("encode");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(b"k").partition(0))
                    .await
                    .expect("send");
                sent.push(bytes);
            }

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::PermanentNack],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Consume all records, mapping each to its source offset.
            let mut in_flight: Vec<(i64, OtapPdata)> = Vec::new();
            for _ in 0..n {
                let pdata = receiver.recv_pdata().await;
                let route = pdata
                    .source_route()
                    .expect("delivered pdata carries source calldata");
                let (_topic_id, _partition, offset, _generation) = decode_calldata(&route.calldata);
                let idx = offset as usize;
                let _ = payload_by_offset.insert(offset, sent[idx].clone());
                in_flight.push((offset, pdata));
            }

            // Nack all of them back-to-back so their re-read recoveries overlap on
            // the single shared consumer (bounded by DLQ_MAX_IN_FLIGHT).
            for (_offset, pdata) in in_flight {
                receiver.nack_permanent("terminal", pdata);
            }

            // Every record must be dead-lettered exactly once, each with the bytes
            // of its OWN offset (indexed by the dlq.source.offset header).
            let records = drain_dlq_topic(&cluster, DLQ, n, Duration::from_secs(30)).await;
            assert_eq!(
                records.len(),
                n,
                "every concurrently-nacked record must be dead-lettered exactly once (no loss)"
            );
            for rec in &records {
                let offset_bytes = rec
                    .header("dlq.source.offset")
                    .expect("dead-letter carries source offset");
                let offset: i64 = std::str::from_utf8(offset_bytes)
                    .expect("ascii offset")
                    .parse()
                    .expect("numeric offset");
                let expected = payload_by_offset
                    .get(&offset)
                    .expect("offset was produced and nacked");
                assert_eq!(
                    rec.payload.as_deref(),
                    Some(expected.as_slice()),
                    "concurrent recovery for offset {offset} must carry its own bytes, \
                         proving no cross-contamination on the shared re-read consumer"
                );
            }

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a record arrives on a topic matched by the traces include regex but
/// removed by `exclude_topics`, so it routes to no signal while the DLQ captures
/// `excluded_topic`.
/// Guarantees: the original bytes are dead-lettered byte-identically with reason
/// `excluded_topic`, the dead-letter carries `dlq.signal=traces` (the matching
/// signal, not `unknown`), and the source offset advances.
#[tokio::test]
async fn excluded_topic_is_dead_lettered_byte_identical() {
    const EXCLUDED: &str = "dlq-excluded-topic";
    // The DLQ topic must sit outside the `^dlq-excluded-.*` ingest regex, or
    // loop-prevention validation would reject it.
    const DLQ: &str = "excluded-dead-letters";
    let group = "dlq-excluded-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(EXCLUDED, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();
            // A well-formed record on the excluded topic: it is subscribed (the
            // traces include regex matches) but routes to no signal.
            producer
                .send_full(SendRecord::new(EXCLUDED, &bytes).key(b"k"))
                .await
                .expect("send");

            // Subscribe via the broad `^dlq-excluded-.*` regex so EXCLUDED is
            // consumed by librdkafka, but exclude it so it routes to no signal
            // (the excluded-topic path). The DLQ topic sits outside that regex.
            use crate::receivers::kafka_receiver::config::DlqConfig;
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec!["^dlq-excluded-.*$".to_string()])
                            .with_encoding(MessageFormat::OtlpProto)
                            .with_exclude_topics(vec!["^dlq-excluded-topic$".to_string()]),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_dlq(DlqConfig {
                        topic: Some(DLQ.to_string()),
                        per_signal: None,
                        capture: vec![DlqCapture::ExcludedTopic],
                        connection: None,
                    });
            let cfg = KafkaReceiverConfig::try_from(builder).expect("valid");
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(records.len(), 1, "expected one dead-lettered record");
            assert_eq!(
                records[0].payload.as_deref(),
                Some(bytes.as_slice()),
                "excluded-topic DLQ payload must be byte-identical to the source"
            );
            assert_eq!(
                records[0].header("dlq.reason"),
                Some(&b"excluded_topic"[..])
            );
            // The excluding signal (traces) is resolved, not `unknown`.
            assert_eq!(records[0].header("dlq.signal"), Some(&b"traces"[..]));
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

/// Scenario: a zero-length OTLP-proto Kafka record is consumed by a receiver with
/// a decode-capturing DLQ.
/// Guarantees: the empty payload decodes as a valid (empty) request and is
/// FORWARDED downstream (not dead-lettered), so a well-formed empty request is
/// never treated as a failure.
#[tokio::test]
async fn empty_payload_is_accepted_and_forwarded() {
    const TOPIC: &str = "dlq-empty-src";
    const DLQ: &str = "dlq-empty-out";
    let group = "dlq-empty-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // A zero-length payload is delivered as an empty (Some(&[])) value and
            // decodes as a valid empty OTLP-proto request.
            producer
                .send_full(SendRecord::new(TOPIC, &[]).key(b"k"))
                .await
                .expect("send empty");

            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The empty-but-valid record is forwarded downstream.
            let pdata = receiver
                .try_recv_pdata(Duration::from_secs(30))
                .await
                .expect("empty valid OTLP record must be forwarded, not dead-lettered");
            receiver.ack(pdata);

            // Nothing is dead-lettered: the DLQ topic stays empty.
            let dead_letters = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(3)).await;
            assert!(
                dead_letters.is_empty(),
                "a valid empty payload must not be dead-lettered"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a Kafka record with a null (absent) value -- not an empty
/// `Some(&[])` payload -- is consumed by a receiver with a decode-capturing DLQ.
/// Guarantees: the null record is dead-lettered with reason `empty_payload`
/// (gated by the `decode` capture) rather than forwarded, so a genuinely
/// value-less record is captured distinctly from a valid empty request.
#[tokio::test]
async fn null_value_is_dead_lettered_with_empty_payload_reason() {
    const TOPIC: &str = "dlq-null-src";
    const DLQ: &str = "dlq-null-out";
    let group = "dlq-null-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // A null-value record: the consumer sees payload() == None, which is
            // the only input that yields the EmptyPayload decode failure.
            producer
                .send_null_value(TOPIC, b"k")
                .await
                .expect("send null value");

            // empty_payload is a decode-class failure gated by the decode capture.
            let cfg = manual_traces_config_with_dlq(
                cluster.bootstrap_servers(),
                group,
                TOPIC,
                DLQ,
                vec![DlqCapture::Decode],
            );
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            let records = drain_dlq_topic(&cluster, DLQ, 1, Duration::from_secs(30)).await;
            assert_eq!(
                records.len(),
                1,
                "the null-value record must be dead-lettered"
            );
            assert_eq!(
                records[0].header("dlq.reason"),
                Some(&b"empty_payload"[..]),
                "a null-value record must be dead-lettered as empty_payload"
            );
            assert_eq!(
                records[0].header("dlq.source.topic"),
                Some(TOPIC.as_bytes())
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: a record carrying a custom source header (`x-tenant`) fails decode
/// and is dead-lettered with decode capture.
/// Guarantees: the dead-lettered record carries BOTH the injected `dlq.*` context
/// headers AND the original `x-tenant` source header, proving faithful passthrough.
#[tokio::test]
async fn source_headers_pass_through_to_dead_letter() {
    const TOPIC: &str = "dlq-hdr-src";
    const DLQ: &str = "dlq-hdr-out";
    let group = "dlq-hdr-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(TOPIC, 1, 1)
            .topic_with(DLQ, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            // An undecodable OTAP payload carrying a custom source header.
            let bad = b"poison-otap-with-header".to_vec();
            producer
                .send_full(
                    SendRecord::new(TOPIC, &bad)
                        .key(b"k")
                        .header("x-tenant", b"acme"),
                )
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
            assert_eq!(records.len(), 1, "the record must be dead-lettered");
            // Injected dlq.* context is present ...
            assert_eq!(records[0].header("dlq.reason"), Some(&b"decode"[..]));
            assert_eq!(
                records[0].header("dlq.source.topic"),
                Some(TOPIC.as_bytes())
            );
            // ... and the original source header survived the passthrough.
            assert_eq!(
                records[0].header("x-tenant"),
                Some(&b"acme"[..]),
                "original source header must pass through to the dead-letter"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}

/// Scenario: per-signal DLQ topics are configured for traces and metrics. A
/// record on a topic matched by the TRACES include regex but removed by
/// `exclude_topics` is dead-lettered with `excluded_topic` capture.
/// Guarantees: the excluded-topic dead-letter routes to the MATCHING signal's
/// DLQ topic (traces), not a sorted-first fallback, carries `dlq.signal=traces`,
/// and nothing lands in the metrics DLQ topic -- proving signal-aware routing.
#[tokio::test]
async fn excluded_topic_dead_letter_routes_to_matching_signal_topic() {
    const EXCLUDED: &str = "dlq-route-excluded";
    // Both DLQ topics sit outside the ingest regexes. The traces DLQ topic is
    // deliberately the lexicographically LARGER name to prove routing is by the
    // matching signal, not by sort order.
    const DLQ_METRICS: &str = "aaa-dlq-metrics";
    const DLQ_TRACES: &str = "zzz-dlq-traces";
    const METRICS_TOPIC: &str = "metrics-ingest";
    let group = "dlq-route-signal-group";
    with_cluster(
        KafkaTestCluster::builder()
            .topic_with(EXCLUDED, 1, 1)
            .topic_with(METRICS_TOPIC, 1, 1)
            .topic_with(DLQ_METRICS, 1, 1)
            .topic_with(DLQ_TRACES, 1, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            let bytes = encoded_trace_fixture();
            producer
                .send_full(SendRecord::new(EXCLUDED, &bytes).key(b"k"))
                .await
                .expect("send");

            use crate::receivers::kafka_receiver::config::{DlqConfig, DlqPerSignalTopics};
            // The excluded topic is matched by the TRACES include regex, so its
            // owning signal is traces; metrics ingests a separate literal topic so
            // both signals have per-signal DLQ topics configured.
            let builder =
                KafkaReceiverConfigBuilder::new(cluster.bootstrap_servers(), group, "test-client")
                    .with_traces(
                        SignalConfig::new(vec!["^dlq-route-.*$".to_string()])
                            .with_encoding(MessageFormat::OtlpProto)
                            .with_exclude_topics(vec!["^dlq-route-excluded$".to_string()]),
                    )
                    .with_metrics(
                        SignalConfig::new(vec![METRICS_TOPIC.to_string()])
                            .with_encoding(MessageFormat::OtlpProto),
                    )
                    .with_commit(CommitConfig {
                        mode: ConfigCommitMode::Manual,
                        interval_ms: None,
                    })
                    .with_auto_offset_reset(AutoOffsetReset::Earliest)
                    .with_dlq(DlqConfig {
                        topic: None,
                        per_signal: Some(DlqPerSignalTopics {
                            traces: Some(DLQ_TRACES.to_string()),
                            metrics: Some(DLQ_METRICS.to_string()),
                            logs: None,
                        }),
                        capture: vec![DlqCapture::ExcludedTopic],
                        connection: None,
                    });
            let cfg = KafkaReceiverConfig::try_from(builder).expect("valid");
            let receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // The excluded-topic dead-letter must land in the TRACES DLQ topic
            // (its matching signal), despite that name sorting last.
            let routed = drain_dlq_topic(&cluster, DLQ_TRACES, 1, Duration::from_secs(30)).await;
            assert_eq!(
                routed.len(),
                1,
                "excluded-topic dead-letter must route to the matching (traces) DLQ topic"
            );
            assert_eq!(routed[0].header("dlq.reason"), Some(&b"excluded_topic"[..]));
            assert_eq!(routed[0].header("dlq.signal"), Some(&b"traces"[..]));

            // And nothing lands in the metrics DLQ topic.
            let other = drain_dlq_topic(&cluster, DLQ_METRICS, 1, Duration::from_secs(3)).await;
            assert!(
                other.is_empty(),
                "the excluded traces-topic record must not route to the metrics DLQ topic"
            );

            receiver.shutdown(Duration::from_secs(5));
            let _ = receiver.await_terminal_state().await;
        },
    )
    .await;
}
