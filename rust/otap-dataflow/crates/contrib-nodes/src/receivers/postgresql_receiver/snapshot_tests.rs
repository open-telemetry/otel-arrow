// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    adapter::{Error, PostgreSqlAdapter},
    config::PostgreSqlReceiverConfig,
    query::{CursorColumns, NativeColumn, Signature},
    worker,
};
use otel_arrow_dfe_scraper::database::{
    CellValue, CompiledWatermark, Cursor, CursorRow, DriverAdapter, Row,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tokio_postgres::types::Type;

fn config(statement: &str) -> Value {
    let root = std::env::current_dir().expect("working directory");
    json!({
        "source_id": "snapshot-test",
        "connection": {"host": "localhost", "database": "fixture"},
        "query": {
            "statement": statement,
            "fetch_size_rows": 2,
            "max_rows_per_poll": 2,
            "max_batch_bytes": "10 MiB"
        },
        "watermark": {"mode": "snapshot"},
        "checkpoint": {"directory": root.join("checkpoint")}
    })
}

fn validate(raw: &Value) -> Result<super::config::Validated, Error> {
    PostgreSqlReceiverConfig::parse(raw)?.validate()
}

fn signature(plan: &super::query::Plan) -> Signature {
    let columns = plan
        .projections
        .iter()
        .enumerate()
        .map(|(index, reference)| {
            (
                reference.clone(),
                NativeColumn {
                    oid: 42,
                    attribute: index as i16 + 1,
                    type_oid: Type::TEXT.oid(),
                    modifier: -1,
                    not_null: false,
                    collation: 0,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    Signature {
        columns,
        keys: vec![(42, vec![])],
    }
}

fn row(text: &str) -> CursorRow {
    CursorRow {
        row: Row {
            values: vec![CellValue::String(text.into())],
        },
        cursor: Cursor::Snapshot,
    }
}

/// Scenario: Snapshot mode uses the existing PostgreSQL query fields and omits cursor metadata.
/// Guarantees: It binds no parameters, defaults to one-minute polling, and probes one extra row.
#[test]
fn snapshot_configuration_has_no_cursor_contract() {
    let validated = validate(&config(
        "SELECT a.payload FROM public.audit_logs AS a ORDER BY a.payload ASC",
    ))
    .expect("snapshot config");
    assert!(matches!(
        validated.common.watermark(),
        CompiledWatermark::Snapshot
    ));
    assert_eq!(validated.common.watermark().initial(), Cursor::Snapshot);
    assert_eq!(
        validated.common.interval(),
        std::time::Duration::from_secs(60)
    );
    assert_eq!(validated.common.fetch_size_rows(), 2);
    assert_eq!(validated.plan.cursor, CursorColumns::Snapshot);
    assert!(validated.plan.sql.ends_with("LIMIT 3"));
    assert!(validated.common.output().timestamp_column.is_none());
}

/// Scenario: Snapshot SQL contains no ordering, direct ordering, static filters, or cursor binds.
/// Guarantees: Safe complete-result forms are accepted while cursor parameters and unsupported AST shapes fail closed.
#[test]
fn snapshot_sql_contract_is_bounded() {
    for statement in [
        "SELECT a.payload FROM public.audit_logs AS a",
        "SELECT a.payload FROM public.audit_logs AS a ORDER BY a.payload DESC",
        "SELECT a.payload FROM public.audit_logs AS a WHERE a.payload = 'ready'",
    ] {
        assert!(validate(&config(statement)).is_ok(), "{statement}");
    }
    for statement in [
        "SELECT a.payload FROM public.audit_logs AS a WHERE a.payload > :last_value",
        "SELECT a.payload FROM public.audit_logs AS a ORDER BY lower(a.payload)",
        "SELECT a.payload FROM public.audit_logs AS a LIMIT 1",
        "SELECT count(*) AS payload FROM public.audit_logs AS a",
    ] {
        assert!(validate(&config(statement)).is_err(), "{statement}");
    }
}

/// Scenario: Snapshot result columns are nullable and have no unique index.
/// Guarantees: Metadata discovery imposes no cursor nullability, type, or uniqueness requirement.
#[test]
fn snapshot_metadata_needs_no_cursor_or_unique_key() {
    let validated =
        validate(&config("SELECT a.payload FROM public.audit_logs AS a")).expect("snapshot config");
    assert!(
        super::query::ResolvedSchema::discover(
            &validated.plan,
            &validated.common,
            &signature(&validated.plan)
        )
        .is_ok()
    );
}

/// Scenario: A complete snapshot is exactly bounded, one row too long, or too large in normalized bytes.
/// Guarantees: Exact results succeed and every overflow returns an error without a partial page.
#[test]
fn snapshot_admission_is_all_or_nothing() {
    let mut rows = Vec::new();
    let mut payload = 0;
    assert!(
        worker::admit_row(&mut rows, &mut payload, row("one"), 2, 4096, true).expect("first row")
    );
    assert!(
        worker::admit_row(&mut rows, &mut payload, row("two"), 2, 4096, true).expect("second row")
    );
    assert_eq!(
        worker::admit_row(&mut rows, &mut payload, row("three"), 2, 4096, true),
        Err(Error::SnapshotRowLimit { limit: 2 })
    );

    let mut rows = Vec::new();
    let mut payload = 0;
    assert_eq!(
        worker::admit_row(&mut rows, &mut payload, row("too large"), 2, 1, true),
        Err(Error::SnapshotByteLimit { limit: 1 })
    );
    assert!(rows.is_empty());
}

/// Scenario: Snapshot row and normalized-byte limits are deterministic source-local failures.
/// Guarantees: Only confirmed complete-result bounds pause this source; ordinary protocol limits remain terminal.
#[test]
fn snapshot_limits_pause_only_the_source() {
    use otel_arrow_dfe_engine::error::ReceiverErrorKind;

    for error in [
        Error::SnapshotRowLimit { limit: 100 },
        Error::SnapshotByteLimit { limit: 4096 },
    ] {
        assert!(!PostgreSqlAdapter::is_retryable(&error));
        assert!(PostgreSqlAdapter::should_pause_source(&error));
        assert_eq!(
            PostgreSqlAdapter::classify_error(&error),
            ReceiverErrorKind::Configuration
        );
    }
    assert!(
        Error::SnapshotRowLimit { limit: 100 }
            .to_string()
            .contains("100 rows")
    );
    assert!(
        Error::SnapshotByteLimit { limit: 4096 }
            .to_string()
            .contains("4096 bytes")
    );
    assert!(!PostgreSqlAdapter::should_pause_source(&Error::Limit));
}

/// Scenario: Snapshot semantics stay fixed while polling and checkpoint policies are tuned.
/// Guarantees: Query identity changes invalidate progress and operational tuning preserves it.
#[test]
fn snapshot_fingerprint_tracks_only_semantic_identity() {
    let raw = config("SELECT a.payload FROM public.audit_logs AS a");
    let baseline = validate(&raw).expect("snapshot").fingerprint;

    let changed = config("SELECT a.other_payload FROM public.audit_logs AS a");
    assert_ne!(
        baseline,
        validate(&changed).expect("changed query").fingerprint
    );

    let mut tuned = raw;
    tuned["query"]["interval"] = json!("2m");
    tuned["query"]["fetch_size_rows"] = json!(1);
    tuned["query"]["max_rows_per_poll"] = json!(3);
    tuned["query"]["max_batch_bytes"] = json!("12 MiB");
    tuned["checkpoint"]["directory"] = json!(
        std::env::current_dir()
            .expect("working directory")
            .join("other-checkpoint")
    );
    tuned["checkpoint"]["on_permanent_nack"] = json!("retry");
    assert_eq!(baseline, validate(&tuned).expect("tuned").fingerprint);
}
