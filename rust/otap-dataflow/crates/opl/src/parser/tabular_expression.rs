// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::DataExpression;
use data_engine_kql_parser::{
    ParserError,
    tabular_expressions::{
        parse_extend_expression, parse_project_away_expression, parse_project_expression,
        parse_project_keep_expression, parse_project_rename_expression, parse_where_expression,
    },
};
use data_engine_parser_abstractions::ParserScope;
use pest::iterators::Pair;

use crate::parser::Rule;

pub(crate) fn parse_tabular_expression(
    tabular_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<DataExpression>, ParserError> {
    let mut rules = tabular_expression_rule.into_inner();

    // Note: This is the identifier. In a query like logs | extend a=b the
    // indentifier is "logs" which is not currently used for anything.
    let _ = rules.next().unwrap();

    let mut expressions = Vec::new();

    for rule in rules {
        for e in parse_tabular_expression_rule(rule, scope)? {
            expressions.push(e);
        }
    }

    Ok(expressions)
}

pub(crate) fn parse_tabular_expression_rule(
    tabular_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<Vec<DataExpression>, ParserError> {
    let mut expressions = Vec::new();

    match tabular_expression_rule.as_rule() {
        Rule::extend_expression => {
            let extend_expressions = parse_extend_expression(tabular_expression_rule, scope)?;

            for e in extend_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_expression => {
            let project_expressions = parse_project_expression(tabular_expression_rule, scope)?;

            for e in project_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_keep_expression => {
            let project_keep_expressions =
                parse_project_keep_expression(tabular_expression_rule, scope)?;

            for e in project_keep_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_away_expression => {
            let project_away_expressions =
                parse_project_away_expression(tabular_expression_rule, scope)?;

            for e in project_away_expressions {
                expressions.push(DataExpression::Transform(e));
            }
        }
        Rule::project_rename_expression => expressions.push(DataExpression::Transform(
            parse_project_rename_expression(tabular_expression_rule, scope)?,
        )),
        Rule::where_expression => {
            expressions.push(parse_where_expression(tabular_expression_rule, scope)?)
        }
        _ => panic!("Unexpected rule in tabular_expression: {tabular_expression_rule}"),
    }

    Ok(expressions)
}
