// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConvertScalarExpression {
    /// Converts the value returned by the inner scalar expression into a bool or returns null for invalid input.
    Boolean(ConversionScalarExpression),

    /// Converts the value returned by the inner scalar expression into a DateTime or returns null for invalid input.
    DateTime(ConversionScalarExpression),

    /// Converts the value returned by the inner scalar expression into a double or returns null for invalid input.
    Double(ConversionScalarExpression),

    /// Converts the value returned by the inner scalar expression into an integer or returns null for invalid input.
    Integer(ConversionScalarExpression),

    /// Converts the value returned by the inner scalar expression into a string or returns an empty string for invalid input.
    String(ConversionScalarExpression),
}

impl ConvertScalarExpression {
    pub(crate) fn is_always_convertable_to_numeric(value_type: &ValueType) -> bool {
        matches!(
            value_type,
            ValueType::DateTime | ValueType::Boolean | ValueType::Integer | ValueType::Double
        )
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            ConvertScalarExpression::Boolean(c) => {
                match c.get_inner_expression().try_resolve_value_type(pipeline)? {
                    Some(v) if Self::is_always_convertable_to_numeric(&v) => {
                        Ok(Some(ValueType::Boolean))
                    }
                    _ => Ok(None),
                }
            }
            ConvertScalarExpression::DateTime(c) => {
                match c.get_inner_expression().try_resolve_value_type(pipeline)? {
                    Some(v) if Self::is_always_convertable_to_numeric(&v) => {
                        Ok(Some(ValueType::DateTime))
                    }
                    _ => Ok(None),
                }
            }
            ConvertScalarExpression::Double(c) => {
                match c.get_inner_expression().try_resolve_value_type(pipeline)? {
                    Some(v) if Self::is_always_convertable_to_numeric(&v) => {
                        Ok(Some(ValueType::Double))
                    }
                    _ => Ok(None),
                }
            }
            ConvertScalarExpression::Integer(c) => {
                match c.get_inner_expression().try_resolve_value_type(pipeline)? {
                    Some(v) if Self::is_always_convertable_to_numeric(&v) => {
                        Ok(Some(ValueType::Integer))
                    }
                    _ => Ok(None),
                }
            }
            ConvertScalarExpression::String(_) => Ok(Some(ValueType::String)),
        }
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            ConvertScalarExpression::Boolean(c) => {
                if let Some(v) = c.get_inner_expression().try_resolve_static(pipeline)? {
                    if let Some(b) = v.to_value().convert_to_bool() {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                c.query_location.clone(),
                                b,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                c.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            ConvertScalarExpression::DateTime(c) => {
                if let Some(v) = c.get_inner_expression().try_resolve_static(pipeline)? {
                    if let Some(d) = v.to_value().convert_to_datetime() {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::DateTime(DateTimeScalarExpression::new(
                                c.query_location.clone(),
                                d,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                c.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            ConvertScalarExpression::Double(c) => {
                if let Some(v) = c.get_inner_expression().try_resolve_static(pipeline)? {
                    if let Some(d) = v.to_value().convert_to_double() {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Double(DoubleScalarExpression::new(
                                c.query_location.clone(),
                                d,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                c.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            ConvertScalarExpression::Integer(c) => {
                if let Some(v) = c.get_inner_expression().try_resolve_static(pipeline)? {
                    if let Some(i) = v.to_value().convert_to_integer() {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                c.query_location.clone(),
                                i,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                c.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            ConvertScalarExpression::String(c) => {
                if let Some(v) = c.get_inner_expression().try_resolve_static(pipeline)? {
                    let v = v.to_value();
                    if let Value::Null = v {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                c.query_location.clone(),
                                "",
                            )),
                        )))
                    } else {
                        let mut value = None;

                        v.convert_to_string(&mut |s| {
                            value = Some(StringScalarExpression::new(c.query_location.clone(), s));
                        });

                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::String(
                                value.expect("Inner value did not return a string"),
                            ),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Expression for ConvertScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ConvertScalarExpression::Boolean(c) => c.get_query_location(),
            ConvertScalarExpression::DateTime(d) => d.get_query_location(),
            ConvertScalarExpression::Double(c) => c.get_query_location(),
            ConvertScalarExpression::Integer(c) => c.get_query_location(),
            ConvertScalarExpression::String(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ConvertScalarExpression::Boolean(_) => "ConvertScalar(Boolean)",
            ConvertScalarExpression::DateTime(_) => "ConvertScalar(DateTime)",
            ConvertScalarExpression::Double(_) => "ConvertScalar(Double)",
            ConvertScalarExpression::Integer(_) => "ConvertScalar(Integer)",
            ConvertScalarExpression::String(_) => "ConvertScalar(String)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversionScalarExpression {
    query_location: QueryLocation,
    inner_expression: Box<ScalarExpression>,
}

impl ConversionScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: ScalarExpression,
    ) -> ConversionScalarExpression {
        Self {
            query_location,
            inner_expression: inner_expression.into(),
        }
    }

    pub fn get_inner_expression(&self) -> &ScalarExpression {
        &self.inner_expression
    }
}

impl Expression for ConversionScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ConversionScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_try_resolve() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, Option<ValueType>, Option<Value>)>)
        where
            F: Fn(ConversionScalarExpression) -> ConvertScalarExpression,
        {
            for (inner, expected_type, expected_value) in input {
                let e = build(ConversionScalarExpression::new(
                    QueryLocation::new_fake(),
                    inner,
                ));

                let pipeline = Default::default();

                let actual_type = e.try_resolve_value_type(&pipeline).unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e.try_resolve_static(&pipeline).unwrap();
                assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
            }
        }

        run_test(
            ConvertScalarExpression::Boolean,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Some(ValueType::Boolean),
                    Some(Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.0),
                    )),
                    Some(ValueType::Boolean),
                    Some(Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Boolean),
                    Some(Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "true"),
                    )),
                    None,
                    Some(Value::Boolean(&BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            ConvertScalarExpression::DateTime,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "8/8/2025"),
                    )),
                    None,
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        date_utils::create_utc(2025, 8, 8, 0, 0, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
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
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18.18"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        18.18,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
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
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 18.18),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        18,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        18,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "18"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        18,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );
    }

    #[test]
    pub fn test_string_try_resolve_value_type() {
        // Test string conversion always returns string value type
        let expression = ConvertScalarExpression::String(ConversionScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
        ));

        assert_eq!(
            Some(ValueType::String),
            expression
                .try_resolve_value_type(&Default::default())
                .unwrap()
        );
    }

    #[test]
    pub fn test_string_try_resolve_static() {
        let run_test = |input: StaticScalarExpression, expected: Value| {
            let expression = ConvertScalarExpression::String(ConversionScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(input),
            ));

            let pipeline = Default::default();

            let actual = expression.try_resolve_static(&pipeline).unwrap();

            assert_eq!(Some(expected), actual.as_ref().map(|v| v.to_value()));
        };

        run_test(
            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                QueryLocation::new_fake(),
                18,
            )),
            Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "18",
            )),
        );

        run_test(
            StaticScalarExpression::Null(NullScalarExpression::new(QueryLocation::new_fake())),
            Value::String(&StringScalarExpression::new(QueryLocation::new_fake(), "")),
        );
    }
}
