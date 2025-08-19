// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MathScalarExpression {
    /// Returns the sum of two values.
    Add(BinaryMathmaticalScalarExpression),

    /// Returns a value rounded down to the nearest multiple of a given bin
    /// size, effectively grouping similar values into buckets.
    Bin(BinaryMathmaticalScalarExpression),

    /// Returns the smallest integral value greater than or equal to the specified number.
    Ceiling(UnaryMathmaticalScalarExpression),

    /// Returns the result of dividing two values.
    Divide(BinaryMathmaticalScalarExpression),

    /// Returns the largest integral value less than or equal to the specified number.
    Floor(UnaryMathmaticalScalarExpression),

    /// Returns the remainder of a division operation between two values.
    Modulus(BinaryMathmaticalScalarExpression),

    /// Returns the result of multiplying two values.
    Multiply(BinaryMathmaticalScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(UnaryMathmaticalScalarExpression),

    /// Returns the result of subtracting two values.
    Subtract(BinaryMathmaticalScalarExpression),
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
                    value => {
                        if ConvertScalarExpression::is_always_convertable_to_numeric(&value) {
                            Ok(Some(ValueType::Integer))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            MathScalarExpression::Add(b)
            | MathScalarExpression::Bin(b)
            | MathScalarExpression::Divide(b)
            | MathScalarExpression::Modulus(b)
            | MathScalarExpression::Multiply(b)
            | MathScalarExpression::Subtract(b) => {
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
        unary_expression: &UnaryMathmaticalScalarExpression,
        op: F,
    ) -> Result<Option<ResolvedStaticScalarExpression<'a>>, ExpressionError>
    where
        F: FnOnce(&Value) -> Option<NumericValue>,
    {
        if let Some(v) = unary_expression
            .get_value_expression()
            .try_resolve_static(pipeline)?
        {
            if let Some(i) = (op)(&v.to_value()) {
                match i {
                    NumericValue::Integer(i) => Ok(Some(ResolvedStaticScalarExpression::Value(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            unary_expression.query_location.clone(),
                            i,
                        )),
                    ))),
                    NumericValue::Double(d) => Ok(Some(ResolvedStaticScalarExpression::Value(
                        StaticScalarExpression::Double(DoubleScalarExpression::new(
                            unary_expression.query_location.clone(),
                            d,
                        )),
                    ))),
                }
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
        binary_expression: &BinaryMathmaticalScalarExpression,
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
                    match v {
                        NumericValue::Integer(v) => {
                            Ok(Some(ResolvedStaticScalarExpression::Value(
                                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                    binary_expression.query_location.clone(),
                                    v,
                                )),
                            )))
                        }
                        NumericValue::Double(v) => Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Double(DoubleScalarExpression::new(
                                binary_expression.query_location.clone(),
                                v,
                            )),
                        ))),
                    }
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
pub struct UnaryMathmaticalScalarExpression {
    query_location: QueryLocation,
    value_expression: Box<ScalarExpression>,
}

impl UnaryMathmaticalScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value_expression: ScalarExpression,
    ) -> UnaryMathmaticalScalarExpression {
        Self {
            query_location,
            value_expression: value_expression.into(),
        }
    }

    pub fn get_value_expression(&self) -> &ScalarExpression {
        &self.value_expression
    }
}

impl Expression for UnaryMathmaticalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "UnaryMathmaticalScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryMathmaticalScalarExpression {
    query_location: QueryLocation,
    left_expression: Box<ScalarExpression>,
    right_expression: Box<ScalarExpression>,
}

impl BinaryMathmaticalScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        left_expression: ScalarExpression,
        right_expression: ScalarExpression,
    ) -> BinaryMathmaticalScalarExpression {
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

impl Expression for BinaryMathmaticalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "BinaryMathmaticalScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_try_resolve_unary() {
        fn run_test<F>(build: F, input: Vec<(ScalarExpression, Option<ValueType>, Option<Value>)>)
        where
            F: Fn(UnaryMathmaticalScalarExpression) -> MathScalarExpression,
        {
            for (inner, expected_type, expected_value) in input {
                let e = build(UnaryMathmaticalScalarExpression::new(
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
            F: Fn(BinaryMathmaticalScalarExpression) -> MathScalarExpression,
        {
            for (left, right, expected_type, expected_value) in input {
                let e = build(BinaryMathmaticalScalarExpression::new(
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
