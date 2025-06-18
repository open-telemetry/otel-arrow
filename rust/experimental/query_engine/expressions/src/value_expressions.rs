use crate::{ScalarExpression, SourceScalarExpression, VariableScalarExpression};

#[derive(Debug, Clone, PartialEq)]
pub enum MutableValueExpression {
    /// Source value.
    ///
    /// Note: Source may refer to the source itself (root) or data on the source
    /// accessed via [`crate::ValueAccessor`] selectors.
    Source(SourceScalarExpression),

    /// Variable value.
    ///
    /// Note: Variable may refer to the variable itself (root) or data on the
    /// variable accessed via [`crate::ValueAccessor`] selectors.
    Variable(VariableScalarExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmutableValueExpression {
    /// Scalar value.
    Scalar(ScalarExpression),
}
