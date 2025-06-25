use crate::{
    DoubleScalarExpression, Expression, ExpressionError, IntegerScalarExpression,
    LogicalExpression, QueryLocation, StaticScalarExpression, Value, ValueAccessor,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarExpression {
    /// Resolve a value from the mutable query source.
    Source(SourceScalarExpression),

    /// Resolve a value from an immutable record attached to a query.
    ///
    /// Attached data is related to the query source but not necessarily owned.
    /// For example when processing an OpenTelemetry LogRecord it is common to
    /// inspect the Resource and/or Instrumentation Scope associated with the
    /// LogRecord. In the context of the query engine "resource" and "scope"
    /// would be considered attached data. This data is immutable because it may
    /// be associated to many other LogRecords and mutation could impact
    /// unrelated records.
    Attached(AttachedScalarExpression),

    /// Resolve a value from a query variable.
    ///
    /// Note: Variables are scoped to the execution of a query for a given
    /// record. Each time a query is run for a record it starts with no
    /// variables defined. Variables cannot be shared or reused across
    /// executions.
    Variable(VariableScalarExpression),

    /// Resolve a static value provided directly in a query.
    Static(StaticScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(NegateScalarExpression),

    /// Boolean value returned by the inner logical expression.
    Logical(Box<LogicalExpression>),

    /// Returns one of two inner scalar expressions based on a logical condition.
    Conditional(ConditionalScalarExpression),
}

impl ScalarExpression {
    pub fn to_value(&self) -> Option<Value> {
        if let ScalarExpression::Static(s) = self {
            return Some(s.to_value());
        }
        None
    }

    pub fn try_resolve_static(&self) -> Result<Option<StaticScalarExpression>, ExpressionError> {
        match self {
            ScalarExpression::Source(_) => Ok(None),
            ScalarExpression::Attached(_) => Ok(None),
            ScalarExpression::Variable(_) => Ok(None),
            ScalarExpression::Static(s) => Ok(Some(s.clone())),
            ScalarExpression::Negate(n) => n.try_resolve_static(),
            ScalarExpression::Logical(l) => l.try_resolve_static(),
            ScalarExpression::Conditional(c) => c.try_resolve_static(),
        }
    }
}

impl Expression for ScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ScalarExpression::Source(s) => s.get_query_location(),
            ScalarExpression::Attached(a) => a.get_query_location(),
            ScalarExpression::Variable(v) => v.get_query_location(),
            ScalarExpression::Static(s) => s.get_query_location(),
            ScalarExpression::Negate(n) => n.get_query_location(),
            ScalarExpression::Logical(l) => l.get_query_location(),
            ScalarExpression::Conditional(c) => c.get_query_location(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceScalarExpression {
    query_location: QueryLocation,
    accessor: ValueAccessor,
}

impl SourceScalarExpression {
    pub fn new(query_location: QueryLocation, accessor: ValueAccessor) -> SourceScalarExpression {
        Self {
            query_location,
            accessor,
        }
    }

    pub fn get_value_accessor(&self) -> &ValueAccessor {
        &self.accessor
    }
}

impl Expression for SourceScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttachedScalarExpression {
    query_location: QueryLocation,
    name: Box<str>,
    accessor: ValueAccessor,
}

impl AttachedScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        name: &str,
        accessor: ValueAccessor,
    ) -> AttachedScalarExpression {
        Self {
            query_location,
            name: name.into(),
            accessor,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value_accessor(&self) -> &ValueAccessor {
        &self.accessor
    }
}

impl Expression for AttachedScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableScalarExpression {
    query_location: QueryLocation,
    name: Box<str>,
    accessor: ValueAccessor,
}

impl VariableScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        name: &str,
        accessor: ValueAccessor,
    ) -> VariableScalarExpression {
        Self {
            query_location,
            name: name.into(),
            accessor,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_value_accessor(&self) -> &ValueAccessor {
        &self.accessor
    }
}

impl Expression for VariableScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NegateScalarExpression {
    query_location: QueryLocation,
    inner_expression: Box<ScalarExpression>,
}

impl NegateScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: ScalarExpression,
    ) -> NegateScalarExpression {
        Self {
            query_location,
            inner_expression: inner_expression.into(),
        }
    }

    pub fn get_inner_expression(&self) -> &ScalarExpression {
        &self.inner_expression
    }

    pub fn try_resolve_static(&self) -> Result<Option<StaticScalarExpression>, ExpressionError> {
        if let Some(s) = self.inner_expression.try_resolve_static()? {
            match s {
                StaticScalarExpression::Integer(i) => {
                    return Ok(Some(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(
                            self.query_location.clone(),
                            -i.get_value(),
                        ),
                    )));
                }
                StaticScalarExpression::Double(i) => {
                    return Ok(Some(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(
                            self.query_location.clone(),
                            i.get_value() * -1.0,
                        ),
                    )));
                }
                _ => {
                    return Err(ExpressionError::TypeMismatch(
                        self.query_location.clone(),
                        "Negate expression can only be used with integer and double types".into(),
                    ));
                }
            }
        }

        Ok(None)
    }
}

impl Expression for NegateScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalScalarExpression {
    query_location: QueryLocation,
    condition: Box<LogicalExpression>,
    true_expression: Box<ScalarExpression>,
    false_expression: Box<ScalarExpression>,
}

impl ConditionalScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        condition: LogicalExpression,
        true_expression: ScalarExpression,
        false_expression: ScalarExpression,
    ) -> ConditionalScalarExpression {
        Self {
            query_location,
            condition: condition.into(),
            true_expression: true_expression.into(),
            false_expression: false_expression.into(),
        }
    }

    pub fn get_condition(&self) -> &LogicalExpression {
        &self.condition
    }

    pub fn get_true_expression(&self) -> &ScalarExpression {
        &self.true_expression
    }

    pub fn get_false_expression(&self) -> &ScalarExpression {
        &self.false_expression
    }

    pub fn try_resolve_static(&self) -> Result<Option<StaticScalarExpression>, ExpressionError> {
        let condition = self.condition.try_resolve_static()?;

        if condition.is_none() {
            return Ok(None);
        }

        match condition.unwrap() {
            StaticScalarExpression::Boolean(b) => {
                if b.get_value() {
                    let true_e = self.true_expression.try_resolve_static()?;

                    if true_e.is_none() {
                        return Ok(None);
                    }

                    return Ok(Some(true_e.unwrap()));
                }

                let false_e = self.false_expression.try_resolve_static()?;

                if false_e.is_none() {
                    return Ok(None);
                }

                Ok(Some(false_e.unwrap()))
            }
            _ => panic!("LogicalExpression did not return a bool value"),
        }
    }
}

impl Expression for ConditionalScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[cfg(test)]
mod tests {
    use crate::BooleanScalarExpression;

    use super::*;

    #[test]
    pub fn test_try_resolve_static() {
        let run_test_success = |expression: ScalarExpression, expected: Option<StaticScalarExpression>| {
            let actual = expression.try_resolve_static().unwrap();

            assert_eq!(expected, actual)
        };

        run_test_success(
            ScalarExpression::Attached(AttachedScalarExpression::new(QueryLocation::new_fake(), "resource", ValueAccessor::new())),
            None
        );

        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new())),
            None
        );

        run_test_success(
            ScalarExpression::Variable(VariableScalarExpression::new(QueryLocation::new_fake(), "var", ValueAccessor::new())),
            None
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true))),
            Some(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))
        );

        run_test_success(
            ScalarExpression::Negate(NegateScalarExpression::new(QueryLocation::new_fake(), ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))))),
            Some(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), -1)))
        );

        run_test_success(
            ScalarExpression::Logical(LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))).into()),
            Some(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))
        );

        run_test_success(
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))),
                ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new())))),
            Some(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1)))
        );
    }

    #[test]
    pub fn test_negate_try_resolve_static() {
        let run_test_success = |expression: NegateScalarExpression, expected: Option<StaticScalarExpression>| {
            let actual = expression.try_resolve_static().unwrap();

            assert_eq!(expected, actual)
        };

        let run_test_failure = |expression: NegateScalarExpression| {
            let actual = expression.try_resolve_static().unwrap_err();

            assert!(matches!(actual, ExpressionError::TypeMismatch(_, _)));
        };

        run_test_success(
            NegateScalarExpression::new(QueryLocation::new_fake(), ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new()))),
            None
        );

        run_test_success(
            NegateScalarExpression::new(QueryLocation::new_fake(), ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1)))),
            Some(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), -1)))
        );

        run_test_success(
            NegateScalarExpression::new(QueryLocation::new_fake(), ScalarExpression::Static(StaticScalarExpression::Double(DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0)))),
            Some(StaticScalarExpression::Double(DoubleScalarExpression::new(QueryLocation::new_fake(), -1.0)))
        );

        run_test_failure(
            NegateScalarExpression::new(QueryLocation::new_fake(), ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))),
        );
    }

    #[test]
    pub fn test_conditional_try_resolve_static() {
        let run_test_success = |expression: ConditionalScalarExpression, expected: Option<StaticScalarExpression>| {
            let actual = expression.try_resolve_static().unwrap();

            assert_eq!(expected, actual)
        };

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new()))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 0)))),
            None
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))),
                ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new())),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 0)))),
            None
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), false)))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))),
                ScalarExpression::Source(SourceScalarExpression::new(QueryLocation::new_fake(), ValueAccessor::new()))),
            None
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), true)))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 0)))),
            Some(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1)))
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(BooleanScalarExpression::new(QueryLocation::new_fake(), false)))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 1))),
                ScalarExpression::Static(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 0)))),
            Some(StaticScalarExpression::Integer(IntegerScalarExpression::new(QueryLocation::new_fake(), 0)))
        );
    }
}