use crate::{primitives::*, resolved_static_scalar_expression::ResolvedStaticScalarExpression, *};

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

    /// A constant static value defined in a collection on [`PipelineExpression`].
    Constant(ConstantScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(NegateScalarExpression),

    /// Boolean value returned by the inner logical expression.
    Logical(Box<LogicalExpression>),

    /// Returns one of two inner scalar expressions based on a logical condition.
    Conditional(ConditionalScalarExpression),
}

impl ScalarExpression {
    pub fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            ScalarExpression::Source(_) => Ok(None),
            ScalarExpression::Attached(_) => Ok(None),
            ScalarExpression::Variable(_) => Ok(None),
            ScalarExpression::Static(s) => Ok(Some(s.get_value_type())),
            ScalarExpression::Constant(c) => Ok(Some(c.get_value_type())),
            ScalarExpression::Negate(n) => n.try_resolve_value_type(pipeline),
            ScalarExpression::Logical(_) => Ok(Some(ValueType::Boolean)),
            ScalarExpression::Conditional(c) => c.try_resolve_value_type(pipeline),
        }
    }

    pub fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            ScalarExpression::Source(_) => Ok(None),
            ScalarExpression::Attached(_) => Ok(None),
            ScalarExpression::Variable(_) => Ok(None),
            ScalarExpression::Static(s) => Ok(Some(ResolvedStaticScalarExpression::Reference(s))),
            ScalarExpression::Constant(c) => Ok(Some(c.resolve_static(pipeline))),
            ScalarExpression::Negate(n) => n.try_resolve_static(pipeline),
            ScalarExpression::Logical(l) => l.try_resolve_static(pipeline),
            ScalarExpression::Conditional(c) => c.try_resolve_static(pipeline),
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
            ScalarExpression::Constant(c) => c.get_query_location(),
            ScalarExpression::Negate(n) => n.get_query_location(),
            ScalarExpression::Logical(l) => l.get_query_location(),
            ScalarExpression::Conditional(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ScalarExpression::Source(_) => "ScalarExpression(Source)",
            ScalarExpression::Attached(_) => "ScalarExpression(Attached)",
            ScalarExpression::Variable(_) => "ScalarExpression(Variable)",
            ScalarExpression::Static(s) => s.get_name(),
            ScalarExpression::Negate(_) => "ScalarExpression(Negate)",
            ScalarExpression::Logical(_) => "ScalarExpression(Logical)",
            ScalarExpression::Conditional(_) => "ScalarExpression(Conditional)",
            ScalarExpression::Constant(c) => c.get_name(),
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

    fn get_name(&self) -> &'static str {
        "SourceScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttachedScalarExpression {
    query_location: QueryLocation,
    name: StringScalarExpression,
    accessor: ValueAccessor,
}

impl AttachedScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        name: StringScalarExpression,
        accessor: ValueAccessor,
    ) -> AttachedScalarExpression {
        Self {
            query_location,
            name,
            accessor,
        }
    }

    pub fn get_name(&self) -> &StringScalarExpression {
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

    fn get_name(&self) -> &'static str {
        "AttachedScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableScalarExpression {
    query_location: QueryLocation,
    name: StringScalarExpression,
    accessor: ValueAccessor,
}

impl VariableScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        name: StringScalarExpression,
        accessor: ValueAccessor,
    ) -> VariableScalarExpression {
        Self {
            query_location,
            name,
            accessor,
        }
    }

    pub fn get_name(&self) -> &StringScalarExpression {
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

    fn get_name(&self) -> &'static str {
        "VariableScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantScalarExpression {
    /// A constant which is retrieved via a lookup to the collection maintained
    /// on [`PipelineExpression`].
    Reference(ReferenceConstantScalarExpression),

    /// A constant which has been copied from the collection maintained on
    /// [`PipelineExpression`] into a local expression.
    Copy(CopyConstantScalarExpression),
}

impl ConstantScalarExpression {
    pub(crate) fn get_value_type(&self) -> ValueType {
        match self {
            ConstantScalarExpression::Reference(r) => r.get_value_type(),
            ConstantScalarExpression::Copy(c) => c.get_value().get_value_type(),
        }
    }

    pub fn to_value<'a, 'b, 'c>(&'a self, pipeline: &'b PipelineExpression) -> Value<'c>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            ConstantScalarExpression::Reference(r) => {
                let constant_id = r.get_constant_id();

                pipeline
                    .get_constant(constant_id)
                    .unwrap_or_else(|| {
                        panic!("Constant for id '{constant_id}' was not found on pipeline")
                    })
                    .to_value()
            }
            ConstantScalarExpression::Copy(c) => c.value.to_value(),
        }
    }

    pub(crate) fn resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> ResolvedStaticScalarExpression<'c>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            ConstantScalarExpression::Reference(r) => {
                let constant_id = r.get_constant_id();

                ResolvedStaticScalarExpression::Reference(
                    pipeline.get_constant(constant_id).unwrap_or_else(|| {
                        panic!("Constant for id '{constant_id}' was not found on pipeline")
                    }),
                )
            }
            ConstantScalarExpression::Copy(c) => {
                ResolvedStaticScalarExpression::Reference(c.get_value())
            }
        }
    }
}

impl Expression for ConstantScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ConstantScalarExpression::Reference(r) => r.get_query_location(),
            ConstantScalarExpression::Copy(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ConstantScalarExpression::Reference(_) => "ConstantScalar(Reference)",
            ConstantScalarExpression::Copy(_) => "ConstantScalar(Copy)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceConstantScalarExpression {
    query_location: QueryLocation,
    value_type: ValueType,
    constant_id: usize,
}

impl ReferenceConstantScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value_type: ValueType,
        constant_id: usize,
    ) -> ReferenceConstantScalarExpression {
        Self {
            query_location,
            value_type,
            constant_id,
        }
    }

    pub fn get_value_type(&self) -> ValueType {
        self.value_type.clone()
    }

    pub fn get_constant_id(&self) -> usize {
        self.constant_id
    }
}

impl Expression for ReferenceConstantScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ReferenceConstantScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CopyConstantScalarExpression {
    query_location: QueryLocation,
    constant_id: usize,
    value: StaticScalarExpression,
}

impl CopyConstantScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        constant_id: usize,
        value: StaticScalarExpression,
    ) -> CopyConstantScalarExpression {
        Self {
            query_location,
            constant_id,
            value,
        }
    }

    pub fn get_constant_id(&self) -> usize {
        self.constant_id
    }

    pub fn get_value(&self) -> &StaticScalarExpression {
        &self.value
    }
}

impl Expression for CopyConstantScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "CopyConstantScalarExpression"
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

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        let s = self.try_resolve_static(pipeline)?;
        if let Some(s) = s {
            Ok(Some(s.get_value_type()))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn try_resolve_static(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression>, ExpressionError> {
        if let Some(s) = self.inner_expression.try_resolve_static(pipeline)? {
            match s.to_value() {
                Value::Integer(i) => {
                    return Ok(Some(ResolvedStaticScalarExpression::Value(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            self.query_location.clone(),
                            -i.get_value(),
                        )),
                    )));
                }
                Value::Double(d) => {
                    return Ok(Some(ResolvedStaticScalarExpression::Value(
                        StaticScalarExpression::Double(DoubleScalarExpression::new(
                            self.query_location.clone(),
                            -d.get_value(),
                        )),
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

    fn get_name(&self) -> &'static str {
        "NegateScalarExpression"
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

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if let Some(s) = self.try_resolve_static(pipeline)? {
            return Ok(Some(s.get_value_type()));
        }

        let true_e = self.true_expression.try_resolve_static(pipeline)?;
        let false_e = self.false_expression.try_resolve_static(pipeline)?;

        if true_e.is_some() && false_e.is_some() {
            let true_type = true_e.unwrap().get_value_type();
            let false_type = false_e.unwrap().get_value_type();

            if true_type == false_type {
                return Ok(Some(true_type));
            }
        }

        Ok(None)
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let condition = self.condition.try_resolve_static(pipeline)?;

        if condition.is_none() {
            return Ok(None);
        }

        match condition.unwrap().to_value() {
            Value::Boolean(b) => {
                if b.get_value() {
                    let true_e = self.true_expression.try_resolve_static(pipeline)?;

                    if true_e.is_none() {
                        return Ok(None);
                    }

                    return Ok(Some(true_e.unwrap()));
                }

                let false_e = self.false_expression.try_resolve_static(pipeline)?;

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

    fn get_name(&self) -> &'static str {
        "ConditionalScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use crate::{BooleanScalarExpression, StringScalarExpression};

    use super::*;

    #[test]
    pub fn try_resolve_value_type() {
        let run_test_success = |expression: ScalarExpression, expected: Option<ValueType>| {
            let actual = expression
                .try_resolve_value_type(&Default::default())
                .unwrap();

            assert_eq!(expected, actual)
        };

        run_test_success(
            ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "resource".into()),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var".into()),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
            Some(ValueType::Boolean),
        );

        run_test_success(
            ScalarExpression::Negate(NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
            )),
            Some(ValueType::Integer),
        );

        run_test_success(
            ScalarExpression::Logical(
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ))
                .into(),
            ),
            Some(ValueType::Boolean),
        );

        run_test_success(
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
            )),
            Some(ValueType::Integer),
        );

        run_test_success(
            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::String,
                    0,
                ),
            )),
            Some(ValueType::String),
        );

        run_test_success(
            ScalarExpression::Constant(ConstantScalarExpression::Copy(
                CopyConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    0,
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                ),
            )),
            Some(ValueType::Integer),
        );
    }

    #[test]
    pub fn test_conditional_try_resolve_value_type() {
        let run_test_success = |expression: ConditionalScalarExpression,
                                expected: Option<ValueType>| {
            let actual = expression
                .try_resolve_value_type(&Default::default())
                .unwrap();

            assert_eq!(expected, actual)
        };

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                ))),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            Some(ValueType::Integer),
        );

        // Note: Type is not resolved here because true & false branches return
        // different types.
        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                ))),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            Some(ValueType::Integer),
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            Some(ValueType::Integer),
        );
    }

    #[test]
    pub fn test_try_resolve_static() {
        let run_test_success =
            |expression: ScalarExpression, expected: Option<StaticScalarExpression>| {
                let mut pipeline: PipelineExpression = Default::default();

                pipeline.push_constant(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                ));

                let actual = expression.try_resolve_static(&pipeline).unwrap();

                assert_eq!(expected, actual.map(|v| v.as_ref().clone()))
            };

        run_test_success(
            ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "resource".into()),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var".into()),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
            Some(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );

        run_test_success(
            ScalarExpression::Negate(NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
            )),
            Some(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
            )),
        );

        run_test_success(
            ScalarExpression::Logical(
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                ))
                .into(),
            ),
            Some(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        );

        run_test_success(
            ScalarExpression::Conditional(ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
            )),
            Some(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::String,
                    0,
                ),
            )),
            Some(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );

        run_test_success(
            ScalarExpression::Constant(ConstantScalarExpression::Copy(
                CopyConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    0,
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                ),
            )),
            Some(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );
    }

    #[test]
    pub fn test_negate_try_resolve_static() {
        let run_test_success =
            |expression: NegateScalarExpression, expected: Option<StaticScalarExpression>| {
                let pipeline = Default::default();

                let actual_static = expression
                    .try_resolve_static(&pipeline)
                    .unwrap()
                    .map(|v| v.as_ref().clone());

                assert_eq!(expected, actual_static);
                if actual_static.is_none() {
                    assert_eq!(None, expression.try_resolve_value_type(&pipeline).unwrap());
                } else {
                    assert_eq!(
                        ScalarExpression::Static(actual_static.unwrap())
                            .try_resolve_value_type(&pipeline)
                            .unwrap(),
                        expression.try_resolve_value_type(&pipeline).unwrap()
                    );
                }
            };

        let run_test_failure = |expression: NegateScalarExpression| {
            let pipeline = Default::default();

            assert!(matches!(
                expression.try_resolve_static(&pipeline).unwrap_err(),
                ExpressionError::TypeMismatch(_, _)
            ));
            assert!(matches!(
                expression.try_resolve_value_type(&pipeline).unwrap_err(),
                ExpressionError::TypeMismatch(_, _)
            ));
        };

        run_test_success(
            NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
            ),
            None,
        );

        run_test_success(
            NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
            ),
            Some(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
            )),
        );

        run_test_success(
            NegateScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Double(
                    DoubleScalarExpression::new(QueryLocation::new_fake(), 1.0),
                )),
            ),
            Some(StaticScalarExpression::Double(DoubleScalarExpression::new(
                QueryLocation::new_fake(),
                -1.0,
            ))),
        );

        run_test_failure(NegateScalarExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )),
        ));
    }

    #[test]
    pub fn test_conditional_try_resolve_static() {
        let run_test_success =
            |expression: ConditionalScalarExpression, expected: Option<StaticScalarExpression>| {
                let pipeline = Default::default();

                let actual = expression
                    .try_resolve_static(&pipeline)
                    .unwrap()
                    .map(|v| v.as_ref().clone());

                assert_eq!(expected, actual)
            };

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                ))),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
            ),
            None,
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            Some(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        run_test_success(
            ConditionalScalarExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
            Some(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
            )),
        );
    }
}
