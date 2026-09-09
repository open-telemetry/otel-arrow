// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic routing and payload/topic configuration tests.

use super::*;

// ---- Routing and payload correctness ----

/// Scenario (routing and payload correctness): traces, metrics, and logs are configured
/// on fully distinct topics.
/// Guarantees: validation succeeds, so a disjoint multi-signal topic layout is
/// accepted.
#[test]
fn validate_all_distinct_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
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
        });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): a logs signal selects Syslog encoding.
/// Guarantees: validation accepts Syslog for logs so Kafka records can use the shared
/// Syslog decoder.
#[test]
fn validate_syslog_encoding_for_logs_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_logs(
        SignalConfig::new(vec!["logs-topic".to_string()]).with_encoding(MessageFormat::Syslog),
    );
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): a traces signal selects Syslog encoding.
/// Guarantees: validation rejects the logs-only encoding before the receiver starts.
#[test]
fn validate_syslog_encoding_for_traces_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(
        SignalConfig::new(vec!["traces-topic".to_string()]).with_encoding(MessageFormat::Syslog),
    );
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("syslog encoding is not supported for traces"));
}

/// Scenario (routing and payload correctness): a metrics signal selects Syslog encoding.
/// Guarantees: validation rejects the logs-only encoding before the receiver starts.
#[test]
fn validate_syslog_encoding_for_metrics_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_metrics(
        SignalConfig::new(vec!["metrics-topic".to_string()]).with_encoding(MessageFormat::Syslog),
    );
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("syslog encoding is not supported for metrics"));
}

/// Scenario (routing and payload correctness): no signal has any topic configured.
/// Guarantees: validation fails, so a receiver that would subscribe to nothing is
/// rejected.
#[test]
fn validate_all_empty_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("at least one signal"));
}

/// Scenario (routing and payload correctness): traces and metrics share a topic.
/// Guarantees: validation fails, so one topic cannot feed two signal decoders.
#[test]
fn validate_traces_equals_metrics_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        })
        .with_metrics(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("overlap"));
}

/// Scenario (routing and payload correctness): traces and logs share a topic.
/// Guarantees: validation fails, so one topic cannot feed two signal decoders.
#[test]
fn validate_traces_equals_logs_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("overlap"));
}

/// Scenario (routing and payload correctness): metrics and logs share a topic.
/// Guarantees: validation fails, so one topic cannot feed two signal decoders.
#[test]
fn validate_metrics_equals_logs_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_metrics(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["same-topic".to_string()],
            ..Default::default()
        });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("overlap"));
}

/// Scenario (routing and payload correctness): only one signal is configured and the
/// others are empty.
/// Guarantees: validation succeeds, so a single-signal receiver is valid.
#[test]
fn validate_one_set_others_empty_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["only-traces".to_string()],
        ..Default::default()
    });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): two signals share one topic within
/// larger multi-topic lists.
/// Guarantees: validation fails, so any cross-signal topic overlap is rejected.
#[test]
fn validate_multi_topic_overlap_across_signals_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["topic-a".to_string(), "topic-b".to_string()],
            ..Default::default()
        })
        .with_metrics(SignalConfig {
            topics: vec!["topic-c".to_string(), "topic-b".to_string()],
            ..Default::default()
        });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("overlap"));
}

/// Scenario (routing and payload correctness): signals have multi-topic lists that are
/// pairwise disjoint.
/// Guarantees: validation succeeds, so disjoint multi-topic layouts are accepted.
#[test]
fn validate_multi_topic_disjoint_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["traces-a".to_string(), "traces-b".to_string()],
            ..Default::default()
        })
        .with_metrics(SignalConfig {
            topics: vec!["metrics-a".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["logs-a".to_string(), "logs-b".to_string()],
            ..Default::default()
        });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): a signal is configured with a
/// syntactically invalid topic regex.
/// Guarantees: validation fails, so a bad regex is rejected before the receiver starts.
#[test]
fn validate_invalid_regex_topic_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-(invalid".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("invalid regex topic pattern in traces"));
}

/// Scenario (routing and payload correctness): a signal is configured with a valid
/// `^`-anchored topic regex.
/// Guarantees: validation succeeds, so regex topic subscriptions are accepted.
#[test]
fn validate_valid_regex_topic_succeeds() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string()],
        ..Default::default()
    });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): exclude_topics is set on a signal whose
/// include topics are not regexes.
/// Guarantees: validation fails, so exclude patterns are only allowed alongside regex
/// includes.
#[test]
fn validate_exclude_topics_without_regex_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["traces-prod".to_string()],
        exclude_topics: vec!["^traces-test$".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("exclude_topics is only allowed when"));
}

/// Scenario (routing and payload correctness): exclude_topics is set alongside a regex
/// include.
/// Guarantees: validation succeeds, so regex include plus exclude patterns compose.
#[test]
fn validate_exclude_topics_with_regex_succeeds() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string()],
        exclude_topics: vec!["^traces-test$".to_string()],
        ..Default::default()
    });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): an exclude_topics entry is an empty
/// string.
/// Guarantees: validation fails, so an empty exclude pattern is rejected.
#[test]
fn validate_exclude_topics_empty_string_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string()],
        exclude_topics: vec!["".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("non-empty"));
}

/// Scenario (routing and payload correctness): an exclude_topics entry is an invalid
/// regex.
/// Guarantees: validation fails, so a bad exclude regex is rejected.
#[test]
fn validate_exclude_topics_invalid_regex_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string()],
        exclude_topics: vec!["^(invalid".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("invalid regex in traces.exclude_topics"));
}

/// Scenario (routing and payload correctness): a literal topic name is empty.
/// Guarantees: validation fails, so an empty topic name is rejected.
#[test]
fn validate_empty_topic_name_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("traces.topics"), "unexpected error: {err}");
    assert!(err.contains("empty"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): a literal topic name is `.`.
/// Guarantees: validation fails, so the reserved `.` topic name is rejected.
#[test]
fn validate_dot_topic_name_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_metrics(SignalConfig {
        topics: vec![".".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("metrics.topics"), "unexpected error: {err}");
    assert!(err.contains("ambiguous"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): a literal topic name is `..`.
/// Guarantees: validation fails, so the reserved `..` topic name is rejected.
#[test]
fn validate_dotdot_topic_name_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_logs(SignalConfig {
        topics: vec!["..".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("logs.topics"), "unexpected error: {err}");
    assert!(err.contains("ambiguous"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): a literal topic name contains characters
/// outside the Kafka-allowed set.
/// Guarantees: validation fails, so a syntactically invalid topic name is rejected.
#[test]
fn validate_topic_name_invalid_chars_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["bad/topic".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("traces.topics"), "unexpected error: {err}");
    assert!(err.contains("invalid character"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): a literal topic name exceeds the Kafka
/// length limit.
/// Guarantees: validation fails, so an over-long topic name is rejected.
#[test]
fn validate_topic_name_too_long_fails() {
    let long_topic = "a".repeat(250);
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec![long_topic],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("traces.topics"), "unexpected error: {err}");
    assert!(err.contains("maximum length"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): a topic entry is a regex pattern.
/// Guarantees: Kafka name-syntax validation is skipped for it, so regex patterns are
/// not rejected as invalid names.
#[test]
fn validate_regex_topic_skips_name_validation() {
    // Regex patterns start with '^' and should NOT be validated as literal topic names
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string()],
        ..Default::default()
    });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): a signal mixes literal and regex topic
/// entries.
/// Guarantees: validation succeeds, so literal and regex topics can be combined.
#[test]
fn validate_mixed_literal_and_regex_topics() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["valid-literal".to_string(), "^traces-.*".to_string()],
        ..Default::default()
    });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): an invalid literal name appears among
/// valid regex topics.
/// Guarantees: validation fails, so a bad literal is still caught in a mixed list.
#[test]
fn validate_invalid_literal_among_regex_topics_fails() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["^traces-.*".to_string(), "bad topic".to_string()],
        ..Default::default()
    });
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("traces.topics"), "unexpected error: {err}");
    assert!(err.contains("invalid character"), "unexpected error: {err}");
}

/// Scenario (routing and payload correctness): valid literal topics are configured
/// across signals.
/// Guarantees: validation succeeds, so a well-formed literal topic layout is accepted.
#[test]
fn validate_valid_literal_topics_across_signals() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["my-traces".to_string()],
            ..Default::default()
        })
        .with_metrics(SignalConfig {
            topics: vec!["my.metrics".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["my_logs".to_string()],
            ..Default::default()
        });
    assert!(KafkaReceiverConfig::try_from(cfg).is_ok());
}

/// Scenario (routing and payload correctness): all three signals are configured with
/// topics.
/// Guarantees: `all_topics` returns every configured topic, so the subscription set
/// covers all signals.
#[test]
fn all_topics_returns_all_configured() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
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
    let topics = cfg.all_topics();
    assert_eq!(topics.len(), 3);
    assert!(topics.contains(&"traces-topic"));
    assert!(topics.contains(&"metrics-topic"));
    assert!(topics.contains(&"logs-topic"));
}

/// Scenario (routing and payload correctness): no signal has topics.
/// Guarantees: `all_topics` is empty, so nothing is subscribed.
#[test]
fn all_topics_returns_empty_when_none_configured() {
    // Validation requires at least one signal with topics, so we verify
    // that a builder with no topics fails validation.
    let result = KafkaReceiverConfig::try_from(KafkaReceiverConfigBuilder::new("b", "g", "c"));
    assert!(result.is_err());
}

/// Scenario (routing and payload correctness): only some signals are configured.
/// Guarantees: `all_topics` returns only the configured signals' topics, so
/// unconfigured signals contribute nothing.
#[test]
fn all_topics_returns_only_set_topics() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["l".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    let topics = cfg.all_topics();
    assert_eq!(topics.len(), 2);
    assert!(topics.contains(&"t"));
    assert!(topics.contains(&"l"));
}

/// Scenario (routing and payload correctness): signals have multi-topic lists.
/// Guarantees: `all_topics` returns the flattened union, so multi-topic subscriptions
/// are fully covered.
#[test]
fn all_topics_returns_flattened_multi_topic_list() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t1".to_string(), "t2".to_string()],
            ..Default::default()
        })
        .with_metrics(SignalConfig {
            topics: vec!["m1".to_string()],
            ..Default::default()
        })
        .with_logs(SignalConfig {
            topics: vec!["l1".to_string(), "l2".to_string(), "l3".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    let topics = cfg.all_topics();
    assert_eq!(topics.len(), 6);
    assert!(topics.contains(&"t1"));
    assert!(topics.contains(&"t2"));
    assert!(topics.contains(&"m1"));
    assert!(topics.contains(&"l1"));
    assert!(topics.contains(&"l2"));
    assert!(topics.contains(&"l3"));
}

/// Scenario (routing and payload correctness): a header-extraction config is
/// deserialized.
/// Guarantees: the extraction rules parse, so header-to-attribute mapping is
/// configurable.
#[test]
fn header_extraction_deserialize() {
    let json = json!({"key": "tenant.id", "value_type": "string"});
    let extraction: HeaderExtraction = serde_json::from_value(json).unwrap();
    assert_eq!(
        extraction,
        HeaderExtraction {
            key: "tenant.id".to_string(),
            value_type: AttributeValueType::String,
        }
    );
}

/// Scenario (routing and payload correctness): a config with
/// resource-attrs-from-headers is deserialized.
/// Guarantees: the mapping parses onto the config, so header-derived resource
/// attributes are configurable.
#[test]
fn deserialize_config_with_resource_attrs_from_headers() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["traces"]},
        "resource_attrs_from_headers": {
            "x-tenant-id": {"key": "tenant.id", "value_type": "string"},
            "x-env": {"key": "deployment.env", "value_type": "string"}
        }
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json)
        .expect("should deserialize config with resource_attrs_from_headers");
    let extractors = cfg.resource_attrs_from_headers();
    assert_eq!(extractors.len(), 2);
    assert_eq!(
        extractors.get("x-tenant-id"),
        Some(&HeaderExtraction {
            key: "tenant.id".to_string(),
            value_type: AttributeValueType::String,
        })
    );
    assert_eq!(
        extractors.get("x-env"),
        Some(&HeaderExtraction {
            key: "deployment.env".to_string(),
            value_type: AttributeValueType::String,
        })
    );
}

/// Scenario (routing and payload correctness): the resource-attrs-from-headers getter
/// is read.
/// Guarantees: it returns the configured mapping, so downstream extraction reads a
/// stable accessor.
#[test]
fn getter_returns_resource_attrs_from_headers() {
    let mut extractors = HashMap::new();
    _ = extractors.insert(
        "x-tenant".to_string(),
        HeaderExtraction {
            key: "tenant".to_string(),
            value_type: AttributeValueType::String,
        },
    );
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_resource_attrs_from_headers(extractors.clone())
        .try_into()
        .unwrap();
    assert_eq!(cfg.resource_attrs_from_headers(), &extractors);
}

/// Scenario (routing and payload correctness): every attribute value type is
/// deserialized.
/// Guarantees: each type parses, so all header attribute value kinds are configurable.
#[test]
fn attribute_value_type_deserialize_all_variants() {
    assert_eq!(
        serde_json::from_value::<AttributeValueType>(json!("string")).unwrap(),
        AttributeValueType::String
    );
    assert_eq!(
        serde_json::from_value::<AttributeValueType>(json!("bool")).unwrap(),
        AttributeValueType::Bool
    );
    assert_eq!(
        serde_json::from_value::<AttributeValueType>(json!("int")).unwrap(),
        AttributeValueType::Int
    );
    assert_eq!(
        serde_json::from_value::<AttributeValueType>(json!("float")).unwrap(),
        AttributeValueType::Float
    );
}

/// Scenario (routing and payload correctness): a header extraction using every value
/// type is deserialized.
/// Guarantees: all value types parse together, so mixed-type header extraction is
/// configurable.
#[test]
fn header_extraction_deserialize_all_value_types() {
    let cases = vec![
        ("string", AttributeValueType::String),
        ("bool", AttributeValueType::Bool),
        ("int", AttributeValueType::Int),
        ("float", AttributeValueType::Float),
    ];
    for (type_str, expected_type) in cases {
        let json = json!({"key": "k", "value_type": type_str});
        let extraction: HeaderExtraction = serde_json::from_value(json).unwrap();
        assert_eq!(
            extraction,
            HeaderExtraction {
                key: "k".to_string(),
                value_type: expected_type,
            }
        );
    }
}

/// Scenario (routing and payload correctness): a signal is configured without an
/// explicit encoding.
/// Guarantees: it defaults to OTLP proto, so the zero-copy default encoding applies.
#[test]
fn per_signal_encoding_defaults_to_otlp_proto() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
    assert_eq!(cfg.metrics_encoding(), MessageFormat::OtlpProto);
    assert_eq!(cfg.logs_encoding(), MessageFormat::OtlpProto);
}

/// Scenario (routing and payload correctness): signals are configured with different
/// encodings.
/// Guarantees: each signal keeps its own encoding, so per-signal encoding is
/// independent.
#[test]
fn per_signal_encoding_can_differ() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"], "encoding": "otlp_proto"},
        "metrics": {"topics": ["m"], "encoding": "otap_proto"},
        "logs": {"topics": ["l"], "encoding": "otap_proto"}
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).unwrap();
    assert_eq!(cfg.traces_encoding(), MessageFormat::OtlpProto);
    assert_eq!(cfg.metrics_encoding(), MessageFormat::OtapProto);
    assert_eq!(cfg.logs_encoding(), MessageFormat::OtapProto);
}
