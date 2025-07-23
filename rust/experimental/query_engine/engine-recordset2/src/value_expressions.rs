use std::{cell::RefMut, ops::Deref, slice::Iter};

use data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, resolved_value_mut::*,
    scalar_expressions::execute_scalar_expression, *,
};

pub fn execute_immutable_value_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    immutable_value_expression: &'a ImmutableValueExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match immutable_value_expression {
        ImmutableValueExpression::Scalar(scalar_expression) => {
            let value = execute_scalar_expression(execution_context, scalar_expression)?;

            if execution_context.is_enabled(LogLevel::Verbose) {
                execution_context.log(LogMessage::new(
                    LogLevel::Verbose,
                    immutable_value_expression,
                    format!("Evaluated as: {value}"),
                ));
            }

            Ok(value)
        }
    }
}

pub fn execute_mutable_value_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    mutable_value_expression: &'a MutableValueExpression,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match mutable_value_expression {
        MutableValueExpression::Source(s) => {
            let record = execution_context.get_record().borrow_mut();
            let mut selectors = s.get_value_accessor().get_selectors().iter();

            let value =
                select_from_borrowed_root_map(execution_context, s, record, &mut selectors)?;

            if execution_context.is_enabled(LogLevel::Verbose) {
                log_mutable_value_expression_evaluated(
                    execution_context,
                    mutable_value_expression,
                    &value,
                );
            }

            Ok(value)
        }
        MutableValueExpression::Variable(v) => {
            let selectors = v.get_value_accessor().get_selectors();

            if selectors.is_empty() {
                let variables = execution_context.get_variables().borrow_mut();

                return Ok(Some(ResolvedValueMut::MapKey {
                    map: variables,
                    key: ResolvedStringValue::Value(v.get_name()),
                }));
            }

            let variable = RefMut::filter_map(
                execution_context.get_variables().borrow_mut(),
                |vars| match vars.get_mut(v.get_name().get_value()) {
                    ValueMutGetResult::Found(v) => Some(v),
                    _ => None,
                },
            );

            if variable.is_err() {
                if execution_context.is_enabled(LogLevel::Verbose) {
                    execution_context.log(LogMessage::new(
                        LogLevel::Verbose,
                        v,
                        format!(
                            "Variable with name '{}' was not found",
                            v.get_name().get_value()
                        ),
                    ));
                }
                return Ok(None);
            }

            let variable = variable.unwrap();
            let mut selectors = selectors.iter();

            select_from_as_value_mut(
                execution_context,
                variable,
                selectors.next().unwrap(),
                &mut selectors,
            )
        }
    }
}

fn log_mutable_value_expression_evaluated<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    mutable_value_expression: &'a MutableValueExpression,
    value: &Option<ResolvedValueMut<'_, '_>>,
) {
    execution_context.log(LogMessage::new(
        LogLevel::Verbose,
        mutable_value_expression,
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
        ),
    ));
}

fn select_from_borrowed_root_map<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    root_expression: &'a dyn Expression,
    root: RefMut<'b, dyn MapValueMut + 'static>,
    selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match selectors.next() {
        Some(s) => select_from_map_value_mut(execution_context, root, s, selectors),
        None => {
            if execution_context.is_enabled(LogLevel::Verbose) {
                execution_context.log(LogMessage::new(
                    LogLevel::Verbose,
                    root_expression,
                    "Resolved root map for accessor expression".into(),
                ));
            }

            Ok(Some(ResolvedValueMut::Map(root)))
        }
    }
}

fn select_from_map_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    current_borrow: RefMut<'b, dyn MapValueMut + 'static>,
    current_scalar: &'a ScalarExpression,
    remaining_selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = execute_scalar_expression(execution_context, current_scalar)?;

    match value.try_resolve_string() {
        Ok(map_key) => {
            if let Some(next_scalar) = remaining_selectors.next() {
                let next_borrow = RefMut::filter_map(current_borrow, |v| {
                    match v.get_mut(map_key.get_value()) {
                        ValueMutGetResult::Found(v) => {
                            if execution_context.is_enabled(LogLevel::Verbose) {
                                execution_context.log(LogMessage::new(
                                            LogLevel::Verbose,
                                            current_scalar,
                                            format!("Resolved '{:?}' value for key '{}' specified in accessor expression", v.get_value_type(), map_key.get_value()),
                                        ));
                            }
                            Some(v)
                        }
                        _ => {
                            if execution_context.is_enabled(LogLevel::Warn) {
                                execution_context.log(LogMessage::new(
                                    LogLevel::Warn,
                                    current_scalar,
                                    format!(
                                        "Could not find map key '{}' specified in accessor expression",
                                        map_key.get_value()
                                    ),
                                ));
                            }
                            None
                        }
                    }
                });

                match next_borrow {
                    Ok(v) => select_from_as_value_mut(
                        execution_context,
                        v,
                        next_scalar,
                        remaining_selectors,
                    ),
                    Err(_) => Ok(None),
                }
            } else {
                Ok(Some(ResolvedValueMut::MapKey {
                    map: current_borrow,
                    key: map_key,
                }))
            }
        }
        Err(v) => {
            if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    current_scalar,
                    format!("Unexpected scalar expression with '{:?}' value encountered when expecting string in accessor expression", v.get_value_type())
                ));
            }
            Ok(None)
        }
    }
}

fn select_from_array_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    current_borrow: RefMut<'b, dyn ArrayValueMut + 'static>,
    current_scalar: &'a ScalarExpression,
    remaining_selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = execute_scalar_expression(execution_context, current_scalar)?;

    if let Value::Integer(array_index) = value.to_value() {
        if let Some(next_scalar) = remaining_selectors.next() {
            let next_borrow = RefMut::filter_map(current_borrow, |v| {
                match validate_array_index(
                    execution_context,
                    current_scalar,
                    array_index.get_value(),
                    v.deref(),
                ) {
                    Some(i) => match v.get_mut(i) {
                        ValueMutGetResult::Found(v) => {
                            if execution_context.is_enabled(LogLevel::Verbose) {
                                execution_context.log(LogMessage::new(
                                                LogLevel::Verbose,
                                                current_scalar,
                                                format!("Resolved '{:?}' value for array index '{}' specified in accessor expression", v.get_value_type(), array_index.get_value()),
                                            ));
                            }
                            Some(v)
                        }
                        _ => {
                            if execution_context.is_enabled(LogLevel::Warn) {
                                execution_context.log(LogMessage::new(
                                                LogLevel::Warn,
                                                current_scalar,
                                                format!("Could not find array index '{}' specified in accessor expression", array_index.get_value()),
                                            ));
                            }
                            None
                        }
                    },
                    None => None,
                }
            });

            match next_borrow {
                Ok(v) => {
                    select_from_as_value_mut(execution_context, v, next_scalar, remaining_selectors)
                }
                Err(_) => Ok(None),
            }
        } else {
            match validate_array_index(
                execution_context,
                current_scalar,
                array_index.get_value(),
                current_borrow.deref(),
            ) {
                Some(i) => Ok(Some(ResolvedValueMut::ArrayIndex {
                    array: current_borrow,
                    index: i,
                })),
                None => Ok(None),
            }
        }
    } else {
        if execution_context.is_enabled(LogLevel::Warn) {
            execution_context.log(LogMessage::new(
                LogLevel::Warn,
                current_scalar,
                format!("Unexpected scalar expression with '{:?}' value encountered when expecting integer in accessor expression", value.get_value_type())
            ));
        }
        Ok(None)
    }
}

fn validate_array_index<'a, TRecord: Record>(
    execution_context: &ExecutionContext<'a, '_, '_, TRecord>,
    expression: &'a ScalarExpression,
    mut index: i64,
    array: &dyn ArrayValueMut,
) -> Option<usize> {
    let length = array.len() as i64;

    if index < 0 {
        index += length;
    }

    if index < 0 || index >= length {
        if execution_context.is_enabled(LogLevel::Warn) {
            execution_context.log(LogMessage::new(
                LogLevel::Warn,
                expression,
                format!("Array index '{index}' specified in accessor expression is invalid"),
            ));
        }

        None
    } else {
        Some(index as usize)
    }
}

fn select_from_as_value_mut<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    current_borrow: RefMut<'b, dyn AsValueMut + 'static>,
    current_scalar: &'a ScalarExpression,
    remaining_selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<Option<ResolvedValueMut<'b, 'c>>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match current_borrow.get_value_type() {
        ValueType::Array => {
            let next_borrow = RefMut::map(current_borrow, |v| {
                if let Some(ValueMut::Array(a)) = v.to_value_mut() {
                    a
                } else {
                    panic!("Array was not returned from ValueMut")
                }
            });

            select_from_array_value_mut(
                execution_context,
                next_borrow,
                current_scalar,
                remaining_selectors,
            )
        }
        ValueType::Map => {
            let next_borrow = RefMut::map(current_borrow, |v| {
                if let Some(ValueMut::Map(m)) = v.to_value_mut() {
                    m
                } else {
                    panic!("Map was not returned from ValueMut")
                }
            });

            select_from_map_value_mut(
                execution_context,
                next_borrow,
                current_scalar,
                remaining_selectors,
            )
        }
        _ => {
            if execution_context.is_enabled(LogLevel::Warn) {
                execution_context.log(LogMessage::new(
                    LogLevel::Warn,
                    current_scalar,
                    format!("Unexpected '{:?}' value encountered when expecting an array or map in accessor expression", current_borrow.get_value_type())
                ));
            }

            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_execute_immutable_value_expression() {
        let record = TestRecord::new();

        let run_test = |immutable_value_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

            let value =
                execute_immutable_value_expression(&execution_context, &immutable_value_expression)
                    .unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ImmutableValueExpression::Scalar(ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            )),
            Value::String(&ValueStorage::new("hello world".into())),
        );
    }

    #[test]
    fn test_execute_source_mutable_value_expression() {
        let record = TestRecord::new()
            .with_key_value(
                "key1".into(),
                OwnedValue::String(ValueStorage::new("value1".into())),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(ValueStorage::new(1)),
                    OwnedValue::Integer(ValueStorage::new(2)),
                    OwnedValue::Integer(ValueStorage::new(3)),
                ])),
            )
            .with_key_value(
                "key3".into(),
                OwnedValue::Map(MapValueStorage::new(HashMap::new())),
            );

        let run_test = |scalar_expression, validate: &dyn Fn(Option<ResolvedValueMut>)| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

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
                        Value::String(&ValueStorage::new("key1".into())),
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
                        Value::String(&ValueStorage::new("sub-key".into())),
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
        let record = TestRecord::new();

        let run_test = |scalar_expression, validate: &dyn Fn(Option<ResolvedValueMut>)| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context =
                ExecutionContext::new(LogLevel::Verbose, &pipeline, None, record.clone());

            {
                let mut variables = execution_context.get_variables().borrow_mut();

                variables.set(
                    "key1",
                    ResolvedValue::Computed(OwnedValue::String(ValueStorage::new("value1".into()))),
                );
                variables.set(
                    "key2",
                    ResolvedValue::Computed(OwnedValue::Array(ArrayValueStorage::new(vec![
                        OwnedValue::Integer(ValueStorage::new(1)),
                        OwnedValue::Integer(ValueStorage::new(2)),
                        OwnedValue::Integer(ValueStorage::new(3)),
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
                        Value::String(&ValueStorage::new("key1".into())),
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
                        Value::String(&ValueStorage::new("key1".into())),
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
}
