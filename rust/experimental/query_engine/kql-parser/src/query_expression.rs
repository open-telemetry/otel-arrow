// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::Parser;

use crate::{
    KqlPestParser, Rule, scalar_expression::parse_scalar_expression, shared_expressions::*,
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
        errors.push(ParserError::from_pest_error(query, pest_error));

        return Err(errors);
    }

    let query_rules = parse_result.unwrap().next().unwrap().into_inner();

    for rule in query_rules {
        match rule.as_rule() {
            Rule::variable_definition_expression => {
                match parse_variable_definition_expression(rule, &state) {
                    Ok(variable_definition_expression) => {
                        let mut validated = false;

                        if let TransformExpression::Set(s) = &variable_definition_expression {
                            if let MutableValueExpression::Variable(v) = s.get_destination() {
                                let name = v.get_name().get_value();

                                let mut scalar = s.get_source().clone();

                                if let ScalarExpression::Static(s) = scalar {
                                    state.push_constant(name, s);
                                } else {
                                    let c = match scalar.try_resolve_static(
                                        &state.get_pipeline().get_resolution_scope(),
                                    ) {
                                        Ok(Some(ResolvedStaticScalarExpression::Computed(s))) => {
                                            Some(s)
                                        }
                                        Ok(Some(
                                            ResolvedStaticScalarExpression::FoldEligibleReference(
                                                s,
                                            ),
                                        )) => Some(s.clone()),
                                        Ok(None)
                                        | Ok(Some(ResolvedStaticScalarExpression::Reference(_))) => {
                                            None
                                        }
                                        Err(e) => {
                                            errors.push((&e).into());
                                            continue;
                                        }
                                    };

                                    match c {
                                        Some(c) => {
                                            state.push_constant(name, c);
                                        }
                                        None => {
                                            state.push_global_variable(name, scalar);
                                        }
                                    }
                                }

                                validated = true;
                            }
                        }

                        if !validated {
                            panic!("Unexpected variable_definition_expression encountered");
                        }
                    }
                    Err(e) => errors.push(e),
                }
            }
            Rule::user_defined_function_definition_expression => {
                let query_location = to_query_location(&rule);

                let mut udf_rules = rule.into_inner();

                let identifier_rule = udf_rules.next().unwrap();

                let name = identifier_rule.as_str().trim();
                if state.is_well_defined_identifier(name) {
                    errors.push(ParserError::QueryLanguageDiagnostic {
                        location: to_query_location(&identifier_rule).clone(),
                        diagnostic_id: "KS201",
                        message: format!(
                            "A variable or function with the name '{name}' has already been declared"
                        ),
                    });
                }

                let mut arguments = HashMap::new();
                let mut parameters = Vec::new();
                let mut parameter_names = Vec::new();
                let mut default_values = HashMap::new();
                let mut expressions = Vec::new();
                let mut return_type = None;

                for rule in udf_rules {
                    match rule.as_rule() {
                        Rule::user_defined_function_parameter_definition_expression => {
                            let location = to_query_location(&rule);
                            let mut rules = rule.into_inner();
                            let parameter_name = rules.next().unwrap().as_str();

                            let table_schema_or_type = rules.next().unwrap();

                            match table_schema_or_type.as_rule() {
                                Rule::type_literal => {
                                    let value_type = parse_type_literal(table_schema_or_type.as_str());
                                    if let Some(default_value) = rules.next() {
                                        match parse_scalar_expression(default_value, &state) {
                                            Ok(mut s) => {
                                                match s.try_resolve_static(&state.get_pipeline().get_resolution_scope()) {
                                                    Ok(Some(resolved_static)) => {
                                                        if let Some(value_type) = value_type.as_ref()
                                                            && *value_type != resolved_static.get_value_type() {
                                                            errors.push(ParserError::QueryLanguageDiagnostic {
                                                                location: s.get_query_location().clone(),
                                                                diagnostic_id: "KS141",
                                                                message: format!("The expression must have the type '{value_type}'"),
                                                            });
                                                            continue;
                                                        }
                                                        default_values.insert(parameter_name.into(), s);
                                                    }
                                                    Ok(None) => {
                                                        errors.push(ParserError::QueryLanguageDiagnostic {
                                                            location: s.get_query_location().clone(),
                                                            diagnostic_id: "KS132",
                                                            message: "The expression must be a literal scalar value".into(),
                                                        });
                                                        continue;
                                                    }
                                                    Err(e) => {
                                                        errors.push(ParserError::from(&e));
                                                        continue;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                errors.push(e);
                                                continue;
                                            }
                                        }
                                    }

                                    if arguments.insert(parameter_name.into(), (parameters.len(), value_type.clone())).is_some() {
                                        errors.push(ParserError::SyntaxError(location.clone(), format!("Parameter with name '{parameter_name}' was already defined")));
                                    }

                                    parameter_names.push(parameter_name.into());
                                    parameters.push(PipelineFunctionParameter::new(location, PipelineFunctionParameterType::Scalar(value_type)));
                                }
                                Rule::user_defined_function_tabular_parameter_definition_expression => {
                                    let location = to_query_location(&table_schema_or_type);
                                    let tabular_parameter = table_schema_or_type.into_inner();
                                    if !tabular_parameter.as_str().is_empty() {
                                        // Note: Empty means user did wildcard schema a la TableName:(*)
                                        errors.push(ParserError::SyntaxNotSupported(location, "Tabular schema definition is not supported".into()));
                                        continue;
                                    }

                                    if arguments.insert(parameter_name.into(), (parameters.len(), Some(ValueType::Map))).is_some() {
                                        errors.push(ParserError::SyntaxError(location.clone(), format!("Parameter with name '{parameter_name}' was already defined")));
                                    }

                                    parameter_names.push(parameter_name.into());
                                    parameters.push(PipelineFunctionParameter::new(location, PipelineFunctionParameterType::MutableValue(None)));
                                }
                                _ => panic!("Unexpected rule in user_defined_function_parameter_definition_expression: {table_schema_or_type}"),
                            }
                        }
                        Rule::user_defined_function_body_definition_expression => {
                            let scope = state
                                .create_scope(Default::default())
                                .without_source()
                                .with_arguments(arguments);

                            for rule in rule.into_inner() {
                                match rule.as_rule() {
                                    Rule::variable_definition_expression => {
                                        match parse_variable_definition_expression(rule, &scope) {
                                            Ok(t) => {
                                                if let TransformExpression::Set(s) = &t
                                                    && let MutableValueExpression::Variable(v) =
                                                        s.get_destination()
                                                {
                                                    let name = v.get_name().get_value();

                                                    scope.push_variable_name(name);
                                                } else {
                                                    panic!(
                                                        "Unexpected TransformExpression encountered"
                                                    );
                                                }

                                                expressions
                                                    .push(PipelineFunctionExpression::Transform(t));
                                            }
                                            Err(e) => errors.push(e),
                                        }
                                    }
                                    Rule::scalar_expression => {
                                        match parse_scalar_expression(rule, &scope) {
                                            Ok(mut s) => {
                                                match state.try_resolve_value_type(&mut s) {
                                                    Ok(Some(value_type)) => {
                                                        return_type = Some(value_type);
                                                    }
                                                    Ok(None) => {}
                                                    Err(e) => errors.push(e),
                                                }
                                                expressions
                                                    .push(PipelineFunctionExpression::Return(s));
                                            }
                                            Err(e) => errors.push(e),
                                        }
                                    }
                                    _ => panic!(
                                        "Unexpected rule in user_defined_function_body_definition_expression: {rule}"
                                    ),
                                }
                            }
                            break;
                        }
                        _ => panic!(
                            "Unexpected rule in user_defined_function_definition_expression: {rule}"
                        ),
                    }
                }

                let func = PipelineFunction::new_with_expressions(
                    query_location,
                    parameters,
                    return_type,
                    expressions,
                );

                state.push_function(name, func, parameter_names, default_values);
            }
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

            if let Some(expected_id) = expected_id {
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
            } else if let ParserError::SyntaxNotSupported(_, msg) = &errors[0] {
                assert_eq!(expected_msg, msg);
            } else if let ParserError::SyntaxError(_, msg) = &errors[0] {
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Unexpected error type");
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

        run_test_success(
            "let a = 1; let b = a; let c = b;",
            PipelineExpressionBuilder::new("let a = 1; let b = a; let c = b;")
                .with_constants(vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                    StaticScalarExpression::Constant(CopyConstantScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    StaticScalarExpression::Constant(CopyConstantScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                ])
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
                    ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueType::Integer,
                        0,
                        ValueAccessor::new(),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
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
                            ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueType::String,
                                1,
                                ValueAccessor::new(),
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
                        ),
                    )),
                    DataExpression::Transform(TransformExpression::Set(
                        SetTransformExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
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
            "A variable or function with the name 'var1' has already been declared",
        );

        run_test_failure(
            "s | join some_table on id",
            None,
            "Syntax 'join some_table on id' supplied in query is not supported",
        );

        run_test_success(
            "let f = () {};",
            PipelineExpressionBuilder::new("let f = () {};")
                .with_functions(vec![PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![],
                )])
                .build()
                .unwrap(),
        );

        run_test_success(
            "let f = (a1:long) { };",
            PipelineExpressionBuilder::new("let f = (a1:long) { };")
                .with_functions(vec![PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![PipelineFunctionParameter::new(
                        QueryLocation::new_fake(),
                        PipelineFunctionParameterType::Scalar(Some(ValueType::Integer)),
                    )],
                    None,
                    vec![],
                )])
                .build()
                .unwrap(),
        );

        run_test_success(
            "let f = (a1:long, a2: dynamic){};",
            PipelineExpressionBuilder::new("let f = (a1:long, a2: dynamic){};")
                .with_functions(vec![PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![
                        PipelineFunctionParameter::new(
                            QueryLocation::new_fake(),
                            PipelineFunctionParameterType::Scalar(Some(ValueType::Integer)),
                        ),
                        PipelineFunctionParameter::new(
                            QueryLocation::new_fake(),
                            PipelineFunctionParameterType::Scalar(None),
                        ),
                    ],
                    None,
                    vec![],
                )])
                .build()
                .unwrap(),
        );

        run_test_success(
            "let f = (a1:long=1, a2:string='hello', a3:(*)){1;};",
            PipelineExpressionBuilder::new("let f = (a1:long=1, a2:string='hello', a3:(*)){1;};")
                .with_functions(vec![PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![
                        PipelineFunctionParameter::new(
                            QueryLocation::new_fake(),
                            PipelineFunctionParameterType::Scalar(Some(ValueType::Integer)),
                        ),
                        PipelineFunctionParameter::new(
                            QueryLocation::new_fake(),
                            PipelineFunctionParameterType::Scalar(Some(ValueType::String)),
                        ),
                        PipelineFunctionParameter::new(
                            QueryLocation::new_fake(),
                            PipelineFunctionParameterType::MutableValue(None),
                        ),
                    ],
                    Some(ValueType::Integer),
                    vec![PipelineFunctionExpression::Return(
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                        )),
                    )],
                )])
                .build()
                .unwrap(),
        );

        run_test_success(
            "let f = (){ let a = 'hello world'; a };",
            PipelineExpressionBuilder::new("let f = (){ let a = 'hello world'; a };")
                .with_functions(vec![PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![
                        PipelineFunctionExpression::Transform(TransformExpression::Set(
                            SetTransformExpression::new(
                                QueryLocation::new_fake(),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "hello world",
                                    ),
                                )),
                                MutableValueExpression::Variable(VariableScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                                    ValueAccessor::new(),
                                )),
                            ),
                        )),
                        PipelineFunctionExpression::Return(ScalarExpression::Variable(
                            VariableScalarExpression::new(
                                QueryLocation::new_fake(),
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                                ValueAccessor::new(),
                            ),
                        )),
                    ],
                )])
                .build()
                .unwrap(),
        );

        run_test_failure(
            "let f = (a:int, a:int){};",
            None,
            "Parameter with name 'a' was already defined",
        );

        run_test_failure(
            "let f = (a:int, a:(*)){};",
            None,
            "Parameter with name 'a' was already defined",
        );

        run_test_failure(
            "let f = (T:(a:int, b:int)){};",
            None,
            "Tabular schema definition is not supported",
        );

        run_test_failure(
            "let f = (a:string=1){};",
            Some("KS141"),
            "The expression must have the type 'String'",
        );

        run_test_failure(
            "let f = (a:datetime=now()){};",
            Some("KS132"),
            "The expression must be a literal scalar value",
        );
    }
}
