// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::mem::size_of;
use std::time::Duration;

fn polling() -> PollingConfig {
    PollingConfig {
        interval: Duration::from_secs(1),
        timeout: Duration::from_secs(1),
        fetch_size: 100,
        max_rows_per_poll: 100,
        max_batch_bytes: 10 * 1024 * 1024,
        max_normalized_bytes: 5 * 1024 * 1024,
    }
}

fn watermark() -> WatermarkConfig {
    WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            column: "EVENT_TS".to_owned(),
            bind: "last_timestamp".to_owned(),
            initial: "1970-01-01 00:00:00".to_owned(),
            timezone: "UTC".to_owned(),
        },
        tie_breaker: TieBreakerCursorConfig {
            column: "EVENT_ID".to_owned(),
            bind: "last_tie_breaker".to_owned(),
            initial: 0,
        },
    }
}

fn checkpoint_config() -> CheckpointConfig {
    CheckpointConfig {
        directory: "${engine.state_dir}/oracle".to_owned(),
        on_nack: OnNack::Rewind,
        nack_backoff: Duration::from_secs(1),
        max_consecutive_failures: 5,
    }
}

/// Scenario: A query is modifying, locking, or not a directly validated SELECT.
/// Guarantees: Unsafe SQL is rejected before any database connection is opened.
#[test]
fn rejects_queries_outside_the_read_only_contract() {
    for sql in [
        "DELETE FROM AUDIT_LOGS",
        "SELECT * FROM AUDIT_LOGS FOR UPDATE",
        "WITH rows AS (SELECT 1 FROM DUAL) SELECT * FROM rows",
    ] {
        assert!(matches!(
            CompiledQuery::compile(
                sql.to_owned(),
                polling(),
                &watermark(),
                &checkpoint_config(),
                OutputConfig::default(),
            ),
            Err(QueryError::NotReadOnly)
        ));
    }
}

/// Scenario: A row, page, byte, or timing limit is outside its supported range.
/// Guarantees: Every polling resource remains positive and bounded, including aggregate
/// in-flight memory.
#[test]
fn rejects_invalid_polling_bounds() {
    for invalid in [
        PollingConfig {
            max_normalized_bytes: 0,
            ..polling()
        },
        PollingConfig {
            max_normalized_bytes: 257 * 1024 * 1024,
            ..polling()
        },
        PollingConfig {
            max_batch_bytes: 0,
            ..polling()
        },
        PollingConfig {
            fetch_size: 0,
            ..polling()
        },
        PollingConfig {
            max_rows_per_poll: 10_001,
            ..polling()
        },
        PollingConfig {
            interval: Duration::from_secs(48 * 60 * 60),
            ..polling()
        },
        PollingConfig {
            max_batch_bytes: 512 * 1024 * 1024,
            ..polling()
        },
        PollingConfig {
            fetch_size: 10_001,
            ..polling()
        },
        PollingConfig {
            fetch_size: 101,
            ..polling()
        },
    ] {
        assert!(invalid.validate().is_err());
    }
}

/// Scenario: watermark mode is configured as scalar or snapshot.
/// Guarantees: unimplemented modes are rejected by the schema instead of silently inheriting
/// composite behavior that they do not actually describe.
#[test]
fn rejects_unsupported_watermark_modes() {
    for mode in ["scalar", "snapshot"] {
        let value = serde_json::json!({
            "mode": mode,
            "timestamp": {
                "column": "EVENT_TS",
                "bind": "last_timestamp",
                "initial": "1970-01-01 00:00:00",
                "timezone": "UTC"
            },
            "tie_breaker": {
                "column": "EVENT_ID",
                "bind": "last_tie_breaker",
                "initial": 0
            }
        });

        assert!(
            serde_json::from_value::<WatermarkConfig>(value).is_err(),
            "watermark mode '{mode}' must be rejected"
        );
    }
}

/// Scenario: composite cursor fields are inconsistent or use unsupported semantics.
/// Guarantees: shared cursor bounds, distinct binds and columns, and UTC-only semantics are all
/// enforced before a database connection is opened.
#[test]
fn rejects_invalid_composite_watermarks() {
    let non_utc = WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            timezone: "America/New_York".to_owned(),
            ..watermark().timestamp().clone()
        },
        tie_breaker: watermark().tie_breaker().clone(),
    };
    assert!(non_utc.validate().is_err());

    let duplicate_bind = WatermarkConfig::Composite {
        timestamp: watermark().timestamp().clone(),
        tie_breaker: TieBreakerCursorConfig {
            bind: "last_timestamp".to_owned(),
            ..watermark().tie_breaker().clone()
        },
    };
    assert!(duplicate_bind.validate().is_err());

    let colon_bind = WatermarkConfig::Composite {
        timestamp: TimestampCursorConfig {
            bind: ":last_timestamp".to_owned(),
            ..watermark().timestamp().clone()
        },
        tie_breaker: watermark().tie_breaker().clone(),
    };
    assert!(colon_bind.validate().is_err());
}

/// Scenario: a NACK policy or checkpoint bound outside the supported contract is configured.
/// Guarantees: only the implemented rewind policy is accepted, and backoff and failure budgets
/// remain explicit and finite so a receiver cannot retry forever.
#[test]
fn rejects_unsupported_checkpoint_policy_and_bounds() {
    assert!(serde_json::from_value::<OnNack>(serde_json::json!("fail")).is_err());
    assert!(serde_json::from_value::<OnNack>(serde_json::json!("rewind")).is_ok());

    for invalid in [
        CheckpointConfig {
            nack_backoff: Duration::ZERO,
            ..checkpoint_config()
        },
        CheckpointConfig {
            nack_backoff: Duration::from_micros(500),
            ..checkpoint_config()
        },
        CheckpointConfig {
            max_consecutive_failures: 0,
            ..checkpoint_config()
        },
        CheckpointConfig {
            directory: "state/../../escape".to_owned(),
            ..checkpoint_config()
        },
    ] {
        assert!(invalid.validate().is_err());
    }
}

/// Scenario: a valid composite configuration is compiled into a query plan.
/// Guarantees: cursor binds, columns, the initial cursor, and both byte ceilings are carried into
/// the plan the adapter executes, and SQL text is redacted from diagnostics.
#[test]
fn compiles_a_composite_query_plan() {
    let query = CompiledQuery::compile(
        "SELECT EVENT_TS, EVENT_ID FROM EVENTS ORDER BY EVENT_TS ASC, EVENT_ID ASC".to_owned(),
        polling(),
        &watermark(),
        &checkpoint_config(),
        OutputConfig::default(),
    )
    .expect("composite query should compile");

    assert_eq!(query.watermark().timestamp_bind, "last_timestamp");
    assert_eq!(query.watermark().tie_breaker_bind, "last_tie_breaker");
    assert_eq!(query.watermark().initial.tie_breaker, 0);
    assert_eq!(query.fetch_size(), 100);
    assert_eq!(query.max_batch_bytes(), 10 * 1024 * 1024);
    assert_eq!(query.max_normalized_bytes(), 5 * 1024 * 1024);
    assert!(format!("{query:?}").contains("<redacted>"));
    assert!(!format!("{query:?}").contains("EVENT_TS ASC"));
}

/// Scenario: A result contains many NULL values with no dynamic payload bytes.
/// Guarantees: Memory accounting includes row and CellValue allocations, not only scalar payloads.
#[test]
fn normalized_size_includes_structural_allocations() {
    let row = Row {
        values: vec![CellValue::Null; 100],
    };

    assert!(row.normalized_size() >= (size_of::<Row>() + 100 * size_of::<CellValue>()) as u64);
}
