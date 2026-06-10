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
mod temporal;

#[allow(missing_docs)]
mod pest {
    #[derive(pest_derive::Parser)]
    #[grammar = "opl/opl.pest"]
    pub struct OplPestParser;
}

pub(crate) use pest::Rule;

use crate::opl::parser::pipeline::parse_pipeline;

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
    use data_engine_expressions::{
        DataExpression, DiscardDataExpression, EqualToLogicalExpression, LogicalExpression,
        NotLogicalExpression, QueryLocation, ScalarExpression, SourceScalarExpression,
        StaticScalarExpression, StringScalarExpression, ValueAccessor,
    };
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

    #[test]
    fn test_parse_comments() {
        let programs = [
            // simple with inline comment
            "
            // inline comment
            logs | where severity_text == \"ERROR\"
            ",
            // many inline comments
            "
            // inline comment
            // adjacent line inline comment
            logs | 
            // yet another inline comment before operator
            where severity_text == \"ERROR\"  // comment end of line
            // comment End of program
            ",
            // simple block comment
            "
            /*
              block comment
            */
            logs | where severity_text == \"ERROR\"
            ",
            // many block comments
            "
            /* start of program comment */
            logs /* end of line comment */ 
            /* start of line comment */ | where severity_text /* another */ == \"ERROR\" /* hello */
            /* 
            end of program comment 
            */
            ",
        ];

        fn gen_simple_equals_filter_expected(source: &str, str: &str) -> DataExpression {
            DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        source,
                                    )),
                                )]),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), str),
                            )),
                            false,
                        )),
                    )),
                ),
            )
        }

        for program in programs {
            let result = OplParser::parse(program).unwrap();

            // expect we've only parsed the one expression, despite the comments
            let data_exprs = result.pipeline.get_expressions();
            assert_eq!(data_exprs.len(), 1);
            pretty_assertions::assert_eq!(
                &data_exprs[0],
                &gen_simple_equals_filter_expected("severity_text", "ERROR")
            );
        }

        // test that we support the comment characters inside the text
        let result = OplParser::parse("logs | where body == \"http://example.com\"").unwrap();
        let data_exprs = result.pipeline.get_expressions();
        assert_eq!(data_exprs.len(), 1);
        pretty_assertions::assert_eq!(
            &data_exprs[0],
            &gen_simple_equals_filter_expected("body", "http://example.com")
        );

        let result = OplParser::parse("logs | where body == \"example /* comment */\"").unwrap();
        let data_exprs = result.pipeline.get_expressions();
        assert_eq!(data_exprs.len(), 1);
        pretty_assertions::assert_eq!(
            &data_exprs[0],
            &gen_simple_equals_filter_expected("body", "example /* comment */")
        );
    }

    #[test]
    fn test_parse_drop() {
        let result = OplParser::parse("logs | drop");
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected =
            DataExpression::Discard(DiscardDataExpression::new(QueryLocation::new_fake()));
        pretty_assertions::assert_eq!(&expressions[0], &expected);
    }

    #[test]
    fn test_parse_drop_in_if_else() {
        let query = r#"
            logs | if (severity_text == "DEBUG") {
                drop
            }
        "#;
        let result = OplParser::parse(query);
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        // The outer expression should be a Conditional containing a Discard in the branch
        match &expressions[0] {
            DataExpression::Conditional(cond) => {
                let branches = cond.get_branches();
                assert_eq!(branches.len(), 1);
                let branch_exprs = branches[0].get_expressions();
                assert_eq!(branch_exprs.len(), 1);
                let expected =
                    DataExpression::Discard(DiscardDataExpression::new(QueryLocation::new_fake()));
                pretty_assertions::assert_eq!(&branch_exprs[0], &expected);
            }
            other => panic!("expected Conditional, got {other:?}"),
        }
    }
}
