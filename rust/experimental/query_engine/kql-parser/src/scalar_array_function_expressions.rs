// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_array_concat_expression(
    array_concat_expression_rule: Pair<Rule>,
    scope: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&array_concat_expression_rule);

    let array_concat_rules = array_concat_expression_rule.into_inner();

    let mut values = Vec::new();

    for rule in array_concat_rules {
        let mut scalar = parse_scalar_expression(rule, scope)?;

        if let Some(t) = scope.try_resolve_value_type(&mut scalar)? {
            if t != ValueType::Array {
                return Err(ParserError::QueryLanguageDiagnostic {
                    location: scalar.get_query_location().clone(),
                    diagnostic_id: "KS234",
                    message: "The expression value must be a dynamic array".into(),
                });
            }
        }

        values.push(scalar);
    }

    Ok(ScalarExpression::Collection(
        CollectionScalarExpression::Concat(CombineScalarExpression::new(
            query_location.clone(),
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(query_location, values),
            )),
        )),
    ))
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_array_concat_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();

            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let state = ParserState::new(input);

            let mut result = KqlPestParser::parse(Rule::logical_expression, input).unwrap();

            let error = parse_scalar_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "array_concat(parse_json('[]'))",
            ScalarExpression::Collection(CollectionScalarExpression::Concat(
                CombineScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Collection(CollectionScalarExpression::List(
                        ListScalarExpression::new(
                            QueryLocation::new_fake(),
                            vec![ScalarExpression::Parse(ParseScalarExpression::Json(
                                ParseJsonScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            "[]",
                                        ),
                                    )),
                                ),
                            ))],
                        ),
                    )),
                ),
            )),
        );

        run_test_success(
            "array_concat(parse_json('[]'), parse_json('[]'))",
            ScalarExpression::Collection(CollectionScalarExpression::Concat(
                CombineScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Collection(CollectionScalarExpression::List(
                        ListScalarExpression::new(
                            QueryLocation::new_fake(),
                            vec![
                                ScalarExpression::Parse(ParseScalarExpression::Json(
                                    ParseJsonScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        ScalarExpression::Static(StaticScalarExpression::String(
                                            StringScalarExpression::new(
                                                QueryLocation::new_fake(),
                                                "[]",
                                            ),
                                        )),
                                    ),
                                )),
                                ScalarExpression::Parse(ParseScalarExpression::Json(
                                    ParseJsonScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        ScalarExpression::Static(StaticScalarExpression::String(
                                            StringScalarExpression::new(
                                                QueryLocation::new_fake(),
                                                "[]",
                                            ),
                                        )),
                                    ),
                                )),
                            ],
                        ),
                    )),
                ),
            )),
        );

        run_test_failure(
            "array_concat(\"hello\")",
            "KS234",
            "The expression value must be a dynamic array",
        );
    }
}
