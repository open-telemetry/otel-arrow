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
                v as &dyn AsStaticValue
            });
            let mut selectors = s.get_value_accessor().get_selectors().iter();

            let value = select_from_borrowed_value(
                execution_context,
                BorrowSource::Source,
                record,
                &mut selectors,
            )?;

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

            let value = select_from_borrowed_value(
                execution_context,
                BorrowSource::Variable,
                variable.unwrap(),
                &mut selectors,
            )?;

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
                Value::Integer(i) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(-i.get_value()),
                )),
                Value::Double(d) => ResolvedValue::Computed(OwnedValue::Double(
                    DoubleValueStorage::new(-d.get_value()),
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
                BooleanValueStorage::new(value),
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
        ScalarExpression::Case(c) => {
            let expressions_with_conditions = c.get_expressions_with_conditions();

            // Evaluate conditions in order and return first matching result
            for (condition, expression) in expressions_with_conditions {
                if execute_logical_expression(execution_context, condition)? {
                    let inner_value = execute_scalar_expression(execution_context, expression)?;

                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        scalar_expression,
                        || format!("Evaluated as: {inner_value}"),
                    );

                    return Ok(inner_value);
                }
            }

            // No condition matched, return else expression
            let inner_value =
                execute_scalar_expression(execution_context, c.get_else_expression())?;

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {inner_value}"),
            );

            Ok(inner_value)
        }
        ScalarExpression::Convert(c) => {
            let value = match c {
                ConvertScalarExpression::Boolean(c) => {
                    let v = execute_scalar_expression(execution_context, c.get_inner_expression())?;

                    if let Some(b) = v.to_value().convert_to_bool() {
                        ResolvedValue::Computed(OwnedValue::Boolean(BooleanValueStorage::new(b)))
                    } else {
                        ResolvedValue::Computed(OwnedValue::Null)
                    }
                }
                ConvertScalarExpression::Double(c) => {
                    let v = execute_scalar_expression(execution_context, c.get_inner_expression())?;

                    if let Some(d) = v.to_value().convert_to_double() {
                        ResolvedValue::Computed(OwnedValue::Double(DoubleValueStorage::new(d)))
                    } else {
                        ResolvedValue::Computed(OwnedValue::Null)
                    }
                }
                ConvertScalarExpression::Integer(c) => {
                    let v = execute_scalar_expression(execution_context, c.get_inner_expression())?;

                    if let Some(i) = v.to_value().convert_to_integer() {
                        ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(i)))
                    } else {
                        ResolvedValue::Computed(OwnedValue::Null)
                    }
                }
                ConvertScalarExpression::String(c) => {
                    let v = execute_scalar_expression(execution_context, c.get_inner_expression())?;

                    if v.get_value_type() == ValueType::String {
                        v
                    } else {
                        let mut string_value = None;
                        v.to_value().convert_to_string(&mut |s| {
                            string_value = Some(StringValueStorage::new(s.into()))
                        });
                        ResolvedValue::Computed(OwnedValue::String(
                            string_value.expect("Inner value did not return a string"),
                        ))
                    }
                }
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {value}"),
            );

            Ok(value)
        }
        ScalarExpression::Length(l) => {
            let inner_value =
                execute_scalar_expression(execution_context, l.get_inner_expression())?;

            let v = match inner_value.to_value() {
                Value::String(s) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(s.get_value().chars().count() as i64),
                )),
                Value::Array(a) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(a.len() as i64),
                )),
                Value::Map(m) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(m.len() as i64),
                )),
                _ => ResolvedValue::Computed(OwnedValue::Null),
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {v}"),
            );

            Ok(v)
        }
        ScalarExpression::ReplaceString(r) => {
            let haystack_value =
                execute_scalar_expression(execution_context, r.get_haystack_expression())?;
            let needle_value =
                execute_scalar_expression(execution_context, r.get_needle_expression())?;
            let replacement_value =
                execute_scalar_expression(execution_context, r.get_replacement_expression())?;

            let v = if let Some(result) = Value::replace_matches(
                r.get_query_location(),
                &haystack_value.to_value(),
                &needle_value.to_value(),
                &replacement_value.to_value(),
                r.get_case_insensitive(),
            ) {
                ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(result)))
            } else {
                ResolvedValue::Computed(OwnedValue::Null)
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {v}"),
            );

            Ok(v)
        }
        ScalarExpression::Slice(s) => {
            let inner_value = execute_scalar_expression(execution_context, s.get_source())?;

            let range_start_inclusive = match s.get_range_start_inclusive() {
                Some(start) => s.validate_resolved_range_value(
                    "start",
                    execute_scalar_expression(execution_context, start)?.to_value(),
                )?,
                None => 0,
            };
            let range_end_exclusive = match s.get_range_end_exclusive() {
                Some(end) => Some(s.validate_resolved_range_value(
                    "end",
                    execute_scalar_expression(execution_context, end)?.to_value(),
                )?),
                None => None,
            };

            let v = match inner_value.try_resolve_string() {
                Ok(string_value) => {
                    let range_end_exclusive = s.validate_slice_range(
                        "String",
                        string_value.get_value().chars().count(),
                        range_start_inclusive,
                        range_end_exclusive,
                    )?;

                    ResolvedValue::Slice(
                        string_value.get_borrow_source(),
                        Slice::String(StringSlice::new(
                            string_value,
                            range_start_inclusive,
                            range_end_exclusive,
                        )),
                    )
                }
                Err(v) => match v.try_resolve_array() {
                    Ok(array_value) => {
                        let range_end_exclusive = s.validate_slice_range(
                            "Array",
                            array_value.len(),
                            range_start_inclusive,
                            range_end_exclusive,
                        )?;

                        ResolvedValue::Slice(
                            array_value.get_borrow_source(),
                            Slice::Array(ArraySlice::new(
                                array_value,
                                range_start_inclusive,
                                range_end_exclusive,
                            )),
                        )
                    }
                    Err(_) => ResolvedValue::Computed(OwnedValue::Null),
                },
            };

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {v}"),
            );

            Ok(v)
        }
        ScalarExpression::ParseJson(p) => {
            let inner_value =
                execute_scalar_expression(execution_context, p.get_inner_expression())?;

            let v = ResolvedValue::Computed(match inner_value.to_value() {
                Value::String(s) => {
                    OwnedValue::from_json(s.get_value()).unwrap_or(OwnedValue::Null)
                }
                _ => OwnedValue::Null,
            });

            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                scalar_expression,
                || format!("Evaluated as: {v}"),
            );

            Ok(v)
        }
    }
}

fn select_from_borrowed_value<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    borrow_source: BorrowSource,
    borrow: Ref<'b, dyn AsStaticValue + 'static>,
    selectors: &mut Iter<'a, ScalarExpression>,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'b: 'c,
{
    match selectors.next() {
        Some(s) => {
            let value = execute_scalar_expression(execution_context, s)?;

            let next = match value.to_value() {
                Value::String(map_key) => Ref::filter_map(borrow, |v| {
                    if let Value::Map(m) = v.to_value() {
                        match m.get(map_key.get_value()) {
                            Some(v) => {
                                execution_context.add_diagnostic_if_enabled(
                                                RecordSetEngineDiagnosticLevel::Verbose,
                                                s,
                                                || format!("Resolved '{}' value for key '{}' specified in accessor expression", v.to_value(), map_key.get_value()),
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
                Value::Integer(array_index) => Ref::filter_map(borrow, |v| {
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
                                                    || format!("Resolved '{}' value for index '{index}' specified in accessor expression", v.to_value()),
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
                select_from_borrowed_value(execution_context, borrow_source, v, selectors)
            } else {
                Ok(ResolvedValue::Computed(OwnedValue::Null))
            }
        }
        None => Ok(ResolvedValue::Borrowed(borrow_source, borrow)),
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
                                            || format!("Resolved '{}' value for key '{}' specified in accessor expression", v.to_value(), map_key.get_value()),
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
                                                || format!("Resolved '{}' value for index '{index}' specified in accessor expression", v.to_value()),
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
                OwnedValue::String(StringValueStorage::new("value1".into())),
            )
            .with_key_value(
                "key2".into(),
                OwnedValue::Array(ArrayValueStorage::new(vec![
                    OwnedValue::Integer(IntegerValueStorage::new(1)),
                    OwnedValue::Integer(IntegerValueStorage::new(2)),
                    OwnedValue::Integer(IntegerValueStorage::new(3)),
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
            Value::String(&StringValueStorage::new("value1".into())),
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
            Value::Integer(&IntegerValueStorage::new(1)),
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
            Value::Integer(&IntegerValueStorage::new(3)),
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
                OwnedValue::String(StringValueStorage::new("hello world".into())),
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
            Value::String(&StringValueStorage::new("hello world".into())),
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
                ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                    "hello world".into(),
                ))),
            );
            execution_context.get_variables().borrow_mut().set(
                "var2",
                ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                    "key1".into(),
                    OwnedValue::String(StringValueStorage::new("hello world".into())),
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
            Value::String(&StringValueStorage::new("hello world".into())),
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
            Value::String(&StringValueStorage::new("hello world".into())),
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
            Value::Integer(&IntegerValueStorage::new(18)),
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
            Value::Integer(&IntegerValueStorage::new(99)),
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
            Value::Integer(&IntegerValueStorage::new(-18)),
        );

        run_test(
            ScalarExpression::Negate(NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Double(
                    DoubleScalarExpression::new(QueryLocation::new_fake(), 18.18),
                )),
            )),
            Value::Double(&DoubleValueStorage::new(-18.18)),
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
                    false,
                ))
                .into(),
            ),
            Value::Boolean(&BooleanValueStorage::new(true)),
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
            Value::Boolean(&BooleanValueStorage::new(true)),
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
            Value::Boolean(&BooleanValueStorage::new(false)),
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
            Value::Integer(&IntegerValueStorage::new(18)),
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
            Value::Integer(&IntegerValueStorage::new(-18)),
        );
    }

    #[test]
    fn test_execute_convert_scalar_expression() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, Value)>)
        where
            F: Fn(ConversionScalarExpression) -> ConvertScalarExpression,
        {
            for (inner, expected) in input {
                let e = ScalarExpression::Convert(build(ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    inner,
                )));

                let pipeline = Default::default();

                let record = TestRecord::new();

                let execution_context = ExecutionContext::new(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    &pipeline,
                    None,
                    record,
                );

                let actual = execute_scalar_expression(&execution_context, &e).unwrap();
                assert_eq!(expected, actual.to_value());
            }
        }

        run_test(
            ConvertScalarExpression::Boolean,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.0),
                    )),
                    Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "true"),
                    )),
                    Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            ConvertScalarExpression::Double,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.0),
                    )),
                    Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.0,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.0,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18.0"),
                    )),
                    Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.0,
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            ConvertScalarExpression::Integer,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.0),
                    )),
                    Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                    )),
                    Value::Null,
                ),
            ],
        );

        run_test(
            ConvertScalarExpression::String,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Value::String(&StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "true",
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.18),
                    )),
                    Value::String(&StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "18.18",
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Value::String(&StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "18",
                    )),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "")),
                ),
            ],
        );
    }

    #[test]
    fn test_execute_length_scalar_expression() {
        fn run_test(input: Vec<(ScalarExpression, Value)>) {
            for (inner, expected) in input {
                let e = ScalarExpression::Length(LengthScalarExpression::new(
                    QueryLocation::new_fake(),
                    inner,
                ));

                let pipeline = Default::default();

                let record = TestRecord::new();

                let execution_context = ExecutionContext::new(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    &pipeline,
                    None,
                    record,
                );

                let actual = execute_scalar_expression(&execution_context, &e).unwrap();
                assert_eq!(expected, actual.to_value());
            }
        }

        run_test(vec![
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "Hello, 世界!"),
                )),
                Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 10)),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                1,
                            )),
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                2,
                            )),
                        ],
                    ),
                )),
                Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 2)),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Map(MapScalarExpression::new(
                    QueryLocation::new_fake(),
                    HashMap::from([
                        (
                            "key1".into(),
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                1,
                            )),
                        ),
                        (
                            "key2".into(),
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                QueryLocation::new_fake(),
                                2,
                            )),
                        ),
                    ]),
                ))),
                Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 2)),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                Value::Null,
            ),
        ]);
    }

    #[test]
    fn test_execute_replace_string_scalar_expression() {
        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            expected: Value,
        ) {
            let e = ScalarExpression::ReplaceString(ReplaceStringScalarExpression::new(
                QueryLocation::new_fake(),
                haystack,
                needle,
                replacement,
                false, // case_insensitive
            ));

            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Basic string replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "A magic trick can turn a cat into a dog",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "A magic trick can turn a hamster into a dog",
            )),
        );

        // Multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi world hi",
            )),
        );

        // No matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "no matches here",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "xyz",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "abc",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "no matches here",
            )),
        );

        // Invalid input type
        run_test(
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "search",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "replace",
            ))),
            Value::Null,
        );
    }

    #[test]
    fn test_execute_replace_string_scalar_expression_with_regex() {
        use regex::Regex;

        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            expected: Value,
        ) {
            let e = ScalarExpression::ReplaceString(ReplaceStringScalarExpression::new(
                QueryLocation::new_fake(),
                haystack,
                needle,
                replacement,
                false, // case_insensitive
            ));

            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Simple regex replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world 123",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"\d+").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "XXX",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world XXX",
            )),
        );

        // Regex with capture groups
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "2023-12-25",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "$3/$2/$1",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "25/12/2023",
            )),
        );

        // Multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat cat dog cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"cat").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hamster hamster dog hamster",
            )),
        );

        // Regex with no matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(r"\d+").unwrap(),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "XXX",
            ))),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            )),
        );
    }

    #[test]
    fn test_execute_replace_string_scalar_expression_case_insensitive() {
        fn run_test(
            haystack: ScalarExpression,
            needle: ScalarExpression,
            replacement: ScalarExpression,
            case_insensitive: bool,
            expected: Value,
        ) {
            let e = ScalarExpression::ReplaceString(ReplaceStringScalarExpression::new(
                QueryLocation::new_fake(),
                haystack,
                needle,
                replacement,
                case_insensitive,
            ));

            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let actual = execute_scalar_expression(&execution_context, &e).unwrap();
            assert_eq!(expected, actual.to_value());
        }

        // Case-sensitive replacement (default)
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            false,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World", // No replacement because "hello" != "Hello"
            )),
        );

        // Case-insensitive replacement
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Hello World",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi",
            ))),
            true,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hi World", // "Hello" replaced with "hi"
            )),
        );

        // Case-insensitive with multiple matches
        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "CAT cat Cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "cat",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "dog",
            ))),
            true,
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "dog dog dog", // All variants of "cat" replaced
            )),
        );
    }

    #[test]
    fn test_execute_case_scalar_expression() {
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

        // Test simple case: case(true, "success", "failure") -> "success"
        run_test(
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![(
                    LogicalExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            true,
                        )),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "success"),
                    )),
                )],
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "failure"),
                )),
            )),
            Value::String(&StringValueStorage::new("success".into())),
        );

        // Test fallback to else: case(false, "success", "failure") -> "failure"
        run_test(
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![(
                    LogicalExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            false,
                        )),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "success"),
                    )),
                )],
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "failure"),
                )),
            )),
            Value::String(&StringValueStorage::new("failure".into())),
        );

        // Test multiple conditions: case(false, "first", true, "second", "else") -> "second"
        run_test(
            ScalarExpression::Case(CaseScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    (
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "first"),
                        )),
                    ),
                    (
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                true,
                            )),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "second"),
                        )),
                    ),
                ],
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "else"),
                )),
            )),
            Value::String(&StringValueStorage::new("second".into())),
        );
    }

    #[test]
    fn test_execute_slice_scalar_expression() {
        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap_err();

            match expected {
                ExpressionError::TypeMismatch(_, msg) => {
                    if let ExpressionError::TypeMismatch(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
                ExpressionError::ValidationFailure(_, msg) => {
                    if let ExpressionError::ValidationFailure(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
            }
        }

        let string_source = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(QueryLocation::new_fake(), "Hello world!"),
        ));

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                ))),
                None,
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "Range start for a slice expression cannot be a negative value".into(),
            ),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                ))),
                None,
            ),
            ExpressionError::TypeMismatch(
                QueryLocation::new_fake(),
                "Range start for a slice expression should be an integer type".into(),
            ),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
                ))),
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "Range end for a slice expression cannot be a negative value".into(),
            ),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                ))),
            ),
            ExpressionError::TypeMismatch(
                QueryLocation::new_fake(),
                "Range end for a slice expression should be an integer type".into(),
            ),
        );
    }

    #[test]
    fn test_execute_string_slice_scalar_expression() {
        fn run_test_success(input: SliceScalarExpression, expected: Value) {
            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(expected, actual.to_value());
        }

        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap_err();

            match expected {
                ExpressionError::TypeMismatch(_, msg) => {
                    if let ExpressionError::TypeMismatch(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
                ExpressionError::ValidationFailure(_, msg) => {
                    if let ExpressionError::ValidationFailure(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
            }
        }

        let string_source = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(QueryLocation::new_fake(), "こんにちは"),
        ));

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                None,
                None,
            ),
            Value::String(&StringValueStorage::new("こんにちは".into())),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                None,
            ),
            Value::String(&StringValueStorage::new("んにちは".into())),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            Value::String(&StringValueStorage::new("".into())),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Value::String(&StringValueStorage::new("ん".into())),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Value::String(&StringValueStorage::new("こん".into())),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                ))),
            ),
            Value::String(&StringValueStorage::new("こんにちは".into())),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "String slice index starts at '2' but ends at '1'".into(),
            ),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                ))),
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "String slice index ends at '6' which is beyond the length of '5'".into(),
            ),
        );
    }

    #[test]
    fn test_execute_array_slice_scalar_expression() {
        fn run_test_success(input: SliceScalarExpression, expected: Value) {
            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(expected, actual.to_value());
        }

        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let pipeline = Default::default();

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap_err();

            match expected {
                ExpressionError::TypeMismatch(_, msg) => {
                    if let ExpressionError::TypeMismatch(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
                ExpressionError::ValidationFailure(_, msg) => {
                    if let ExpressionError::ValidationFailure(_, actual_msg) = actual {
                        assert_eq!(msg, actual_msg)
                    } else {
                        panic!("Unexpected ExpressionError")
                    }
                }
            }
        }

        let array_values = vec![
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                0,
            )),
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                1,
            )),
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                2,
            )),
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                3,
            )),
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                4,
            )),
        ];

        let array_source = ScalarExpression::Static(StaticScalarExpression::Array(
            ArrayScalarExpression::new(QueryLocation::new_fake(), array_values.clone()),
        ));

        run_test_success(
            SliceScalarExpression::new(QueryLocation::new_fake(), array_source.clone(), None, None),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone(),
            )),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                None,
            ),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone().drain(1..).collect(),
            )),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone().drain(1..1).collect(),
            )),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone().drain(1..2).collect(),
            )),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone().drain(..2).collect(),
            )),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                ))),
            ),
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                array_values.clone().drain(..5).collect(),
            )),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "Array slice index starts at '2' but ends at '1'".into(),
            ),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                array_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                ))),
            ),
            ExpressionError::ValidationFailure(
                QueryLocation::new_fake(),
                "Array slice index ends at '6' which is beyond the length of '5'".into(),
            ),
        );
    }

    #[test]
    pub fn test_execute_parse_json_scalar_expression() {
        fn run_test_success(input: &str, expected_value: Value) {
            let pipeline = Default::default();

            let expression = ScalarExpression::ParseJson(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            ));

            let record = TestRecord::new();

            let execution_context = ExecutionContext::new(
                RecordSetEngineDiagnosticLevel::Verbose,
                &pipeline,
                None,
                record,
            );

            let actual_value = execute_scalar_expression(&execution_context, &expression).unwrap();
            assert_eq!(expected_value, actual_value.to_value());
        }

        run_test_success(
            "18",
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
        );
        run_test_success("hello world", Value::Null);
    }
}
