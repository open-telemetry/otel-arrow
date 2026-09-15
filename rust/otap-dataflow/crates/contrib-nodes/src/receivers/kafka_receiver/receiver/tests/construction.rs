// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver construction and configuration unit tests: static-member id
//! suffixing, topic-overlap validation, offset-tracker creation, and JSON config parsing.

use super::*;

// ---- Construction and configuration ----

/// Scenario (construction and configuration): a receiver is built on a multi-core
/// pipeline with a configured `group.instance.id`.
/// Guarantees: the instance id is suffixed with the deployment generation and
/// the core id so each (generation, core) joins the consumer group as a
/// distinct static member.
#[test]
fn new_suffixes_group_instance_id_with_core_id_when_multi_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with(3, 4);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1-g0-3"),
        "multi-core pipeline should suffix group.instance.id with generation \
         and core id"
    );
}

/// Scenario (construction and configuration): a receiver is built on a single-core
/// pipeline with a configured `group.instance.id`.
/// Guarantees: the instance id is suffixed with the deployment generation (but
/// not a core id on a single-core pipeline), so a new pipeline generation
/// during a live-reconfiguration cutover is a distinct static member from the
/// draining old generation.
#[test]
fn new_suffixes_group_instance_id_with_generation_when_single_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with(0, 1);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1-g0"),
        "single-core pipeline should suffix group.instance.id with the \
         deployment generation"
    );
}

/// Scenario (construction and configuration): two receivers are built from the
/// same operator `group.instance.id` but different deployment generations, as
/// happens during a live-reconfiguration cutover (old generation draining, new
/// generation starting).
/// Guarantees: the resolved `group.instance.id`s differ by generation suffix,
/// so the new instance is a distinct static member and does not fence the old
/// instance's offset commits during the drain overlap.
#[test]
fn new_distinct_generations_yield_distinct_group_instance_ids() {
    let cfg_old = make_config_with_group_instance_id("instance-1");
    let ctx_old = make_pipeline_ctx_with_generation(0, 1, 7);
    let old = KafkaReceiver::new(ctx_old, cfg_old).expect("old receiver should build");

    let cfg_new = make_config_with_group_instance_id("instance-1");
    let ctx_new = make_pipeline_ctx_with_generation(0, 1, 8);
    let new = KafkaReceiver::new(ctx_new, cfg_new).expect("new receiver should build");

    assert_eq!(old.config.group_instance_id(), Some("instance-1-g7"));
    assert_eq!(new.config.group_instance_id(), Some("instance-1-g8"));
    assert_ne!(
        old.config.group_instance_id(),
        new.config.group_instance_id(),
        "distinct generations must resolve to distinct static member ids so a \
         cutover does not fence the draining old instance",
    );
}

/// Scenario (construction and configuration): the configured `group.instance.id`
/// is short enough on its own, but appending the single-core generation suffix
/// pushes the resolved id one character past Kafka's 249-character limit.
/// Guarantees: receiver construction fails with a clear configuration error
/// (naming group.instance.id and the 249-character limit) instead of deferring
/// an opaque static-member join rejection to the broker.
#[test]
fn new_rejects_group_instance_id_when_resolved_exceeds_kafka_limit() {
    // Single-core suffix is "-g0" (3 chars); a 247-char base resolves to 250.
    let base = "a".repeat(247);
    let cfg = make_config_with_group_instance_id(&base);
    let ctx = make_pipeline_ctx_with(0, 1);
    let err = match KafkaReceiver::new(ctx, cfg) {
        Ok(_) => panic!("resolved group.instance.id over 249 chars must be rejected"),
        Err(e) => e.to_string(),
    };
    assert!(
        err.contains("group.instance.id"),
        "error should name group.instance.id: {err}"
    );
    assert!(
        err.contains("249"),
        "error should cite the 249-character Kafka limit: {err}"
    );
}

/// Scenario (construction and configuration): the configured `group.instance.id`
/// is sized so that appending the single-core generation suffix resolves to
/// exactly Kafka's 249-character limit.
/// Guarantees: the boundary value is accepted (the limit is inclusive), so the
/// off-by-one is guarded -- 249 builds while 250 is rejected.
#[test]
fn new_accepts_group_instance_id_at_kafka_limit_boundary() {
    // Single-core suffix is "-g0" (3 chars); a 246-char base resolves to 249.
    let base = "a".repeat(246);
    let cfg = make_config_with_group_instance_id(&base);
    let ctx = make_pipeline_ctx_with(0, 1);
    let receiver =
        KafkaReceiver::new(ctx, cfg).expect("resolved id of exactly 249 chars must build");
    let resolved = receiver
        .config
        .group_instance_id()
        .expect("resolved id present");
    assert_eq!(
        resolved.len(),
        249,
        "resolved id should sit exactly on the inclusive 249-character limit"
    );
    assert_eq!(resolved, format!("{base}-g0"));
}

/// Scenario (construction and configuration): a receiver is built on a multi-core
/// pipeline whose deployment generation and core id are both multi-digit values.
/// Guarantees: the resolved `group.instance.id` embeds the full multi-digit
/// generation and core id (`-g{gen}-{core}`), so wide deployments still yield a
/// correctly formatted distinct static member per (generation, core).
#[test]
fn new_suffixes_group_instance_id_with_multi_digit_generation_and_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with_generation(12, 16, 123);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1-g123-12"),
        "multi-digit generation and core id must both appear in full"
    );
}

/// Scenario (construction and configuration): a receiver is built on a single-core
/// pipeline whose deployment generation is a multi-digit value.
/// Guarantees: the resolved `group.instance.id` embeds the full multi-digit
/// generation with no core suffix (`-g{gen}`), so a long-lived pipeline that has
/// been reconfigured many times still resolves to a correctly formatted id.
#[test]
fn new_suffixes_group_instance_id_with_multi_digit_generation_single_core() {
    let cfg = make_config_with_group_instance_id("instance-1");
    let ctx = make_pipeline_ctx_with_generation(0, 1, 1024);
    let receiver = KafkaReceiver::new(ctx, cfg).expect("receiver should build");
    assert_eq!(
        receiver.config.group_instance_id(),
        Some("instance-1-g1024"),
        "single-core pipeline embeds the full multi-digit generation and no core"
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
    let cfg = auto_traces_config("b:9092", "g", "c", "t");
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
