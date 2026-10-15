// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Bounded parameter compilation, AST allowlist, and fixed catalog queries.

use super::adapter::{Error, Result, database};
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use otel_arrow_dfe_scraper::database::{CompiledQuery, MAX_QUERY_BYTES};
use sqlparser::{
    ast::{
        BinaryOperator as Op, Expr, Ident, JoinConstraint, JoinOperator, SelectItem, SetExpr,
        Statement, TableFactor, Value,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
    tokenizer::{Token, Tokenizer},
};
use std::collections::{BTreeMap, BTreeSet};
use tokio_postgres::{GenericClient, Transaction, types::Type};

const RELATION_SQL: &str = "SELECT c.oid, c.relkind::text, c.relpersistence::text, c.relrowsecurity, \
     EXISTS (SELECT 1 FROM pg_catalog.pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid) \
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace \
     WHERE n.nspname=$1 AND c.relname=$2 LIMIT 2";
const ATTRIBUTES_SQL: &str = "SELECT a.attname::text,a.attnum,a.atttypid,a.atttypmod,a.attnotnull,a.attcollation,t.typcollation \
     FROM pg_catalog.pg_attribute a JOIN pg_catalog.pg_type t ON t.oid=a.atttypid \
     WHERE a.attrelid=$1 AND a.attname=ANY($2) AND a.attnum>0 AND NOT a.attisdropped \
     LIMIT (pg_catalog.cardinality($2) + 1)";
const INDEXES_SQL: &str = "SELECT i.indkey::smallint[], i.indnkeyatts, i.indexrelid FROM pg_catalog.pg_index i \
     WHERE i.indrelid=$1 AND i.indisunique AND i.indisvalid AND i.indisready \
     AND i.indimmediate AND i.indpred IS NULL AND i.indexprs IS NULL \
     AND NOT EXISTS (SELECT 1 FROM pg_catalog.unnest(i.indclass::oid[]) k(oid) \
       JOIN pg_catalog.pg_opclass o ON o.oid=k.oid \
       JOIN pg_catalog.pg_namespace n ON n.oid=o.opcnamespace WHERE n.nspname<>'pg_catalog') \
     AND NOT EXISTS (SELECT 1 FROM pg_catalog.generate_series(0,i.indnkeyatts-1) k(n) \
       JOIN pg_catalog.pg_attribute a ON a.attrelid=i.indrelid AND a.attnum=i.indkey[k.n] \
       WHERE i.indcollation[k.n]<>a.attcollation) \
     ORDER BY i.indexrelid LIMIT 129";

pub(crate) struct CatalogStatements {
    relation: tokio_postgres::Statement,
    attributes: tokio_postgres::Statement,
    indexes: tokio_postgres::Statement,
}

impl CatalogStatements {
    pub async fn prepare(
        client: &impl GenericClient,
        operation: &super::adapter::Operation,
    ) -> Result<Self> {
        operation.check()?;
        let relation = client
            .prepare_typed(RELATION_SQL, &[Type::TEXT, Type::TEXT])
            .await
            .map_err(database)?;
        operation.check()?;
        let attributes = client
            .prepare_typed(ATTRIBUTES_SQL, &[Type::OID, Type::TEXT_ARRAY])
            .await
            .map_err(database)?;
        operation.check()?;
        let indexes = client
            .prepare_typed(INDEXES_SQL, &[Type::OID])
            .await
            .map_err(database)?;
        operation.check()?;
        Ok(Self {
            relation,
            attributes,
            indexes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ColumnRef {
    pub alias: String,
    pub name: String,
}
#[derive(Clone)]
pub(crate) struct Relation {
    pub schema: String,
    pub name: String,
    pub alias: String,
    pub join: Vec<(ColumnRef, ColumnRef)>,
}
#[derive(Clone)]
pub(crate) struct Plan {
    pub sql: String,
    pub relations: Vec<Relation>,
    pub projections: Vec<ColumnRef>,
    pub referenced: BTreeSet<ColumnRef>,
    pub filters: Vec<(ColumnRef, Option<Value>)>,
    pub output_names: Vec<String>,
    pub timestamp: usize,
    pub tie: usize,
    pub output_timestamp: Option<usize>,
}

fn identifier(id: &Ident) -> Result<String> {
    super::config::validate_name(&id.value, 63).map_err(|_| Error::Sql)?;
    Ok(if id.quote_style.is_some() {
        id.value.clone()
    } else {
        id.value.to_ascii_lowercase()
    })
}
fn column(expr: &Expr) -> Result<ColumnRef> {
    if let Expr::CompoundIdentifier(ids) = expr
        && let [alias, name] = ids.as_slice()
    {
        return Ok(ColumnRef {
            alias: identifier(alias)?,
            name: identifier(name)?,
        });
    }
    Err(Error::Sql)
}
fn unnest(mut expr: &Expr) -> &Expr {
    while let Expr::Nested(inner) = expr {
        expr = inner;
    }
    expr
}
fn bind(expr: &Expr, expected: &str) -> bool {
    matches!(unnest(expr), Expr::Value(v) if matches!(&v.value, Value::Placeholder(p) if p == expected))
}
fn comparison(expr: &Expr, left: &ColumnRef, op: Op, param: &str) -> bool {
    matches!(unnest(expr), Expr::BinaryOp { left: l, op: o, right: r }
        if *o == op && column(unnest(l)).as_ref() == Ok(left) && bind(r, param))
}
fn keyset(expr: &Expr, ts: &ColumnRef, tie: &ColumnRef) -> bool {
    match unnest(expr) {
        Expr::BinaryOp {
            left,
            op: Op::Or,
            right,
        } => {
            comparison(left, ts, Op::Gt, "$1")
                && matches!(unnest(right), Expr::BinaryOp { left: a, op: Op::And, right: b }
                    if comparison(a, ts, Op::Eq, "$1") && comparison(b, tie, Op::Gt, "$2"))
        }
        Expr::BinaryOp {
            left,
            op: Op::Gt,
            right,
        } => {
            matches!((unnest(left), unnest(right)), (Expr::Tuple(l), Expr::Tuple(r))
                if l.len() == 2 && r.len() == 2 && column(&l[0]).as_ref() == Ok(ts)
                && column(&l[1]).as_ref() == Ok(tie) && bind(&r[0], "$1") && bind(&r[1], "$2"))
        }
        _ => false,
    }
}

fn canonicalize_keyset(expr: &mut Expr, ts: &ColumnRef, tie: &ColumnRef) {
    if let Expr::Nested(inner) = expr {
        canonicalize_keyset(inner, ts, tie);
        return;
    }
    if keyset(expr, ts, tie)
        && let Expr::BinaryOp {
            left,
            op: Op::Or,
            right,
        } = expr
        && let Expr::BinaryOp {
            left: timestamp,
            right: timestamp_bind,
            ..
        } = unnest(left)
        && let Expr::BinaryOp { right, .. } = unnest(right)
        && let Expr::BinaryOp {
            left: id,
            right: id_bind,
            ..
        } = unnest(right)
    {
        // Catalog proof requires native, non-null cursor columns; their built-in
        // comparisons make this exact OR keyset equivalent to row comparison.
        *expr = Expr::BinaryOp {
            left: Box::new(Expr::Tuple(vec![
                unnest(timestamp).clone(),
                unnest(id).clone(),
            ])),
            op: Op::Gt,
            right: Box::new(Expr::Tuple(vec![
                unnest(timestamp_bind).clone(),
                unnest(id_bind).clone(),
            ])),
        };
    } else if let Expr::BinaryOp {
        left,
        op: Op::And,
        right,
    } = expr
    {
        canonicalize_keyset(left, ts, tie);
        canonicalize_keyset(right, ts, tie);
    }
}

fn literal(expr: &Expr) -> Result<Value> {
    match unnest(expr) {
        Expr::Value(value)
            if matches!(
                &value.value,
                Value::Number(..) | Value::Boolean(_) | Value::SingleQuotedString(_)
            ) =>
        {
            Ok(value.value.clone())
        }
        Expr::UnaryOp {
            op: sqlparser::ast::UnaryOperator::Minus,
            expr,
        } => match unnest(expr) {
            Expr::Value(v) => match &v.value {
                Value::Number(n, long) => Ok(Value::Number(format!("-{n}"), *long)),
                _ => Err(Error::Sql),
            },
            _ => Err(Error::Sql),
        },
        _ => Err(Error::Sql),
    }
}
fn where_clause(
    expr: &Expr,
    ts: &ColumnRef,
    tie: &ColumnRef,
    filters: &mut Vec<(ColumnRef, Option<Value>)>,
) -> Result<usize> {
    if keyset(expr, ts, tie) {
        return Ok(1);
    }
    match unnest(expr) {
        Expr::BinaryOp {
            left,
            op: Op::And,
            right,
        } => Ok(where_clause(left, ts, tie, filters)? + where_clause(right, ts, tie, filters)?),
        Expr::BinaryOp {
            left,
            op: Op::Eq | Op::NotEq | Op::Gt | Op::Lt | Op::GtEq | Op::LtEq,
            right,
        } => {
            filters.push((column(unnest(left))?, Some(literal(right)?)));
            Ok(0)
        }
        Expr::IsNull(c) | Expr::IsNotNull(c) => {
            filters.push((column(unnest(c))?, None));
            Ok(0)
        }
        _ => Err(Error::Sql),
    }
}

pub(crate) fn compile_parameters(sql: &str, timestamp: &str, tie: &str) -> Result<String> {
    fn valid(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 64
            && name
                .bytes()
                .enumerate()
                .all(|(i, b)| b == b'_' || b.is_ascii_lowercase() || (i > 0 && b.is_ascii_digit()))
    }
    if sql.is_empty()
        || sql.len() > MAX_QUERY_BYTES
        || timestamp == tie
        || !valid(timestamp)
        || !valid(tie)
    {
        return Err(Error::Sql);
    }
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .map_err(|_| Error::Sql)?;
    if tokens.len() > 4096 {
        return Err(Error::Sql);
    }
    // Token locations count characters, while string edits need byte offsets.
    // Preserve UTF-8 literals/comments outside actual parameter tokens verbatim.
    let mut offsets = BTreeMap::new();
    let (mut line, mut col) = (1u64, 1u64);
    for (index, ch) in sql.char_indices() {
        let _ = offsets.insert((line, col), index);
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    let _ = offsets.insert((line, col), sql.len());
    let mut edits = Vec::new();
    let (mut named, mut positional, mut depth) = (false, false, 0usize);
    let mut used = [false; 2];
    let mut i = 0;
    while i < tokens.len() {
        let token = &tokens[i];
        match &token.token {
            Token::LParen => {
                depth += 1;
                if depth > 16 {
                    return Err(Error::Sql);
                }
            }
            Token::RParen => {
                depth = depth.checked_sub(1).ok_or(Error::Sql)?;
            }
            Token::SemiColon => return Err(Error::Sql),
            Token::Colon => {
                let next = tokens.get(i + 1).ok_or(Error::Sql)?;
                let Token::Word(word) = &next.token else {
                    return Err(Error::Sql);
                };
                if word.quote_style.is_some() {
                    return Err(Error::Sql);
                }
                let role = if word.value == timestamp {
                    0
                } else if word.value == tie {
                    1
                } else {
                    return Err(Error::Sql);
                };
                let start = *offsets
                    .get(&(token.span.start.line, token.span.start.column))
                    .ok_or(Error::Sql)?;
                let end = *offsets
                    .get(&(next.span.end.line, next.span.end.column))
                    .ok_or(Error::Sql)?;
                if sql.get(start..end) != Some(format!(":{}", word.value).as_str()) {
                    return Err(Error::Sql);
                }
                edits.push((start, end, if role == 0 { "$1" } else { "$2" }));
                used[role] = true;
                named = true;
                i += 1;
            }
            Token::Placeholder(p) => {
                let role = match p.as_str() {
                    "$1" => 0,
                    "$2" => 1,
                    _ => return Err(Error::Sql),
                };
                used[role] = true;
                positional = true;
            }
            _ => {}
        }
        i += 1;
    }
    if depth != 0 || (named && positional) || !used.into_iter().all(|v| v) {
        return Err(Error::Sql);
    }
    let mut result = String::with_capacity(sql.len());
    let mut previous = 0;
    for (start, end, replacement) in edits {
        result.push_str(&sql[previous..start]);
        result.push_str(replacement);
        previous = end;
    }
    result.push_str(&sql[previous..]);
    if result.len() > MAX_QUERY_BYTES {
        return Err(Error::Sql);
    }
    Ok(result)
}

fn parse(sql: &str) -> Result<Vec<Statement>> {
    Parser::new(&PostgreSqlDialect {})
        .with_recursion_limit(64)
        .try_with_sql(sql)
        .map_err(|_| Error::Sql)?
        .parse_statements()
        .map_err(|_| Error::Sql)
}

fn relation(factor: &TableFactor) -> Result<Relation> {
    let TableFactor::Table {
        name,
        alias: Some(alias),
        ..
    } = factor
    else {
        return Err(Error::Sql);
    };
    let tokens = Tokenizer::new(&PostgreSqlDialect {}, &name.to_string())
        .tokenize()
        .map_err(|_| Error::Sql)?;
    let [Token::Word(schema), Token::Period, Token::Word(table)] = tokens.as_slice() else {
        return Err(Error::Sql);
    };
    let schema = identifier(&if let Some(quote) = schema.quote_style {
        Ident::with_quote(quote, &schema.value)
    } else {
        Ident::new(&schema.value)
    })?;
    let table = identifier(&if let Some(quote) = table.quote_style {
        Ident::with_quote(quote, &table.value)
    } else {
        Ident::new(&table.value)
    })?;
    if schema.starts_with("pg_") || schema == "information_schema" {
        return Err(Error::Sql);
    }
    let normalized = format!("SELECT 1 FROM {name} AS {}", alias.name);
    let parsed = parse(&normalized)?;
    let Statement::Query(query) = &parsed[0] else {
        return Err(Error::Sql);
    };
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Err(Error::Sql);
    };
    if &select.from[0].relation != factor {
        return Err(Error::Sql);
    }
    Ok(Relation {
        schema,
        name: table,
        alias: identifier(&alias.name)?,
        join: vec![],
    })
}

fn joins(expr: &Expr, pairs: &mut Vec<(ColumnRef, ColumnRef)>) -> Result<()> {
    match unnest(expr) {
        Expr::BinaryOp {
            left,
            op: Op::And,
            right,
        } => {
            joins(left, pairs)?;
            joins(right, pairs)
        }
        Expr::BinaryOp {
            left,
            op: Op::Eq,
            right,
        } => {
            pairs.push((column(unnest(left))?, column(unnest(right))?));
            Ok(())
        }
        _ => Err(Error::Sql),
    }
}

impl Plan {
    pub fn compile(common: &CompiledQuery) -> Result<Self> {
        let watermark = common.watermark().as_composite().ok_or(Error::Config)?;
        let sql = compile_parameters(
            common.sql(),
            &watermark.timestamp_bind,
            &watermark.tie_breaker_bind,
        )?;
        let statements = parse(&sql)?;
        let [Statement::Query(query)] = statements.as_slice() else {
            return Err(Error::Sql);
        };
        let SetExpr::Select(select) = query.body.as_ref() else {
            return Err(Error::Sql);
        };
        if select.from.len() != 1 || !(1..=128).contains(&select.projection.len()) {
            return Err(Error::Sql);
        }
        let from = &select.from[0];
        if from.joins.len() > 3 {
            return Err(Error::Sql);
        }
        let mut relations = vec![relation(&from.relation)?];
        for join in &from.joins {
            let JoinOperator::Inner(JoinConstraint::On(expr)) = &join.join_operator else {
                return Err(Error::Sql);
            };
            let mut rel = relation(&join.relation)?;
            if relations.iter().any(|r| r.alias == rel.alias) {
                return Err(Error::Sql);
            }
            joins(expr, &mut rel.join)?;
            for (left, right) in &mut rel.join {
                if left.alias == rel.alias {
                    std::mem::swap(left, right);
                }
                if right.alias != rel.alias || !relations.iter().any(|r| r.alias == left.alias) {
                    return Err(Error::Sql);
                }
            }
            relations.push(rel);
        }
        let mut projections = Vec::new();
        let mut output_names = Vec::new();
        let mut unique_names = BTreeSet::new();
        for item in &select.projection {
            let (expr, alias) = match item {
                SelectItem::UnnamedExpr(expr) => (expr, None),
                SelectItem::ExprWithAlias { expr, alias } => (expr, Some(identifier(alias)?)),
                _ => return Err(Error::Sql),
            };
            let reference = column(expr)?;
            let name = alias.unwrap_or_else(|| reference.name.clone());
            if !unique_names.insert(name.to_ascii_lowercase()) {
                return Err(Error::Sql);
            }
            output_names.push(name);
            projections.push(reference);
        }
        let column_index = |name: &str| {
            output_names
                .iter()
                .position(|column| column.eq_ignore_ascii_case(name))
                .ok_or(Error::Config)
        };
        let timestamp = column_index(&watermark.timestamp_column)?;
        let tie = column_index(&watermark.tie_breaker_column)?;
        if timestamp == tie {
            return Err(Error::Config);
        }
        let output_timestamp = common
            .output()
            .timestamp_column
            .as_deref()
            .map(column_index)
            .transpose()?;
        let ts = &projections[timestamp];
        let id = &projections[tie];
        if ts.alias != relations[0].alias
            || id.alias != relations[0].alias
            || projections.iter().filter(|c| *c == ts).count() != 1
            || projections.iter().filter(|c| *c == id).count() != 1
        {
            return Err(Error::Sql);
        }
        let selection = select.selection.as_ref().ok_or(Error::Sql)?;
        let mut filters = vec![];
        if where_clause(selection, ts, id, &mut filters)? != 1 {
            return Err(Error::Sql);
        }
        let order = query.order_by.as_ref().ok_or(Error::Sql)?.to_string();
        let ts_expr = match &select.projection[timestamp] {
            SelectItem::UnnamedExpr(e) | SelectItem::ExprWithAlias { expr: e, .. } => e,
            _ => return Err(Error::Sql),
        };
        let id_expr = match &select.projection[tie] {
            SelectItem::UnnamedExpr(e) | SelectItem::ExprWithAlias { expr: e, .. } => e,
            _ => return Err(Error::Sql),
        };
        if !["", " ASC"].iter().any(|a| {
            ["", " ASC"]
                .iter()
                .any(|b| order == format!("ORDER BY {ts_expr}{a}, {id_expr}{b}"))
        }) {
            return Err(Error::Sql);
        }
        // Equality with an allowlisted reconstruction rejects every unexamined
        // query/select field, including new parser fields, before any rewrite.
        let projection = select
            .projection
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let allowed = parse(&format!(
            "SELECT {projection} FROM {from} WHERE {selection} {order}"
        ))?;
        if allowed != statements {
            return Err(Error::Sql);
        }
        // Filters and joins can change which rows advance the cursor even when
        // their columns are not emitted, so they need the same catalog checks.
        let mut referenced: BTreeSet<_> = projections.iter().cloned().collect();
        for rel in &relations {
            for (left, right) in &rel.join {
                let _ = referenced.insert(left.clone());
                let _ = referenced.insert(right.clone());
            }
        }
        for (col, _) in &filters {
            let _ = referenced.insert(col.clone());
        }
        if referenced
            .iter()
            .any(|c| !relations.iter().any(|r| r.alias == c.alias))
        {
            return Err(Error::Sql);
        }
        let mut optimized = query.clone();
        let SetExpr::Select(select) = optimized.body.as_mut() else {
            return Err(Error::Sql);
        };
        canonicalize_keyset(select.selection.as_mut().ok_or(Error::Sql)?, ts, id);
        let sql = format!("{optimized} LIMIT {}", common.max_rows());
        Ok(Self {
            sql,
            relations,
            projections,
            referenced,
            filters,
            output_names,
            timestamp,
            tie,
            output_timestamp,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeColumn {
    pub oid: u32,
    pub attribute: i16,
    pub type_oid: u32,
    pub modifier: i32,
    pub not_null: bool,
    pub collation: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Signature {
    pub columns: BTreeMap<ColumnRef, NativeColumn>,
    pub keys: Vec<(u32, Vec<(u32, Vec<i16>)>)>,
}

pub(crate) struct ResultColumn {
    pub name: String,
    pub source_type: String,
    pub nullable: bool,
    pub type_modifier: i32,
}

pub(crate) struct ResolvedSchema {
    pub columns: Vec<ResultColumn>,
    pub native_types: Vec<Type>,
}

impl ResolvedSchema {
    pub fn discover(plan: &Plan, common: &CompiledQuery, signature: &Signature) -> Result<Self> {
        if plan.projections.len() != plan.output_names.len() {
            return Err(Error::Metadata);
        }
        let mut columns = Vec::with_capacity(plan.output_names.len());
        let mut native_types = Vec::with_capacity(plan.output_names.len());
        for (reference, name) in plan.projections.iter().zip(&plan.output_names) {
            let source = signature.columns.get(reference).ok_or(Error::Metadata)?;
            let native = Type::from_oid(source.type_oid).ok_or(Error::Metadata)?;
            let native = super::value::native_type(native.name())?;
            if source.modifier < -1
                || (matches!(native, Type::TIMESTAMP | Type::TIMESTAMPTZ)
                    && !(-1..=6).contains(&source.modifier))
            {
                return Err(Error::Metadata);
            }
            columns.push(ResultColumn {
                name: name.clone(),
                source_type: native.name().to_owned(),
                nullable: !source.not_null,
                type_modifier: source.modifier,
            });
            native_types.push(native);
        }
        let timestamp = &columns[plan.timestamp];
        let tie = &columns[plan.tie];
        if timestamp.nullable
            || tie.nullable
            || !matches!(
                native_types[plan.timestamp],
                Type::TIMESTAMP | Type::TIMESTAMPTZ
            )
            || !matches!(native_types[plan.tie], Type::INT2 | Type::INT4 | Type::INT8)
            || plan.output_timestamp.is_some_and(|index| {
                !matches!(native_types[index], Type::TIMESTAMP | Type::TIMESTAMPTZ)
            })
        {
            return Err(Error::Metadata);
        }
        let watermark = common.watermark().as_composite().ok_or(Error::Config)?;
        let _ = super::value::cursor_time(&watermark.initial.timestamp, timestamp.type_modifier)?;
        super::value::check_tie(watermark.initial.tie_breaker, &tie.source_type)?;
        Ok(Self {
            columns,
            native_types,
        })
    }
}

impl Plan {
    pub async fn catalog(
        &self,
        tx: &Transaction<'_>,
        statements: &CatalogStatements,
        operation: &super::adapter::Operation,
    ) -> Result<Signature> {
        let mut columns = BTreeMap::new();
        let mut keys = vec![];
        for relation in &self.relations {
            operation.check()?;
            let rows = tx
                .query(&statements.relation, &[&relation.schema, &relation.name])
                .await
                .map_err(database)?;
            operation.check()?;
            let [row] = rows.as_slice() else {
                return Err(Error::Metadata);
            };
            let oid: u32 = row.try_get(0).map_err(database)?;
            if row.try_get::<_, &str>(1).map_err(database)? != "r"
                || row.try_get::<_, &str>(2).map_err(database)? != "p"
                || row.try_get::<_, bool>(3).map_err(database)?
                || row.try_get::<_, bool>(4).map_err(database)?
            {
                return Err(Error::Metadata);
            }
            let requested: BTreeSet<_> = self
                .referenced
                .iter()
                .filter(|c| c.alias == relation.alias)
                .map(|c| c.name.as_str())
                .collect();
            // The validated SQL bounds this array. Return one extra row so a
            // catalog mismatch cannot hide behind truncation to the expected size.
            let names: Vec<_> = requested.iter().copied().collect();
            operation.check()?;
            // Attributes and indexes depend only on the relation OID, and the
            // contract is revalidated on every page. Issue them together so the
            // connection pipelines both instead of paying two serial waits.
            let (rows, index_rows) = futures::future::try_join(
                tx.query(&statements.attributes, &[&oid, &names]),
                tx.query(&statements.indexes, &[&oid]),
            )
            .await
            .map_err(database)?;
            operation.check()?;
            if rows.len() != requested.len() {
                return Err(Error::Metadata);
            }
            let mut remaining = requested;
            for row in rows {
                operation.check()?;
                let name: &str = row.try_get(0).map_err(database)?;
                if !remaining.remove(name) {
                    return Err(Error::Metadata);
                }
                let col = NativeColumn {
                    oid,
                    attribute: row.try_get(1).map_err(database)?,
                    type_oid: row.try_get(2).map_err(database)?,
                    modifier: row.try_get(3).map_err(database)?,
                    not_null: row.try_get(4).map_err(database)?,
                    collation: row.try_get(5).map_err(database)?,
                };
                let native = Type::from_oid(col.type_oid).ok_or(Error::Metadata)?;
                let _ = super::value::native_type(native.name()).map_err(|_| Error::Metadata)?;
                if col.collation != row.try_get::<_, u32>(6).map_err(database)? {
                    return Err(Error::Metadata);
                }
                let _ = columns.insert(
                    ColumnRef {
                        alias: relation.alias.clone(),
                        name: name.to_owned(),
                    },
                    col,
                );
            }
            operation.check()?;
            if index_rows.len() > 128 {
                return Err(Error::Limit);
            }
            let mut unique = vec![];
            for row in index_rows {
                let mut attrs: Vec<i16> = row.try_get(0).map_err(database)?;
                let n: i16 = row.try_get(1).map_err(database)?;
                if n <= 0 || n as usize > attrs.len() || attrs.len() > 32 {
                    return Err(Error::Metadata);
                }
                attrs.truncate(n as usize);
                unique.push((row.try_get::<_, u32>(2).map_err(database)?, attrs));
            }
            if !relation.join.is_empty() {
                let mut joined = BTreeSet::new();
                for (left, right) in &relation.join {
                    let l = columns.get(left).ok_or(Error::Metadata)?;
                    let r = columns.get(right).ok_or(Error::Metadata)?;
                    if l.type_oid != r.type_oid || l.collation != r.collation {
                        return Err(Error::Metadata);
                    }
                    let _ = joined.insert(r.attribute);
                }
                // Covering a unique key on the newly joined side prevents that
                // join from multiplying a driving row into duplicate cursor positions.
                if !unique
                    .iter()
                    .any(|(_, key)| key.iter().all(|attribute| joined.contains(attribute)))
                {
                    return Err(Error::Metadata);
                }
            }
            keys.push((oid, unique));
        }
        for (reference, literal) in &self.filters {
            if let Some(literal) = literal {
                let col = columns.get(reference).ok_or(Error::Metadata)?;
                compatible_literal(col.type_oid, literal)?;
            }
        }
        Ok(Signature { columns, keys })
    }
}

fn strict_absolute_date(text: &str) -> bool {
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .is_ok_and(|value| (1..=9999).contains(&value.year()))
}

fn strict_absolute_timestamp(text: &str) -> bool {
    DateTime::parse_from_rfc3339(text)
        .map(|value| value.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f"))
                .map(|value| value.and_utc())
        })
        .is_ok_and(|value| (1..=9999).contains(&value.year()))
}

pub(crate) fn compatible_literal(oid: u32, value: &Value) -> Result<()> {
    let ty = Type::from_oid(oid).ok_or(Error::Metadata)?;
    let valid = match value {
        Value::Boolean(_) => ty == Type::BOOL,
        Value::Number(n, _) => match ty {
            Type::INT2 => n.parse::<i16>().is_ok(),
            Type::INT4 => n.parse::<i32>().is_ok(),
            Type::INT8 => n.parse::<i64>().is_ok(),
            Type::NUMERIC => true,
            Type::FLOAT4 => n.parse::<f32>().is_ok_and(f32::is_finite),
            Type::FLOAT8 => n.parse::<f64>().is_ok_and(f64::is_finite),
            _ => false,
        },
        Value::SingleQuotedString(text) => match ty {
            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::UUID => true,
            Type::DATE => strict_absolute_date(text),
            Type::TIMESTAMP | Type::TIMESTAMPTZ => {
                strict_absolute_date(text) || strict_absolute_timestamp(text)
            }
            _ => false,
        },
        _ => false,
    };
    if valid { Ok(()) } else { Err(Error::Metadata) }
}
