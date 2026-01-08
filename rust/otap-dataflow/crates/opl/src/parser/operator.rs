// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    DataExpression, DiscardDataExpression, LogicalExpression, NotLogicalExpression, QueryLocation,
};
use data_engine_parser_abstractions::{ParserError, ParserState, to_query_location};
use pest::iterators::Pair;

use crate::parser::Rule;
use crate::parser::expression::parse_expression;

pub(crate) fn parse_operator_call(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    // TODO -- this probably doesn't need to be a loop
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::where_operator_call => parse_where_operator_call(rule, state)?,
            _ => {
                todo!("invalid rule found in operator_call {rule:#?}")
            }
        };
    }
    
    Ok(())
}

pub(crate) fn parse_where_operator_call(
    operator_call_rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    let operator_call_query_location = to_query_location(&operator_call_rule);
    if let Some(rule) = operator_call_rule.into_inner().next() {
        match rule.as_rule() {
            Rule::expression => {
                let rule_query_location = to_query_location(&rule);
                let predicate = parse_expression(rule)?;
                let discard_expr = DiscardDataExpression::new(operator_call_query_location)
                    .with_predicate(LogicalExpression::Not(NotLogicalExpression::new(
                        rule_query_location,
                        predicate,
                    )));
                state.push_expression(DataExpression::Discard(discard_expr));
            }
            _ => {
                todo!("invalid rule in where operator call {rule:#?}")
            }
        }
    }

    Ok(())
}
