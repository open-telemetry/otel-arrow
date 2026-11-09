// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::database::{CheckpointConfig, OutputConfig, PollingConfig, WatermarkConfig};
use serde_json::json;

fn query(sql: &str, mode: &str) -> CompiledQuery {
    let polling = serde_json::from_value::<PollingConfig>(json!({
        "interval":"1m", "timeout":"30s", "fetch_size_rows":1,
        "max_rows_per_poll":2, "max_batch_bytes":100000
    }))
    .unwrap();
    let checkpoint = serde_json::from_value::<CheckpointConfig>(json!({
        "directory":"unused", "on_nack":"rewind", "nack_backoff":"1s", "max_consecutive_failures":3
    }))
    .unwrap();
    let watermark = match mode {
        "snapshot" => json!({"mode":"snapshot"}),
        "composite" => json!({
            "mode":"composite",
            "timestamp":{"column":"ts","bind":"last_ts","initial":"2026-01-01T00:00:00Z","timezone":"UTC"},
            "tie_breaker":{"column":"id","bind":"last_id","initial":0}
        }),
        kind => json!({"mode":"scalar","column":"id","bind":"last_id",
            "initial":{"type":kind,"value":if kind == "string" {json!("!")} else {json!(0)}}}),
    };
    let watermark: WatermarkConfig = serde_json::from_value(watermark).unwrap();
    CompiledQuery::compile(
        sql.into(),
        polling,
        &watermark,
        &checkpoint,
        OutputConfig::default(),
    )
    .unwrap()
}

fn columns() -> Vec<ColumnMetadata> {
    ["id", "ts", "payload"]
        .into_iter()
        .map(|name| ColumnMetadata {
            name: name.into(),
            source_type: "int8".into(),
            nullable: true,
        })
        .collect()
}

fn execution(sql: &str, mode: &str, system: DatabaseSystem) -> String {
    let query = query(sql, mode);
    SourceQuery::compile(&query, system)
        .unwrap()
        .paged_sql(&query, &columns())
        .unwrap()
}

/// Scenario: Each dialect keeps the explicit scalar bind, ORDER BY, and native business expressions.
/// Guarantees: Only parameter spelling and bounded fetch/null checks change, never the source joins or filters.
#[test]
fn explicit_native_queries_share_one_path() {
    for (system, bind, datetime) in [
        (
            DatabaseSystem::SqlServer,
            "@last_id",
            "CAST('2026-11-01T00:00:00' AS datetime2(7))",
        ),
        (
            DatabaseSystem::Oracle,
            ":last_id",
            "TIMESTAMP '2026-11-01 00:00:00'",
        ),
        (
            DatabaseSystem::PostgreSQL,
            ":last_id",
            "'2026-11-01T00:00:00Z'::timestamptz",
        ),
    ] {
        let sql = format!(
            "WITH eligible AS (SELECT id, customer_id FROM events WHERE created_at >= {datetime}) SELECT e.id, COALESCE(c.name, 'unknown') AS payload FROM eligible e LEFT JOIN customers c ON c.id=e.customer_id WHERE e.id > {bind} AND (c.status IN ('ready','retry') OR c.priority > 5) ORDER BY e.id ASC"
        );
        let plan = execution(&sql, "int64", system);
        assert!(plan.starts_with("WITH eligible AS"));
        assert!(plan.contains(datetime));
        assert!(plan.contains("LEFT JOIN customers"));
        assert!(plan.contains("AND (c.status IN ('ready','retry') OR c.priority > 5)"));
        assert!(plan.contains("e.id IS NULL"));
        assert!(!plan.contains("__otap_source"));
        let dialect: &dyn Dialect = match system {
            DatabaseSystem::SqlServer => &MsSqlDialect {},
            DatabaseSystem::Oracle => &OracleDialect {},
            DatabaseSystem::PostgreSQL => &PostgreSqlDialect {},
        };
        assert_eq!(Parser::parse_sql(dialect, &plan).unwrap().len(), 1);
    }
}

/// Scenario: Composite OR and PostgreSQL tuple keysets have differently nested operands.
/// Guarantees: Null probes cover the whole mandatory guard and never bind to only its equality arm.
#[test]
fn composite_guard_spans_preserve_boolean_precedence() {
    for predicate in [
        "e.ts > :last_ts OR (e.ts = :last_ts AND e.id > :last_id)",
        "((e.ts) > (:last_ts)) OR (((e.ts) = (:last_ts)) AND ((e.id) > (:last_id)))",
        "(e.ts, e.id) > (:last_ts, :last_id)",
    ] {
        let sql = format!(
            "SELECT e.ts, e.id FROM events e WHERE ({predicate}) AND e.status = 'ready' ORDER BY e.ts, e.id"
        );
        let result = execution(&sql, "composite", DatabaseSystem::PostgreSQL);
        let ast = Parser::parse_sql(&PostgreSqlDialect {}, &result).unwrap();
        let Statement::Query(q) = &ast[0] else {
            panic!("query")
        };
        let SetExpr::Select(select) = q.body.as_ref() else {
            panic!("select")
        };
        let Expr::BinaryOp {
            left,
            op: Op::And,
            right,
        } = unnest(select.selection.as_ref().unwrap())
        else {
            panic!("AND protects business filter: {result}")
        };
        assert_eq!(right.to_string(), "e.status = 'ready'");
        assert!(
            matches!(unnest(left),Expr::BinaryOp {right,op:Op::Or,..} if matches!(unnest(right),Expr::IsNull(_)))
        );
        assert!(result.ends_with("e.ts NULLS FIRST, e.id NULLS FIRST\nLIMIT 3"));
    }
}

/// Scenario: Keyset-like expressions can skip timestamp ties or let rows bypass the checkpoint.
/// Guarantees: Only exactly one mandatory guard is accepted; business OR is allowed only beneath AND.
#[test]
fn rejects_bypass_inclusive_wrong_or_missing_guards() {
    for predicate in [
        "e.ts > :last_ts AND e.id > :last_id",
        "e.ts >= :last_ts OR (e.ts = :last_ts AND e.id > :last_id)",
        "e.ts > :last_ts OR (e.ts = :last_ts AND e.id >= :last_id)",
        "(e.ts > :last_ts OR (e.ts = :last_ts AND e.id > :last_id)) OR e.status = 'ready'",
        "NOT (e.ts > :last_ts OR (e.ts = :last_ts AND e.id > :last_id))",
        "(e.ts, e.id) >= (:last_ts, :last_id)",
        "EXISTS (SELECT 1 FROM other WHERE e.ts > :last_ts OR (e.ts = :last_ts AND e.id > :last_id))",
    ] {
        let sql = format!("SELECT e.ts,e.id FROM events e WHERE {predicate} ORDER BY e.ts,e.id");
        assert_eq!(
            SourceQuery::compile(&query(&sql, "composite"), DatabaseSystem::PostgreSQL)
                .unwrap_err(),
            SourceQueryError::Keyset,
            "{sql}"
        );
    }
    let duplicated = "SELECT id FROM events WHERE id > :last_id AND id > :last_id ORDER BY id";
    assert_eq!(
        SourceQuery::compile(&query(duplicated, "int64"), DatabaseSystem::PostgreSQL).unwrap_err(),
        SourceQueryError::Keyset
    );
}

/// Scenario: A cursor is selected under an alias, or a predicate mistakenly references a different relation.
/// Guarantees: The guard and order are tied to the selected cursor, not merely a matching suffix.
#[test]
fn cursor_projection_and_order_match_the_guard() {
    let valid = "SELECT e.source_id AS id FROM events e WHERE e.source_id > :last_id ORDER BY id";
    assert!(execution(valid, "int64", DatabaseSystem::PostgreSQL).contains("e.source_id > $1"));
    for sql in [
        "SELECT e.id FROM events e JOIN other o ON e.id=o.id WHERE o.id > :last_id ORDER BY e.id",
        "SELECT e.source_id AS id FROM events e WHERE e.id > :last_id ORDER BY id",
        "SELECT e.id+1 AS id FROM events e WHERE e.id > :last_id ORDER BY id",
        "SELECT e.id FROM events e WHERE e.id > :last_id ORDER BY e.id DESC",
        "SELECT e.id FROM events e WHERE e.id > :last_id ORDER BY e.id NULLS LAST",
        "SELECT e.id FROM events e WHERE e.id > :last_id",
        "SELECT e.id FROM events e ORDER BY e.id",
    ] {
        assert!(
            SourceQuery::compile(&query(sql, "int64"), DatabaseSystem::PostgreSQL).is_err(),
            "{sql}"
        );
    }
}

/// Scenario: Authored SQL uses native or named cursor bindings, including misleading quoted text.
/// Guarantees: Real parameters map once to configured roles; undeclared/mixed binds and assignments fail.
#[test]
fn parameter_binding_is_token_aware() {
    for bind in [":last_id", "$1"] {
        let sql = format!(
            "SELECT id, ':last_id $2' AS payload FROM events WHERE id > {bind} ORDER BY id"
        );
        let plan = execution(&sql, "int64", DatabaseSystem::PostgreSQL);
        assert!(plan.contains("id > $1"));
        assert!(plan.contains("':last_id $2'"));
    }
    let sql = "SELECT id, N'@last_id; -- can''t' AS payload FROM events WHERE id > @last_id ORDER BY id -- end";
    let plan = execution(sql, "int64", DatabaseSystem::SqlServer);
    assert!(plan.starts_with("SELECT TOP (@P2)"));
    assert!(plan.contains("N'@last_id; -- can''t'"));
    assert!(plan.contains("id > @P1"));
    for sql in [
        "SELECT id FROM events WHERE id > :undeclared ORDER BY id",
        "SELECT id FROM events WHERE id > $2 ORDER BY id",
        "SELECT id FROM events WHERE id > $1 AND id <> :last_id ORDER BY id",
        "SELECT id FROM events WHERE id > : last_id ORDER BY id",
    ] {
        assert_eq!(
            SourceQuery::compile(&query(sql, "int64"), DatabaseSystem::PostgreSQL).unwrap_err(),
            SourceQueryError::Parameters
        );
    }
    assert!(
        SourceQuery::compile(
            &query(
                "SELECT @last_id = id FROM events WHERE id > @last_id ORDER BY id",
                "int64"
            ),
            DatabaseSystem::SqlServer
        )
        .is_err()
    );
}

/// Scenario: Oracle string cursors use the documented BINARY comparison and order.
/// Guarantees: The existing spelling compiles while linguistic or missing collations fail.
#[test]
fn oracle_binary_string_keys_remain_explicit() {
    let sql = "SELECT id FROM events WHERE id COLLATE BINARY > :last_id COLLATE BINARY ORDER BY id COLLATE BINARY ASC";
    let plan = execution(sql, "string", DatabaseSystem::Oracle);
    assert!(plan.contains("id COLLATE BINARY > :last_id COLLATE BINARY"));
    assert!(plan.ends_with("id COLLATE BINARY ASC NULLS FIRST"));
    for modified in [
        sql.replace(" COLLATE BINARY", ""),
        sql.replacen("COLLATE BINARY", "COLLATE BINARY_CI", 1),
    ] {
        assert!(SourceQuery::compile(&query(&modified, "string"), DatabaseSystem::Oracle).is_err());
    }
}

/// Scenario: Broad snapshot statements include grouping, windows, sets, wildcards, and native limits.
/// Guarantees: Snapshots retain their native SQL and do not acquire a checkpoint predicate or parameters.
#[test]
fn snapshots_remain_broad_and_parameter_free() {
    for sql in [
        "SELECT *, ROW_NUMBER() OVER (ORDER BY id) AS n FROM events",
        "SELECT status, COUNT(*) AS n FROM events GROUP BY status HAVING COUNT(*) > 0",
        "SELECT id FROM events UNION ALL SELECT id FROM archived",
        "SELECT id FROM events ORDER BY id DESC LIMIT 10 OFFSET 1",
        "WITH e AS (SELECT * FROM events) SELECT id FROM e",
    ] {
        assert_eq!(execution(sql, "snapshot", DatabaseSystem::PostgreSQL), sql);
    }
    assert_eq!(
        SourceQuery::compile(
            &query("SELECT id FROM events WHERE id > :last_id", "snapshot"),
            DatabaseSystem::Oracle
        )
        .unwrap_err(),
        SourceQueryError::Parameters
    );
}

/// Scenario: Mutation, locks, multiple statements, or conflicting top-level paging are requested.
/// Guarantees: Relaxed business SQL does not bypass the common read-only and bounded-paging contracts.
#[test]
fn rejects_mutation_locking_and_pagination() {
    for sql in [
        "DELETE FROM events",
        "SELECT id INTO copied FROM events",
        "SELECT id FROM events; DELETE FROM events",
        "WITH d AS (DELETE FROM events RETURNING id) SELECT id FROM d",
        "SELECT id FROM (SELECT id FROM events FOR UPDATE) e",
    ] {
        assert!(SourceQuery::compile(&query(sql, "snapshot"), DatabaseSystem::PostgreSQL).is_err());
    }
    for suffix in [" LIMIT 2", " OFFSET 1", " FETCH FIRST 2 ROWS ONLY"] {
        let sql = format!("SELECT id FROM events WHERE id > :last_id ORDER BY id{suffix}");
        assert_eq!(
            SourceQuery::compile(&query(&sql, "int64"), DatabaseSystem::PostgreSQL).unwrap_err(),
            SourceQueryError::Pagination
        );
    }
    for hint in ["WITH (NOLOCK)", "WITH (UPDLOCK)", "(READPAST)"] {
        let sql = format!("SELECT id FROM events {hint}");
        assert_eq!(
            SourceQuery::compile(&query(&sql, "snapshot"), DatabaseSystem::SqlServer).unwrap_err(),
            SourceQueryError::Locking
        );
    }
}

/// Scenario: Result metadata is missing, duplicated, or does not include the configured cursor.
/// Guarantees: Native schemas cannot silently change which result field advances progress.
#[test]
fn validates_live_output_names() {
    let query = query(
        "SELECT id FROM events WHERE id > :last_id ORDER BY id",
        "int64",
    );
    let source = SourceQuery::compile(&query, DatabaseSystem::Oracle).unwrap();
    for columns in [
        vec![],
        vec![columns()[0].clone(), columns()[0].clone()],
        vec![columns()[1].clone()],
    ] {
        assert_eq!(
            source.paged_sql(&query, &columns),
            Err(SourceQueryError::Columns)
        );
    }
}

/// Scenario: Fingerprints use the original dialect-specific SQL representation.
/// Guarantees: SQL Server canonicalization, Oracle trimming, and PostgreSQL raw-text identity remain compatible.
#[test]
fn fingerprint_spelling_is_compatible() {
    let sql = " \nSELECT [id], N'can''t' AS t FROM dbo . events WHERE id != 7; \n";
    let plan = SourceQuery::compile(&query(sql, "snapshot"), DatabaseSystem::SqlServer).unwrap();
    assert_eq!(
        plan.fingerprint_sql(),
        "SELECT [id] , N'can''t' AS t FROM dbo . events WHERE id <> 7"
    );
    let oracle = " \nSELECT id FROM events; \n";
    assert_eq!(
        SourceQuery::compile(&query(oracle, "snapshot"), DatabaseSystem::Oracle)
            .unwrap()
            .fingerprint_sql(),
        "SELECT id FROM events"
    );
    assert_eq!(
        SourceQuery::compile(&query(oracle, "snapshot"), DatabaseSystem::PostgreSQL)
            .unwrap()
            .fingerprint_sql(),
        oracle
    );
}

/// Scenario: Multibyte literals and comments precede explicit predicates and ordering.
/// Guarantees: Character spans are mapped to bytes without corrupting native query text.
#[test]
fn edits_preserve_unicode_literals_and_comments() {
    let sql = "WITH e AS (SELECT id, '\u{1f30d}' AS payload FROM events) SELECT id,payload FROM e WHERE id > :last_id AND payload <> '--;:' ORDER BY id; -- tail";
    let plan = execution(sql, "int64", DatabaseSystem::PostgreSQL);
    assert!(plan.contains("'\u{1f30d}'"));
    assert!(plan.contains("AND payload <> '--;:'"));
    assert!(plan.ends_with("-- tail\nLIMIT 3"));
    assert_eq!(
        Parser::parse_sql(&PostgreSqlDialect {}, &plan)
            .unwrap()
            .len(),
        1
    );
}
