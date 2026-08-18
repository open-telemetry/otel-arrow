// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use data_engine_expressions::{
    BranchDataExpression, DataExpression, DataExpressionBranch, EqualToLogicalExpression,
    GetRecordTypeScalarExpression, LogicalExpression, PipelineFunction, ScalarExpression,
    StaticScalarExpression, StringScalarExpression,
};
use data_engine_parser_abstractions::{
    ParserError, ParserFunction, ParserScope, ParserState, to_query_location,
};
use pest::iterators::Pair;

use crate::opl::parser::operator::parse_operator_call;
use crate::opl::parser::{Rule, invalid_child_rule_error};

/// Trait for building pipelines.
///
/// This abstracts away the details of how expressions are added to a pipeline, so the same parser
/// utility functions can be used targeting different pipeline builder. In practice, this is useful
/// when building nested pipelines for some expressions which nest pipeline stages, such as if/else
pub(crate) trait PipelineBuilder {
    fn push_data_expression(&mut self, data_expression: DataExpression);

    /// push a function definition, returns the function ID
    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize;

    /// get the definition of a function by name. Returns `None` if no function with the passed
    /// name exists
    fn get_function(&self, name: &str) -> Option<&ParserFunction>;
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

    fn get_function(&self, name: &str) -> Option<&ParserFunction> {
        self.parser_state.get_function_id(name)
    }
}

/// simple [`PipelineBuilder`] implementation for collecting nested data expressions and
/// function definitions
pub(crate) struct InnerPipelineBuilder<'a> {
    data_exprs: Vec<DataExpression>,

    pub parent: &'a mut dyn PipelineBuilder,
}

impl<'a> InnerPipelineBuilder<'a> {
    pub fn new(parent: &'a mut dyn PipelineBuilder) -> Self {
        Self::new_with_capacity(None, parent)
    }

    pub fn new_with_capacity(
        data_expr_capacity: Option<usize>,
        parent: &'a mut dyn PipelineBuilder,
    ) -> Self {
        Self {
            data_exprs: Vec::with_capacity(data_expr_capacity.unwrap_or_default()),
            parent,
        }
    }

    pub fn into_parts(self) -> (Vec<DataExpression>, &'a mut dyn PipelineBuilder) {
        (self.data_exprs, self.parent)
    }
}

impl<'a> PipelineBuilder for InnerPipelineBuilder<'a> {
    fn push_data_expression(&mut self, data_expression: DataExpression) {
        self.data_exprs.push(data_expression)
    }

    fn push_function_definition(&mut self, name: &str, definition: PipelineFunction) -> usize {
        self.parent.push_function_definition(name, definition)
    }

    fn get_function(&self, name: &str) -> Option<&ParserFunction> {
        self.parent.get_function(name)
    }
}

pub(crate) fn parse_pipeline(
    rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    let pipeline_query_location = to_query_location(&rule);
    let mut metric_concrete_type_condition = None;
    let mut root_pipeline_builder = RootPipelineBuilder::new(state);
    let mut inner_pipeline_builder = InnerPipelineBuilder::new(&mut root_pipeline_builder);
    for rule in rule.into_inner() {
        match rule.as_rule() {
            Rule::source => {
                let Some(source_rule) = rule.into_inner().next() else {
                    continue;
                };
                let source_query_location = to_query_location(&source_rule);

                // try to determine the type of metrics selected by this pipeline.
                // for example, if the caller supplies a query like: "gauges | ... "
                // we should only execute on metrics batches and only on the rows
                // that contain gauges, ignoring other batches and non gauge rows
                if matches!(source_rule.as_rule(), Rule::metric_type_source) {
                    let metric_type_name = match source_rule.as_str() {
                        "gauges" => "Gauge",
                        "sums" => "Sum",
                        "histograms" => "Histogram",
                        "exponential_histograms" => "ExponentialHistogram",
                        "summaries" => "Summary",
                        _ => {
                            return Err(ParserError::SyntaxNotSupported(
                                source_query_location,
                                format!("Unknown source identifier {:?}", source_rule.as_str()),
                            ));
                        }
                    };

                    metric_concrete_type_condition =
                        Some((source_query_location, metric_type_name));
                }
            }
            Rule::pipeline_stage => {
                parse_pipeline_stage(rule, &mut inner_pipeline_builder)?;
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

    let (exprs, _) = inner_pipeline_builder.into_parts();

    // if the source was some concrete metric type, create a single condition selecting the only
    // rows containing this metric type. Effectively, this transforms the query into something like:
    // signals | if (is <metric type name>) { <... pipeline ...> }
    if let Some((source_query_location, metric_type_name)) = metric_concrete_type_condition {
        let branch_expr = BranchDataExpression::new(pipeline_query_location.clone(), true)
            .with_branch(DataExpressionBranch::new(
                pipeline_query_location.clone(),
                Some(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    source_query_location.clone(),
                    ScalarExpression::GetRecordType(GetRecordTypeScalarExpression::new(
                        source_query_location.clone(),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(
                            source_query_location.clone(),
                            metric_type_name,
                        ),
                    )),
                    false,
                ))),
                exprs,
            ));
        root_pipeline_builder.push_data_expression(DataExpression::Branch(branch_expr));
    } else {
        exprs
            .into_iter()
            .for_each(|expr| root_pipeline_builder.push_data_expression(expr));
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
    use crate::opl::parser::pest::OplPestParser;
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
