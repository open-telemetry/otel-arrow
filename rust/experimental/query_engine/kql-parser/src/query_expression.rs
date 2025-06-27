use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{
    Rule, shared_expressions::parse_let_expression, tabular_expressions::parse_tabular_expression,
};

pub(crate) fn parse_query(
    query_rule: Pair<Rule>,
    state: &mut ParserState,
) -> Result<PipelineExpression, ParserError> {
    let query_location = to_query_location(&query_rule);

    let query_rules = query_rule.into_inner();

    let mut pipeline = PipelineExpression::new(query_location);

    for rule in query_rules {
        match rule.as_rule() {
            Rule::let_expression => {
                let let_expression = parse_let_expression(rule, state)?;

                let mut validated = false;

                if let TransformExpression::Set(s) = &let_expression {
                    if let MutableValueExpression::Variable(v) = s.get_destination() {
                        let name = v.get_name();

                        state.push_variable_name(name);
                        validated = true;
                    }
                }

                if !validated {
                    panic!("Unexpected let_expression encountered");
                }

                pipeline.push_expression(DataExpression::Transform(let_expression));
            }
            Rule::tabular_expression => {
                let expressions = parse_tabular_expression(rule, state)?;

                for e in expressions {
                    pipeline.push_expression(e);
                }
            }
            Rule::EOI => {}
            _ => panic!("Unexpected rule in query: {rule}"),
        }
    }

    Ok(pipeline)
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::KqlParser;

    use super::*;

    #[test]
    pub fn test_parse_query() {
        let run_test_success = |input: &str, expected: PipelineExpression| {
            let mut state = ParserState::new(input);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::query, input).unwrap();

            let expression = parse_query(result.next().unwrap(), &mut state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let mut state = ParserState::new(input);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::query, input).unwrap();

            let error = parse_query(result.next().unwrap(), &mut state).unwrap_err();

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

        run_test_success("", PipelineExpression::new(QueryLocation::new_fake()));

        run_test_success(
            "let var1 = 1;",
            PipelineExpression::new_with_expressions(
                QueryLocation::new_fake(),
                vec![DataExpression::Transform(TransformExpression::Set(
                    SetTransformExpression::new(
                        QueryLocation::new_fake(),
                        ImmutableValueExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                1,
                            )),
                        )),
                        MutableValueExpression::Variable(VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            "var1",
                            ValueAccessor::new(),
                        )),
                    ),
                ))],
            ),
        );

        run_test_success(
            "i | extend a = 1",
            PipelineExpression::new_with_expressions(
                QueryLocation::new_fake(),
                vec![DataExpression::Transform(TransformExpression::Set(
                    SetTransformExpression::new(
                        QueryLocation::new_fake(),
                        ImmutableValueExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                1,
                            )),
                        )),
                        MutableValueExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                            )]),
                        )),
                    ),
                ))],
            ),
        );

        run_test_success(
            "let var1 = 1; i | extend a = var1;",
            PipelineExpression::new_with_expressions(
                QueryLocation::new_fake(),
                vec![
                    DataExpression::Transform(TransformExpression::Set(
                        SetTransformExpression::new(
                            QueryLocation::new_fake(),
                            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    1,
                                )),
                            )),
                            MutableValueExpression::Variable(VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                "var1",
                                ValueAccessor::new(),
                            )),
                        ),
                    )),
                    DataExpression::Transform(TransformExpression::Set(
                        SetTransformExpression::new(
                            QueryLocation::new_fake(),
                            ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                                VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "var1",
                                    ValueAccessor::new(),
                                ),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                                )]),
                            )),
                        ),
                    )),
                ],
            ),
        );

        run_test_success(
            "i | extend a = 1; i_other | extend b = 2;",
            PipelineExpression::new_with_expressions(
                QueryLocation::new_fake(),
                vec![
                    DataExpression::Transform(TransformExpression::Set(
                        SetTransformExpression::new(
                            QueryLocation::new_fake(),
                            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    1,
                                )),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                                )]),
                            )),
                        ),
                    )),
                    DataExpression::Transform(TransformExpression::Set(
                        SetTransformExpression::new(
                            QueryLocation::new_fake(),
                            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    2,
                                )),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                                )]),
                            )),
                        ),
                    )),
                ],
            ),
        );

        run_test_failure(
            "let var1 = 1; let var1 = 2;",
            "KS201",
            "A variable with the name 'var1' has already been declared",
        );
    }
}
