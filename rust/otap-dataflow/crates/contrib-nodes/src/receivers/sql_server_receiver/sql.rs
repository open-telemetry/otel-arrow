// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Fail-closed T-SQL subset and parameterized server-side paging.

use super::config::{SqlServerConfigError, error};
use otel_arrow_dfe_scraper::database::{MAX_QUERY_BYTES, ScalarValue, WatermarkConfig};
use sqlparser::{
    ast::{
        BinaryOperator, CastKind, DataType, ExactNumberInfo, Expr, Function, FunctionArg,
        FunctionArgExpr, FunctionArgumentList, FunctionArguments, GroupByExpr, Ident, ObjectName,
        OrderBy, OrderByExpr, OrderByKind, OrderByOptions, Query, Select, SelectFlavor, SelectItem,
        SetExpr, Statement, TableFactor, UnaryOperator, Value,
    },
    dialect::MsSqlDialect,
    parser::Parser,
    tokenizer::{Token, Tokenizer, Whitespace},
};
use std::{collections::HashSet, fmt};

const MAX_TOKENS: usize = 2048;
const MAX_DEPTH: usize = 32;
const MAX_COLUMNS: usize = 256;

/// Source of one native parameter, in positional driver binding order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SqlServerParameter {
    /// The scalar signed-integer cursor.
    ScalarInt64,
    /// The scalar or composite datetime2 cursor.
    Timestamp,
    /// The composite signed-integer tie-breaker.
    TieBreaker,
    /// The page-row limit, or snapshot limit plus one.
    PageLimit,
}

/// Validated SQL and a driver binding plan, not proof of live metadata or source safety.
pub struct SqlServerStatement {
    canonical_sql: String,
    driver_sql: String,
    parameters: Vec<SqlServerParameter>,
}

impl fmt::Debug for SqlServerStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlServerStatement")
            .field("sql", &"<redacted>")
            .field("parameters", &self.parameters)
            .finish()
    }
}

impl SqlServerStatement {
    /// Returns canonical operator SQL for the shared plan and compatibility fingerprint.
    #[must_use]
    pub fn canonical_sql(&self) -> &str {
        &self.canonical_sql
    }

    /// Returns SQL with a generated TOP bound and native @P1, @P2, ... parameters.
    ///
    /// All values must be supplied through the driver's typed parameter API.
    #[must_use]
    pub fn driver_sql(&self) -> &str {
        &self.driver_sql
    }

    /// Returns native parameter roles in binding order, without customer values.
    #[must_use]
    pub fn parameters(&self) -> &[SqlServerParameter] {
        &self.parameters
    }

    pub(super) fn compile(
        sql: &str,
        watermark: &WatermarkConfig,
    ) -> Result<Self, SqlServerConfigError> {
        let (query, tokens) = parse(sql)?;
        let select = validate_shape(&query)?;
        let snapshot = matches!(watermark, WatermarkConfig::Snapshot {});
        validate_projections(select, snapshot)?;
        let mut parameters = Vec::new();
        let mut binds = Vec::new();
        match watermark {
            WatermarkConfig::Snapshot {} => validate_snapshot(select, &query)?,
            WatermarkConfig::Scalar {
                column,
                bind,
                initial,
            } => {
                validate_keyset(select, &query, &[column])?;
                let predicate = select.selection.as_ref().ok_or_else(unsupported)?;
                if !comparison(predicate, column, BinaryOperator::Gt, bind) {
                    return Err(keyset_error());
                }
                parameters.push(match initial {
                    ScalarValue::Int64(_) => SqlServerParameter::ScalarInt64,
                    ScalarValue::Timestamp(_) => SqlServerParameter::Timestamp,
                    _ => return Err(keyset_error()),
                });
                binds.push(bind.as_str());
            }
            WatermarkConfig::Composite {
                timestamp,
                tie_breaker,
            } => {
                validate_keyset(select, &query, &[&timestamp.column, &tie_breaker.column])?;
                let predicate = select.selection.as_ref().ok_or_else(unsupported)?;
                let Expr::BinaryOp {
                    left,
                    op: BinaryOperator::Or,
                    right,
                } = unnest(predicate)
                else {
                    return Err(keyset_error());
                };
                let Expr::BinaryOp {
                    left: equal,
                    op: BinaryOperator::And,
                    right: greater,
                } = unnest(right)
                else {
                    return Err(keyset_error());
                };
                if !comparison(left, &timestamp.column, BinaryOperator::Gt, &timestamp.bind)
                    || !comparison(
                        equal,
                        &timestamp.column,
                        BinaryOperator::Eq,
                        &timestamp.bind,
                    )
                    || !comparison(
                        greater,
                        &tie_breaker.column,
                        BinaryOperator::Gt,
                        &tie_breaker.bind,
                    )
                {
                    return Err(keyset_error());
                }
                parameters.extend([
                    SqlServerParameter::Timestamp,
                    SqlServerParameter::TieBreaker,
                ]);
                binds.extend([timestamp.bind.as_str(), tie_breaker.bind.as_str()]);
            }
        }
        let canonical_sql = render(&tokens, None)?;
        if canonical_sql.len() > MAX_QUERY_BYTES {
            return Err(error(format!(
                "canonical query.statement must not exceed {MAX_QUERY_BYTES} bytes"
            )));
        }
        parameters.push(SqlServerParameter::PageLimit);
        Ok(Self {
            canonical_sql,
            driver_sql: render(&tokens, Some(&binds))?,
            parameters,
        })
    }
}

fn parse(sql: &str) -> Result<(Box<Query>, Vec<Token>), SqlServerConfigError> {
    if sql.len() > MAX_QUERY_BYTES {
        return Err(error(format!(
            "query.statement must not exceed {MAX_QUERY_BYTES} bytes"
        )));
    }
    let dialect = MsSqlDialect {};
    let tokens = Tokenizer::new(&dialect, sql)
        .tokenize()
        .map_err(|_| error("query.statement is not valid supported T-SQL"))?;
    let mut tokens = tokens
        .into_iter()
        .filter(|token| {
            !matches!(
                token,
                Token::Whitespace(Whitespace::Space | Whitespace::Newline | Whitespace::Tab)
            )
        })
        .collect::<Vec<_>>();
    if tokens.len() > MAX_TOKENS
        || tokens
            .iter()
            .any(|token| matches!(token, Token::Whitespace(_)))
    {
        return Err(error(
            "query.statement must use at most 2048 tokens and no comments",
        ));
    }
    if matches!(tokens.last(), Some(Token::SemiColon)) {
        _ = tokens.pop();
    }
    // The parser normalizes == to equality, but token rendering preserves its invalid spelling.
    if tokens.iter().any(|token| matches!(token, Token::DoubleEq)) {
        return Err(unsupported());
    }
    if tokens.iter().any(|token| matches!(token, Token::SemiColon))
        || !matches!(tokens.first(), Some(Token::Word(word)) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("SELECT"))
    {
        return Err(error("query.statement must contain exactly one SELECT"));
    }
    if matches!(tokens.get(1), Some(Token::Word(word)) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("ALL"))
    {
        return Err(unsupported());
    }
    let mut parser = Parser::new(&dialect)
        .with_recursion_limit(MAX_DEPTH)
        .with_tokens(tokens.clone());
    let mut statements = parser
        .parse_statements()
        .map_err(|_| error("query.statement is invalid, unsupported, or too deeply nested"))?;
    // parse_statements can stop at END before consuming the complete input.
    if statements.len() != 1 || !matches!(parser.peek_token_ref().token, Token::EOF) {
        return Err(error("query.statement must contain exactly one SELECT"));
    }
    match statements.pop() {
        Some(Statement::Query(query)) => Ok((query, tokens)),
        _ => Err(unsupported()),
    }
}

fn validate_shape(query: &Query) -> Result<&Select, SqlServerConfigError> {
    let Query {
        with,
        body,
        order_by: _,
        limit_clause,
        fetch,
        locks,
        for_clause,
        settings,
        format_clause,
        pipe_operators,
    } = query;
    if with.is_some()
        || limit_clause.is_some()
        || fetch.is_some()
        || !locks.is_empty()
        || for_clause.is_some()
        || settings.is_some()
        || format_clause.is_some()
        || !pipe_operators.is_empty()
    {
        return Err(unsupported());
    }
    let SetExpr::Select(select) = body.as_ref() else {
        return Err(unsupported());
    };
    let Select {
        select_token: _,
        optimizer_hints,
        distinct,
        select_modifiers,
        top,
        top_before_distinct: _,
        projection: _,
        exclude,
        into,
        from,
        lateral_views,
        prewhere,
        selection: _,
        connect_by,
        group_by: _,
        cluster_by,
        distribute_by,
        sort_by,
        having: _,
        named_window,
        qualify,
        window_before_qualify: _,
        value_table_mode,
        flavor,
    } = select.as_ref();
    if !optimizer_hints.is_empty()
        || distinct.is_some()
        || select_modifiers.is_some()
        || top.is_some()
        || exclude.is_some()
        || into.is_some()
        || !lateral_views.is_empty()
        || prewhere.is_some()
        || !connect_by.is_empty()
        || !cluster_by.is_empty()
        || !distribute_by.is_empty()
        || !sort_by.is_empty()
        || !named_window.is_empty()
        || qualify.is_some()
        || value_table_mode.is_some()
        || *flavor != SelectFlavor::Standard
        || from.len() != 1
        || !from[0].joins.is_empty()
    {
        return Err(unsupported());
    }
    let TableFactor::Table {
        name,
        alias,
        args,
        with_hints,
        version,
        with_ordinality,
        partitions,
        json_path,
        sample,
        index_hints,
    } = &from[0].relation
    else {
        return Err(unsupported());
    };
    if name.0.len() != 2
        || alias.is_some()
        || args.is_some()
        || !with_hints.is_empty()
        || version.is_some()
        || *with_ordinality
        || !partitions.is_empty()
        || json_path.is_some()
        || sample.is_some()
        || !index_hints.is_empty()
    {
        return Err(error(
            "query.statement requires one schema-qualified table or view without aliases, joins, arguments, or hints",
        ));
    }
    validate_object_name(name)?;
    Ok(select)
}

fn validate_projections(select: &Select, snapshot: bool) -> Result<(), SqlServerConfigError> {
    if select.projection.is_empty() || select.projection.len() > MAX_COLUMNS {
        return Err(error(
            "query.statement must explicitly select 1..256 named columns",
        ));
    }
    let mut names = HashSet::new();
    for projection in &select.projection {
        let (expr, name) = match projection {
            SelectItem::UnnamedExpr(expr @ Expr::Identifier(name)) => (expr, name),
            SelectItem::ExprWithAlias { expr, alias } => (expr, alias),
            _ => {
                return Err(error(
                    "selected expressions require explicit result names; wildcards are unsupported",
                ));
            }
        };
        validate_ident(name)?;
        if !names.insert(name.value.to_ascii_lowercase()) {
            return Err(error(
                "query.statement contains duplicate result column names",
            ));
        }
        validate_expr(expr, snapshot, 0)?;
    }
    Ok(())
}

fn validate_keyset(
    select: &Select,
    query: &Query,
    columns: &[&String],
) -> Result<(), SqlServerConfigError> {
    if !matches!(&select.group_by, GroupByExpr::Expressions(exprs, modifiers) if exprs.is_empty() && modifiers.is_empty())
        || select.having.is_some()
    {
        return Err(keyset_error());
    }
    for column in columns {
        if !select.projection.iter().any(|projection| {
            matches!(projection, SelectItem::UnnamedExpr(Expr::Identifier(id)) if id.value == **column)
        }) {
            return Err(error("cursor columns must be selected directly, without aliases, using their configured spelling"));
        }
    }
    let OrderBy { kind, interpolate } = query.order_by.as_ref().ok_or_else(keyset_error)?;
    let OrderByKind::Expressions(expressions) = kind else {
        return Err(keyset_error());
    };
    if interpolate.is_some()
        || expressions.len() != columns.len()
        || expressions.iter().zip(columns).any(|(order, column)| {
            let OrderByExpr {
                expr,
                options: OrderByOptions { asc, nulls_first },
                with_fill,
            } = order;
            !matches!(expr, Expr::Identifier(id) if id.value == **column && validate_ident(id).is_ok())
                || *asc == Some(false)
                || nulls_first.is_some()
                || with_fill.is_some()
        })
    {
        return Err(keyset_error());
    }
    Ok(())
}

fn comparison(expr: &Expr, column: &str, operator: BinaryOperator, bind: &str) -> bool {
    matches!(unnest(expr), Expr::BinaryOp { left, op, right }
        if *op == operator
            && matches!(unnest(left), Expr::Identifier(id) if id.value == column && validate_ident(id).is_ok())
            && is_bind(unnest(right), bind))
}

fn is_bind(expr: &Expr, bind: &str) -> bool {
    match expr {
        Expr::Identifier(id) if id.quote_style.is_none() => {
            id.value.strip_prefix('@') == Some(bind)
        }
        Expr::Value(value) => {
            matches!(&value.value, Value::Placeholder(value) if value.strip_prefix('@') == Some(bind))
        }
        _ => false,
    }
}

fn unnest(mut expr: &Expr) -> &Expr {
    while let Expr::Nested(inner) = expr {
        expr = inner;
    }
    expr
}

fn render(tokens: &[Token], binds: Option<&[&str]>) -> Result<String, SqlServerConfigError> {
    let mut rendered = Vec::with_capacity(tokens.len() + 1);
    for (index, token) in tokens.iter().enumerate() {
        // sqlparser's national-string Display does not escape embedded quotes.
        // Render validated tokens explicitly rather than changing literal semantics.
        rendered.push(match token {
            Token::SingleQuotedString(value) => format!("'{}'", value.replace('\'', "''")),
            Token::NationalStringLiteral(value) => format!("N'{}'", value.replace('\'', "''")),
            Token::Word(word) if word.quote_style == Some('[') => {
                format!("[{}]", word.value.replace(']', "]]"))
            }
            Token::Word(word) if word.quote_style.is_none() && word.value.starts_with('@') => {
                render_bind(&word.value, binds)?
            }
            Token::Placeholder(value) => render_bind(value, binds)?,
            _ => token.to_string(),
        });
        if index == 0
            && let Some(binds) = binds
        {
            rendered.push(format!("TOP (@P{})", binds.len() + 1));
        }
    }
    Ok(rendered.join(" "))
}

fn render_bind(value: &str, binds: Option<&[&str]>) -> Result<String, SqlServerConfigError> {
    let Some(binds) = binds else {
        return Ok(value.to_owned());
    };
    let index = binds
        .iter()
        .position(|bind| value.strip_prefix('@') == Some(*bind))
        .ok_or_else(keyset_error)?;
    Ok(format!("@P{}", index + 1))
}

fn validate_snapshot(select: &Select, query: &Query) -> Result<(), SqlServerConfigError> {
    for expr in [&select.selection, &select.having].into_iter().flatten() {
        validate_expr(expr, true, 0)?;
    }
    match &select.group_by {
        GroupByExpr::Expressions(expressions, modifiers) if modifiers.is_empty() => {
            for expr in expressions {
                validate_expr(expr, false, 0)?;
            }
        }
        _ => return Err(unsupported()),
    }
    if let Some(OrderBy { kind, interpolate }) = &query.order_by {
        let OrderByKind::Expressions(expressions) = kind else {
            return Err(unsupported());
        };
        if interpolate.is_some() {
            return Err(unsupported());
        }
        for OrderByExpr {
            expr,
            options:
                OrderByOptions {
                    asc: _,
                    nulls_first,
                },
            with_fill,
        } in expressions
        {
            if nulls_first.is_some() || with_fill.is_some() {
                return Err(unsupported());
            }
            validate_expr(expr, true, 0)?;
        }
    }
    Ok(())
}

fn validate_expr(expr: &Expr, aggregates: bool, depth: usize) -> Result<(), SqlServerConfigError> {
    if depth >= MAX_DEPTH {
        return Err(error("query.statement expressions are too deeply nested"));
    }
    let next = |expr| validate_expr(expr, aggregates, depth + 1);
    match expr {
        Expr::Identifier(id) => validate_ident(id),
        Expr::Value(value)
            if matches!(
                value.value,
                Value::Number(_, _)
                    | Value::SingleQuotedString(_)
                    | Value::NationalStringLiteral(_)
                    | Value::Null
            ) =>
        {
            Ok(())
        }
        Expr::Nested(inner) | Expr::IsNull(inner) | Expr::IsNotNull(inner) => next(inner),
        Expr::UnaryOp {
            op: UnaryOperator::Plus | UnaryOperator::Minus | UnaryOperator::Not,
            expr,
        } => next(expr),
        Expr::BinaryOp {
            left,
            op:
                BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Multiply
                | BinaryOperator::Divide
                | BinaryOperator::Modulo
                | BinaryOperator::Eq
                | BinaryOperator::NotEq
                | BinaryOperator::Gt
                | BinaryOperator::GtEq
                | BinaryOperator::Lt
                | BinaryOperator::LtEq
                | BinaryOperator::And
                | BinaryOperator::Or,
            right,
        } => {
            next(left)?;
            next(right)
        }
        Expr::Function(function) => validate_function(function, aggregates, depth + 1),
        Expr::Cast {
            kind: CastKind::Cast,
            expr,
            data_type: DataType::Decimal(ExactNumberInfo::PrecisionAndScale(precision, scale)),
            array: false,
            format: None,
        } if (1..=38).contains(precision)
            && u64::try_from(*scale).is_ok_and(|scale| scale <= *precision) =>
        {
            next(expr)
        }
        _ => Err(unsupported()),
    }
}

fn validate_function(
    function: &Function,
    aggregates: bool,
    depth: usize,
) -> Result<(), SqlServerConfigError> {
    let Function {
        name,
        uses_odbc_syntax,
        parameters,
        args,
        filter,
        null_treatment,
        over,
        within_group,
    } = function;
    if name.0.len() != 1
        || *uses_odbc_syntax
        || !matches!(parameters, FunctionArguments::None)
        || filter.is_some()
        || null_treatment.is_some()
        || over.is_some()
        || !within_group.is_empty()
    {
        return Err(unsupported());
    }
    let Some(name) = name.0[0].as_ident() else {
        return Err(unsupported());
    };
    if name.quote_style.is_some() {
        return Err(unsupported());
    }
    let name = name.value.to_ascii_uppercase();
    let aggregate = matches!(
        name.as_str(),
        "COUNT" | "COUNT_BIG" | "MIN" | "MAX" | "SUM" | "AVG"
    );
    if (aggregate && !aggregates)
        || (!aggregate
            && !matches!(
                name.as_str(),
                "ABS" | "COALESCE" | "ISNULL" | "LOWER" | "UPPER" | "LEN" | "DATALENGTH"
            ))
    {
        return Err(unsupported());
    }
    let FunctionArguments::List(FunctionArgumentList {
        duplicate_treatment,
        args,
        clauses,
    }) = args
    else {
        return Err(unsupported());
    };
    if duplicate_treatment.is_some() || !clauses.is_empty() {
        return Err(unsupported());
    }
    let valid_arity = match name.as_str() {
        "COALESCE" => args.len() >= 2,
        "ISNULL" => args.len() == 2,
        _ => args.len() == 1,
    };
    if !valid_arity {
        return Err(error(
            "query.statement uses an unsupported function argument count",
        ));
    }
    for arg in args {
        match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => {
                validate_expr(expr, aggregates && !aggregate, depth)?;
            }
            FunctionArg::Unnamed(FunctionArgExpr::Wildcard)
                if matches!(name.as_str(), "COUNT" | "COUNT_BIG") => {}
            _ => return Err(unsupported()),
        }
    }
    Ok(())
}

fn validate_object_name(name: &ObjectName) -> Result<(), SqlServerConfigError> {
    for part in &name.0 {
        validate_ident(part.as_ident().ok_or_else(unsupported)?)?;
    }
    Ok(())
}

fn validate_ident(id: &Ident) -> Result<(), SqlServerConfigError> {
    validate_column_name(&id.value)?;
    if id.quote_style.is_some_and(|quote| quote != '[') {
        return Err(error(
            "query.statement supports regular or bracketed identifiers only",
        ));
    }
    Ok(())
}

pub(super) fn validate_column_name(value: &str) -> Result<(), SqlServerConfigError> {
    if value.trim().is_empty()
        || value.encode_utf16().count() > 128
        || value.chars().any(char::is_control)
        || value.starts_with(['@', '#'])
    {
        return Err(error(
            "SQL Server identifiers must contain 1..128 UTF-16 units, no control characters, and no variable or temporary-table prefix",
        ));
    }
    Ok(())
}

fn keyset_error() -> SqlServerConfigError {
    error(
        "query.statement requires the strict configured keyset predicate and matching ascending ORDER BY",
    )
}

fn unsupported() -> SqlServerConfigError {
    error("query.statement uses unsupported SQL Server syntax")
}
