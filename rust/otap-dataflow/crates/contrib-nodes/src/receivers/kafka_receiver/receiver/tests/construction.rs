// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver construction and configuration unit tests: static-member id
//! suffixing, topic-overlap validation, offset-tracker creation, and JSON config parsing.

use super::*;

// ---- Construction and configuration ----

/// Scenario (construction and configuration): a receiver is built on a multi-core
/// pipeline with a configured `group.instance.id`.
/// Guarantees: the instance id is suffixed with the core id so each core joins the
/// consumer group as a distinct static member.
#[test]
fn new_suffixes_group_instance_id_with_core_id_when_multi_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with(3, 4);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1-3"),
        "multi-core pipeline should suffix group.instance.id with core id"
    );
}

/// Scenario (construction and configuration): a receiver is built on a single-core
/// pipeline with a configured `group.instance.id`.
/// Guarantees: the instance id is left unchanged, so a single-core deployment keeps the
/// operator-provided static member id.
#[test]
fn new_keeps_group_instance_id_unchanged_when_single_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with(0, 1);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1"),
        "single-core pipeline should leave group.instance.id unchanged"
    );
}

/// Scenario (construction and configuration): a receiver is built without a
/// `group.instance.id`.
/// Guarantees: no instance id is synthesized, so the consumer joins as a dynamic
/// (non-static) group member.
#[test]
fn new_leaves_group_instance_id_absent_when_unset() {
    let cfg = make_config(&["t"], &["m"], &["l"], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx_with(2, 4);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        None,
        "unset group.instance.id should remain absent"
    );
}

/// Scenario (construction and configuration): a receiver is constructed with traces,
/// metrics, and logs on distinct topics.
/// Guarantees: construction succeeds, so a valid multi-signal configuration is
/// accepted.
#[test]
fn new_succeeds_with_distinct_topics() {
    let cfg = make_config(&["t"], &["m"], &["l"], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg);
    assert!(receiver.is_ok());
}

/// Scenario (construction and configuration): two signals are configured to share the
/// same topic.
/// Guarantees: config validation fails with an overlap error, so one topic cannot feed
/// two signal decoders.
#[test]
fn new_fails_with_overlapping_topics() {
    let result = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["same".to_string()]))
            .with_metrics(SignalConfig::new(vec!["same".to_string()])),
    );
    assert!(result.is_err());
    // The error is now `KafkaReceiverError::ConfigOverlappingTopics`;
    // assert against its Display string.
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("overlap"),
        "expected overlap error, got: {err_str}"
    );
}

/// Scenario (construction and configuration): a receiver is built in the default
/// manual-commit mode.
/// Guarantees: an empty offset tracker is present, so the manual at-least-once commit
/// path is wired and ready.
#[test]
fn new_creates_offset_tracker_when_auto_commit_disabled() {
    let cfg = make_config(&["t"], &["m"], &[], MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit()); // default is manual (not auto)
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
    // offset_tracker is always present; verify it starts empty
    assert_eq!(receiver.offset_tracker.total_pending(), 0);
}

/// Scenario (construction and configuration): a receiver is built with auto-commit
/// enabled.
/// Guarantees: construction succeeds and the tracker starts empty, so auto-commit mode
/// builds without engaging the manual tracker.
#[test]
fn new_succeeds_when_auto_commit_enabled() {
    let cfg = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["t".to_string()]))
            .with_commit(CommitConfig {
                mode: ConfigCommitMode::Auto,
                interval_ms: Some(1000),
            })
            .with_isolation_level(IsolationLevel::ReadUncommitted),
    )
    .expect("test config should be valid");
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");
    // offset_tracker exists but won't be used when auto-commit is enabled
    assert_eq!(receiver.offset_tracker.total_pending(), 0);
}

/// Scenario (construction and configuration): a receiver is built from a complete JSON
/// config with all required fields and topics.
/// Guarantees: `from_config` succeeds, so a well-formed operator config deserializes
/// and builds.
#[test]
fn from_config_succeeds_with_valid_json() {
    let json: Value = serde_json::json!({
        "brokers": "kafka:9092",
        "group_id": "my-group",
        "client_id": "my-client",
        "traces": {"topics": ["traces"]},
        "metrics": {"topics": ["metrics"]},
        "logs": {"topics": ["logs"]}
    });
    let ctx = make_pipeline_ctx();
    let result = KafkaReceiver::from_config(ctx, &json);
    assert!(result.is_ok());
}

/// Scenario (construction and configuration): a receiver is built from JSON missing the
/// required brokers/group_id/client_id fields.
/// Guarantees: `from_config` returns an error, so an incomplete config is rejected
/// rather than silently defaulted.
#[test]
fn from_config_fails_with_missing_required_fields() {
    // brokers, group_id, client_id are required
    let json: Value = serde_json::json!({});
    let ctx = make_pipeline_ctx();
    let result = KafkaReceiver::from_config(ctx, &json);
    assert!(result.is_err());
}

/// Scenario (construction and configuration): a receiver is built from JSON with
/// required fields but no signal topics.
/// Guarantees: `from_config` returns an error, so a receiver that would subscribe to
/// nothing is rejected.
#[test]
fn from_config_fails_with_no_topics() {
    // Required fields present but no topics configured
    let json: Value = serde_json::json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c"
    });
    let ctx = make_pipeline_ctx();
    let result = KafkaReceiver::from_config(ctx, &json);
    assert!(result.is_err());
}

/// Scenario (construction and configuration): a receiver is built from JSON that puts
/// two signals on the same topic.
/// Guarantees: `from_config` returns an error, so overlapping-topic configs are
/// rejected at deserialization time.
#[test]
fn from_config_fails_with_overlapping_topics() {
    let json: Value = serde_json::json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["same"]},
        "metrics": {"topics": ["same"]}
    });
    let ctx = make_pipeline_ctx();
    let result = KafkaReceiver::from_config(ctx, &json);
    assert!(result.is_err());
}
