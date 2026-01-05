// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    ConditionalDataExpression, ConditionalDataExpressionBranch, DataExpression, LogicalExpression,
    QueryLocation, TransformExpression,
};
use data_engine_parser_abstractions::{ParserError, ParserScope, to_query_location};
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    tabular_expressions::parse_tabular_expression_rule,
};

pub fn parse_if_else_expression(
    if_else_expression: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&if_else_expression);
    let mut conditional_expr = ConditionalDataExpression::new(query_location);

    // keep track of the location of the current branch
    let mut branch_loc_start = 0;
    let mut branch_loc_line = 0;
    let mut branch_log_col = 0;

    let mut next_condition: Option<LogicalExpression> = None;
    let mut next_branch: Vec<DataExpression> = Vec::new();

    let inner = if_else_expression.into_inner();
    for rule in inner {
        match rule.as_rule() {
            Rule::if_condition_expression => {
                branch_loc_start = rule.as_span().start();
                let (line_number, column_number) = rule.line_col();
                branch_loc_line = line_number;
                branch_log_col = column_number;

                // TODO no unwrap?
                let condition_rule = rule.into_inner().next().unwrap();
                let condition_expr = match condition_rule.as_rule() {
                    Rule::logical_expression => parse_logical_expression(condition_rule, scope)?,
                    // TODO no panic?
                    _ => panic!("Unexpected rule in if_condition_expression: {condition_rule}"),
                };
                next_condition = Some(condition_expr);
            }
            Rule::if_else_branch_expression => {
                let branch_loc_end = rule.as_span().end();
                for tabular_rule in rule.into_inner() {
                    let mut expr = parse_tabular_expression_rule(tabular_rule, scope)?;
                    next_branch.append(&mut expr);
                }
                let curr_branch = next_branch;
                next_branch = Vec::new();

                // TODO no unwrap
                let query_location = QueryLocation::new(
                    branch_loc_start,
                    branch_loc_end,
                    branch_loc_line,
                    branch_log_col,
                )
                .unwrap();

                let data_expr_branch = ConditionalDataExpressionBranch::new(
                    query_location,
                    next_condition.take().unwrap(),
                    curr_branch,
                );

                conditional_expr = conditional_expr.with_branch(data_expr_branch);
            }
            Rule::else_expression => {
                let mut else_branch_exprs = Vec::new();
                // TODO no unwrap
                // TODO check the type
                let branch_rules = rule.into_inner().next().unwrap();

                for tabular_rule in branch_rules.into_inner() {
                    let mut expr = parse_tabular_expression_rule(tabular_rule, scope)?;
                    else_branch_exprs.append(&mut expr);
                }
                conditional_expr = conditional_expr.with_default_branch(else_branch_exprs);
            }
            // TODO no panic?
            _ => panic!("Unexpected rule in if_else_expression: {rule}"),
        }
    }

    Ok(DataExpression::Conditional(conditional_expr))
}
