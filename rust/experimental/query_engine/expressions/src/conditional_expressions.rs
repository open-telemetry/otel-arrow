// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    DataExpression, Expression, ExpressionError, LogicalExpression, PipelineResolutionScope,
    QueryLocation,
};

/// Conditional data expression.
///
/// This is used to optionally apply nested [`DataExpression`s] to some subset of data which
/// matches a predicate.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalExpression {
    query_location: QueryLocation,

    /// branches which will conditionally process
    branches: Vec<ConditionalExpressionBranch>,

    /// if `Some`, data that does not match the condition in any of the other branches
    /// will be handled by this branch
    default_branch: Option<Vec<DataExpression>>,
}

impl ConditionalExpression {
    pub fn new(query_location: QueryLocation) -> Self {
        Self {
            query_location,
            branches: Vec::new(),
            default_branch: None,
        }
    }

    pub fn with_branch(mut self, branch: ConditionalExpressionBranch) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn with_default_branch(mut self, expressions: Vec<DataExpression>) -> Self {
        self.default_branch = Some(expressions);
        self
    }

    pub fn get_branches(&self) -> &[ConditionalExpressionBranch] {
        &self.branches
    }

    pub fn get_default_branch(&self) -> Option<&[DataExpression]> {
        self.default_branch.as_deref()
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        // TODO - not sure what folding would be here.
        // I guess if some branch had predicate static false, we could just remove it?
        Ok(())
    }
}

impl Expression for ConditionalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ConditionalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        // TODO this isn't right ....

        writeln!(f, "Conditional")?;
        if self.branches.is_empty() {
            writeln!(f, "{indent}├── Branches: []")?;
        } else {
            writeln!(f, "{indent}├── Branches:")?;
            let last_idx = self.branches.len() - 1;
            for (i, branch) in self.branches.iter().enumerate() {
                if i == last_idx {
                    writeln!(f, "{indent}│   └── {i}")?;
                    // func.fmt_with_indent(f, "│       ")?;
                } else {
                    writeln!(f, "{indent}│   ├── {i}")?;
                    // func.fmt_with_indent(f, "│   │   ")?;
                }
            }
        }

        if let Some(default_branch) = self.default_branch.as_ref() {
            if default_branch.len() == 1 {
                writeln!(f, "{indent}└── Default Branch:")?;
            } else {
                writeln!(f, "{indent}├── Default Branch:")?;
            }
            let last_idx = default_branch.len() - 1;
            for (i, expr) in default_branch.iter().enumerate() {
                if i == last_idx {
                    write!(f, "{indent}")?;
                    writeln!(f, "└───┬── {i}")?;
                    write!(f, "{indent}    └── ")?;
                    expr.fmt_with_indent(f, format!("{indent}        ").as_str())?;
                } else {
                    if i == 0 {
                        writeln!(f, "{indent}│   ├── {i}")?;
                    } else {
                        writeln!(f, "{indent}├───┬── {i}")?;
                    }
                    write!(f, "{indent}│   └── ")?;
                    expr.fmt_with_indent(f, format!("{indent}│       ").as_str())?;
                }
            }
        } else {
            writeln!(f, "{indent}└── Default Branch: None")?;
        }

        Ok(())
    }
}

/// TODO docs
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalExpressionBranch {
    query_location: QueryLocation,

    /// the condition that data must match to be handled by this branch
    condition: LogicalExpression,

    /// the expressions to apply to the data handled by this branch
    expressions: Vec<DataExpression>,
}

impl ConditionalExpressionBranch {
    pub fn new(
        query_location: QueryLocation,
        condition: LogicalExpression,
        expressions: Vec<DataExpression>,
    ) -> Self {
        Self {
            query_location,
            condition,
            expressions,
        }
    }

    pub fn get_condition(&self) -> &LogicalExpression {
        &self.condition
    }

    pub fn get_expressions(&self) -> &[DataExpression] {
        &self.expressions
    }
}

#[cfg(test)]
mod test {
    use crate::{
        EqualToLogicalExpression, IntegerScalarExpression, MoveTransformExpression,
        MutableValueExpression, PipelineExpression, ScalarExpression, SourceScalarExpression,
        StaticScalarExpression, TransformExpression, ValueAccessor,
    };

    use super::*;

    #[test]
    fn test_fmt_with_indent() {
        let expr = ConditionalExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                    )),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    false,
                )),
                vec![
                    DataExpression::Transform(TransformExpression::Move(
                        MoveTransformExpression::new(
                            QueryLocation::new_fake(),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new(),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new(),
                            )),
                        ),
                    )),
                    DataExpression::Transform(TransformExpression::Move(
                        MoveTransformExpression::new(
                            QueryLocation::new_fake(),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new(),
                            )),
                            MutableValueExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new(),
                            )),
                        ),
                    )),
                ],
            ))
            .with_default_branch(vec![
                DataExpression::Transform(TransformExpression::Move(MoveTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                ))),
                DataExpression::Transform(TransformExpression::Move(MoveTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                ))),
                DataExpression::Transform(TransformExpression::Move(MoveTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                ))),
                DataExpression::Transform(TransformExpression::Move(MoveTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                ))),
            ]);

        let mut pipeline = PipelineExpression::new("");
        pipeline.push_expression(DataExpression::Conditional(expr));
        println!("{}", pipeline);
    }
}
