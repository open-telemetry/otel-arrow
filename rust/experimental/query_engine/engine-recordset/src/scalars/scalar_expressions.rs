// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{cell::Ref, slice::Iter, sync::LazyLock};

use data_engine_expressions::*;

use crate::{
    execution_context::*,
    logical_expressions::execute_logical_expression,
    scalars::{
        execute_collection_scalar_expression, execute_convert_scalar_expression,
        execute_math_scalar_expression, execute_parse_scalar_expression,
        execute_temporal_scalar_expression, execute_text_scalar_expression,
    },
    transform::transform_expressions::execute_transform_expression,
    *,
};

static VALUE_TYPE_NAMES: LazyLock<Vec<StringValueStorage>> = LazyLock::new(|| {
    let mut items = Vec::new();
    for value_type in ValueType::get_value_types() {
        let name: &str = value_type.into();
        items.push(StringValueStorage::new(name.into()));
    }
    items
});

pub fn execute_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    scalar_expression: &'a ScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = match scalar_expression {
        ScalarExpression::Source(s) => {
            execute_source_scalar_expression(execution_context, scalar_expression, s)?
        }
        ScalarExpression::Attached(a) => {
            let name = a.get_name().get_value();

            if let Some(Some(record)) = execution_context
                .get_attached_records()
                .map(|v| v.get_attached_record(name))
            {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    scalar_expression,
                    || format!("Resolved attached data with name '{name}'"),
                );

                let mut selectors = a.get_value_accessor().get_selectors().iter();

                select_from_value(
                    execution_context,
                    Value::Map(record),
                    scalar_expression,
                    &mut selectors,
                )?
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    scalar_expression,
                    || format!("Attached record matching name '{name}' could not be found"),
                );
                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
        ScalarExpression::Variable(v) => {
            execute_variable_scalar_expression(execution_context, scalar_expression, v)?
        }
        ScalarExpression::Static(s) => ResolvedValue::Value(s.to_value()),
        ScalarExpression::Constant(c) => {
            let constant_id = c.get_constant_id();

            let constant = execution_context
                .get_pipeline()
                .get_constant(constant_id)
                .unwrap_or_else(|| {
                    panic!("Constant for id '{constant_id}' was not found on pipeline")
                });

            if execution_context
                .is_diagnostic_level_enabled(RecordSetEngineDiagnosticLevel::Verbose)
            {
                let (line, column) = constant.get_query_location().get_line_and_column_numbers();
                execution_context.add_diagnostic(RecordSetEngineDiagnostic::new(
                    RecordSetEngineDiagnosticLevel::Verbose,
                    scalar_expression,
                    format!("Resolved '{}' constant with id '{constant_id}' defined on line {line} at column {column}", constant.get_value_type()),
                ));
            }

            let value_accessor = c.get_value_accessor();

            match value_accessor.has_selectors() {
                true => {
                    let mut selectors = value_accessor.get_selectors().iter();

                    select_from_value(
                        execution_context,
                        constant.to_value(),
                        scalar_expression,
                        &mut selectors,
                    )?
                }
                false => ResolvedValue::Value(constant.to_value()),
            }
        }
        ScalarExpression::Collection(c) => {
            return execute_collection_scalar_expression(execution_context, c);
        }
        ScalarExpression::Logical(l) => {
            let value = execute_logical_expression(execution_context, l)?;

            // Note: Return here skips logging because execute_logical_expression does that
            return Ok(ResolvedValue::Computed(OwnedValue::Boolean(
                BooleanValueStorage::new(value),
            )));
        }
        ScalarExpression::Coalesce(c) => {
            let mut value = ResolvedValue::Computed(OwnedValue::Null);

            for expression in c.get_expressions() {
                value = execute_scalar_expression(execution_context, expression)?;
                if value.get_value_type() != ValueType::Null {
                    break;
                }
            }

            value
        }
        ScalarExpression::Conditional(c) => {
            let inner_scalar =
                match execute_logical_expression(execution_context, c.get_condition())? {
                    true => c.get_true_expression(),
                    false => c.get_false_expression(),
                };

            execute_scalar_expression(execution_context, inner_scalar)?
        }
        ScalarExpression::Case(c) => {
            let expressions_with_conditions = c.get_expressions_with_conditions();

            let mut result = None;

            // Evaluate conditions in order and return first matching result
            for (condition, expression) in expressions_with_conditions {
                if execute_logical_expression(execution_context, condition)? {
                    result = Some(execute_scalar_expression(execution_context, expression)?);
                    break;
                }
            }

            match result {
                Some(v) => v,
                None => {
                    // No condition matched, return else expression
                    execute_scalar_expression(execution_context, c.get_else_expression())?
                }
            }
        }
        ScalarExpression::Convert(c) => {
            return execute_convert_scalar_expression(execution_context, c);
        }
        ScalarExpression::Length(l) => {
            let inner_value =
                execute_scalar_expression(execution_context, l.get_inner_expression())?;

            match inner_value.to_value() {
                Value::String(s) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(s.get_value().chars().count() as i64),
                )),
                Value::Array(a) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(a.len() as i64),
                )),
                Value::Map(m) => ResolvedValue::Computed(OwnedValue::Integer(
                    IntegerValueStorage::new(m.len() as i64),
                )),
                value => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        l,
                        || {
                            format!(
                                "Cannot calculate the length of '{:?}' input",
                                value.get_value_type()
                            )
                        },
                    );
                    ResolvedValue::Computed(OwnedValue::Null)
                }
            }
        }
        ScalarExpression::Slice(s) => {
            let inner_value = execute_scalar_expression(execution_context, s.get_source())?;

            let range_start_inclusive = match s.get_range_start_inclusive() {
                Some(start) => SliceScalarExpression::validate_resolved_range_value(
                    start.get_query_location(),
                    "start",
                    execute_scalar_expression(execution_context, start)?.to_value(),
                )?,
                None => 0,
            };
            let range_end_exclusive = match s.get_range_end_exclusive() {
                Some(end) => Some(SliceScalarExpression::validate_resolved_range_value(
                    end.get_query_location(),
                    "end",
                    execute_scalar_expression(execution_context, end)?.to_value(),
                )?),
                None => None,
            };

            match inner_value.try_resolve_string() {
                Ok(string_value) => {
                    let range_end_exclusive = SliceScalarExpression::validate_slice_range(
                        s.get_query_location(),
                        "String",
                        string_value.get_value().chars().count(),
                        range_start_inclusive,
                        range_end_exclusive,
                    )?;

                    ResolvedValue::Slice(Slice::String(StringSlice::from_char_range(
                        string_value,
                        range_start_inclusive,
                        range_end_exclusive,
                    )))
                }
                Err(v) => match v.try_resolve_array() {
                    Ok(array_value) => {
                        let range_end_exclusive = SliceScalarExpression::validate_slice_range(
                            s.get_query_location(),
                            "Array",
                            array_value.len(),
                            range_start_inclusive,
                            range_end_exclusive,
                        )?;

                        ResolvedValue::Slice(Slice::Array(ArraySlice::new(
                            array_value,
                            range_start_inclusive,
                            range_end_exclusive,
                        )))
                    }
                    Err(e) => {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            s,
                            || format!("Cannot take a slice of '{:?}' input", e.get_value_type()),
                        );
                        ResolvedValue::Computed(OwnedValue::Null)
                    }
                },
            }
        }
        ScalarExpression::Parse(p) => {
            return execute_parse_scalar_expression(execution_context, p);
        }
        ScalarExpression::Temporal(t) => {
            return execute_temporal_scalar_expression(execution_context, t);
        }
        ScalarExpression::Text(t) => {
            return execute_text_scalar_expression(execution_context, t);
        }
        ScalarExpression::Math(m) => {
            return execute_math_scalar_expression(execution_context, m);
        }
        ScalarExpression::GetType(g) => {
            let value_type =
                execute_scalar_expression(execution_context, g.get_value())?.get_value_type();

            let value_type_name = &VALUE_TYPE_NAMES[value_type as usize];

            ResolvedValue::Value(Value::String(value_type_name))
        }
        ScalarExpression::Select(s) => {
            let selectors_expression = s.get_selectors();

            match execute_scalar_expression(execution_context, selectors_expression)?.to_value() {
                Value::Array(selectors) => {
                    let mut value = execute_scalar_expression(execution_context, s.get_value())?;

                    if !selectors.is_empty() {
                        for i in 0..selectors.len() {
                            let selector = selectors
                                .get(i)
                                .expect("Selector could not be found")
                                .to_value();

                            match value.select(execution_context, selectors_expression, selector)? {
                                Some(v) => value = v,
                                None => {
                                    value = ResolvedValue::Computed(OwnedValue::Null);
                                    break;
                                }
                            }
                        }
                    }

                    value
                }
                v => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        s.get_selectors(),
                        || {
                            format!(
                                "Value of '{}' type returned by scalar expression was not an array",
                                v.get_value_type()
                            )
                        },
                    );
                    return Ok(ResolvedValue::Computed(OwnedValue::Null));
                }
            }
        }
        ScalarExpression::Argument(a) => execute_argument_scalar_expression(execution_context, a)?,
        ScalarExpression::InvokeFunction(i) => {
            let function_id = i.get_function_id();

            let func = execution_context
                .get_pipeline()
                .get_function(function_id)
                .unwrap_or_else(|| {
                    panic!("Function for id '{function_id}' was not found on pipeline")
                });

            let args = i.get_arguments();

            let parameters = func.get_parameters();

            if args.len() != parameters.len() {
                return Err(ExpressionError::ValidationFailure(
                    i.get_query_location().clone(),
                    "Invalid number of arguments specified for function invocation".into(),
                ));
            }

            let arguments = ExecutionContextArgumentContainer {
                parent_execution_context: execution_context,
                arguments: args,
            };

            let func_execution_context = execution_context.create_scope(Some(&arguments));

            let return_value = match func.get_implementation() {
                PipelineFunctionImplementation::Expressions(expressions) => {
                    let mut return_value = None;

                    for e in expressions {
                        match e {
                            PipelineFunctionExpression::Transform(t) => {
                                execute_transform_expression(&func_execution_context, t)?
                            }
                            PipelineFunctionExpression::Return(s) => {
                                return_value =
                                    Some(execute_scalar_expression(&func_execution_context, s)?);
                                break;
                            }
                        }
                    }

                    match return_value {
                        Some(v) => ResolvedValue::Computed(v.into()),
                        None => ResolvedValue::Computed(OwnedValue::Null),
                    }
                }
                PipelineFunctionImplementation::External(name) => {
                    let func_implementation =
                        execution_context.get_external_function_implementation(name);

                    ResolvedValue::Computed(
                        func_implementation
                            .invoke(i, &func_execution_context)?
                            .into(),
                    )
                }
            };

            if execution_context
                .is_diagnostic_level_enabled(RecordSetEngineDiagnosticLevel::Verbose)
            {
                execution_context.add_diagnostic(
                    RecordSetEngineDiagnostic::new(
                        RecordSetEngineDiagnosticLevel::Verbose,
                        scalar_expression,
                        format!("Executed function '{function_id}'"),
                    )
                    .with_nested_diagnostics(func_execution_context.take_diagnostics()),
                );
            }

            // Note: Exit without normal logging because the above add_diagnostic call handles that
            return Ok(return_value);
        }
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        scalar_expression,
        || format!("Evaluated as: {value}"),
    );

    Ok(value)
}

pub(crate) fn execute_source_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    source_scalar_expression: &'a SourceScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    Ok(if let Some(record) = execution_context.get_record() {
        let mut selectors = source_scalar_expression
            .get_value_accessor()
            .get_selectors()
            .iter();

        select_from_borrowed_value(
            execution_context,
            BorrowSource::Source,
            record.borrow(),
            expression,
            &mut selectors,
        )?
    } else {
        execution_context.add_diagnostic_if_enabled(
            RecordSetEngineDiagnosticLevel::Warn,
            expression,
            || "Source could not be found".into(),
        );
        ResolvedValue::Computed(OwnedValue::Null)
    })
}

pub(crate) fn execute_variable_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    expression: &'a dyn Expression,
    variable_scalar_expression: &'a VariableScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let variable_name = variable_scalar_expression.get_name().get_value();

    Ok(
        if let Some(variable) = execution_context
            .get_variables()
            .get_global_or_local_variable(variable_name)
        {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Verbose,
                expression,
                || {
                    format!(
                        "Resolved '{}' variable with name '{variable_name}'",
                        variable.get_value_type()
                    )
                },
            );

            let mut selectors = variable_scalar_expression
                .get_value_accessor()
                .get_selectors()
                .iter();

            select_from_borrowed_value(
                execution_context,
                BorrowSource::Variable,
                variable,
                expression,
                &mut selectors,
            )?
        } else {
            execution_context.add_diagnostic_if_enabled(
                RecordSetEngineDiagnosticLevel::Warn,
                expression,
                || format!("Variable matching name '{variable_name}' could not be found"),
            );
            ResolvedValue::Computed(OwnedValue::Null)
        },
    )
}

pub(crate) fn execute_argument_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    argument_scalar_expression: &'a ArgumentScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let mut value = execution_context
        .get_arguments()
        .expect("Arguments were not found")
        .get_argument(argument_scalar_expression.get_argument_id())?;

    let selectors = argument_scalar_expression
        .get_value_accessor()
        .get_selectors();
    if !selectors.is_empty() {
        for selector in selectors {
            match value.select(
                execution_context,
                selector,
                execute_scalar_expression(execution_context, selector)?.to_value(),
            )? {
                Some(v) => value = v,
                None => {
                    value = ResolvedValue::Computed(OwnedValue::Null);
                    break;
                }
            }
        }
    }

    Ok(value)
}

fn select_from_borrowed_value<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    borrow_source: BorrowSource,
    borrow: Ref<'b, dyn AsStaticValue + 'static>,
    expression: &'a dyn Expression,
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
                        match m.get_static(map_key.get_value()) {
                            Ok(Some(v)) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Verbose,
                                    expression,
                                    || format!("Resolved {} value for key '{}' specified in accessor expression", ResolvedValue::Value(v.to_value()), map_key.get_value()),
                                );
                                Some(v)
                            }
                            Ok(None) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    expression,
                                    || format!("Could not find map key '{}' specified in accessor expression", map_key.get_value()),
                                );
                                None
                            }
                            Err(e) => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Error,
                                    s,
                                    || format!("Interior mutability is not supported by the target map: {e}"),
                                );
                                None
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
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
                                expression,
                                || format!("Array index '{index}' specified in accessor expression is invalid"),
                            );
                            None
                        } else {
                            match a.get_static(index as usize) {
                                Ok(Some(v)) => {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        expression,
                                        || format!("Resolved {} value for index '{index}' specified in accessor expression", ResolvedValue::Value(v.to_value())),
                                    );
                                    Some(v)
                                }
                                Ok(None) => {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Warn,
                                        expression,
                                        || format!("Could not find array index '{index}' specified in accessor expression"),
                                    );
                                    None
                                }
                                Err(e) => {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Error,
                                        expression,
                                        || format!("Interior mutability is not supported by the target array: {e}"),
                                    );
                                    None
                                }
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
                            || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{:?}' value", array_index.get_value(), v.get_value_type()),
                        );
                        None
                    }
                }),
                v => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Unexpected scalar expression with '{}' value type encountered in accessor expression", v.get_value_type()),
                    );
                    return Ok(ResolvedValue::Computed(OwnedValue::Null));
                }
            };

            if let Ok(v) = next {
                select_from_borrowed_value(
                    execution_context,
                    borrow_source,
                    v,
                    expression,
                    selectors,
                )
            } else {
                Ok(ResolvedValue::Computed(OwnedValue::Null))
            }
        }
        None => Ok(ResolvedValue::Borrowed(borrow_source, borrow)),
    }
}

fn select_from_value<'a, 'b, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, TRecord>,
    root: Value<'b>,
    expression: &'a ScalarExpression,
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
                                    expression,
                                    || format!("Resolved {} value for key '{}' specified in accessor expression", ResolvedValue::Value(v.to_value()), map_key.get_value()),
                                );
                                Some(v.to_value())
                            }
                            None => {
                                execution_context.add_diagnostic_if_enabled(
                                    RecordSetEngineDiagnosticLevel::Warn,
                                    expression,
                                    || format!("Could not find map key '{}' specified in accessor expression", map_key.get_value()),
                                );
                                None
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
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
                                expression,
                                || format!("Array index '{index}' specified in accessor expression is invalid"),
                            );
                            None
                        } else {
                            match a.get(index as usize) {
                                Some(v) => {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Verbose,
                                        expression,
                                        || format!("Resolved {} value for index '{index}' specified in accessor expression", ResolvedValue::Value(v.to_value())),
                                    );
                                    Some(v.to_value())
                                }
                                None => {
                                    execution_context.add_diagnostic_if_enabled(
                                        RecordSetEngineDiagnosticLevel::Warn,
                                        expression,
                                        || format!("Could not find array index '{index}' specified in accessor expression"),
                                    );
                                    None
                                }
                            }
                        }
                    } else {
                        execution_context.add_diagnostic_if_enabled(
                            RecordSetEngineDiagnosticLevel::Warn,
                            expression,
                            || format!("Could not search for array index '{}' specified in accessor expression because current node is a '{:?}' value", array_index.get_value(), root.get_value_type()),
                        );
                        None
                    }
                }
                v => {
                    execution_context.add_diagnostic_if_enabled(
                        RecordSetEngineDiagnosticLevel::Warn,
                        expression,
                        || format!("Unexpected scalar expression with '{}' value type encountered in accessor expression", v.get_value_type()),
                    );
                    None
                }
            };

            if let Some(v) = next {
                select_from_value(execution_context, v, expression, selectors)
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

    use chrono::{TimeDelta, Utc};
    use regex::Regex;

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
            let mut test = TestExecutionContext::new().with_record(record.clone());

            let execution_context = test.create_execution_context();

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

        // Test invalid access (using a bool value)
        run_test(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )]),
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
            let mut test = TestExecutionContext::new()
                .with_attached_records(attached_records.clone())
                .with_record(record.clone());

            let execution_context = test.create_execution_context();

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

        // Test invalid access (using a bool value)
        run_test(
            ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )]),
            )),
            Value::Null,
        );
    }

    #[test]
    fn test_execute_variable_scalar_expression() {
        let run_test = |scalar_expression, expected_value: Value| {
            let mut test = TestExecutionContext::new();

            test.set_global_variable(
                "gvar1",
                ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(18))),
            );

            let execution_context = test.create_execution_context();

            {
                let mut variables = execution_context.get_variables().get_local_variables_mut();

                variables.set(
                    "var1",
                    ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new(
                        "hello world".into(),
                    ))),
                );
                variables.set(
                    "var2",
                    ResolvedValue::Computed(OwnedValue::Map(MapValueStorage::new(HashMap::from(
                        [(
                            "key1".into(),
                            OwnedValue::String(StringValueStorage::new("hello world".into())),
                        )],
                    )))),
                );
            }

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

        // Test global variable resolution
        run_test(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "gvar1"),
                ValueAccessor::new(),
            )),
            Value::Integer(&IntegerValueStorage::new(18)),
        );
    }

    #[test]
    fn test_execute_constant_scalar_expression() {
        let run_test = |scalar_expression, expected_value: Value| {
            let pipeline = PipelineExpressionBuilder::new("")
                .with_constants(vec![StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )])
                .with_constants(vec![StaticScalarExpression::Map(MapScalarExpression::new(
                    QueryLocation::new_fake(),
                    HashMap::from([(
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "value1",
                        )),
                    )]),
                ))])
                .build()
                .unwrap();

            let mut test = TestExecutionContext::new().with_pipeline(pipeline);

            let execution_context = test.create_execution_context();

            let value = execute_scalar_expression(&execution_context, &scalar_expression).unwrap();

            assert_eq!(expected_value, value.to_value());
        };

        run_test(
            ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                QueryLocation::new_fake(),
                ValueType::Integer,
                0,
                ValueAccessor::new(),
            )),
            Value::Integer(&IntegerValueStorage::new(18)),
        );

        run_test(
            ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                QueryLocation::new_fake(),
                ValueType::String,
                1,
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            Value::String(&StringValueStorage::new("value1".into())),
        );
    }

    #[test]
    fn test_execute_logical_scalar_expression() {
        let run_test = |scalar_expression, expected_value: Value| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
        let run_test = |scalar_expression, expected_value: Value| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
        let run_test = |scalar_expression, expected_value: Value| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
    fn test_execute_length_scalar_expression() {
        fn run_test(input: Vec<(ScalarExpression, Value)>) {
            for (inner, expected) in input {
                let e = ScalarExpression::Length(LengthScalarExpression::new(
                    QueryLocation::new_fake(),
                    inner,
                ));

                let mut test = TestExecutionContext::new();

                let execution_context = test.create_execution_context();

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
    fn test_execute_case_scalar_expression() {
        let run_test = |scalar_expression, expected_value: Value| {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
                _ => {
                    panic!("Unexpected ExpressionError")
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
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(expected.to_string(), actual.to_value().to_string());
        }

        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
                _ => {
                    panic!("Unexpected ExpressionError")
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
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::Slice(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(expected, actual.to_value());
        }

        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

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
                _ => {
                    panic!("Unexpected ExpressionError")
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
    fn test_execute_get_type_scalar_expression() {
        fn run_test_success(input: ScalarExpression, expected: &str) {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::GetType(GetTypeScalarExpression::new(
                QueryLocation::new_fake(),
                input,
            ));

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(
                OwnedValue::String(StringValueStorage::new(expected.into())).to_value(),
                actual.to_value()
            );
        }

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            ))),
            "Array",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
            "Boolean",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::DateTime(
                DateTimeScalarExpression::new(QueryLocation::new_fake(), Utc::now().into()),
            )),
            "DateTime",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                0.0,
            ))),
            "Double",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
            )),
            "Integer",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Map(MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::new(),
            ))),
            "Map",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
            "Null",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Regex(RegexScalarExpression::new(
                QueryLocation::new_fake(),
                Regex::new(".*").unwrap(),
            ))),
            "Regex",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "",
            ))),
            "String",
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                TimeSpanScalarExpression::new(QueryLocation::new_fake(), TimeDelta::minutes(1)),
            )),
            "TimeSpan",
        );
    }

    #[test]
    fn test_execute_select_scalar_expression() {
        fn run_test_success(input: SelectScalarExpression, expected: &str) {
            let mut test =
                TestExecutionContext::new().with_record(TestRecord::new().with_key_value(
                    "Attributes".into(),
                    OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                        "key1".into(),
                        OwnedValue::String(StringValueStorage::new("value1".into())),
                    )]))),
                ));

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::Select(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(
                OwnedValue::String(StringValueStorage::new(expected.into())).to_value(),
                actual.to_value()
            );
        }

        fn run_test_failure(input: SelectScalarExpression) {
            let mut test = TestExecutionContext::new();

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::Select(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(OwnedValue::Null.to_value(), actual.to_value());
        }

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "value1"),
                )),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(QueryLocation::new_fake(), vec![]),
                )),
            ),
            "value1",
        );

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Map(MapScalarExpression::new(
                    QueryLocation::new_fake(),
                    HashMap::from([(
                        "key1".into(),
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "value1",
                        )),
                    )]),
                ))),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        ))],
                    ),
                )),
            ),
            "value1",
        );

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "value1",
                        ))],
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::Array(
                    ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )],
                    ),
                )),
            ),
            "value1",
        );

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Parse(ParseScalarExpression::Json(
                    ParseJsonScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                r#"{"key1":[{"key2":"value1"}]}"#,
                            ),
                        )),
                    ),
                )),
                ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                    ParseJsonPathScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "$.key1[0].key2",
                            ),
                        )),
                    ),
                )),
            ),
            "value1",
        );

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Parse(ParseScalarExpression::Json(
                    ParseJsonScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                r#"{"key1":[null,{"key2":"value1"}]}"#,
                            ),
                        )),
                    ),
                )),
                ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                    ParseJsonPathScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "$.key1[-1].key2",
                            ),
                        )),
                    ),
                )),
            ),
            "value1",
        );

        run_test_success(
            SelectScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                    ParseJsonPathScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "$.Attributes.key1",
                            ),
                        )),
                    ),
                )),
            ),
            "value1",
        );

        // Test invalid selectors (not an array)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"[18]"#),
                )),
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Selectors should be an array",
            ))),
        ));

        // Test invalid index access (positive)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"[18]"#),
                )),
            ))),
            ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                ParseJsonPathScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "$[1]"),
                    )),
                ),
            )),
        ));

        // Test invalid index access (negative)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"[18]"#),
                )),
            ))),
            ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                ParseJsonPathScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "$[-2]"),
                    )),
                ),
            )),
        ));

        // Test invalid key access
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"{"key1":"value1"}"#),
                )),
            ))),
            ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                ParseJsonPathScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "$.unknown_key"),
                    )),
                ),
            )),
        ));

        // Test invalid accessor (map accessed by index)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"{"key1":"value1"}"#),
                )),
            ))),
            ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                ParseJsonPathScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "$[0]"),
                    )),
                ),
            )),
        ));

        // Test invalid accessor (array accessed by key)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Parse(ParseScalarExpression::Json(ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), r#"[0]"#),
                )),
            ))),
            ScalarExpression::Parse(ParseScalarExpression::JsonPath(
                ParseJsonPathScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "$.key1"),
                    )),
                ),
            )),
        ));

        // Test invalid accessor (bool value)
        run_test_failure(SelectScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::Map(MapScalarExpression::new(
                QueryLocation::new_fake(),
                HashMap::from([(
                    "key1".into(),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "value1",
                    )),
                )]),
            ))),
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )],
            ))),
        ));
    }

    #[test]
    fn test_execute_argument_scalar_expression() {
        fn run_test_success(input: ArgumentScalarExpression, expected: &str) {
            let mut test =
                TestExecutionContext::new().with_record(TestRecord::new().with_key_value(
                    "Attributes".into(),
                    OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                        "key1".into(),
                        OwnedValue::String(StringValueStorage::new("value1".into())),
                    )]))),
                ));

            let execution_context = test.create_execution_context();

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
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                        ]),
                    ),
                )),
            ];

            let execution_context_arguments = ExecutionContextArgumentContainer {
                parent_execution_context: &execution_context,
                arguments: &arguments,
            };

            let scope = execution_context.create_scope(Some(&execution_context_arguments));

            let expression = ScalarExpression::Argument(input);

            let actual = execute_scalar_expression(&scope, &expression).unwrap();

            assert_eq!(
                OwnedValue::String(StringValueStorage::new(expected.into())).to_value(),
                actual.to_value()
            );
        }

        run_test_success(
            ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::String),
                0,
                ValueAccessor::new(),
            ),
            "hello world",
        );

        run_test_success(
            ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::String),
                1,
                ValueAccessor::new(),
            ),
            "value1",
        );
    }

    #[test]
    fn test_execute_invoke_function_scalar_expression() {
        fn run_test_success(input: InvokeFunctionScalarExpression, expected: &str) {
            let mut test = TestExecutionContext::new()
                .with_record(TestRecord::new().with_key_value(
                    "Attributes".into(),
                    OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                        "key1".into(),
                        OwnedValue::String(StringValueStorage::new("value1".into())),
                    )]))),
                ))
                .with_pipeline(
                    PipelineExpressionBuilder::new("")
                        .with_functions(vec![
                            PipelineFunction::new_with_expressions(
                                QueryLocation::new_fake(),
                                vec![PipelineFunctionParameter::new(
                                    QueryLocation::new_fake(),
                                    PipelineFunctionParameterType::Scalar(Some(ValueType::String)),
                                )],
                                Some(ValueType::String),
                                vec![PipelineFunctionExpression::Return(
                                    ScalarExpression::Argument(ArgumentScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        Some(ValueType::String),
                                        0,
                                        ValueAccessor::new(),
                                    )),
                                )],
                            ),
                            PipelineFunction::new_with_expressions(
                                QueryLocation::new_fake(),
                                vec![PipelineFunctionParameter::new(
                                    QueryLocation::new_fake(),
                                    PipelineFunctionParameterType::Scalar(Some(ValueType::String)),
                                )],
                                Some(ValueType::Integer),
                                vec![PipelineFunctionExpression::Return(
                                    ScalarExpression::Argument(ArgumentScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        Some(ValueType::String),
                                        0,
                                        ValueAccessor::new(),
                                    )),
                                )],
                            ),
                        ])
                        .build()
                        .unwrap(),
                );

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::InvokeFunction(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(
                OwnedValue::String(StringValueStorage::new(expected.into())).to_value(),
                actual.to_value()
            );
        }

        run_test_success(
            InvokeFunctionScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::Integer),
                0,
                vec![InvokeFunctionArgument::Scalar(ScalarExpression::Source(
                    SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "Attributes",
                                ),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                            )),
                        ]),
                    ),
                ))],
            ),
            "value1",
        );
    }

    #[test]
    fn test_execute_invoke_external_function_scalar_expression() {
        fn run_test_success(input: InvokeFunctionScalarExpression, expected: &str) {
            let mut test = TestExecutionContext::new()
                .with_record(TestRecord::new().with_key_value(
                    "Attributes".into(),
                    OwnedValue::Map(MapValueStorage::new(HashMap::from([
                        (
                            "key1".into(),
                            OwnedValue::String(StringValueStorage::new("value1".into())),
                        ),
                        (
                            "key2".into(),
                            OwnedValue::Map(MapValueStorage::new(HashMap::from([(
                                "subkey1".into(),
                                OwnedValue::String(StringValueStorage::new("subvalue1".into())),
                            )]))),
                        ),
                    ]))),
                ))
                .with_pipeline(
                    PipelineExpressionBuilder::new("")
                        .with_functions(vec![PipelineFunction::new_external(
                            "my_func",
                            vec![
                                PipelineFunctionParameter::new(
                                    QueryLocation::new_fake(),
                                    PipelineFunctionParameterType::Scalar(Some(ValueType::String)),
                                ),
                                PipelineFunctionParameter::new(
                                    QueryLocation::new_fake(),
                                    PipelineFunctionParameterType::MutableValue(Some(
                                        ValueType::Map,
                                    )),
                                ),
                            ],
                            Some(ValueType::String),
                        )])
                        .build()
                        .unwrap(),
                )
                .with_external_function_implementation(
                    "my_func",
                    RecordSetEngineFunctionClosureCallback::new(|_, ec| {
                        let arguments = ec.get_arguments().unwrap();

                        let mut value1 = arguments.get_argument(0)?;

                        arguments.copy_value_if_required_for_write(&mut value1, 1);

                        {
                            let mut mut_value2 = arguments.get_argument_mut(1)?;
                            match mut_value2.to_static_value_mut() {
                                Some(StaticValueMut::Map(m)) => {
                                    let old = m.set("subkey1", value1);
                                    match old {
                                        ValueMutWriteResult::Updated(old) => {
                                            assert_eq!("subvalue1", old.to_value().to_string())
                                        }
                                        _ => panic!("unexpected result"),
                                    }
                                }
                                _ => panic!("value2 wasn't a map"),
                            }
                        }

                        let value2 = arguments.get_argument(1)?;
                        match value2.to_value() {
                            Value::Map(m) => {
                                assert_eq!(1, m.len());
                                assert_eq!(
                                    Some("value1".into()),
                                    m.get("subkey1").map(|v| v.to_value().to_string())
                                );
                            }
                            _ => panic!("value2 wasn't a map"),
                        }

                        arguments.get_argument(0)
                    }),
                );

            let execution_context = test.create_execution_context();

            let expression = ScalarExpression::InvokeFunction(input);

            let actual = execute_scalar_expression(&execution_context, &expression).unwrap();

            assert_eq!(
                OwnedValue::String(StringValueStorage::new(expected.into())).to_value(),
                actual.to_value()
            );
        }

        run_test_success(
            InvokeFunctionScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::Integer),
                0,
                vec![
                    InvokeFunctionArgument::Scalar(ScalarExpression::Source(
                        SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "Attributes",
                                    ),
                                )),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                                )),
                            ]),
                        ),
                    )),
                    InvokeFunctionArgument::MutableValue(MutableValueExpression::Source(
                        SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new_with_selectors(vec![
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "Attributes",
                                    ),
                                )),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "key2"),
                                )),
                            ]),
                        ),
                    )),
                ],
            ),
            "value1",
        );
    }
}
