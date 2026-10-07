// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_scraper::database::{CompiledWatermark, Cursor, ScalarValue};

fn scalar_config(value: ScalarValue) -> Value {
    let string = matches!(value, ScalarValue::String(_));
    let mut config = documented_config();
    config["watermark"] = serde_json::json!({"mode":"scalar", "column":"CURSOR_KEY", "bind":"last_key", "initial":value});
    config["query"]["statement"] = if string {
        "SELECT CURSOR_KEY, PAYLOAD FROM AUDIT_LOGS WHERE CURSOR_KEY COLLATE BINARY > :last_key COLLATE BINARY ORDER BY CURSOR_KEY COLLATE BINARY ASC"
    } else {
        "SELECT CURSOR_KEY, PAYLOAD FROM AUDIT_LOGS WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC"
    }.into();
    config
}

fn scalar_values() -> Vec<ScalarValue> {
    vec![
        ScalarValue::Int64(i64::MIN),
        ScalarValue::UInt64(u64::MAX),
        ScalarValue::String("!".into()),
        ScalarValue::Timestamp("2026-01-01T00:00:00.123456789Z".into()),
    ]
}

/// Scenario: Oracle parses each supported scalar watermark type.
/// Guarantees: Typed keys compile and non-temporal keys never become OTLP event timestamps.
#[test]
fn scalar_types_compile_without_timestamp_coercion() {
    for value in scalar_values() {
        let timestamp = matches!(value, ScalarValue::Timestamp(_));
        let config = parsed(scalar_config(value.clone())).expect("scalar config");
        let query = config.query();
        let CompiledWatermark::Scalar(spec) = query.watermark() else {
            panic!("scalar mode")
        };
        assert!(spec.initial.same_type(&value));
        assert_eq!(spec.initial.compare(&value), Ok(std::cmp::Ordering::Equal));
        assert_eq!(
            query.output().timestamp_column.as_deref(),
            if timestamp { Some("CURSOR_KEY") } else { None }
        );
        assert_eq!(query.output().validation_columns, ["CURSOR_KEY"]);
        query
            .watermark()
            .validate_cursor(&query.watermark().initial())
            .expect("typed initial");
    }
}

/// Scenario: Scalar predicates are absent, non-strict, broadened, or disguised in literals/subqueries.
/// Guarantees: Only the supported exclusive keyset predicate and matching outer ASC order are accepted.
#[test]
fn scalar_sql_rejects_unsafe_lookalikes() {
    for sql in [
        "SELECT CURSOR_KEY FROM T ORDER BY CURSOR_KEY ASC",
        "SELECT PAYLOAD FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT OTHER AS CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY + 1 AS CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY >= :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY = :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key OR 1 = 1 ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key_extra ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > ':last_key' ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY DESC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY OTHER ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC FOR UPDATE",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC; DELETE FROM T",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key UNION SELECT CURSOR_KEY FROM U ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM (SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC)",
    ] {
        let mut value = scalar_config(ScalarValue::Int64(0));
        value["query"]["statement"] = sql.into();
        assert!(parsed(value).is_err(), "{sql}");
    }
    let mut value = scalar_config(ScalarValue::Int64(0));
    value["query"]["statement"] =
        "select CURSOR_KEY from T where (CURSOR_KEY > :last_key) order by CURSOR_KEY asc;".into();
    assert!(parsed(value).is_ok());
    let mut value = scalar_config(ScalarValue::Int64(0));
    value["query"]["statement"] =
        "SELECT CURSOR_KEY, PAYLOAD AS BODY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC"
            .into();
    assert!(parsed(value).is_ok());
}

/// Scenario: String keys would inherit linguistic or column-defined collation.
/// Guarantees: Both comparison operands and ordering must explicitly select BINARY collation.
#[test]
fn string_queries_require_explicit_binary_collation() {
    let raw = scalar_config(ScalarValue::String("!".into()));
    let sql = raw["query"]["statement"].as_str().expect("statement");
    for rewritten in [
        sql.replace(" COLLATE BINARY", ""),
        sql.replacen("COLLATE BINARY", "COLLATE BINARY_CI", 1),
        sql.replacen(" COLLATE BINARY", "", 1),
    ] {
        let mut value = raw.clone();
        value["query"]["statement"] = rewritten.into();
        assert!(parsed(value).is_err());
    }
    assert!(parsed(raw).is_ok());
}

/// Scenario: Oracle scalar initialization includes empty strings, NUL, oversized text, or excess timestamp precision.
/// Guarantees: Invalid cursors fail before connecting and diagnostics never reveal their values.
#[test]
fn scalar_initial_values_fail_closed_and_redacted() {
    for value in [
        ScalarValue::String("".into()),
        ScalarValue::String("private\0value".into()),
        ScalarValue::String("private".repeat(200)),
        ScalarValue::Timestamp("private-invalid".into()),
        ScalarValue::Timestamp("2026-01-01T00:00:00.1234567891Z".into()),
    ] {
        let error = parsed(scalar_config(value))
            .err()
            .expect("invalid scalar")
            .to_string();
        assert!(!error.contains("private"));
        assert!(!error.contains("1234567891"));
    }
}

/// Scenario: Scalar semantics change while the local Oracle client installation moves independently.
/// Guarantees: Semantic changes invalidate scalar progress and client relocation preserves it.
#[test]
fn scalar_fingerprints_track_mode_type_and_query() {
    let raw = scalar_config(ScalarValue::Int64(0));
    let base = parsed(raw.clone()).expect("base");
    for value in [ScalarValue::UInt64(0), ScalarValue::Int64(1)] {
        assert_ne!(
            base.config_fingerprint(),
            parsed(scalar_config(value))
                .expect("changed type/value")
                .config_fingerprint()
        );
    }
    for (field, old, new) in [
        ("column", "CURSOR_KEY", "NEW_KEY"),
        ("bind", "last_key", "new_key"),
    ] {
        let mut changed = raw.clone();
        changed["watermark"][field] = new.into();
        changed["query"]["statement"] = raw["query"]["statement"]
            .as_str()
            .expect("sql")
            .replace(old, new)
            .into();
        assert_ne!(
            base.config_fingerprint(),
            parsed(changed).expect("changed key").config_fingerprint()
        );
    }
    let mut relocated = raw;
    relocated["connection"]["instant_client_dir"] = "/opt/oracle/ic-23".into();
    assert_eq!(
        base.config_fingerprint(),
        parsed(relocated).expect("relocation").config_fingerprint()
    );
    assert_ne!(
        base.config_fingerprint(),
        parsed(documented_config())
            .expect("composite")
            .config_fingerprint()
    );
}

/// Scenario: An existing composite configuration is compiled after adding scalar support.
/// Guarantees: Its original ordered fingerprint payload is unchanged, preserving checkpoint identity.
#[test]
fn composite_fingerprint_remains_byte_compatible() {
    let raw = documented_config();
    let config = parsed(raw.clone()).expect("composite");
    let query = config.query();
    let spec = query.watermark().as_composite().expect("composite");
    let fields = [
        ("source_id", raw["source_id"].clone()),
        (
            "connect_string",
            raw["connection"]["connect_string"].clone(),
        ),
        ("statement", serde_json::json!(query.sql())),
        ("timestamp_column", serde_json::json!(spec.timestamp_column)),
        ("timestamp_bind", serde_json::json!(spec.timestamp_bind)),
        (
            "timestamp_initial",
            serde_json::json!(spec.initial.timestamp),
        ),
        (
            "tie_breaker_column",
            serde_json::json!(spec.tie_breaker_column),
        ),
        ("tie_breaker_bind", serde_json::json!(spec.tie_breaker_bind)),
        (
            "tie_breaker_initial",
            serde_json::json!(spec.initial.tie_breaker),
        ),
    ];
    let legacy = format!(
        "{{{}}}",
        fields
            .iter()
            .map(|(key, value)| format!("\"{key}\":{value}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert_eq!(
        config.config_fingerprint(),
        blake3::hash(legacy.as_bytes()).to_hex().as_str()
    );
}

/// Scenario: The Oracle factory reopens a durable scalar checkpoint for each supported type.
/// Guarantees: Typed progress and exact source leases survive factory construction and teardown.
#[test]
fn oracle_factory_preserves_scalar_checkpoints() {
    for value in scalar_values() {
        let dir = tempfile::tempdir().expect("state");
        let mut raw = scalar_config(value);
        raw["checkpoint"]["directory"] = serde_json::json!(dir.path());
        let config = parsed(raw.clone()).expect("config");
        let cursor = config.query().watermark().initial();
        assert!(matches!(cursor, Cursor::Scalar(_)));
        let store = CheckpointStore::new(
            dir.path(),
            "group",
            "pipeline",
            "oracle-audit",
            config.source_id(),
            config.config_fingerprint().into(),
        );
        let lease = SourceLease::acquire(store.lease_key()).expect("seed lease");
        let (saved, _) = store.write(0, &cursor).expect("checkpoint");
        drop(lease);
        let receiver = build(&pipeline_context(), "oracle-audit", &raw).expect("factory");
        assert!(SourceLease::acquire(store.lease_key()).is_err());
        assert_eq!(store.read().expect("read"), Some(saved.clone()));
        drop(receiver);
        let _lease = SourceLease::acquire(store.lease_key()).expect("released");
        assert_eq!(store.read().expect("restart"), Some(saved));
    }
}

/// Scenario: Equivalent scalar timestamp initial positions use UTC and non-UTC offsets.
/// Guarantees: The latest PR's canonical UTC normalization also yields identical scalar checkpoint fingerprints.
#[test]
fn scalar_initial_offsets_share_checkpoint_identity() {
    let utc = parsed(scalar_config(ScalarValue::Timestamp(
        "2026-01-01T06:30:00Z".into(),
    )))
    .expect("UTC");
    let offset = parsed(scalar_config(ScalarValue::Timestamp(
        "2026-01-01 12:00:00 +05:30".into(),
    )))
    .expect("offset");
    assert_eq!(
        utc.query().watermark().initial(),
        offset.query().watermark().initial()
    );
    assert_eq!(utc.config_fingerprint(), offset.config_fingerprint());
}
