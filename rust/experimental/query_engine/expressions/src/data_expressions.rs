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
            DataExpression::Conditional(_) => {
                // TODO
                Ok(())
            }
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

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalDataExpression {
    query_location: QueryLocation,
    branches: Vec<(LogicalExpression, Vec<DataExpression>)>,
    default_branch: Option<Vec<DataExpression>>,
}

impl ConditionalDataExpression {
    pub fn get_branches(&self) -> &[(LogicalExpression, Vec<DataExpression>)] {
        &self.branches
    }

    pub fn get_default_branch(&self) -> Option<&Vec<DataExpression>> {
        self.default_branch.as_ref()
    }
}

impl Expression for ConditionalDataExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ConditionalDataExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Conditional")?;
        writeln!(f, "{indent}└── TODO ")?;
        Ok(())
    }
}

pub struct ConditionalDataExpressionBuilder {
    inner: ConditionalDataExpression,
}

impl ConditionalDataExpressionBuilder {
    pub fn from_if(condition: LogicalExpression, branch: Vec<DataExpression>) -> Self {
        let inner = ConditionalDataExpression {
            // TODO doubt fake is the correct thing to use here
            query_location: QueryLocation::new_fake(),
            branches: vec![(condition, branch)],
            default_branch: None,
        };

        Self { inner }
    }

    pub fn with_else_if(mut self, condition: LogicalExpression, branch: Vec<DataExpression>) -> Self {
        self.inner.branches.push((condition, branch));
        self
    }

    pub fn with_else(mut self, branch: Vec<DataExpression>) -> Self {
        self.inner.default_branch = Some(branch);
        self
    }

    pub fn build(self) -> ConditionalDataExpression {
        self.inner
    }
}
