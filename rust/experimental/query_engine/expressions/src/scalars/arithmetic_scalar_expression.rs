use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticScalarExpression {
    Add(ArithmeticOperationExpression),
    Subtract(ArithmeticOperationExpression),
    Multiply(ArithmeticOperationExpression),
    Divide(ArithmeticOperationExpression),
    Modulo(ArithmeticOperationExpression),
}

impl ArithmeticScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        let expr = match self {
            ArithmeticScalarExpression::Add(expr)
            | ArithmeticScalarExpression::Subtract(expr)
            | ArithmeticScalarExpression::Multiply(expr)
            | ArithmeticScalarExpression::Divide(expr)
            | ArithmeticScalarExpression::Modulo(expr) => expr,
        };

        let left_type = expr.left.try_resolve_value_type(pipeline)?;
        let right_type = expr.right.try_resolve_value_type(pipeline)?;

        match (left_type, right_type) {
            (Some(left), Some(right)) => {
                match self {
                    ArithmeticScalarExpression::Add(_)
                    | ArithmeticScalarExpression::Subtract(_)
                    | ArithmeticScalarExpression::Multiply(_) => {
                        match (&left, &right) {
                            (ValueType::Integer, ValueType::Integer) => {
                                Ok(Some(ValueType::Integer))
                            }
                            (ValueType::Double, ValueType::Double) => Ok(Some(ValueType::Double)),
                            (ValueType::Integer, ValueType::Double)
                            | (ValueType::Double, ValueType::Integer) => {
                                Ok(Some(ValueType::Double))
                            }
                            _ => Ok(None), // Invalid types for arithmetic
                        }
                    }
                    ArithmeticScalarExpression::Divide(_) => match (&left, &right) {
                        (ValueType::Integer, ValueType::Integer)
                        | (ValueType::Double, ValueType::Double)
                        | (ValueType::Integer, ValueType::Double)
                        | (ValueType::Double, ValueType::Integer) => Ok(Some(ValueType::Double)),
                        _ => Ok(None),
                    },
                    ArithmeticScalarExpression::Modulo(_) => match (&left, &right) {
                        (ValueType::Integer, ValueType::Integer) => Ok(Some(ValueType::Integer)),
                        _ => Ok(None),
                    },
                }
            }
            _ => Ok(None), // One or both operands have unknown type
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
        let expr = match self {
            ArithmeticScalarExpression::Add(expr)
            | ArithmeticScalarExpression::Subtract(expr)
            | ArithmeticScalarExpression::Multiply(expr)
            | ArithmeticScalarExpression::Divide(expr)
            | ArithmeticScalarExpression::Modulo(expr) => expr,
        };

        let left_static = expr.left.try_resolve_static(pipeline)?;
        let right_static = expr.right.try_resolve_static(pipeline)?;

        match (left_static, right_static) {
            (Some(left), Some(right)) => {
                let left_value = left.to_value();
                let right_value = right.to_value();

                let result = match self {
                    ArithmeticScalarExpression::Add(_) => {
                        match (&left_value, &right_value) {
                            (Value::Integer(l), Value::Integer(r)) => Some(
                                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                    expr.query_location.clone(),
                                    l.get_value() + r.get_value(),
                                )),
                            ),
                            (Value::Double(l), Value::Double(r)) => {
                                Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                    expr.query_location.clone(),
                                    l.get_value() + r.get_value(),
                                )))
                            }
                            _ => {
                                // Try to convert to double
                                if let (Some(l), Some(r)) = (
                                    left_value.convert_to_double(),
                                    right_value.convert_to_double(),
                                ) {
                                    Some(StaticScalarExpression::Double(
                                        DoubleScalarExpression::new(
                                            expr.query_location.clone(),
                                            l + r,
                                        ),
                                    ))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    ArithmeticScalarExpression::Subtract(_) => match (&left_value, &right_value) {
                        (Value::Integer(l), Value::Integer(r)) => Some(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                expr.query_location.clone(),
                                l.get_value() - r.get_value(),
                            )),
                        ),
                        (Value::Double(l), Value::Double(r)) => {
                            Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                expr.query_location.clone(),
                                l.get_value() - r.get_value(),
                            )))
                        }
                        _ => {
                            if let (Some(l), Some(r)) = (
                                left_value.convert_to_double(),
                                right_value.convert_to_double(),
                            ) {
                                Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                    expr.query_location.clone(),
                                    l - r,
                                )))
                            } else {
                                None
                            }
                        }
                    },
                    ArithmeticScalarExpression::Multiply(_) => match (&left_value, &right_value) {
                        (Value::Integer(l), Value::Integer(r)) => Some(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                expr.query_location.clone(),
                                l.get_value() * r.get_value(),
                            )),
                        ),
                        (Value::Double(l), Value::Double(r)) => {
                            Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                expr.query_location.clone(),
                                l.get_value() * r.get_value(),
                            )))
                        }
                        _ => {
                            if let (Some(l), Some(r)) = (
                                left_value.convert_to_double(),
                                right_value.convert_to_double(),
                            ) {
                                Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                    expr.query_location.clone(),
                                    l * r,
                                )))
                            } else {
                                None
                            }
                        }
                    },
                    ArithmeticScalarExpression::Divide(_) => {
                        // Division always returns double
                        if let (Some(l), Some(r)) = (
                            left_value.convert_to_double(),
                            right_value.convert_to_double(),
                        ) {
                            if r == 0.0 {
                                return Err(ExpressionError::ValidationFailure(
                                    expr.query_location.clone(),
                                    "Division by zero".to_string(),
                                ));
                            }
                            Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                                expr.query_location.clone(),
                                l / r,
                            )))
                        } else {
                            None
                        }
                    }
                    ArithmeticScalarExpression::Modulo(_) => match (&left_value, &right_value) {
                        (Value::Integer(l), Value::Integer(r)) => {
                            if r.get_value() == 0 {
                                return Err(ExpressionError::ValidationFailure(
                                    expr.query_location.clone(),
                                    "Modulo by zero".to_string(),
                                ));
                            }
                            Some(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(
                                    expr.query_location.clone(),
                                    l.get_value() % r.get_value(),
                                ),
                            ))
                        }
                        _ => None,
                    },
                };

                Ok(result.map(ResolvedStaticScalarExpression::Value))
            }
            _ => Ok(None), // Can't resolve statically if operands aren't static
        }
    }
}

impl Expression for ArithmeticScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ArithmeticScalarExpression::Add(expr)
            | ArithmeticScalarExpression::Subtract(expr)
            | ArithmeticScalarExpression::Multiply(expr)
            | ArithmeticScalarExpression::Divide(expr)
            | ArithmeticScalarExpression::Modulo(expr) => &expr.query_location,
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ArithmeticScalarExpression::Add(_) => "ArithmeticScalar(Add)",
            ArithmeticScalarExpression::Subtract(_) => "ArithmeticScalar(Subtract)",
            ArithmeticScalarExpression::Multiply(_) => "ArithmeticScalar(Multiply)",
            ArithmeticScalarExpression::Divide(_) => "ArithmeticScalar(Divide)",
            ArithmeticScalarExpression::Modulo(_) => "ArithmeticScalar(Modulo)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArithmeticOperationExpression {
    query_location: QueryLocation,
    left: Box<ScalarExpression>,
    right: Box<ScalarExpression>,
}

impl ArithmeticOperationExpression {
    pub fn new(
        query_location: QueryLocation,
        left: ScalarExpression,
        right: ScalarExpression,
    ) -> Self {
        Self {
            query_location,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn get_left(&self) -> &ScalarExpression {
        &self.left
    }

    pub fn get_right(&self) -> &ScalarExpression {
        &self.right
    }
}

impl Expression for ArithmeticOperationExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ArithmeticOperationExpression"
    }
}
