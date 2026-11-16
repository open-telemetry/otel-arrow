// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_contrib_data_engine_expressions::*;
use otel_arrow_contrib_data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::ottl::Rule;

#[allow(dead_code)]
pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<'_, Rule>,
    _state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule
        .into_inner()
        .next()
        .expect("scalar_expression must have an inner rule");

    let scalar = match scalar_rule.as_rule() {
        Rule::true_literal | Rule::false_literal => {
            ScalarExpression::Static(parse_standard_bool_literal(scalar_rule))
        }
        Rule::float_literal => {
            ScalarExpression::Static(parse_standard_double_literal(scalar_rule, Some("float"))?)
        }
        Rule::integer_literal => {
            ScalarExpression::Static(parse_standard_integer_literal(scalar_rule)?)
        }
        Rule::string_literal => {
            ScalarExpression::Static(parse_standard_string_literal(scalar_rule)?)
        }
        Rule::null_literal => ScalarExpression::Static(parse_standard_null_literal(scalar_rule)),
        Rule::field_expression => parse_field_path_expression(scalar_rule)?,
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, _state)?,
        Rule::identifier_expression => {
            // parse this as a field on the source
            let query_location = to_query_location(&scalar_rule);
            let value_accessor = ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    query_location.clone(),
                    scalar_rule.as_str(),
                )),
            )]);

            ScalarExpression::Source(SourceScalarExpression::new(query_location, value_accessor))
        }
        _ => panic!("Unexpected rule in scalar_expression: {scalar_rule}"),
    };

    Ok(scalar)
}

fn parse_field_path_expression(
    field_rule: Pair<'_, Rule>,
) -> Result<ScalarExpression, ParserError> {
    let field_location = to_query_location(&field_rule);

    let mut selectors = Vec::new();

    for part in field_rule.into_inner() {
        let part_location = to_query_location(&part);

        let selector = match part.as_rule() {
            Rule::ident => ScalarExpression::Static(StaticScalarExpression::String(
                StringScalarExpression::new(part_location, part.as_str()),
            )),

            Rule::string_literal => {
                let value = parse_standard_string_literal(part)?;
                ScalarExpression::Static(value)
            }
            Rule::integer_literal => {
                let value = parse_standard_integer_literal(part)?;
                ScalarExpression::Static(value)
            }
            other => {
                return Err(ParserError::SyntaxNotSupported(
                    part_location,
                    format!("unexpected rule in field expression: {other:?}"),
                ));
            }
        };

        selectors.push(selector);
    }

    Ok(ScalarExpression::Source(SourceScalarExpression::new(
        field_location,
        ValueAccessor::new_with_selectors(selectors),
    )))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::ottl::parser::OttlPestParser;

    use super::*;

    #[test]
    fn test_parse_scalar_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let state = ParserState::new(input);

            let mut result = OttlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "1",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "(1)",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            "true",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );

        run_test_success(
            "false",
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            )),
        );

        run_test_success(
            "\"hello world\"",
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );

        run_test_success(
            "nil",
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
        );

        run_test_success(
            "resource.attributes.service.name",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "service"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "name"),
                    )),
                ]),
            )),
        );

        run_test_success(
            r#"attributes["group"]"#,
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "group"),
                    )),
                ]),
            )),
        );

        run_test_success(
            "metrics.data_points[0].foo",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "metrics"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "data_points"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "foo"),
                    )),
                ]),
            )),
        );
    }
}
