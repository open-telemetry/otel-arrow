// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Operational-visibility config tests: lag-refresh interval validation and idempotency defaults.

use super::*;

// ---- Operational visibility ----

/// Scenario (operational visibility): the lag-refresh interval is left unset.
/// Guarantees: validation succeeds, so omitting the lag-refresh interval disables
/// periodic lag refresh.
#[test]
fn validate_lag_refresh_interval_none_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c").with_traces(SignalConfig {
        topics: vec!["t".to_string()],
        ..Default::default()
    });
    let cfg = KafkaReceiverConfig::try_from(cfg).expect("None interval must be valid");
    assert_eq!(cfg.lag_refresh_interval_ms(), None);
}

/// Scenario (operational visibility): the lag-refresh interval is set to zero.
/// Guarantees: validation fails, so a zero lag-refresh interval is rejected.
#[test]
fn validate_lag_refresh_interval_zero_is_invalid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_lag_refresh_interval_ms(Some(0));
    let err = KafkaReceiverConfig::try_from(cfg).unwrap_err().to_string();
    assert!(err.contains("must be > 0"), "unexpected error: {err}");
}

/// Scenario (operational visibility): the lag-refresh interval is set to a positive
/// value.
/// Guarantees: validation succeeds, so a positive lag-refresh interval is accepted.
#[test]
fn validate_lag_refresh_interval_some_value_is_valid() {
    let cfg = KafkaReceiverConfigBuilder::new("b", "g", "c")
        .with_traces(SignalConfig {
            topics: vec!["t".to_string()],
            ..Default::default()
        })
        .with_lag_refresh_interval_ms(Some(5000));
    let cfg = KafkaReceiverConfig::try_from(cfg).expect("positive interval must be valid");
    assert_eq!(cfg.lag_refresh_interval_ms(), Some(5000));
}

/// Scenario (operational visibility): the idempotency flag is read on a default config.
/// Guarantees: it defaults to false, so duplicate-suppression is opt-in.
#[test]
fn enable_idempotency_defaults_to_false() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]}
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).expect("should deserialize");
    assert!(!cfg.is_idempotent());
}

/// Scenario (operational visibility): an idempotency-enabled config is deserialized.
/// Guarantees: the flag parses as true, so idempotent dedupe is configurable.
#[test]
fn enable_idempotency_deserialized_when_true() {
    let json = json!({
        "brokers": "b:9092",
        "group_id": "g",
        "client_id": "c",
        "traces": {"topics": ["t"]},
        "enable_idempotency": true
    });
    let cfg: KafkaReceiverConfig = serde_json::from_value(json).expect("should deserialize");
    assert!(cfg.is_idempotent());
}
