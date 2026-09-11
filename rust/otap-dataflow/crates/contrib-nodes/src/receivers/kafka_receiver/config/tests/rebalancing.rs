// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-group rebalancing config tests.

use super::*;

// ---- Consumer-group rebalancing ----

/// Scenario (consumer-group rebalancing): the `range` rebalance strategy is mapped to
/// its librdkafka value.
/// Guarantees: it maps to `"range"`, so the strategy reaches librdkafka correctly.
#[test]
fn rebalance_strategy_to_librdkafka_value_range() {
    assert_eq!(RebalanceStrategy::Range.to_librdkafka_value(), "range");
}

/// Scenario (consumer-group rebalancing): the `roundrobin` rebalance strategy is mapped
/// to its librdkafka value.
/// Guarantees: it maps to `"roundrobin"`, so the strategy reaches librdkafka correctly.
#[test]
fn rebalance_strategy_to_librdkafka_value_round_robin() {
    assert_eq!(
        RebalanceStrategy::RoundRobin.to_librdkafka_value(),
        "roundrobin"
    );
}

/// Scenario (consumer-group rebalancing): the `cooperative-sticky` rebalance strategy
/// is mapped to its librdkafka value.
/// Guarantees: it maps to `"cooperative-sticky"`, so the strategy reaches librdkafka
/// correctly.
#[test]
fn rebalance_strategy_to_librdkafka_value_cooperative_sticky() {
    assert_eq!(
        RebalanceStrategy::CooperativeSticky.to_librdkafka_value(),
        "cooperative-sticky"
    );
}

/// Scenario (consumer-group rebalancing): a client config is built with a rebalance
/// strategy configured.
/// Guarantees: `partition.assignment.strategy` is set, so the operator-selected
/// strategy reaches the consumer.
#[test]
fn build_client_config_sets_rebalance_strategy() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_rebalance_strategy(RebalanceStrategy::CooperativeSticky);
    let client_config = cfg.build_client_config();
    assert_eq!(
        client_config.get("partition.assignment.strategy"),
        Some("cooperative-sticky")
    );
}

/// Scenario (consumer-group rebalancing): a client config is built without a rebalance
/// strategy.
/// Guarantees: the property is omitted, so librdkafka's default assignor applies.
#[test]
fn build_client_config_no_rebalance_strategy_when_none() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c");
    let client_config = cfg.build_client_config();
    assert_eq!(client_config.get("partition.assignment.strategy"), None);
}

/// Scenario (consumer-group rebalancing): a config with a rebalance strategy is
/// deserialized.
/// Guarantees: the strategy parses, so it is configurable via JSON.
#[test]
fn deserialize_config_with_rebalance_strategy() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "rebalance_strategy": "cooperative_sticky"
    });
    let cfg: KafkaReceiverConfig =
        serde_json::from_value(json).expect("should deserialize config with rebalance_strategy");
    assert_eq!(
        cfg.rebalance_strategy(),
        Some(RebalanceStrategy::CooperativeSticky)
    );
}

/// Scenario (consumer-group rebalancing): the rebalance-strategy getter is read on a
/// default config.
/// Guarantees: it returns none, so the default assignor is used unless configured.
#[test]
fn rebalance_strategy_getter_returns_none_by_default() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .try_into()
        .unwrap();
    assert_eq!(cfg.rebalance_strategy(), None);
}

/// Scenario (consumer-group rebalancing): a rebalance strategy is set via the builder.
/// Guarantees: the value is applied, so the builder configures the assignment strategy.
#[test]
fn builder_with_rebalance_strategy() {
    let cfg: KafkaReceiverConfig = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_rebalance_strategy(RebalanceStrategy::RoundRobin)
        .try_into()
        .unwrap();
    assert_eq!(
        cfg.rebalance_strategy(),
        Some(RebalanceStrategy::RoundRobin)
    );
}
