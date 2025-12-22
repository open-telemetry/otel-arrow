// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum DataExpression {
    /// Discard data expression.
    Discard(DiscardDataExpression),

    /// Summary data expression.
    Summary(SummaryDataExpression),

    /// Transform data expression.
    Transform(TransformExpression),

    /// Conditional data expression.
    Conditional(ConditionalDataExpression),
}

impl DataExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            DataExpression::Discard(d) => d.try_fold(scope),
            DataExpression::Summary(s) => s.try_fold(scope),
            DataExpression::Transform(t) => t.try_fold(scope),
            DataExpression::Conditional(c) => c.try_fold(scope),
        }
    }
}

impl Expression for DataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            DataExpression::Discard(d) => d.get_query_location(),
            DataExpression::Summary(s) => s.get_query_location(),
            DataExpression::Transform(t) => t.get_query_location(),
            DataExpression::Conditional(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            DataExpression::Discard(_) => "DataExpression(Discard)",
            DataExpression::Summary(_) => "DataExpression(Summary)",
            DataExpression::Transform(_) => "DataExpression(Transform)",
            DataExpression::Conditional(_) => "DataExpression(Conditional)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            DataExpression::Discard(d) => d.fmt_with_indent(f, indent),
            DataExpression::Summary(s) => s.fmt_with_indent(f, indent),
            DataExpression::Transform(t) => t.fmt_with_indent(f, indent),
            DataExpression::Conditional(c) => c.fmt_with_indent(f, indent),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscardDataExpression {
    query_location: QueryLocation,
    predicate: Option<LogicalExpression>,
}

impl DiscardDataExpression {
    pub fn new(query_location: QueryLocation) -> DiscardDataExpression {
        Self {
            query_location,
            predicate: None,
        }
    }

    pub fn with_predicate(mut self, predicate: LogicalExpression) -> DiscardDataExpression {
        self.predicate = Some(predicate);

        self
    }

    pub fn get_predicate(&self) -> Option<&LogicalExpression> {
        self.predicate.as_ref()
    }

    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        if let Some(p) = &mut self.predicate
            && let Some(b) = p.try_resolve_static(scope)?
            && b
        {
            // Note: If predicate evaluates to static true we can clear it as
            // everything will be discarded by default.
            self.predicate = None
        }

        Ok(())
    }
}

impl Expression for DiscardDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "DiscardDataExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Discard")?;
        match self.predicate.as_ref() {
            None => writeln!(f, "{indent}└── Predicate: None")?,
            Some(p) => {
                writeln!(f, "{indent}└── Predicate:")?;
                write!(f, "{indent}    └── ")?;
                p.fmt_with_indent(f, format!("{indent}        ").as_str())?;
            }
        }
        Ok(())
    }
}

/// Conditional data expression.
///
/// This is used to define a data operation where some nested [`DataExpression`]s are applied to
/// a subset of data which matches a predicate condition. Each combination of condition/expressions
/// forms a "branch". The "default branch" defines how to optionally handle data that matches no
/// other branch's condition.
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
        _scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        // TODO support folding here.. what we should do is:
        //
        // 1) for each branch, see if's condition can be folded into a boolean literal using
        // LogicalExpression::try_resolve_static. If so:
        // - if the static is false, discard the branch because it would evaluate on zero rows
        // - if the static is true, discard all the other branches and the default branch because
        //   no rows would be evaluated by them
        //
        // 2) recursively call try_fold on all the expressions in every remaining branch and the
        // default branch if still present.
        //
        // Before doing this, we should support filtering by static literals. Otherwise, we won't
        // be able to evaluate the resolved static as a filter so we can write unit tests that will
        // evaluate the folded expr (https://github.com/open-telemetry/otel-arrow/issues/1508)

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
