// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Dead-letter-queue configuration parsing, validation, and resolution tests.

use super::*;
use otel_arrow_dfe_config::SignalType;

/// Build a base config JSON with a manual-commit traces receiver and a `dlq`
/// block merged in.
fn manual_with_dlq(dlq: serde_json::Value) -> serde_json::Value {
    json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "commit": {"mode": "manual"},
        "traces": {"topics": ["otlp_spans"]},
        "dlq": dlq,
    })
}

fn parse(json: serde_json::Value) -> Result<KafkaReceiverConfig, serde_json::Error> {
    serde_json::from_value(json)
}

/// Extract the [`KafkaReceiverError`] from a serde deserialization failure by
/// re-running validation on the builder, so tests can assert on the exact
/// validation variant.
fn expect_dlq_error(json: serde_json::Value) -> String {
    let err = parse(json).expect_err("expected DLQ config validation to fail");
    err.to_string()
}

/// Scenario: a DLQ block is present but commit mode is `auto`.
/// Guarantees: validation fails, since DLQ delivery requires the receiver to
/// control offset commits (manual mode only).
#[test]
fn dlq_requires_manual_commit() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "commit": {"mode": "auto", "interval_ms": 1000},
        "traces": {"topics": ["otlp_spans"]},
        "dlq": {"topic": "otel_dlq"},
    });
    let msg = expect_dlq_error(json);
    assert!(
        msg.contains("commit.mode manual"),
        "unexpected error: {msg}"
    );
}

/// Scenario: a DLQ with only a global topic under manual commit.
/// Guarantees: the topic resolves for the ingesting traces signal and capture
/// defaults to all three categories.
#[test]
fn dlq_global_topic_resolves_and_defaults_capture() {
    let cfg = parse(manual_with_dlq(json!({"topic": "otel_dlq"})))
        .expect("valid DLQ config should deserialize");
    let dlq = cfg.dlq().expect("dlq enabled");
    assert_eq!(dlq.topic_for(SignalType::Traces), Some("otel_dlq"));
    assert!(dlq.capture_decode);
    assert!(dlq.capture_unknown_topic);
    assert!(dlq.capture_permanent_nack);
}

/// Scenario: a per-signal DLQ topic override is set for traces.
/// Guarantees: the override wins over the global topic for that signal.
#[test]
fn dlq_per_signal_override_wins() {
    let cfg = parse(manual_with_dlq(json!({
        "topic": "otel_dlq",
        "per_signal": {"traces": "otel_dlq_traces"},
    })))
    .expect("valid DLQ config");
    let dlq = cfg.dlq().expect("dlq enabled");
    assert_eq!(dlq.topic_for(SignalType::Traces), Some("otel_dlq_traces"));
}

/// Scenario: an ingesting signal has neither a global nor per-signal DLQ topic.
/// Guarantees: validation fails with a missing-topic error for that signal.
#[test]
fn dlq_missing_topic_for_ingesting_signal_fails() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "commit": {"mode": "manual"},
        "traces": {"topics": ["otlp_spans"]},
        "dlq": {"per_signal": {"metrics": "otel_dlq_metrics"}},
    });
    let msg = expect_dlq_error(json);
    assert!(
        msg.contains("no topic for signal 'traces'"),
        "unexpected: {msg}"
    );
}

/// Scenario: a DLQ topic equals a configured ingest topic on the same cluster.
/// Guarantees: validation fails (loop prevention), since the receiver would
/// consume its own dead-letter output.
#[test]
fn dlq_topic_overlapping_ingest_literal_fails() {
    let json = manual_with_dlq(json!({"topic": "otlp_spans"}));
    let msg = expect_dlq_error(json);
    assert!(msg.contains("overlaps"), "unexpected: {msg}");
}

/// Scenario: a DLQ topic is matched by an ingest regex pattern on the same cluster.
/// Guarantees: validation fails (loop prevention) for regex ingest topics too.
#[test]
fn dlq_topic_matching_ingest_regex_fails() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "commit": {"mode": "manual"},
        "traces": {"topics": ["^otlp_.*$"], "exclude_topics": []},
        "dlq": {"topic": "otlp_dlq"},
    });
    let msg = expect_dlq_error(json);
    assert!(msg.contains("overlaps"), "unexpected: {msg}");
}

/// Scenario: a DLQ topic equals an ingest topic but points at a different cluster.
/// Guarantees: validation succeeds, since a separate cluster cannot create a loop.
#[test]
fn dlq_topic_overlap_on_different_cluster_ok() {
    let cfg = parse(manual_with_dlq(json!({
        "topic": "otlp_spans",
        "connection": {"brokers": "dlq-broker:9092"},
    })))
    .expect("overlap on a separate cluster is allowed");
    assert!(cfg.dlq().is_some());
}

/// Scenario: an explicit empty capture list is provided.
/// Guarantees: validation fails, since at least one category must be captured.
#[test]
fn dlq_empty_capture_fails() {
    let json = manual_with_dlq(json!({"topic": "otel_dlq", "capture": []}));
    let msg = expect_dlq_error(json);
    assert!(
        msg.contains("capture must not be empty"),
        "unexpected: {msg}"
    );
}

/// Scenario: a narrowed capture set with only `decode` is provided.
/// Guarantees: only decode is captured; the other categories are disabled.
#[test]
fn dlq_narrowed_capture_set() {
    let cfg = parse(manual_with_dlq(json!({
        "topic": "otel_dlq",
        "capture": ["decode"],
    })))
    .expect("valid DLQ config");
    let dlq = cfg.dlq().expect("dlq enabled");
    assert!(dlq.capture_decode);
    assert!(!dlq.capture_unknown_topic);
    assert!(!dlq.capture_permanent_nack);
}

/// Scenario: an invalid DLQ topic name is configured.
/// Guarantees: validation rejects it with an invalid-topic error.
#[test]
fn dlq_invalid_topic_name_fails() {
    let json = manual_with_dlq(json!({"topic": "bad topic!"}));
    let msg = expect_dlq_error(json);
    assert!(msg.contains("dlq topic"), "unexpected: {msg}");
}

/// Scenario: the DLQ connection auth is malformed.
/// Guarantees: validation surfaces the connection error.
#[test]
fn dlq_invalid_connection_tls_fails() {
    let json = manual_with_dlq(json!({
        "topic": "otel_dlq",
        "connection": {"tls": {"cert_file": "cert.pem"}},
    }));
    let msg = expect_dlq_error(json);
    assert!(msg.contains("dlq.connection"), "unexpected: {msg}");
}

/// Scenario: the DLQ producer client config is built for a valid DLQ.
/// Guarantees: the producer `client.id` is auto-derived as `{client_id}-dlq`,
/// and no producer tuning is set so librdkafka's own defaults apply.
#[test]
fn dlq_producer_client_id_auto_derived_and_no_tuning() {
    let cfg = parse(manual_with_dlq(json!({"topic": "otel_dlq"}))).expect("valid");
    let producer = cfg
        .build_dlq_producer_config()
        .expect("dlq present implies producer config");
    assert_eq!(producer.get("client.id"), Some("c-dlq"));
    // No producer tuning is set -> librdkafka defaults apply.
    assert_eq!(producer.get("compression.type"), None);
    assert_eq!(producer.get("request.required.acks"), None);
    assert_eq!(producer.get("message.timeout.ms"), None);
}

/// Scenario: no `dlq` block is present.
/// Guarantees: the DLQ is disabled (accessor returns None).
#[test]
fn no_dlq_block_disables_dlq() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "commit": {"mode": "manual"},
        "traces": {"topics": ["otlp_spans"]},
    });
    let cfg = parse(json).expect("valid");
    assert!(cfg.dlq().is_none());
}

/// Scenario: an unknown field is present in the dlq block.
/// Guarantees: deserialization fails (deny_unknown_fields), catching typos.
#[test]
fn dlq_unknown_field_rejected() {
    let json = manual_with_dlq(json!({"topic": "otel_dlq", "bogus": true}));
    assert!(parse(json).is_err());
}
