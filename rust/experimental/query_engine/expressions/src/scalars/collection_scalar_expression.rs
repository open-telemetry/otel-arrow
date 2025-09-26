// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum CollectionScalarExpression {
    /// Returns an array containing all the elements provided in a sequence of arrays.
    Concat(CombineScalarExpression),

    /// Returns a sequence of inner scalar expressions as an array value.
    List(ListScalarExpression),
}

impl CollectionScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            CollectionScalarExpression::Concat(c) => c.try_resolve_array_value_type(scope),
            CollectionScalarExpression::List(_) => Ok(Some(ValueType::Array)),
        }
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        match self {
            CollectionScalarExpression::Concat(_) => {
                // Note: Arrays don't get folded so there isn't a static
                // resolution path for concat, it is always a runtime thing.
                Ok(None)
            }
            CollectionScalarExpression::List(c) => c.try_resolve_static(scope),
        }
    }
}

impl Expression for CollectionScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            CollectionScalarExpression::Concat(c) => c.get_query_location(),
            CollectionScalarExpression::List(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            CollectionScalarExpression::Concat(_) => "CollectionScalar(Concat)",
            CollectionScalarExpression::List(_) => "CollectionScalar(List)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            CollectionScalarExpression::Concat(c) => c.fmt_with_indent(f, indent),
            CollectionScalarExpression::List(c) => c.fmt_with_indent(f, indent),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombineScalarExpression {
    query_location: QueryLocation,
    values_expression: Box<ScalarExpression>,
}

impl CombineScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        values_expression: ScalarExpression,
    ) -> CombineScalarExpression {
        Self {
            query_location,
            values_expression: values_expression.into(),
        }
    }

    pub fn get_values_expression(&self) -> &ScalarExpression {
        &self.values_expression
    }

    pub(crate) fn try_resolve_array_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        let values = &mut self.values_expression;

        match values
            .try_resolve_static(scope)?
            .as_ref()
            .map(|v| v.to_value())
        {
            Some(Value::Array(a)) => {
                let completed = a.get_items(&mut IndexValueClosureCallback::new(|_, v| {
                    v.get_value_type() == ValueType::Array
                }));

                if completed {
                    Ok(Some(ValueType::Array))
                } else {
                    Ok(None)
                }
            }
            Some(v) => {
                let t = v.get_value_type();
                Err(ExpressionError::TypeMismatch(
                    values.get_query_location().clone(),
                    format!(
                        "Value of '{:?}' type returned by scalar expression was not an array",
                        t
                    ),
                ))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn try_resolve_string_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        let values = &mut self.values_expression;

        match values.try_resolve_value_type(scope)? {
            Some(ValueType::Array) => Ok(Some(ValueType::String)),
            Some(v) => Err(ExpressionError::TypeMismatch(
                values.get_query_location().clone(),
                format!(
                    "Value of '{:?}' type returned by scalar expression was not an array",
                    v
                ),
            )),
            None => Ok(None),
        }
    }

    pub(crate) fn try_resolve_string_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let (mut values, len) = match self
            .values_expression
            .try_resolve_static(scope)?
            .as_ref()
            .map(|v| v.as_ref())
        {
            Some(StaticScalarExpression::Array(v)) => {
                let value_expressions = v.get_values();

                let mut len = 0;
                let mut values = Vec::with_capacity(value_expressions.len());

                for expression in value_expressions {
                    match expression.try_fold() {
                        None => return Ok(None),
                        Some(e) => {
                            let s = StringScalarExpression::new(
                                expression.get_query_location().clone(),
                                e.to_value().to_string().as_str(),
                            );

                            len += s.get_value().len();

                            values.push(s);
                        }
                    }
                }

                (values, len)
            }
            Some(v) => {
                let t = v.get_value_type();
                return Err(ExpressionError::TypeMismatch(
                    self.values_expression.get_query_location().clone(),
                    format!(
                        "Value of '{:?}' type returned by scalar expression was not an array",
                        t
                    ),
                ));
            }
            None => return Ok(None),
        };

        if values.len() == 1 {
            Ok(Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::String(values.drain(0..1).next().unwrap()),
            )))
        } else {
            let mut s = String::with_capacity(len);

            for v in values {
                s.push_str(v.get_value());
            }

            Ok(Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::String(StringScalarExpression::new(
                    self.query_location.clone(),
                    s.as_str(),
                )),
            )))
        }
    }
}

impl Expression for CombineScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "CombineScalarExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        write!(f, "Concat(Scalar): ")?;
        self.values_expression
            .fmt_with_indent(f, format!("{indent}                ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListScalarExpression {
    query_location: QueryLocation,
    value_expressions: Vec<ScalarExpression>,
}

impl ListScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        value_expressions: Vec<ScalarExpression>,
    ) -> ListScalarExpression {
        Self {
            query_location,
            value_expressions,
        }
    }

    pub fn get_value_expressions(&self) -> &[ScalarExpression] {
        &self.value_expressions
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let mut values = Vec::new();

        let expressions: Vec<ScalarStaticResolutionResult> = self
            .value_expressions
            .iter_mut()
            .map(|v| v.try_resolve_static(scope))
            .collect();

        for v in expressions {
            match v?.and_then(|v| v.try_fold()) {
                Some(v) => {
                    values.push(v);
                }
                None => return Ok(None),
            }
        }

        Ok(Some(ResolvedStaticScalarExpression::Computed(
            StaticScalarExpression::Array(ArrayScalarExpression::new(
                self.query_location.clone(),
                values,
            )),
        )))
    }
}

impl Expression for ListScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ListScalarExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "List")?;
        let values = &self.value_expressions;
        if values.is_empty() {
            writeln!(f, "{indent}└── Values: []")?;
        } else {
            for (i, v) in values.iter().enumerate() {
                let last = i + 1 == values.len();
                let header = format!("Values[{i}](Scalar): ");
                if last {
                    write!(f, "{indent}└── {header}")?;
                    v.fmt_with_indent(
                        f,
                        format!("{indent}    {}", " ".repeat(header.len())).as_str(),
                    )?;
                } else {
                    write!(f, "{indent}├── {header}")?;
                    v.fmt_with_indent(
                        f,
                        format!("{indent}│   {}", " ".repeat(header.len())).as_str(),
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_resolve_value_type() {
        let run_test_success = |mut expression: ScalarExpression, expected: Option<ValueType>| {
            let pipeline: PipelineExpression = Default::default();

            let actual = expression
                .try_resolve_value_type(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, actual)
        };

        run_test_success(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(QueryLocation::new_fake(), vec![]),
            )),
            Some(ValueType::Array),
        );
    }

    #[test]
    fn test_try_resolve_static() {
        let run_test_success =
            |mut expression: ScalarExpression,
             expected_resolved: Option<StaticScalarExpression>,
             expected_state: Option<ScalarExpression>| {
                let mut pipeline = PipelineExpressionBuilder::new("")
                    .with_constants(vec![StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                    )])
                    .build()
                    .unwrap();

                pipeline.push_constant(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                ));

                let actual = expression
                    .try_resolve_static(&pipeline.get_resolution_scope())
                    .unwrap();

                assert_eq!(expected_resolved, actual.map(|v| v.as_ref().clone()));

                if let Some(s) = expected_state {
                    assert_eq!(s, expression);
                }
            };

        run_test_success(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                        )),
                    ],
                ),
            )),
            Some(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        2,
                    )),
                ],
            ))),
            None,
        );

        run_test_success(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        ScalarExpression::Constant(ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::String,
                            0,
                            ValueAccessor::new(),
                        )),
                    ],
                ),
            )),
            None,
            Some(ScalarExpression::Collection(
                CollectionScalarExpression::List(ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Source(SourceScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueAccessor::new(),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Constant(
                            CopyConstantScalarExpression::new(
                                QueryLocation::new_fake(),
                                0,
                                StaticScalarExpression::String(StringScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    "a",
                                )),
                            ),
                        )),
                    ],
                )),
            )),
        );
    }

    #[test]
    pub fn test_concat_collection_scalar_expression_try_resolve() {
        let run_test = |values: ScalarExpression, expected_type: Option<ValueType>| {
            let mut e = CombineScalarExpression::new(QueryLocation::new_fake(), values);

            let pipeline: PipelineExpression = Default::default();

            let actual_type = e
                .try_resolve_array_value_type(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(expected_type, actual_type);
        };

        run_test(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            ))),
            Some(ValueType::Array),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::Array(ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![],
                    )),
                    StaticScalarExpression::Array(ArrayScalarExpression::new(
                        QueryLocation::new_fake(),
                        vec![],
                    )),
                ],
            ))),
            Some(ValueType::Array),
        );

        run_test(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(QueryLocation::new_fake(), vec![]),
            )),
            Some(ValueType::Array),
        );

        run_test(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::Array(
                            ArrayScalarExpression::new(QueryLocation::new_fake(), vec![]),
                        )),
                        ScalarExpression::Collection(CollectionScalarExpression::List(
                            ListScalarExpression::new(QueryLocation::new_fake(), vec![]),
                        )),
                    ],
                ),
            )),
            None,
        );
    }
}
