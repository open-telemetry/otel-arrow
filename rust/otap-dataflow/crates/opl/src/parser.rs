// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Utilities for parsing OPL programs
//!
use data_engine_expressions::QueryLocation;
use data_engine_kql_parser::map_parse_error;
use data_engine_kql_parser_macros::BaseRuleCompatible;
use data_engine_parser_abstractions::{
    Parser, ParserError, ParserOptions, ParserResult, ParserState, to_query_location,
};
use pest::Parser as _;

use crate::parser::tabular_expression::parse_tabular_expression;

mod tabular_expression;

#[derive(pest_derive::Parser, BaseRuleCompatible)]
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

        // parse query
        let mut parser_rules = match OplPestParser::parse(Rule::query, query) {
            Ok(query_rules) => query_rules,
            Err(pest_error) => {
                errors.push(map_parse_error(query, pest_error));
                return Err(errors);
            }
        };

        // get the main query rule
        let query_rule = match parser_rules.next() {
            Some(rule) => rule,
            None => {
                // query is invalid at the start
                let query_location =
                    QueryLocation::new(1, query.len(), 1, 1).expect("valid query location");

                errors.push(ParserError::SyntaxError(
                    query_location,
                    "No query found".to_string(),
                ));
                return Err(errors);
            }
        };

        // build pipeline from query rules
        for rule in query_rule.into_inner() {
            match rule.as_rule() {
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
                other => errors.push(ParserError::SyntaxError(
                    to_query_location(&rule),
                    format!("Unexpected rule in OPL query: {:?}", other),
                )),
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
    use data_engine_expressions::{
        DataExpression, DiscardDataExpression, GreaterThanLogicalExpression,
        IntegerScalarExpression, LogicalExpression, NotLogicalExpression, QueryLocation,
        ScalarExpression, SourceScalarExpression, StaticScalarExpression, StringScalarExpression,
        ValueAccessor,
    };
    use data_engine_kql_parser::Parser;

    use crate::parser::OplParser;

    #[test]
    fn test_olp_parser() {
        // smoke test to ensure we can parse transform expressions from the query
        let result = OplParser::parse("logs | where severity_number > 0");
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = &pipeline.get_expressions();
        assert_eq!(
            expressions,
            &[DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "severity_number",
                                    ))
                                ),])
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                            )),
                        )),
                    )),
                ),
            ),]
        )
    }

    #[test]
    fn test_parse_empty_query() {
        let result = OplParser::parse("");
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = &pipeline.get_expressions();
        assert_eq!(expressions.len(), 0);
    }
}
