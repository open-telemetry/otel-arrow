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

    /// Output data expression
    Output(OutputDataExpression),
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
            DataExpression::Output(o) => o.try_fold(scope),
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
            DataExpression::Output(o) => o.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            DataExpression::Discard(_) => "DataExpression(Discard)",
            DataExpression::Summary(_) => "DataExpression(Summary)",
            DataExpression::Transform(_) => "DataExpression(Transform)",
            DataExpression::Conditional(_) => "DataExpression(Conditional)",
            DataExpression::Output(_) => "DataExpression(Output)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            DataExpression::Discard(d) => d.fmt_with_indent(f, indent),
            DataExpression::Summary(s) => s.fmt_with_indent(f, indent),
            DataExpression::Transform(t) => t.fmt_with_indent(f, indent),
            DataExpression::Conditional(c) => c.fmt_with_indent(f, indent),
            DataExpression::Output(o) => o.fmt_with_indent(f, indent),
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

    /// Branches which will conditionally process
    branches: Vec<ConditionalDataExpressionBranch>,

    /// If `Some`, data that does not match the condition in any of the other branches
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
        // TODO support folding. What this should do is:
        //
        // 1) for each branch, check if's condition can be folded into a boolean literal using
        // `LogicalExpression::try_resolve_static`. If so:
        // - if the result is false, discard the branch because it would evaluate on zero rows
        // - if the result is true, discard all the subsequent branches and the default branch
        //   because no rows would be evaluated by them
        //
        // 2) recursively call try_fold on all the expressions in every remaining branch and the
        // default branch if still present.
        //
        // Before doing this, we should support filtering by static literals so we can write unit
        // tests that will evaluate the folded expr. Without this, we won't be able to evaluate the
        // resolved static as a filter. https://github.com/open-telemetry/otel-arrow/issues/1508

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

    /// The condition that data must match to be handled by this branch
    condition: LogicalExpression,

    /// The expressions to apply to the data handled by this branch
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

/// Data expression representing an operation that emits data to a sink.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputDataExpression {
    query_location: QueryLocation,
    output: OutputExpression,
}

impl OutputDataExpression {
    pub fn new(query_location: QueryLocation, output: OutputExpression) -> Self {
        Self {
            query_location,
            output,
        }
    }

    pub fn get_output(&self) -> &OutputExpression {
        &self.output
    }

    pub fn try_fold(&mut self, _scope: &PipelineResolutionScope) -> Result<(), ExpressionError> {
        // No folding currently supported for output expressions.
        Ok(())
    }
}

impl Expression for OutputDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "OutputDataExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Output:")?;
        write!(f, "{indent}└── ")?;
        match &self.output {
            OutputExpression::NamedSink(expr) => {
                expr.fmt_with_indent(f, format!("{indent}    ").as_str())
            }
        }
    }
}

/// Expression representing an operation that emits data to a sink.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputExpression {
    /// Output data to a sink identified by name.
    // Currently this contains a static string because it's the only way we handle identifying
    // where to output the data. In the future we could support dynamic sink identified by a
    // variable, result of a function call, or other some expression, at which point we can change
    // this to contain the more general `StaticExpression`.
    NamedSink(StringScalarExpression),
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt;

    // Helper struct to test fmt_with_indent by implementing Display
    struct DisplayWrapper<'a, T: Expression>(&'a T, &'a str);

    impl<'a, T: Expression> fmt::Display for DisplayWrapper<'a, T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt_with_indent(f, self.1)
        }
    }

    #[test]
    fn test_output_expression_fmt_with_indent() {
        let string_expr = StringScalarExpression::new(QueryLocation::new_fake(), "sink_name");
        let output_expr = OutputExpression::NamedSink(string_expr.clone());
        let output_data_expr = OutputDataExpression::new(QueryLocation::new_fake(), output_expr);
        let output = format!("{}", DisplayWrapper(&output_data_expr, ""));
        assert_eq!(
            output,
            format!(
                "Output:\n\
                └── {string_expr:?}\n"
            )
        );
    }
}
