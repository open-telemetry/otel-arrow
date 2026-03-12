// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineExpression {
    query: Box<str>,
    query_location: QueryLocation,
    constants: Vec<StaticScalarExpression>,
    functions: Vec<PipelineFunction>,
    initializations: Vec<PipelineInitialization>,
    expressions: Vec<DataExpression>,
}

impl PipelineExpression {
    pub(crate) fn new(query: &str) -> PipelineExpression {
        Self {
            query: query.into(),
            query_location: QueryLocation::new(0, query.len(), 1, 1).unwrap(),
            constants: Vec::new(),
            functions: Vec::new(),
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

    pub(crate) fn push_function(&mut self, value: PipelineFunction) -> usize {
        self.functions.push(value);
        self.functions.len() - 1
    }

    pub fn get_constants(&self) -> &[StaticScalarExpression] {
        &self.constants
    }

    pub fn get_constant(&self, constant_id: usize) -> Option<&StaticScalarExpression> {
        self.constants.get(constant_id)
    }

    pub fn get_functions(&self) -> &[PipelineFunction] {
        &self.functions
    }

    pub fn get_function(&self, function_id: usize) -> Option<&PipelineFunction> {
        self.functions.get(function_id)
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
            functions: &self.functions,
        }
    }

    pub(crate) fn optimize(&mut self) -> Result<(), Vec<ExpressionError>> {
        let mut errors = Vec::new();
        {
            let scope = PipelineResolutionScope {
                constants: &self.constants,
                functions: &self.functions,
            };

            for e in &mut self.expressions {
                if let Err(e) = e.try_fold(&scope) {
                    errors.push(e);
                }
            }
        }

        for i in 0..self.functions.len() {
            // There's a challenge here where `PipelineResolutionScope` needs to shared-borrow the
            // functions, but we need to mutably borrow the expressions inside the function body so
            // that they can be folded. The trick here is that we take the expressions from the
            // body temporarily.
            //
            // The main limitation of this approach is that we will not to optimize an expression
            // inside a recursive function by inspecting the function's body. However,
            // currently this kind of optimization only occurs for functions without mutable
            // parameters that have a single static return
            // (see crate::scalars::scalar_expressions::InvokeFunctionScalarExpression::try_fold`)
            // It would be very unusual to write such a function recursively, and in the off-chance
            // that such a function, which is why we can accept this limitation.

            // if the function is defined as a list of expressions, temporarily take the
            // expressions so we can fold them, while still borrowing immutably the other functions
            let mut tmp_func_exprs = {
                let func = &mut self.functions[i];
                if let PipelineFunctionImplementation::Expressions(func_exprs) =
                    &mut func.implementation
                {
                    std::mem::take(func_exprs)
                } else {
                    Vec::new()
                }
            };

            // fold the function expressions..
            {
                let scope = PipelineResolutionScope {
                    constants: &self.constants,
                    functions: &self.functions,
                };
                for e in &mut tmp_func_exprs {
                    if let Err(e) = e.try_fold(&scope) {
                        errors.push(e);
                    }
                }
            }

            // replace the function expressions from where they were taken above.
            {
                let func = &mut self.functions[i];
                if let PipelineFunctionImplementation::Expressions(func_exprs) =
                    &mut func.implementation
                {
                    *func_exprs = tmp_func_exprs;
                }
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

        if self.functions.is_empty() {
            writeln!(f, "├── Functions: []")?;
        } else {
            writeln!(f, "├── Functions:")?;
            let last_idx = self.functions.len() - 1;
            for (i, func) in self.functions.iter().enumerate() {
                if i == last_idx {
                    writeln!(f, "│   └── {i}")?;
                    func.fmt_with_indent(f, "│       ")?;
                } else {
                    writeln!(f, "│   ├── {i}")?;
                    func.fmt_with_indent(f, "│   │   ")?;
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
    functions: &'a Vec<PipelineFunction>,
}

impl<'a> PipelineResolutionScope<'a> {
    pub fn get_constant(&self, constant_id: usize) -> Option<&'a StaticScalarExpression> {
        self.constants.get(constant_id)
    }

    pub fn get_function(&self, function_id: usize) -> Option<&'a PipelineFunction> {
        self.functions.get(function_id)
    }

    /// create a new empty pipeline resolution scope. Can be used in tests for verifying
    /// folding/optimization logic
    #[cfg(test)]
    pub fn new_for_test(
        constants: &'a Vec<StaticScalarExpression>,
        functions: &'a Vec<PipelineFunction>,
    ) -> Self {
        Self {
            constants,
            functions,
        }
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

    pub fn with_functions(mut self, functions: Vec<PipelineFunction>) -> PipelineExpressionBuilder {
        for f in functions {
            self.push_function(f);
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

    pub fn push_function(&mut self, value: PipelineFunction) -> usize {
        self.pipeline.push_function(value)
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

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineFunctionExpression {
    Conditional(ConditionalDataExpression),
    Discard(DiscardDataExpression),
    Transform(TransformExpression),
    Return(ScalarExpression),
}

impl PipelineFunctionExpression {
    pub(crate) fn try_fold(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<(), ExpressionError> {
        match self {
            Self::Conditional(c) => c.try_fold(scope),
            Self::Discard(d) => d.try_fold(scope),
            Self::Transform(t) => t.try_fold(scope),
            Self::Return(_r) => Ok(()), // no need to fold return
        }
    }
}

impl PipelineFunctionExpression {
    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        match self {
            PipelineFunctionExpression::Conditional(c) => {
                write!(f, "Conditional: ")?;
                c.fmt_with_indent(f, format!("{indent}           ").as_str())
            }
            PipelineFunctionExpression::Discard(d) => {
                write!(f, "Discard: ")?;
                d.fmt_with_indent(f, format!("{indent}           ").as_str())
            }
            PipelineFunctionExpression::Transform(t) => {
                write!(f, "Transform: ")?;
                t.fmt_with_indent(f, format!("{indent}           ").as_str())
            }
            PipelineFunctionExpression::Return(s) => {
                write!(f, "Return: ")?;
                s.fmt_with_indent(f, format!("{indent}        ").as_str())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineFunction {
    query_location: QueryLocation,
    parameters: Vec<PipelineFunctionParameter>,
    return_value_type: Option<ValueType>,
    implementation: PipelineFunctionImplementation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineFunctionImplementation {
    Expressions(Vec<PipelineFunctionExpression>),
    External(Box<str>),
}

impl PipelineFunctionImplementation {
    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        match self {
            PipelineFunctionImplementation::Expressions(e) => {
                if e.is_empty() {
                    writeln!(f, "(Expressions): None")?;
                } else {
                    writeln!(f, "(Expressions):")?;
                    let last_idx = e.len() - 1;
                    for (i, p) in e.iter().enumerate() {
                        if i == last_idx {
                            write!(f, "{indent}    └── ")?;
                            p.fmt_with_indent(f, format!("{indent}        ").as_str())?;
                        } else {
                            write!(f, "{indent}    ├── ")?;
                            p.fmt_with_indent(f, format!("{indent}    │   ").as_str())?;
                        }
                    }
                }
            }
            PipelineFunctionImplementation::External(id) => {
                writeln!(f, "(External): {id}")?;
            }
        }

        Ok(())
    }
}

impl PipelineFunction {
    pub fn new_with_expressions(
        query_location: QueryLocation,
        parameters: Vec<PipelineFunctionParameter>,
        return_value_type: Option<ValueType>,
        expressions: Vec<PipelineFunctionExpression>,
    ) -> PipelineFunction {
        Self {
            query_location,
            parameters,
            return_value_type,
            implementation: PipelineFunctionImplementation::Expressions(expressions),
        }
    }

    pub fn new_external(
        name: &str,
        parameters: Vec<PipelineFunctionParameter>,
        return_value_type: Option<ValueType>,
    ) -> PipelineFunction {
        Self {
            query_location: QueryLocation::new(0, 0, 1, 1).unwrap(),
            parameters,
            return_value_type,
            implementation: PipelineFunctionImplementation::External(name.into()),
        }
    }

    pub fn get_parameters(&self) -> &[PipelineFunctionParameter] {
        &self.parameters
    }

    pub fn get_return_value_type(&self) -> Option<ValueType> {
        self.return_value_type.clone()
    }

    pub fn get_implementation(&self) -> &PipelineFunctionImplementation {
        &self.implementation
    }

    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        if self.parameters.is_empty() {
            writeln!(f, "{indent}├── Parameters: None")?;
        } else {
            writeln!(f, "{indent}├── Parameters: ")?;
            let last_idx = self.parameters.len() - 1;
            for (i, p) in self.parameters.iter().enumerate() {
                if i == last_idx {
                    p.fmt_with_indent(f, format!("{indent}│   └── ").as_str())?;
                } else {
                    p.fmt_with_indent(f, format!("{indent}│   ├── ").as_str())?;
                }
            }
        }

        match &self.return_value_type {
            Some(t) => writeln!(f, "{indent}├── ReturnType: {t}")?,
            None => writeln!(f, "{indent}├── ReturnType: None")?,
        }

        write!(f, "{indent}└── Implementation")?;
        self.implementation.fmt_with_indent(f, indent)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineFunctionParameter {
    query_location: QueryLocation,
    parameter_type: PipelineFunctionParameterType,
}

impl PipelineFunctionParameter {
    pub fn new(
        query_location: QueryLocation,
        parameter_type: PipelineFunctionParameterType,
    ) -> PipelineFunctionParameter {
        Self {
            query_location,
            parameter_type,
        }
    }

    pub fn get_parameter_type(&self) -> PipelineFunctionParameterType {
        self.parameter_type.clone()
    }

    pub fn get_value_type(&self) -> Option<ValueType> {
        match &self.parameter_type {
            PipelineFunctionParameterType::Scalar(value_type) => value_type.clone(),
            PipelineFunctionParameterType::MutableValue(value_type) => value_type.clone(),
        }
    }

    pub(crate) fn fmt_with_indent(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
    ) -> std::fmt::Result {
        write!(f, "{indent}")?;

        let value_type = match &self.parameter_type {
            PipelineFunctionParameterType::Scalar(v) => {
                write!(f, "Scalar(")?;
                v
            }
            PipelineFunctionParameterType::MutableValue(v) => {
                write!(f, "MutableValue(")?;
                v
            }
        };

        match value_type {
            Some(v) => writeln!(f, "{v:?})")?,
            None => writeln!(f, "Any)")?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineFunctionParameterType {
    Scalar(Option<ValueType>),
    MutableValue(Option<ValueType>),
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

    #[test]
    fn test_folds_function_implementation_expressions() {
        let actual = PipelineExpressionBuilder::new("")
            .with_functions(vec![PipelineFunction::new_with_expressions(
                QueryLocation::new_fake(),
                vec![],
                None,
                vec![PipelineFunctionExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        // This logical expression should be folded into a static `false`
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
            )])
            .with_expressions(vec![DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::InvokeFunction(InvokeFunctionScalarExpression::new(
                        QueryLocation::new_fake(),
                        None,
                        0,
                        Vec::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                ),
            ))])
            .build()
            .unwrap();

        let expected = PipelineExpressionBuilder::new("")
            .with_functions(vec![PipelineFunction::new_with_expressions(
                QueryLocation::new_fake(),
                vec![],
                None,
                vec![PipelineFunctionExpression::Discard(
                    DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                        // folded logical expr:
                        LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                QueryLocation::new_fake(),
                                false,
                            )),
                        )),
                    ),
                )],
            )])
            .with_expressions(vec![DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::InvokeFunction(InvokeFunctionScalarExpression::new(
                        QueryLocation::new_fake(),
                        None,
                        0,
                        Vec::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "a",
                            )),
                        )]),
                    )),
                ),
            ))])
            .build()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_folds_function_implementation_expressions_that_invoke_other_functions() {
        let actual = PipelineExpressionBuilder::new("")
            .with_functions(vec![
                PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![PipelineFunctionExpression::Return(
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        )),
                    )],
                ),
                PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![PipelineFunctionExpression::Discard(
                        DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                            // This logical expression should be folded into a static `false`
                            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                                QueryLocation::new_fake(),
                                ScalarExpression::InvokeFunction(
                                    InvokeFunctionScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        None,
                                        0,
                                        Vec::new(),
                                    ),
                                ),
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(QueryLocation::new_fake(), "b"),
                                )),
                                false,
                            )),
                        ),
                    )],
                ),
            ])
            .with_expressions(vec![DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::InvokeFunction(InvokeFunctionScalarExpression::new(
                        QueryLocation::new_fake(),
                        None,
                        1,
                        Vec::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "c",
                            )),
                        )]),
                    )),
                ),
            ))])
            .build()
            .unwrap();

        let expected = PipelineExpressionBuilder::new("")
            .with_functions(vec![
                PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![PipelineFunctionExpression::Return(
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        )),
                    )],
                ),
                PipelineFunction::new_with_expressions(
                    QueryLocation::new_fake(),
                    vec![],
                    None,
                    vec![PipelineFunctionExpression::Discard(
                        DiscardDataExpression::new(QueryLocation::new_fake()).with_predicate(
                            // folded logical expr:
                            LogicalExpression::Scalar(ScalarExpression::Static(
                                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    false,
                                )),
                            )),
                        ),
                    )],
                ),
            ])
            .with_expressions(vec![DataExpression::Transform(TransformExpression::Set(
                SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::InvokeFunction(InvokeFunctionScalarExpression::new(
                        QueryLocation::new_fake(),
                        None,
                        1,
                        Vec::new(),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "c",
                            )),
                        )]),
                    )),
                ),
            ))])
            .build()
            .unwrap();

        assert_eq!(expected, actual);
    }
}
