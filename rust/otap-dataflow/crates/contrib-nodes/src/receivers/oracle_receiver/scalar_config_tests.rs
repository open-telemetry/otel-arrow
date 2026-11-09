// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_scraper::database::{CompiledWatermark, Cursor, ScalarValue};

fn scalar_config(value: ScalarValue) -> Value {
    let mut config = documented_config();
    config["watermark"] = serde_json::json!({"mode":"scalar", "column":"CURSOR_KEY", "bind":"last_key", "initial":value});
    config["query"]["statement"] = if matches!(value, ScalarValue::String(_)) {
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

/// Scenario: User SQL violates the required explicit keyset or result ordering.
/// Guarantees: Broader business expressions cannot bypass the checkpoint guard.
#[test]
fn scalar_sql_rejects_unsafe_lookalikes() {
    for sql in [
        "SELECT PAYLOAD FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT OTHER AS CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY + 1 AS CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY >= :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY = :last_key ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key OR 1 = 1 ORDER BY CURSOR_KEY ASC",
        "SELECT CURSOR_KEY FROM T WHERE CURSOR_KEY > :last_key_extra ORDER BY CURSOR_KEY ASC",
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
        "SELECT CURSOR_KEY, PAYLOAD AS BODY FROM T WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC".into();
    assert!(parsed(value).is_ok());
}

/// Scenario: A string query explicitly supplies its BINARY comparison and ordering.
/// Guarantees: The authored collation remains in the bound SQL and null-key checks do not change it.
#[test]
fn string_queries_require_explicit_binary_collation() {
    use otel_arrow_dfe_scraper::database::{ColumnMetadata, DatabaseSystem, SourceQuery};
    let query = parsed(scalar_config(ScalarValue::String("!".into())))
        .unwrap()
        .query();
    let source = SourceQuery::compile(&query, DatabaseSystem::Oracle).unwrap();
    let sql = source
        .paged_sql(
            &query,
            &[ColumnMetadata {
                name: "CURSOR_KEY".into(),
                source_type: "VARCHAR2".into(),
                nullable: true,
            }],
        )
        .unwrap();
    assert_eq!(sql.matches("COLLATE BINARY").count(), 3);
    assert!(sql.contains(":last_key COLLATE BINARY"));
    assert!(sql.ends_with("COLLATE BINARY ASC NULLS FIRST"));
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

/// Scenario: An unchanged explicit query is compared with the original fingerprint format.
/// Guarantees: Broader validation does not require existing users to reset their durable cursors.
#[test]
fn composite_fingerprint_remains_compatible() {
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

/// Scenario: Original Oracle composite, scalar, and snapshot configurations reopen durable state.
/// Guarantees: Frozen pre-relaxation identity payloads match without a migration or progress reset.
#[test]
fn original_identity_payloads_reopen_checkpoints() {
    // Identity bytes from 9145c0f's field order, SQL spelling, and timestamp normalization.
    const COMPOSITE: &str = r#"{"source_id":"oracle-audit","connect_string":"database.contoso.com:1521/ORCL","statement":"SELECT AUDIT_ID, LAST_UPDATED, PAYLOAD FROM AUDIT_LOGS WHERE (LAST_UPDATED > :last_timestamp OR (LAST_UPDATED = :last_timestamp AND AUDIT_ID > :last_tie_breaker)) ORDER BY LAST_UPDATED ASC, AUDIT_ID ASC","timestamp_column":"LAST_UPDATED","timestamp_bind":"last_timestamp","timestamp_initial":"1970-01-01 00:00:00.000000000 +00:00","tie_breaker_column":"AUDIT_ID","tie_breaker_bind":"last_tie_breaker","tie_breaker_initial":0}"#;
    const SCALAR: &str = r#"{"source_id":"oracle-audit","connect_string":"database.contoso.com:1521/ORCL","statement":"SELECT CURSOR_KEY, PAYLOAD FROM AUDIT_LOGS WHERE CURSOR_KEY > :last_key ORDER BY CURSOR_KEY ASC","mode":"scalar","column":"CURSOR_KEY","bind":"last_key","initial":{"type":"int64","value":0}}"#;
    const SNAPSHOT: &str = r#"{"source_id":"oracle-audit","connect_string":"database.contoso.com:1521/ORCL","statement":"SELECT PAYLOAD FROM AUDIT_LOGS","mode":"snapshot"}"#;
    let mut snapshot = documented_config();
    snapshot["watermark"] = serde_json::json!({"mode":"snapshot"});
    snapshot["query"]["statement"] = "SELECT PAYLOAD FROM AUDIT_LOGS".into();
    for (raw, legacy) in [
        (documented_config(), COMPOSITE),
        (scalar_config(ScalarValue::Int64(0)), SCALAR),
        (snapshot, SNAPSHOT),
    ] {
        let config = parsed(raw).unwrap();
        let expected = blake3::hash(legacy.as_bytes()).to_hex().to_string();
        assert_eq!(config.config_fingerprint(), expected);
        let dir = tempfile::tempdir().unwrap();
        let old = CheckpointStore::new(
            dir.path(),
            "group",
            "pipeline",
            "oracle",
            "source",
            expected,
        );
        let lease = SourceLease::acquire(old.lease_key()).unwrap();
        let progress = match config.query().watermark().initial() {
            Cursor::Composite(mut cursor) => {
                cursor.tie_breaker = 17;
                Cursor::Composite(cursor)
            }
            Cursor::Scalar(_) => ScalarValue::Int64(17).into(),
            Cursor::Snapshot => Cursor::Snapshot,
        };
        let (saved, _) = old.write(0, &progress).unwrap();
        drop(lease);
        let current = CheckpointStore::new(
            dir.path(),
            "group",
            "pipeline",
            "oracle",
            "source",
            config.config_fingerprint().into(),
        );
        assert_eq!(current.read().unwrap(), Some(saved));
    }
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
