use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum MathScalarExpression {
    /// Returns the smallest integral value greater than or equal to the specified number.
    Ceiling(UnaryMathmaticalScalarExpression),

    /// Returns the largest integral value less than or equal to the specified number.
    Floor(UnaryMathmaticalScalarExpression),
}

impl MathScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            MathScalarExpression::Ceiling(u) | MathScalarExpression::Floor(u) => {
                match u.get_value_expression().try_resolve_value_type(pipeline)? {
                    Some(v)
                        if ConvertScalarExpression::is_always_convertable_to_numeric(v.clone()) =>
                    {
                        Ok(Some(ValueType::Integer))
                    }
                    _ => Ok(None),
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
            MathScalarExpression::Ceiling(u) => {
                if let Some(v) = u.get_value_expression().try_resolve_static(pipeline)? {
                    if let Some(i) = Value::ceiling(&v.to_value()) {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                u.query_location.clone(),
                                i,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                u.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
            MathScalarExpression::Floor(u) => {
                if let Some(v) = u.get_value_expression().try_resolve_static(pipeline)? {
                    if let Some(i) = Value::floor(&v.to_value()) {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Integer(IntegerScalarExpression::new(
                                u.query_location.clone(),
                                i,
                            )),
                        )))
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::Null(NullScalarExpression::new(
                                u.query_location.clone(),
                            )),
                        )))
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Expression for MathScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            MathScalarExpression::Ceiling(u) => u.get_query_location(),
            MathScalarExpression::Floor(u) => u.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            MathScalarExpression::Ceiling(_) => "MathScalar(Ceiling)",
            MathScalarExpression::Floor(_) => "MathScalar(Floor)",
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
