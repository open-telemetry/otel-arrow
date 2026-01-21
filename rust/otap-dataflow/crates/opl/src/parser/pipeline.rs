// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::DataExpression;
use data_engine_parser_abstractions::{ParserError, ParserState, to_query_location};
use pest::iterators::Pair;

use crate::parser::operator::parse_operator_call;
use crate::parser::{Rule, invalid_child_rule_error};

/// Trait for building pipelines.
///
/// This abstracts away the details of how expressions are added to a pipeline, so the same parser
/// utility functions can be used targetting different pipeline builder. In pratice, this is useful
/// when building nested pipelines for some expressions which nest pipeline stages, such as if/else
pub(crate) trait PipelineBuilder {
    fn push_data_expression(&mut self, data_expression: DataExpression);
}

impl PipelineBuilder for Vec<DataExpression> {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.push(data_expression);
    }
}

impl PipelineBuilder for ParserState {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.push_expression(data_expression);
    }
}

pub(crate) fn parse_pipeline(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::source => {
                // ignore for now
            }
            Rule::pipeline_stage => parse_pipeline_stage(rule, state)?,
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::pipeline,
                    invalid_rule,
                ));
            }
        }
    }
    Ok(())
}

pub(crate) fn parse_pipeline_stage(
    rule: Pair<'_, Rule>,
    pipeline_builder: &mut dyn PipelineBuilder,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::operator_call => parse_operator_call(rule, pipeline_builder)?,
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::pipeline_stage,
                    invalid_rule,
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::pest::OplPestParser;
    use pest::Parser as _;

    #[test]
    fn test_parse_pipeline() {
        let input = "logs | where severity == 'error' | where x == 42";
        let mut parser_state = ParserState::new(input);
        parse_pipeline(
            OplPestParser::parse(Rule::pipeline, input)
                .expect("Failed to parse input")
                .next()
                .expect("No pipeline rule found"),
            &mut parser_state,
        )
        .unwrap();

        let pipeline = parser_state.build().unwrap();
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 2);
    }
}
