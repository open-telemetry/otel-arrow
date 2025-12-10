// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefMut, ops::Deref, vec::Drain};

use data_engine_expressions::*;

use crate::{execution_context::*, scalars::*, *};

pub fn execute_mutable_value_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    mutable_value_expression: &'a MutableValueExpression,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match mutable_value_expression {
        MutableValueExpression::Source(s) => {
            let value = if let Some(record) = execution_context.get_record() {
                let mut selectors = capture_selector_values_for_mutable_write(
                    execution_context,
                    mutable_value_expression,
                    s.get_value_accessor().get_selectors(),
                )?;

                select_from_borrowed_root_map(
                    execution_context,
                    mutable_value_expression,
                    record.borrow_mut(),
                    selectors.drain(..),
                )
            } else {
                None
            };

            log_mutable_value_expression_evaluated(
                execution_context,
                mutable_value_expression,
                &value,
            );

            Ok(value)
        }
        MutableValueExpression::Variable(v) => {
            let mut selectors = capture_selector_values_for_mutable_write(
                execution_context,
                mutable_value_expression,
                v.get_value_accessor().get_selectors(),
            )?;

            if selectors.is_empty() {
                let variables = execution_context.get_variables().get_local_variables_mut();

                let value = Some(ResolvedValueMut::MapKey {
                    map: variables,
                    key: ResolvedStringValue::Value(v.get_name()),
                });

                log_mutable_value_expression_evaluated(
                    execution_context,
                    mutable_value_expression,
                    &value,
                );

                return Ok(value);
            }

            let variable_name = v.get_name().get_value();

            let variable = match RefMut::filter_map(
                execution_context.get_variables().get_local_variables_mut(),
                |vars| match vars.get_mut(variable_name) {
                    ValueMutGetResult::Found(v) => Some(v),
                    _ => None,
                },
            ) {
                Err(_) => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        mutable_value_expression,
                        || format!("Variable with name '{variable_name}' was not found"),
                    );
                    return Ok(None);
                }
                Ok(v) => v,
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                mutable_value_expression,
                || {
                    format!(
                        "Resolved '{}' variable with name '{variable_name}'",
                        variable.get_value_type()
                    )
                },
            );

            let mut selectors = selectors.drain(..);

            let value = select_from_as_value_mut(
                execution_context,
                mutable_value_expression,
                variable,
                selectors.next().unwrap(),
                selectors,
            );

            log_mutable_value_expression_evaluated(
                execution_context,
                mutable_value_expression,
                &value,
            );

            Ok(value)
        }
        MutableValueExpression::Argument(a) => {
            let mut selectors = capture_selector_values_for_mutable_write(
                execution_context,
                mutable_value_expression,
                a.get_value_accessor().get_selectors(),
            )?;

            let value = execution_context
                .get_arguments()
                .expect("Arguments were not found")
                .get_argument_mut(a.get_argument_id())?;

            let value = if !selectors.is_empty() {
                let mut selectors = selectors.drain(..);

                value.value.and_then(|v| match v {
                    ResolvedValueMut::Map(root) => {
                        select_from_borrowed_root_map(execution_context, a, root, selectors)
                    }
                    ResolvedValueMut::MapKey { map, key } => {
                        match resolve_map_key_mut(execution_context, a, map, key.get_value()) {
                            Some(v) => select_from_as_value_mut(
                                execution_context,
                                a,
                                v,
                                selectors.next().unwrap(),
                                selectors,
                            ),
                            None => None,
                        }
                    }
                    ResolvedValueMut::ArrayIndex { array, index } => {
                        match validate_array_index(
                            execution_context,
                            a,
                            index as i64,
                            array.deref(),
                        ) {
                            Some(i) => {
                                match resolve_array_index_mut(execution_context, a, array, i) {
                                    None => None,
                                    Some(v) => select_from_as_value_mut(
                                        execution_context,
                                        a,
                                        v,
                                        selectors.next().unwrap(),
                                        selectors,
                                    ),
                                }
                            }
                            None => None,
                        }
                    }
                })
            } else {
                value.value
            };

            log_mutable_value_expression_evaluated(
                execution_context,
                mutable_value_expression,
                &value,
            );

            Ok(value)
        }
    }
}

fn capture_selector_values_for_mutable_write<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    mutable_value_expression: &'a MutableValueExpression,
    selectors: &'a [ScalarExpression],
) -> Result<Vec<(&'a ScalarExpression, ResolvedValue<'c>)>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let mut results = Vec::with_capacity(selectors.len());

    for selector in selectors {
        let mut value = execute_scalar_expression(execution_context, selector)?;

        if value.copy_if_borrowed_from_target(execution_context, mutable_value_expression) {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                mutable_value_expression,
                || format!("Copied the resolved selector value '{value}' into temporary storage because the value came from the mutable target"));
        }

        results.push((selector, value));
    }

    Ok(results)
}

fn log_mutable_value_expression_evaluated<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, TRecord>,
    mutable_value_expression: &'a MutableValueExpression,
    value: &Option<ResolvedValueMut<'_, '_>>,
) {
    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        mutable_value_expression,
        || {
            format!(
                "Evaluated as: {}",
                match value.as_ref() {
                    None => "NotFound".into(),
                    Some(v) => match v {
                        ResolvedValueMut::Map(_) => "Root write".into(),
                        ResolvedValueMut::MapKey { map: _, key } => {
                            format!("Map write for '{key}' key")
                        }
                        ResolvedValueMut::ArrayIndex { array: _, index } =>
                            format!("Array write for '{index}' index"),
                    },
                }
            )
        },
    );
}

fn select_from_borrowed_root_map<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    root_expression: &'a dyn Expression,
    root: RefMut<'b, dyn MapValueMut + 'static>,
    mut selectors: Drain<(&'a ScalarExpression, ResolvedValue<'c>)>,
) -> Option<ResolvedValueMut<'b, 'c>>
where
    'a: 'c,
    'b: 'c,
{
    match selectors.next() {
        Some(s) => {
            select_from_map_value_mut(execution_context, root_expression, root, s, selectors)
        }
        None => {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                root_expression,
                || "Resolved root map for accessor expression".into(),
            );
            Some(ResolvedValueMut::Map(root))
        }
    }
}

fn select_from_map_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    current_borrow: RefMut<'b, dyn MapValueMut + 'static>,
    current_selector: (&'a ScalarExpression, ResolvedValue<'c>),
    mut remaining_selectors: Drain<(&'a ScalarExpression, ResolvedValue<'c>)>,
) -> Option<ResolvedValueMut<'b, 'c>>
where
    'a: 'c,
    'b: 'c,
{
    match current_selector.1.try_resolve_string() {
        Ok(map_key) => {
            if let Some(next_selector) = remaining_selectors.next() {
                match resolve_map_key_mut(
                    execution_context,
                    expression,
                    current_borrow,
                    map_key.get_value(),
                ) {
                    Some(v) => select_from_as_value_mut(
                        execution_context,
                        expression,
                        v,
                        next_selector,
                        remaining_selectors,
                    ),
                    None => None,
                }
            } else {
                Some(ResolvedValueMut::MapKey {
                    map: current_borrow,
                    key: map_key,
                })
            }
        }
        Err(v) => {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Warn,
                expression,
                || format!("Unexpected scalar expression with '{:?}' value encountered when expecting string in accessor expression", v.get_value_type())
            );
            None
        }
    }
}

fn select_from_array_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    current_borrow: RefMut<'b, dyn ArrayValueMut + 'static>,
    current_selector: (&'a ScalarExpression, ResolvedValue<'c>),
    mut remaining_selectors: Drain<(&'a ScalarExpression, ResolvedValue<'c>)>,
) -> Option<ResolvedValueMut<'b, 'c>>
where
    'a: 'c,
    'b: 'c,
{
    if let Value::Integer(array_index) = current_selector.1.to_value() {
        match validate_array_index(
            execution_context,
            current_selector.0,
            array_index.get_value(),
            current_borrow.deref(),
        ) {
            Some(i) => {
                if let Some(next_selector) = remaining_selectors.next() {
                    match resolve_array_index_mut(execution_context, expression, current_borrow, i)
                    {
                        Some(v) => select_from_as_value_mut(
                            execution_context,
                            expression,
                            v,
                            next_selector,
                            remaining_selectors,
                        ),
                        None => None,
                    }
                } else {
                    Some(ResolvedValueMut::ArrayIndex {
                        array: current_borrow,
                        index: i,
                    })
                }
            }
            None => None,
        }
    } else {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            expression,
            || format!("Unexpected scalar expression with '{:?}' value encountered when expecting integer in accessor expression", current_selector.1.get_value_type())
        );
        None
    }
}

fn select_from_as_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    current_borrow: RefMut<'b, dyn AsStaticValueMut + 'static>,
    current_selector: (&'a ScalarExpression, ResolvedValue<'c>),
    remaining_selectors: Drain<(&'a ScalarExpression, ResolvedValue<'c>)>,
) -> Option<ResolvedValueMut<'b, 'c>>
where
    'a: 'c,
    'b: 'c,
{
    match current_borrow.get_value_type() {
        ValueType::Array => {
            let next_borrow = RefMut::map(current_borrow, |v| {
                if let Some(StaticValueMut::Array(a)) = v.to_static_value_mut() {
                    a
                } else {
                    panic!("Array was not returned from ValueMut")
                }
            });

            select_from_array_value_mut(
                execution_context,
                expression,
                next_borrow,
                current_selector,
                remaining_selectors,
            )
        }
        ValueType::Map => {
            let next_borrow = RefMut::map(current_borrow, |v| {
                if let Some(StaticValueMut::Map(m)) = v.to_static_value_mut() {
                    m
                } else {
                    panic!("Map was not returned from ValueMut")
                }
            });

            select_from_map_value_mut(
                execution_context,
                expression,
                next_borrow,
                current_selector,
                remaining_selectors,
            )
        }
        _ => {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Warn,
                expression,
                || format!("Unexpected '{:?}' value encountered when expecting an array or map in accessor expression", current_borrow.get_value_type())
            );
            None
        }
    }
}

pub(crate) fn validate_array_index<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    mut index: i64,
    array: &dyn ArrayValueMut,
) -> Option<usize> {
    let length = array.len() as i64;

    if index < 0 {
        index += length;
    }

    if index < 0 || index >= length {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            expression,
            || format!("Array index '{index}' specified in accessor expression is invalid"),
        );
        None
    } else {
        Some(index as usize)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_execute_source_mutable_value_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::String(StringValueStorage::new("value1".into())),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(IntegerValueStorage::new(1)),
                    OwnedValue::Integer(IntegerValueStorage::new(2)),
                    OwnedValue::Integer(IntegerValueStorage::new(3)),
                ])),
            )
            .with_key_value(
                "key3".into(),
                OwnedValue::Map(MapValueStorage::new(HashMap::new())),
            );

        let run_test = |scalar_expression, validate: &dyn Fn(Option<ResolvedValueMut>)| {
            let mut test = TestExecutionContext::new().with_record(record.clone());

            let execution_context = test.create_execution_context();

            let value =
                execute_mutable_value_expression(&execution_context, &scalar_expression).unwrap();

            (validate)(value);
        };

        // Test selecting the root
        run_test(
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new(),
            )),
            &|v| {
                assert!(matches!(v, Some(ResolvedValueMut::Map(_))));
            },
        );

        // Test selecting a string key
        run_test(
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("key1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            },
        );

        // Test selecting a map key
        run_test(
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key3"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "sub-key"),
                    )),
                ]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("sub-key".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            },
        );

        // Test selecting an array index
        run_test(
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                ]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(0, index);
                } else {
                    panic!()
                }
            },
        );

        // Test selecting a negative array index
        run_test(
            MutableValueExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                    )),
                ]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(2, index);
                } else {
                    panic!()
                }
            },
        );
    }

    #[test]
    fn test_execute_variable_mutable_value_expression() {
        let run_test = |scalar_expression, validate: &dyn Fn(Option<ResolvedValueMut>)| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            {
                let mut variables = execution_context.get_variables().get_local_variables_mut();

                variables.set(
                    "key1",
                    ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                        "value1".into(),
                    ))),
                );
                variables.set(
                    "key2",
                    ResolvedValue::Computed(OwnedValue::Array(ArrayValueStorage::new(vec![
                        OwnedValue::Integer(IntegerValueStorage::new(1)),
                        OwnedValue::Integer(IntegerValueStorage::new(2)),
                        OwnedValue::Integer(IntegerValueStorage::new(3)),
                    ]))),
                );
                variables.set(
                    "key3",
                    ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::new()))),
                );
            }

            let value =
                execute_mutable_value_expression(&execution_context, &scalar_expression).unwrap();

            (validate)(value);
        };

        // Test selecting a variable
        run_test(
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                ValueAccessor::new(),
            )),
            &|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("key1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            },
        );

        // Test selecting a string key
        run_test(
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key3"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("key1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            },
        );

        // Test selecting an array index
        run_test(
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                    )),
                )]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(0, index);
                } else {
                    panic!()
                }
            },
        );

        // Test selecting a negative array index
        run_test(
            MutableValueExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1,
                    )),
                )]),
            )),
            &|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(2, index);
                } else {
                    panic!()
                }
            },
        );
    }

    #[allow(clippy::type_complexity)]
    #[test]
    fn test_execute_argument_mutable_value_expression() {
        let run_test = |expression,
                        validate: Option<&dyn Fn(Option<ResolvedValueMut>)>,
                        error_message: Option<&str>| {
            let record = TestRecord::new().with_key_value(
                "key1".into(),
                OwnedValue::String(StringValueStorage::new("value1".into())),
            );

            let mut test = TestExecutionContext::new().with_record(record);

            let execution_context = test.create_execution_context();

            {
                let mut variables = execution_context.get_variables().get_local_variables_mut();

                variables.set(
                    "var1",
                    ResolvedValue::Computed(OwnedValue::Array(ArrayValueStorage::new(vec![
                        OwnedValue::Integer(IntegerValueStorage::new(1)),
                        OwnedValue::Integer(IntegerValueStorage::new(2)),
                        OwnedValue::Integer(IntegerValueStorage::new(3)),
                    ]))),
                );
            }

            let arguments = vec![
                InvokeFunctionArgument::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                )),
                InvokeFunctionArgument::MutableValue(MutableValueExpression::Source(
                    SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "key1",
                            )),
                        )]),
                    ),
                )),
                InvokeFunctionArgument::MutableValue(MutableValueExpression::Source(
                    SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new()),
                )),
                InvokeFunctionArgument::MutableValue(MutableValueExpression::Variable(
                    VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                        ValueAccessor::new(),
                    ),
                )),
            ];

            let execution_context_arguments = ExecutionContextArgumentContainer {
                parent_execution_context: &execution_context,
                arguments: &arguments,
            };

            let scope = execution_context.create_scope(Some(&execution_context_arguments));

            if let Some(expected_msg) = error_message {
                let error = execute_mutable_value_expression(&scope, &expression).unwrap_err();

                if let ExpressionError::NotSupported(_, msg) = error {
                    assert_eq!(expected_msg, msg);
                } else {
                    panic!()
                }
            } else {
                let value = execute_mutable_value_expression(&scope, &expression).unwrap();

                (validate.unwrap())(value);
            }
        };

        // Test selecting a scalar value. Scalar values cannot be mutated so this leads to an error.
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::String),
                0,
                ValueAccessor::new(),
            )),
            None,
            Some("Argument for id '0' cannot be mutated"),
        );

        // Test selecting a source value (source.key1)
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::String),
                1,
                ValueAccessor::new(),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("key1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting source value
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::Map),
                2,
                ValueAccessor::new(),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::Map(m)) = v {
                    assert_eq!(1, m.len());
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting source value with a selector (key1) to a string
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::String),
                2,
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("key1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting a variable value (var1)
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                None,
                3,
                ValueAccessor::new(),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::MapKey { map: _, key }) = v {
                    assert_eq!(
                        Value::String(&StringValueStorage::new("var1".into())),
                        key.to_value()
                    );
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting a variable value (var1) to an array index (0)
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                None,
                3,
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                    )),
                )]),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(0, index);
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting a variable value (var1) to an array index (-1)
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                None,
                3,
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1,
                    )),
                )]),
            )),
            Some(&|v| {
                if let Some(ResolvedValueMut::ArrayIndex { array: _, index }) = v {
                    assert_eq!(2, index);
                } else {
                    panic!()
                }
            }),
            None,
        );

        // Test selecting a variable value (var1) to an invalid array index (10) which returns None/Null
        run_test(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                None,
                3,
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        10,
                    )),
                )]),
            )),
            Some(&|v| {
                assert!(v.is_none());
            }),
            None,
        );
    }
}
