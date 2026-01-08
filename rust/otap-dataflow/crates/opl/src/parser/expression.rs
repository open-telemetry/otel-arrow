// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::parser::Rule;
use data_engine_expressions::{
    LogicalExpression, OrLogicalExpression, QueryLocation, ScalarExpression,
    StaticScalarExpression, StringScalarExpression,
};
use data_engine_parser_abstractions::{ParserError, to_query_location};
use pest::iterators::{Pair, Pairs};

fn parse_next_child_rule<F>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    child_parser: F,
) -> Result<LogicalExpression, ParserError>
where
    F: Fn(Pair<'_, Rule>) -> Result<LogicalExpression, ParserError>,
{
    if let Some(rule) = rules.next() {
        if rule.as_rule() == expected_rule {
            child_parser(rule)
        } else {
            todo!("invalid rule")
        }
    } else {
        todo!("no child rule")
    }
}

fn parse_maybe_right<F>(
    rules: &mut Pairs<'_, Rule>,
    expected_rule: Rule,
    parser: F,
) -> Option<Result<LogicalExpression, ParserError>>
where
    F: Fn(Pair<'_, Rule>) -> Result<LogicalExpression, ParserError>,
{
    let right_rule = rules.next()?;
    if right_rule.as_rule() == expected_rule {
        Some(parser(right_rule))
    } else {
        todo!("invalid right rule")
    }
}

pub(crate) fn parse_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    println!("expression rule {:#?}", rule);
    let mut inner_rules = rule.into_inner();
    parse_next_child_rule(&mut inner_rules, Rule::or_expression, parse_or_expression)
}

pub(crate) fn parse_or_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    let left_expr =
        parse_next_child_rule(&mut inner_rules, Rule::and_expression, parse_and_expression)?;

    let maybe_right = parse_maybe_right(&mut inner_rules, Rule::or_expression, parse_or_expression)
        .transpose()?;

    Ok(match maybe_right {
        Some(right_expr) => LogicalExpression::Or(OrLogicalExpression::new(
            query_location,
            left_expr,
            right_expr,
        )),
        None => left_expr,
    })
}

pub(crate) fn parse_and_expression(rule: Pair<'_, Rule>) -> Result<LogicalExpression, ParserError> {
    // TODO for real
    Ok(LogicalExpression::Scalar(ScalarExpression::Static(
        StaticScalarExpression::String(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "hello",
        )),
    )))
}
