use std::{cell::Ref, slice::Iter};

use data_engine_expressions::*;

use crate::{
    execution_context::ExecutionContext, logical_expressions::execute_logical_expression, *,
};

pub fn execute_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    scalar_expression: &'a ScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    match scalar_expression {
        ScalarExpression::Source(s) => {
            let record = Ref::map(execution_context.get_record().borrow(), |v| {
                v as &dyn AsValue
            });
            let mut selectors = s.get_value_accessor().get_selectors().iter();

            let value = select_from_borrowed_value(execution_context, record, &mut selectors)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {value}"),
            );

            Ok(value)
        }
        ScalarExpression::Attached(a) => {
            if let Some(Some(record)) = execution_context
                .get_attached_records()
                .map(|v| v.get_attached_record(a.get_name().get_value()))
            {
                let mut selectors = a.get_value_accessor().get_selectors().iter();

                let value =
                    select_from_value(execution_context, Value::Map(record), &mut selectors)?;

                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    scalar_expression,
                    || format!("Evaluated as: {value}"),
                );

                Ok(value)
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    scalar_expression,
                    || format!("Evaluated as 'null' because attached record matching name '{}' could not be found", a.get_name().get_value()),
                );
                Ok(ResolvedValue::Computed(OwnedValue::Null))
            }
        }
        ScalarExpression::Variable(v) => {
            let variable = Ref::filter_map(execution_context.get_variables().borrow(), |vars| {
                vars.get(v.get_name().get_value())
            });

            if variable.is_err() {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    scalar_expression,
                    || {
                        format!(
                            "Variable with name '{}' was not found",
                            v.get_name().get_value()
                        )
                    },
                );
                return Ok(ResolvedValue::Computed(OwnedValue::Null));
            }

            let mut selectors = v.get_value_accessor().get_selectors().iter();

            let value =
                select_from_borrowed_value(execution_context, variable.unwrap(), &mut selectors)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {value}"),
            );

            Ok(value)
        }
        ScalarExpression::Static(s) => {
            let value = s.to_value();

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {value}"),
            );

            Ok(ResolvedValue::Value(value))
        }
        ScalarExpression::Constant(c) => match c {
            ConstantScalarExpression::Reference(r) => {
                let constant_id = r.get_constant_id();

                let constant = execution_context
                    .get_pipeline()
                    .get_constant(constant_id)
                    .unwrap_or_else(|| {
                        panic!("Constant for id '{constant_id}' was not found on pipeline")
                    });

                let value = constant.to_value();

                if execution_context
                    .is_diagnostic_level_enabled(RecordSetEngineDiagnosticLevel::Verbose)
                {
                    let (line, column) =
                        constant.get_query_location().get_line_and_column_numbers();
                    execution_context.add_diagnostic(RecordSetEngineDiagnostic::new(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            scalar_expression,
                            format!("Resolved constant with id '{constant_id}' on line {line} at column {column} as: {value}"),
                        ));
                }

                Ok(ResolvedValue::Value(value))
            }
            ConstantScalarExpression::Copy(c) => {
                let constant_id = c.get_constant_id();

                let constant_copy = c.get_value();

                let value = constant_copy.to_value();

                if execution_context
                    .is_diagnostic_level_enabled(RecordSetEngineDiagnosticLevel::Verbose)
                {
                    let (line, column) = constant_copy
                        .get_query_location()
                        .get_line_and_column_numbers();
                    execution_context.add_diagnostic(RecordSetEngineDiagnostic::new(
                            RecordSetEngineDiagnosticLevel::Verbose,
                            scalar_expression,
                            format!("Resolved constant with id '{constant_id}' from copy of value on line {line} at column {column} as: {value}"),
                        ));
                }

                Ok(ResolvedValue::Value(value))
            }
        },
        ScalarExpression::Negate(n) => {
            let inner_value =
                execute_scalar_expression(execution_context, n.get_inner_expression())?;

            let v = match inner_value.to_value() {
                Value::Integer(i) => {
                    ResolvedValue::Computed(OwnedValue::Integer(ValueStorage::<i64>::new(
                        -i.get_value(),
                    )))
                }
                Value::Double(d) => ResolvedValue::Computed(OwnedValue::Double(
                    ValueStorage::<f64>::new(-d.get_value()),
                )),
                _ => {
                    return Err(ExpressionError::TypeMismatch(
                        n.get_query_location().clone(),
                        "Negate expression can only be used with integer and double types".into(),
                    ));
                }
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {v}"),
            );

            Ok(v)
        }
        ScalarExpression::Logical(l) => {
            let value = execute_logical_expression(execution_context, l)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {value}"),
            );

            Ok(ResolvedValue::Computed(OwnedValue::Boolean(
                ValueStorage::new(value),
            )))
        }
        ScalarExpression::Coalesce(c) => {
            for expression in c.get_expressions() {
                let value = execute_scalar_expression(execution_context, expression)?;
                if value.get_value_type() != ValueType::Null {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        scalar_expression,
                        || format!("Evaluated as: {value}"),
                    );

                    return Ok(value);
                }
            }

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || "Evaluated as: null".into(),
            );

            Ok(ResolvedValue::Computed(OwnedValue::Null))
        }
        ScalarExpression::Conditional(c) => {
            let inner_scalar =
                match execute_logical_expression(execution_context, c.get_condition())? {
                    true => c.get_true_expression(),
                    false => c.get_false_expression(),
                };

            let inner_value = execute_scalar_expression(execution_context, inner_scalar)?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {inner_value}"),
            );

            Ok(inner_value)
        }
    }
}

fn select_from_borrowed_value<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    root: Ref<'b, dyn AsValue + 'static>,
    selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'b: 'c,
{
    match selectors.next() {
        Some(s) => {
            let value = execute_scalar_expression(execution_context, s)?;

            let next = match value.to_value() {
                Value::String(map_key) => Ref::filter_map(root, |v| {
                    if let Value::Map(m) = v.to_value() {
                        match m.get(map_key.get_value()) {
                            Some(v) => {
                                execution_context.add_diagnostic_if_enabled(
                                                RecordSetEngineDiagnosticLevel::Verbose,
                                                s,
                                                || format!("Resolved '{:?}' value for key '{}' specified in accessor expression", v.get_value_type(), map_key.get_value()),
                                            );
                                Some(v)
                            }
                            None => {
                                execution_context.add_diagnostic_if_enabled(
                                            RecordSetEngineDiagnosticLevel::Warn,
                                            s,
                                            || format!("Could not find map key '{}' specified in accessor expression", map_key.get_value()),
                                        );
                                None
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    s,
                                    || format!("Could not search for map key '{}' specified in accessor expression because current node is a '{:?}' value", map_key.get_value(), v.get_value_type()),
                                );

                        None
                    }
                }),
                Value::Integer(array_index) => Ref::filter_map(root, |v| {
                    if let Value::Array(a) = v.to_value() {
                        let mut index = array_index.get_value();
                        if index < 0 {
                            index += a.len() as i64;
                        }
                        if index < 0 {
                            execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Warn,
                                        s,
                                        || format!("Array index '{index}' specified in accessor expression is invalid"),
                                    );
                            None
                        } else {
                            match a.get(index as usize) {
                                Some(v) => {
                                    execution_context.add_diagnostic_if_enabled(
                                                    RecordSetEngineDiagnosticLevel::Verbose,
                                                    s,
                                                    || format!("Resolved '{:?}' value for index '{index}' specified in accessor expression", v.get_value_type()),
                                                );
                                    Some(v)
                                }
                                None => {
                                    execution_context.add_diagnostic_if_enabled(
                                                RecordSetEngineDiagnosticLevel::Warn,
                                                s,
                                                || format!("Could not find array index '{index}' specified in accessor expression"),
                                            );
                                    None
                                }
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    s,
                                    || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{:?}' value", array_index.get_value(), v.get_value_type()),
                                );
                        None
                    }
                }),
                _ => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        s,
                        || "Unexpected scalar expression encountered in accessor expression".into(),
                    );
                    return Ok(ResolvedValue::Computed(OwnedValue::Null));
                }
            };

            if let Ok(v) = next {
                select_from_borrowed_value(execution_context, v, selectors)
            } else {
                Ok(ResolvedValue::Computed(OwnedValue::Null))
            }
        }
        None => Ok(ResolvedValue::Borrowed(root)),
    }
}

fn select_from_value<'a, 'b, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    root: Value<'b>,
    selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<ResolvedValue<'b>, ExpressionError> {
    match selectors.next() {
        Some(s) => {
            let value = execute_scalar_expression(execution_context, s)?;

            let next = match value.to_value() {
                Value::String(map_key) => {
                    if let Value::Map(m) = root {
                        match m.get(map_key.get_value()) {
                            Some(v) => {
                                execution_context.add_diagnostic_if_enabled(
                                            RecordSetEngineDiagnosticLevel::Verbose,
                                            s,
                                            || format!("Resolved '{:?}' value for key '{}' specified in accessor expression", v.get_value_type(), map_key.get_value()),
                                        );
                                Some(v.to_value())
                            }
                            None => {
                                execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Warn,
                                        s,
                                        || format!("Could not find map key '{}' specified in accessor expression", map_key.get_value()),
                                    );
                                None
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                s,
                                || format!("Could not search for map key '{}' specified in accessor expression because current node is a '{:?}' value", map_key.get_value(), root.get_value_type()),
                            );
                        None
                    }
                }
                Value::Integer(array_index) => {
                    if let Value::Array(a) = root {
                        let mut index = array_index.get_value();
                        if index < 0 {
                            index += a.len() as i64;
                        }
                        if index < 0 {
                            execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    s,
                                    || format!("Array index '{index}' specified in accessor expression is invalid"),
                                );
                            None
                        } else {
                            match a.get(index as usize) {
                                Some(v) => {
                                    execution_context.add_diagnostic_if_enabled(
                                                RecordSetEngineDiagnosticLevel::Verbose,
                                                s,
                                                || format!("Resolved '{:?}' value for index '{index}' specified in accessor expression", v.get_value_type()),
                                            );
                                    Some(v.to_value())
                                }
                                None => {
                                    execution_context.add_diagnostic_if_enabled(
                                            RecordSetEngineDiagnosticLevel::Warn,
                                            s,
                                            || format!("Could not find array index '{index}' specified in accessor expression"),
                                        );
                                    None
                                }
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                                RecordSetEngineDiagnosticLevel::Warn,
                                s,
                                || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{:?}' value", array_index.get_value(), root.get_value_type()),
                            );

                        None
                    }
                }
                _ => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        s,
                        || "Unexpected scalar expression encountered in accessor expression".into(),
                    );

                    None
                }
            };

            if let Some(v) = next {
                select_from_value(execution_context, v, selectors)
            } else {
                Ok(ResolvedValue::Computed(OwnedValue::Null))
            }
        }
        None => Ok(ResolvedValue::Value(root)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_execute_source_scalar_expression() {
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
            );

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        // Test selecting the root
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new(),
            )),
            Value::Map(&record),
        );

        // Test selecting a string key
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            Value::String(&ValueStorage::new("value1".into())),
        );

        // Test selecting an unknown string key
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "unknown_key",
                    )),
                )]),
            )),
            Value::Null,
        );

        // Test selecting an array index
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
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
            Value::Integer(&ValueStorage::new(1)),
        );

        // Test selecting a negative array index
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
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
            Value::Integer(&ValueStorage::new(3)),
        );

        // Test selecting an invalid array index
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                ]),
            )),
            Value::Null,
        );
    }

    #[test]
    fn test_execute_attached_scalar_expression() {
        let record = TestRecord::new();

        let mut attached_records = TestAttachedRecords::new();

        attached_records.push(
            "resource",
            MapValueStorage::new(HashMap::from([(
                "key1".into(),
                OwnedValue::String(ValueStorage::new("hello world".into())),
            )])),
        );

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                Some(&attached_records),
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        // Test null is returned when record is not found
        run_test(
            ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "invalid"),
                ValueAccessor::new(),
            )),
            Value::Null,
        );

        // Test pathed resolution
        run_test(
            ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            Value::String(&ValueStorage::new("hello world".into())),
        );
    }

    #[test]
    fn test_execute_variable_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            execution_context.get_variables().borrow_mut().set(
                "var1",
                ResolvedValue::Computed(OwnedValue::String(ValueStorage::new(
                    "hello world".into(),
                ))),
            );
            execution_context.get_variables().borrow_mut().set(
                "var2",
                ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                    "key1".into(),
                    OwnedValue::String(ValueStorage::new("hello world".into())),
                )])))),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        // Test null is returned when record is not found
        run_test(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "invalid"),
                ValueAccessor::new(),
            )),
            Value::Null,
        );

        // Test resolution
        run_test(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var1"),
                ValueAccessor::new(),
            )),
            Value::String(&ValueStorage::new("hello world".into())),
        );

        // Test path resolution
        run_test(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var2"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            Value::String(&ValueStorage::new("hello world".into())),
        );
    }

    #[test]
    fn test_execute_constant_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("")
                .with_constants(vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )])
                .build()
                .unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::Integer,
                    0,
                ),
            )),
            Value::Integer(&ValueStorage::new(18)),
        );

        run_test(
            ScalarExpression::Constant(ConstantScalarExpression::Copy(
                CopyConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    1,
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        99,
                    )),
                ),
            )),
            Value::Integer(&ValueStorage::new(99)),
        );
    }

    #[test]
    fn test_execute_negate_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Negate(NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            Value::Integer(&ValueStorage::new(-18)),
        );

        run_test(
            ScalarExpression::Negate(NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Double(
                    DoubleScalarExpression::new(QueryLocation::new_fake(), 18.18),
                )),
            )),
            Value::Double(&ValueStorage::new(-18.18)),
        );
    }

    #[test]
    fn test_execute_logical_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Logical(
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                ))
                .into(),
            ),
            Value::Boolean(&ValueStorage::new(true)),
        );
    }

    #[test]
    fn test_execute_coalesce_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Coalesce(CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                ))],
            )),
            Value::Boolean(&ValueStorage::new(true)),
        );

        run_test(
            ScalarExpression::Coalesce(CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), false),
                    )),
                ],
            )),
            Value::Boolean(&ValueStorage::new(false)),
        );

        run_test(
            ScalarExpression::Coalesce(CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            )),
            Value::Null,
        );
    }

    #[test]
    fn test_execute_conditional_scalar_expression() {
        let record = TestRecord::new();

        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("").build().unwrap();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record.clone(),
            );

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -18),
                )),
            )),
            Value::Integer(&ValueStorage::new(18)),
        );

        run_test(
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -18),
                )),
            )),
            Value::Integer(&ValueStorage::new(-18)),
        );
    }
}
