// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;

use crate::{execution_context::*, scalars::execute_scalar_expression, *};

pub fn execute_convert_scalar_expression<'a, 'b, 'c, TRecord: Record>(
    execution_context: &'b ExecutionContext<'a, '_, '_, TRecord>,
    convert_scalar_expression: &'a ConvertScalarExpression,
) -> Result<ResolvedValue<'c>, ExpressionError>
where
    'a: 'c,
    'b: 'c,
{
    let value = match convert_scalar_expression {
        ConvertScalarExpression::Boolean(c) => {
            let inner_value =
                execute_scalar_expression(execution_context, c.get_inner_expression())?;

            let value = inner_value.to_value();

            if let Some(b) = value.convert_to_bool() {
                ResolvedValue::Computed(OwnedValue::Boolean(BooleanValueStorage::new(b)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    convert_scalar_expression,
                    || {
                        format!(
                            "Input of '{:?}' type could not be converted into a bool",
                            value.get_value_type()
                        )
                    },
                );

                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
        ConvertScalarExpression::DateTime(c) => {
            let inner_value =
                execute_scalar_expression(execution_context, c.get_inner_expression())?;

            let value = inner_value.to_value();

            if let Some(d) = value.convert_to_datetime() {
                ResolvedValue::Computed(OwnedValue::DateTime(DateTimeValueStorage::new(d)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    convert_scalar_expression,
                    || {
                        format!(
                            "Input of '{:?}' type could not be converted into a DateTime",
                            value.get_value_type()
                        )
                    },
                );
                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
        ConvertScalarExpression::Double(c) => {
            let inner_value =
                execute_scalar_expression(execution_context, c.get_inner_expression())?;

            let value = inner_value.to_value();

            if let Some(d) = value.convert_to_double() {
                ResolvedValue::Computed(OwnedValue::Double(DoubleValueStorage::new(d)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    convert_scalar_expression,
                    || {
                        format!(
                            "Input of '{:?}' type could not be converted into a double",
                            value.get_value_type()
                        )
                    },
                );
                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
        ConvertScalarExpression::Integer(c) => {
            let inner_value =
                execute_scalar_expression(execution_context, c.get_inner_expression())?;

            let value = inner_value.to_value();

            if let Some(i) = value.convert_to_integer() {
                ResolvedValue::Computed(OwnedValue::Integer(IntegerValueStorage::new(i)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    convert_scalar_expression,
                    || {
                        format!(
                            "Input of '{:?}' type could not be converted into an integer",
                            value.get_value_type()
                        )
                    },
                );
                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
        ConvertScalarExpression::String(c) => {
            let v = execute_scalar_expression(execution_context, c.get_inner_expression())?;
            let value_type = v.get_value_type();
            if value_type == ValueType::String {
                v
            } else if value_type == ValueType::Null {
                ResolvedValue::Computed(OwnedValue::String(StringValueStorage::new("".into())))
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
        ConvertScalarExpression::TimeSpan(c) => {
            let inner_value =
                execute_scalar_expression(execution_context, c.get_inner_expression())?;

            let value = inner_value.to_value();

            if let Some(t) = value.convert_to_timespan() {
                ResolvedValue::Computed(OwnedValue::TimeSpan(TimeSpanValueStorage::new(t)))
            } else {
                execution_context.add_diagnostic_if_enabled(
                    RecordSetEngineDiagnosticLevel::Warn,
                    convert_scalar_expression,
                    || {
                        format!(
                            "Input of '{:?}' type could not be converted into a TimeSpan",
                            value.get_value_type()
                        )
                    },
                );
                ResolvedValue::Computed(OwnedValue::Null)
            }
        }
    };

    execution_context.add_diagnostic_if_enabled(
        RecordSetEngineDiagnosticLevel::Verbose,
        convert_scalar_expression,
        || format!("Evaluated as: '{value}'"),
    );

    Ok(value)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

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

                let mut test = TestExecutionContext::new();

                let execution_context = test.create_execution_context();

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
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            Utc.timestamp_nanos(1).into(),
                        ),
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
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            Utc.timestamp_nanos(1).into(),
                        ),
                    )),
                    Value::Double(&DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)),
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
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            Utc.timestamp_nanos(1).into(),
                        ),
                    )),
                    Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 1)),
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
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            Utc.timestamp_nanos(1).into(),
                        ),
                    )),
                    Value::String(&StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "1970-01-01T00:00:00.000000001Z",
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
}
