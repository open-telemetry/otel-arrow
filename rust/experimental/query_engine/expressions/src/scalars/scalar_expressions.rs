use crate::{primitives::*, resolved_static_scalar_expression::ResolvedStaticScalarExpression, *};

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarExpression {
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

    /// A constant static value defined in a collection on [`PipelineExpression`].
    Constant(ConstantScalarExpression),

    /// Returns one of many inner scalar expressions based on multiple logical conditions.
    Case(CaseScalarExpression),

    /// Returns the first non-null scalar expression in a list.
    Coalesce(CoalesceScalarExpression),

    /// Returns one of two inner scalar expressions based on a logical condition.
    Conditional(ConditionalScalarExpression),

    /// Convert scalar values into different types.
    Convert(ConvertScalarExpression),

    /// Contains scalar functions for working with date and time values.
    Temporal(TemporalScalarExpression),

    /// Returns the number of characters in an inner string value, the number of
    /// items in an inner array/map values, or null for invalid input.
    Length(LengthScalarExpression),

    /// Boolean value returned by the inner logical expression.
    Logical(Box<LogicalExpression>),

    /// Contains scalar functions for performing mathematical operations.
    Math(MathScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(NegateScalarExpression),

    /// Contains scalar functions for performing parsing operations.
    Parse(ParseScalarExpression),

    /// Returns a string with all occurrences of a lookup value replaced with a
    /// replacement value or null for invalid input.
    ReplaceString(ReplaceStringScalarExpression),

    /// Returns a slice of characters from an inner string value, a slice of
    /// items from an inner array value, or null for invalid input.
    Slice(SliceScalarExpression),

    /// Resolve a value from the mutable query source.
    Source(SourceScalarExpression),

    /// Resolve a static value provided directly in a query.
    Static(StaticScalarExpression),

    /// Resolve a value from a query variable.
    ///
    /// Note: Variables are scoped to the execution of a query for a given
    /// record. Each time a query is run for a record it starts with no
    /// variables defined. Variables cannot be shared or reused across
    /// executions.
    Variable(VariableScalarExpression),
}

impl ScalarExpression {
    pub fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            ScalarExpression::Source(s) => Ok(s.get_value_type()),
            ScalarExpression::Attached(_) => Ok(None),
            ScalarExpression::Variable(_) => Ok(None),
            ScalarExpression::Static(s) => Ok(Some(s.get_value_type())),
            ScalarExpression::Constant(c) => Ok(Some(c.get_value_type())),
            ScalarExpression::Negate(n) => n.try_resolve_value_type(pipeline),
            ScalarExpression::Logical(_) => Ok(Some(ValueType::Boolean)),
            ScalarExpression::Coalesce(c) => c.try_resolve_value_type(pipeline),
            ScalarExpression::Conditional(c) => c.try_resolve_value_type(pipeline),
            ScalarExpression::Case(c) => c.try_resolve_value_type(pipeline),
            ScalarExpression::Convert(c) => c.try_resolve_value_type(pipeline),
            ScalarExpression::Length(l) => l.try_resolve_value_type(pipeline),
            ScalarExpression::ReplaceString(r) => r.try_resolve_value_type(pipeline),
            ScalarExpression::Slice(s) => s.try_resolve_value_type(pipeline),
            ScalarExpression::Parse(p) => p.try_resolve_value_type(pipeline),
            ScalarExpression::Temporal(t) => t.try_resolve_value_type(pipeline),
            ScalarExpression::Math(m) => m.try_resolve_value_type(pipeline),
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
            ScalarExpression::Coalesce(c) => c.try_resolve_static(pipeline),
            ScalarExpression::Conditional(c) => c.try_resolve_static(pipeline),
            ScalarExpression::Case(c) => c.try_resolve_static(pipeline),
            ScalarExpression::Convert(c) => c.try_resolve_static(pipeline),
            ScalarExpression::Length(l) => l.try_resolve_static(pipeline),
            ScalarExpression::ReplaceString(r) => r.try_resolve_static(pipeline),
            ScalarExpression::Slice(s) => s.try_resolve_static(pipeline),
            ScalarExpression::Parse(p) => p.try_resolve_static(pipeline),
            ScalarExpression::Temporal(t) => t.try_resolve_static(pipeline),
            ScalarExpression::Math(m) => m.try_resolve_static(pipeline),
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
            ScalarExpression::Coalesce(c) => c.get_query_location(),
            ScalarExpression::Conditional(c) => c.get_query_location(),
            ScalarExpression::Case(c) => c.get_query_location(),
            ScalarExpression::Convert(c) => c.get_query_location(),
            ScalarExpression::Length(l) => l.get_query_location(),
            ScalarExpression::ReplaceString(r) => r.get_query_location(),
            ScalarExpression::Slice(s) => s.get_query_location(),
            ScalarExpression::Parse(p) => p.get_query_location(),
            ScalarExpression::Temporal(t) => t.get_query_location(),
            ScalarExpression::Math(m) => m.get_query_location(),
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
            ScalarExpression::Coalesce(_) => "ScalarExpression(Coalesce)",
            ScalarExpression::Conditional(_) => "ScalarExpression(Conditional)",
            ScalarExpression::Case(_) => "ScalarExpression(Case)",
            ScalarExpression::Constant(c) => c.get_name(),
            ScalarExpression::Convert(c) => c.get_name(),
            ScalarExpression::Length(_) => "ScalarExpression(Length)",
            ScalarExpression::ReplaceString(_) => "ScalarExpression(ReplaceString)",
            ScalarExpression::Slice(_) => "ScalarExpression(Slice)",
            ScalarExpression::Parse(p) => p.get_name(),
            ScalarExpression::Temporal(t) => t.get_name(),
            ScalarExpression::Math(m) => m.get_name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceScalarExpression {
    query_location: QueryLocation,
    accessor: ValueAccessor,
    value_type: Option<ValueType>,
}

impl SourceScalarExpression {
    pub fn new(query_location: QueryLocation, accessor: ValueAccessor) -> SourceScalarExpression {
        let mut value_type = None;
        if !accessor.has_selectors() {
            // Note: The root source it is always a map value so we can
            // automatically set the type when no selectors are present.
            value_type = Some(ValueType::Map);
        }
        Self::new_with_value_type(query_location, accessor, value_type)
    }

    pub fn new_with_value_type(
        query_location: QueryLocation,
        accessor: ValueAccessor,
        value_type: Option<ValueType>,
    ) -> SourceScalarExpression {
        Self {
            query_location,
            accessor,
            value_type,
        }
    }

    pub fn get_value_accessor(&self) -> &ValueAccessor {
        &self.accessor
    }

    pub fn get_value_type(&self) -> Option<ValueType> {
        self.value_type.clone()
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
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
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
pub struct CoalesceScalarExpression {
    query_location: QueryLocation,
    expressions: Vec<ScalarExpression>,
}

impl CoalesceScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        expressions: Vec<ScalarExpression>,
    ) -> CoalesceScalarExpression {
        Self {
            query_location,
            expressions,
        }
    }

    pub fn get_expressions(&self) -> &Vec<ScalarExpression> {
        &self.expressions
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        for expression in &self.expressions {
            match expression.try_resolve_value_type(pipeline)? {
                Some(r) => {
                    if r != ValueType::Null {
                        return Ok(Some(r));
                    }
                }
                None => return Ok(None),
            }
        }

        Ok(Some(ValueType::Null))
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        for expression in &self.expressions {
            match expression.try_resolve_static(pipeline)? {
                Some(r) => {
                    if r.get_value_type() != ValueType::Null {
                        return Ok(Some(r));
                    }
                }
                None => return Ok(None),
            }
        }

        Ok(Some(ResolvedStaticScalarExpression::Value(
            StaticScalarExpression::Null(NullScalarExpression::new(self.query_location.clone())),
        )))
    }
}

impl Expression for CoalesceScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "CoalesceScalarExpression"
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
            if let (Some(true_expr), Some(false_expr)) = (true_e, false_e) {
                let true_type = true_expr.get_value_type();
                let false_type = false_expr.get_value_type();

                if true_type == false_type {
                    return Ok(Some(true_type));
                }
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

#[derive(Debug, Clone, PartialEq)]
pub struct CaseScalarExpression {
    query_location: QueryLocation,
    expressions_with_conditions: Vec<(LogicalExpression, ScalarExpression)>,
    else_expression: Box<ScalarExpression>,
}

impl CaseScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        expressions_with_conditions: Vec<(LogicalExpression, ScalarExpression)>,
        else_expression: ScalarExpression,
    ) -> CaseScalarExpression {
        Self {
            query_location,
            expressions_with_conditions,
            else_expression: else_expression.into(),
        }
    }

    pub fn get_expressions_with_conditions(&self) -> &Vec<(LogicalExpression, ScalarExpression)> {
        &self.expressions_with_conditions
    }

    pub fn get_else_expression(&self) -> &ScalarExpression {
        &self.else_expression
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if let Some(s) = self.try_resolve_static(pipeline)? {
            return Ok(Some(s.get_value_type()));
        }

        // Check if all expressions (including else) have the same static type
        let mut resolved_type: Option<ValueType> = None;

        for (_, expr) in &self.expressions_with_conditions {
            if let Some(expr_static) = expr.try_resolve_static(pipeline)? {
                let expr_type = expr_static.get_value_type();
                if let Some(existing_type) = &resolved_type {
                    if *existing_type != expr_type {
                        return Ok(None); // Types don't match
                    }
                } else {
                    resolved_type = Some(expr_type);
                }
            } else {
                return Ok(None); // Can't resolve type
            }
        }

        if let Some(else_static) = self.else_expression.try_resolve_static(pipeline)? {
            let else_type = else_static.get_value_type();
            if let Some(existing_type) = &resolved_type {
                if *existing_type == else_type {
                    return Ok(Some(else_type));
                }
            } else {
                return Ok(Some(else_type));
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
        // Check each condition in order
        for (condition, expression) in &self.expressions_with_conditions {
            let condition_result = condition.try_resolve_static(pipeline)?;

            if condition_result.is_none() {
                return Ok(None);
            }

            match condition_result.unwrap().to_value() {
                Value::Boolean(b) => {
                    if b.get_value() {
                        // This condition is true, return its expression
                        let expr_result = expression.try_resolve_static(pipeline)?;
                        if expr_result.is_none() {
                            return Ok(None);
                        }
                        return Ok(Some(expr_result.unwrap()));
                    }
                    // This condition is false, continue to next condition
                }
                _ => panic!("LogicalExpression did not return a bool value"),
            }
        }

        // No condition was true, return else expression
        let else_result = self.else_expression.try_resolve_static(pipeline)?;
        if else_result.is_none() {
            return Ok(None);
        }
        Ok(Some(else_result.unwrap()))
    }
}

impl Expression for CaseScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "CaseScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LengthScalarExpression {
    query_location: QueryLocation,
    inner_expression: Box<ScalarExpression>,
}

impl LengthScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: ScalarExpression,
    ) -> LengthScalarExpression {
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
        if let Some(v) = self
            .get_inner_expression()
            .try_resolve_value_type(pipeline)?
        {
            Ok(Some(match v {
                ValueType::String | ValueType::Array | ValueType::Map => ValueType::Integer,
                _ => ValueType::Null,
            }))
        } else {
            Ok(None)
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
        if let Some(v) = self.get_inner_expression().try_resolve_static(pipeline)? {
            Ok(Some(ResolvedStaticScalarExpression::Value(
                match v.to_value() {
                    Value::String(s) => {
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            self.query_location.clone(),
                            s.get_value().chars().count() as i64,
                        ))
                    }
                    Value::Array(a) => StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(self.query_location.clone(), a.len() as i64),
                    ),
                    Value::Map(m) => StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        self.query_location.clone(),
                        m.len() as i64,
                    )),
                    _ => StaticScalarExpression::Null(NullScalarExpression::new(
                        self.query_location.clone(),
                    )),
                },
            )))
        } else {
            Ok(None)
        }
    }
}

impl Expression for LengthScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "LengthScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceStringScalarExpression {
    query_location: QueryLocation,
    haystack_expression: Box<ScalarExpression>,
    needle_expression: Box<ScalarExpression>,
    replacement_expression: Box<ScalarExpression>,
    case_insensitive: bool,
}

impl ReplaceStringScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        haystack_expression: ScalarExpression,
        needle_expression: ScalarExpression,
        replacement_expression: ScalarExpression,
        case_insensitive: bool,
    ) -> ReplaceStringScalarExpression {
        Self {
            query_location,
            haystack_expression: haystack_expression.into(),
            needle_expression: needle_expression.into(),
            replacement_expression: replacement_expression.into(),
            case_insensitive,
        }
    }

    pub fn get_haystack_expression(&self) -> &ScalarExpression {
        &self.haystack_expression
    }

    pub fn get_needle_expression(&self) -> &ScalarExpression {
        &self.needle_expression
    }

    pub fn get_replacement_expression(&self) -> &ScalarExpression {
        &self.replacement_expression
    }

    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if let Some(v) = self
            .get_haystack_expression()
            .try_resolve_value_type(pipeline)?
        {
            Ok(Some(match v {
                ValueType::String => ValueType::String,
                _ => ValueType::Null,
            }))
        } else {
            Ok(None)
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
        let haystack_static = self
            .get_haystack_expression()
            .try_resolve_static(pipeline)?;
        let needle_static = self.get_needle_expression().try_resolve_static(pipeline)?;
        let replacement_static = self
            .get_replacement_expression()
            .try_resolve_static(pipeline)?;

        if let (Some(haystack), Some(needle), Some(replacement)) =
            (haystack_static, needle_static, replacement_static)
        {
            let haystack_value = haystack.to_value();
            let needle_value = needle.to_value();
            let replacement_value = replacement.to_value();

            if let Some(result) = Value::replace_matches(
                &haystack_value,
                &needle_value,
                &replacement_value,
                self.case_insensitive,
            ) {
                Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        self.query_location.clone(),
                        &result,
                    )),
                )))
            } else {
                Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::Null(NullScalarExpression::new(
                        self.query_location.clone(),
                    )),
                )))
            }
        } else {
            Ok(None)
        }
    }
}

impl Expression for ReplaceStringScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ReplaceStringScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SliceScalarExpression {
    query_location: QueryLocation,
    source: Box<ScalarExpression>,
    range_start_inclusive: Option<Box<ScalarExpression>>,
    range_end_exclusive: Option<Box<ScalarExpression>>,
}

impl SliceScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        source: ScalarExpression,
        range_start_inclusive: Option<ScalarExpression>,
        range_end_exclusive: Option<ScalarExpression>,
    ) -> SliceScalarExpression {
        Self {
            query_location,
            source: source.into(),
            range_start_inclusive: range_start_inclusive.map(|v| v.into()),
            range_end_exclusive: range_end_exclusive.map(|v| v.into()),
        }
    }

    pub fn get_source(&self) -> &ScalarExpression {
        &self.source
    }

    pub fn get_range_start_inclusive(&self) -> Option<&ScalarExpression> {
        self.range_start_inclusive.as_deref()
    }

    pub fn get_range_end_exclusive(&self) -> Option<&ScalarExpression> {
        self.range_end_exclusive.as_deref()
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if let Some(s) = self.try_resolve_static(pipeline)? {
            return Ok(Some(s.get_value_type()));
        }

        if let Some(t) = self.get_source().try_resolve_value_type(pipeline)? {
            match t {
                ValueType::Array => Ok(Some(ValueType::Array)),
                ValueType::String => Ok(Some(ValueType::String)),
                _ => Ok(Some(ValueType::Null)),
            }
        } else {
            Ok(None)
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
        let range_start_inclusive = match self.get_range_start_inclusive() {
            Some(s) => match s.try_resolve_static(pipeline)? {
                Some(v) => self.validate_resolved_range_value("start", v.to_value())?,
                None => return Ok(None),
            },
            None => 0,
        };

        let range_end_exclusive = match self.get_range_end_exclusive() {
            Some(s) => match s.try_resolve_static(pipeline)? {
                Some(v) => Some(self.validate_resolved_range_value("end", v.to_value())?),
                None => return Ok(None),
            },
            None => None,
        };

        match self.get_source().try_resolve_static(pipeline)? {
            Some(s) => match s.to_value() {
                Value::Array(a) => {
                    self.validate_slice_range(
                        "Array",
                        a.len(),
                        range_start_inclusive,
                        range_end_exclusive,
                    )?;

                    // Note: We don't return an array slice statically. This
                    // would cause potentially a lot of expansion inside the
                    // expression tree. At runtime a slice of an array is just a
                    // pointer to some data.
                    Ok(None)
                }
                Value::String(s) => {
                    let range_end_exclusive = self.validate_slice_range(
                        "String",
                        s.get_value().chars().count(),
                        range_start_inclusive,
                        range_end_exclusive,
                    )?;

                    // Note: We only return statically small string slices. The
                    // idea here is to prevent expansion of the expression tree
                    // for large values. At runtime a slice of a string is just
                    // a pointer to some data.
                    if range_end_exclusive - range_start_inclusive > 32 {
                        Ok(None)
                    } else {
                        Ok(Some(ResolvedStaticScalarExpression::Value(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                self.query_location.clone(),
                                &Self::slice_string(
                                    s.get_value(),
                                    range_start_inclusive,
                                    range_end_exclusive,
                                ),
                            )),
                        )))
                    }
                }
                _ => Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::Null(NullScalarExpression::new(
                        self.query_location.clone(),
                    )),
                ))),
            },
            None => Ok(None),
        }
    }

    pub fn validate_resolved_range_value(
        &self,
        name: &str,
        value: Value,
    ) -> Result<usize, ExpressionError> {
        if let Value::Integer(i) = value {
            let v = i.get_value();
            if v < 0 {
                return Err(ExpressionError::ValidationFailure(
                    self.get_query_location().clone(),
                    format!("Range {name} for a slice expression cannot be a negative value"),
                ));
            }
            Ok(v as usize)
        } else {
            Err(ExpressionError::TypeMismatch(
                self.get_query_location().clone(),
                format!("Range {name} for a slice expression should be an integer type"),
            ))
        }
    }

    pub fn validate_slice_range(
        &self,
        name: &str,
        target_length: usize,
        range_start_inclusive: usize,
        range_end_exclusive: Option<usize>,
    ) -> Result<usize, ExpressionError> {
        let end = range_end_exclusive.unwrap_or(target_length);

        if range_start_inclusive > end {
            return Err(ExpressionError::ValidationFailure(
                self.query_location.clone(),
                format!(
                    "{name} slice index starts at '{range_start_inclusive}' but ends at '{end}'"
                ),
            ));
        }
        if end > target_length {
            return Err(ExpressionError::ValidationFailure(
                self.query_location.clone(),
                format!(
                    "{name} slice index ends at '{end}' which is beyond the length of '{target_length}'"
                ),
            ));
        }

        Ok(end)
    }

    pub fn slice_string(
        value: &str,
        range_start_inclusive: usize,
        range_end_exclusive: usize,
    ) -> String {
        String::from_iter(
            value
                .chars()
                .skip(range_start_inclusive)
                .take(range_end_exclusive - range_start_inclusive),
        )
    }
}

impl Expression for SliceScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "SliceScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
                StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
                ValueAccessor::new(),
            )),
            None,
        );

        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new(),
            )),
            Some(ValueType::Map),
        );

        run_test_success(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "var"),
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
    pub fn test_coalesce_try_resolve_value_type() {
        let run_test = |expression: CoalesceScalarExpression, expected: Option<Value>| {
            let pipeline = Default::default();

            let actual_type = expression.try_resolve_value_type(&pipeline).unwrap();

            assert_eq!(expected.as_ref().map(|v| v.get_value_type()), actual_type);

            let actual_static = expression.try_resolve_static(&pipeline).unwrap();

            assert_eq!(expected, actual_static.as_ref().map(|v| v.to_value()));
        };

        // Test first expression is unknown
        run_test(
            CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "field",
                        )),
                    )]),
                ))],
            ),
            None,
        );

        // Test first expression is known
        run_test(
            CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                ))],
            ),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "value",
            ))),
        );

        // Test first expression is known null and second expression is known
        run_test(
            CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "value"),
                    )),
                ],
            ),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "value",
            ))),
        );

        // Test first expression is known null and second expression is unknown
        run_test(
            CoalesceScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    ScalarExpression::Static(StaticScalarExpression::Null(
                        NullScalarExpression::new(QueryLocation::new_fake()),
                    )),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "field",
                            )),
                        )]),
                    )),
                ],
            ),
            None,
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
                StringScalarExpression::new(QueryLocation::new_fake(), "resource"),
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
                StringScalarExpression::new(QueryLocation::new_fake(), "var"),
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

    #[test]
    pub fn test_length_scalar_expression_try_resolve() {
        fn run_test(input: Vec<(ScalarExpression, Option<ValueType>, Option<Value>)>) {
            for (inner, expected_type, expected_value) in input {
                let e = LengthScalarExpression::new(QueryLocation::new_fake(), inner);

                let pipeline = Default::default();

                let actual_type = e.try_resolve_value_type(&pipeline).unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e.try_resolve_static(&pipeline).unwrap();
                assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
            }
        }

        run_test(vec![
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "Hello, 世界!"),
                )),
                Some(ValueType::Integer),
                Some(Value::Integer(&IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    10,
                ))),
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
                Some(ValueType::Integer),
                Some(Value::Integer(&IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    2,
                ))),
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
                Some(ValueType::Integer),
                Some(Value::Integer(&IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    2,
                ))),
            ),
            (
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                    )]),
                )),
                None,
                None,
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                Some(ValueType::Null),
                Some(Value::Null),
            ),
        ]);
    }

    #[test]
    #[allow(clippy::type_complexity)]
    pub fn test_replace_string_scalar_expression_try_resolve() {
        fn run_test(
            input: Vec<(
                ScalarExpression,
                ScalarExpression,
                ScalarExpression,
                Option<ValueType>,
                Option<Value>,
            )>,
        ) {
            for (text, lookup, replacement, expected_type, expected_value) in input {
                let e = ReplaceStringScalarExpression::new(
                    QueryLocation::new_fake(),
                    text,
                    lookup,
                    replacement,
                    false, // case_insensitive
                );

                let pipeline = Default::default();

                let actual_type = e.try_resolve_value_type(&pipeline).unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e.try_resolve_static(&pipeline).unwrap();
                assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
            }
        }

        run_test(vec![
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "A magic trick can turn a cat into a dog",
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "cat"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hamster"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "A magic trick can turn a hamster into a dog",
                ))),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world hello"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hi"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hi world hi",
                ))),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "no matches here"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "xyz"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "abc"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "no matches here",
                ))),
            ),
            (
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "search"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "replace"),
                )),
                None,
                None,
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "search"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "replace"),
                )),
                Some(ValueType::Null),
                Some(Value::Null),
            ),
        ]);
    }

    #[test]
    pub fn test_slice_scalar_expression_try_resolve() {
        fn run_test_failure(input: SliceScalarExpression, expected: ExpressionError) {
            let pipeline = Default::default();

            let actual_type = input.try_resolve_value_type(&pipeline).unwrap_err();
            validate_error(&expected, actual_type);

            let actual_value = input.try_resolve_static(&pipeline).unwrap_err();
            validate_error(&expected, actual_value);

            fn validate_error(expected: &ExpressionError, actual: ExpressionError) {
                match expected {
                    ExpressionError::TypeMismatch(_, msg) => {
                        if let ExpressionError::TypeMismatch(_, actual_msg) = actual {
                            assert_eq!(msg, &actual_msg)
                        } else {
                            panic!("Unexpected ExpressionError")
                        }
                    }
                    ExpressionError::ValidationFailure(_, msg) => {
                        if let ExpressionError::ValidationFailure(_, actual_msg) = actual {
                            assert_eq!(msg, &actual_msg)
                        } else {
                            panic!("Unexpected ExpressionError")
                        }
                    }
                    _ => panic!("Unexpected ExpressionError"),
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
    pub fn test_string_slice_scalar_expression_try_resolve() {
        fn run_test_success(
            input: SliceScalarExpression,
            expected_value_type: Option<ValueType>,
            expected_value: Option<Value>,
        ) {
            let pipeline = Default::default();

            let actual_type = input.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(expected_value_type, actual_type);

            let actual_value = input.try_resolve_static(&pipeline).unwrap();
            assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
        }

        fn run_test_failure(input: SliceScalarExpression, expected_msg: &str) {
            let pipeline = Default::default();

            let actual_type = input.try_resolve_value_type(&pipeline).unwrap_err();
            if let ExpressionError::ValidationFailure(_, a) = actual_type {
                assert_eq!(expected_msg, a);
            } else {
                panic!("Unexpected ExpressionError")
            }

            let actual_value = input.try_resolve_static(&pipeline).unwrap_err();
            if let ExpressionError::ValidationFailure(_, a) = actual_value {
                assert_eq!(expected_msg, a);
            } else {
                panic!("Unexpected ExpressionError")
            }
        }

        let small_string_source = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(QueryLocation::new_fake(), "こんにちは"),
        ));

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                None,
                None,
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "こんにちは",
            ))),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                None,
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "んにちは",
            ))),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "",
            ))),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "ん",
            ))),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "こん",
            ))),
        );

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                None,
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                ))),
            ),
            Some(ValueType::String),
            Some(Value::String(&StringScalarExpression::new(
                QueryLocation::new_fake(),
                "こんにちは",
            ))),
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
            ),
            "String slice index starts at '2' but ends at '1'",
        );

        run_test_failure(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                small_string_source.clone(),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                ))),
                Some(ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                ))),
            ),
            "String slice index ends at '6' which is beyond the length of '5'",
        );

        let large_string_source = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Aenean non lectus eget orci aliquam maximus fringilla sed sapien proin",
            ),
        ));

        run_test_success(
            SliceScalarExpression::new(
                QueryLocation::new_fake(),
                large_string_source.clone(),
                None,
                None,
            ),
            Some(ValueType::String),
            None,
        );
    }

    #[test]
    pub fn test_array_slice_scalar_expression_try_resolve() {
        fn run_test_success(
            input: SliceScalarExpression,
            expected_value_type: Option<ValueType>,
            expected_value: Option<Value>,
        ) {
            let pipeline = Default::default();

            let actual_type = input.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(expected_value_type, actual_type);

            let actual_value = input.try_resolve_static(&pipeline).unwrap();
            assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
        }

        fn run_test_failure(input: SliceScalarExpression, expected_msg: &str) {
            let pipeline = Default::default();

            let actual_type = input.try_resolve_value_type(&pipeline).unwrap_err();
            if let ExpressionError::ValidationFailure(_, a) = actual_type {
                assert_eq!(expected_msg, a);
            } else {
                panic!("Unexpected ExpressionError")
            }

            let actual_value = input.try_resolve_static(&pipeline).unwrap_err();
            if let ExpressionError::ValidationFailure(_, a) = actual_value {
                assert_eq!(expected_msg, a);
            } else {
                panic!("Unexpected ExpressionError")
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
            Some(ValueType::Array),
            None,
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
            "Array slice index starts at '2' but ends at '1'",
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
            "Array slice index ends at '6' which is beyond the length of '5'",
        );
    }
}
