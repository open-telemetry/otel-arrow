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
/// Guarantees: Only parameter spelling and fetch bounds change; no null predicates or order modifiers are added.
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
        assert!(!plan.contains("IS NULL"));
        assert!(!plan.contains("NULLS"));
        assert!(!plan.contains("__otap_source"));
        let expected = match system {
            DatabaseSystem::SqlServer => {
                sql.replace("@last_id", "@P1")
                    .replacen("SELECT e.id", "SELECT TOP (@P2) e.id", 1)
            }
            DatabaseSystem::Oracle => sql.clone(),
            DatabaseSystem::PostgreSQL => format!("{}\nLIMIT 3", sql.replace(":last_id", "$1")),
        };
        assert_eq!(plan, expected);
        let dialect: &dyn Dialect = match system {
            DatabaseSystem::SqlServer => &MsSqlDialect {},
            DatabaseSystem::Oracle => &OracleDialect {},
            DatabaseSystem::PostgreSQL => &PostgreSqlDialect {},
        };
        assert_eq!(Parser::parse_sql(dialect, &plan).unwrap().len(), 1);
    }
}

/// Scenario: Composite OR and PostgreSQL tuple keysets have differently nested operands.
/// Guarantees: Authored Boolean structure and cursor order remain unchanged through parameter binding.
#[test]
fn composite_guards_preserve_authored_boolean_precedence() {
    for predicate in [
        "e.ts > :last_ts OR (e.ts = :last_ts AND e.id > :last_id)",
        "((e.ts) > (:last_ts)) OR (((e.ts) = (:last_ts)) AND ((e.id) > (:last_id)))",
        "(e.ts, e.id) > (:last_ts, :last_id)",
    ] {
        let sql = format!(
            "SELECT e.ts, e.id FROM events e WHERE ({predicate}) AND e.status = 'ready' ORDER BY e.ts, e.id"
        );
        let result = execution(&sql, "composite", DatabaseSystem::PostgreSQL);
        let bound = sql.replace(":last_ts", "$1").replace(":last_id", "$2");
        assert_eq!(result, format!("{bound}\nLIMIT 3"));
        assert!(Parser::parse_sql(&PostgreSqlDialect {}, &result).is_ok());
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
        "SELECT e.id FROM events e WHERE e.id > :last_id ORDER BY e.id USING >",
        "SELECT e.id FROM events e WHERE e.id > :last_id ORDER BY e.id USING <",
        "SELECT e.id FROM events e WHERE e.id > :last_id",
        "SELECT e.id FROM events e ORDER BY e.id",
    ] {
        assert!(
            SourceQuery::compile(&query(sql, "int64"), DatabaseSystem::PostgreSQL).is_err(),
            "{sql}"
        );
    }
}

/// Scenario: An ORDER BY name differs from the cursor output under native identifier case rules.
/// Guarantees: An unrelated same-spelled source column cannot be mistaken for the cursor result alias.
#[test]
fn order_aliases_follow_native_identifier_case() {
    for (system, bind, wrong_order) in [
        (DatabaseSystem::PostgreSQL, ":last_id", "\"ID\""),
        (DatabaseSystem::Oracle, ":last_id", "\"id\""),
        (DatabaseSystem::SqlServer, "@last_id", "ID"),
    ] {
        let sql = format!("SELECT e.id FROM events e WHERE e.id > {bind} ORDER BY {wrong_order}");
        assert_eq!(
            SourceQuery::compile(&query(&sql, "int64"), system).unwrap_err(),
            SourceQueryError::Ordering,
        );
    }
    for (system, bind, alias) in [
        (DatabaseSystem::PostgreSQL, ":last_id", "\"ID\""),
        (DatabaseSystem::Oracle, ":last_id", "\"id\""),
        (DatabaseSystem::SqlServer, "@last_id", "[id]"),
    ] {
        let sql = format!(
            "SELECT e.source_key AS {alias} FROM events e WHERE e.source_key > {bind} ORDER BY {alias}"
        );
        assert!(SourceQuery::compile(&query(&sql, "int64"), system).is_ok());
    }
}

/// Scenario: Another result alias shadows the unqualified input column used as the cursor.
/// Guarantees: ORDER BY cannot sort by the shadowing result value and checkpoint past unread keys.
#[test]
fn order_aliases_cannot_shadow_the_source_cursor() {
    for (system, bind) in [
        (DatabaseSystem::PostgreSQL, ":last_id"),
        (DatabaseSystem::Oracle, ":last_id"),
        (DatabaseSystem::SqlServer, "@last_id"),
    ] {
        let sql = format!(
            "SELECT source_id AS id, other_id AS source_id FROM events WHERE source_id > {bind} ORDER BY source_id"
        );
        assert_eq!(
            SourceQuery::compile(&query(&sql, "int64"), system).unwrap_err(),
            SourceQueryError::Ordering,
        );
        for sql in [
            format!(
                "SELECT source_id AS id, other_id FROM events WHERE source_id > {bind} ORDER BY source_id"
            ),
            format!(
                "SELECT e.source_id AS id, e.other_id AS source_id FROM events e WHERE e.source_id > {bind} ORDER BY e.source_id"
            ),
        ] {
            assert!(SourceQuery::compile(&query(&sql, "int64"), system).is_ok());
        }
    }
    let implicit_alias = "SELECT source_id AS id, source_id() FROM events WHERE source_id > :last_id ORDER BY source_id";
    assert_eq!(
        SourceQuery::compile(&query(implicit_alias, "int64"), DatabaseSystem::PostgreSQL)
            .unwrap_err(),
        SourceQueryError::Ordering,
    );
    assert!(
        SourceQuery::compile(
            &query(
                &implicit_alias.replace("ORDER BY source_id", "ORDER BY id"),
                "int64"
            ),
            DatabaseSystem::PostgreSQL,
        )
        .is_ok()
    );
}

/// Scenario: A valid ascending ORDER BY wraps its cursor in one or more pairs of parentheses.
/// Guarantees: Parenthesized ordering is preserved exactly and the executed SQL remains valid.
#[test]
fn parenthesized_ordering_is_not_rewritten() {
    for (system, dialect) in [
        (
            DatabaseSystem::PostgreSQL,
            &PostgreSqlDialect {} as &dyn Dialect,
        ),
        (DatabaseSystem::Oracle, &OracleDialect {} as &dyn Dialect),
    ] {
        for order in ["(id)", "((id)) ASC", "(((id)))"] {
            let sql = format!("SELECT id FROM events WHERE (id > :last_id) ORDER BY {order}");
            let compiled = execution(&sql, "int64", system);
            assert!(
                Parser::parse_sql(dialect, &compiled).is_ok(),
                "invalid generated ordering: {compiled}"
            );
            assert!(compiled.contains(&format!("ORDER BY {order}")));
            assert!(!compiled.contains("NULLS"));
        }
    }
    let sql = "SELECT id FROM events WHERE id COLLATE BINARY > :last_id COLLATE BINARY ORDER BY ((id COLLATE BINARY)) ASC";
    let compiled = execution(sql, "string", DatabaseSystem::Oracle);
    assert!(Parser::parse_sql(&OracleDialect {}, &compiled).is_ok());
    assert_eq!(compiled, sql);
}

/// Scenario: The author specifies native null filters and default, NULLS FIRST, or NULLS LAST ordering.
/// Guarantees: PostgreSQL and Oracle retain the user's WHERE and ORDER BY null semantics without metadata NOT NULL requirements.
#[test]
fn authored_null_semantics_are_preserved() {
    for system in [DatabaseSystem::PostgreSQL, DatabaseSystem::Oracle] {
        for placement in ["", " NULLS FIRST", " NULLS LAST"] {
            let sql = format!(
                "SELECT e.id, e.payload FROM events e WHERE e.id > :last_id AND (e.payload IS NULL OR e.status = 'ready') ORDER BY e.id ASC{placement}"
            );
            let compiled = execution(&sql, "int64", system);
            let expected = if system == DatabaseSystem::PostgreSQL {
                format!("{}\nLIMIT 3", sql.replace(":last_id", "$1"))
            } else {
                sql
            };
            assert_eq!(compiled, expected);
            assert_eq!(compiled.matches("IS NULL").count(), 1);
            assert!(!compiled.contains("e.id IS NULL"));
        }
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
    assert_eq!(plan, sql);
    for placement in [" NULLS FIRST", " NULLS LAST"] {
        let authored = format!("{sql}{placement}");
        assert_eq!(
            execution(&authored, "string", DatabaseSystem::Oracle),
            authored
        );
    }
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

/// Scenario: A lone carriage return ends a native line comment but is swallowed by the parser's SQL Server/Oracle lexer.
/// Guarantees: Text hidden by that lexer cannot conceal another executable statement from validation.
#[test]
fn carriage_return_comments_cannot_hide_statements() {
    for system in [DatabaseSystem::SqlServer, DatabaseSystem::Oracle] {
        let hidden = "SELECT id FROM events -- comment\r; DELETE FROM events";
        assert!(SourceQuery::compile(&query(hidden, "snapshot"), system).is_err());
        for ending in ["\n", "\r\n"] {
            let sql = format!("SELECT id FROM events -- comment{ending}WHERE id > 0");
            let source = SourceQuery::compile(&query(&sql, "snapshot"), system).unwrap();
            assert_eq!(source.sql(), sql);
        }
        for sql in [
            "SELECT 'a\rb' AS payload FROM events",
            "SELECT id FROM events /* comment\rstill a comment */",
        ] {
            assert_eq!(
                SourceQuery::compile(&query(sql, "snapshot"), system)
                    .unwrap()
                    .sql(),
                sql
            );
        }
    }
    let postgres = "SELECT id FROM events -- comment\rWHERE id > 0";
    assert_eq!(
        SourceQuery::compile(&query(postgres, "snapshot"), DatabaseSystem::PostgreSQL)
            .unwrap()
            .sql(),
        postgres,
    );
    assert!(
        SourceQuery::compile(
            &query(
                "SELECT id FROM events -- comment\r; DELETE FROM events",
                "snapshot"
            ),
            DatabaseSystem::PostgreSQL,
        )
        .is_err()
    );
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
