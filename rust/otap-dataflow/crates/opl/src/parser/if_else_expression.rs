// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    ConditionalDataExpression, ConditionalDataExpressionBranch, DataExpression, Expression,
    LogicalExpression, QueryLocation,
};
use data_engine_parser_abstractions::{ParserError, ParserScope, to_query_location};
use pest::iterators::Pair;

use crate::{
    Rule, logical_expressions::parse_logical_expression,
    tabular_expressions::parse_tabular_expression_rule,
};

pub fn parse_if_else_expression(
    if_else_expression: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<DataExpression, ParserError> {
    let query_location = to_query_location(&if_else_expression);
    let mut conditional_expr = ConditionalDataExpression::new(query_location);

    // keep track of the location of the current branch
    let mut branch_location_start = 0;
    let mut branch_location_line = 0;
    let mut branch_location_col = 0;

    let mut next_condition: Option<LogicalExpression> = None;
    let mut next_branch: Vec<DataExpression> = Vec::new();

    let inner = if_else_expression.into_inner();
    for rule in inner {
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
                next_condition = Some(parse_logical_expression(condition_rule, scope)?);
            }

            // parse the pipeline of data expressions for this branch
            Rule::if_else_branch_expression => {
                let branch_loc_end = rule.as_span().end();

                // parse all the rules
                for tabular_rule in rule.into_inner() {
                    let mut expr = parse_tabular_expression_rule(tabular_rule, scope)?;
                    next_branch.append(&mut expr);
                }

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
                let mut else_branch_exprs = Vec::new();
                let branch_rules = rule.into_inner().next().ok_or_else(|| {
                    // under normal invocation of this function this shouldn't happen as this
                    // missing expression should be caught by the parser
                    ParserError::SyntaxError(
                        conditional_expr.get_query_location().clone(),
                        "expected else_expression to contain one inner if_else_branch_expression"
                            .to_string(),
                    )
                })?;

                for tabular_rule in branch_rules.into_inner() {
                    let mut expr = parse_tabular_expression_rule(tabular_rule, scope)?;
                    else_branch_exprs.append(&mut expr);
                }
                conditional_expr = conditional_expr.with_default_branch(else_branch_exprs);
            }
            _ => {
                return Err(ParserError::SyntaxError(
                    conditional_expr.get_query_location().clone(),
                    format!("invalid rule found in if_else_expression {rule}"),
                ));
            }
        }
    }

    Ok(DataExpression::Conditional(conditional_expr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kql_parser::KqlParser;
    use data_engine_expressions::{
        EqualToLogicalExpression, MutableValueExpression, ScalarExpression, SetTransformExpression,
        SourceScalarExpression, StaticScalarExpression, StringScalarExpression,
        TransformExpression, ValueAccessor,
    };
    use data_engine_parser_abstractions::Parser;

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

        let result = KqlParser::parse(query);
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

        let result = KqlParser::parse(query);
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

        let result = KqlParser::parse(query);
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

        let result = KqlParser::parse(query);
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
}
