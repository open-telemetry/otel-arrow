// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::DataExpression;
use data_engine_kql_parser::{
    ParserError, tabular_expressions::parse_common_tabular_expression_rule,
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
    // identifier is "logs" which is not currently used for anything.
    let _ = rules.next();

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
    // TODO handle custom tabular expressions here
    parse_common_tabular_expression_rule(tabular_expression_rule, scope)
}
