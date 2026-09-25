// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::mem::size_of;
use std::time::Duration;

fn polling() -> PollingConfig {
    PollingConfig {
        interval: Duration::from_secs(1),
        timeout: Duration::from_secs(1),
        fetch_size_rows: 100,
        max_rows_per_poll: 100,
        max_batch_bytes: 10 * 1024 * 1024,
        catch_up: CatchUpConfig::default(),
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

fn polling_json() -> serde_json::Value {
    serde_json::json!({
        "interval": "1s",
        "timeout": "1s",
        "fetch_size_rows": 100,
        "max_rows_per_poll": 100,
        "max_batch_bytes": 10485760
    })
}

/// Scenario: Polling configuration omits cycle budgets or overrides only one limit.
/// Guarantees: Bounded multi-page polling is the default and partial overrides retain the other limit.
#[test]
fn default_catch_up_and_partial_overrides_compile() {
    for (override_value, pages, duration) in [
        (None, 32, Duration::from_secs(10)),
        (Some(serde_json::json!({})), 32, Duration::from_secs(10)),
        (
            Some(serde_json::json!({"max_pages": 1})),
            1,
            Duration::from_secs(10),
        ),
        (
            Some(serde_json::json!({"max_duration": "1s"})),
            32,
            Duration::from_secs(1),
        ),
    ] {
        let mut value = polling_json();
        if let Some(override_value) = override_value {
            value["catch_up"] = override_value;
        }
        let config: PollingConfig = serde_json::from_value(value).expect("polling config");
        let query = CompiledQuery::compile(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS".to_owned(),
            config,
            &watermark(),
            &checkpoint_config(),
            OutputConfig::default(),
        )
        .expect("bounded query");
        assert_eq!(query.catch_up().max_pages, pages);
        assert_eq!(query.catch_up().max_duration, duration);
        assert_eq!(query.interval(), Duration::from_secs(1));
    }
}

/// Scenario: Cycle configuration is null, misspelled, or uses a fractional page count.
/// Guarantees: Invalid settings cannot silently disable bounded paging or bypass validation.
#[test]
fn rejects_invalid_catch_up_configuration_shapes() {
    for (catch_up, field) in [
        (serde_json::Value::Null, "CatchUpConfig"),
        (
            serde_json::json!({"max_pages": 32, "max_duration": "10s", "extra": true}),
            "extra",
        ),
    ] {
        let mut value = polling_json();
        value["catch_up"] = catch_up;
        let error = serde_json::from_value::<PollingConfig>(value).expect_err("invalid schema");
        assert!(error.to_string().contains(field), "{error}");
    }
    for pages in [serde_json::json!(0.5), serde_json::json!(-1)] {
        let mut value = polling_json();
        value["catch_up"] = serde_json::json!({"max_pages": pages, "max_duration": "10s"});
        assert!(serde_json::from_value::<PollingConfig>(value).is_err());
    }
}

/// Scenario: Catch-up budgets exceed either bound, including a positive sub-millisecond duration.
/// Guarantees: Compilation rejects invalid budgets with the explicit offending configuration field.
#[test]
fn rejects_invalid_catch_up_bounds() {
    for (pages, duration, field) in [
        (0, "10s", "query.catch_up.max_pages"),
        (1025, "10s", "query.catch_up.max_pages"),
        (32, "0s", "query.catch_up.max_duration"),
        (32, "0.5ms", "query.catch_up.max_duration"),
        (32, "300.001s", "query.catch_up.max_duration"),
    ] {
        let mut value = polling_json();
        value["catch_up"] = serde_json::json!({"max_pages": pages, "max_duration": duration});
        let config: PollingConfig = serde_json::from_value(value).expect("valid schema");
        let error = CompiledQuery::compile(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS".to_owned(),
            config,
            &watermark(),
            &checkpoint_config(),
            OutputConfig::default(),
        )
        .expect_err("invalid budget");
        assert!(error.to_string().contains(field), "{error}");
    }
}

/// Scenario: Explicit catch-up budgets use inclusive bounds or illustrative intermediate values.
/// Guarantees: The compiled plan and its clone return the same copyable budgets and expose them in Debug.
#[test]
fn accepts_catch_up_bounds_and_preserves_compiled_budgets() {
    for (pages, duration, expected) in [
        (1, "1ms", Duration::from_millis(1)),
        (1024, "5min", Duration::from_secs(300)),
        (32, "10s", Duration::from_secs(10)),
        (2, "1.5ms", Duration::from_micros(1500)),
    ] {
        let mut value = polling_json();
        value["catch_up"] = serde_json::json!({"max_pages": pages, "max_duration": duration});
        let config: PollingConfig = serde_json::from_value(value).expect("explicit config");
        let query = CompiledQuery::compile(
            "SELECT EVENT_TS, EVENT_ID FROM EVENTS".to_owned(),
            config,
            &watermark(),
            &checkpoint_config(),
            OutputConfig::default(),
        )
        .expect("valid budget");
        for plan in [&query, &query.clone()] {
            let budget = plan.catch_up();
            let copied = budget;
            assert_eq!(budget.max_pages, pages);
            assert_eq!(copied.max_duration, expected);
            let debug = format!("{plan:?}");
            assert!(debug.contains("catch_up: CatchUpConfig"));
            assert!(debug.contains(&format!("max_pages: {pages}")));
            assert!(debug.contains(&format!("max_duration: {expected:?}")));
            assert!(!debug.contains("SELECT"));
        }
    }
}

/// Scenario: Deserialized watermark settings contain a customer timestamp and row identifier.
/// Guarantees: Direct and nested Debug output redact initial values without changing configuration or query binds.
#[test]
fn watermark_debug_redacts_initial_values() {
    let timestamp = "2037-04-05 06:07:08.987654321";
    let row_id = 873_654_219_087_321_i64;
    let config: WatermarkConfig = serde_json::from_value(serde_json::json!({
        "mode": "composite",
        "timestamp": {
            "column": "EVENT_TS",
            "bind": "last_timestamp",
            "initial": timestamp,
            "timezone": "UTC"
        },
        "tie_breaker": {
            "column": "EVENT_ID",
            "bind": "last_tie_breaker",
            "initial": row_id
        }
    }))
    .expect("watermark configuration");
    config.validate().expect("valid watermark");
    assert_eq!(config.timestamp().initial, timestamp);
    assert_eq!(config.tie_breaker().initial, row_id);

    for debug in [
        format!("{:?}", config.timestamp()),
        format!("{:#?}", config.timestamp()),
        format!("{:?}", config.tie_breaker()),
        format!("{:#?}", config.tie_breaker()),
        format!("{config:?}"),
        format!("{config:#?}"),
    ] {
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains(timestamp));
        assert!(!debug.contains(&row_id.to_string()));
    }

    let query = CompiledQuery::compile(
        "SELECT EVENT_TS, EVENT_ID FROM EVENTS".to_owned(),
        polling(),
        &config,
        &checkpoint_config(),
        OutputConfig::default(),
    )
    .expect("compiled query");
    assert_eq!(query.watermark().initial.timestamp, timestamp);
    assert_eq!(query.watermark().initial.tie_breaker, row_id);
}

/// Scenario: A query does not start with SELECT.
/// Guarantees: The shared filter rejects non-SELECT leading keywords without opening a connection.
#[test]
fn rejects_queries_outside_the_read_only_contract() {
    for sql in [
        "DELETE FROM AUDIT_LOGS",
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

/// Scenario: Operator SQL is SELECT ... FOR UPDATE.
/// Guarantees: The shared compiler does not parse locking clauses; a leading
/// SELECT is accepted so a read-only account and vendor checks remain the control.
#[test]
fn accepts_select_statements_that_include_for_update() {
    assert!(
        CompiledQuery::compile(
            "SELECT * FROM AUDIT_LOGS FOR UPDATE".to_owned(),
            polling(),
            &watermark(),
            &checkpoint_config(),
            OutputConfig::default(),
        )
        .is_ok()
    );
}

/// Scenario: A row, page, byte, or timing limit is outside its supported range.
/// Guarantees: Invalid configured row, fetch, byte, and timing bounds are rejected before execution.
#[test]
fn rejects_invalid_polling_bounds() {
    for invalid in [
        PollingConfig {
            max_batch_bytes: 257 * 1024 * 1024,
            ..polling()
        },
        PollingConfig {
            max_batch_bytes: 0,
            ..polling()
        },
        PollingConfig {
            fetch_size_rows: 0,
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
            fetch_size_rows: 10_001,
            ..polling()
        },
        PollingConfig {
            fetch_size_rows: 101,
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

/// Scenario: A polling configuration supplies only max_batch_bytes as its byte limit.
/// Guarantees: It deserializes without another setting, bounds both representations, and keeps SQL redacted.
#[test]
fn compiles_a_composite_query_plan() {
    let config: PollingConfig = serde_json::from_value(serde_json::json!({
        "interval": "1s",
        "timeout": "1s",
        "fetch_size_rows": 100,
        "max_rows_per_poll": 100,
        "max_batch_bytes": 10 * 1024 * 1024
    }))
    .expect("original polling schema without an additional byte-limit field");
    let query = CompiledQuery::compile(
        "SELECT EVENT_TS, EVENT_ID FROM EVENTS ORDER BY EVENT_TS ASC, EVENT_ID ASC".to_owned(),
        config,
        &watermark(),
        &checkpoint_config(),
        OutputConfig::default(),
    )
    .expect("composite query should compile");

    assert_eq!(query.watermark().timestamp_bind, "last_timestamp");
    assert_eq!(query.watermark().tie_breaker_bind, "last_tie_breaker");
    assert_eq!(query.watermark().initial.tie_breaker, 0);
    assert_eq!(query.fetch_size_rows(), 100);
    assert!(format!("{query:?}").contains("fetch_size_rows: 100"));
    assert_eq!(query.max_batch_bytes(), 10 * 1024 * 1024);
    assert_eq!(query.max_normalized_bytes(), query.max_batch_bytes());
    assert!(format!("{query:?}").contains("<redacted>"));
    assert!(!format!("{query:?}").contains("EVENT_TS ASC"));
}

/// Scenario: A polling block supplies the old fetch_size spelling instead of, or alongside, fetch_size_rows.
/// Guarantees: Unknown legacy fields are rejected rather than ignored or interpreted as a second limit.
#[test]
fn rejects_legacy_fetch_size_name() {
    for include_new_name in [false, true] {
        let mut value = polling_json();
        value["fetch_size"] = serde_json::json!(50);
        if !include_new_name {
            _ = value
                .as_object_mut()
                .expect("polling object")
                .remove("fetch_size_rows");
        }
        let error = serde_json::from_value::<PollingConfig>(value).expect_err("legacy field");
        assert!(error.to_string().contains("unknown field `fetch_size`"));
    }
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

/// Scenario: A cursor repeats sensitive database values in a row, page, and compiled query.
/// Guarantees: Debug output hides both cursor components while serialization preserves their exact values.
#[test]
fn cursor_debug_redacts_nested_rows_pages_and_queries() {
    let timestamp = "2037-01-02 03:04:05.987654321";
    let tie_breaker = 834_592_176_004_i64;
    let cursor = CompositeCursor::new(timestamp.to_owned(), tie_breaker);
    let serialized = serde_json::to_value(&cursor).expect("cursor JSON");
    assert_eq!(serialized["timestamp"], timestamp);
    assert_eq!(serialized["tie_breaker"], tie_breaker);

    let row = CursorRow {
        row: Row {
            values: vec![
                CellValue::Timestamp(timestamp.to_owned()),
                CellValue::Int64(tie_breaker),
            ],
        },
        cursor: cursor.clone(),
    };
    let page = QueryPage {
        columns: vec![
            ColumnMetadata {
                name: "EVENT_TS".to_owned(),
                source_type: "TIMESTAMP".to_owned(),
                nullable: false,
            },
            ColumnMetadata {
                name: "EVENT_ID".to_owned(),
                source_type: "NUMBER".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![row.clone()],
    };
    let mut watermark = watermark();
    let WatermarkConfig::Composite {
        timestamp: configured_time,
        tie_breaker: configured_id,
    } = &mut watermark;
    configured_time.initial = timestamp.to_owned();
    configured_id.initial = tie_breaker;
    let query = CompiledQuery::compile(
        "SELECT EVENT_TS, EVENT_ID FROM PRIVATE_QUERY_TABLE".to_owned(),
        polling(),
        &watermark,
        &checkpoint_config(),
        OutputConfig::default(),
    )
    .expect("query plan");

    for debug in [
        format!("{cursor:?}"),
        format!("{row:?}"),
        format!("{page:?}"),
        format!("{query:?}"),
    ] {
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains(timestamp));
        assert!(!debug.contains(&tie_breaker.to_string()));
        assert!(!debug.contains("PRIVATE_QUERY_TABLE"));
    }
}
