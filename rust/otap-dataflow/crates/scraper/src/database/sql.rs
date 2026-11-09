// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! One source-query contract for native SQL and receiver-owned keyset paging.

use super::{
    ColumnMetadata, CompiledQuery, CompiledWatermark, DatabaseSystem, MAX_QUERY_BYTES, ScalarValue,
};
use sqlparser::{
    ast::{
        Expr, FunctionArg, FunctionArgExpr, OrderByKind, OrderBySort, Query, Select, SetExpr, Statement,
        TableFactor, Value, ValueWithSpan, Visit, Visitor,
    },
    dialect::{Dialect, MsSqlDialect, OracleDialect, PostgreSqlDialect},
    parser::Parser,
    tokenizer::{Token, TokenWithSpan, Tokenizer, Whitespace},
};
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
    ops::ControlFlow,
};

const MAX_TOKENS: usize = 4096;
const MAX_NESTING: usize = 128;

/// A read-only native source and the parts needed to page its final result.
///
/// SQL is retained verbatim rather than rendered from the AST: rendering can
/// change native literal escaping, optimizer comments, or dialect extensions.
#[derive(Clone)]
pub struct SourceQuery {
    system: DatabaseSystem,
    source: String,
    fingerprint: String,
    prefix: String,
    body: String,
}

impl fmt::Debug for SourceQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceQuery")
            .field("system", &self.system)
            .field("sql", &"<redacted>")
            .finish()
    }
}

impl SourceQuery {
    /// Validates one native row-producing statement, without a business-SQL allowlist.
    pub fn compile(
        query: &CompiledQuery,
        system: DatabaseSystem,
    ) -> Result<Self, SourceQueryError> {
        let dialect: &dyn Dialect = match system {
            DatabaseSystem::Oracle => &OracleDialect {},
            DatabaseSystem::SqlServer => &MsSqlDialect {},
            DatabaseSystem::PostgreSQL => &PostgreSqlDialect {},
        };
        let sql = query.sql();
        if sql.is_empty() || sql.len() > MAX_QUERY_BYTES {
            return Err(SourceQueryError::Size);
        }
        let tokens = Tokenizer::new(dialect, sql)
            .tokenize_with_location()
            .map_err(|_| SourceQueryError::Syntax)?;
        let mut significant: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        if significant.len() > MAX_TOKENS {
            return Err(SourceQueryError::Complexity);
        }
        let terminator = if significant
            .last()
            .is_some_and(|t| t.token == Token::SemiColon)
        {
            significant.pop()
        } else {
            None
        };
        if significant.iter().any(|t| t.token == Token::SemiColon) {
            return Err(SourceQueryError::Statement);
        }
        if significant.iter().any(|token| {
            matches!(&token.token, Token::Word(word)
                if word.quote_style.is_none()
                    && word.value.starts_with('@')
                    && !word.value.starts_with("@@"))
        }) {
            return Err(SourceQueryError::Parameters);
        }
        if system == DatabaseSystem::SqlServer
            && significant.iter().any(|t| t.token == Token::DoubleEq)
        {
            return Err(SourceQueryError::Syntax);
        }
        let mut parser = Parser::new(dialect)
            .with_recursion_limit(MAX_NESTING)
            .with_tokens(tokens.iter().map(|t| t.token.clone()).collect());
        let statements = parser
            .parse_statements()
            .map_err(|_| SourceQueryError::Syntax)?;
        if !matches!(parser.peek_token_ref().token, Token::EOF) {
            return Err(SourceQueryError::Statement);
        }
        let [Statement::Query(statement)] = statements.as_slice() else {
            return Err(SourceQueryError::Statement);
        };
        if let ControlFlow::Break(error) = statements.visit(&mut ReadOnly {
            system,
            expression_depth: 0,
        }) {
            return Err(error);
        }
        let snapshot = matches!(query.watermark(), CompiledWatermark::Snapshot);
        if !snapshot {
            if statement.limit_clause.is_some()
                || statement.fetch.is_some()
                || matches!(statement.body.as_ref(), SetExpr::Select(select) if select.top.is_some())
            {
                return Err(SourceQueryError::Pagination);
            }
            validate_order(statement, query.watermark())?;
        }

        let offsets = offsets(sql);
        let start = |token: &TokenWithSpan| {
            offsets
                .get(&(token.span.start.line, token.span.start.column))
                .copied()
                .ok_or(SourceQueryError::Syntax)
        };
        let fingerprint = tokens
            .iter()
            .filter(|token| {
                !matches!(
                    token.token,
                    Token::SemiColon
                        | Token::Whitespace(
                            Whitespace::Space | Whitespace::Tab | Whitespace::Newline
                        )
                )
            })
            .map(|token| {
                let begin = start(token)?;
                let end = offsets
                    .get(&(token.span.end.line, token.span.end.column))
                    .copied()
                    .ok_or(SourceQueryError::Syntax)?;
                sql.get(begin..end)
                    .map(str::to_owned)
                    .ok_or(SourceQueryError::Syntax)
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(" ");
        let mut source = sql.to_owned();
        if let Some(terminator) = terminator {
            _ = source.remove(start(terminator)?);
        }
        let mut depth = 0usize;
        let mut body_start = None;
        let mut order_start = None;
        for (index, token) in significant.iter().enumerate() {
            if depth == 0 {
                if body_start.is_none()
                    && ["SELECT", "VALUES", "TABLE"]
                        .iter()
                        .any(|word| is_word(&token.token, word))
                {
                    body_start = Some(start(token)?);
                }
                if is_word(&token.token, "ORDER")
                    && significant
                        .get(index + 1)
                        .is_some_and(|next| is_word(&next.token, "BY"))
                {
                    order_start = Some(start(token)?);
                }
            }
            match token.token {
                Token::LParen => depth += 1,
                Token::RParen => depth = depth.checked_sub(1).ok_or(SourceQueryError::Syntax)?,
                _ => {}
            }
        }
        let body_start = if statement.with.is_some() {
            body_start.ok_or(SourceQueryError::Composition)?
        } else {
            0
        };
        let body_end = if !snapshot && statement.order_by.is_some() {
            order_start.ok_or(SourceQueryError::Composition)?
        } else {
            source.len()
        };
        let prefix = source[..body_start].to_owned();
        let body = source[body_start..body_end].to_owned();
        Ok(Self {
            system,
            source: source.trim().to_owned(),
            fingerprint,
            prefix,
            body,
        })
    }

    /// Returns the native source SQL, with at most its final statement terminator removed.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.source
    }

    /// Returns stable token text for identity, retaining literals and optimizer comments.
    #[must_use]
    pub fn fingerprint_sql(&self) -> &str {
        &self.fingerprint
    }

    /// Composes paging around the complete source result using resolved output names.
    ///
    /// Snapshots execute their native SQL unchanged and use bounded driver fetches.
    /// Keysets include null keys so SQL three-valued logic cannot silently lose them.
    /// Drivers must inspect one successor cursor before returning a partial page.
    pub fn paged_sql(
        &self,
        query: &CompiledQuery,
        columns: &[ColumnMetadata],
    ) -> Result<String, SourceQueryError> {
        let mut names = HashSet::new();
        if columns.is_empty()
            || columns.iter().any(|column| {
                column.name.trim().is_empty() || !names.insert(column.name.to_ascii_lowercase())
            })
        {
            return Err(SourceQueryError::Columns);
        }
        if matches!(query.watermark(), CompiledWatermark::Snapshot) {
            return Ok(self.source.clone());
        }
        let alias = quote(self.system, "__otap_source");
        let column = |name: &str| {
            columns
                .iter()
                .find(|column| column.name.eq_ignore_ascii_case(name))
                .map(|column| format!("{alias}.{}", quote(self.system, &column.name)))
                .ok_or(SourceQueryError::Columns)
        };
        let bind = |index: usize, name: &str| match self.system {
            DatabaseSystem::Oracle => format!(":{name}"),
            DatabaseSystem::SqlServer => format!("@P{index}"),
            DatabaseSystem::PostgreSQL => format!("${index}"),
        };
        let (nulls, predicate, keys, parameters) = match query.watermark() {
            CompiledWatermark::Snapshot => unreachable!("snapshot returned above"),
            CompiledWatermark::Scalar(cursor) => {
                let key = column(&cursor.column)?;
                let parameter = bind(1, &cursor.bind);
                let (ordered_key, parameter) = if self.system == DatabaseSystem::Oracle
                    && matches!(cursor.initial, ScalarValue::String(_))
                {
                    (
                        format!("{key} COLLATE BINARY"),
                        format!("{parameter} COLLATE BINARY"),
                    )
                } else {
                    (key.clone(), parameter)
                };
                (
                    format!("{key} IS NULL"),
                    format!("{ordered_key} > {parameter}"),
                    vec![ordered_key],
                    1,
                )
            }
            CompiledWatermark::Composite(cursor) => {
                let timestamp = column(&cursor.timestamp_column)?;
                let integer = column(&cursor.tie_breaker_column)?;
                let timestamp_bind = bind(1, &cursor.timestamp_bind);
                let integer_bind = bind(2, &cursor.tie_breaker_bind);
                (
                    format!("{timestamp} IS NULL OR {integer} IS NULL"),
                    format!(
                        "{timestamp} > {timestamp_bind} OR ({timestamp} = {timestamp_bind} AND {integer} > {integer_bind})"
                    ),
                    vec![timestamp, integer],
                    2,
                )
            }
        };
        let null_order = if self.system == DatabaseSystem::SqlServer {
            ""
        } else {
            " NULLS FIRST"
        };
        let order = keys
            .iter()
            .map(|key| format!("{key} ASC{null_order}"))
            .collect::<Vec<_>>()
            .join(", ");
        let top = if self.system == DatabaseSystem::SqlServer {
            format!("TOP (@P{}) ", parameters + 1)
        } else {
            String::new()
        };
        let limit = if self.system == DatabaseSystem::PostgreSQL {
            format!(" LIMIT {}", query.max_rows() + 1)
        } else {
            String::new()
        };
        // Hoist WITH ahead of the generated SELECT, particularly for SQL Server.
        // Newlines prevent a trailing line comment from swallowing generated SQL.
        Ok(format!(
            "{}\nSELECT {top}* FROM (\n{}\n) {alias} WHERE ({nulls}) OR ({predicate}) ORDER BY {order}{limit}",
            self.prefix, self.body,
        ))
    }
}

fn quote(system: DatabaseSystem, name: &str) -> String {
    if system == DatabaseSystem::SqlServer {
        format!("[{}]", name.replace(']', "]]"))
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

fn is_word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn offsets(sql: &str) -> BTreeMap<(u64, u64), usize> {
    let mut result = BTreeMap::new();
    let (mut line, mut column) = (1, 1);
    for (offset, ch) in sql.char_indices() {
        let _ = result.insert((line, column), offset);
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    let _ = result.insert((line, column), sql.len());
    result
}

fn validate_order(query: &Query, watermark: &CompiledWatermark) -> Result<(), SourceQueryError> {
    let Some(order) = &query.order_by else {
        return Ok(());
    };
    let expected = match watermark {
        CompiledWatermark::Snapshot => return Ok(()),
        CompiledWatermark::Scalar(cursor) => vec![cursor.column.as_str()],
        CompiledWatermark::Composite(cursor) => vec![
            cursor.timestamp_column.as_str(),
            cursor.tie_breaker_column.as_str(),
        ],
    };
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return Err(SourceQueryError::Ordering);
    };
    if order.interpolate.is_some() || expressions.len() != expected.len() {
        return Err(SourceQueryError::Ordering);
    }
    for (expression, expected) in expressions.iter().zip(expected) {
        let name = match &expression.expr {
            Expr::Identifier(id) => Some(id),
            Expr::CompoundIdentifier(ids) => ids.last(),
            _ => None,
        };
        if !name.is_some_and(|id| id.value.eq_ignore_ascii_case(expected))
            || !matches!(expression.options.sort, None | Some(OrderBySort::Asc))
            || expression.with_fill.is_some()
        {
            return Err(SourceQueryError::Ordering);
        }
    }
    Ok(())
}

struct ReadOnly {
    system: DatabaseSystem,
    expression_depth: usize,
}

fn locking_hint(expression: &Expr) -> bool {
    matches!(expression, Expr::Identifier(id) if [
        "NOLOCK", "READUNCOMMITTED", "UPDLOCK", "XLOCK", "HOLDLOCK",
        "TABLOCK", "TABLOCKX", "PAGLOCK", "ROWLOCK", "READPAST",
        "READCOMMITTEDLOCK", "REPEATABLEREAD", "SERIALIZABLE",
    ].iter().any(|name| id.value.eq_ignore_ascii_case(name)))
}

impl Visitor for ReadOnly {
    type Break = SourceQueryError;

    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<Self::Break> {
        if !matches!(statement, Statement::Query(_)) {
            return ControlFlow::Break(SourceQueryError::Statement);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<Self::Break> {
        if !query.locks.is_empty() {
            return ControlFlow::Break(SourceQueryError::Locking);
        }
        if query.for_clause.is_some() || query.settings.is_some() || query.format_clause.is_some() {
            return ControlFlow::Break(SourceQueryError::Composition);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_select(&mut self, select: &Select) -> ControlFlow<Self::Break> {
        if select.into.is_some() {
            return ControlFlow::Break(SourceQueryError::Statement);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_value(&mut self, value: &ValueWithSpan) -> ControlFlow<Self::Break> {
        if matches!(&value.value, Value::Placeholder(name) if !name.starts_with("@@")) {
            return ControlFlow::Break(SourceQueryError::Parameters);
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expression: &Expr) -> ControlFlow<Self::Break> {
        self.expression_depth += 1;
        if self.expression_depth > MAX_NESTING {
            return ControlFlow::Break(SourceQueryError::Complexity);
        }
        if matches!(expression, Expr::Identifier(id) if id.quote_style.is_none() && id.value.starts_with('@') && !id.value.starts_with("@@"))
        {
            return ControlFlow::Break(SourceQueryError::Parameters);
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<Self::Break> {
        self.expression_depth -= 1;
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<Self::Break> {
        if let TableFactor::Table {
            with_hints, args, ..
        } = table
            && (with_hints.iter().any(locking_hint)
                || (self.system == DatabaseSystem::SqlServer && args.as_ref().is_some_and(|args| {
                    args.args.iter().any(|argument| matches!(argument,
                        FunctionArg::Unnamed(FunctionArgExpr::Expr(expression)) if locking_hint(expression)))
                })))
        {
            return ControlFlow::Break(SourceQueryError::Locking);
        }
        ControlFlow::Continue(())
    }
}

/// Redacted, actionable failures of the shared source-query contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SourceQueryError {
    /// Query text is absent or exceeds the shared bounded-input limit.
    #[error("query.statement must be nonempty and at most 16 KiB")]
    Size,
    /// The dialect parser cannot represent the statement.
    #[error("query.statement could not be parsed in the selected database dialect")]
    Syntax,
    /// Expression nesting exceeds the bounded parser/visitor budget.
    #[error("query.statement must use at most 4096 SQL tokens and 128 nested expressions")]
    Complexity,
    /// A mutation, command, or additional executable statement is present.
    #[error(
        "query.statement must contain one read-only tabular query, without writes, INTO, or additional statements"
    )]
    Statement,
    /// User cursor predicates belong to the previous query contract.
    #[error(
        "query.statement must be source-only SQL without parameters; remove handwritten cursor predicates and keep the watermark configuration (checkpoint migration may be required)"
    )]
    Parameters,
    /// A user limit would conflict with receiver-owned incremental pagination.
    #[error(
        "incremental query.statement must not contain a top-level TOP, LIMIT, OFFSET, or FETCH; the receiver supplies paging"
    )]
    Pagination,
    /// The optional final order differs from receiver-owned cursor ordering.
    #[error("omit the final ORDER BY or order only by the configured cursor columns ascending")]
    Ordering,
    /// Explicit locks or isolation overrides violate read-only collection.
    #[error("query.statement must not request row locks or override collection isolation")]
    Locking,
    /// A native construct cannot be composed without changing semantics.
    #[error("query.statement contains a native clause that cannot be composed as a tabular source")]
    Composition,
    /// Native result metadata does not identify an unambiguous output contract.
    #[error(
        "query results must have unique nonempty column names and include the configured cursor columns"
    )]
    Columns,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{CheckpointConfig, OutputConfig, PollingConfig, WatermarkConfig};
    use serde_json::json;

    fn query(sql: &str, snapshot: bool) -> CompiledQuery {
        let polling: PollingConfig = serde_json::from_value(json!({
            "interval": "1m", "timeout": "30s", "fetch_size_rows": 1,
            "max_rows_per_poll": 2, "max_batch_bytes": 100000
        }))
        .unwrap();
        let checkpoint: CheckpointConfig = serde_json::from_value(json!({
            "directory": "unused", "on_nack": "rewind",
            "nack_backoff": "1s", "max_consecutive_failures": 3
        }))
        .unwrap();
        let watermark = if snapshot {
            WatermarkConfig::Snapshot {}
        } else {
            WatermarkConfig::Scalar {
                column: "id".into(),
                bind: "last_id".into(),
                initial: ScalarValue::Int64(0),
            }
        };
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
        vec![ColumnMetadata {
            name: "id".into(),
            source_type: "int8".into(),
            nullable: true,
        }]
    }

    /// Scenario: Native source queries use joins, CTEs, expressions, and static datetime filters.
    /// Guarantees: Every receiver shares the broad read-only policy and owns its outer cursor.
    #[test]
    fn native_sources_share_one_contract() {
        for (system, timestamp) in [
            (
                DatabaseSystem::PostgreSQL,
                "'2026-11-01T00:00:00Z'::timestamptz",
            ),
            (DatabaseSystem::Oracle, "TIMESTAMP '2026-11-01 00:00:00'"),
            (
                DatabaseSystem::SqlServer,
                "CAST('2026-11-01T00:00:00' AS datetime2(7))",
            ),
        ] {
            let sql = format!(
                "WITH events AS (SELECT id, customer_id FROM events_view WHERE created_at >= {timestamp} AND (status IN ('ready', 'retry') OR priority > 5)) SELECT e.id, COALESCE(c.name, 'unknown') AS name FROM events e LEFT JOIN customers c ON e.customer_id = c.id"
            );
            let query = query(&sql, false);
            let source = SourceQuery::compile(&query, system).unwrap();
            let paged = source.paged_sql(&query, &columns()).unwrap();
            assert!(paged.starts_with("WITH events AS"));
            assert!(paged.contains("LEFT JOIN customers"));
            assert!(paged.contains(timestamp));
            assert!(paged.contains(" IS NULL"));
            assert!(paged.contains("ORDER BY"));
            assert!(!paged.contains("FROM (\nWITH"));
        }
    }

    /// Scenario: Snapshot sources contain aggregation, windows, sets, wildcards, or native limits.
    /// Guarantees: Native snapshot semantics are preserved without injected cursor predicates.
    #[test]
    fn snapshot_preserves_native_result_semantics() {
        for sql in [
            "SELECT status, COUNT(*) AS total FROM events GROUP BY status HAVING COUNT(*) > 1",
            "SELECT *, ROW_NUMBER() OVER (ORDER BY id) AS position FROM events",
            "SELECT id FROM first_events UNION ALL SELECT id FROM second_events",
            "SELECT id FROM events ORDER BY id DESC LIMIT 10 OFFSET 2",
            "SELECT e.id FROM events e WHERE EXISTS (SELECT 1 FROM customers c WHERE c.id = e.customer_id)",
        ] {
            let query = query(sql, true);
            let source = SourceQuery::compile(&query, DatabaseSystem::PostgreSQL).unwrap();
            assert_eq!(source.paged_sql(&query, &columns()).unwrap(), sql);
        }
    }

    /// Scenario: A mutation, locking clause, or second statement is hidden in otherwise read-like SQL.
    /// Guarantees: Nested operations are checked before any native preparation or execution.
    #[test]
    fn rejects_mutations_and_locks_at_every_depth() {
        for sql in [
            "DELETE FROM events",
            "SELECT id INTO copied_events FROM events",
            "SELECT id FROM events; DELETE FROM events",
            "WITH removed AS (DELETE FROM events RETURNING id) SELECT id FROM removed",
            "SELECT id FROM events FOR UPDATE",
            "SELECT id FROM (SELECT id FROM events FOR SHARE) e",
        ] {
            assert!(
                SourceQuery::compile(&query(sql, true), DatabaseSystem::PostgreSQL).is_err(),
                "{sql}"
            );
        }
        for hint in ["NOLOCK", "UPDLOCK", "XLOCK", "HOLDLOCK"] {
            let sql = format!("SELECT id FROM events WITH ({hint})");
            assert_eq!(
                SourceQuery::compile(&query(&sql, true), DatabaseSystem::SqlServer).unwrap_err(),
                SourceQueryError::Locking
            );
        }
        assert_eq!(
            SourceQuery::compile(
                &query("SELECT id FROM events (READPAST)", true),
                DatabaseSystem::SqlServer
            )
            .unwrap_err(),
            SourceQueryError::Locking
        );
    }

    /// Scenario: A configuration still contains a manually authored cursor parameter.
    /// Guarantees: Every dialect reports a migration error instead of retaining a legacy execution path.
    #[test]
    fn rejects_legacy_cursor_parameters() {
        for (system, sql) in [
            (
                DatabaseSystem::PostgreSQL,
                "SELECT id FROM events WHERE id > $1",
            ),
            (
                DatabaseSystem::PostgreSQL,
                "SELECT id FROM events WHERE id > :last_id",
            ),
            (
                DatabaseSystem::Oracle,
                "SELECT id FROM events WHERE id > :last_id",
            ),
            (
                DatabaseSystem::SqlServer,
                "SELECT id FROM events WHERE id > @last_id",
            ),
            (DatabaseSystem::SqlServer, "SELECT @out = id FROM events"),
        ] {
            assert_eq!(
                SourceQuery::compile(&query(sql, false), system).unwrap_err(),
                SourceQueryError::Parameters
            );
        }
    }

    /// Scenario: User paging or descending ordering conflicts with receiver-owned incremental progress.
    /// Guarantees: Conflicting outer clauses are rejected, while matching ordering is moved outside.
    #[test]
    fn owns_incremental_ordering_and_limits() {
        for sql in [
            "SELECT id FROM events LIMIT 2",
            "SELECT id FROM events OFFSET 1",
            "SELECT id FROM events FETCH FIRST 2 ROWS ONLY",
        ] {
            assert_eq!(
                SourceQuery::compile(&query(sql, false), DatabaseSystem::PostgreSQL).unwrap_err(),
                SourceQueryError::Pagination
            );
        }
        assert_eq!(
            SourceQuery::compile(
                &query("SELECT TOP (2) id FROM events", false),
                DatabaseSystem::SqlServer
            )
            .unwrap_err(),
            SourceQueryError::Pagination
        );
        assert_eq!(
            SourceQuery::compile(
                &query("SELECT id FROM events ORDER BY id DESC", false),
                DatabaseSystem::PostgreSQL
            )
            .unwrap_err(),
            SourceQueryError::Ordering
        );
        let query = query("SELECT e.id FROM events e ORDER BY e.id ASC", false);
        let source = SourceQuery::compile(&query, DatabaseSystem::PostgreSQL).unwrap();
        let paged = source.paged_sql(&query, &columns()).unwrap();
        assert_eq!(paged.matches("ORDER BY").count(), 1);
        assert!(paged.ends_with("ASC NULLS FIRST LIMIT 3"));
    }

    /// Scenario: SQL contains comments, delimiter-like string contents, and escaped native literals.
    /// Guarantees: Original literal bytes survive compilation and generated SQL cannot be swallowed by comments.
    #[test]
    fn preserves_literal_and_comment_semantics() {
        let sql = "/* source */ SELECT id, N'can''t; -- change' AS message FROM dbo.events -- tail\n; -- final";
        let query = query(sql, false);
        let source = SourceQuery::compile(&query, DatabaseSystem::SqlServer).unwrap();
        let paged = source.paged_sql(&query, &columns()).unwrap();
        assert!(paged.contains("N'can''t; -- change'"));
        assert!(paged.contains("-- final\n)"));
        assert!(paged.contains("TOP (@P2)"));
    }

    /// Scenario: Source result names are missing, ambiguous, or differ only by case.
    /// Guarantees: No cursor or body field is resolved through an ambiguous result schema.
    #[test]
    fn rejects_ambiguous_result_columns() {
        let query = query("SELECT * FROM events", false);
        let source = SourceQuery::compile(&query, DatabaseSystem::PostgreSQL).unwrap();
        assert_eq!(
            source.paged_sql(&query, &[]),
            Err(SourceQueryError::Columns)
        );
        let mut duplicate = columns();
        duplicate.push(ColumnMetadata {
            name: "ID".into(),
            source_type: "int8".into(),
            nullable: false,
        });
        assert_eq!(
            source.paged_sql(&query, &duplicate),
            Err(SourceQueryError::Columns)
        );
        let mut missing = columns();
        missing[0].name = "another_key".into();
        assert_eq!(
            source.paged_sql(&query, &missing),
            Err(SourceQueryError::Columns)
        );
    }

    /// Scenario: Source formatting changes while literal values and optimizer comments remain significant.
    /// Guarantees: Cosmetic whitespace preserves identity without erasing literal or hint changes.
    #[test]
    fn fingerprints_preserve_sql_semantics() {
        let compile =
            |sql| SourceQuery::compile(&query(sql, true), DatabaseSystem::SqlServer).unwrap();
        let first = compile("SELECT id, N'a  b' AS text FROM events");
        let formatted = compile("\nSELECT\tid,\nN'a  b' AS text\nFROM events;\n");
        assert_eq!(first.fingerprint_sql(), formatted.fingerprint_sql());
        assert_ne!(
            first.fingerprint_sql(),
            compile("SELECT id, N'a b' AS text FROM events").fingerprint_sql()
        );
        assert_ne!(
            first.fingerprint_sql(),
            compile("SELECT /* hint */ id, N'a  b' AS text FROM events").fingerprint_sql()
        );
    }

    /// Scenario: Multibyte literals occur before a CTE's terminal SELECT and final ORDER BY.
    /// Guarantees: Token character locations are converted to byte boundaries without corrupting native SQL.
    #[test]
    fn cte_composition_preserves_utf8_boundaries() {
        let query = query(
            "WITH x AS (SELECT '\u{1f30d}' AS label, id FROM events) SELECT id, label FROM x ORDER BY id",
            false,
        );
        let source = SourceQuery::compile(&query, DatabaseSystem::PostgreSQL).unwrap();
        let sql = source.paged_sql(&query, &columns()).unwrap();
        assert!(sql.starts_with("WITH x AS (SELECT '\u{1f30d}'"));
        assert!(sql.contains("SELECT id, label FROM x "));
        assert_eq!(sql.matches("ORDER BY").count(), 1);
    }
}
