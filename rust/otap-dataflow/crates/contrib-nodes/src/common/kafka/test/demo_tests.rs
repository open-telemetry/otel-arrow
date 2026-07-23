// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Capability-demo tests that exercise the test suite directly with plain
//! producers/consumers (no node), proving the suite is robust and documenting
//! its usage. Demo 10 (wrapper smoke) is feature-gated and lives beside the
//! wrappers' own concerns.

use std::time::Duration;

use rdkafka::types::{RDKafkaApiKey, RDKafkaRespErr};

use super::cluster::KafkaTestCluster;
use super::consumer::committed_offset;
use super::message::count_by_partition;
use super::wait::{poll_until, poll_until_async};
use super::with_cluster;

// Wrapper-smoke-test dependencies, grouped by the node feature that gates them
// so each feature carries a single `#[cfg]` on its import group rather than one
// tag per line.
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
use common_wrapper_deps::*;
#[cfg(feature = "kafka-exporter")]
use exporter_wrapper_deps::*;
#[cfg(feature = "kafka-receiver")]
use receiver_wrapper_deps::*;

/// Imports shared by both the exporter and receiver wrapper smoke tests.
#[cfg(any(feature = "kafka-exporter", feature = "kafka-receiver"))]
mod common_wrapper_deps {
    pub(super) use crate::common::kafka::MessageFormat;
    pub(super) use crate::common::kafka::node_harness::KafkaTopics;
    pub(super) use otap_df_pdata::OtlpProtoBytes;
    pub(super) use prost::Message as _;
}

/// Imports used only by the exporter wrapper smoke test.
#[cfg(feature = "kafka-exporter")]
mod exporter_wrapper_deps {
    pub(super) use crate::common::kafka::node_harness::KafkaExporterHarness;
    pub(super) use bytes::Bytes;
    pub(super) use otap_df_otap::pdata::{Context, OtapPdata};
    pub(super) use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    pub(super) use otap_df_pdata::proto::opentelemetry::logs::v1::{
        LogRecord, ResourceLogs, ScopeLogs,
    };
}

/// Imports used only by the receiver wrapper smoke test.
#[cfg(feature = "kafka-receiver")]
mod receiver_wrapper_deps {
    pub(super) use crate::common::kafka::node_harness::KafkaReceiverHarness;
    pub(super) use otap_df_pdata::TryIntoWithOptions;
    pub(super) use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    pub(super) use otap_df_pdata::proto::opentelemetry::trace::v1::{
        ResourceSpans, ScopeSpans, Span,
    };
}

/// Scenario: produce records to each of 3 partitions of a topic and consume
/// them all back.
/// Guarantees: multi-partition topics created via the builder distribute
/// produced records across all partitions and every record round-trips.
#[tokio::test]
async fn demo_multi_partition_produce_consume() {
    with_cluster(
        KafkaTestCluster::builder().topic_with("demo-multi", 3, 1),
        |cluster| async move {
            let consumer = cluster.consumer().subscribe(&["demo-multi"]);
            let producer = cluster.producer().build();
            producer
                .produce_per_partition("demo-multi", 3, 4, b"payload")
                .await;

            let msgs = consumer.recv_n(12).await;
            let by_partition = count_by_partition(&msgs);
            for partition in 0..3 {
                assert_eq!(
                    by_partition.get(&("demo-multi".to_string(), partition)),
                    Some(&4),
                    "partition {partition} should hold 4 records"
                );
            }
        },
    )
    .await;
}

/// Scenario: two consumers join one group on a 3-partition topic.
/// Guarantees: `wait_for_assignment` drives the rebalance and partitions are
/// split across the two members (each gets at least one).
#[tokio::test]
async fn demo_multiple_consumers_rebalance() {
    with_cluster(
        KafkaTestCluster::builder().topic_with("demo-rebalance", 3, 1),
        |cluster| async move {
            let group = "demo-rebalance-group";
            let a = cluster
                .consumer()
                .group_id(group)
                .subscribe(&["demo-rebalance"]);
            let b = cluster
                .consumer()
                .group_id(group)
                .subscribe(&["demo-rebalance"]);

            let a_ok = a.wait_for_assignment(1, Duration::from_secs(10)).await;
            let b_ok = b.wait_for_assignment(1, Duration::from_secs(10)).await;
            assert!(a_ok && b_ok, "both consumers should receive an assignment");

            let total = a.assignment().len() + b.assignment().len();
            assert_eq!(
                total, 3,
                "the 3 partitions must be split across both members"
            );
        },
    )
    .await;
}

/// Scenario: consume + auto-commit, then poll until the broker reports the
/// committed offset.
/// Guarantees: `inspect_group().committed_offset` observes committed progress
/// once the consumer has consumed and committed.
#[tokio::test]
async fn demo_committed_offset_advancement() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-commit"),
        |cluster| async move {
            let group = "demo-commit-group";
            let producer = cluster.producer().build();
            producer.send_n("demo-commit", &[b"a", b"b", b"c"]).await;

            let consumer = cluster
                .consumer()
                .group_id(group)
                .enable_auto_commit(true)
                .subscribe(&["demo-commit"]);
            let _ = consumer.recv_n(3).await;

            let insp = cluster.inspect_group(group);
            let advanced = poll_until(Duration::from_secs(5), Duration::from_millis(250), || {
                insp.committed_offset("demo-commit", 0)
                    .is_some_and(|o| o >= 3)
            })
            .await;
            assert!(advanced, "committed offset should reach 3");
        },
    )
    .await;
}

/// Scenario: inspect topology and watermarks after producing records.
/// Guarantees: `partitions().len()` reflects the created partition count and
/// `message_count` reflects produced records per partition.
#[tokio::test]
async fn demo_topology_and_watermark_inspection() {
    with_cluster(
        KafkaTestCluster::builder().topic_with("demo-inspect", 2, 1),
        |cluster| async move {
            let producer = cluster.producer().build();
            producer
                .produce_per_partition("demo-inspect", 2, 3, b"x")
                .await;
            producer.flush(Duration::from_secs(5));

            let insp = cluster.inspect();
            assert!(insp.topic_exists("demo-inspect"));
            assert_eq!(insp.partitions("demo-inspect").len(), 2);
            assert_eq!(insp.message_count("demo-inspect", 0), 3);
            assert_eq!(insp.message_count("demo-inspect", 1), 3);
        },
    )
    .await;
}

/// Scenario: produce a record to a specific partition.
/// Guarantees: `send_to_partition` places the record on the requested
/// partition, observed via the consumed message's partition field.
#[tokio::test]
async fn demo_produce_to_specific_partition() {
    with_cluster(
        KafkaTestCluster::builder().topic_with("demo-partition", 3, 1),
        |cluster| async move {
            let consumer = cluster.consumer().subscribe(&["demo-partition"]);
            let producer = cluster.producer().build();
            producer
                .send_to_partition("demo-partition", 2, b"pinned")
                .await
                .expect("send to partition 2");

            let msg = consumer.recv().await;
            let _ = msg.assert_topic("demo-partition").assert_payload(b"pinned");
            assert_eq!(msg.partition, 2, "record should land on partition 2");
        },
    )
    .await;
}

/// Scenario: two independent producers write to the same topic.
/// Guarantees: multiple `TestProducer` instances coexist and all their records
/// are consumed.
#[tokio::test]
async fn demo_multiple_producers() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-producers"),
        |cluster| async move {
            let consumer = cluster.consumer().subscribe(&["demo-producers"]);
            let p1 = cluster.producer().client_id("p1").build();
            let p2 = cluster.producer().client_id("p2").build();
            p1.send("demo-producers", b"from-1").await.expect("p1 send");
            p2.send("demo-producers", b"from-2").await.expect("p2 send");

            let msgs = consumer.recv_n(2).await;
            let payloads: Vec<Vec<u8>> = msgs.iter().filter_map(|m| m.payload.clone()).collect();
            assert!(payloads.contains(&b"from-1".to_vec()));
            assert!(payloads.contains(&b"from-2".to_vec()));
        },
    )
    .await;
}

/// Scenario: take all brokers down, then bring them back up.
/// Guarantees: a produce that fails while brokers are down succeeds once they
/// are back up (fault injection + recovery works).
#[tokio::test]
async fn demo_broker_down_up_recovery() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-recovery"),
        |cluster| async move {
            let producer = cluster
                .producer()
                .message_timeout(Duration::from_millis(500))
                .build();

            producer
                .send("demo-recovery", b"before")
                .await
                .expect("before down");

            cluster.faults().all_brokers_down();
            let failed = producer.send("demo-recovery", b"during").await;
            assert!(
                failed.is_err(),
                "produce should fail while brokers are down"
            );

            cluster.faults().all_brokers_up();
            let recovered = poll_until_async(
                Duration::from_secs(10),
                Duration::from_millis(250),
                || async { producer.send("demo-recovery", b"after").await.is_ok() },
            )
            .await;
            assert!(recovered, "delivery should resume once brokers are back up");
        },
    )
    .await;
}

/// Scenario: inject per-broker round-trip latency.
/// Guarantees: with an extended message timeout, a produce still succeeds under
/// injected latency (the rtt knob is wired correctly).
#[tokio::test]
async fn demo_round_trip_time_latency() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-rtt"),
        |cluster| async move {
            cluster
                .faults()
                .round_trip_time(1, Duration::from_millis(200));
            let producer = cluster
                .producer()
                .message_timeout(Duration::from_secs(10))
                .build();
            producer
                .send("demo-rtt", b"slow-but-ok")
                .await
                .expect("produce should succeed within extended timeout");
        },
    )
    .await;
}

/// Scenario: inject metadata request errors, then clear them.
/// Guarantees: the client recovers and delivers once the injected errors are
/// cleared.
#[tokio::test]
async fn demo_request_error_injection() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-errinject"),
        |cluster| async move {
            cluster.faults().inject_request_errors(
                RDKafkaApiKey::Metadata,
                &[
                    RDKafkaRespErr::RD_KAFKA_RESP_ERR_BROKER_NOT_AVAILABLE,
                    RDKafkaRespErr::RD_KAFKA_RESP_ERR_BROKER_NOT_AVAILABLE,
                ],
            );
            cluster
                .faults()
                .clear_request_errors(RDKafkaApiKey::Metadata);

            let producer = cluster
                .producer()
                .message_timeout(Duration::from_secs(10))
                .build();
            let recovered = poll_until_async(
                Duration::from_secs(10),
                Duration::from_millis(250),
                || async { producer.send("demo-errinject", b"ok").await.is_ok() },
            )
            .await;
            assert!(recovered, "client should recover after errors are cleared");
        },
    )
    .await;
}

/// Scenario: exercise the standalone `committed_offset` probe helper.
/// Guarantees: a group with no committed offsets reports `None`.
#[tokio::test]
async fn demo_committed_offset_probe_none_when_unconsumed() {
    with_cluster(
        KafkaTestCluster::builder().topic("demo-probe"),
        |cluster| async move {
            let offset = committed_offset(
                cluster.bootstrap_servers(),
                "never-consumed-group",
                "demo-probe",
                0,
            );
            assert_eq!(
                offset, None,
                "unconsumed group should have no committed offset"
            );
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// Demo 10: wrapper smoke tests (feature-gated).
// ---------------------------------------------------------------------------

/// Scenario: drive the Kafka exporter via `KafkaExporterHarness` and observe
/// its output on the broker through the test-suite consumer.
/// Guarantees: the exporter wrapper wires up, accepts pdata, produces to the
/// configured topic with the OTLP format header, and shuts down cleanly.
#[cfg(feature = "kafka-exporter")]
#[tokio::test]
async fn demo_wrapper_exporter_smoke() {
    with_cluster(
        KafkaTestCluster::builder().topic("wrap-exp-logs"),
        |cluster| async move {
            let consumer = cluster.consumer().subscribe(&["wrap-exp-logs"]);

            let req = ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs {
                    scope_logs: vec![ScopeLogs {
                        log_records: vec![LogRecord {
                            time_unix_nano: 1,
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let bytes = req.encode_to_vec();
            let pdata = OtapPdata::new(
                Context::default(),
                OtlpProtoBytes::ExportLogsRequest(Bytes::from(bytes.clone())).into(),
            );

            let exporter = KafkaExporterHarness::start_for(
                &cluster,
                KafkaTopics::logs("wrap-exp-logs", MessageFormat::OtlpProto),
            );
            exporter.send_pdata(pdata).await.expect("send pdata");

            let _ = consumer
                .recv()
                .await
                .assert_topic("wrap-exp-logs")
                .assert_payload(&bytes)
                .assert_format_otlp();

            exporter.shutdown(Duration::from_secs(1)).await;
        },
    )
    .await;
}

/// Scenario: feed the Kafka receiver via the test-suite producer through
/// `KafkaReceiverHarness` and read + ack the decoded pdata.
/// Guarantees: the receiver wrapper wires up, consumes broker records, decodes
/// them to OtapPdata, supports ack, and shuts down cleanly.
#[cfg(feature = "kafka-receiver")]
#[tokio::test]
async fn demo_wrapper_receiver_smoke() {
    with_cluster(
        KafkaTestCluster::builder().topic("wrap-rcv-traces"),
        |cluster| async move {
            let req = ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans {
                    scope_spans: vec![ScopeSpans {
                        spans: vec![Span {
                            trace_id: vec![1u8; 16],
                            span_id: vec![1u8; 8],
                            name: "span-1".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let bytes = req.encode_to_vec();

            let producer = cluster.producer().build();
            producer
                .send("wrap-rcv-traces", &bytes)
                .await
                .expect("send");

            let mut receiver = KafkaReceiverHarness::start_for(
                &cluster,
                KafkaTopics::traces("wrap-rcv-traces", MessageFormat::OtlpProto),
            );
            let mut pdata = receiver.recv_pdata().await;
            let proto: OtlpProtoBytes = pdata
                .take_payload()
                .try_into_with_default()
                .expect("to OtlpProtoBytes");
            assert!(matches!(proto, OtlpProtoBytes::ExportTracesRequest(_)));
            assert_eq!(proto.as_bytes(), &bytes);

            receiver.ack(pdata);
            receiver.shutdown(Duration::from_secs(5));
            receiver.await_stopped().await;
        },
    )
    .await;
}
