// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    adapter::Error,
    config::{PostgreSqlReceiverConfig, Validated},
    query::{CursorColumns, Plan, ResolvedSchema, ScalarKind},
    tests::mock_schema,
    value,
};
use otel_arrow_dfe_scraper::database::{CellValue, CompiledWatermark, Cursor, ScalarValue};
use serde_json::{Value, json};
use tokio_postgres::types::Type;

fn config(initial: Value, kind: &str) -> Value {
    json!({
        "source_id": "scalar-test",
        "connection": {"host": "localhost", "database": "fixture"},
        "query": {
            "statement": "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :last_value ORDER BY a.audit_id ASC",
            "interval": "5m", "fetch_size_rows": 1000,
            "max_rows_per_poll": 10000, "max_batch_bytes": "10 MiB"
        },
        "watermark": {
            "mode": "scalar", "column": "audit_id", "bind": "last_value",
            "initial": {"type": kind, "value": initial}
        },
        "checkpoint": {"directory": std::env::current_dir().unwrap().join("checkpoint")}
    })
}

fn validate(raw: &Value) -> Result<Validated, Error> {
    PostgreSqlReceiverConfig::parse(raw)?.validate()
}

fn resolve(
    validated: &Validated,
    native: Type,
    modifier: i32,
) -> Result<(Plan, ResolvedSchema), Error> {
    let CompiledWatermark::Scalar(cursor) = validated.common.watermark() else {
        panic!("scalar fixture");
    };
    let schema = mock_schema(&[
        (&cursor.column, native, modifier, true),
        ("payload", Type::TEXT, -1, true),
    ]);
    let plan = Plan::resolve(&validated.common, &validated.source, &schema)?;
    Ok((plan, schema))
}

/// Scenario: A scalar query retains its explicit cursor predicate and ascending ordering.
/// Guarantees: Native metadata supplies positions and bounded execution includes a successor probe.
#[test]
fn integer_scalar_configuration_uses_native_result_metadata() {
    let validated = validate(&config(json!(0), "int64")).unwrap();
    let CompiledWatermark::Scalar(cursor) = validated.common.watermark() else {
        panic!("scalar watermark");
    };
    assert_eq!(cursor.column, "audit_id");
    assert_eq!(cursor.bind, "last_value");
    assert_eq!(cursor.initial, ScalarValue::Int64(0));
    assert_eq!(validated.common.fetch_size_rows(), 1000);
    let (plan, _) = resolve(&validated, Type::INT8, -1).unwrap();
    assert!(plan.sql.contains("a.audit_id > $1"));
    assert!(plan.sql.contains("a.audit_id IS NULL"));
    assert!(plan.sql.ends_with("LIMIT 10001"));
    assert_eq!(
        plan.cursor,
        CursorColumns::Scalar {
            index: 0,
            kind: ScalarKind::Int64
        }
    );
}

/// Scenario: Equivalent initial timestamps use offsets or timezone-less UTC text.
/// Guarantees: Configuration and fingerprints retain one exact microsecond UTC value.
#[test]
fn timestamp_scalar_initial_is_normalized() {
    let mut fingerprints = Vec::new();
    for initial in [
        "2026-01-01T00:00:00.123456Z",
        "2026-01-01T02:00:00.123456+02:00",
        "2026-01-01 00:00:00.123456",
    ] {
        let mut raw = config(json!(initial), "timestamp");
        raw["query"]["statement"] = json!(
            "SELECT a.last_updated, a.payload FROM public.audit_logs a WHERE a.last_updated > :last_value ORDER BY a.last_updated"
        );
        raw["watermark"]["column"] = json!("last_updated");
        let validated = validate(&raw).unwrap();
        let CompiledWatermark::Scalar(cursor) = validated.common.watermark() else {
            panic!("scalar watermark");
        };
        assert_eq!(
            cursor.initial,
            ScalarValue::Timestamp("2026-01-01T00:00:00.123456Z".into())
        );
        let (plan, _) = resolve(&validated, Type::TIMESTAMPTZ, 6).unwrap();
        assert_eq!(
            plan.cursor,
            CursorColumns::Scalar {
                index: 0,
                kind: ScalarKind::Timestamp
            }
        );
        fingerprints.push(validated.fingerprint);
    }
    assert!(fingerprints.windows(2).all(|pair| pair[0] == pair[1]));
}

/// Scenario: Cursor identity changes independently of operational polling and checkpoint settings.
/// Guarantees: Semantic changes invalidate fingerprints while budget tuning preserves progress.
#[test]
fn scalar_fingerprint_tracks_semantic_identity() {
    let raw = config(json!(0), "int64");
    let baseline = validate(&raw).unwrap().fingerprint;
    for (field, value) in [
        ("column", json!("sequence_id")),
        ("bind", json!("new_value")),
        ("initial", json!({"type": "int64", "value": 1})),
        (
            "initial",
            json!({"type": "timestamp", "value": "2026-01-01T00:00:00Z"}),
        ),
    ] {
        let mut changed = raw.clone();
        changed["watermark"][field] = value;
        if field == "column" {
            changed["query"]["statement"] = json!(
                "SELECT a.sequence_id,a.payload FROM public.audit_logs a WHERE a.sequence_id > :last_value ORDER BY a.sequence_id"
            );
        } else if field == "bind" {
            changed["query"]["statement"] = json!(
                raw["query"]["statement"]
                    .as_str()
                    .unwrap()
                    .replace(":last_value", ":new_value")
            );
        }
        assert_ne!(baseline, validate(&changed).unwrap().fingerprint);
    }
    let mut changed = raw.clone();
    changed["query"]["statement"] = json!(
        "SELECT sequence_id AS audit_id, payload FROM public.other WHERE sequence_id > :last_value ORDER BY audit_id"
    );
    assert_ne!(baseline, validate(&changed).unwrap().fingerprint);
    let mut tuned = raw;
    tuned["query"]["interval"] = json!("2m");
    tuned["query"]["fetch_size_rows"] = json!(500);
    tuned["query"]["max_rows_per_poll"] = json!(9000);
    tuned["query"]["max_batch_bytes"] = json!("12 MiB");
    tuned["query"]["catch_up"] = json!({"max_pages": 8, "max_duration": "5s"});
    tuned["checkpoint"]["directory"] = json!(std::env::current_dir().unwrap().join("other"));
    tuned["checkpoint"]["on_permanent_nack"] = json!("retry");
    assert_eq!(baseline, validate(&tuned).unwrap().fingerprint);
}

/// Scenario: Scalar configuration uses unsupported types or cross-mode fields.
/// Guarantees: Invalid typed configuration fails before any connection is opened.
#[test]
fn scalar_configuration_rejects_unsupported_shapes() {
    for raw in [
        config(json!("key"), "string"),
        config(json!(u64::MAX), "uint64"),
        config(json!("0"), "int64"),
        config(json!(0), "timestamp"),
    ] {
        assert!(validate(&raw).is_err());
    }
    let mut raw = config(json!(0), "int64");
    raw["watermark"]["timestamp"] = json!({"column": "ts"});
    assert!(validate(&raw).is_err());
}

/// Scenario: SQL omits its final order or conflicts with bounded incremental paging.
/// Guarantees: Mandatory cursor safeguards remain while native filters and aliases are accepted.
#[test]
fn scalar_sql_uses_the_shared_source_contract() {
    let base = config(json!(0), "int64");
    for sql in [
        "SELECT a.audit_id, a.payload FROM public.audit_logs a WHERE a.audit_id > :last_value",
        "SELECT a.audit_id, a.payload FROM public.audit_logs a WHERE a.audit_id > $1",
        "SELECT a.audit_id, a.payload FROM public.audit_logs a ORDER BY a.audit_id DESC",
        "SELECT a.audit_id, a.payload FROM public.audit_logs a LIMIT 10",
    ] {
        let mut raw = base.clone();
        raw["query"]["statement"] = json!(sql);
        assert!(validate(&raw).is_err(), "{sql}");
    }
    for sql in [
        "SELECT a.audit_id, a.payload FROM public.audit_logs a WHERE a.audit_id > :last_value AND a.created_at >= TIMESTAMP '2026-11-01 00:00:00' AND a.status IN ('ready', 'retry') ORDER BY a.audit_id",
        "SELECT a.sequence_id AS audit_id, a.payload FROM public.audit_view a WHERE a.sequence_id > :last_value ORDER BY audit_id",
    ] {
        let mut raw = base.clone();
        raw["query"]["statement"] = json!(sql);
        assert!(validate(&raw).is_ok(), "{sql}");
    }
}

/// Scenario: Cursor metadata comes from an expression or view without NOT NULL or index proof.
/// Guarantees: Supported native types are accepted and actual null cursor values remain errors.
#[test]
fn scalar_metadata_requires_native_type_not_catalog_provenance() {
    let validated = validate(&config(json!(0), "int64")).unwrap();
    for native in [Type::INT2, Type::INT4, Type::INT8] {
        let (_, schema) = resolve(&validated, native, -1).unwrap();
        assert!(schema.columns[0].nullable);
    }
    for native in [Type::NUMERIC, Type::TEXT, Type::TIMESTAMP] {
        assert!(resolve(&validated, native, -1).is_err());
    }
    assert!(value::scalar_cell(&CellValue::Null, ScalarKind::Int64).is_err());
}

/// Scenario: A scalar timestamp uses either native timestamp representation and finite precision.
/// Guarantees: Initial binding preserves exact UTC values and rejects precision loss.
#[test]
fn timestamp_scalar_uses_native_timestamp_types() {
    let validated = validate(&config(json!("2026-01-01T00:00:00.123456Z"), "timestamp")).unwrap();
    for native in [Type::TIMESTAMP, Type::TIMESTAMPTZ] {
        let (plan, schema) = resolve(&validated, native.clone(), 6).unwrap();
        assert_eq!(schema.native_types[0], native);
        assert!(value::parameters(&validated.common.watermark().initial(), &plan, &schema).is_ok());
        assert!(resolve(&validated, native, 3).is_err());
    }
}

/// Scenario: Scalar bindings reach native integer limits and encounter a different checkpoint mode.
/// Guarantees: No coercion, overflow, or snapshot-as-scalar interpretation is allowed.
#[test]
fn scalar_parameters_and_ordering_are_exact() {
    let validated = validate(&config(json!(0), "int64")).unwrap();
    let (plan, schema) = resolve(&validated, Type::INT2, -1).unwrap();
    assert!(value::parameters(&ScalarValue::Int64(i16::MAX as i64).into(), &plan, &schema).is_ok());
    assert!(
        value::parameters(
            &ScalarValue::Int64(i16::MAX as i64 + 1).into(),
            &plan,
            &schema
        )
        .is_err()
    );
    assert!(value::parameters(&Cursor::Snapshot, &plan, &schema).is_err());
    let first = value::cursor_position(&ScalarValue::Int64(1).into()).unwrap();
    let next = value::cursor_position(&ScalarValue::Int64(2).into()).unwrap();
    assert!(first.strictly_precedes(&next).unwrap());
    assert!(!next.strictly_precedes(&first).unwrap());
}
