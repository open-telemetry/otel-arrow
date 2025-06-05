use crate::{Expression, QueryLocation, ValueAccessor, ValueSelector};

#[derive(Debug, Clone, PartialEq)]
pub enum ScalarExpression
{
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

    /// Resolve a static integer value provided directly in a query.
    Integer(IntegerScalarExpression),

    /// Resolve a static string value provided directly in a query.
    String(StringScalarExpression),

    /// Negate the value returned by the inner scalar expression.
    Negate(NegateScalarExpression)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceScalarExpression {
    query_location: QueryLocation,
    accessor: ValueAccessor
}

impl SourceScalarExpression {
    pub fn new(query_location: QueryLocation, accessor: ValueAccessor) -> SourceScalarExpression {
        Self { query_location, accessor }
    }

    pub fn get_selectors(&self) -> &Vec<ValueSelector> {
        self.accessor.get_selectors()
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
    accessor: ValueAccessor
}

impl AttachedScalarExpression {
    pub fn new(query_location: QueryLocation, name: &str, accessor: ValueAccessor) -> AttachedScalarExpression {
        Self { query_location, name: name.into(), accessor }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_selectors(&self) -> &Vec<ValueSelector> {
        self.accessor.get_selectors()
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
    accessor: ValueAccessor
}

impl VariableScalarExpression {
    pub fn new(query_location: QueryLocation, name: &str, accessor: ValueAccessor) -> VariableScalarExpression {
        Self { query_location, name: name.into(), accessor }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_selectors(&self) -> &Vec<ValueSelector> {
        self.accessor.get_selectors()
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
    inner_expression: Box<ScalarExpression>
}

impl NegateScalarExpression {
    pub fn new(query_location: QueryLocation, inner_expression: ScalarExpression) -> NegateScalarExpression {
        Self { query_location, inner_expression: inner_expression.into() }
    }

    pub fn get_inner_expression(&self) -> &ScalarExpression {
        &self.inner_expression
    }
}

impl Expression for NegateScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntegerScalarExpression {
    query_location: QueryLocation,
    value: i64
}

impl IntegerScalarExpression {
    pub fn new(query_location: QueryLocation, value: i64) -> IntegerScalarExpression {
        Self { query_location, value }
    }

    pub fn get_value(&self) -> i64 {
        self.value
    }
}

impl Expression for IntegerScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringScalarExpression {
    query_location: QueryLocation,
    value: Box<str>
}

impl StringScalarExpression {
    pub fn new(query_location: QueryLocation, value: &str) -> StringScalarExpression {
        Self { query_location, value: value.into() }
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

impl Expression for StringScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}