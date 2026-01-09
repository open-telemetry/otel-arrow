// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TODO

// TODO should this just have a result type?
use ::pest::{Parser as _, iterators::Pair};
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState,
};

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

/// TODO
pub struct OplParser;

impl Parser for OplParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        let parse_result = match pest::OplPestParser::parse(Rule::program, query) {
            Ok(rules) => rules,
            Err(e) => {
                // TODO generic handling of parser error in parser abstraction
                todo!("handle error {e}")
            }
        };

        let mut state = ParserState::new_with_options(query, options);

        for rule in parse_result {
            match rule.as_rule() {
                Rule::program => {
                    if let Err(e) = parse_program(rule, &mut state) {
                        todo!("handle error {e}")
                    }
                }

                _ => {
                    todo!("handle invalid query rule {rule:?}")
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
            _ => {
                todo!("handle invalid rule program rule {rule:?}")
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use data_engine_parser_abstractions::Parser;

    use super::OplParser;

    // a better test name
    #[test]
    fn smoke_test_parser() {
        let result = OplParser::parse("logs | where 1 == \"y\"");
        println!("{:#?}", result.unwrap().pipeline)
    }
}
