// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    adapter::Error,
    config::PostgreSqlReceiverConfig,
    query::{CursorColumns, NativeColumn, ScalarKind, Signature},
    value,
};
use otel_arrow_dfe_scraper::database::{CompiledWatermark, Cursor, ScalarValue};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio_postgres::types::Type;

fn config(initial: Value, kind: &str) -> Value {
    let root = std::env::current_dir().expect("working directory");
    json!({
        "source_id": "scalar-test",
        "connection": {"host": "localhost", "database": "fixture"},
        "query": {
            "statement": "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :last_value ORDER BY a.audit_id ASC",
            "interval": "5m",
            "fetch_size_rows": 1000,
            "max_rows_per_poll": 10000,
            "max_batch_bytes": "10 MiB"
        },
        "watermark": {
            "mode": "scalar",
            "column": "audit_id",
            "bind": "last_value",
            "initial": {"type": kind, "value": initial}
        },
        "checkpoint": {"directory": root.join("checkpoint")}
    })
}

fn validate(raw: &Value) -> Result<super::config::Validated, Error> {
    PostgreSqlReceiverConfig::parse(raw)?.validate()
}

fn signature(
    plan: &super::query::Plan,
    cursor_type: Type,
    modifier: i32,
    not_null: bool,
    unique: bool,
) -> Signature {
    let mut columns = BTreeMap::new();
    for (index, reference) in plan.projections.iter().enumerate() {
        let ty = if index == 0 {
            cursor_type.clone()
        } else {
            Type::TEXT
        };
        let _ = columns.insert(
            reference.clone(),
            NativeColumn {
                oid: 42,
                attribute: index as i16 + 1,
                type_oid: ty.oid(),
                modifier: if index == 0 { modifier } else { -1 },
                not_null: index != 0 || not_null,
                collation: 0,
            },
        );
    }
    Signature {
        columns,
        keys: vec![(42, if unique { vec![(43, vec![1])] } else { vec![] })],
    }
}

/// Scenario: The documented integer scalar configuration is parsed and compiled.
/// Guarantees: The shared flat scalar shape becomes one typed bind and preserves existing query limits.
#[test]
fn integer_scalar_configuration_uses_existing_postgresql_schema() {
    let validated = validate(&config(json!(0), "int64")).expect("scalar config");
    let CompiledWatermark::Scalar(watermark) = validated.common.watermark() else {
        panic!("scalar watermark")
    };
    assert_eq!(watermark.column, "audit_id");
    assert_eq!(watermark.bind, "last_value");
    assert_eq!(watermark.initial, ScalarValue::Int64(0));
    assert_eq!(validated.common.fetch_size_rows(), 1000);
    assert_eq!(validated.common.max_rows(), 10000);
    assert!(validated.plan.sql.contains("a.audit_id > $1"));
    assert!(validated.plan.sql.ends_with("LIMIT 10000"));
    assert_eq!(
        validated.plan.cursor,
        CursorColumns::Scalar {
            index: 0,
            kind: ScalarKind::Int64
        }
    );
}

/// Scenario: Equivalent timestamp scalar initial values use offsets or timezone-less UTC text.
/// Guarantees: Configuration, checkpoints, and fingerprints retain one exact microsecond UTC value.
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
            "SELECT a.last_updated, a.payload FROM public.audit_logs AS a WHERE a.last_updated > :last_value ORDER BY a.last_updated ASC"
        );
        raw["watermark"]["column"] = json!("last_updated");
        let validated = validate(&raw).expect("timestamp scalar");
        let CompiledWatermark::Scalar(watermark) = validated.common.watermark() else {
            panic!("scalar watermark")
        };
        assert_eq!(
            watermark.initial,
            ScalarValue::Timestamp("2026-01-01T00:00:00.123456Z".into())
        );
        assert_eq!(
            validated.plan.cursor,
            CursorColumns::Scalar {
                index: 0,
                kind: ScalarKind::Timestamp
            }
        );
        fingerprints.push(validated.fingerprint);
    }
    assert!(fingerprints.windows(2).all(|pair| pair[0] == pair[1]));
}

/// Scenario: Scalar cursor semantics change while polling and checkpoint policies are tuned.
/// Guarantees: Key type, value, column, bind, and SQL affect identity; operational tuning does not.
#[test]
fn scalar_fingerprint_tracks_semantic_identity() {
    let raw = config(json!(0), "int64");
    let baseline = validate(&raw).expect("scalar").fingerprint;

    let mut changed = raw.clone();
    changed["watermark"]["initial"]["value"] = json!(1);
    assert_ne!(
        baseline,
        validate(&changed).expect("changed initial").fingerprint
    );

    let mut changed = raw.clone();
    changed["watermark"]["bind"] = json!("new_value");
    changed["query"]["statement"] = json!(
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :new_value ORDER BY a.audit_id ASC"
    );
    assert_ne!(
        baseline,
        validate(&changed).expect("changed bind").fingerprint
    );

    let mut changed = raw.clone();
    changed["watermark"]["column"] = json!("sequence_id");
    changed["query"]["statement"] = json!(
        "SELECT a.sequence_id, a.payload FROM public.audit_logs AS a WHERE a.sequence_id > :last_value ORDER BY a.sequence_id ASC"
    );
    assert_ne!(
        baseline,
        validate(&changed).expect("changed column").fingerprint
    );

    let mut changed = raw.clone();
    changed["watermark"]["initial"]["type"] = json!("timestamp");
    changed["watermark"]["initial"]["value"] = json!("2026-01-01T00:00:00Z");
    changed["watermark"]["column"] = json!("last_updated");
    changed["query"]["statement"] = json!(
        "SELECT a.last_updated, a.payload FROM public.audit_logs AS a WHERE a.last_updated > :last_value ORDER BY a.last_updated ASC"
    );
    assert_ne!(
        baseline,
        validate(&changed).expect("changed type").fingerprint
    );

    let mut tuned = raw;
    tuned["query"]["interval"] = json!("2m");
    tuned["query"]["fetch_size_rows"] = json!(500);
    tuned["query"]["max_rows_per_poll"] = json!(9000);
    tuned["query"]["max_batch_bytes"] = json!("12 MiB");
    tuned["query"]["catch_up"] = json!({"max_pages": 8, "max_duration": "5s"});
    tuned["checkpoint"]["directory"] = json!(
        std::env::current_dir()
            .expect("working directory")
            .join("other-checkpoint")
    );
    tuned["checkpoint"]["on_permanent_nack"] = json!("retry");
    assert_eq!(baseline, validate(&tuned).expect("tuned").fingerprint);
}

/// Scenario: Scalar configuration uses unsupported types, malformed initial values, or cross-mode fields.
/// Guarantees: Invalid customer input fails before opening a PostgreSQL connection.
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
    let mut extra = config(json!(0), "int64");
    extra["watermark"]["timestamp"] = json!({"column": "ts"});
    assert!(validate(&extra).is_err());

    let mut nested = config(json!(0), "int64");
    nested["watermark"] = json!({
        "mode": "scalar",
        "value": {
            "column": "audit_id",
            "bind": "last_value",
            "type": "int64",
            "initial": 0
        }
    });
    assert!(validate(&nested).is_err());
}

/// Scenario: Scalar SQL omits, duplicates, or reorders the configured keyset predicate.
/// Guarantees: Only one strict greater-than bind and ascending cursor ordering are accepted.
#[test]
fn scalar_sql_contract_is_strict() {
    let base = config(json!(0), "int64");
    for statement in [
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a ORDER BY a.audit_id ASC",
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id >= :last_value ORDER BY a.audit_id ASC",
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :other ORDER BY a.audit_id ASC",
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :last_value ORDER BY a.audit_id DESC",
        "SELECT a.audit_id, a.payload FROM public.audit_logs AS a WHERE a.audit_id > :last_value ORDER BY a.audit_id, a.payload",
    ] {
        let mut raw = base.clone();
        raw["query"]["statement"] = json!(statement);
        assert!(validate(&raw).is_err(), "{statement}");
    }
}

/// Scenario: Scalar metadata changes native type, nullability, or unique-index coverage.
/// Guarantees: Exact supported types and a single-column driving key are required.
#[test]
fn scalar_metadata_requires_exact_type_and_unique_key() {
    let validated = validate(&config(json!(0), "int64")).expect("scalar config");
    assert!(
        super::query::ResolvedSchema::discover(
            &validated.plan,
            &validated.common,
            &signature(&validated.plan, Type::INT8, -1, true, true)
        )
        .is_ok()
    );
    for (ty, not_null, unique) in [
        (Type::INT8, false, true),
        (Type::INT8, true, false),
        (Type::NUMERIC, true, true),
    ] {
        assert!(
            super::query::ResolvedSchema::discover(
                &validated.plan,
                &validated.common,
                &signature(&validated.plan, ty, -1, not_null, unique)
            )
            .is_err()
        );
    }
}

/// Scenario: A timestamp scalar uses either PostgreSQL timestamp representation.
/// Guarantees: Native metadata, initial binding, and cursor extraction retain the canonical UTC value.
#[test]
fn timestamp_scalar_uses_native_timestamp_types() {
    let mut raw = config(json!("2026-01-01T00:00:00.123456Z"), "timestamp");
    raw["query"]["statement"] = json!(
        "SELECT a.last_updated, a.payload FROM public.audit_logs AS a WHERE a.last_updated > :last_value ORDER BY a.last_updated ASC"
    );
    raw["watermark"]["column"] = json!("last_updated");
    let validated = validate(&raw).expect("timestamp scalar");

    for native in [Type::TIMESTAMP, Type::TIMESTAMPTZ] {
        let schema = super::query::ResolvedSchema::discover(
            &validated.plan,
            &validated.common,
            &signature(&validated.plan, native.clone(), 6, true, true),
        )
        .expect("timestamp schema");
        assert_eq!(schema.native_types[0], native);
        assert!(
            value::parameters(
                &Cursor::Scalar(ScalarValue::Timestamp("2026-01-01T00:00:00.123456Z".into())),
                &validated.plan,
                &schema
            )
            .is_ok()
        );
    }
}

/// Scenario: Scalar cursor binding reaches native integer widths and compares ordered positions.
/// Guarantees: Values bind without coercion and incompatible checkpoint modes fail closed.
#[test]
fn scalar_parameters_and_ordering_are_exact() {
    let validated = validate(&config(json!(0), "int64")).expect("scalar config");
    let schema = super::query::ResolvedSchema::discover(
        &validated.plan,
        &validated.common,
        &signature(&validated.plan, Type::INT2, -1, true, true),
    )
    .expect("schema");
    assert!(
        value::parameters(
            &Cursor::Scalar(ScalarValue::Int64(i16::MAX as i64)),
            &validated.plan,
            &schema
        )
        .is_ok()
    );
    assert!(
        value::parameters(
            &Cursor::Scalar(ScalarValue::Int64(i16::MAX as i64 + 1)),
            &validated.plan,
            &schema
        )
        .is_err()
    );
    assert!(value::parameters(&Cursor::Snapshot, &validated.plan, &schema).is_err());
    let first = value::cursor_position(&Cursor::Scalar(ScalarValue::Int64(1))).expect("first");
    let next = value::cursor_position(&Cursor::Scalar(ScalarValue::Int64(2))).expect("next");
    assert!(
        first
            .strictly_precedes(&next)
            .expect("compatible positions")
    );
    assert!(
        !next
            .strictly_precedes(&first)
            .expect("compatible positions")
    );
}
