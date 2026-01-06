// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for parsing OPL programs
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState,
};
use pest::{Parser as _, RuleType, iterators::Pair};

mod query_expression;
mod tabular_expression;

#[derive(pest_derive::Parser)]
#[grammar = "../../../../experimental/query_engine/kql-parser/src/base.pest"]
#[grammar = "opl.pest"]
struct OplPestParser {}

/// Parser for OPL programs
pub struct OplParser {}

impl Parser for OplParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        let mut state = ParserState::new_with_options(query, options);

        let parse_result = OplPestParser::parse(Rule::query, query);

        let query_rules = parse_result.unwrap().next().unwrap().into_inner();

        todo!()
    }
}

pub fn parse_tabular_expression_rule<T: RuleType>(
    tabular_expression_rule: Pair<T>,
) -> Result<(), ParserError> {
    // let kql_rule: data_engine_kql_parser::Rule = tabular_expression_rule.into();
    // parse_tabular_expression_rule(kql_rule)
    Ok(())
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::Parser;

    use crate::parser::OplParser;

    #[test]
    fn test_olp_parser() {
        let result = OplParser::parse("logs | where x == 1");
        assert!(result.is_ok());
    }
}
