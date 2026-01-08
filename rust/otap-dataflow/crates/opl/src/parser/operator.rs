// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_parser_abstractions::ParserError;
use pest::iterators::Pair;

use crate::parser::Rule;

pub(crate) fn parse_operator_call(
    rule: Pair<'_, Rule>
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::where_operator_call => parse_where_operator_call(rule)?,
            _ => {
                todo!("invalid rule found in operator_call {rule:#?}")
            }
        }
    }
    todo!()
}

pub (crate) fn parse_where_operator_call(
    rule: Pair<'_, Rule>
) -> Result<(), ParserError> {
    println!("rule: {rule:#?}");
    todo!()
}