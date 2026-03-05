// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use data_engine_expressions::{DataExpression, PipelineFunction};
use data_engine_parser_abstractions::{ParserError, ParserScope, ParserState, to_query_location};
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

    /// push a function definition, returns the function ID
    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize;
}

// TODO - should probably add a wrapper around this that keeps track of the function IDs
// TODO - need to somehow merge the function Ids from the child back into this ...

impl PipelineBuilder for ParserState {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.push_expression(data_expression);
    }

    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize {
        self.push_function(name, definition, Vec::new(), HashMap::new());
        return self
            .get_function_id(name)
            .expect("should have function with name")
            .get_id();
    }
}

/// simple [`PipelineBuilder`] implementation for collecting nested data expressions and
/// function definitions
pub(crate) struct InnerPipelineBuilder {
    data_exprs: Vec<DataExpression>,
    functions: Vec<(String, PipelineFunction)>,
}

impl InnerPipelineBuilder {
    pub fn new() -> Self {
        Self::new_with_capacities(None, None)
    }

    pub fn new_with_capacities(
        data_expr_capacity: Option<usize>,
        functions_capacity: Option<usize>,
    ) -> Self {
        Self {
            data_exprs: Vec::with_capacity(data_expr_capacity.unwrap_or_default()),
            functions: Vec::with_capacity(functions_capacity.unwrap_or_default()),
        }
    }

    pub fn into_data_exprs(self) -> Vec<DataExpression> {
        self.data_exprs
    }
}

impl PipelineBuilder for InnerPipelineBuilder {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.data_exprs.push(data_expression)
    }

    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize {
        self.functions.push((name.to_string(), definition));
        // TODO this isn't correct
        self.functions.len() - 1
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
