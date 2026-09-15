// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receive-loop integration tests: end-to-end consume and topic routing per signal and encoding.

use super::*;

/// Scenario (routing and payload correctness): OTLP-proto trace records produced to a Kafka topic are consumed
/// by an auto-commit receiver.
/// Guarantees: each delivered pdata decodes to an `ExportTracesRequest` whose
/// bytes are byte-for-byte identical to what was produced (lossless round-trip).
#[tokio::test]
async fn test_kafka_receiver_traces() {
    const TOPIC: &str = "test-traces-proto";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..3 {
                let key = format!("test-key-{i}");
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

            for _ in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
                assert_eq!(proto.as_bytes(), &bytes);
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): OTLP-proto log records produced to a Kafka topic are consumed
/// by an auto-commit receiver.
/// Guarantees: each delivered pdata decodes to an `ExportLogsRequest` whose
/// bytes are byte-for-byte identical to what was produced.
#[tokio::test]
async fn test_kafka_receiver_logs() {
    const TOPIC: &str = "test-logs-proto";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_logs_service_request();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[],
                &[],
                &[TOPIC],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for _ in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                assert!(matches!(proto, OtlpProtoBytes::ExportLogsRequest(_)));
                assert_eq!(proto.as_bytes(), &bytes);
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): OTLP-proto metric records produced to a Kafka topic are consumed
/// by an auto-commit receiver.
/// Guarantees: each delivered pdata decodes to an `ExportMetricsRequest` whose
/// bytes are byte-for-byte identical to what was produced.
#[tokio::test]
async fn test_kafka_receiver_metrics() {
    const TOPIC: &str = "test-metrics-proto";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let req = create_metrics_service_request();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");

            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[],
                &[TOPIC],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for _ in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                assert!(matches!(proto, OtlpProtoBytes::ExportMetricsRequest(_)));
                assert_eq!(proto.as_bytes(), &bytes);
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): OTAP-Arrow trace records produced to a Kafka topic are consumed
/// by an auto-commit receiver configured for the OTAP format.
/// Guarantees: each delivered pdata is an `OtapArrowRecords::Traces` payload.
#[tokio::test]
async fn test_kafka_receiver_traces_otap() {
    const TOPIC: &str = "test-traces-otap";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let bytes = create_traces_with_spans_otap_bytes();

            for i in 0..3 {
                let key = format!("test-key-{i}");
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
                MessageFormat::OtapProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let payload: OtapPayload = pdata.take_payload();
                assert!(
                    matches!(
                        payload.into_data(),
                        PayloadData::OtapArrowRecords(OtapArrowRecords::Traces(_))
                    ),
                    "Expected OtapArrowRecords::Traces for message {i}"
                );
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): OTAP-Arrow metric records produced to a Kafka topic are consumed
/// by an auto-commit receiver configured for the OTAP format.
/// Guarantees: each delivered pdata is an `OtapArrowRecords::Metrics` payload
/// equal to the produced default metrics records.
#[tokio::test]
async fn test_kafka_receiver_metrics_otap() {
    const TOPIC: &str = "test-metrics-otap";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let bytes = create_metrics_otap_arrow_records_bytes();

            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[],
                &[TOPIC],
                &[],
                MessageFormat::OtapProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let payload: OtapPayload = pdata.take_payload();
                if let PayloadData::OtapArrowRecords(arrow_records) = payload.into_data() {
                    let expected = OtapArrowRecords::Metrics(Metrics::default());
                    assert_eq!(expected, arrow_records);
                } else {
                    panic!("Expected OtapArrowRecords::Metrics for message {i}");
                }
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): OTAP-Arrow log records produced to a Kafka topic are consumed
/// by an auto-commit receiver configured for the OTAP format.
/// Guarantees: each delivered pdata is an `OtapArrowRecords::Logs` payload
/// equal to the produced default logs records.
#[tokio::test]
async fn test_kafka_receiver_logs_otap() {
    const TOPIC: &str = "test-logs-otap";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let bytes = create_logs_otap_arrow_records_bytes();

            for i in 0..3 {
                let key = format!("test-key-{i}");
                producer
                    .send_full(SendRecord::new(TOPIC, &bytes).key(key.as_bytes()))
                    .await
                    .expect("Failed to send message");
            }

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[],
                &[],
                &[TOPIC],
                MessageFormat::OtapProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let payload: OtapPayload = pdata.take_payload();
                if let PayloadData::OtapArrowRecords(arrow_records) = payload.into_data() {
                    let expected = OtapArrowRecords::Logs(Logs::default());
                    assert_eq!(expected, arrow_records);
                } else {
                    panic!("Expected OtapArrowRecords::Logs for message {i}");
                }
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): the receiver's traces signal is configured with the default
/// per-signal encoding `OtlpProto`, but a record is produced with OTAP-Arrow
/// payload bytes plus a per-message `MessageFormat: otap` Kafka header.
/// Guarantees: closes the Area 6 "per-message header override" subtask -- the
/// `MessageFormat` header overrides the per-signal `OtlpProto` default so the
/// receiver decodes the payload via the OTAP path (the delivered pdata is an
/// `OtapArrowRecords::Traces`, which is only possible if the override took
/// effect; had the header been ignored, the OTAP bytes would be mis-handled as
/// OtlpProto). Protects `detect_message_format` (`receiver.rs:115`) and its use
/// on the per-signal decode path.
#[tokio::test]
async fn test_kafka_receiver_message_format_header_overrides_signal_default() {
    const TOPIC: &str = "test-format-override";
    with_cluster(
        KafkaTestCluster::builder().topic(TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            // OTAP-Arrow wire bytes, but the per-signal default below is OTLP.
            let otap_bytes = create_traces_with_spans_otap_bytes();

            // Produce with the per-message MessageFormat=otap header so the
            // receiver must override its OtlpProto per-signal default.
            for i in 0..3 {
                let key = format!("override-key-{i}");
                producer
                    .send_full(
                        SendRecord::new(TOPIC, &otap_bytes)
                            .key(key.as_bytes())
                            .header("MessageFormat", MSG_FORMAT_OTAP),
                    )
                    .await
                    .expect("Failed to send message");
            }

            // Per-signal traces encoding is deliberately OtlpProto (the
            // default); only the per-message header should switch it to OTAP.
            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TOPIC],
                &[],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for i in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let payload: OtapPayload = pdata.take_payload();
                assert!(
                    matches!(
                        payload.into_data(),
                        PayloadData::OtapArrowRecords(OtapArrowRecords::Traces(_))
                    ),
                    "message {i}: MessageFormat=otap header must override the \
                         OtlpProto per-signal default and decode via the OTAP path",
                );
            }

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a single receiver subscribes
/// simultaneously to a distinct traces topic, metrics topic, and logs topic
/// (disjoint across signals), and one OTLP-proto record is produced to each.
/// Guarantees: each record is routed to the decoder for its own signal --
/// the traces topic yields an `ExportTracesRequest`, the metrics topic an
/// `ExportMetricsRequest`, and the logs topic an `ExportLogsRequest` -- so
/// concurrent multi-signal topic routing dispatches every topic to the
/// correct signal without cross-contamination.
#[tokio::test]
async fn multi_signal_topics_route_to_correct_decoders() {
    const TRACES_TOPIC: &str = "route-multi-traces";
    const METRICS_TOPIC: &str = "route-multi-metrics";
    const LOGS_TOPIC: &str = "route-multi-logs";
    with_cluster(
        KafkaTestCluster::builder()
            .topic(TRACES_TOPIC)
            .topic(METRICS_TOPIC)
            .topic(LOGS_TOPIC),
        |cluster| async move {
            let producer = cluster.producer().build();

            let traces_req = create_traces_with_spans();
            let mut traces_bytes = vec![];
            traces_req.encode(&mut traces_bytes).expect("encode traces");
            let metrics_req = create_metrics_service_request();
            let mut metrics_bytes = vec![];
            metrics_req
                .encode(&mut metrics_bytes)
                .expect("encode metrics");
            let logs_req = create_logs_service_request();
            let mut logs_bytes = vec![];
            logs_req.encode(&mut logs_bytes).expect("encode logs");

            producer
                .send_full(SendRecord::new(TRACES_TOPIC, &traces_bytes).key(b"t"))
                .await
                .expect("send traces");
            producer
                .send_full(SendRecord::new(METRICS_TOPIC, &metrics_bytes).key(b"m"))
                .await
                .expect("send metrics");
            producer
                .send_full(SendRecord::new(LOGS_TOPIC, &logs_bytes).key(b"l"))
                .await
                .expect("send logs");

            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &[TRACES_TOPIC],
                &[METRICS_TOPIC],
                &[LOGS_TOPIC],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Records may arrive in any order; classify each by its decoded
            // signal type and assert all three signals are represented.
            let mut saw_traces = false;
            let mut saw_metrics = false;
            let mut saw_logs = false;
            for _ in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                match proto {
                    OtlpProtoBytes::ExportTracesRequest(ref b) => {
                        assert_eq!(b.as_ref(), &traces_bytes, "traces payload preserved");
                        saw_traces = true;
                    }
                    OtlpProtoBytes::ExportMetricsRequest(ref b) => {
                        assert_eq!(b.as_ref(), &metrics_bytes, "metrics payload preserved");
                        saw_metrics = true;
                    }
                    OtlpProtoBytes::ExportLogsRequest(ref b) => {
                        assert_eq!(b.as_ref(), &logs_bytes, "logs payload preserved");
                        saw_logs = true;
                    }
                }
            }
            assert!(
                saw_traces && saw_metrics && saw_logs,
                "each signal topic must route to its own decoder \
                     (traces={saw_traces}, metrics={saw_metrics}, logs={saw_logs})",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a receiver's traces signal is
/// configured with a single `^`-prefixed regex subscription (`^route-regex-.*`)
/// and records are produced to three independently-created broker topics
/// that all match the pattern.
/// Guarantees: the receiver consumes records from every topic matching the
/// regex subscription -- not just a literal topic name -- so pattern-based
/// subscription delivers from all matching topics.
#[tokio::test]
async fn regex_topic_subscription_consumes_all_matching_topics() {
    const TOPIC_A: &str = "route-regex-alpha";
    const TOPIC_B: &str = "route-regex-beta";
    const TOPIC_C: &str = "route-regex-gamma";
    with_cluster(
        KafkaTestCluster::builder()
            .topic(TOPIC_A)
            .topic(TOPIC_B)
            .topic(TOPIC_C),
        |cluster| async move {
            let producer = cluster.producer().build();
            let req = create_traces_with_spans();
            let mut bytes = vec![];
            req.encode(&mut bytes).expect("encode");
            for topic in [TOPIC_A, TOPIC_B, TOPIC_C] {
                producer
                    .send_full(SendRecord::new(topic, &bytes).key(topic.as_bytes()))
                    .await
                    .unwrap_or_else(|e| panic!("send to {topic}: {e}"));
            }

            // Single regex subscription that matches all three topics.
            let cfg = auto_config(
                cluster.bootstrap_servers(),
                &["^route-regex-.*"],
                &[],
                &[],
                MessageFormat::OtlpProto,
                HashMap::new(),
            );
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            // Correlate delivered records back to their source topic via the
            // stamped calldata topic-id is not name-resolvable here, so
            // instead assert that exactly three records (one per matching
            // topic) are delivered and their payloads round-trip.
            let mut delivered = 0;
            for _ in 0..3 {
                let mut pdata = receiver.recv_pdata().await;
                let proto: OtlpProtoBytes = pdata
                    .take_payload()
                    .try_into_with_default()
                    .expect("to OtlpProtoBytes");
                assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
                assert_eq!(proto.as_bytes(), &bytes, "payload preserved");
                delivered += 1;
            }
            assert_eq!(
                delivered, 3,
                "regex subscription must consume from all three matching topics",
            );
            // No fourth record exists: the pattern matched exactly the three
            // produced topics.
            assert!(
                receiver
                    .try_recv_pdata(Duration::from_secs(2))
                    .await
                    .is_none(),
                "no extra records beyond the three matching topics",
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}

/// Scenario (routing and payload correctness): a manual-commit receiver
/// configured with `isolation_level: read_committed` consumes records
/// produced (non-transactionally) to its topic.
/// Guarantees: the receiver still delivers every record and commits the full
/// count under the read-committed isolation level -- so selecting
/// read-committed does not break ordinary (non-transactional) consumption.
#[tokio::test]
async fn read_committed_isolation_delivers_and_commits() {
    const TOPIC: &str = "route-readcommitted-traces";
    const RECORDS: i64 = 3;
    let group = "route-readcommitted-group";
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
                    .with_isolation_level(IsolationLevel::ReadCommitted);
            let cfg = KafkaReceiverConfig::try_from(builder).expect("test config valid");
            let mut receiver = KafkaReceiverHarness::start(&cluster, cfg);

            for _ in 0..RECORDS {
                let pdata = receiver.recv_pdata().await;
                receiver.ack(pdata);
            }

            let brokers = cluster.bootstrap_servers().to_string();
            let committed = poll_until(Duration::from_secs(5), Duration::from_millis(150), || {
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed")
                    .is_some_and(|o| o >= RECORDS)
            })
            .await;
            assert!(
                committed,
                "read_committed receiver must deliver and commit all {RECORDS} \
                     non-transactional records, got {:?}",
                committed_offset(&brokers, group, TOPIC, 0)
                    .expect("kafka-test: committed-offset probe failed"),
            );

            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
