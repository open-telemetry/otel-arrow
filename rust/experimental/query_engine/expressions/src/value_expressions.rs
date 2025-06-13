use crate::{ScalarExpression, SourceScalarExpression, VariableScalarExpression};

#[derive(Debug, Clone, PartialEq)]
pub enum MutableValueExpression {
    Source(SourceScalarExpression),

    Variable(VariableScalarExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImmutableValueExpression {
    Scalar(ScalarExpression),
}
