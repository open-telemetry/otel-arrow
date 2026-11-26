// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::database::{Cursor, ScalarWatermark};
use crate::{CheckpointError, CheckpointStore, SourceBinding, SourceLease};
use serde_json::json;

fn endpoint() -> DatabaseEndpoint<'static> {
    DatabaseEndpoint::Network {
        host: "db.example",
        port: 5432,
        database: "events",
    }
}

fn scalar(initial: ScalarValue) -> CompiledWatermark {
    CompiledWatermark::Scalar(ScalarWatermark {
        column: "id".into(),
        bind: "last_id".into(),
        initial,
    })
}

fn context() -> SourceContext {
    serde_json::from_value(json!({"principal":"reader", "schema":"public"})).unwrap()
}

fn identity(
    sql: &str,
    watermark: &CompiledWatermark,
    context: Option<&SourceContext>,
) -> LogicalSourceId {
    LogicalSourceId::new(
        DatabaseSystem::PostgreSQL,
        endpoint(),
        sql,
        watermark,
        context,
    )
}

/// Scenario: The same query uses different storage paths, node names, and starting positions.
/// Guarantees: Logical identity is path-independent while storage locks remain distinct and are not claimed as source ownership.
#[test]
fn logical_identity_is_independent_of_storage_and_initial_position() {
    let first = identity(
        "SELECT id FROM events",
        &scalar(ScalarValue::Int64(0)),
        None,
    );
    let second = identity(
        "SELECT id FROM events",
        &scalar(ScalarValue::Int64(100)),
        None,
    );
    assert_eq!(first, second);
    let root = tempfile::tempdir_in(".").unwrap();
    let a = CheckpointStore::new(
        root.path(),
        "group-a",
        "pipeline-a",
        "receiver-a",
        "a",
        "legacy".into(),
    );
    let b = CheckpointStore::new(
        &root.path().join("elsewhere"),
        "group-b",
        "pipeline-b",
        "receiver-b",
        "b",
        "legacy".into(),
    );
    assert_ne!(a.lease_key(), b.lease_key());
    let a = SourceBinding::acquire_for_source(a, first).unwrap();
    let b = SourceBinding::acquire_for_source(b, second).unwrap();
    assert_eq!(a.logical_source_id(), b.logical_source_id());
}

/// Scenario: Database, query predicate, cursor interpretation, or execution semantics changes.
/// Guarantees: Distinct configured ranges and interpretations cannot share the same logical ID.
#[test]
fn logical_identity_tracks_database_query_cursor_and_context() {
    let watermark = scalar(ScalarValue::Int64(0));
    let original = identity("SELECT id FROM events WHERE tenant = 1", &watermark, None);
    assert_ne!(
        original,
        LogicalSourceId::new(
            DatabaseSystem::SqlServer,
            endpoint(),
            "SELECT id FROM events WHERE tenant = 1",
            &watermark,
            None,
        )
    );
    for (column, bind) in [("other_id", "last_id"), ("id", "other_bind")] {
        let changed = CompiledWatermark::Scalar(ScalarWatermark {
            column: column.into(),
            bind: bind.into(),
            initial: ScalarValue::Int64(0),
        });
        assert_ne!(
            original,
            identity("SELECT id FROM events WHERE tenant = 1", &changed, None)
        );
    }
    assert_ne!(
        original,
        identity("SELECT id FROM events WHERE tenant = 2", &watermark, None)
    );
    assert_ne!(
        original,
        identity(
            "SELECT id FROM events WHERE tenant = 1",
            &CompiledWatermark::Snapshot,
            None
        )
    );
    assert_ne!(
        original,
        identity(
            "SELECT id FROM events WHERE tenant = 1",
            &scalar(ScalarValue::String("0".into())),
            None
        )
    );
    assert_ne!(
        original,
        identity(
            "SELECT id FROM events WHERE tenant = 1",
            &watermark,
            Some(&context())
        )
    );
    for changed in [
        DatabaseEndpoint::Network {
            host: "other.example",
            port: 5432,
            database: "events",
        },
        DatabaseEndpoint::Network {
            host: "db.example",
            port: 5433,
            database: "events",
        },
        DatabaseEndpoint::Network {
            host: "db.example",
            port: 5432,
            database: "Events",
        },
    ] {
        assert_ne!(
            original,
            LogicalSourceId::new(
                DatabaseSystem::PostgreSQL,
                changed,
                "SELECT id FROM events WHERE tenant = 1",
                &watermark,
                None
            )
        );
    }
}

/// Scenario: Explicit context is introduced or its principal, schema, or visibility revision changes.
/// Guarantees: Existing progress fails closed without rewriting state and remains recoverable with the original context.
#[test]
fn context_changes_reject_saved_progress_without_reset() {
    let root = tempfile::tempdir_in(".").unwrap();
    let store =
        |fingerprint| CheckpointStore::new(root.path(), "g", "p", "r", "source", fingerprint);
    let legacy = blake3::hash(b"original adapter fingerprint")
        .to_hex()
        .to_string();
    let original = store(legacy.clone());
    let _lease = SourceLease::acquire(original.lease_key()).unwrap();
    let saved = original
        .write(0, &Cursor::Scalar(ScalarValue::Int64(42)))
        .unwrap()
        .0;
    let declared = context();
    let mut contexts = vec![declared.clone()];
    let mut principal = declared.clone();
    principal.principal = "other-reader".into();
    contexts.push(principal);
    let mut schema = declared.clone();
    schema.schema = "other-schema".into();
    contexts.push(schema);
    let mut policy = declared.clone();
    policy.revision = "2".into();
    contexts.push(policy);
    let mut fingerprints = std::collections::HashSet::new();
    let other_root = tempfile::tempdir_in(".").unwrap();
    let contextual_store =
        |fingerprint| CheckpointStore::new(other_root.path(), "g", "p", "r", "source", fingerprint);
    let fresh = contextual_store(declared.checkpoint_fingerprint(&legacy));
    let _contextual_lease = SourceLease::acquire(fresh.lease_key()).unwrap();
    let contextual_saved = fresh
        .write(0, &Cursor::Scalar(ScalarValue::Int64(43)))
        .unwrap()
        .0;
    for (index, context) in contexts.into_iter().enumerate() {
        let fingerprint = context.checkpoint_fingerprint(&legacy);
        assert!(fingerprints.insert(fingerprint.clone()));
        assert!(matches!(
            store(fingerprint.clone()).read(),
            Err(CheckpointError::FingerprintMismatch { .. })
        ));
        if index > 0 {
            assert!(matches!(
                contextual_store(fingerprint).read(),
                Err(CheckpointError::FingerprintMismatch { .. })
            ));
            assert_ne!(
                identity("SELECT 1", &CompiledWatermark::Snapshot, Some(&declared)),
                identity("SELECT 1", &CompiledWatermark::Snapshot, Some(&context)),
            );
        }
        assert_eq!(original.read().unwrap(), Some(saved.clone()));
        assert_eq!(fresh.read().unwrap(), Some(contextual_saved.clone()));
    }
}

/// Scenario: A context declaration contains typos, missing semantics, or unbounded/control data.
/// Guarantees: Invalid declarations are rejected with no reflected principal/schema values.
#[test]
fn source_context_is_strict_and_bounded() {
    for value in [
        json!({"principal":"reader"}),
        json!({"principal":"reader","schema":""}),
        json!({"principal":"reader","schema":"public","revision":"\nprivate"}),
        json!({"principal":"reader","schema":"public","role":"private"}),
        json!({"principal":"private".repeat(200),"schema":"public"}),
    ] {
        let error = serde_json::from_value::<SourceContext>(value).unwrap_err();
        assert!(!error.to_string().contains("private"));
    }
    let value = context();
    let roundtrip: SourceContext =
        serde_json::from_value(serde_json::to_value(&value).unwrap()).unwrap();
    assert_eq!(
        value.checkpoint_fingerprint("legacy"),
        roundtrip.checkpoint_fingerprint("legacy")
    );
}
