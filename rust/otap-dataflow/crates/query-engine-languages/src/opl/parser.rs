// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OpenTelemetry Data Processing Language (OPL) parser implementation.

use std::vec;

use ::pest::{Parser as _, iterators::Pair};
use data_engine_expressions::{PipelineFunction, QueryLocation, ScalarExpression};
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

use crate::opl::parser::{
    expression::parse_expression,
    pipeline::{RootPipelineBuilder, parse_pipeline},
};

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

impl OplParser {
    /// Parse the expression into a [`ScalarExpression`]
    ///
    /// Returns the parsed expression and definition of any functions referenced by the expression.
    pub fn parse_expr_with_options(
        expr: &str,
        options: ParserOptions,
    ) -> Result<(ScalarExpression, Vec<PipelineFunction>), Vec<ParserError>> {
        let mut parse_result = match pest::OplPestParser::parse(Rule::expression, expr) {
            Ok(rules) => rules,
            Err(pest_error) => {
                return Err(vec![ParserError::from_pest_error(expr, pest_error)]);
            }
        };

        let rule = match parse_result.next() {
            Some(rule) => rule,
            None => {
                // safety: this is a valid query location
                let query_location =
                    QueryLocation::new(0, expr.len(), 1, 1).expect("valid query location");

                return Err(vec![ParserError::SyntaxError(
                    query_location,
                    "Unable to parse: no rules".into(),
                )]);
            }
        };

        let mut state = ParserState::new_with_options(expr, options);
        let pipeline_builder = RootPipelineBuilder::new(&mut state);
        let result = parse_expression(rule, &pipeline_builder).map_err(|e| vec![e])?;
        let pipeline = state.build()?;

        Ok((result.into(), pipeline.get_functions().to_vec()))
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
        BooleanScalarExpression, BranchDataExpression, DataExpression, DataExpressionBranch,
        DiscardDataExpression, EqualToLogicalExpression, GetRecordTypeScalarExpression,
        LogicalExpression, MatchesLogicalExpression, NotLogicalExpression, QueryLocation,
        RegexScalarExpression, ScalarExpression, SourceScalarExpression, StaticScalarExpression,
        StringScalarExpression, ValueAccessor,
    };
    use data_engine_parser_abstractions::Parser;
    use regex::Regex;

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
    fn test_parse_empty_string() {
        let result = OplParser::parse("");
        assert!(result.is_err());

        let errors = result.err().unwrap();
        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        assert!(
            error
                .to_string()
                .contains("Syntax '' supplied in query is not supported")
        )
    }

    #[test]
    fn test_parse_escape_sequence_double_backslash() {
        let result =
            OplParser::parse(r#"logs | where (matches(attributes["code"], "\\d+"))"#).unwrap();
        let data_exprs = result.pipeline.get_expressions();
        assert_eq!(data_exprs.len(), 1);
        pretty_assertions::assert_eq!(
            &data_exprs[0],
            &DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::Not(NotLogicalExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::Matches(MatchesLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            "attributes",
                                        )
                                    ),),
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            "code",
                                        )
                                    ),)
                                ]),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Regex(
                                RegexScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    Regex::new("\\d+").unwrap()
                                ),
                            )),
                        )),
                    )),
                ),
            )
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
            DataExpression::Branch(branch_expr) => {
                let branches = branch_expr.get_branches();
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

    /// Scenario: Parse OPL pipelines that use plural concrete metric types as source.
    /// Guarantees: The parser produces a query plan that only processes rows that have the
    /// selected metric type
    #[test]
    fn test_parses_program_for_metrics_types() {
        let test_cases = [
            ("gauges", "Gauge"),
            ("sums", "Sum"),
            ("histograms", "Histogram"),
            ("exponential_histograms", "ExponentialHistogram"),
            ("summaries", "Summary"),
        ];

        for (source, metric_type_name) in test_cases {
            let query = format!("{source} | where true");
            let pipeline = OplParser::parse(&query).unwrap().pipeline;
            let expressions = pipeline.get_expressions();
            assert_eq!(expressions.len(), 1);
            pretty_assertions::assert_eq!(
                expressions[0],
                DataExpression::Branch(
                    BranchDataExpression::new(QueryLocation::new_fake(), true).with_branch(
                        DataExpressionBranch::new(
                            QueryLocation::new_fake(),
                            Some(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                                QueryLocation::new_fake(),
                                ScalarExpression::GetRecordType(
                                    GetRecordTypeScalarExpression::new(QueryLocation::new_fake())
                                ),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        metric_type_name
                                    )
                                )),
                                false
                            ))),
                            vec![DataExpression::Discard(
                                DiscardDataExpression::new(QueryLocation::new_fake())
                                    .with_predicate(LogicalExpression::Scalar(
                                        ScalarExpression::Static(StaticScalarExpression::Boolean(
                                            BooleanScalarExpression::new(
                                                QueryLocation::new_fake(),
                                                false
                                            )
                                        ))
                                    ))
                            )]
                        )
                    )
                )
            )
        }
    }
}
