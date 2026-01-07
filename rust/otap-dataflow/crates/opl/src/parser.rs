// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for parsing OPL programs
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState,
};
use data_engine_parser_macros::{BaseRuleCompatible, ScalarExprPrattParser};
use pest::{Parser as _, RuleType, iterators::Pair};

use crate::parser::tabular_expression::parse_tabular_expression;

mod query_expression;
mod tabular_expression;

#[derive(pest_derive::Parser, BaseRuleCompatible, ScalarExprPrattParser)]
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
        let state = ParserState::new_with_options(query, options);
        let mut errors = Vec::new();

        let mut parser_rules = match OplPestParser::parse(Rule::query, query) {
            Ok(query_rules) => query_rules,
            Err(e) => {
                todo!()
            }
        };

        let query_rule = parser_rules.next().unwrap();
        for rule in query_rule.into_inner() {
            match rule.as_rule() {
                Rule::variable_definition_expression => {
                    todo!("handle variable expression")
                }
                Rule::tabular_expression => {
                    let expressions = match parse_tabular_expression(rule, &state) {
                        Ok(exprs) => exprs,
                        Err(e) => {
                            errors.push(e);
                            continue;
                        }
                    };
                    for expr in expressions {
                        state.push_expression(expr);
                    }
                }
                Rule::EOI => {}
                other => {
                    todo!("unexpected rule {other:#?}")
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ParserResult::new(state.build()?))
    }
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::Parser;

    use crate::parser::OplParser;

    #[test]
    fn test_olp_parser() {
        let result = OplParser::parse("logs | where x == 1; logs | where x == 2");
        // let result = OplParser::parse("");
        assert!(result.is_ok());

        println!("result: {:#?}", result.unwrap());
    }
}
