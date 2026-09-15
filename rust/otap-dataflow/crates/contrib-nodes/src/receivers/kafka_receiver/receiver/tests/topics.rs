// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Topic matching and CallData codec unit tests: literal/regex topic routing,
//! the topic registry, and Ack/Nack CallData encode/decode.

use super::*;

/// Scenario (routing and payload correctness): a topic list of exact names (no regexes)
/// is matched against candidate topics.
/// Guarantees: only exact-name matches succeed and an empty list matches nothing, so
/// exact topic subscription is precise.
#[test]
fn matches_any_topic_exact() {
    let topics = vec!["traces".to_string()];
    let regexes = vec![None];
    assert!(matches_any_topic(&topics, &regexes, "traces"));
    assert!(!matches_any_topic(&topics, &regexes, "other"));

    // Empty list matches nothing
    assert!(!matches_any_topic(&[], &[], "traces"));
}

/// Scenario (routing and payload correctness): a `^`-anchored regex topic pattern is
/// matched against candidate topics.
/// Guarantees: topics matching the regex are accepted and non-matching topics rejected,
/// so regex topic subscription works.
#[test]
fn matches_any_topic_regex() {
    let topics = vec!["^traces-.*".to_string()];
    let re = Regex::new("^traces-.*").unwrap();
    let regexes = vec![Some(re)];
    assert!(matches_any_topic(&topics, &regexes, "traces-prod"));
    assert!(matches_any_topic(&topics, &regexes, "traces-staging"));
    assert!(!matches_any_topic(&topics, &regexes, "metrics-prod"));
}

/// Scenario (routing and payload correctness): a mixed list of exact names and a regex
/// is matched against candidate topics.
/// Guarantees: a topic matching any exact entry or the regex is accepted and all others
/// rejected, so mixed exact/regex lists compose correctly.
#[test]
fn matches_any_topic_multi_topic_list() {
    let topics = vec![
        "traces-a".to_string(),
        "traces-b".to_string(),
        "^traces-regex-.*".to_string(),
    ];
    let re = Regex::new("^traces-regex-.*").unwrap();
    let regexes = vec![None, None, Some(re)];

    assert!(matches_any_topic(&topics, &regexes, "traces-a"));
    assert!(matches_any_topic(&topics, &regexes, "traces-b"));
    assert!(matches_any_topic(&topics, &regexes, "traces-regex-foo"));
    assert!(!matches_any_topic(&topics, &regexes, "traces-c"));
    assert!(!matches_any_topic(&topics, &regexes, "metrics"));
}

/// Scenario (routing and payload correctness): a real receiver's compiled per-signal
/// topic matchers are queried for regex traces, exact metrics, and unconfigured logs.
/// Guarantees: each candidate routes to the correct signal (or none), so the receiver's
/// compiled topic regexes drive per-signal dispatch.
#[test]
fn matches_topic_routing_with_receiver() {
    let cfg = make_config(&["^traces-.*"], &["metrics"], &[], MessageFormat::OtlpProto);
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Regex traces
    assert!(matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-prod",
    ));
    assert!(matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-staging",
    ));

    // Exact metrics
    assert!(matches_any_topic(
        receiver.config.metrics_topics(),
        &receiver.metrics_topic_regexes,
        "metrics",
    ));
    assert!(!matches_any_topic(
        receiver.config.metrics_topics(),
        &receiver.metrics_topic_regexes,
        "metrics-prod",
    ));

    // Unconfigured logs
    assert!(!matches_any_topic(
        receiver.config.logs_topics(),
        &receiver.logs_topic_regexes,
        "logs-prod",
    ));
}

/// Scenario (routing and payload correctness): a receiver configured with several exact
/// and regex topics per signal is queried across candidates.
/// Guarantees: each candidate matches only the intended signal's topic set, so
/// multi-topic per-signal routing is correct.
#[test]
fn matches_topic_routing_multi_topic_receiver() {
    let cfg = make_config(
        &["traces-a", "traces-b", "^traces-regex-.*"],
        &["metrics-x", "metrics-y"],
        &["logs"],
        MessageFormat::OtlpProto,
    );
    let ctx = make_pipeline_ctx();
    let receiver = KafkaReceiver::new(ctx, cfg).expect("should create");

    // Multiple traces topics
    assert!(matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-a",
    ));
    assert!(matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-b",
    ));
    assert!(matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-regex-prod",
    ));
    assert!(!matches_any_topic(
        receiver.config.traces_topics(),
        &receiver.traces_topic_regexes,
        "traces-c",
    ));

    // Multiple metrics topics
    assert!(matches_any_topic(
        receiver.config.metrics_topics(),
        &receiver.metrics_topic_regexes,
        "metrics-x",
    ));
    assert!(matches_any_topic(
        receiver.config.metrics_topics(),
        &receiver.metrics_topic_regexes,
        "metrics-y",
    ));
    assert!(!matches_any_topic(
        receiver.config.metrics_topics(),
        &receiver.metrics_topic_regexes,
        "metrics-z",
    ));

    // Single logs topic still works
    assert!(matches_any_topic(
        receiver.config.logs_topics(),
        &receiver.logs_topic_regexes,
        "logs",
    ));
}

/// Scenario (routing and payload correctness): a signal is configured with a
/// syntactically invalid topic regex.
/// Guarantees: config validation fails at construction, so an invalid regex is rejected
/// before the receiver starts.
#[test]
fn invalid_regex_topic_fails_at_construction() {
    // Unbalanced parenthesis is an invalid regex -- rejected at config validation time
    let result = KafkaReceiverConfig::try_from(
        KafkaReceiverConfigBuilder::new("unused:9092", "g", "c")
            .with_traces(SignalConfig::new(vec!["^traces-(".to_string()])),
    );
    assert!(
        result.is_err(),
        "invalid regex should fail at config construction"
    );
}

/// Scenario (routing and payload correctness): distinct topics are interned into the
/// topic registry.
/// Guarantees: each new topic gets a sequential id and a repeat lookup returns the same
/// id, so topic ids are stable and `Copy`.
#[test]
fn topic_registry_assigns_sequential_ids() {
    let mut reg = TopicRegistry::new();

    assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
    assert_eq!(reg.get_or_assign("metrics-prod"), Some(1));
    assert_eq!(reg.get_or_assign("logs-prod"), Some(2));

    // Same topic returns the same ID.
    assert_eq!(reg.get_or_assign("traces-prod"), Some(0));
}

/// Scenario (routing and payload correctness): a topic is interned and then looked up
/// by id.
/// Guarantees: the id maps back to the original name and an unknown id returns `None`,
/// so the id/name mapping round-trips.
#[test]
fn topic_registry_name_for_roundtrip() {
    let mut reg = TopicRegistry::new();

    let id = reg.get_or_assign("my-topic").expect("id assigned");
    assert_eq!(reg.name_for(id).as_deref(), Some("my-topic"));
    assert_eq!(reg.name_for(99), None);
}

/// Scenario (routing and payload correctness): a range of (topic_id, partition, offset,
/// generation) tuples, including values that overflow the legacy `u8` id, are encoded
/// into calldata and decoded back.
/// Guarantees: every field round-trips exactly, so the offset-correlation calldata is
/// lossless across the full value range.
#[test]
fn encode_decode_calldata_roundtrip() {
    let cases: Vec<(u32, i32, i64, u64)> = vec![
        (0, 0, 0, 0),
        (0, 0, 100, 1),
        (1, 3, 999_999, 7),
        (2, 11, i64::MAX, u64::MAX),
        (5, 0, 42, 0),
        (10, 1, 1_000_000, 12_345),
        (255, 2, 0, 3),
        // Values that would have been truncated by the old `u8` ID.
        (256, 7, 1, 1),
        (65_536, 9, 2, 2),
        (u32::MAX, i32::MAX, i64::MAX, u64::MAX),
        (u32::MAX, -1, 0, 9),
    ];

    for (topic_id, partition, offset, generation) in cases {
        let generation = DeliveryGeneration::from_raw(generation);
        let calldata = encode_calldata(topic_id, partition, offset, generation);
        let (dec_tid, dec_part, dec_off, dec_gen) = decode_calldata(&calldata);
        assert_eq!(dec_tid, topic_id, "topic_id mismatch");
        assert_eq!(dec_part, partition, "partition mismatch");
        assert_eq!(dec_off, offset, "offset mismatch");
        assert_eq!(dec_gen, generation, "generation mismatch");
    }
}

/// Scenario (routing and payload correctness): a (topic_id, partition, offset,
/// generation) tuple is encoded.
/// Guarantees: the calldata occupies exactly three slots, pinning the on-wire calldata
/// layout.
#[test]
fn encode_calldata_produces_three_slots() {
    let calldata = encode_calldata(1, 5, 42, DeliveryGeneration::from_raw(3));
    assert_eq!(calldata.len(), 3);
}

/// Scenario (routing and payload correctness): a legacy two-slot calldata (no
/// generation slot) is decoded.
/// Guarantees: the missing generation defaults to 0 while the other fields decode
/// correctly, so older calldata stays backward-compatible.
#[test]
fn decode_legacy_two_slot_calldata_defaults_generation_zero() {
    // A calldata without the generation slot decodes as generation 0.
    let legacy: CallData = smallvec![
        Context8u8::from(((7u64) << 32) | 5u64),
        Context8u8::from(42u64),
    ];
    let (topic_id, partition, offset, generation) = decode_calldata(&legacy);
    assert_eq!(topic_id, 7);
    assert_eq!(partition, 5);
    assert_eq!(offset, 42);
    assert_eq!(generation, DeliveryGeneration::from_raw(0));
}
