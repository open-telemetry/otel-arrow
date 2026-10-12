// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{SqlServerParameter as Parameter, SqlServerReceiverConfig};
use otel_arrow_dfe_scraper::database::{CompiledWatermark, ScalarValue};
use serde_json::{Value, json};
use std::time::Duration;

fn composite() -> Value {
    json!({
        "source_id": "sql-events",
        "connection": {
            "host": "sql.example.test",
            "database": "telemetry"
        },
        "query": {
            "statement": "SELECT EVENT_ID, EVENT_TS, PAYLOAD FROM dbo.EVENTS \
                WHERE EVENT_TS > @last_timestamp \
                OR (EVENT_TS = @last_timestamp AND EVENT_ID > @last_id) \
                ORDER BY EVENT_TS ASC, EVENT_ID ASC",
            "max_rows_per_poll": 1000,
            "max_batch_bytes": "4MiB"
        },
        "watermark": {
            "mode": "composite",
            "timestamp": {
                "column": "EVENT_TS", "bind": "last_timestamp",
                "initial": "2026-01-01T00:00:00Z", "timezone": "UTC"
            },
            "tie_breaker": {
                "column": "EVENT_ID", "bind": "last_id", "initial": 0
            }
        },
        "checkpoint": {
            "directory": "state/sql-server",
            "on_nack": "rewind",
            "nack_backoff": "1s",
            "max_consecutive_failures": 5
        }
    })
}

fn scalar(initial: Value) -> Value {
    let mut config = composite();
    config["query"]["statement"] = json!(
        "SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS WHERE EVENT_ID > @last_key ORDER BY EVENT_ID"
    );
    config["watermark"] = json!({
        "mode": "scalar", "column": "EVENT_ID", "bind": "last_key", "initial": initial
    });
    config
}

fn snapshot(statement: &str) -> Value {
    let mut config = composite();
    config["query"]["statement"] = json!(statement);
    config["watermark"] = json!({"mode":"snapshot"});
    config
}

fn parse(value: Value) -> SqlServerReceiverConfig {
    serde_json::from_value(value).expect("valid configuration")
}

fn rejects(value: Value) {
    assert!(serde_json::from_value::<SqlServerReceiverConfig>(value).is_err());
}

/// Scenario: The documented composite query omits optional operating settings.
/// Guarantees: Shared defaults survive and typed cursor slots precede a bound TOP limit.
#[test]
fn composite_defaults_and_native_binding_plan() {
    let config = parse(composite());
    assert_eq!(config.source_id(), "sql-events");
    assert_eq!(config.connection().host(), "sql.example.test");
    assert_eq!(config.connection().port(), 1433);
    assert_eq!(config.connection().database(), "telemetry");
    assert_eq!(
        config.connection().connect_timeout(),
        Duration::from_secs(10)
    );
    assert!(config.connection().ca_file().is_none());
    assert_eq!(config.query().interval(), Duration::from_secs(60));
    assert_eq!(config.query().timeout(), Duration::from_secs(30));
    assert_eq!(config.query().fetch_size_rows(), 300);
    assert_eq!(config.query().max_batch_bytes(), 4 * 1024 * 1024);
    assert_eq!(config.query().catch_up().max_pages, 32);
    assert_eq!(config.checkpoint().max_consecutive_failures, 5);
    assert_eq!(config.page_limit(), 1000);
    assert_eq!(
        config.statement().parameters(),
        [
            Parameter::Timestamp,
            Parameter::TieBreaker,
            Parameter::PageLimit
        ]
    );
    assert_eq!(
        config.statement().driver_sql(),
        "SELECT TOP (@P3) EVENT_ID , EVENT_TS , PAYLOAD FROM dbo . EVENTS \
         WHERE EVENT_TS > @P1 OR ( EVENT_TS = @P1 AND EVENT_ID > @P2 ) \
         ORDER BY EVENT_TS ASC , EVENT_ID ASC"
    );
    assert!(config.query().sql().contains("@last_timestamp"));
    assert!(!config.statement().driver_sql().contains("2026"));
}

/// Scenario: Scalar keys are signed integers or exactly representable datetime2 timestamps.
/// Guarantees: Binding order and event-time mapping reflect the declared scalar type.
#[test]
fn scalar_modes_preserve_declared_types() {
    for (initial, expected) in [
        (
            json!({"type":"int64", "value":i64::MIN}),
            Parameter::ScalarInt64,
        ),
        (
            json!({"type":"timestamp", "value":"2026-01-01T00:00:00.1234567Z"}),
            Parameter::Timestamp,
        ),
    ] {
        let config = parse(scalar(initial));
        assert_eq!(
            config.statement().parameters(),
            [expected, Parameter::PageLimit]
        );
        assert_eq!(
            config.statement().driver_sql(),
            "SELECT TOP (@P2) EVENT_ID , PAYLOAD FROM dbo . EVENTS WHERE EVENT_ID > @P1 ORDER BY EVENT_ID"
        );
        assert_eq!(
            config.query().output().timestamp_column.is_some(),
            expected == Parameter::Timestamp
        );
        assert!(matches!(
            config.query().watermark(),
            CompiledWatermark::Scalar(_)
        ));
    }
    for initial in [
        json!({"type":"uint64", "value":0}),
        json!({"type":"string", "value":"0"}),
        json!({"type":"int64", "value":1.5}),
    ] {
        rejects(scalar(initial));
    }
}

/// Scenario: Snapshot queries use projections, expressions, aggregates, filtering, and grouping.
/// Guarantees: Snapshots bind only a limit-plus-one probe and never acquire a source cursor.
#[test]
fn snapshot_queries_use_complete_result_probe() {
    for statement in [
        "SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS",
        "SELECT STATUS, COUNT(*) AS TOTAL FROM dbo.EVENTS GROUP BY STATUS HAVING COUNT(*) > 0",
        "SELECT COALESCE(PAYLOAD, N'empty') AS MESSAGE FROM dbo.EVENTS WHERE STATUS = 'ready' ORDER BY MESSAGE DESC",
        "SELECT CAST(AMOUNT AS decimal(19,4)) AS AMOUNT FROM dbo.EVENTS",
    ] {
        let config = parse(snapshot(statement));
        assert_eq!(config.page_limit(), 1001);
        assert_eq!(config.statement().parameters(), [Parameter::PageLimit]);
        assert!(
            config
                .statement()
                .driver_sql()
                .starts_with("SELECT TOP (@P1)")
        );
        assert!(matches!(
            config.query().watermark(),
            CompiledWatermark::Snapshot
        ));
        assert!(config.query().output().timestamp_column.is_none());
    }
}

/// Scenario: Every allowlisted function is called with zero through three arguments.
/// Guarantees: COALESCE accepts two or more, ISNULL exactly two, and other functions exactly one.
#[test]
fn validates_supported_function_arities() {
    for name in [
        "ABS",
        "COALESCE",
        "ISNULL",
        "LOWER",
        "UPPER",
        "LEN",
        "DATALENGTH",
        "COUNT",
        "COUNT_BIG",
        "MIN",
        "MAX",
        "SUM",
        "AVG",
        "abs",
        "coalesce",
        "isnull",
    ] {
        for (count, arguments) in ["", "EVENT_ID", "EVENT_ID, 1", "EVENT_ID, 1, 2"]
            .into_iter()
            .enumerate()
        {
            let statement = format!("SELECT {name}({arguments}) AS RESULT FROM dbo.EVENTS");
            let expected = match name.to_ascii_uppercase().as_str() {
                "COALESCE" => count >= 2,
                "ISNULL" => count == 2,
                _ => count == 1,
            };
            assert_eq!(
                serde_json::from_value::<SqlServerReceiverConfig>(snapshot(&statement)).is_ok(),
                expected,
                "unexpected preparation result for: {statement}"
            );
        }
    }
}

/// Scenario: Function calls contain wildcards or invalid nested calls.
/// Guarantees: Only single-argument COUNT wildcards pass, and nested calls also enforce arity.
#[test]
fn validates_wildcard_and_nested_function_arities() {
    for expression in ["COUNT(*)", "COUNT_BIG(*)", "ABS(ABS(EVENT_ID))"] {
        _ = parse(snapshot(&format!(
            "SELECT {expression} AS RESULT FROM dbo.EVENTS"
        )));
    }
    for expression in [
        "COUNT(*, EVENT_ID)",
        "COUNT_BIG(*, EVENT_ID)",
        "SUM(*)",
        "COALESCE(*, EVENT_ID)",
        "ABS(ABS())",
        "COALESCE(EVENT_ID, ISNULL(EVENT_ID))",
    ] {
        let statement = format!("SELECT {expression} AS RESULT FROM dbo.EVENTS");
        assert!(
            serde_json::from_value::<SqlServerReceiverConfig>(snapshot(&statement)).is_err(),
            "accepted invalid function call: {statement}"
        );
    }
}

/// Scenario: Cursor-like text appears inside quoted strings or bracketed identifiers.
/// Guarantees: Only real configured parameters are rewritten; literals retain their exact content.
#[test]
fn rewriting_respects_literals_and_brackets() {
    let mut value = composite();
    value["query"]["statement"] = json!(
        "SELECT [EVENT_ID], [EVENT_TS], N'@last_timestamp; -- /* ''x''' AS MESSAGE, \
         [payload@last_id] FROM [audit schema].[event]]rows] \
         WHERE [EVENT_TS] > @last_timestamp \
         OR ([EVENT_TS] = @last_timestamp AND [EVENT_ID] > @last_id) \
         ORDER BY [EVENT_TS], [EVENT_ID]"
    );
    let config = parse(value);
    let sql = config.statement().driver_sql();
    assert!(sql.contains("N'@last_timestamp; -- /* ''x'''"));
    assert!(sql.contains("[payload@last_id]"));
    assert!(sql.contains("[event]]rows]"));
    assert!(sql.contains("[EVENT_TS] > @P1"));
    assert!(sql.contains("[EVENT_ID] > @P2"));
}

/// Scenario: Logical bind names use the same spelling as the generated positional slots.
/// Guarantees: Rewriting is one-pass and cannot accidentally rewrite an already-generated slot.
#[test]
fn logical_bind_slot_collisions_are_unambiguous() {
    let mut value = composite();
    value["watermark"]["timestamp"]["bind"] = json!("P2");
    value["watermark"]["tie_breaker"]["bind"] = json!("P1");
    let original = value["query"]["statement"].as_str().unwrap();
    value["query"]["statement"] = json!(
        original
            .replace("@last_timestamp", "@P2")
            .replace("@last_id", "@P1")
    );
    let config = parse(value);
    assert!(config.statement().driver_sql().contains("EVENT_TS > @P1"));
    assert!(config.statement().driver_sql().contains("EVENT_ID > @P2"));
}

/// Scenario: A statement uses extra operations, hints, external sources, or unsupported result shapes.
/// Guarantees: Unsafe or ambiguous SQL never reaches a driver binding plan.
#[test]
fn rejects_unsupported_statement_shapes() {
    for statement in [
        "SELECT EVENT_ID FROM dbo.EVENTS; DELETE FROM dbo.EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS;;",
        "SELECT EVENT_ID FROM dbo.EVENTS -- comment",
        "SELECT /* comment */ EVENT_ID FROM dbo.EVENTS",
        "SELECT EVENT_ID INTO dbo.COPY FROM dbo.EVENTS",
        "SELECT @out = EVENT_ID FROM dbo.EVENTS",
        "SELECT * FROM dbo.EVENTS",
        "SELECT e.* FROM dbo.EVENTS AS e",
        "SELECT DISTINCT EVENT_ID FROM dbo.EVENTS",
        "SELECT ALL EVENT_ID FROM dbo.EVENTS",
        "SELECT TOP (1) EVENT_ID FROM dbo.EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS ORDER BY EVENT_ID OFFSET 0 ROWS FETCH NEXT 1 ROWS ONLY",
        "SELECT EVENT_ID FROM dbo.EVENTS WITH (NOLOCK)",
        "SELECT EVENT_ID FROM dbo.EVENTS WITH (UPDLOCK, HOLDLOCK)",
        "SELECT EVENT_ID FROM dbo.EVENTS (READPAST)",
        "SELECT EVENT_ID FROM server.db.dbo.EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS AS e",
        "SELECT EVENT_ID FROM dbo.EVENTS JOIN dbo.OTHER ON 1 = 1",
        "SELECT EVENT_ID FROM dbo.EVENTS, dbo.OTHER",
        "SELECT EVENT_ID FROM (SELECT EVENT_ID FROM dbo.EVENTS) AS t",
        "SELECT EVENT_ID FROM dbo.EVENTS UNION ALL SELECT EVENT_ID FROM dbo.EVENTS",
        "WITH t AS (SELECT EVENT_ID FROM dbo.EVENTS) SELECT EVENT_ID FROM t",
        "SELECT dbo.customer_function(PAYLOAD) AS MESSAGE FROM dbo.EVENTS",
        "SELECT NEXT VALUE FOR dbo.SEQ AS EVENT_ID FROM dbo.EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS FOR JSON AUTO",
        "SELECT EVENT_ID FROM OPENQUERY(remote, 'SELECT EVENT_ID FROM EVENTS')",
        "SELECT EVENT_ID FROM #EVENTS",
        "SELECT EVENT_ID FROM EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID IN (SELECT EVENT_ID FROM dbo.OTHER)",
        "SELECT SUM(EVENT_ID) OVER () AS TOTAL FROM dbo.EVENTS",
        "SELECT EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID > @undeclared",
        "SELECT EVENT_ID FROM dbo.EVENTS OPTION (MAXDOP 1)",
        "SELECT EVENT_ID FROM dbo.EVENTS; EXEC sp_who",
    ] {
        assert!(
            serde_json::from_value::<SqlServerReceiverConfig>(snapshot(statement)).is_err(),
            "accepted unsupported statement: {statement}"
        );
    }
}

/// Scenario: A keyset predicate or ordering resembles the required form but changes its meaning.
/// Guarantees: No non-strict, reordered, derived, aliased, or collation-dependent cursor is accepted.
#[test]
fn rejects_keyset_lookalikes() {
    let base = composite();
    let sql = base["query"]["statement"].as_str().unwrap();
    for statement in [
        sql.replace("EVENT_TS > @last_timestamp", "EVENT_TS >= @last_timestamp"),
        sql.replace("AND EVENT_ID >", "OR EVENT_ID >"),
        sql.replace("@last_id)", "@last_id) OR 1 = 1"),
        sql.replace("@last_timestamp", "'@last_timestamp'"),
        sql.replace("@last_timestamp", "@last_timestamp_extra"),
        sql.replace("EVENT_ID ASC", "EVENT_ID DESC"),
        sql.replace(
            "ORDER BY EVENT_TS ASC, EVENT_ID ASC",
            "ORDER BY EVENT_ID, EVENT_TS",
        ),
        sql.replace("SELECT EVENT_ID", "SELECT EVENT_ID AS EVENT_ID"),
        sql.replace("SELECT EVENT_ID", "SELECT EVENT_ID + 1 AS EVENT_ID"),
        sql.replace("WHERE EVENT_TS", "WHERE event_ts"),
        sql.replace("WHERE EVENT_TS", "WHERE \"EVENT_TS\""),
        sql.replace(
            "ORDER BY EVENT_TS ASC",
            "ORDER BY EVENT_TS COLLATE Latin1_General_BIN2 ASC",
        ),
        sql.replace(
            "SELECT EVENT_ID, EVENT_TS, PAYLOAD",
            "SELECT EVENT_ID, EVENT_TS, SUM(EVENT_ID) AS TOTAL",
        ),
    ] {
        let mut value = base.clone();
        value["query"]["statement"] = json!(statement);
        rejects(value);
    }
}

/// Scenario: Scalar predicates, key projections, or orderings violate exclusive keyset semantics.
/// Guarantees: A scalar plan never accepts extra predicates, fake binds, or a hidden cursor alias.
#[test]
fn rejects_scalar_lookalikes() {
    let base = scalar(json!({"type":"int64","value":0}));
    for statement in [
        "SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS WHERE EVENT_ID >= @last_key ORDER BY EVENT_ID",
        "SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS WHERE EVENT_ID > @last_key AND STATUS = 1 ORDER BY EVENT_ID",
        "SELECT EVENT_ID, PAYLOAD FROM dbo.EVENTS WHERE EVENT_ID > '@last_key' ORDER BY EVENT_ID",
        "SELECT EVENT_ID AS EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID > @last_key ORDER BY EVENT_ID",
        "SELECT EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID > @last_key ORDER BY EVENT_ID DESC",
        "SELECT EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID > @last_key",
        "SELECT EVENT_ID FROM dbo.EVENTS WHERE EVENT_ID > @last_key ORDER BY EVENT_ID, PAYLOAD",
    ] {
        let mut value = base.clone();
        value["query"]["statement"] = json!(statement);
        rejects(value);
    }
}

/// Scenario: Selected columns are duplicated, unnamed, or exceed the fixed metadata column bound.
/// Guarantees: The shared case-insensitive mapper receives a bounded, unambiguous result contract.
#[test]
fn validates_projection_names_and_count() {
    for statement in [
        "SELECT EVENT_ID, event_id FROM dbo.EVENTS",
        "SELECT EVENT_ID, PAYLOAD AS event_id FROM dbo.EVENTS",
        "SELECT COUNT(*) FROM dbo.EVENTS",
        "SELECT 1 + 2 FROM dbo.EVENTS",
        "SELECT [@last_key] FROM dbo.EVENTS",
        "SELECT [bad\nname] FROM dbo.EVENTS",
    ] {
        rejects(snapshot(statement));
    }
    let projection = |count| {
        (0..count)
            .map(|n| format!("C{n}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    _ = parse(snapshot(&format!(
        "SELECT {} FROM dbo.EVENTS",
        projection(256)
    )));
    rejects(snapshot(&format!(
        "SELECT {} FROM dbo.EVENTS",
        projection(257)
    )));
}

/// Scenario: SQL is oversized, malformed, unterminated, or deeply nested.
/// Guarantees: Parsing fails with bounded work and never returns raw parser diagnostics.
#[test]
fn bounds_and_redacts_parser_failures() {
    for statement in [
        format!(
            "SELECT '{}' AS PAYLOAD FROM dbo.EVENTS",
            "x".repeat(16 * 1024)
        ),
        format!(
            "SELECT {}1{} AS N FROM dbo.EVENTS",
            "(".repeat(100),
            ")".repeat(100)
        ),
        format!(
            "SELECT {} AS N FROM dbo.EVENTS",
            vec!["1"; 1100].join(" + ")
        ),
        "SELECT 'sensitive_unterminated".to_owned(),
        "SELECT [sensitive_unterminated FROM dbo.EVENTS".to_owned(),
        "EXEC sensitive_procedure".to_owned(),
    ] {
        let error = serde_json::from_value::<SqlServerReceiverConfig>(snapshot(&statement))
            .unwrap_err()
            .to_string();
        assert!(!error.contains("sensitive"));
        assert!(!error.contains(&statement));
    }
}

/// Scenario: Equivalent UTC instants arrive with offsets or timezone-less UTC spelling.
/// Guarantees: Fingerprints and source positions use one exact seven-digit datetime2 representation.
#[test]
fn canonicalizes_exact_datetime2_values() {
    let mut baseline = None;
    for value in [
        "2026-01-01T00:00:00.1234567Z",
        "2026-01-01T02:00:00.123456700+02:00",
        "2026-01-01 00:00:00.1234567",
    ] {
        let mut raw = composite();
        raw["watermark"]["timestamp"]["initial"] = json!(value);
        let config = parse(raw);
        let CompiledWatermark::Composite(cursor) = config.query().watermark() else {
            panic!("expected composite");
        };
        assert_eq!(cursor.initial.timestamp, "2026-01-01T00:00:00.1234567Z");
        if let Some(fingerprint) = &baseline {
            assert_eq!(config.config_fingerprint(), fingerprint);
        } else {
            baseline = Some(config.config_fingerprint().to_owned());
        }
    }
}

/// Scenario: Cursor text would round, overflow SQL Server dates, or encode a leap second.
/// Guarantees: No lossy timestamp is accepted, including input beyond chrono's nanosecond precision.
#[test]
fn rejects_lossy_or_invalid_timestamps() {
    for initial in [
        "2026-01-01T00:00:00.12345678Z",
        "2026-01-01T00:00:00.123456789Z",
        "2026-01-01T00:00:00.1234567001Z",
        "0000-01-01T00:00:00Z",
        "+10000-01-01T00:00:00Z",
        "0001-01-01T00:00:00+01:00",
        "2016-12-31T23:59:60Z",
        "2026-01-01T00:00:00+99:99",
        "not-a-timestamp",
    ] {
        let mut raw = composite();
        raw["watermark"]["timestamp"]["initial"] = json!(initial);
        rejects(raw);
        rejects(scalar(json!({"type":"timestamp","value":initial})));
    }
}

/// Scenario: Supported datetime2 boundary dates and integer extremes initialize a scalar cursor.
/// Guarantees: Valid source positions are preserved even outside OTLP's event-time range.
#[test]
fn accepts_source_cursor_boundaries() {
    for value in ["0001-01-01T00:00:00Z", "9999-12-31T23:59:59.9999999Z"] {
        let config = parse(scalar(json!({"type":"timestamp","value":value})));
        assert!(matches!(
            config.query().watermark(),
            CompiledWatermark::Scalar(_)
        ));
    }
    for value in [i64::MIN, 0, i64::MAX] {
        let config = parse(scalar(json!({"type":"int64","value":value})));
        let CompiledWatermark::Scalar(cursor) = config.query().watermark() else {
            panic!("expected scalar");
        };
        assert_eq!(cursor.initial, ScalarValue::Int64(value));
    }
}

/// Scenario: An operator changes only polling budgets, TLS file paths, or checkpoint operations.
/// Guarantees: Operational tuning and secret-location changes cannot silently reset compatibility.
#[test]
fn fingerprint_excludes_operational_configuration() {
    let expected = parse(composite()).config_fingerprint().to_owned();
    for (path, value) in [
        ("/query/interval", json!("2m")),
        ("/query/timeout", json!("60s")),
        ("/query/fetch_size_rows", json!(100)),
        ("/query/max_rows_per_poll", json!(2000)),
        ("/query/max_batch_bytes", json!("8MiB")),
        ("/connection/connect_timeout", json!("20s")),
        ("/checkpoint/directory", json!("elsewhere")),
        ("/checkpoint/nack_backoff", json!("2s")),
        ("/checkpoint/max_consecutive_failures", json!(10)),
    ] {
        let mut raw = composite();
        if path == "/connection/connect_timeout"
            || path == "/query/interval"
            || path == "/query/timeout"
            || path == "/query/fetch_size_rows"
        {
            let (object, field) = path.trim_start_matches('/').split_once('/').unwrap();
            raw[object][field] = value;
        } else {
            *raw.pointer_mut(path).unwrap() = value;
        }
        assert_eq!(parse(raw).config_fingerprint(), expected);
    }
    let mut raw = composite();
    raw["connection"]["tls"] = json!({"ca_file":"certs/enterprise.pem"});
    raw["query"]["catch_up"] = json!({"max_pages":1});
    assert_eq!(parse(raw).config_fingerprint(), expected);
}

/// Scenario: A semantic source/query/cursor input changes, or only harmless formatting changes.
/// Guarantees: Semantic changes invalidate the fingerprint while SQL whitespace does not.
#[test]
fn fingerprint_tracks_source_semantics() {
    let original = composite();
    let expected = parse(original.clone()).config_fingerprint().to_owned();
    for (path, value) in [
        ("/source_id", json!("other-source")),
        ("/connection/host", json!("other.example.test")),
        ("/connection/database", json!("other-database")),
        (
            "/watermark/timestamp/initial",
            json!("2026-01-02T00:00:00Z"),
        ),
        ("/watermark/tie_breaker/initial", json!(1)),
    ] {
        let mut raw = original.clone();
        *raw.pointer_mut(path).unwrap() = value;
        assert_ne!(parse(raw).config_fingerprint(), expected);
    }
    let mut raw = original.clone();
    raw["query"]["statement"] = json!(
        original["query"]["statement"]
            .as_str()
            .unwrap()
            .replace("PAYLOAD", "OTHER_PAYLOAD")
    );
    assert_ne!(parse(raw).config_fingerprint(), expected);
    let mut raw = original.clone();
    raw["query"]["statement"] = json!(format!(
        "\n{}\n;",
        original["query"]["statement"]
            .as_str()
            .unwrap()
            .replace(' ', "\n")
    ));
    assert_eq!(parse(raw).config_fingerprint(), expected);
    let mut raw = original;
    raw["connection"]["host"] = json!("SQL.EXAMPLE.TEST");
    assert_eq!(parse(raw).config_fingerprint(), expected);
}

/// Scenario: Unknown fields attempt to supply credentials, weaken TLS, or change the outer schema.
/// Guarantees: Receiver-local secrets, insecure overrides, and null optional settings fail closed.
#[test]
fn rejects_credentials_insecure_options_and_unknown_fields() {
    for (object, field, value) in [
        ("connection", "password", json!("secret")),
        ("connection", "username", json!("reader")),
        (
            "connection",
            "connect_string",
            json!("Server=host;Password=secret"),
        ),
        ("connection", "trust_server_certificate", json!(true)),
        ("connection", "encrypt", json!(false)),
        ("connection", "tls", json!({"insecure_skip_verify":true})),
        ("connection", "tls", json!({"ca_file":null})),
        ("query", "fetch_size", json!(100)),
        ("query", "output", json!({})),
    ] {
        let mut raw = composite();
        raw[object][field] = value;
        rejects(raw);
    }
    let mut raw = composite();
    raw["authentication"] = json!({"password_file":"secret"});
    rejects(raw);
}

/// Scenario: Required sections are missing or nested policy objects contain unknown fields.
/// Guarantees: Preparation preserves strict shared schemas instead of silently ignoring input.
#[test]
fn rejects_missing_sections_and_unknown_nested_fields() {
    for field in [
        "source_id",
        "connection",
        "query",
        "watermark",
        "checkpoint",
    ] {
        let mut raw = composite();
        _ = raw.as_object_mut().unwrap().remove(field);
        rejects(raw);
    }
    for path in [
        "/watermark",
        "/watermark/timestamp",
        "/watermark/tie_breaker",
        "/checkpoint",
    ] {
        let mut raw = composite();
        raw.pointer_mut(path).unwrap()["unknown"] = json!(true);
        rejects(raw);
    }
    let mut raw = composite();
    raw["query"]["catch_up"] = json!({"unknown":true});
    rejects(raw);
}

/// Scenario: Source identity, cursor relationships, or shared checkpoint/catch-up policy is invalid.
/// Guarantees: Only bounded, distinct cursor plans with valid shared persistence policy compile.
#[test]
fn rejects_invalid_identity_cursor_and_shared_policy() {
    for source_id in ["", "bad/source", "non-ascii-\u{00e9}", &"x".repeat(257)] {
        let mut raw = composite();
        raw["source_id"] = json!(source_id);
        rejects(raw);
    }
    let mut raw = composite();
    raw["source_id"] = json!("x".repeat(256));
    _ = parse(raw);
    for (path, value) in [
        ("/watermark/timestamp/timezone", json!("Europe/Paris")),
        ("/watermark/timestamp/bind", json!("@last_timestamp")),
        ("/watermark/tie_breaker/bind", json!("LAST_TIMESTAMP")),
        ("/watermark/tie_breaker/column", json!("event_ts")),
        ("/checkpoint/directory", json!("")),
        ("/checkpoint/directory", json!("state/../other")),
        ("/checkpoint/on_nack", json!("skip")),
        ("/checkpoint/nack_backoff", json!("0s")),
        ("/checkpoint/max_consecutive_failures", json!(0)),
    ] {
        let mut raw = composite();
        *raw.pointer_mut(path).unwrap() = value;
        rejects(raw);
    }
    for catch_up in [
        json!({"max_pages":0}),
        json!({"max_pages":1025}),
        json!({"max_duration":"0s"}),
        json!({"max_duration":"301s"}),
    ] {
        let mut raw = composite();
        raw["query"]["catch_up"] = catch_up;
        rejects(raw);
    }
}

/// Scenario: Bounds, field types, endpoint syntax, or duration precision are invalid.
/// Guarantees: Invalid input never falls back to defaults or changes endpoint interpretation.
#[test]
fn rejects_invalid_operational_values() {
    for (object, field, values) in [
        (
            "query",
            "interval",
            vec![json!("59s"), json!("60.5s"), json!("25h"), Value::Null],
        ),
        (
            "query",
            "timeout",
            vec![json!("0s"), json!("301s"), json!("1.5s"), Value::Null],
        ),
        (
            "query",
            "max_rows_per_poll",
            vec![json!(0), json!(10001), json!(299)],
        ),
        (
            "query",
            "fetch_size_rows",
            vec![json!(0), json!(1001), Value::Null],
        ),
        (
            "query",
            "max_batch_bytes",
            vec![json!(0), json!("257MiB"), Value::Null],
        ),
        (
            "connection",
            "host",
            vec![
                json!(""),
                json!("host;Password=x"),
                json!("host\\instance"),
                json!("user@host"),
                json!("https://host"),
                json!("host:1433"),
            ],
        ),
        (
            "connection",
            "port",
            vec![json!(0), json!(65536), Value::Null],
        ),
        (
            "connection",
            "database",
            vec![json!(""), json!("db\nname"), json!("a".repeat(129))],
        ),
        (
            "connection",
            "connect_timeout",
            vec![json!("0s"), json!("301s"), Value::Null],
        ),
    ] {
        for value in values {
            let mut raw = composite();
            raw[object][field] = value;
            rejects(raw);
        }
    }
    for host in ["127.0.0.1", "::1", "localhost"] {
        let mut raw = composite();
        raw["connection"]["host"] = json!(host);
        _ = parse(raw);
    }
}

/// Scenario: Configuration or parser failures and Debug formatting handle sensitive sentinels.
/// Guarantees: No endpoint, SQL, source value, cursor, or supplied credential appears in diagnostics.
#[test]
fn all_preparation_diagnostics_are_redacted() {
    let mut raw = composite();
    raw["connection"]["host"] = json!("sensitive-host.example");
    raw["connection"]["database"] = json!("sensitive-database");
    raw["query"]["statement"] = json!(
        raw["query"]["statement"]
            .as_str()
            .unwrap()
            .replace("PAYLOAD", "'sensitive-row' AS PAYLOAD")
    );
    let config = parse(raw.clone());
    let debug = format!(
        "{config:?} {:#?} {:?} {:?}",
        config.connection(),
        config.statement(),
        config.query()
    );
    for sentinel in [
        "sensitive-host",
        "sensitive-database",
        "sensitive-row",
        "2026-01-01",
    ] {
        assert!(!debug.contains(sentinel), "{debug}");
    }
    for (object, field) in [
        ("connection", "host"),
        ("query", "interval"),
        ("query", "statement"),
        ("watermark", "initial"),
    ] {
        let mut invalid = raw.clone();
        invalid[object][field] = json!("SENSITIVE;customer-data");
        let err = serde_json::from_value::<SqlServerReceiverConfig>(invalid).unwrap_err();
        assert!(!format!("{err} {err:?}").contains("SENSITIVE"));
    }
}

/// Scenario: Preparation uses a nonexistent CA file and checkpoint directory.
/// Guarantees: Configuration validation neither opens secrets/network sessions nor creates state.
#[test]
fn preparation_has_no_filesystem_side_effects() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("not-created");
    let ca = dir.path().join("not-loaded.pem");
    let mut raw = composite();
    raw["checkpoint"]["directory"] = json!(state);
    raw["connection"]["tls"] = json!({"ca_file":ca});
    let config = parse(raw);
    assert_eq!(config.connection().ca_file(), Some(ca.as_path()));
    assert!(!state.exists());
    assert!(!ca.exists());
}

/// Scenario: Quoted payloads and escaped identifiers contain SQL-looking delimiters.
/// Guarantees: The emitted native SQL still parses as one query with unchanged projections.
#[test]
fn native_sql_preserves_literal_and_identifier_semantics() {
    use sqlparser::{ast::Statement, dialect::MsSqlDialect, parser::Parser};

    let dialect = MsSqlDialect {};
    for literal in [
        "'ordinary''quote; -- text'",
        "N'unicode \u{1f30d}''; DELETE FROM dbo.EVENTS; --'",
        "N'@P1 @P2 @last_key /* still text */'",
    ] {
        let original = format!(
            "SELECT {literal} AS [message]]name], EVENT_ID FROM [audit schema].[event]]rows]"
        );
        let config = parse(snapshot(&original));
        let original_ast = Parser::parse_sql(&dialect, &original).unwrap();
        let native_ast = Parser::parse_sql(&dialect, config.statement().driver_sql()).unwrap();
        assert_eq!(native_ast.len(), 1);
        let (Statement::Query(original), Statement::Query(native)) =
            (&original_ast[0], &native_ast[0])
        else {
            panic!("expected SELECT queries");
        };
        let original = original.body.as_select().unwrap();
        let native = native.body.as_select().unwrap();
        assert_eq!(original.projection, native.projection);
        assert_eq!(original.from, native.from);
        assert!(original.top.is_none());
        assert!(native.top.is_some());
    }
}

/// Scenario: Configuration is exactly at the shared row/byte and operational time ceilings.
/// Guarantees: Boundary values are accepted and the snapshot overflow probe cannot overflow.
#[test]
fn accepts_exact_shared_limits() {
    let mut raw = snapshot("SELECT EVENT_ID FROM dbo.EVENTS");
    raw["query"]["max_rows_per_poll"] = json!(10000);
    raw["query"]["fetch_size_rows"] = json!(10000);
    raw["query"]["max_batch_bytes"] = json!(256 * 1024 * 1024);
    raw["query"]["interval"] = json!("24h");
    raw["query"]["timeout"] = json!("5m");
    raw["query"]["catch_up"] = json!({"max_pages":1024,"max_duration":"5m"});
    raw["connection"]["connect_timeout"] = json!("5m");
    let config = parse(raw);
    assert_eq!(config.page_limit(), 10001);
    assert_eq!(config.query().max_batch_bytes(), 256 * 1024 * 1024);
    assert_eq!(config.query().catch_up().max_pages, 1024);
}

/// Scenario: The preparation-only SQL Server feature is linked into a host.
/// Guarantees: No SQL Server receiver factory is registered by this layer.
#[test]
fn preparation_does_not_register_runtime_factory() {
    assert!(
        !otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES
            .iter()
            .any(|factory| factory.name == "urn:otel:receiver:sql_server")
    );
}
