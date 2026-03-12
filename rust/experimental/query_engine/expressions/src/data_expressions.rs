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

    /// Output data expression.
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

    /// Target to which to apply the discard expression. The intention is that this will be used
    /// if the discard expression is used in a location where the data to be discarded is not
    /// obvious from context or needs to be explicitly configured, such as if this appears inside
    /// a function implementation expression
    target: Option<MutableValueExpression>,
}

impl DiscardDataExpression {
    pub fn new(query_location: QueryLocation) -> DiscardDataExpression {
        Self {
            query_location,
            predicate: None,
            target: None,
        }
    }

    pub fn with_predicate(mut self, predicate: LogicalExpression) -> DiscardDataExpression {
        self.predicate = Some(predicate);

        self
    }

    pub fn with_target(mut self, target: MutableValueExpression) -> DiscardDataExpression {
        self.target = Some(target);

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
        match self.target.as_ref() {
            None => writeln!(f, "{indent}├── Target: None")?,
            Some(t) => {
                writeln!(f, "{indent}├── Target")?;
                write!(f, "{indent}│   └── ")?;
                t.fmt_with_indent(f, format!("{indent}│       ").as_str())?;
            }
        }
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

    /// The try_fold implementation for this type of expression will recursively call the relevant
    /// optimization methods on each branch's conditions and the expressions within each branch. It
    /// also will remove branches if it determines that they would not evaluate on any rows.
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        let mut i = 0;
        while i < self.branches.len() {
            let static_logical_expr = {
                let branch = &mut self.branches[i];
                branch.condition.try_resolve_static(scope)?
            };

            if let Some(static_logical_expr) = static_logical_expr {
                if static_logical_expr {
                    // here everything will pass the filter. That means we can drop the test of the
                    // branches because all remaining rows will be evaluated by this branch
                    _ = self.branches.split_off(i + 1);

                    // drop the default branch - no need to keep it as there will be now rows for
                    // for it to evaluate
                    self.default_branch = None;
                } else {
                    // here nothing will pass the filter, so we can basically discard this branch
                    self.branches.remove(i);
                    continue;
                }
            }

            // optimize the expressions inside the branch
            for expression in &mut self.branches[i].expressions {
                expression.try_fold(scope)?;
            }

            i += 1;
        }

        if let Some(default_branch) = self.default_branch.as_mut() {
            // optimize the expressions inside the branch
            for expression in default_branch {
                expression.try_fold(scope)?;
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
    /// Currently this contains a static string because it's the only way we handle identifying
    /// where to output the data. In the future we could support dynamic sink identified by a
    /// variable, result of a function call, or other some expression, at which point we can change
    /// this to contain the more general `StaticExpression`.
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

    #[test]
    fn test_fold_conditional_expr_removes_everything_after_all_true_condition() {
        let mut expr = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    // this should get folded into a static false
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                            )),
                            false,
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                // this should also get folded to static true, meaning this branch will get the rest of the rows
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    false,
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out",
                    )),
                ))],
            ))
            // this should get removed
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    false,
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out2",
                    )),
                ))],
            ))
            // this should also get removed
            .with_default_branch(vec![DataExpression::Output(OutputDataExpression::new(
                QueryLocation::new_fake(),
                OutputExpression::NamedSink(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "out2",
                )),
            ))]);

        let constants = Vec::new();
        let functions = Vec::new();
        let scope = PipelineResolutionScope::new_for_test(&constants, &functions);
        expr.try_fold(&scope).unwrap();

        let expected = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out",
                    )),
                ))],
            ));

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_fold_conditional_expr_removes_everything_all_true_condition_is_last() {
        // test to ensure we don't go out of bounds when trying to remove branches following
        // the branch chose condition always evaluates to always true
        let mut expr = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    // this should get folded into a static false
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                            )),
                            false,
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                // this should also get folded to static true, meaning this branch will get the rest of the rows
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    false,
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out",
                    )),
                ))],
            ))
            // this should also get removed
            .with_default_branch(vec![DataExpression::Output(OutputDataExpression::new(
                QueryLocation::new_fake(),
                OutputExpression::NamedSink(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "out2",
                )),
            ))]);

        let constants = Vec::new();
        let functions = Vec::new();
        let scope = PipelineResolutionScope::new_for_test(&constants, &functions);
        expr.try_fold(&scope).unwrap();

        let expected = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out",
                    )),
                ))],
            ));

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_fold_conditional_expr_removes_branch_for_all_false_condition() {
        let mut expr = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    // this should get folded into a static false
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                            )),
                            false,
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                // this will evaluate to all false, which means this branch should get removed
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                    )),
                    false,
                )),
                vec![DataExpression::Output(OutputDataExpression::new(
                    QueryLocation::new_fake(),
                    OutputExpression::NamedSink(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "out",
                    )),
                ))],
            ))
            // this should be kept
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "y",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    // this should get folded into a static false as well
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                            )),
                            false,
                        )),
                    ),
                )],
            ))
            // this should be kept
            .with_default_branch(vec![DataExpression::Output(OutputDataExpression::new(
                QueryLocation::new_fake(),
                OutputExpression::NamedSink(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "out2",
                )),
            ))]);

        let constants = Vec::new();
        let functions = Vec::new();
        let scope = PipelineResolutionScope::new_for_test(&constants, &functions);
        expr.try_fold(&scope).unwrap();

        let expected = ConditionalDataExpression::new(QueryLocation::new_fake())
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                    ),
                )],
            ))
            .with_branch(ConditionalDataExpressionBranch::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "y",
                            )),
                        )]),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                    false,
                )),
                vec![DataExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                    ),
                )],
            ))
            // this should be kept because
            .with_default_branch(vec![DataExpression::Output(OutputDataExpression::new(
                QueryLocation::new_fake(),
                OutputExpression::NamedSink(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "out2",
                )),
            ))]);

        assert_eq!(expr, expected);
    }

    #[test]
    fn test_format_with_indent_discard_target() {
        let discard_expr = DiscardDataExpression::new(QueryLocation::new_fake()).with_target(
            MutableValueExpression::Argument(ArgumentScalarExpression::new(
                QueryLocation::new_fake(),
                Some(ValueType::Map),
                0,
                ValueAccessor::new(),
            )),
        );
        let output = format!("{}", DisplayWrapper(&discard_expr, ""));
        assert_eq!(
            output,
            "Discard\n\
            ├── Target\n\
            │   └── Argument\n\
            │       ├── ValueType: Some(Map)\n\
            │       └── Id: 0\n\
            └── Predicate: None\n"
        );
    }

    #[test]
    fn test_format_with_indent_discard_target_no_target() {
        let discard_expr = DiscardDataExpression::new(QueryLocation::new_fake());
        let output = format!("{}", DisplayWrapper(&discard_expr, ""));
        assert_eq!(
            output,
            "Discard\n\
            ├── Target: None\n\
            └── Predicate: None\n"
        );
    }
}
