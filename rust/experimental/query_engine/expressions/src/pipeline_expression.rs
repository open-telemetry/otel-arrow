// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    query: Box<str>,
    query_location: QueryLocation,
    constants: Vec<StaticScalarExpression>,
    initializations: Vec<PipelineInitialization>,
    expressions: Vec<DataExpression>,
}

impl PipelineExpression {
    pub(crate) fn new(query: &str) -> PipelineExpression {
        Self {
            query: query.into(),
            query_location: QueryLocation::new(0, query.len(), 1, 1).unwrap(),
            constants: Vec::new(),
            initializations: Vec::new(),
            expressions: Vec::new(),
        }
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }

    pub fn get_query_slice(&self, query_location: &QueryLocation) -> &str {
        let (start, end) = query_location.get_start_and_end_positions();

        &self.query[start..end]
    }

    pub(crate) fn push_constant(&mut self, value: StaticScalarExpression) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn get_constants(&self) -> &[StaticScalarExpression] {
        &self.constants
    }

    pub fn get_constant(&self, constant_id: usize) -> Option<&StaticScalarExpression> {
        self.constants.get(constant_id)
    }

    pub fn get_expressions(&self) -> &[DataExpression] {
        &self.expressions
    }

    pub(crate) fn push_expression(&mut self, expression: DataExpression) {
        self.expressions.push(expression);
    }

    pub(crate) fn push_global_variable(&mut self, name: &str, value: ScalarExpression) {
        self.initializations
            .push(PipelineInitialization::SetGlobalVariable {
                name: name.into(),
                value,
            });
    }

    pub fn get_initializations(&self) -> &[PipelineInitialization] {
        &self.initializations
    }

    pub fn get_resolution_scope(&self) -> PipelineResolutionScope<'_> {
        PipelineResolutionScope {
            constants: &self.constants,
        }
    }

    pub(crate) fn optimize(&mut self) -> Result<(), Vec<ExpressionError>> {
        let scope = PipelineResolutionScope {
            constants: &self.constants,
        };

        let mut errors = Vec::new();
        for e in &mut self.expressions {
            if let Err(e) = e.try_fold(&scope) {
                errors.push(e);
            }
        }
        Ok(())
    }
}

impl Default for PipelineExpression {
    fn default() -> Self {
        PipelineExpression::new("")
    }
}

impl Expression for PipelineExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "PipelineExpression"
    }
}

impl std::fmt::Display for PipelineExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pipeline")?;

        writeln!(f, "├── Query: {:?}", self.query)?;

        if self.constants.is_empty() {
            writeln!(f, "├── Constants: []")?;
        } else {
            writeln!(f, "├── Constants:")?;
            let last_idx = self.constants.len() - 1;
            for (i, c) in self.constants.iter().enumerate() {
                if i == last_idx {
                    write!(f, "│   └── {i} = ")?;
                    c.fmt_with_indent(f, "│       ")?;
                } else {
                    write!(f, "│   ├── {i} = ")?;
                    c.fmt_with_indent(f, "│   │   ")?;
                }
            }
        }

        if self.initializations.is_empty() {
            writeln!(f, "├── Initializations: []")?;
        } else {
            writeln!(f, "├── Initializations:")?;
            let last_idx = self.initializations.len() - 1;
            for (i, e) in self.initializations.iter().enumerate() {
                if i == last_idx {
                    write!(f, "│   └── ")?;
                    e.fmt_with_indent(f, "│       ")?;
                } else {
                    write!(f, "│   ├── ")?;
                    e.fmt_with_indent(f, "│   │   ")?;
                }
            }
        }

        if self.expressions.is_empty() {
            writeln!(f, "└── Expressions: []")?;
        } else {
            writeln!(f, "└── Expressions:")?;
            let last_idx = self.expressions.len() - 1;
            for (i, e) in self.expressions.iter().enumerate() {
                if i == last_idx {
                    write!(f, "    └── ")?;
                    e.fmt_with_indent(f, "        ")?;
                } else {
                    write!(f, "    ├── ")?;
                    e.fmt_with_indent(f, "    │   ")?;
                }
            }
        }

        Ok(())
    }
}

pub struct PipelineResolutionScope<'a> {
    constants: &'a Vec<StaticScalarExpression>,
}

impl<'a> PipelineResolutionScope<'a> {
    pub fn get_constant(&self, constant_id: usize) -> Option<&'a StaticScalarExpression> {
        self.constants.get(constant_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineInitialization {
    SetGlobalVariable {
        name: String,
        value: ScalarExpression,
    },
}

impl PipelineInitialization {
    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        match self {
            PipelineInitialization::SetGlobalVariable { name, value } => {
                writeln!(f, "SetGlobalVariable")?;
                writeln!(f, "{indent}├── Name: {name:?}")?;
                write!(f, "{indent}└── Value(Scalar): ")?;
                value.fmt_with_indent(f, format!("{indent}                   ").as_str())?;
            }
        }

        Ok(())
    }
}

pub struct PipelineExpressionBuilder {
    pipeline: PipelineExpression,
}

impl PipelineExpressionBuilder {
    pub fn new(query: &str) -> PipelineExpressionBuilder {
        Self {
            pipeline: PipelineExpression::new(query),
        }
    }

    pub fn with_constants(
        mut self,
        constants: Vec<StaticScalarExpression>,
    ) -> PipelineExpressionBuilder {
        for c in constants {
            self.push_constant(c);
        }

        self
    }

    pub fn with_global_variables(
        mut self,
        variables: Vec<(&str, ScalarExpression)>,
    ) -> PipelineExpressionBuilder {
        for (name, value) in variables {
            self.push_global_variable(name, value);
        }

        self
    }

    pub fn with_expressions(
        mut self,
        expressions: Vec<DataExpression>,
    ) -> PipelineExpressionBuilder {
        for expression in expressions {
            self.push_expression(expression);
        }

        self
    }

    pub fn push_constant(&mut self, value: StaticScalarExpression) -> usize {
        self.pipeline.push_constant(value)
    }

    pub fn push_global_variable(&mut self, name: &str, value: ScalarExpression) {
        self.pipeline.push_global_variable(name, value)
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.pipeline.push_expression(expression);
    }

    pub fn build(self) -> Result<PipelineExpression, Vec<ExpressionError>> {
        let mut p = self.pipeline;
        p.optimize()?;
        Ok(p)
    }
}

impl AsRef<PipelineExpression> for PipelineExpressionBuilder {
    fn as_ref(&self) -> &PipelineExpression {
        &self.pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_folding_test() {
        let actual = PipelineExpressionBuilder::new("")
            .with_constants(vec![StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            )])
            .with_expressions(vec![DataExpression::Discard(
                DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                    LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::Boolean(
                            BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                        )),
                        ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::Boolean,
                            0,
                            ValueAccessor::new(),
                        )),
                        false,
                    )),
                ),
            )])
            .build()
            .unwrap();

        let mut expected = PipelineExpression::new("");

        expected.push_constant(StaticScalarExpression::Boolean(
            BooleanScalarExpression::new(QueryLocation::new_fake(), true),
        ));

        // Note: In this test the predicate evaluates to a static true so it
        // gets elided completely.
        expected.push_expression(DataExpression::Discard(DiscardDataExpression::new(
            QueryLocation::new_fake(),
        )));

        assert_eq!(expected, actual);
    }
}
