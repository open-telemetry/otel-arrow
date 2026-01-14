// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry Data Processing Language (OPL) parser implementation.

use std::vec;

use ::pest::{Parser as _, iterators::Pair};
use data_engine_expressions::QueryLocation;
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState, to_query_location,
};

mod assignment;
mod expression;
mod operator;
mod pipeline;

#[allow(missing_docs)]
mod pest {
    #[derive(pest_derive::Parser)]
    #[grammar = "opl.pest"]
    pub struct OplPestParser;
}

pub(crate) use pest::Rule;

use crate::parser::pipeline::parse_pipeline;

/// Parser for OPL programs.
pub struct OplParser;

impl Parser for OplParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        let parse_result = match pest::OplPestParser::parse(Rule::program, query) {
            Ok(rules) => rules,
            Err(pest_error) => {
                return Err(vec![ParserError::from_pest_error(query, pest_error)]);
            }
        };

        let mut state = ParserState::new_with_options(query, options);

        for rule in parse_result {
            match rule.as_rule() {
                Rule::program => {
                    if let Err(e) = parse_program(rule, &mut state) {
                        return Err(vec![e]);
                    }
                }

                invalid_rule => {
                    let query_location = to_query_location(&rule);
                    let err = ParserError::SyntaxError(
                        query_location,
                        format!("Invalid top-level rule. Expected program, found {invalid_rule:?}"),
                    );
                    return Err(vec![err]);
                }
            }
        }

        Ok(ParserResult::new(state.build()?))
    }
}

fn parse_program(rule: Pair<'_, Rule>, state: &mut ParserState) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::pipeline => parse_pipeline(rule, state)?,
            Rule::EOI => {}
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::program,
                    invalid_rule,
                ));
            }
        }
    }

    Ok(())
}

/// Helper to create a standardized error for invalid child rules.
pub(crate) fn invalid_child_rule_error(
    query_location: QueryLocation,
    parent_rule: Rule,
    found_rule: Rule,
) -> ParserError {
    ParserError::SyntaxError(
        query_location,
        format!("Invalid child rule found in {parent_rule:?} {found_rule:?}"),
    )
}

#[cfg(test)]
mod test {
    use data_engine_parser_abstractions::Parser;

    use super::OplParser;

    #[test]
    fn parser_smoke_test() {
        let result = OplParser::parse("logs | where 1 == \"y\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_invalid_syntax() {
        let result = OplParser::parse("logs | where ");
        assert!(result.is_err());

        let errors = result.err().unwrap();
        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        assert!(
            error
                .to_string()
                .contains("supplied in query is not supported")
        );
    }
}
