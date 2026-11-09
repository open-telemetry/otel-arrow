// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Native authored SQL with a mandatory checkpoint guard and bounded fetching.

use super::{
    ColumnMetadata, CompiledQuery, CompiledWatermark, DatabaseSystem, MAX_QUERY_BYTES, ScalarValue,
};
use sqlparser::{
    ast::{
        BinaryOperator as Op, Expr, FunctionArg, FunctionArgExpr, Ident, OrderByKind, OrderBySort,
        Query, Select, SelectItem, SetExpr, Statement, TableFactor, Visit, Visitor,
    },
    dialect::{Dialect, MsSqlDialect, OracleDialect, PostgreSqlDialect},
    parser::Parser,
    tokenizer::{Span, Token, TokenWithSpan, Tokenizer, Whitespace},
};
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
    ops::ControlFlow,
    ops::Range,
};

const MAX_TOKENS: usize = 4096;
const MAX_NESTING: usize = 128;

/// One validated authored query, with native parameter names and a bounded fetch plan.
///
/// Edits use token spans rather than AST rendering so literal escaping and
/// optimizer comments retain their exact native meaning.
#[derive(Clone)]
pub struct SourceQuery {
    sql: String,
    execution: String,
    fingerprint: String,
    columns: Vec<String>,
}

impl fmt::Debug for SourceQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceQuery")
            .field("sql", &"<redacted>")
            .finish()
    }
}

impl SourceQuery {
    /// Removes one real final statement terminator before applying an adapter's
    /// legacy trimmed-input bound, without stripping semicolons inside literals.
    pub fn trim_statement(sql: &str, system: DatabaseSystem) -> Result<String, SourceQueryError> {
        let sql = sql.trim();
        let Some(before_terminator) = sql.strip_suffix(';') else {
            if sql.is_empty() || sql.len() > MAX_QUERY_BYTES {
                return Err(SourceQueryError::Size);
            }
            return Ok(sql.to_owned());
        };
        let body = before_terminator.trim_end();
        if body.is_empty() || body.len() > MAX_QUERY_BYTES {
            return Err(SourceQueryError::Size);
        }
        let dialect: &dyn Dialect = match system {
            DatabaseSystem::Oracle => &OracleDialect {},
            DatabaseSystem::SqlServer => &MsSqlDialect {},
            DatabaseSystem::PostgreSQL => &PostgreSqlDialect {},
        };
        // Tokenize only the bounded body, not arbitrarily long whitespace before
        // its terminator. A trailing line comment may itself end in a semicolon.
        let tokens = tokenize(body, dialect)?;
        if matches!(
            tokens.last().map(|t| &t.token),
            Some(Token::Whitespace(Whitespace::SingleLineComment { .. }))
        ) && !before_terminator[body.len()..].contains(['\r', '\n'])
        {
            if sql.len() > MAX_QUERY_BYTES {
                return Err(SourceQueryError::Size);
            }
            return Ok(sql.to_owned());
        }
        if tokens.iter().any(|token| token.token == Token::SemiColon) {
            return Err(SourceQueryError::Statement);
        }
        Ok(body.to_owned())
    }

    /// Validates native read-only SQL, declared binds, and the required incremental guard.
    pub fn compile(
        query: &CompiledQuery,
        system: DatabaseSystem,
    ) -> Result<Self, SourceQueryError> {
        let dialect: &dyn Dialect = match system {
            DatabaseSystem::Oracle => &OracleDialect {},
            DatabaseSystem::SqlServer => &MsSqlDialect {},
            DatabaseSystem::PostgreSQL => &PostgreSqlDialect {},
        };
        let authored = query.sql();
        if authored.trim().is_empty() || authored.len() > MAX_QUERY_BYTES {
            return Err(SourceQueryError::Size);
        }
        let tokens = tokenize(authored, dialect)?;
        let fingerprint = match system {
            DatabaseSystem::PostgreSQL => authored.to_owned(),
            DatabaseSystem::Oracle => authored.trim().trim_end_matches(';').trim().to_owned(),
            DatabaseSystem::SqlServer => canonical_tsql(&tokens),
        };
        let (names, roles, string_key) = match query.watermark() {
            CompiledWatermark::Snapshot => (vec![], vec![], false),
            CompiledWatermark::Scalar(cursor) => (
                vec![cursor.column.as_str()],
                vec![cursor.bind.as_str()],
                matches!(cursor.initial, ScalarValue::String(_)),
            ),
            CompiledWatermark::Composite(cursor) => (
                vec![
                    cursor.timestamp_column.as_str(),
                    cursor.tie_breaker_column.as_str(),
                ],
                vec![
                    cursor.timestamp_bind.as_str(),
                    cursor.tie_breaker_bind.as_str(),
                ],
                false,
            ),
        };
        let sql = compile_parameters(authored, &tokens, &roles, system)?;
        let tokens = tokenize(&sql, dialect)?;
        let mut parser = Parser::new(dialect)
            .with_recursion_limit(MAX_NESTING)
            .with_tokens_with_locations(tokens.clone());
        let statements = parser
            .parse_statements()
            .map_err(|_| SourceQueryError::Syntax)?;
        if !matches!(parser.peek_token_ref().token, Token::EOF) {
            return Err(SourceQueryError::Statement);
        }
        let [Statement::Query(statement)] = statements.as_slice() else {
            return Err(SourceQueryError::Statement);
        };
        if let ControlFlow::Break(error) = statements.visit(&mut ReadOnly { system, depth: 0 }) {
            return Err(error);
        }
        let execution = if names.is_empty() {
            sql.clone()
        } else {
            compile_keyset(
                &sql,
                &tokens,
                statement,
                &names,
                &roles,
                string_key,
                system,
                query.max_rows(),
            )?
        };
        Ok(Self {
            sql,
            execution,
            fingerprint,
            columns: names.into_iter().map(str::to_owned).collect(),
        })
    }

    /// Returns authored SQL with only native parameter substitution and terminator removal.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Returns the original dialect's checkpoint-identity spelling.
    #[must_use]
    pub fn fingerprint_sql(&self) -> &str {
        &self.fingerprint
    }

    /// Returns bounded execution SQL after checking the actual result-column names.
    pub fn paged_sql(
        &self,
        _query: &CompiledQuery,
        columns: &[ColumnMetadata],
    ) -> Result<String, SourceQueryError> {
        let mut names = HashSet::new();
        if columns.is_empty()
            || columns
                .iter()
                .any(|c| c.name.trim().is_empty() || !names.insert(c.name.to_ascii_lowercase()))
            || self
                .columns
                .iter()
                .any(|c| !names.contains(&c.to_ascii_lowercase()))
        {
            return Err(SourceQueryError::Columns);
        }
        Ok(self.execution.clone())
    }
}

fn tokenize(sql: &str, dialect: &dyn Dialect) -> Result<Vec<TokenWithSpan>, SourceQueryError> {
    let tokens = Tokenizer::new(dialect, sql)
        .tokenize_with_location()
        .map_err(|_| SourceQueryError::Syntax)?;
    // sqlparser 0.63 stops -- comments at CR only in its PostgreSQL dialect.
    // SQL Server also treats CR as a line end, so accepting the opaque token
    // could hide native executable SQL. Do not normalize literal contents.
    if !dialect.is::<PostgreSqlDialect>()
        && tokens.iter().enumerate().any(|(token_index, token)| {
            matches!(&token.token, Token::Whitespace(Whitespace::SingleLineComment { comment, .. })
                if comment.as_bytes().iter().enumerate().any(|(index, byte)|
                    *byte == b'\r'
                        && comment.as_bytes().get(index + 1) != Some(&b'\n')
                        // The lexer now leaves LF in the following whitespace token.
                        && !(index + 1 == comment.len()
                            && tokens.get(token_index + 1).is_some_and(|next|
                                matches!(next.token, Token::Whitespace(Whitespace::Newline))
                                    && token.span.end == next.span.start))))
        })
    {
        return Err(SourceQueryError::CommentLineEnding);
    }
    if tokens
        .iter()
        .filter(|t| !matches!(t.token, Token::Whitespace(_)))
        .count()
        > MAX_TOKENS
    {
        return Err(SourceQueryError::Complexity);
    }
    Ok(tokens)
}

// Keep SQL Server's original v1 token canonicalization, including != -> <>.
// The fingerprint is never used for execution and never includes generated paging.
fn canonical_tsql(tokens: &[TokenWithSpan]) -> String {
    tokens
        .iter()
        .filter(|t| {
            !matches!(
                t.token,
                Token::SemiColon
                    | Token::Whitespace(Whitespace::Space | Whitespace::Tab | Whitespace::Newline)
            )
        })
        .map(|t| match &t.token {
            Token::SingleQuotedString(v) => format!("'{}'", v.replace('\'', "''")),
            Token::NationalStringLiteral(v) => format!("N'{}'", v.replace('\'', "''")),
            Token::Word(w) if w.quote_style == Some('[') => {
                format!("[{}]", w.value.replace(']', "]]"))
            }
            token => token.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

struct Edit {
    range: Range<usize>,
    text: String,
}

fn edit_sql(sql: &str, mut edits: Vec<Edit>) -> Result<String, SourceQueryError> {
    edits.sort_by_key(|edit| (edit.range.start, edit.range.end));
    let mut result = String::new();
    let mut offset = 0;
    for edit in edits {
        if edit.range.start < offset || edit.range.end > sql.len() {
            return Err(SourceQueryError::Composition);
        }
        result.push_str(
            sql.get(offset..edit.range.start)
                .ok_or(SourceQueryError::Composition)?,
        );
        result.push_str(&edit.text);
        offset = edit.range.end;
    }
    result.push_str(&sql[offset..]);
    Ok(result)
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

fn byte_range(
    offsets: &BTreeMap<(u64, u64), usize>,
    span: Span,
) -> Result<Range<usize>, SourceQueryError> {
    let start = *offsets
        .get(&(span.start.line, span.start.column))
        .ok_or(SourceQueryError::Composition)?;
    let end = *offsets
        .get(&(span.end.line, span.end.column))
        .ok_or(SourceQueryError::Composition)?;
    Ok(start..end)
}

fn native_bind(system: DatabaseSystem, roles: &[&str], index: usize) -> String {
    match system {
        DatabaseSystem::Oracle => format!(":{}", roles[index]),
        DatabaseSystem::SqlServer => format!("@P{}", index + 1),
        DatabaseSystem::PostgreSQL => format!("${}", index + 1),
    }
}

fn compile_parameters(
    sql: &str,
    tokens: &[TokenWithSpan],
    roles: &[&str],
    system: DatabaseSystem,
) -> Result<String, SourceQueryError> {
    let positions = offsets(sql);
    let significant: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.token, Token::Whitespace(_)))
        .collect();
    let mut edits = Vec::new();
    let mut named = false;
    let mut positional = false;
    let mut index = 0;
    while index < significant.len() {
        let token = significant[index];
        if token.token == Token::SemiColon {
            if index + 1 != significant.len() {
                return Err(SourceQueryError::Statement);
            }
            edits.push(Edit {
                range: byte_range(&positions, token.span)?,
                text: String::new(),
            });
            index += 1;
            continue;
        }
        if system == DatabaseSystem::SqlServer && token.token == Token::DoubleEq {
            return Err(SourceQueryError::Syntax);
        }
        let mut range = byte_range(&positions, token.span)?;
        let parameter = match &token.token {
            Token::Colon => {
                let next = significant
                    .get(index + 1)
                    .ok_or(SourceQueryError::Parameters)?;
                let Token::Word(word) = &next.token else {
                    return Err(SourceQueryError::Parameters);
                };
                let next_range = byte_range(&positions, next.span)?;
                if word.quote_style.is_some() || range.end != next_range.start {
                    return Err(SourceQueryError::Parameters);
                }
                range.end = next_range.end;
                index += 1;
                Some(format!(":{}", word.value))
            }
            Token::Placeholder(value) => Some(value.clone()),
            Token::Word(word) if word.quote_style.is_none() && word.value.starts_with('@') => {
                if word.value.starts_with("@@") {
                    None
                } else {
                    Some(word.value.clone())
                }
            }
            _ => None,
        };
        if let Some(parameter) = parameter {
            let role = if system == DatabaseSystem::PostgreSQL && parameter.starts_with('$') {
                positional = true;
                match parameter.as_str() {
                    "$1" if !roles.is_empty() => 0,
                    "$2" if roles.len() == 2 => 1,
                    _ => return Err(SourceQueryError::Parameters),
                }
            } else {
                named = true;
                let prefix = if system == DatabaseSystem::SqlServer {
                    '@'
                } else {
                    ':'
                };
                let name = parameter
                    .strip_prefix(prefix)
                    .ok_or(SourceQueryError::Parameters)?;
                roles
                    .iter()
                    .position(|role| {
                        if system == DatabaseSystem::Oracle {
                            name.eq_ignore_ascii_case(role)
                        } else {
                            name == *role
                        }
                    })
                    .ok_or(SourceQueryError::Parameters)?
            };
            edits.push(Edit {
                range,
                text: native_bind(system, roles, role),
            });
        }
        index += 1;
    }
    if named && positional {
        return Err(SourceQueryError::Parameters);
    }
    edit_sql(sql, edits).map(|sql| sql.trim().to_owned())
}

fn unnest(mut expr: &Expr) -> &Expr {
    while let Expr::Nested(inner) = expr {
        expr = inner;
    }
    expr
}

fn identifiers(expr: &Expr) -> Option<&[Ident]> {
    match unnest(expr) {
        Expr::Identifier(id) => Some(std::slice::from_ref(id)),
        Expr::CompoundIdentifier(ids) => Some(ids),
        _ => None,
    }
}

fn same_ident(left: &Ident, right: &Ident, system: DatabaseSystem) -> bool {
    if system == DatabaseSystem::SqlServer {
        // SQL Server can use a case-sensitive collation; never infer alias equivalence.
        left.value == right.value
    } else {
        let fold = |id: &Ident| {
            if id.quote_style.is_some() {
                id.value.clone()
            } else if system == DatabaseSystem::Oracle {
                id.value.to_ascii_uppercase()
            } else {
                id.value.to_ascii_lowercase()
            }
        };
        fold(left) == fold(right)
    }
}

fn same_column(left: &Expr, right: &Expr, system: DatabaseSystem) -> bool {
    match (identifiers(left), identifiers(right)) {
        (Some(left), Some(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(a, b)| same_ident(a, b, system))
        }
        _ => false,
    }
}

fn named_projection(item: &SelectItem) -> Option<(&Expr, &Ident)> {
    match item {
        SelectItem::UnnamedExpr(expr) => identifiers(expr)
            .and_then(|ids| ids.last())
            .map(|name| (expr, name)),
        SelectItem::ExprWithAlias { expr, alias } => Some((expr, alias)),
        _ => None,
    }
}

fn projected_cursor<'a>(select: &'a Select, output: &str) -> Result<&'a Expr, SourceQueryError> {
    let mut matches = select
        .projection
        .iter()
        .filter_map(named_projection)
        .filter(|(_, name)| name.value.eq_ignore_ascii_case(output));
    let (expr, _) = matches.next().ok_or(SourceQueryError::Columns)?;
    if matches.next().is_some() || identifiers(expr).is_none() {
        return Err(SourceQueryError::Columns);
    }
    Ok(expr)
}

fn order_matches_cursor(order: &Expr, key: &Expr, select: &Select, system: DatabaseSystem) -> bool {
    if let Expr::Identifier(id) = order {
        // A bare ORDER BY name resolves output aliases before input columns.
        // Unknown native expression names must not silently shadow an input key.
        let mut output_match = false;
        let mut unknown_names = false;
        for item in &select.projection {
            let Some((expr, name)) = named_projection(item) else {
                unknown_names = true;
                continue;
            };
            if same_ident(id, name, system) {
                output_match = true;
                if !same_column(expr, key, system) {
                    return false;
                }
            }
        }
        if output_match {
            return true;
        }
        if unknown_names {
            return false;
        }
    }
    same_column(order, key, system)
}

fn binary_collation(expr: &Expr) -> Option<&Expr> {
    if let Expr::Collate { expr, collation } = unnest(expr)
        && collation.to_string().eq_ignore_ascii_case("BINARY")
    {
        return Some(unnest(expr));
    }
    None
}

fn parameter(expr: &Expr, expected: &str) -> bool {
    match unnest(expr) {
        Expr::Identifier(id) => id.quote_style.is_none() && id.value == expected,
        Expr::Value(value) => {
            matches!(&value.value, sqlparser::ast::Value::Placeholder(name) if name == expected)
        }
        _ => false,
    }
}

fn comparison(
    expr: &Expr,
    key: &Expr,
    op: Op,
    bind: &str,
    string_key: bool,
    system: DatabaseSystem,
) -> bool {
    let Expr::BinaryOp {
        left,
        op: actual,
        right,
    } = unnest(expr)
    else {
        return false;
    };
    if *actual != op {
        return false;
    }
    if string_key {
        return matches!((binary_collation(left), binary_collation(right)), (Some(left), Some(right))
            if same_column(left, key, system) && parameter(right, bind));
    }
    same_column(left, key, system) && parameter(right, bind)
}

fn is_guard(
    expr: &Expr,
    keys: &[&Expr],
    binds: &[String],
    string_key: bool,
    system: DatabaseSystem,
) -> bool {
    if keys.len() == 1 {
        return comparison(expr, keys[0], Op::Gt, &binds[0], string_key, system);
    }
    match unnest(expr) {
        Expr::BinaryOp {
            left,
            op: Op::Or,
            right,
        } => {
            comparison(left, keys[0], Op::Gt, &binds[0], false, system)
                && matches!(unnest(right), Expr::BinaryOp {left: equal, op: Op::And, right: greater}
                    if comparison(equal, keys[0], Op::Eq, &binds[0], false, system)
                    && comparison(greater, keys[1], Op::Gt, &binds[1], false, system))
        }
        Expr::BinaryOp {
            left,
            op: Op::Gt,
            right,
        } if system == DatabaseSystem::PostgreSQL => {
            matches!((unnest(left), unnest(right)), (Expr::Tuple(l), Expr::Tuple(r))
                if l.len() == 2 && r.len() == 2
                && same_column(&l[0], keys[0], system) && same_column(&l[1], keys[1], system)
                && parameter(&r[0], &binds[0]) && parameter(&r[1], &binds[1]))
        }
        _ => false,
    }
}

fn guards<'a>(
    expr: &'a Expr,
    keys: &[&Expr],
    binds: &[String],
    string_key: bool,
    system: DatabaseSystem,
    found: &mut Vec<&'a Expr>,
) {
    if is_guard(expr, keys, binds, string_key, system) {
        found.push(unnest(expr));
    } else if let Expr::BinaryOp {
        left,
        op: Op::And,
        right,
    } = unnest(expr)
    {
        guards(left, keys, binds, string_key, system, found);
        guards(right, keys, binds, string_key, system, found);
    }
}

fn compile_keyset(
    sql: &str,
    tokens: &[TokenWithSpan],
    query: &Query,
    names: &[&str],
    roles: &[&str],
    string_key: bool,
    system: DatabaseSystem,
    max_rows: usize,
) -> Result<String, SourceQueryError> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Err(SourceQueryError::Keyset);
    };
    if query.limit_clause.is_some() || query.fetch.is_some() || select.top.is_some() {
        return Err(SourceQueryError::Pagination);
    }
    let keys = names
        .iter()
        .map(|name| projected_cursor(select, name))
        .collect::<Result<Vec<_>, _>>()?;
    let binds = roles
        .iter()
        .enumerate()
        .map(|(i, _)| native_bind(system, roles, i))
        .collect::<Vec<_>>();
    let mut found = Vec::new();
    guards(
        select.selection.as_ref().ok_or(SourceQueryError::Keyset)?,
        &keys,
        &binds,
        string_key,
        system,
        &mut found,
    );
    if found.len() != 1 {
        return Err(SourceQueryError::Keyset);
    }
    let order = query.order_by.as_ref().ok_or(SourceQueryError::Ordering)?;
    let OrderByKind::Expressions(expressions) = &order.kind else {
        return Err(SourceQueryError::Ordering);
    };
    if order.interpolate.is_some() || expressions.len() != keys.len() {
        return Err(SourceQueryError::Ordering);
    }
    for (order, key) in expressions.iter().zip(&keys) {
        let expr = if string_key {
            binary_collation(&order.expr).ok_or(SourceQueryError::Ordering)?
        } else {
            unnest(&order.expr)
        };
        if !order_matches_cursor(expr, key, select, system)
            || !matches!(order.options.sort, None | Some(OrderBySort::Asc))
            || order.with_fill.is_some()
        {
            return Err(SourceQueryError::Ordering);
        }
    }
    let mut edits = Vec::new();
    if system == DatabaseSystem::SqlServer {
        let positions = offsets(sql);
        let significant: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_)))
            .collect();
        let mut depth = 0;
        let mut top = None;
        for (index, t) in significant.iter().enumerate() {
            if depth == 0 && is_word(&t.token, "SELECT") {
                let keyword = significant
                    .get(index + 1)
                    .filter(|t| is_word(&t.token, "DISTINCT") || is_word(&t.token, "ALL"))
                    .unwrap_or(t);
                top = Some(byte_range(&positions, keyword.span)?.end);
                break;
            }
            match t.token {
                Token::LParen => depth += 1,
                Token::RParen => depth -= 1,
                _ => {}
            }
        }
        let end = top.ok_or(SourceQueryError::Composition)?;
        edits.push(Edit {
            range: end..end,
            text: format!(" TOP (@P{})", roles.len() + 1),
        });
    }
    let mut result = edit_sql(sql, edits)?;
    if system == DatabaseSystem::PostgreSQL {
        result.push_str(&format!("\nLIMIT {}", max_rows + 1));
    }
    Ok(result)
}

fn is_word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

struct ReadOnly {
    system: DatabaseSystem,
    depth: usize,
}

fn locking_hint(expression: &Expr) -> bool {
    matches!(expression, Expr::Identifier(id) if [
        "NOLOCK", "READUNCOMMITTED", "UPDLOCK", "XLOCK", "HOLDLOCK", "TABLOCK", "TABLOCKX",
        "PAGLOCK", "ROWLOCK", "READPAST", "READCOMMITTEDLOCK", "REPEATABLEREAD", "SERIALIZABLE",
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
        if select.into.is_some() || select.projection.iter().any(|item| {
            matches!(item,SelectItem::ExprWithAlias {alias,..} if alias.quote_style.is_none() && alias.value.starts_with('@'))
        }) { return ControlFlow::Break(SourceQueryError::Statement); }
        ControlFlow::Continue(())
    }
    fn pre_visit_expr(&mut self, _: &Expr) -> ControlFlow<Self::Break> {
        self.depth += 1;
        if self.depth > MAX_NESTING {
            return ControlFlow::Break(SourceQueryError::Complexity);
        }
        ControlFlow::Continue(())
    }
    fn post_visit_expr(&mut self, _: &Expr) -> ControlFlow<Self::Break> {
        self.depth -= 1;
        ControlFlow::Continue(())
    }
    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<Self::Break> {
        if let TableFactor::Table { with_hints,args,.. } = table
            && (with_hints.iter().any(locking_hint) || (self.system == DatabaseSystem::SqlServer && args.as_ref().is_some_and(|args| {
                args.args.iter().any(|arg| matches!(arg,FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) if locking_hint(expr)))
            }))) { return ControlFlow::Break(SourceQueryError::Locking); }
        ControlFlow::Continue(())
    }
}

/// Redacted failures of the native query and checkpoint contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SourceQueryError {
    /// Missing or oversized query.
    #[error("query.statement must be nonempty and at most 16 KiB")]
    Size,
    /// Unparseable native syntax.
    #[error("query.statement could not be parsed in the selected database dialect")]
    Syntax,
    /// The parser cannot safely represent a native line-comment boundary.
    #[error(
        "use LF or CRLF line endings in SQL Server and Oracle -- comments; lone carriage returns cannot be validated safely"
    )]
    CommentLineEnding,
    /// Bounded parser work exceeded.
    #[error("query.statement must use at most 4096 SQL tokens and 128 nested expressions")]
    Complexity,
    /// Multiple operations or non-read-only statement.
    #[error(
        "query.statement must contain one read-only tabular query, without writes, INTO, or additional statements"
    )]
    Statement,
    /// Unknown, mixed, or snapshot parameters.
    #[error(
        "query.statement may use only the configured cursor parameters; do not mix PostgreSQL named and positional binds; snapshots have no binds"
    )]
    Parameters,
    /// No mandatory exclusive keyset guard.
    #[error(
        "incremental query.statement requires the configured strict keyset as a mandatory WHERE condition, optionally ANDed with business filters"
    )]
    Keyset,
    /// Paging conflicts with bounded fetching.
    #[error(
        "incremental query.statement must not contain a top-level TOP, LIMIT, OFFSET, or FETCH; the receiver supplies the page bound"
    )]
    Pagination,
    /// Missing or incompatible ascending order.
    #[error(
        "incremental query.statement requires ORDER BY the configured cursor columns ascending, with Oracle BINARY collation for string keys"
    )]
    Ordering,
    /// Lock or isolation overrides.
    #[error("query.statement must not request row locks or override collection isolation")]
    Locking,
    /// Unsupported query representation.
    #[error("query.statement contains a native clause that cannot be composed as a tabular result")]
    Composition,
    /// Unusable cursor/result names.
    #[error(
        "query results need unique nonempty names; incremental cursor columns must be explicitly selected column references (aliases are allowed)"
    )]
    Columns,
}

#[cfg(test)]
#[path = "sql_tests.rs"]
mod tests;
