// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::{
    MutableValueExpression, ScalarExpression, SetTransformExpression, SourceScalarExpression,
    StaticScalarExpression, StringScalarExpression, ValueAccessor,
};
use data_engine_parser_abstractions::{ParserError, to_query_location};
use pest::iterators::Pair;

use crate::parser::expression::{
    parse_attribute_selection_expression, parse_expression, parse_index_expression,
};
use crate::parser::{Rule, invalid_child_rule_error};

pub(crate) fn parse_assignment_expression(
    rule: Pair<'_, Rule>,
) -> Result<SetTransformExpression, ParserError> {
    let query_location = to_query_location(&rule);
    let mut inner_rules = rule.into_inner();
    if inner_rules.len() != 2 {
        return Err(ParserError::SyntaxError(
            query_location,
            format!("Expected exactly two rules. Found {}", inner_rules.len()),
        ));
    }

    // safety: we've checked just above that there are two rules
    let left = inner_rules.next().expect("two rules");
    let right = inner_rules.next().expect("two rules");

    let left_query_location = to_query_location(&left);
    let destination = match left.as_rule() {
        Rule::identifier_expression => MutableValueExpression::Source(SourceScalarExpression::new(
            left_query_location.clone(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    left_query_location,
                    left.as_str(),
                )),
            )]),
        )),
        Rule::index_expression => match parse_index_expression(left)?.into() {
            ScalarExpression::Source(source_expr) => MutableValueExpression::Source(source_expr),
            other => {
                return Err(ParserError::SyntaxError(
                    left_query_location,
                    format!(
                        "Expected source scalar for index_expression rule, found {:?}",
                        other
                    ),
                ));
            }
        },
        Rule::attribute_selection_expression => match parse_attribute_selection_expression(left)?
            .into()
        {
            ScalarExpression::Source(source_expr) => MutableValueExpression::Source(source_expr),
            other => {
                return Err(ParserError::SyntaxError(
                    left_query_location,
                    format!(
                        "Expected source scalar for attribute_selection_expression rule, found {:?}",
                        other
                    ),
                ));
            }
        },
        invalid_rule => {
            return Err(invalid_child_rule_error(
                left_query_location,
                Rule::assignment_expression,
                invalid_rule,
            ));
        }
    };

    let right_query_location = to_query_location(&right);
    let source = match right.as_rule() {
        Rule::expression => parse_expression(right)?.into(),
        invalid_rule => {
            return Err(invalid_child_rule_error(
                right_query_location,
                Rule::assignment_expression,
                invalid_rule,
            ));
        }
    };

    Ok(SetTransformExpression::new(
        query_location,
        source,
        destination,
    ))
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{
        IntegerScalarExpression, QueryLocation, ScalarExpression, StaticScalarExpression,
        StringScalarExpression,
    };
    use pest::Parser;

    use super::*;
    use crate::parser::pest::OplPestParser;

    #[test]
    fn test_simple_assignment() {
        let input = "a = 1";
        let mut rules = OplPestParser::parse(Rule::assignment_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = rules.next().unwrap();
        let result = parse_assignment_expression(rule).unwrap();

        let expected = SetTransformExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "a",
                    )),
                )]),
            )),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_attribute_assignment() {
        let input = "attributes[\"x\"] = 1";
        let mut rules = OplPestParser::parse(Rule::assignment_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = rules.next().unwrap();
        let result = parse_assignment_expression(rule).unwrap();

        let expected = SetTransformExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "x"),
                    )),
                ]),
            )),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_field_assignment() {
        let input = "instrumentation_scope.name = \"sdk\"";
        let mut rules = OplPestParser::parse(Rule::assignment_expression, input).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = rules.next().unwrap();
        let result = parse_assignment_expression(rule).unwrap();

        let expected = SetTransformExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "sdk",
            ))),
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "instrumentation_scope",
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "name"),
                    )),
                ]),
            )),
        );
        assert_eq!(result, expected);
    }
}
