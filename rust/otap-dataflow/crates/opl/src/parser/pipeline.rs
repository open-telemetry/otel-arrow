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
/// utility functions can be used targeting different pipeline builder. In practice, this is useful
/// when building nested pipelines for some expressions which nest pipeline stages, such as if/else
pub(crate) trait PipelineBuilder {
    fn push_data_expression(&mut self, data_expression: DataExpression);

    /// push a function definition, returns the function ID
    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize;

    fn get_max_func_id(&self) -> Option<usize>;

    fn child_func_id_offset(&self) -> usize {
        self.get_max_func_id().map(|i| i + 1).unwrap_or_default()
    }
}

pub struct RootPipelineBuilder<'a> {
    parser_state: &'a mut ParserState,
    max_func_id: Option<usize>,
}

impl<'a> RootPipelineBuilder<'a> {
    pub fn new(parser_state: &'a mut ParserState) -> Self {
        Self {
            parser_state,
            max_func_id: None,
        }
    }
}

impl PipelineBuilder for RootPipelineBuilder<'_> {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.parser_state.push_expression(data_expression);
    }

    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize {
        self.parser_state
            .push_function(name, definition, Vec::new(), HashMap::new());
        let func_id = self
            .parser_state
            .get_function_id(name)
            .expect("should have function with name")
            .get_id();
        self.max_func_id = Some(self.max_func_id.unwrap_or(0).max(func_id));

        func_id
    }

    fn get_max_func_id(&self) -> Option<usize> {
        self.max_func_id
    }
}

/// simple [`PipelineBuilder`] implementation for collecting nested data expressions and
/// function definitions
pub(crate) struct InnerPipelineBuilder {
    /// The functions that will be appended to this inner pipeline will need to be appended
    /// to the parent pipeline builder eventually. The expressions inside the inner pipeline
    /// will reference functions by ID, but the ID must be the global ID. This value is used
    /// to offset the local IDs to the global function ID.
    func_id_offset: usize,

    data_exprs: Vec<DataExpression>,
    functions: Vec<(String, PipelineFunction)>,
}

impl InnerPipelineBuilder {
    pub fn new(func_id_offset: usize) -> Self {
        Self::new_with_capacities(func_id_offset, None, None)
    }

    pub fn new_with_capacities(
        func_id_offset: usize,
        data_expr_capacity: Option<usize>,
        functions_capacity: Option<usize>,
    ) -> Self {
        Self {
            func_id_offset,
            data_exprs: Vec::with_capacity(data_expr_capacity.unwrap_or_default()),
            functions: Vec::with_capacity(functions_capacity.unwrap_or_default()),
        }
    }

    pub fn into_parts(self) -> (Vec<DataExpression>, Vec<(String, PipelineFunction)>) {
        (self.data_exprs, self.functions)
    }
}

impl PipelineBuilder for InnerPipelineBuilder {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.data_exprs.push(data_expression)
    }

    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize {
        self.functions.push((name.to_string(), definition));
        self.func_id_offset + self.functions.len() - 1
    }

    fn get_max_func_id(&self) -> Option<usize> {
        if !self.functions.is_empty() || self.func_id_offset > 0 {
            Some(self.functions.len() + self.func_id_offset - 1)
        } else {
            None
        }
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
            Rule::pipeline_stage => {
                parse_pipeline_stage(rule, &mut RootPipelineBuilder::new(state))?
            }
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
