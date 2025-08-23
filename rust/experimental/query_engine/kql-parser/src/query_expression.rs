// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

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
        let pest_error = if let Err(error) = parse_result {
            error
        } else {
            unreachable!()
        };

        let (start, end) = match pest_error.location {
            pest::error::InputLocation::Pos(p) => (0, p),
            pest::error::InputLocation::Span(s) => s,
        };

        let (line, column) = match pest_error.line_col {
            pest::error::LineColLocation::Pos(p) => p,
            pest::error::LineColLocation::Span(l, _) => l,
        };

        let content = if line > 0 && column > 0 {
            &query
                .lines()
                .nth(line - 1)
                .expect("Query line did not exist")[column - 1..]
        } else {
            &query[start..end]
        };

        errors.push(ParserError::SyntaxNotSupported(
            QueryLocation::new(start, end, line, column)
                .expect("QueryLocation could not be constructed"),
            format!("Syntax '{content}' supplied in query is not supported"),
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
                            let name = v.get_name().get_value();

                            match s.get_source().clone() {
                                ImmutableValueExpression::Scalar(mut scalar) => {
                                    if let ScalarExpression::Static(s) = scalar {
                                        state.push_constant(name, s)
                                    } else {
                                        match scalar.try_resolve_static(
                                            &state.get_pipeline().get_resolution_scope(),
                                        ) {
                                            Ok(Some(ResolvedStaticScalarExpression::Value(s))) => {
                                                state.push_constant(name, s)
                                            }
                                            Ok(None)
                                            | Ok(Some(
                                                ResolvedStaticScalarExpression::Reference(_),
                                            )) => {
                                                state.push_global_variable(name, scalar);
                                            }
                                            Err(e) => errors.push((&e).into()),
                                        }
                                    }
                                }
                            }

                            validated = true;
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

    state.build()
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

        let run_test_failure = |input: &str, expected_id: Option<&str>, expected_msg: &str| {
            let errors = parse_query(input, Default::default()).unwrap_err();

            if expected_id.is_some() {
                if let ParserError::QueryLanguageDiagnostic {
                    location: _,
                    diagnostic_id: id,
                    message: msg,
                } = &errors[0]
                {
                    assert_eq!(expected_id.unwrap(), *id);
                    assert_eq!(expected_msg, msg);
                } else {
                    panic!("Expected QueryLanguageDiagnostic");
                }
            } else if let ParserError::SyntaxNotSupported(_, msg) = &errors[0] {
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected SyntaxNotSupported");
            }
        };

        run_test_success("", PipelineExpressionBuilder::new("").build().unwrap());

        // Note: The let statement becomes an unreferenced constant so the whole
        // expression essentially becomes a no-op.
        run_test_success(
            "let var1 = 1;",
            PipelineExpressionBuilder::new("let var1 = 1;")
                .with_constants(vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )])
                .build()
                .unwrap(),
        );

        // Note: The let statement becomes an unreferenced folded constant so
        // the whole expression essentially becomes a no-op.
        run_test_success(
            "let var1 = (-toint('1') * -1);",
            PipelineExpressionBuilder::new("let var1 = (-toint('1') * -1);")
                .with_constants(vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )])
                .build()
                .unwrap(),
        );

        // Note: var1 becomes a global variable because "now()" cannot be known
        // statically.
        run_test_success(
            "let var1 = now();",
            PipelineExpressionBuilder::new("let var1 = now();")
                .with_global_variables(vec![(
                    "var1",
                    ScalarExpression::Temporal(TemporalScalarExpression::Now(
                        NowScalarExpression::new(QueryLocation::new_fake()),
                    )),
                )])
                .build()
                .unwrap(),
        );

        run_test_success(
            "i | extend a = 1",
            PipelineExpressionBuilder::new("i | extend a = 1")
                .with_constants(vec![])
                .with_expressions(vec![DataExpression::Transform(TransformExpression::Set(
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
                ))])
                .build()
                .unwrap(),
        );

        // Note: This test folds the constants and ends up as if it was written:
        // "source | extend a = 1, attributes['attr'] = 1".
        run_test_success(
            "let var1 = 1; let var2 = 'attr'; source | extend a = var1, attributes[var2] = 1;",
            PipelineExpressionBuilder::new(
                "let var1 = 1; let var2 = 'attr'; source | extend a = var1, attributes[var2] = 1;",
            )
            .with_constants(vec![
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    1,
                )),
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "attr",
                )),
            ])
            .with_expressions(vec![
                DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Constant(
                        ConstantScalarExpression::Reference(
                            ReferenceConstantScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueType::Integer,
                                0,
                            ),
                        ),
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
                ))),
                DataExpression::Transform(TransformExpression::Set(SetTransformExpression::new(
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
                            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                                ReferenceConstantScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    ValueType::String,
                                    1,
                                ),
                            )),
                        ]),
                    )),
                ))),
            ])
            .build()
            .unwrap(),
        );

        run_test_success(
            "i | extend a = 1; i_other | extend b = 2;",
            PipelineExpressionBuilder::new("i | extend a = 1; i_other | extend b = 2;")
                .with_expressions(vec![
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
                ])
                .build()
                .unwrap(),
        );

        run_test_failure(
            "let var1 = 1; let var1 = 2;",
            Some("KS201"),
            "A variable with the name 'var1' has already been declared",
        );

        run_test_failure(
            "s | join some_table on id",
            None,
            "Syntax 'join some_table on id' supplied in query is not supported",
        );
    }
}
