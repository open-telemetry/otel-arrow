// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    DataExpression, Expression, ExpressionError, LogicalExpression, PipelineResolutionScope,
    QueryLocation,
};

/// Conditional data expression.
///
/// This is used to optionally apply nested [`DataExpression`s] to some subset of data which
/// matches a predicate condition.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalDataExpression {
    query_location: QueryLocation,

    /// branches which will conditionally process
    branches: Vec<ConditionalDataExpressionBranch>,

    /// if `Some`, data that does not match the condition in any of the other branches
    /// will be handled by this branch
    default_branch: Option<Vec<DataExpression>>,
}

impl ConditionalDataExpression {
    pub fn new(query_location: QueryLocation) -> Self {
        Self {
            query_location,
            branches: Vec::new(),
            default_branch: None,
        }
    }

    pub fn with_branch(mut self, branch: ConditionalDataExpressionBranch) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn with_default_branch(mut self, expressions: Vec<DataExpression>) -> Self {
        self.default_branch = Some(expressions);
        self
    }

    pub fn get_branches(&self) -> &[ConditionalDataExpressionBranch] {
        &self.branches
    }

    pub fn get_default_branch(&self) -> Option<&[DataExpression]> {
        self.default_branch.as_deref()
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        let mut branches = Vec::with_capacity(self.branches.len());

        for branch in self.branches.drain(..) {
            // TODO - once the filtering code supports filtering by static literals, we can
            // avoid the clone here (https://github.com/open-telemetry/otel-arrow/issues/1508)
            if let Some(bool) = branch.condition.clone().try_resolve_static(scope)? {
                if bool {
                    // this branch's condition always resolves to true, so it will accept all the
                    // remaining rows and none of the other branches need to be evaluated
                    branches.push(branch);
                    self.default_branch = None;
                    break;
                } else {
                    // this branch always resolves to false, so no need to evaluate it. just skip
                    // pushing it into the result branches
                }
            } else {
                branches.push(branch)
            }
        }

        // recursively execute try_fold
        for branch in &mut branches {
            for expr in &mut branch.expressions {
                expr.try_fold(scope)?;
            }
        }

        // update self branches to the filtered, folded branches
        self.branches = branches;

        // also recursively execute try_fold on the default branch, if present
        if let Some(exprs) = self.default_branch.as_mut() {
            for expr in exprs {
                expr.try_fold(scope)?;
            }
        }

        Ok(())
    }
}

impl Expression for ConditionalDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ConditionalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Conditional:")?;
        if self.branches.is_empty() {
            writeln!(f, "{indent}├── Branches: []")?;
        } else {
            writeln!(f, "{indent}├── Branches:")?;
            let last_idx = self.branches.len() - 1;
            for (i, branch) in self.branches.iter().enumerate() {
                writeln!(f, "{indent}│   ├── Condition:")?;
                write!(f, "{indent}│   │   └── ")?;
                branch
                    .condition
                    .fmt_with_indent(f, &format!("{indent}│   │       "))?;
                if i == last_idx {
                    writeln!(f, "{indent}│   └── Expressions:")?;
                    let last_idx = branch.expressions.len() - 1;
                    for (i, expr) in branch.expressions.iter().enumerate() {
                        if i == last_idx {
                            write!(f, "{indent}│       └── ")?;
                            expr.fmt_with_indent(f, &format!("{indent}│           "))?;
                        } else {
                            write!(f, "{indent}│       ├── ")?;
                            expr.fmt_with_indent(f, &format!("{indent}│       │   "))?;
                        }
                    }
                } else {
                    writeln!(f, "{indent}│   ├── Expressions:")?;
                    let last_idx = branch.expressions.len() - 1;
                    for (i, expr) in branch.expressions.iter().enumerate() {
                        if i == last_idx {
                            write!(f, "{indent}│   │   └── ")?;
                            expr.fmt_with_indent(f, &format!("{indent}│   │       "))?;
                        } else {
                            write!(f, "{indent}│   │   ├── ")?;
                            expr.fmt_with_indent(f, &format!("{indent}│   │   │   "))?;
                        }
                    }
                }
            }
        }

        if let Some(default_branch) = self.default_branch.as_ref() {
            writeln!(f, "{indent}└── Default Branch:")?;
            let last_idx = default_branch.len() - 1;
            for (i, expr) in default_branch.iter().enumerate() {
                if i == last_idx {
                    write!(f, "{indent}    └── ")?;
                    expr.fmt_with_indent(f, &format!("{indent}        "))?
                } else {
                    write!(f, "{indent}    ├── ")?;
                    expr.fmt_with_indent(f, &format!("{indent}    │   "))?;
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
pub struct ConditionalDataExpressionBranch {
    query_location: QueryLocation,

    /// the condition that data must match to be handled by this branch
    condition: LogicalExpression,

    /// the expressions to apply to the data handled by this branch
    expressions: Vec<DataExpression>,
}

impl ConditionalDataExpressionBranch {
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
        let expr = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
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
            .with_branch(ConditionalDataExpressionBranch::new(
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
