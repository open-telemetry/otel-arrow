// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use ::pest::Parser as _;
use ::pest::iterators::Pair;
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState, to_query_location,
};

use crate::ottl::editor_expression::parse_editor_expression;

#[allow(missing_docs)]
mod pest {
    #[derive(pest_derive::Parser)]
    #[grammar = "ottl/ottl.pest"]
    pub(crate) struct OttlPestParser;
}

pub(crate) use pest::OttlPestParser;
pub use pest::Rule;

/// Parser for OTTL Programs
pub struct OttlParser;

impl Parser for OttlParser {
    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<ParserResult, Vec<ParserError>> {
        let parse_result = match OttlPestParser::parse(Rule::program, query) {
            Ok(rules) => rules,
            Err(pest_error) => return Err(vec![ParserError::from_pest_error(query, pest_error)]),
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

fn parse_program(program_rule: Pair<'_, Rule>, state: &mut ParserState) -> Result<(), ParserError> {
    for rule in program_rule.into_inner() {
        match rule.as_rule() {
            Rule::statement_expression => {
                parse_statement_expression(rule, state)?;
            }
            Rule::EOI => {}
            _ => {
                let query_location = to_query_location(&rule);
                return Err(ParserError::SyntaxError(
                    query_location,
                    format!("Invalid child rule found in {:?} {rule:?}", Rule::program),
                ));
            }
        }
    }

    Ok(())
}

fn parse_statement_expression(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        if matches!(rule.as_rule(), Rule::editor_expression) {
            parse_editor_expression(rule, state)?;
        }
    }

    Ok(())
}
