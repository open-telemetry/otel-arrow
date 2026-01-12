// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
<<<<<<< HEAD
    DataExpression, DiscardDataExpression, LogicalExpression, NotLogicalExpression,
    TransformExpression,
=======
    ConditionalDataExpression, ConditionalDataExpressionBranch, DataExpression,
    DiscardDataExpression, Expression, LogicalExpression, NotLogicalExpression, QueryLocation,
>>>>>>> a67f5e8f (added if statement that works, aside from extend used in tests)
};
use data_engine_parser_abstractions::{ParserError, ParserState, to_query_location};
use pest::iterators::Pair;

use crate::parser::assignment::parse_assignment_expression;
use crate::parser::expression::parse_expression;
use crate::parser::{Rule, invalid_child_rule_error};

pub(crate) fn parse_operator_call(
    rule: Pair<'_, Rule>,
    pipeline_builder: &mut dyn PipelineBuilder,
) -> Result<(), ParserError> {
    for rule in rule.into_inner() {
        match rule.as_rule() {
<<<<<<< HEAD
            Rule::set_operator_call => parse_set_operator_call(rule, state)?,
            Rule::where_operator_call => parse_where_operator_call(rule, state)?,
=======
            Rule::if_else_operator_call => parse_if_else_opeartor_call(rule, pipeline_builder)?,
            Rule::where_operator_call => parse_where_operator_call(rule, pipeline_builder)?,
>>>>>>> a67f5e8f (added if statement that works, aside from extend used in tests)
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::operator_call,
                    invalid_rule,
                ));
            }
        };
    }

    Ok(())
}

<<<<<<< HEAD
pub(crate) fn parse_set_operator_call(
    operator_call_rule: Pair<'_, Rule>,
    state: &mut ParserState,
) -> Result<(), ParserError> {
    if let Some(rule) = operator_call_rule.into_inner().next() {
        match rule.as_rule() {
            Rule::assignment_expression => {
                let set_expr = parse_assignment_expression(rule)?;
                let transform_expr = TransformExpression::Set(set_expr);
                state.push_expression(DataExpression::Transform(transform_expr));
            }
            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::assignment_expression,
                    invalid_rule,
=======
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

pub(crate) fn parse_if_else_opeartor_call(
    operator_call_rule: Pair<'_, Rule>,
    pipeline_builder: &mut dyn PipelineBuilder,
) -> Result<(), ParserError> {
    let query_location = to_query_location(&operator_call_rule);
    let mut conditional_expr = ConditionalDataExpression::new(query_location);

    // keep track of the location of the current branch (to fill in query location)
    let mut branch_location_start = 0;
    let mut branch_location_line = 0;
    let mut branch_location_col = 0;

    let mut next_condition: Option<LogicalExpression> = None;
    let mut next_branch: Vec<DataExpression> = Vec::new();

    for rule in operator_call_rule.into_inner() {
        match rule.as_rule() {
            // parse the condition for the branch
            Rule::if_condition_expression => {
                branch_location_start = rule.as_span().start();
                let (line_number, column_number) = rule.line_col();
                branch_location_line = line_number;
                branch_location_col = column_number;

                let condition_rule = rule.into_inner().next().ok_or_else(|| {
                    // under normal invocation of this function this shouldn't happen as this
                    // missing expression should be caught by the parser
                    ParserError::SyntaxError(
                        conditional_expr.get_query_location().clone(),
                        "expected if_condition_expression to contain one inner logical_expression"
                            .to_string(),
                    )
                })?;
                next_condition = Some(parse_expression(condition_rule)?);
            }

            // parse the pipeline of data expressions for this branch
            Rule::if_else_branch_expression => {
                let branch_loc_end = rule.as_span().end();

                // parse all the rules
                let inner_rules = rule.into_inner();
                let mut inner_exprs = Vec::with_capacity(inner_rules.len());
                for inner_rule in inner_rules {
                    parse_operator_call(inner_rule, &mut inner_exprs)?;
                }
                inner_exprs
                    .drain(..)
                    .for_each(|expr| pipeline_builder.push_data_expression(expr));

                // take the data expressions for the branch and reset next_branch
                let curr_branch = next_branch;
                next_branch = Vec::new();

                let query_location = QueryLocation::new(
                    branch_location_start,
                    branch_loc_end,
                    branch_location_line,
                    branch_location_col,
                )
                .map_err(|e| {
                    ParserError::SyntaxError(
                        conditional_expr.get_query_location().clone(),
                        format!("invalid query location {e}"),
                    )
                })?;

                let condition = next_condition.take().ok_or_else(|| {
                    // next_condition should always be Some here as we should see the
                    // if_condition_expression first.Under normal invocation of this function
                    // the order should be enforced by the parser
                    ParserError::SyntaxError(
                        conditional_expr.get_query_location().clone(),
                        "expected if_condition_expression before if_else_branch_expression"
                            .to_string(),
                    )
                })?;

                conditional_expr = conditional_expr.with_branch(
                    ConditionalDataExpressionBranch::new(query_location, condition, curr_branch),
                );
            }

            // parse the data expressions for the else branch
            Rule::else_expression => {
                let branch_rules = rule.into_inner().next().ok_or_else(|| {
                    // under normal invocation of this function this shouldn't happen as this
                    // missing expression should be caught by the parser
                    ParserError::SyntaxError(
                        conditional_expr.get_query_location().clone(),
                        "expected else_expression to contain one inner if_else_branch_expression"
                            .to_string(),
                    )
                })?;

                let inner_rules = branch_rules.into_inner();
                let mut else_branch_exprs = Vec::with_capacity(inner_rules.len());
                for inner_rule in inner_rules {
                    parse_operator_call(inner_rule, &mut else_branch_exprs)?;
                }

                conditional_expr = conditional_expr.with_default_branch(else_branch_exprs);
            }
            _ => {
                return Err(ParserError::SyntaxError(
                    conditional_expr.get_query_location().clone(),
                    format!("invalid rule found in if_else_expression {rule}"),
>>>>>>> a67f5e8f (added if statement that works, aside from extend used in tests)
                ));
            }
        }
    }

<<<<<<< HEAD
    Ok(())
=======
    todo!()
>>>>>>> a67f5e8f (added if statement that works, aside from extend used in tests)
}

pub(crate) fn parse_where_operator_call(
    operator_call_rule: Pair<'_, Rule>,
    pipeline_builder: &mut dyn PipelineBuilder,
) -> Result<(), ParserError> {
    let operator_call_query_location = to_query_location(&operator_call_rule);
    if let Some(rule) = operator_call_rule.into_inner().next() {
        match rule.as_rule() {
            Rule::expression => {
                let rule_query_location = to_query_location(&rule);
                let predicate = parse_expression(rule)?;
                let discard_expr = DiscardDataExpression::new(operator_call_query_location)
                    .with_predicate(LogicalExpression::Not(NotLogicalExpression::new(
                        rule_query_location,
                        predicate.into(),
                    )));
                pipeline_builder.push_data_expression(DataExpression::Discard(discard_expr));
            }

            invalid_rule => {
                let query_location = to_query_location(&rule);
                return Err(invalid_child_rule_error(
                    query_location,
                    Rule::operator_call,
                    invalid_rule,
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use data_engine_expressions::{
        ConditionalDataExpression, ConditionalDataExpressionBranch,
        DataExpression, DiscardDataExpression, EqualToLogicalExpression, LogicalExpression,
        MutableValueExpression, NotLogicalExpression, QueryLocation, ScalarExpression,
        SetTransformExpression, SourceScalarExpression, StaticScalarExpression,
        StringScalarExpression, TransformExpression, ValueAccessor,
    };
    use data_engine_parser_abstractions::{Parser, ParserOptions, ParserState};
    use pest::Parser as _;
    use pretty_assertions::assert_eq;

    use crate::parser::operator::parse_operator_call;
    use crate::parser::pest::OplPestParser;
    use crate::parser::{OplParser, Rule};

    #[test]
    fn test_parse_set_operator_call() {
        let query = "set severity_text = \"ERROR\"";
        let mut state = ParserState::new(query);
        let parse_result = OplPestParser::parse(Rule::operator_call, query).unwrap();
        assert_eq!(parse_result.len(), 1);
        let rule = parse_result.into_iter().next().unwrap();
        parse_operator_call(rule, &mut state).unwrap();
        let result = state.build().unwrap();
        let expressions = result.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected =
            DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "ERROR"),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "severity_text",
                        )),
                    )]),
                )),
            )));

        assert_eq!(&expressions[0], &expected);
    }



    fn equals_logical_expr(field_name: &'static str, value: &'static str) -> LogicalExpression {
        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        field_name,
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                value,
            ))),
            false,
        ))
    }

    fn assign_attribute_expression(
        attr_key: &'static str,
        attr_val: &'static str,
    ) -> DataExpression {
        DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                attr_val,
            ))),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), attr_key),
                    )),
                ]),
            )),
        )))
    }

    #[test]
    pub fn test_parse_if_else_expression() {
        let query = r#"
               logs | if (severity_text == "ERROR") {
                   extend attributes["important"] = "very" | extend attributes["triggers_alarm"] = "true"
               } else if (severity_text == "WARN") {
                   extend attributes["important"] = "somewhat"
               } else if (severity_text == "INFO") {
                   extend attributes["important"] = "rarely"
               } else {
                   extend attributes["important"] = "no"
               }
           "#;

        let result = OplParser::parse(query);
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected = DataExpression::Conditional(
            ConditionalDataExpression::new(QueryLocation::new_fake())
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "ERROR"),
                    vec![
                        assign_attribute_expression("important", "very"),
                        assign_attribute_expression("triggers_alarm", "true"),
                    ],
                ))
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "WARN"),
                    vec![assign_attribute_expression("important", "somewhat")],
                ))
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "INFO"),
                    vec![assign_attribute_expression("important", "rarely")],
                ))
                .with_default_branch(vec![assign_attribute_expression("important", "no")]),
        );
        assert_eq!(expressions[0], expected);
    }

    #[test]
    pub fn test_parse_if_else_expression_no_else() {
        let query = r#"
               logs | if (severity_text == "ERROR") {
                   extend attributes["important"] = "very" | extend attributes["triggers_alarm"] = "true"
               } else if (severity_text == "WARN") {
                   extend attributes["important"] = "somewhat"
               }
           "#;

        let result = OplParser::parse(query);
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected = DataExpression::Conditional(
            ConditionalDataExpression::new(QueryLocation::new_fake())
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "ERROR"),
                    vec![
                        assign_attribute_expression("important", "very"),
                        assign_attribute_expression("triggers_alarm", "true"),
                    ],
                ))
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "WARN"),
                    vec![assign_attribute_expression("important", "somewhat")],
                )),
        );
        assert_eq!(expressions[0], expected);
    }

    #[test]
    pub fn test_parse_if_else_expression_no_elseif() {
        let query = r#"
               logs | if (severity_text == "ERROR") {
                   extend attributes["important"] = "very" | extend attributes["triggers_alarm"] = "true"
               } else {
                   extend attributes["important"] = "no"
               }
           "#;

        let result = OplParser::parse(query);
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected = DataExpression::Conditional(
            ConditionalDataExpression::new(QueryLocation::new_fake())
                .with_branch(ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "ERROR"),
                    vec![
                        assign_attribute_expression("important", "very"),
                        assign_attribute_expression("triggers_alarm", "true"),
                    ],
                ))
                .with_default_branch(vec![assign_attribute_expression("important", "no")]),
        );
        assert_eq!(expressions[0], expected);
    }

    #[test]
    pub fn test_parse_if_else_expression_if_only() {
        let query = r#"
               logs | if (severity_text == "ERROR") {
                   extend attributes["triggers_alarm"] = "true"
               }
           "#;

        let result = OplParser::parse(query);
        assert!(result.is_ok());

        let pipeline = result.unwrap().pipeline;
        let expressions = pipeline.get_expressions();
        assert_eq!(expressions.len(), 1);

        let expected = DataExpression::Conditional(
            ConditionalDataExpression::new(QueryLocation::new_fake()).with_branch(
                ConditionalDataExpressionBranch::new(
                    QueryLocation::new_fake(),
                    equals_logical_expr("severity_text", "ERROR"),
                    vec![assign_attribute_expression("triggers_alarm", "true")],
                ),
            ),
        );
        assert_eq!(expressions[0], expected);
    }

    #[test]
    fn test_parse_where_operator_call() {
        let query = "where value == \"x\"";
        let mut state = ParserState::new_with_options(query, ParserOptions::default());
        let parse_result = OplPestParser::parse(Rule::operator_call, query).unwrap();
        assert_eq!(parse_result.len(), 1);
        let rule = parse_result.into_iter().next().unwrap();
        parse_operator_call(rule, &mut state).unwrap();

        let result = state.build().unwrap();
        let expressions = result.get_expressions();
        assert_eq!(expressions.len(), 1);
        let expected = DataExpression::Discard(
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
                                    "value",
                                )),
                            )]),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "x"),
                        )),
                        true,
                    )),
                )),
            ),
        );

        assert_eq!(&expressions[0], &expected);
    }
}
