// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit and integration tests for the Kafka receiver, split by concern.
//!
//! This module owns the shared test imports, constants, and helper functions;
//! each concern-specific submodule pulls them in via `use super::*`.

use super::*;
use crate::receivers::kafka_receiver::config::{
    AttributeValueType, AutoOffsetReset, CommitConfig, CommitMode as ConfigCommitMode,
    HeaderExtraction, IsolationLevel, KafkaReceiverConfigBuilder, RebalanceStrategy, SignalConfig,
    TransientNackConfig, TransientNackMode,
};

use crate::common::kafka::MessageFormat;
use crate::common::kafka::node_harness::KafkaReceiverHarness;
use crate::common::kafka::node_harness::node_metrics::{FoldedMetrics, metric_value};
use crate::common::kafka::test::cluster::KafkaTestCluster;
use crate::common::kafka::test::consumer::{RebalanceTrigger, committed_offset};
use crate::common::kafka::test::producer::SendRecord;
use crate::common::kafka::test::wait::poll_until;
use crate::common::kafka::test::with_cluster;
use otel_arrow_dfe_config::transport_headers_policy::{CaptureDefaults, CaptureRule};
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::control::RuntimeControlMsg;
use otel_arrow_dfe_pdata::OtlpProtoBytes;
use otel_arrow_dfe_pdata::Producer;
use otel_arrow_dfe_pdata::otap::{Logs, Metrics};
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{
    AnyValue, InstrumentationScope, KeyValue, any_value,
};
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::Resource;
use otel_arrow_dfe_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayload, PayloadData, TryIntoWithOptions};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use prost::Message;
use rdkafka::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::topic_partition_list::{Offset, TopicPartitionList};
use rdkafka::types::RDKafkaRespErr;
use std::collections::HashMap;
use std::time::Duration;

// Test-only re-exports of receiver internals that moved into concern
// submodules, so each `use super::*` test file can reach them by name.
use super::consumer::compute_consumer_lag;
use super::decode::{SignalDecoder, decode_calldata};
use super::offset_feedback::{OffsetFeedbackAction, classify_offset_feedback};
use otel_arrow_dfe_config::SignalType;

// Test-only imports for symbols the split test files reference but the
// receiver implementation no longer imports directly.
use crate::common::kafka::{MSG_FORMAT_OTAP, MSG_FORMAT_SYSLOG};
use crate::receivers::kafka_receiver::identity::DeliveryGeneration;
use bytes::Bytes;
use otel_arrow_dfe_engine::control::{CallData, Context8u8};
use rdkafka::consumer::CommitMode;
use smallvec::smallvec;

/// Number of partitions provisioned for the rebalance integration tests.
const REBALANCE_TEST_PARTITIONS: i32 = 2;
/// Records produced to each partition in the rebalance integration tests.
const REBALANCE_RECORDS_PER_PARTITION: i32 = 5;

mod construction;
mod decode;
mod lifecycle;
mod offsets;
mod operational;
mod rebalance;
mod receive_loop;
mod replay;
mod resilience;
mod topics;
mod transport_headers;

// ---- Shared test helpers ----

fn measurement_counter(
    snapshots: &[otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot],
    metric_set: &str,
    attributes: &[(&str, &str)],
    metric: &str,
) -> u64 {
    snapshots
        .iter()
        .filter(|snapshot| snapshot.descriptor().name == metric_set)
        .filter(|snapshot| {
            attributes
                .iter()
                .all(|(key, value)| snapshot.measurement_attribute_value(key) == Some(*value))
        })
        .filter_map(|snapshot| metric_value(snapshot, metric))
        .sum()
}

fn create_logs_service_request() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "a".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                scope: Some(InstrumentationScope {
                    attributes: vec![KeyValue {
                        key: "b".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                log_records: vec![
                    LogRecord {
                        time_unix_nano: 1,
                        attributes: vec![KeyValue {
                            key: "c".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    LogRecord {
                        time_unix_nano: 2,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn create_metrics_service_request() -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

/// Helper to create a trace request with actual spans containing trace_id and attributes.
fn create_traces_with_spans() -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(InstrumentationScope::default()),
                spans: vec![
                    Span {
                        trace_id: vec![1u8; 16],
                        span_id: vec![1u8; 8],
                        name: "span-1".to_string(),
                        attributes: vec![KeyValue {
                            key: "existing".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue("original".to_string())),
                            }),
                        }],
                        ..Default::default()
                    },
                    Span {
                        trace_id: vec![2u8; 16],
                        span_id: vec![2u8; 8],
                        name: "span-2".to_string(),
                        attributes: vec![KeyValue {
                            key: "existing-2".to_string(),
                            value: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(
                                    "original-2".to_string(),
                                )),
                            }),
                        }],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

/// Create OTAP Arrow wire bytes from the `create_traces_with_spans()` helper,
/// converting a real `ExportTraceServiceRequest` with 2 spans (including
/// trace_ids and attributes) into OTAP Arrow wire format.
fn create_traces_with_spans_otap_bytes() -> Vec<u8> {
    let request = create_traces_with_spans();
    let mut buf = Vec::new();
    request.encode(&mut buf).expect("encode OTLP request");

    // Convert OTLP bytes -> OtapPayload -> OtapArrowRecords
    let payload: OtapPayload = OtlpProtoBytes::ExportTracesRequest(Bytes::from(buf)).into();
    let mut otap_records: OtapArrowRecords = payload
        .try_into_with_default()
        .expect("convert OTLP to OTAP Arrow");

    // Serialize to BatchArrowRecords wire bytes (as the Kafka receiver expects)
    arrow_records_to_bytes(&mut otap_records)
}

fn create_metrics_otap_arrow_records_bytes() -> Vec<u8> {
    let mut arrow_records = OtapArrowRecords::Metrics(Metrics::default());
    arrow_records_to_bytes(&mut arrow_records)
}

fn create_logs_otap_arrow_records_bytes() -> Vec<u8> {
    let mut arrow_records = OtapArrowRecords::Logs(Logs::default());
    arrow_records_to_bytes(&mut arrow_records)
}

fn arrow_records_to_bytes(arrow_records: &mut OtapArrowRecords) -> Vec<u8> {
    let mut producer = Producer::new();
    let bar = producer
        .produce_bar(arrow_records)
        .expect("failed to get batch arrow reocrds");
    let mut bytes = vec![];
    bar.encode(&mut bytes).expect("failed to encode");
    bytes
}

/// Convert an `OtapPdata` (containing OTAP Arrow records) back to an OTLP
/// `ExportTraceServiceRequest` so tests can assert against familiar protobuf
/// structs instead of Arrow column internals.
fn otap_pdata_to_traces(pdata: &mut OtapPdata) -> ExportTraceServiceRequest {
    let otlp: OtlpProtoBytes = pdata
        .take_payload()
        .try_into_with_default()
        .expect("OTAP -> OTLP conversion");
    ExportTraceServiceRequest::decode(otlp.as_bytes()).expect("decode OTLP traces")
}

/// Builds an auto-commit [`KafkaReceiverConfig`] for the given per-signal
/// topics and message format, with optional resource-attribute-from-header
/// extraction. Mirrors the config logic of the former
/// `setup_receiver_harness_with_headers` helper.
fn auto_config(
    brokers: &str,
    traces_topics: &[&str],
    metrics_topics: &[&str],
    logs_topics: &[&str],
    msg_format: MessageFormat,
    resource_attrs_from_headers: HashMap<String, HeaderExtraction>,
) -> KafkaReceiverConfig {
    KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new(brokers, "test-group", "test-client")
            .with_traces(
                SignalConfig::new(traces_topics.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(msg_format),
            )
            .with_metrics(
                SignalConfig::new(metrics_topics.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(msg_format),
            )
            .with_logs(
                SignalConfig::new(logs_topics.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(msg_format),
            )
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Auto,
                interval_ms: Some(1000),
            })
            .with_auto_offset_reset(AutoOffsetReset::Earliest)
            .with_isolation_level(IsolationLevel::ReadUncommitted)
            .with_resource_attrs_from_headers(resource_attrs_from_headers),
    )
    .expect("test config valid")
}

/// Builds a manual-commit [`KafkaReceiverConfig`] for a single traces topic,
/// with an explicit consumer-group id, a safety-net commit timer, and an
/// optional partition-assignment strategy. Mirrors the config logic of the
/// former `setup_manual_traces_harness_with_strategy` helper.
fn manual_traces_config(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
    commit_interval_ms: u64,
    rebalance_strategy: Option<RebalanceStrategy>,
) -> KafkaReceiverConfig {
    let mut builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: Some(commit_interval_ms),
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    if let Some(strategy) = rebalance_strategy {
        builder = builder.with_rebalance_strategy(strategy);
    }
    KafkaReceiverConfig::try_from(builder).expect("test config valid")
}

/// Builds a manual-commit [`KafkaReceiverConfig`] for a single traces topic
/// with NO safety-net commit timer, so offsets are committed purely through
/// ack/nack. Used by tests that need deterministic watermark assertions
/// without a periodic timer racing the acks.
fn manual_traces_config_no_timer(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test config valid")
}

/// Like [`manual_traces_config_no_timer`] but arms the opt-in consumer-lag
/// refresh timer at `lag_refresh_interval_ms`, so a lag-refresh worker is
/// periodically spawned and can be in flight when a shutdown arrives.
fn manual_traces_config_with_lag_refresh(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
    lag_refresh_interval_ms: u64,
) -> KafkaReceiverConfig {
    let builder = KafkaReceiverConfigBuilder::new(brokers, group_id, "test-client")
        .with_traces(
            SignalConfig::new(vec![traces_topic.to_string()])
                .with_encoding(MessageFormat::OtlpProto),
        )
        .with_commit(CommitConfig {
            mode: ConfigCommitMode::Manual,
            interval_ms: None,
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted)
        .with_lag_refresh_interval_ms(Some(lag_refresh_interval_ms));
    KafkaReceiverConfig::try_from(builder).expect("test config valid")
}

/// Like [`manual_traces_config_no_timer`] but configures the traces signal
/// for the OTAP-Arrow encoding, whose decode path validates the payload (so
/// an undecodable record surfaces as a processing error).
fn manual_otap_traces_config_no_timer(
    brokers: &str,
    group_id: &str,
    traces_topic: &str,
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
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    KafkaReceiverConfig::try_from(builder).expect("test config valid")
}

fn make_config(
    traces: &[&str],
    metrics: &[&str],
    logs: &[&str],
    fmt: MessageFormat,
) -> KafkaReceiverConfig {
    KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
            .with_traces(
                SignalConfig::new(traces.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(fmt),
            )
            .with_metrics(
                SignalConfig::new(metrics.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(fmt),
            )
            .with_logs(
                SignalConfig::new(logs.iter().map(|s| (*s).to_string()).collect())
                    .with_encoding(fmt),
            )
            .with_isolation_level(IsolationLevel::ReadUncommitted),
    )
    .expect("test config should be valid")
}

fn make_pipeline_ctx() -> PipelineContext {
    make_pipeline_ctx_with(0, 1)
}

fn make_pipeline_ctx_with(core_id: usize, num_cores: usize) -> PipelineContext {
    let registry = TelemetryRegistryHandle::new();
    let controller_ctx = ControllerContext::new(registry);
    controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), core_id, num_cores, 0)
}

fn make_config_with_group_instance_id(instance_id: &str) -> KafkaReceiverConfig {
    KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["t".to_string()]))
            .with_group_instance_id(instance_id),
    )
    .expect("test config should be valid")
}

/// Build a manual-commit `StreamConsumer` bound to `brokers` in `group`,
/// with librdkafka auto-commit disabled so the test controls committed
/// offsets explicitly.
fn make_manual_consumer(brokers: &str, group: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("group.id", group)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("failed to create consumer")
}
