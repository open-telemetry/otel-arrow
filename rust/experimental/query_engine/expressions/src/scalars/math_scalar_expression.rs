// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MathScalarExpression {
    /// Returns the sum of two values.
    Add(BinaryMathematicalScalarExpression),

    /// Returns a value rounded down to the nearest multiple of a given bin
    /// size, effectively grouping similar values into buckets.
    Bin(BinaryMathematicalScalarExpression),

    /// Returns the smallest integral value greater than or equal to the specified number.
    Ceiling(UnaryMathematicalScalarExpression),

    /// Returns the result of dividing two values.
    Divide(BinaryMathematicalScalarExpression),

    /// Returns the largest integral value less than or equal to the specified number.
    Floor(UnaryMathematicalScalarExpression),

    /// Returns the remainder of a division operation between two values.
    Modulus(BinaryMathematicalScalarExpression),

    /// Returns the result of multiplying two values.
    Multiply(BinaryMathematicalScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(UnaryMathematicalScalarExpression),

    /// Returns the result of subtracting two values.
    Subtract(BinaryMathematicalScalarExpression),
}

impl MathScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            MathScalarExpression::Ceiling(u) | MathScalarExpression::Floor(u) => {
                match u.get_value_expression().try_resolve_value_type(pipeline)? {
                    Some(v) if ConvertScalarExpression::is_always_convertable_to_numeric(&v) => {
                        Ok(Some(ValueType::Integer))
                    }
                    _ => Ok(None),
                }
            }
            MathScalarExpression::Negate(u) => {
                let value = u
                    .get_value_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                match value {
                    ValueType::Integer => Ok(Some(ValueType::Integer)),
                    ValueType::Double => Ok(Some(ValueType::Double)),
                    ValueType::TimeSpan => Ok(Some(ValueType::TimeSpan)),
                    value => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&value) {
                            Ok(Some(ValueType::Integer))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            MathScalarExpression::Bin(b) => {
                let left = b
                    .get_left_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                let right = b
                    .get_right_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                match (left, right) {
                    (ValueType::Integer, ValueType::Integer) => Ok(Some(ValueType::Integer)),
                    (ValueType::Double, ValueType::Double) => Ok(Some(ValueType::Double)),
                    (ValueType::DateTime, right) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&right) {
                            Ok(Some(ValueType::DateTime))
                        } else {
                            Ok(None)
                        }
                    }
                    (ValueType::TimeSpan, right) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&right) {
                            Ok(Some(ValueType::TimeSpan))
                        } else {
                            Ok(None)
                        }
                    }
                    (left, right) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&left)
                            && ConvertScalarExpression::is_always_convertable_to_numeric(&right)
                        {
                            if left == ValueType::Double || right == ValueType::Double {
                                Ok(Some(ValueType::Double))
                            } else {
                                Ok(Some(ValueType::Integer))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            MathScalarExpression::Add(b) | MathScalarExpression::Subtract(b) => {
                let left = b
                    .get_left_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                let right = b
                    .get_right_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                match (left, right) {
                    (ValueType::Integer, ValueType::Integer) => Ok(Some(ValueType::Integer)),
                    (ValueType::Double, ValueType::Double) => Ok(Some(ValueType::Double)),
                    (ValueType::DateTime, vt) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&vt) {
                            Ok(Some(ValueType::DateTime))
                        } else {
                            Ok(None)
                        }
                    }
                    (ValueType::TimeSpan, vt) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&vt) {
                            Ok(Some(ValueType::TimeSpan))
                        } else {
                            Ok(None)
                        }
                    }
                    (left, right) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&left)
                            && ConvertScalarExpression::is_always_convertable_to_numeric(&right)
                        {
                            if left == ValueType::Double || right == ValueType::Double {
                                Ok(Some(ValueType::Double))
                            } else {
                                Ok(Some(ValueType::Integer))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            MathScalarExpression::Divide(b)
            | MathScalarExpression::Modulus(b)
            | MathScalarExpression::Multiply(b) => {
                let left = b
                    .get_left_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                let right = b
                    .get_right_expression()
                    .try_resolve_value_type(pipeline)?
                    .unwrap_or(ValueType::Null);
                match (left, right) {
                    (ValueType::Integer, ValueType::Integer) => Ok(Some(ValueType::Integer)),
                    (ValueType::Double, ValueType::Double) => Ok(Some(ValueType::Double)),
                    (left, right) => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&left)
                            && ConvertScalarExpression::is_always_convertable_to_numeric(&right)
                        {
                            if left == ValueType::Double || right == ValueType::Double {
                                Ok(Some(ValueType::Double))
                            } else {
                                Ok(Some(ValueType::Integer))
                            }
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
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
            MathScalarExpression::Add(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::add)
            }
            MathScalarExpression::Bin(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::bin)
            }
            MathScalarExpression::Ceiling(u) => {
                Self::try_resolve_static_unary_operation(pipeline, u, |v| {
                    Value::ceiling(v).map(NumericValue::Integer)
                })
            }
            MathScalarExpression::Divide(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::divide)
            }
            MathScalarExpression::Floor(u) => {
                Self::try_resolve_static_unary_operation(pipeline, u, |v| {
                    Value::floor(v).map(NumericValue::Integer)
                })
            }
            MathScalarExpression::Modulus(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::modulus)
            }
            MathScalarExpression::Multiply(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::multiply)
            }
            MathScalarExpression::Negate(u) => {
                Self::try_resolve_static_unary_operation(pipeline, u, Value::negate)
            }
            MathScalarExpression::Subtract(b) => {
                Self::try_resolve_static_binary_operation(pipeline, b, Value::subtract)
            }
        }
    }

    fn try_resolve_static_unary_operation<'a, F>(
        pipeline: &PipelineExpression,
        unary_expression: &UnaryMathematicalScalarExpression,
        op: F,
    ) -> Result<Option<ResolvedStaticScalarExpression<'a>>, ExpressionError>
    where
        F: FnOnce(&Value) -> Option<NumericValue>,
    {
        if let Some(v) = unary_expression
            .get_value_expression()
            .try_resolve_static(pipeline)?
        {
            if let Some(v) = (op)(&v.to_value()) {
                Ok(Some(Self::numeric_value_to_static_value(
                    &unary_expression.query_location,
                    v,
                )))
            } else {
                Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::Null(NullScalarExpression::new(
                        unary_expression.query_location.clone(),
                    )),
                )))
            }
        } else {
            Ok(None)
        }
    }

    fn try_resolve_static_binary_operation<'a, F>(
        pipeline: &PipelineExpression,
        binary_expression: &BinaryMathematicalScalarExpression,
        op: F,
    ) -> Result<Option<ResolvedStaticScalarExpression<'a>>, ExpressionError>
    where
        F: FnOnce(&Value, &Value) -> Option<NumericValue>,
    {
        let left = binary_expression
            .get_left_expression()
            .try_resolve_static(pipeline)?;
        let right = binary_expression
            .get_right_expression()
            .try_resolve_static(pipeline)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                if let Some(v) = (op)(&l.to_value(), &r.to_value()) {
                    Ok(Some(Self::numeric_value_to_static_value(
                        &binary_expression.query_location,
                        v,
                    )))
                } else {
                    Ok(Some(ResolvedStaticScalarExpression::Value(
                        StaticScalarExpression::Null(NullScalarExpression::new(
                            binary_expression.query_location.clone(),
                        )),
                    )))
                }
            }
            _ => Ok(None),
        }
    }

    fn numeric_value_to_static_value<'a>(
        query_location: &QueryLocation,
        value: NumericValue,
    ) -> ResolvedStaticScalarExpression<'a> {
        match value {
            NumericValue::Integer(i) => {
                ResolvedStaticScalarExpression::Value(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(query_location.clone(), i),
                ))
            }
            NumericValue::DateTime(d) => {
                ResolvedStaticScalarExpression::Value(StaticScalarExpression::DateTime(
                    DateTimeScalarExpression::new(query_location.clone(), d),
                ))
            }
            NumericValue::Double(d) => {
                ResolvedStaticScalarExpression::Value(StaticScalarExpression::Double(
                    DoubleScalarExpression::new(query_location.clone(), d),
                ))
            }
            NumericValue::TimeSpan(t) => {
                ResolvedStaticScalarExpression::Value(StaticScalarExpression::TimeSpan(
                    TimeSpanScalarExpression::new(query_location.clone(), t),
                ))
            }
        }
    }
}

impl Expression for MathScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            MathScalarExpression::Add(b) => b.get_query_location(),
            MathScalarExpression::Bin(b) => b.get_query_location(),
            MathScalarExpression::Ceiling(u) => u.get_query_location(),
            MathScalarExpression::Divide(b) => b.get_query_location(),
            MathScalarExpression::Floor(u) => u.get_query_location(),
            MathScalarExpression::Modulus(b) => b.get_query_location(),
            MathScalarExpression::Multiply(b) => b.get_query_location(),
            MathScalarExpression::Negate(u) => u.get_query_location(),
            MathScalarExpression::Subtract(b) => b.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            MathScalarExpression::Add(_) => "MathScalar(Add)",
            MathScalarExpression::Bin(_) => "MathScalar(Bin)",
            MathScalarExpression::Ceiling(_) => "MathScalar(Ceiling)",
            MathScalarExpression::Divide(_) => "MathScalar(Divide)",
            MathScalarExpression::Floor(_) => "MathScalar(Floor)",
            MathScalarExpression::Modulus(_) => "MathScalar(Modulus)",
            MathScalarExpression::Multiply(_) => "MathScalar(Multiply)",
            MathScalarExpression::Negate(_) => "MathScalar(Negate)",
            MathScalarExpression::Subtract(_) => "MathScalar(Subtract)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryMathematicalScalarExpression {
    query_location: QueryLocation,
    value_expression: Box<ScalarExpression>,
}

impl UnaryMathematicalScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value_expression: ScalarExpression,
    ) -> UnaryMathematicalScalarExpression {
        Self {
            query_location,
            value_expression: value_expression.into(),
        }
    }

    pub fn get_value_expression(&self) -> &ScalarExpression {
        &self.value_expression
    }
}

impl Expression for UnaryMathematicalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "UnaryMathematicalScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryMathematicalScalarExpression {
    query_location: QueryLocation,
    left_expression: Box<ScalarExpression>,
    right_expression: Box<ScalarExpression>,
}

impl BinaryMathematicalScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        left_expression: ScalarExpression,
        right_expression: ScalarExpression,
    ) -> BinaryMathematicalScalarExpression {
        Self {
            query_location,
            left_expression: left_expression.into(),
            right_expression: right_expression.into(),
        }
    }

    pub fn get_left_expression(&self) -> &ScalarExpression {
        &self.left_expression
    }

    pub fn get_right_expression(&self) -> &ScalarExpression {
        &self.right_expression
    }
}

impl Expression for BinaryMathematicalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "BinaryMathematicalScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;

    use crate::date_utils::create_utc;

    use super::*;

    #[test]
    pub fn test_try_resolve_unary() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, Option<ValueType>, Option<Value>)>)
        where
            F: Fn(UnaryMathematicalScalarExpression) -> MathScalarExpression,
        {
            for (inner, expected_type, expected_value) in input {
                let e = build(UnaryMathematicalScalarExpression::new(
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
            MathScalarExpression::Ceiling,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Floor,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Negate,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.1),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1.1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.1"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1.1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1,
                    ))),
                ),
            ],
        );
    }

    #[test]
    pub fn test_try_resolve_binary() {
        fn run_test<F>(
            build: F,
            input: Vec<(
                ScalarExpression,
                ScalarExpression,
                Option<ValueType>,
                Option<Value>,
            )>,
        ) where
            F: Fn(BinaryMathematicalScalarExpression) -> MathScalarExpression,
        {
            for (left, right, expected_type, expected_value) in input {
                let e = build(BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    left,
                    right,
                ));

                let pipeline = Default::default();

                let actual_type = e.try_resolve_value_type(&pipeline).unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e.try_resolve_static(&pipeline).unwrap();
                assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
            }
        }

        run_test(
            MathScalarExpression::Add,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        2.19,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        19,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        3.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 8, 19, 18, 26, 0, 0),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    Some(ValueType::DateTime),
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        create_utc(2025, 8, 20, 18, 26, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    Some(ValueType::TimeSpan),
                    Some(Value::TimeSpan(&TimeSpanScalarExpression::new(
                        QueryLocation::new_fake(),
                        TimeDelta::days(2),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        2.19,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Subtract,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        -0.16999999999999993,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        -17,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        -1.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 8, 19, 18, 26, 0, 0),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    Some(ValueType::DateTime),
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        create_utc(2025, 8, 18, 18, 26, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    Some(ValueType::TimeSpan),
                    Some(Value::TimeSpan(&TimeSpanScalarExpression::new(
                        QueryLocation::new_fake(),
                        TimeDelta::days(0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        -0.16999999999999993,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Multiply,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.1918,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
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
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        2.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.1918,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Divide,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.01),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 1.18),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        0.8559322033898306,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        0.5,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.01"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        0.8559322033898306,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Modulus,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 10.18),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 3.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.1799999999999997,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 3.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10.18"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "3.00"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        1.1799999999999997,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "3"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );

        run_test(
            MathScalarExpression::Bin,
            vec![
                (
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 10018.18),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 100.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        10000.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10018),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 100),
                    )),
                    Some(ValueType::Integer),
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        10000,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10018),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 100.0),
                    )),
                    Some(ValueType::Double),
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        10000.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 8, 19, 18, 26, 30, 10),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(1),
                        ),
                    )),
                    Some(ValueType::DateTime),
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        create_utc(2025, 8, 19, 0, 0, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 8, 19, 18, 26, 30, 10),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::hours(1),
                        ),
                    )),
                    Some(ValueType::DateTime),
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        create_utc(2025, 8, 19, 18, 0, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(
                            QueryLocation::new_fake(),
                            create_utc(2025, 8, 19, 18, 26, 30, 10),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::minutes(1),
                        ),
                    )),
                    Some(ValueType::DateTime),
                    Some(Value::DateTime(&DateTimeScalarExpression::new(
                        QueryLocation::new_fake(),
                        create_utc(2025, 8, 19, 18, 26, 0, 0),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(16),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                        TimeSpanScalarExpression::new(
                            QueryLocation::new_fake(),
                            TimeDelta::days(7),
                        ),
                    )),
                    Some(ValueType::TimeSpan),
                    Some(Value::TimeSpan(&TimeSpanScalarExpression::new(
                        QueryLocation::new_fake(),
                        TimeDelta::days(14),
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10018.18"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "100.0"),
                    )),
                    None,
                    Some(Value::Double(&DoubleScalarExpression::new(
                        QueryLocation::new_fake(),
                        10000.0,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "10018"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "100"),
                    )),
                    None,
                    Some(Value::Integer(&IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        10000,
                    ))),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "1.18"),
                    )),
                    None,
                    Some(Value::Null),
                ),
                (
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    None,
                    Some(Value::Null),
                ),
            ],
        );
    }
}
