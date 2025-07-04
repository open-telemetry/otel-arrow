use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::Parser;

use crate::{
    KqlPestParser, Rule, shared_expressions::parse_let_expression,
    tabular_expressions::parse_tabular_expression,
};

pub(crate) fn parse_query(
    query: &str,
    options: ParserOptions,
) -> Result<PipelineExpression, Vec<ParserError>> {
    let mut errors = Vec::new();

    let mut state = ParserState::new_with_options(query, options);

    let parse_result = KqlPestParser::parse(Rule::query, query);

    if parse_result.is_err() {
        let pest_error = parse_result.unwrap_err();

        let (start, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };

        let (line, column) = match pest_error.line_col {
            pest::error::LineColLocation::Pos(p) => p,
            pest::error::LineColLocation::Span(l, _) => l,
        };

        errors.push(ParserError::SyntaxNotSupported(
            QueryLocation::new(start, end, line, column)
                .expect("QueryLocation could not be constructed"),
            pest_error.variant.message().into(),
        ));

        return Err(errors);
    }

    let query_rules = parse_result.unwrap().next().unwrap().into_inner();

    for rule in query_rules {
        match rule.as_rule() {
            Rule::let_expression => match parse_let_expression(rule, &state) {
                Ok(let_expression) => {
                    let mut validated = false;

                    if let TransformExpression::Set(s) = &let_expression {
                        if let MutableValueExpression::Variable(v) = s.get_destination() {
                            let name = v.get_name();

                            if let ImmutableValueExpression::Scalar(ScalarExpression::Static(s)) =
                                s.get_source()
                            {
                                state.push_constant(name, s.clone());
                                validated = true;
                            }
                        }
                    }

                    if !validated {
                        panic!("Unexpected let_expression encountered");
                    }
                }
                Err(e) => errors.push(e),
            },
            Rule::tabular_expression => match parse_tabular_expression(rule, &state) {
                Ok(expressions) => {
                    for e in expressions {
                        state.push_expression(e);
                    }
                }
                Err(e) => errors.push(e),
            },
            Rule::EOI => {}
            _ => panic!("Unexpected rule in query: {rule}"),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    match state.build() {
        Ok(p) => Ok(p),
        Err(e) => {
            for err in e {
                errors.push(ParserError::SyntaxError(
                    err.get_query_location().clone(),
                    err.to_string(),
                ));
            }
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_query() {
        let run_test_success = |input: &str, expected: PipelineExpression| {
            let expression = parse_query(input, Default::default()).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let errors = parse_query(input, Default::default()).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = &errors[0]
            {
                assert_eq!(expected_id, *id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success("", PipelineExpressionBuilder::new("").build().unwrap());

        // Note: The let statement becomes an unreferenced constant so the whole
        // expression essentially becomes a no-op.
        run_test_success(
            "let var1 = 1;",
            PipelineExpressionBuilder::new("let var1 = 1;")
                .build()
                .unwrap(),
        );

        run_test_success(
            "i | extend a = 1",
            PipelineExpressionBuilder::new_with_expressions(
                "i | extend a = 1",
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
                            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "a",
                                )),
                            )]),
                        )),
                    ),
                ))],
            )
            .build()
            .unwrap(),
        );

        // Note: This test folds the constants and ends up as if it was written:
        // "source | extend a = 1, attributes['attr'] = 1".
        run_test_success(
            "let var1 = 1; let var2 = 'attr'; source | extend a = var1, attributes[var2] = 1;",
            PipelineExpressionBuilder::new_with_expressions(
                "let var1 = 1; let var2 = 'attr'; source | extend a = var1, attributes[var2] = 1;",
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
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "a",
                                    )),
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
                                    1,
                                )),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            "attributes",
                                        ),
                                    )),
                                    ScalarExpression::Static(StaticScalarExpression::String(
                                        StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            "attr",
                                        ),
                                    )),
                                ]),
                            )),
                        ),
                    )),
                ],
            )
            .build()
            .unwrap(),
        );

        run_test_success(
            "i | extend a = 1; i_other | extend b = 2;",
            PipelineExpressionBuilder::new_with_expressions(
                "i | extend a = 1; i_other | extend b = 2;",
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
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "a",
                                    )),
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
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "b",
                                    )),
                                )]),
                            )),
                        ),
                    )),
                ],
            )
            .build()
            .unwrap(),
        );

        run_test_failure(
            "let var1 = 1; let var1 = 2;",
            "KS201",
            "A variable with the name 'var1' has already been declared",
        );
    }
}
