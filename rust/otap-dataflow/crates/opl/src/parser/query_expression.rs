// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_kql_parser::ParserError;
use data_engine_parser_abstractions::ParserState;
use pest::iterators::Pair;

use crate::parser::Rule;
use crate::parser::tabular_expression::parse_tabular_expression;

pub(crate) fn parse_query_expression(
    query_expression: Pair<Rule>,
    parser_state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in query_expression.into_inner() {
        match rule.as_rule() {
            Rule::variable_definition_expression => {
                todo!("handle variable expression")
            }
            Rule::tabular_expression => {
                let expressions = parse_tabular_expression(rule, parser_state)?;
                for expr in expressions {
                    parser_state.push_expression(expr);
                }
            }
            // TODO confirm if this is reachable ...
            Rule::EOI => {}

            other => {
                todo!("unexpected rule {other:#?}")
            }
        }
    }

    Ok(())
}
