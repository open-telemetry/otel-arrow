// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use otel_arrow_dfe_scraper::database::{CompiledWatermark, Cursor};

fn snapshot_config() -> Value {
    let mut config = documented_config();
    config["watermark"] = serde_json::json!({"mode":"snapshot"});
    config["query"]["statement"] = "SELECT PAYLOAD FROM AUDIT_LOGS".into();
    config
}

/// Scenario: An Oracle snapshot query has no key, ordering, watermark binds, or initial value and uses quoted aliases.
/// Guarantees: Configuration compiles with no cursor-column or event-time requirements and ignores quoted identifier contents.
#[test]
fn snapshot_queries_need_no_order_or_cursor() {
    for sql in [
        "SELECT PAYLOAD FROM AUDIT_LOGS",
        "SELECT PAYLOAD FROM AUDIT_LOGS WHERE PAYLOAD IS NOT NULL",
        "SELECT COUNT(*) AS TOTAL FROM AUDIT_LOGS",
        "SELECT PAYLOAD FROM AUDIT_LOGS ORDER BY PAYLOAD DESC;",
        "SELECT ':not_a_bind' AS PAYLOAD FROM DUAL",
        r#"SELECT PAYLOAD AS "FOR" FROM AUDIT_LOGS"#,
        r#"SELECT PAYLOAD AS "INTO" FROM AUDIT_LOGS"#,
        r#"SELECT PAYLOAD AS ":key" FROM AUDIT_LOGS"#,
        r#"SELECT PAYLOAD AS "SELECT" FROM AUDIT_LOGS"#,
        r#"SELECT PAYLOAD AS "a'" FROM AUDIT_LOGS ORDER BY "a'""#,
        r#"SELECT PAYLOAD AS "a""b" FROM AUDIT_LOGS"#,
    ] {
        let mut raw = snapshot_config();
        raw["query"]["statement"] = sql.into();
        let query = parsed(raw).expect("snapshot config").query();
        assert!(matches!(query.watermark(), CompiledWatermark::Snapshot));
        assert_eq!(query.watermark().initial(), Cursor::Snapshot);
        assert!(query.output().validation_columns.is_empty());
        assert!(query.output().timestamp_column.is_none());
    }
}

/// Scenario: Snapshot SQL contains visible or quote-hidden parameters, locks, extra statements, or nested SELECTs.
/// Guarantees: Quoted identifiers and unsupported Oracle q-quotes cannot hide SQL from the read-only contract.
#[test]
fn snapshot_sql_rejects_unsafe_or_unbound_statements() {
    for sql in [
        "SELECT PAYLOAD FROM AUDIT_LOGS WHERE PAYLOAD = :key",
        "SELECT PAYLOAD FROM AUDIT_LOGS WHERE PAYLOAD = :1",
        "SELECT PAYLOAD FROM AUDIT_LOGS FOR UPDATE",
        "SELECT PAYLOAD INTO TARGET FROM AUDIT_LOGS",
        "DELETE FROM AUDIT_LOGS",
        "SELECT PAYLOAD FROM AUDIT_LOGS; DELETE FROM AUDIT_LOGS",
        "SELECT PAYLOAD FROM AUDIT_LOGS -- ignored",
        "SELECT PAYLOAD FROM (SELECT PAYLOAD FROM AUDIT_LOGS)",
        "SELECT PAYLOAD FROM AUDIT_LOGS UNION SELECT PAYLOAD FROM OTHER",
        r#"SELECT PAYLOAD AS "a'" FROM AUDIT_LOGS WHERE PAYLOAD = :key ORDER BY "a'""#,
        r#"SELECT PAYLOAD AS "a'" FROM (SELECT PAYLOAD FROM AUDIT_LOGS) ORDER BY "a'""#,
        r#"SELECT PAYLOAD AS "unterminated FROM AUDIT_LOGS"#,
        "SELECT q'[it's]' AS PAYLOAD FROM DUAL WHERE DUMMY = :key",
        "SELECT nq'<it's>' AS PAYLOAD FROM DUAL WHERE DUMMY = :key",
        "SELECT Q'{SELECT PAYLOAD FROM AUDIT_LOGS}' AS PAYLOAD FROM DUAL",
    ] {
        let mut raw = snapshot_config();
        raw["query"]["statement"] = sql.into();
        assert!(parsed(raw).is_err(), "{sql}");
    }
    for field in ["column", "bind", "initial", "timestamp", "tie_breaker"] {
        let mut raw = snapshot_config();
        raw["watermark"][field] = "unused".into();
        assert!(parsed(raw).is_err(), "{field}");
    }
}

/// Scenario: Source, SQL, or database identity changes while operational settings or the client installation move.
/// Guarantees: Snapshot checkpoints track semantic identity without invalidating on client relocation.
#[test]
fn snapshot_fingerprint_tracks_only_semantic_identity() {
    let raw = snapshot_config();
    let baseline = parsed(raw.clone()).expect("base");
    for (section, field, value) in [
        ("query", "statement", "SELECT OTHER FROM AUDIT_LOGS"),
        ("connection", "connect_string", "other.example/FREEPDB1"),
    ] {
        let mut changed = raw.clone();
        changed[section][field] = value.into();
        assert_ne!(
            baseline.config_fingerprint(),
            parsed(changed).expect("changed").config_fingerprint()
        );
    }
    let mut changed = raw.clone();
    changed["source_id"] = "another-source".into();
    assert_ne!(
        baseline.config_fingerprint(),
        parsed(changed).expect("source").config_fingerprint()
    );
    let mut changed = raw;
    changed["connection"]["instant_client_dir"] = "/opt/oracle/ic-23".into();
    changed["query"]["interval"] = "2m".into();
    assert_eq!(
        baseline.config_fingerprint(),
        parsed(changed).expect("relocation").config_fingerprint()
    );
    assert_ne!(
        baseline.config_fingerprint(),
        parsed(documented_config())
            .expect("composite")
            .config_fingerprint()
    );
}

/// Scenario: A factory is constructed with a previously acknowledged snapshot checkpoint.
/// Guarantees: No column value is persisted and the normal source lease and revision survive restart.
#[test]
fn snapshot_factory_preserves_checkpoint_and_ownership() {
    let directory = tempfile::tempdir().expect("state directory");
    let mut raw = snapshot_config();
    raw["checkpoint"]["directory"] = serde_json::json!(directory.path());
    let config = parsed(raw.clone()).expect("snapshot config");
    let store = CheckpointStore::new(
        directory.path(),
        "group",
        "pipeline",
        "oracle-audit",
        config.source_id(),
        config.config_fingerprint().into(),
    );
    let lease = SourceLease::acquire(store.lease_key()).expect("seed lease");
    let (saved, _) = store.write(0, &Cursor::Snapshot).expect("checkpoint");
    drop(lease);
    let receiver = build(&pipeline_context(), "oracle-audit", &raw).expect("factory");
    assert!(SourceLease::acquire(store.lease_key()).is_err());
    assert_eq!(store.read().expect("read"), Some(saved.clone()));
    drop(receiver);
    let _lease = SourceLease::acquire(store.lease_key()).expect("released");
    assert_eq!(store.read().expect("restart"), Some(saved));
}
