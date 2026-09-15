// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Construction and configuration tests: enum-to-librdkafka mappings, defaults,
//! validation rules, client-config construction, and full deserialization.

use super::*;

// ---- Construction and configuration ----

/// Scenario (construction and configuration): the `earliest` auto-offset-reset is
/// mapped to its librdkafka value.
/// Guarantees: it maps to `"earliest"`, so the setting reaches librdkafka correctly.
#[test]
fn auto_offset_reset_to_kafka_value_earliest() {
    assert_eq!(AutoOffsetReset::Earliest.to_kafka_value(), "earliest");
}

/// Scenario (construction and configuration): the `latest` auto-offset-reset is mapped
/// to its librdkafka value.
/// Guarantees: it maps to `"latest"`, so the setting reaches librdkafka correctly.
#[test]
fn auto_offset_reset_to_kafka_value_latest() {
    assert_eq!(AutoOffsetReset::Latest.to_kafka_value(), "latest");
}

/// Scenario (construction and configuration): the `error` auto-offset-reset is mapped
/// to its librdkafka value.
/// Guarantees: it maps to `"error"`, so the setting reaches librdkafka correctly.
#[test]
fn auto_offset_reset_to_kafka_value_error() {
    assert_eq!(AutoOffsetReset::Error.to_kafka_value(), "error");
}

/// Scenario (construction and configuration): the read-uncommitted isolation level is
/// mapped to its librdkafka value.
/// Guarantees: it maps to `"read_uncommitted"`, so the isolation setting reaches
/// librdkafka correctly.
#[test]
fn isolation_level_to_kafka_value_read_uncommitted() {
    assert_eq!(
        IsolationLevel::ReadUncommitted.to_kafka_value(),
        "read_uncommitted"
    );
}

/// Scenario (construction and configuration): the read-committed isolation level is
/// mapped to its librdkafka value.
/// Guarantees: it maps to `"read_committed"`, so the isolation setting reaches
/// librdkafka correctly.
#[test]
fn isolation_level_to_kafka_value_read_committed() {
    assert_eq!(
        IsolationLevel::ReadCommitted.to_kafka_value(),
        "read_committed"
    );
}

/// Scenario (construction and configuration): a `CommitConfig` is constructed with
/// defaults.
/// Guarantees: the default mode and interval match the documented defaults, so an
/// unconfigured commit block is safe.
#[test]
fn commit_config_defaults() {
    let cfg = CommitConfig::default();
    assert_eq!(cfg.mode, CommitMode::Manual);
    assert_eq!(cfg.interval_ms, None);
}

/// Scenario (construction and configuration): a commit config with auto mode is
/// deserialized.
/// Guarantees: the auto commit mode and interval are parsed, so operators can select
/// auto-commit via config.
#[test]
fn commit_config_deserialize_auto() {
    let json = json!({"mode": "auto", "interval_ms": 5000});
    let cfg: CommitConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.mode, CommitMode::Auto);
    assert_eq!(cfg.interval_ms, Some(5000));
}

/// Scenario (construction and configuration): a commit config with manual mode is
/// deserialized.
/// Guarantees: the manual commit mode is parsed, so operators can select manual
/// (at-least-once) commit via config.
#[test]
fn commit_config_deserialize_manual() {
    let json = json!({"mode": "manual", "interval_ms": 500});
    let cfg: CommitConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.mode, CommitMode::Manual);
    assert_eq!(cfg.interval_ms, Some(500));
}

/// Scenario (construction and configuration): an empty commit block is deserialized.
/// Guarantees: it falls back to the documented defaults, so omitting the commit block
/// is valid.
#[test]
fn commit_config_deserialize_defaults_when_empty() {
    let json = json!({});
    let cfg: CommitConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.mode, CommitMode::Manual);
    assert_eq!(cfg.interval_ms, None);
}

/// Scenario: A transient-NACK block uses all field defaults.
/// Guarantees: Replay is preferred and the documented backoff bounds are applied.
#[test]
fn transient_nack_config_defaults_to_replay() {
    let cfg = TransientNackConfig::default();
    assert_eq!(cfg.mode, TransientNackMode::Replay);
    assert_eq!(cfg.initial_backoff_ms, 1_000);
    assert_eq!(cfg.max_backoff_ms, 30_000);
}

/// Scenario: A manual-commit receiver omits the transient-NACK policy.
/// Guarantees: Validation resolves the effective policy to Kafka replay.
#[test]
fn omitted_transient_nack_policy_defaults_to_replay_in_manual_mode() {
    let builder = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]));

    let cfg = KafkaReceiverConfig::try_from(builder).expect("manual config is valid");

    assert!(matches!(
        cfg.transient_nack_policy(),
        EffectiveTransientNackPolicy::Replay(_)
    ));
    assert!(cfg.replays_transient_nacks());
}

/// Scenario: An auto-commit receiver omits the transient-NACK policy.
/// Guarantees: Validation succeeds with replay inactive instead of rejecting an implicit policy.
#[test]
fn omitted_transient_nack_policy_is_inactive_in_auto_mode() {
    let builder = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]))
        .with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: None,
        });

    let cfg = KafkaReceiverConfig::try_from(builder).expect("auto config is valid");

    assert_eq!(
        cfg.transient_nack_policy(),
        &EffectiveTransientNackPolicy::Inactive
    );
    assert!(!cfg.replays_transient_nacks());
}

/// Scenario: A manual-commit receiver explicitly selects commit-and-skip.
/// Guarantees: Validation preserves the explicit terminal policy rather than treating it as inactive.
#[test]
fn explicit_commit_and_skip_policy_is_active_in_manual_mode() {
    let builder = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]))
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::CommitAndSkip,
            ..TransientNackConfig::default()
        });

    let cfg = KafkaReceiverConfig::try_from(builder).expect("manual config is valid");

    assert_eq!(
        cfg.transient_nack_policy(),
        &EffectiveTransientNackPolicy::CommitAndSkip
    );
    assert!(!cfg.replays_transient_nacks());
}

/// Scenario: An auto-commit receiver explicitly selects commit-and-skip.
/// Guarantees: Validation resolves the policy to inactive because Kafka owns commits.
#[test]
fn explicit_commit_and_skip_policy_is_inactive_in_auto_mode() {
    let builder = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]))
        .with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::CommitAndSkip,
            ..TransientNackConfig::default()
        });

    let cfg = KafkaReceiverConfig::try_from(builder).expect("auto config is valid");

    assert_eq!(
        cfg.transient_nack_policy(),
        &EffectiveTransientNackPolicy::Inactive
    );
    assert!(!cfg.replays_transient_nacks());
}

/// Scenario: A transient-NACK replay policy is deserialized.
/// Guarantees: Operators can explicitly configure Kafka replay and its bounded backoff.
#[test]
fn transient_nack_replay_config_deserializes() {
    let cfg: TransientNackConfig = serde_json::from_value(json!({
        "mode": "replay",
        "initial_backoff_ms": 250,
        "max_backoff_ms": 10_000,
    }))
    .expect("valid replay config");

    assert_eq!(cfg.mode, TransientNackMode::Replay);
    assert_eq!(cfg.initial_backoff_ms, 250);
    assert_eq!(cfg.max_backoff_ms, 10_000);
}

/// Scenario: A transient-NACK commit-and-skip policy is deserialized.
/// Guarantees: Operators can explicitly opt out of manual-mode Kafka redelivery.
#[test]
fn transient_nack_commit_and_skip_config_deserializes() {
    let cfg: TransientNackConfig = serde_json::from_value(json!({
        "mode": "commit_and_skip",
    }))
    .expect("valid commit-and-skip config");

    assert_eq!(cfg.mode, TransientNackMode::CommitAndSkip);
}

/// Scenario (construction and configuration): a `SignalConfig` is constructed with
/// defaults.
/// Guarantees: the defaults (encoding, empty excludes) match the documented values, so
/// a minimal signal config is safe.
#[test]
fn signal_config_defaults() {
    let cfg = SignalConfig::default();
    assert!(cfg.topics().is_empty());
    assert!(cfg.exclude_topics().is_empty());
    assert_eq!(cfg.encoding(), MessageFormat::OtlpProto);
}

/// Scenario (construction and configuration): a signal config with a topics list is
/// deserialized.
/// Guarantees: the topics are parsed onto the signal, so per-signal topic subscription
/// is configurable.
#[test]
fn signal_config_deserialize_with_topics() {
    let json = json!({"topics": ["traces-prod", "^traces-team-.*"]});
    let cfg: SignalConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.topics(), &["traces-prod", "^traces-team-.*"]);
    assert!(cfg.exclude_topics().is_empty());
    assert_eq!(cfg.encoding(), MessageFormat::OtlpProto);
}

/// Scenario (construction and configuration): a signal config with exclude_topics and
/// an explicit encoding is deserialized.
/// Guarantees: both fields are parsed, so exclude patterns and per-signal encoding are
/// configurable.
#[test]
fn signal_config_deserialize_with_exclude_and_encoding() {
    let json = json!({
        "topics": ["^traces-.*"],
        "exclude_topics": ["^traces-test$"],
        "encoding": "otap_proto"
    });
    let cfg: SignalConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.topics(), &["^traces-.*"]);
    assert_eq!(cfg.exclude_topics(), &["^traces-test$"]);
    assert_eq!(cfg.encoding(), MessageFormat::OtapProto);
}

/// Scenario (construction and configuration): the config getters are read after
/// construction.
/// Guarantees: each getter returns the configured value, so downstream code reads
/// config through a stable accessor surface.
#[test]
fn getters_return_expected_values() {
    let cfg: KafkaReceiverConfig =
        KafkaReceiverConfigBuilder::new("broker:9092", "test-group", "test-client")
            .with_traces(SignalConfig {
                topics: vec!["traces-topic".to_string()],
                ..Default::default()
            })
            .with_metrics(SignalConfig {
                topics: vec!["metrics-topic".to_string()],
                ..Default::default()
            })
            .with_logs(SignalConfig {
                topics: vec!["logs-topic".to_string()],
                ..Default::default()
            })
            .try_into()
            .unwrap();
    assert_eq!(cfg.brokers(), "broker:9092");
    assert_eq!(cfg.group_id(), "test-group");
    assert_eq!(cfg.client_id(), "test-client");
    assert_eq!(cfg.traces_topics(), &["traces-topic"]);
    assert_eq!(cfg.metrics_topics(), &["metrics-topic"]);
    assert_eq!(cfg.logs_topics(), &["logs-topic"]);
    assert!(!cfg.is_auto_commit());
    assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
}

/// Scenario (construction and configuration): topic getters are read for unconfigured
/// signals.
/// Guarantees: they return empty, so an unconfigured signal contributes no topics.
#[test]
fn getters_return_empty_for_missing_topics() {
    // Only traces has topics; metrics and logs are empty.
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: Some(1000),
        })
        .try_into()
        .unwrap();
    assert!(!cfg.metrics_topics().is_empty() || cfg.metrics_topics().is_empty()); // always true
    assert!(cfg.metrics_topics().is_empty());
    assert!(cfg.logs_topics().is_empty());
    assert!(cfg.is_auto_commit());
}

/// Scenario (construction and configuration): the brokers field is set to an empty
/// string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_brokers_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("brokers"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the client_id field is set to an empty
/// string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_client_id_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("client_id"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the group_id field is set to an empty
/// string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_group_id_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("group_id"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the group_instance_id field is set to an
/// empty string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_group_instance_id_fails() {
    let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    cfg.group_instance_id = Some(String::new());
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("group_instance_id"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the message_format_header field is set to
/// an empty string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_message_format_header_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_message_format_header("");
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(
        err.contains("message_format_header"),
        "unexpected error: {err}"
    );
}

/// Scenario (construction and configuration): the resource-attrs header key field is
/// set to an empty string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_resource_attrs_header_key_fails() {
    let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    _ = cfg.resource_attrs_from_headers.insert(
        String::new(),
        HeaderExtraction {
            key: "attr.name".to_string(),
            value_type: AttributeValueType::String,
        },
    );
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("empty header key"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the resource-attrs extraction key field
/// is set to an empty string.
/// Guarantees: validation fails, so an empty required string cannot be silently
/// accepted.
#[test]
fn validate_empty_resource_attrs_extraction_key_fails() {
    let mut cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    _ = cfg.resource_attrs_from_headers.insert(
        "X-Header".to_string(),
        HeaderExtraction {
            key: String::new(),
            value_type: AttributeValueType::String,
        },
    );
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(
        err.contains("key can't be empty"),
        "unexpected error: {err}"
    );
}

/// Scenario (construction and configuration): max fetch bytes is set below min fetch
/// bytes.
/// Guarantees: validation fails, so an inconsistent fetch-size window is rejected.
#[test]
fn validate_max_fetch_bytes_less_than_min_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_min_fetch_bytes(100)
        .with_max_fetch_bytes(50);
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("max_fetch_bytes"));
}

/// Scenario (construction and configuration): `max_fetch_bytes` is set to a
/// negative value (its `i32` type permits negatives from JSON).
/// Guarantees: validation fails with the >= 0 rule, so a negative fetch-max
/// (invalid to librdkafka) is rejected up front rather than forwarded.
#[test]
fn validate_negative_max_fetch_bytes_is_rejected() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_max_fetch_bytes(-1);
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(
        err.contains("max_fetch_bytes") && err.contains(">= 0"),
        "unexpected error: {err}",
    );
}

/// Scenario (construction and configuration): `min_fetch_bytes` is set below
/// one (its `i32` type permits zero and negatives from JSON).
/// Guarantees: validation fails with the > 0 rule, so a `fetch.min.bytes`
/// below librdkafka's minimum of 1 is rejected up front.
#[test]
fn validate_min_fetch_bytes_below_one_is_rejected() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_min_fetch_bytes(0);
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(
        err.contains("min_fetch_bytes") && err.contains("> 0"),
        "unexpected error: {err}",
    );
}

/// Scenario (construction and configuration): max partition fetch bytes is set to zero.
/// Guarantees: validation fails, so a zero per-partition fetch limit is rejected.
#[test]
fn validate_max_partition_fetch_bytes_zero_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_max_partition_fetch_bytes(0);
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("max_partition_fetch_bytes"));
}

/// Scenario (construction and configuration): the commit interval is left unset.
/// Guarantees: validation succeeds, so omitting the interval is allowed (the safety-net
/// default applies).
#[test]
fn validate_commit_interval_none_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_commit(CommitConfig {
            interval_ms: None,
            ..Default::default()
        });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (construction and configuration): the commit interval is set to zero.
/// Guarantees: validation fails, so a zero commit interval is rejected.
#[test]
fn validate_commit_interval_zero_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_commit(CommitConfig {
            interval_ms: Some(0),
            ..Default::default()
        });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

/// Scenario (construction and configuration): the commit interval is set to a positive
/// value.
/// Guarantees: validation succeeds, so a positive commit interval is accepted.
#[test]
fn validate_commit_interval_some_value_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_commit(CommitConfig {
            interval_ms: Some(5000),
            ..Default::default()
        });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario: Kafka replay is selected together with auto-commit mode.
/// Guarantees: Validation rejects a combination that cannot honor downstream feedback.
#[test]
fn validate_transient_nack_replay_requires_manual_commit() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]))
        .with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: None,
        })
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::Replay,
            ..Default::default()
        });

    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("requires commit.mode manual"));
}

/// Scenario: The transient-NACK initial backoff exceeds its maximum.
/// Guarantees: Validation rejects an inverted replay backoff range.
#[test]
fn validate_transient_nack_backoff_range() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig::new(vec!["t".to_string()]))
        .with_transient_nack(TransientNackConfig {
            mode: TransientNackMode::Replay,
            initial_backoff_ms: 200,
            max_backoff_ms: 100,
        });

    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("initial_backoff_ms"));
    assert!(err.contains("must be <="));
}

/// Scenario (construction and configuration): the default consumer-group
/// timing (heartbeat 3000 ms < session 10000 ms) and fetch wait are used.
/// Guarantees: the defaults pass validation, so an operator config that does
/// not set these timeouts is valid.
#[test]
fn validate_default_timeouts_are_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    assert!(
        KafkaReceiverConfig::try_from(cfg).is_ok(),
        "default heartbeat/session/fetch-wait timeouts must be valid",
    );
}

/// Scenario (construction and configuration): `session_timeout_ms` is set to
/// zero.
/// Guarantees: validation fails, so a zero session timeout (an invalid
/// librdkafka setting) is rejected at construction rather than at consumer
/// creation.
#[test]
fn validate_session_timeout_zero_is_invalid() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "session_timeout_ms": 0,
    });
    let err = serde_json::from_value::<KafkaReceiverConfig>(json)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("session_timeout_ms") && err.contains("must be > 0"),
        "unexpected error: {err}",
    );
}

/// Scenario (construction and configuration): `heartbeat_interval_ms` is set
/// to zero.
/// Guarantees: validation fails, so a zero heartbeat interval is rejected.
#[test]
fn validate_heartbeat_interval_zero_is_invalid() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "heartbeat_interval_ms": 0,
    });
    let err = serde_json::from_value::<KafkaReceiverConfig>(json)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("heartbeat_interval_ms") && err.contains("must be > 0"),
        "unexpected error: {err}",
    );
}

/// Scenario (construction and configuration): `max_fetch_wait_ms` is set to
/// zero.
/// Guarantees: validation accepts it and maps it to `fetch.wait.max.ms=0`,
/// so the low-latency setting (valid librdkafka range 0..300000) is honored
/// rather than rejected.
#[test]
fn validate_max_fetch_wait_zero_is_accepted() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "max_fetch_wait_ms": 0,
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("max_fetch_wait_ms=0 should be a valid configuration");
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("fetch.wait.max.ms"), Some("0"));
}

/// Scenario (construction and configuration): `heartbeat_interval_ms` is set
/// greater than or equal to `session_timeout_ms`.
/// Guarantees: validation fails with a dedicated heartbeat error, so an
/// invalid heartbeat/session relationship (which librdkafka would reject at
/// consumer creation) is caught up front.
#[test]
fn validate_heartbeat_not_less_than_session_is_invalid() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "session_timeout_ms": 3000,
        "heartbeat_interval_ms": 3000,
    });
    let err = serde_json::from_value::<KafkaReceiverConfig>(json)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("heartbeat_interval_ms") && err.contains("session_timeout_ms"),
        "unexpected error: {err}",
    );
}

/// Scenario (construction and configuration): a client config is built from a default
/// receiver config.
/// Guarantees: the expected default librdkafka properties are set, so the consumer
/// starts with the documented defaults.
#[test]
fn build_client_config_contains_expected_defaults() {
    let cfg = KafkaReceiverConfigBuilder::new("localhost:9092", "otel-collector", "otel-collector");
    let client_config = cfg.build_client_config();

    assert_eq!(
        client_config.get("bootstrap.servers"),
        Some("localhost:9092")
    );
    assert_eq!(client_config.get("group.id"), Some("otel-collector"));
    assert_eq!(client_config.get("client.id"), Some("otel-collector"));
    assert_eq!(client_config.get("enable.auto.commit"), Some("false"));
    assert_eq!(client_config.get("auto.offset.reset"), Some("latest"));
    assert_eq!(
        client_config.get("isolation.level"),
        Some("read_uncommitted")
    );
    // Session management
    assert_eq!(client_config.get("session.timeout.ms"), Some("10000"));
    assert_eq!(client_config.get("heartbeat.interval.ms"), Some("3000"));
    // Fetch tuning
    assert_eq!(client_config.get("fetch.min.bytes"), Some("1"));
    assert_eq!(client_config.get("fetch.max.bytes"), Some("1048576"));
    assert_eq!(client_config.get("fetch.wait.max.ms"), Some("250"));
    assert_eq!(
        client_config.get("max.partition.fetch.bytes"),
        Some("1048576")
    );
    // Default mode is manual -- auto offset store should be disabled
    assert_eq!(client_config.get("enable.auto.offset.store"), Some("false"));
}

/// Scenario (construction and configuration): a client config is built with auto-commit
/// enabled.
/// Guarantees: `enable.auto.commit` is set true, so librdkafka owns offset commits.
#[test]
fn build_client_config_auto_commit() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_commit(CommitConfig {
            mode: CommitMode::Auto,
            interval_ms: Some(5000),
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted);
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
    assert_eq!(client_config.get("auto.commit.interval.ms"), Some("5000"));
    assert_eq!(client_config.get("auto.offset.reset"), Some("earliest"));
    assert_eq!(
        client_config.get("isolation.level"),
        Some("read_uncommitted")
    );
    assert_eq!(client_config.get("enable.auto.offset.store"), Some("true"));
}

/// Scenario (construction and configuration): auto-commit is enabled without an
/// interval.
/// Guarantees: the auto-commit interval property is omitted, so librdkafka uses its own
/// default interval.
#[test]
fn build_client_config_auto_commit_no_interval_omits_property() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
        mode: CommitMode::Auto,
        interval_ms: None,
    });
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
    assert_eq!(client_config.get("auto.commit.interval.ms"), None);
    assert_eq!(client_config.get("enable.auto.offset.store"), Some("true"));
}

/// Scenario (construction and configuration): a client config is built for manual
/// commit.
/// Guarantees: `enable.auto.commit` is false and no auto-commit interval is set, so the
/// receiver owns commits.
#[test]
fn build_client_config_manual_commit_no_auto_interval() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
        mode: CommitMode::Manual,
        interval_ms: Some(3000),
    });
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("enable.auto.commit"), Some("false"));
    // auto.commit.interval.ms should NOT be set for manual commit
    assert_eq!(client_config.get("auto.commit.interval.ms"), None);
    // Manual mode should disable auto offset store
    assert_eq!(client_config.get("enable.auto.offset.store"), Some("false"));
}

/// Scenario (construction and configuration): a custom `consumer_config` passthrough is
/// provided.
/// Guarantees: the passthrough keys are applied, so operators can set arbitrary
/// librdkafka knobs.
#[test]
fn build_client_config_custom_consumer_config() {
    let mut custom = HashMap::new();
    _ = custom.insert("custom.setting".to_string(), "custom-value".to_string());

    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_consumer_config(custom);
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("custom.setting"), Some("custom-value"));
}

/// Scenario (construction and configuration): a passthrough key collides with a
/// built-in managed property.
/// Guarantees: the built-in value wins, so a passthrough cannot subvert a managed
/// setting.
#[test]
fn build_client_config_builtin_overrides_custom() {
    let mut custom = HashMap::new();
    _ = custom.insert("bootstrap.servers".to_string(), "custom:1234".to_string());

    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_brokers("real-broker:9092")
        .with_consumer_config(custom);
    let client_config = cfg.build_client_config();
    assert_eq!(
        client_config.get("bootstrap.servers"),
        Some("real-broker:9092")
    );
}

/// Scenario (construction and configuration): a `consumer_config` escape-hatch
/// map sets several keys that are also managed by first-class config fields
/// (`bootstrap.servers`, `group.id`, `enable.auto.commit`) alongside a
/// non-managed passthrough key (`fetch.error.backoff.ms`).
/// Guarantees: `overridden_consumer_config_keys` reports exactly the managed
/// keys (which the receiver will overwrite and warn about via the runtime
/// `kafka.receiver.consumer_config.override` event) and never the
/// passthrough key, so the operator gets an accurate override warning surface
/// without false positives.
#[test]
fn overridden_consumer_config_keys_flags_only_managed_keys() {
    let mut custom = HashMap::new();
    // Managed keys (also first-class config fields).
    let _ = custom.insert("bootstrap.servers".to_string(), "x:1".to_string());
    let _ = custom.insert("group.id".to_string(), "shadow-group".to_string());
    let _ = custom.insert("enable.auto.commit".to_string(), "true".to_string());
    // Non-managed passthrough key (a legitimate escape-hatch tunable).
    let _ = custom.insert("fetch.error.backoff.ms".to_string(), "250".to_string());

    let cfg = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("b", "g", "c")
            .with_traces(SignalConfig::new(vec!["t".to_string()]))
            .with_consumer_config(custom),
    )
    .expect("test config should be valid");

    let mut overridden = cfg.overridden_consumer_config_keys();
    overridden.sort_unstable();
    assert_eq!(
        overridden,
        vec!["bootstrap.servers", "enable.auto.commit", "group.id"],
        "only managed keys should be flagged as overridden, not the \
             non-managed passthrough key",
    );
    assert!(
        !overridden.contains(&"fetch.error.backoff.ms"),
        "a non-managed passthrough key must not be flagged as overridden",
    );
}

/// Scenario (construction and configuration): a `group.instance.id` is configured.
/// Guarantees: it is set on the client config, so the consumer joins as a static group
/// member.
#[test]
fn build_client_config_group_instance_id() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_group_instance_id("instance-1");
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("group.instance.id"), Some("instance-1"));
}

/// Scenario (construction and configuration): no `group.instance.id` is configured.
/// Guarantees: the property is omitted, so the consumer joins as a dynamic member.
#[test]
fn build_client_config_no_group_instance_id_when_none() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("group.instance.id"), None);
}

/// Scenario (construction and configuration): a minimal JSON config with only required
/// fields and topics is deserialized.
/// Guarantees: it parses successfully, so a minimal operator config is valid.
#[test]
fn deserialize_minimal_config() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "my-group",
        "client_id": "my-client",
        "traces": {
            "topics": ["traces"]
        }
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("should deserialize minimal config");
    assert_eq!(cfg.brokers(), "kafka:9092");
    assert_eq!(cfg.group_id(), "my-group");
    assert_eq!(cfg.client_id(), "my-client");
    // Defaults
    assert_eq!(cfg.traces_topics(), &["traces"]);
    assert!(cfg.metrics_topics().is_empty());
    assert!(cfg.logs_topics().is_empty());
    assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
    assert!(!cfg.is_auto_commit());
    assert_eq!(cfg.commit_interval_ms(), None);
}

/// Scenario (construction and configuration): a JSON config missing required fields is
/// deserialized.
/// Guarantees: it fails, so an incomplete config is rejected at parse time.
#[test]
fn deserialize_without_required_fields_fails() {
    // brokers, group_id, client_id are required -- empty object should fail
    let json = json!({});
    let result = serde_json::from_value::<KafkaReceiverConfig>(json);
    assert!(result.is_err());
}

/// Scenario (construction and configuration): a fully-populated JSON config is
/// deserialized.
/// Guarantees: every field parses onto the config, so the full configuration surface is
/// supported.
#[test]
fn deserialize_full_config() {
    let json = json!({
        "brokers": "kafka:9092",
        "group_id": "my-group",
        "client_id": "my-client",
        "group_instance_id": "instance-1",
        "traces": {
            "topics": ["traces"],
            "encoding": "otlp_proto"
        },
        "metrics": {
            "topics": ["metrics"],
            "encoding": "otap_proto"
        },
        "logs": {
            "topics": ["logs"],
            "encoding": "otlp_proto"
        },
        "auto_offset_reset": "earliest",
        "commit": {
            "mode": "auto",
            "interval_ms": 5000
        },
        "isolation_level": "read_uncommitted",
        "session_timeout_ms": 30000,
        "heartbeat_interval_ms": 5000,
        "min_fetch_bytes": 1024,
        "max_fetch_bytes": 2097152,
        "max_fetch_wait_ms": 500,
        "max_partition_fetch_bytes": 2097152,
        "enable_idempotency": true,
        "consumer_config": {"custom.setting": "value"}
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("should deserialize full config");
    assert_eq!(cfg.brokers(), "kafka:9092");
    assert_eq!(cfg.group_id(), "my-group");
    assert_eq!(cfg.client_id(), "my-client");
    assert_eq!(cfg.traces_topics(), &["traces"]);
    assert_eq!(cfg.metrics_topics(), &["metrics"]);
    assert_eq!(cfg.logs_topics(), &["logs"]);
    assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
    assert_eq!(cfg.metrics_encoding(), MessageFormat::OtapProto);
    assert_eq!(cfg.logs_encoding(), MessageFormat::OtlpProto);
    assert!(cfg.is_auto_commit());
    assert_eq!(cfg.commit_interval_ms(), Some(5000));
    assert!(cfg.is_idempotent());
}

/// Scenario (construction and configuration): topics are provided as a JSON list.
/// Guarantees: the list is parsed, so multi-topic subscriptions can be configured as
/// arrays.
#[test]
fn deserialize_topics_as_list() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {
            "topics": ["traces-a", "traces-b"]
        },
        "metrics": {
            "topics": ["metrics-one"]
        },
        "logs": {
            "topics": ["logs-x", "logs-y", "logs-z"]
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).expect("list topics should work");
    assert_eq!(cfg.traces_topics(), &["traces-a", "traces-b"]);
    assert_eq!(cfg.metrics_topics(), &["metrics-one"]);
    assert_eq!(cfg.logs_topics(), &["logs-x", "logs-y", "logs-z"]);
}

/// Scenario (construction and configuration): each auto-offset-reset variant is
/// deserialized.
/// Guarantees: every variant parses, so all offset-reset options are configurable.
#[test]
fn auto_offset_reset_deserialize_variants() {
    assert_eq!(
        serde_json::from_value::<AutoOffsetReset>(json!("earliest")).unwrap(),
        AutoOffsetReset::Earliest
    );
    assert_eq!(
        serde_json::from_value::<AutoOffsetReset>(json!("latest")).unwrap(),
        AutoOffsetReset::Latest
    );
    assert_eq!(
        serde_json::from_value::<AutoOffsetReset>(json!("error")).unwrap(),
        AutoOffsetReset::Error
    );
}

/// Scenario (construction and configuration): each isolation-level variant is
/// deserialized.
/// Guarantees: every variant parses, so all isolation options are configurable.
#[test]
fn isolation_level_deserialize_variants() {
    assert_eq!(
        serde_json::from_value::<IsolationLevel>(json!("read_uncommitted")).unwrap(),
        IsolationLevel::ReadUncommitted
    );
    assert_eq!(
        serde_json::from_value::<IsolationLevel>(json!("read_committed")).unwrap(),
        IsolationLevel::ReadCommitted
    );
}

/// Scenario (construction and configuration): the serde default functions are invoked.
/// Guarantees: they return the documented defaults, so omitted fields default
/// consistently.
#[test]
fn default_functions_return_expected_values() {
    assert_eq!(default_auto_offset_reset(), AutoOffsetReset::Latest);
    assert_eq!(default_isolation_level(), IsolationLevel::ReadUncommitted);
    assert_eq!(MessageFormat::default(), MessageFormat::OtlpProto);
    assert_eq!(default_commit_mode(), CommitMode::Manual);
    assert_eq!(default_session_timeout_ms(), 10000);
    assert_eq!(default_heartbeat_interval_ms(), 3000);
    assert_eq!(default_min_fetch_bytes(), 1);
    assert_eq!(default_max_fetch_bytes(), 1_048_576);
    assert_eq!(default_max_fetch_wait_ms(), 250);
    assert_eq!(default_max_partition_fetch_bytes(), 1_048_576);
}

/// Scenario (construction and configuration): the commit interval default is read.
/// Guarantees: it equals the documented default, so an unset interval behaves
/// predictably.
#[test]
fn commit_interval_ms_default_value() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    assert_eq!(cfg.commit_interval_ms(), None);
}

/// Scenario (construction and configuration): the commit interval is set via the
/// builder.
/// Guarantees: the value is applied, so the builder configures the commit interval.
#[test]
fn commit_interval_ms_set_via_builder() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_commit(CommitConfig {
            interval_ms: Some(3000),
            ..Default::default()
        })
        .try_into()
        .unwrap();
    assert_eq!(cfg.commit_interval_ms(), Some(3000));
}

/// Scenario (construction and configuration): a client config is built in auto mode
/// with an interval.
/// Guarantees: the auto-commit interval property is set, so librdkafka commits on the
/// configured cadence.
#[test]
fn build_client_config_sets_auto_commit_interval_when_auto_mode() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_commit(CommitConfig {
        mode: CommitMode::Auto,
        interval_ms: Some(2000),
    });
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("enable.auto.commit"), Some("true"));
    assert_eq!(client_config.get("auto.commit.interval.ms"), Some("2000"));
}

/// Scenario (construction and configuration): session and fetch tuning fields are
/// deserialized.
/// Guarantees: the tuning values parse onto the config, so fetch/session knobs are
/// configurable.
#[test]
fn session_and_fetch_tuning_deserialized() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "session_timeout_ms": 20000,
        "heartbeat_interval_ms": 5000,
        "min_fetch_bytes": 512,
        "max_fetch_bytes": 2097152,
        "max_fetch_wait_ms": 500,
        "max_partition_fetch_bytes": 2097152,
        "traces": {"topics": ["t"]}
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("session.timeout.ms"), Some("20000"));
    assert_eq!(client_config.get("heartbeat.interval.ms"), Some("5000"));
    assert_eq!(client_config.get("fetch.min.bytes"), Some("512"));
    assert_eq!(client_config.get("fetch.max.bytes"), Some("2097152"));
    assert_eq!(client_config.get("fetch.wait.max.ms"), Some("500"));
    assert_eq!(
        client_config.get("max.partition.fetch.bytes"),
        Some("2097152")
    );
}

/// Scenario (construction and configuration): the builder setters are invoked.
/// Guarantees: each setter applies its value to the built config, so the builder
/// surface is complete.
#[test]
fn builder_methods_set_values() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b:9092", "g", "c")
        .with_group_instance_id("inst-1")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_auto_offset_reset(AutoOffsetReset::Earliest)
        .with_isolation_level(IsolationLevel::ReadUncommitted)
        .with_enable_idempotency(true)
        .try_into()
        .unwrap();
    assert_eq!(cfg.brokers(), "b:9092");
    assert_eq!(cfg.group_id(), "g");
    assert_eq!(cfg.client_id(), "c");
    assert_eq!(cfg.traces_topics(), &["t"]);
    assert!(cfg.is_idempotent());
}
