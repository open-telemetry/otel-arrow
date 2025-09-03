// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TextScalarExpression {
    /// Returns a string combining all the provided strings in a sequence.
    Concat(CombineScalarExpression),

    /// Returns a string combining all the provided strings in a sequence with a
    /// separater added between each element.
    Join(JoinTextScalarExpression),

    /// Returns a string with all occurrences of a lookup value replaced with a
    /// replacement value or null for invalid input.
    Replace(ReplaceTextScalarExpression),
}

impl TextScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            TextScalarExpression::Concat(c) => c.try_resolve_string_value_type(scope),
            TextScalarExpression::Join(j) => j.try_resolve_value_type(scope),
            TextScalarExpression::Replace(r) => r.try_resolve_value_type(scope),
        }
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        match self {
            TextScalarExpression::Concat(c) => c.try_resolve_string_static(scope),
            TextScalarExpression::Join(j) => j.try_resolve_static(scope),
            TextScalarExpression::Replace(r) => r.try_resolve_static(scope),
        }
    }
}

impl Expression for TextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TextScalarExpression::Concat(c) => c.get_query_location(),
            TextScalarExpression::Join(j) => j.get_query_location(),
            TextScalarExpression::Replace(u) => u.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TextScalarExpression::Concat(_) => "TextScalar(Concat)",
            TextScalarExpression::Join(_) => "TextScalar(Join)",
            TextScalarExpression::Replace(_) => "TextScalar(Replace)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceTextScalarExpression {
    query_location: QueryLocation,
    haystack_expression: Box<ScalarExpression>,
    needle_expression: Box<ScalarExpression>,
    replacement_expression: Box<ScalarExpression>,
    case_insensitive: bool,
}

impl ReplaceTextScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        haystack_expression: ScalarExpression,
        needle_expression: ScalarExpression,
        replacement_expression: ScalarExpression,
        case_insensitive: bool,
    ) -> ReplaceTextScalarExpression {
        Self {
            query_location,
            haystack_expression: haystack_expression.into(),
            needle_expression: needle_expression.into(),
            replacement_expression: replacement_expression.into(),
            case_insensitive,
        }
    }

    pub fn get_haystack_expression(&self) -> &ScalarExpression {
        &self.haystack_expression
    }

    pub fn get_needle_expression(&self) -> &ScalarExpression {
        &self.needle_expression
    }

    pub fn get_replacement_expression(&self) -> &ScalarExpression {
        &self.replacement_expression
    }

    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    pub(crate) fn try_resolve_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        let haystack_type = self.haystack_expression.try_resolve_value_type(scope)?;

        let needle_type = self.needle_expression.try_resolve_value_type(scope)?;

        let replacement_type = self.replacement_expression.try_resolve_value_type(scope)?;

        Ok(match (haystack_type, needle_type, replacement_type) {
            (Some(ValueType::String), Some(ValueType::String), Some(ValueType::String)) => {
                Some(ValueType::String)
            }
            (Some(ValueType::String), Some(ValueType::Regex), Some(ValueType::String)) => {
                Some(ValueType::String)
            }
            (Some(_), Some(_), Some(_)) => Some(ValueType::Null),
            _ => None,
        })
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        let haystack_static = self.haystack_expression.try_resolve_static(scope)?;
        let needle_static = self.needle_expression.try_resolve_static(scope)?;
        let replacement_static = self.replacement_expression.try_resolve_static(scope)?;

        if let (Some(haystack), Some(needle), Some(replacement)) =
            (haystack_static, needle_static, replacement_static)
        {
            let haystack_value = haystack.to_value();
            let needle_value = needle.to_value();
            let replacement_value = replacement.to_value();

            if let Some(result) = Value::replace_matches(
                &haystack_value,
                &needle_value,
                &replacement_value,
                self.case_insensitive,
            ) {
                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        self.query_location.clone(),
                        &result,
                    )),
                )))
            } else {
                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Null(NullScalarExpression::new(
                        self.query_location.clone(),
                    )),
                )))
            }
        } else {
            Ok(None)
        }
    }
}

impl Expression for ReplaceTextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ReplaceTextScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinTextScalarExpression {
    query_location: QueryLocation,
    separator_expression: Box<ScalarExpression>,
    values_expression: Box<ScalarExpression>,
}

impl JoinTextScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        separator_expression: ScalarExpression,
        values_expression: ScalarExpression,
    ) -> JoinTextScalarExpression {
        Self {
            query_location,
            separator_expression: separator_expression.into(),
            values_expression: values_expression.into(),
        }
    }

    pub fn get_separator_expression(&self) -> &ScalarExpression {
        &self.separator_expression
    }

    pub fn get_values_expression(&self) -> &ScalarExpression {
        &self.values_expression
    }

    pub(crate) fn try_resolve_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if self
            .separator_expression
            .try_resolve_value_type(scope)?
            .is_none()
        {
            return Ok(None);
        }

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

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        let separator = match self.separator_expression.try_resolve_static(scope)? {
            None => return Ok(None),
            Some(s) => {
                let value = s.to_value().to_string();

                StringScalarExpression::new(
                    self.separator_expression.get_query_location().clone(),
                    value.as_str(),
                )
            }
        };

        let (mut values, len) = match self.values_expression.try_resolve_static(scope)? {
            Some(ResolvedStaticScalarExpression::Computed(StaticScalarExpression::Array(v))) => {
                let value_expressions = v.get_values();

                let mut len = 0;
                let mut values = Vec::with_capacity(value_expressions.len());

                for expression in value_expressions {
                    let s = StringScalarExpression::new(
                        expression.get_query_location().clone(),
                        expression.to_value().to_string().as_str(),
                    );

                    len += s.get_value().len();

                    if !values.is_empty() {
                        len += separator.get_value().len();
                        values.push(separator.clone());
                    }

                    values.push(s);
                }

                (values, len)
            }
            Some(ResolvedStaticScalarExpression::Reference(StaticScalarExpression::Array(v)))
            | Some(ResolvedStaticScalarExpression::FoldEligibleReference(
                StaticScalarExpression::Array(v),
            )) => {
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

                            if !values.is_empty() {
                                len += separator.get_value().len();
                                values.push(separator.clone());
                            }

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

impl Expression for JoinTextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "JoinTextScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::type_complexity)]
    pub fn test_replace_text_scalar_expression_try_resolve() {
        fn run_test(
            input: Vec<(
                ScalarExpression,
                ScalarExpression,
                ScalarExpression,
                Option<ValueType>,
                Option<Value>,
            )>,
        ) {
            for (text, lookup, replacement, expected_type, expected_value) in input {
                let mut e = ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    text,
                    lookup,
                    replacement,
                    false, // case_insensitive
                );

                let pipeline: PipelineExpression = Default::default();

                let actual_type = e
                    .try_resolve_value_type(&pipeline.get_resolution_scope())
                    .unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e
                    .try_resolve_static(&pipeline.get_resolution_scope())
                    .unwrap();
                assert_eq!(expected_value, actual_value.as_ref().map(|v| v.to_value()));
            }
        }

        run_test(vec![
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "A magic trick can turn a cat into a dog",
                    ),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "cat"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hamster"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "A magic trick can turn a hamster into a dog",
                ))),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world hello"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hi"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hi world hi",
                ))),
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "no matches here"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "xyz"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "abc"),
                )),
                Some(ValueType::String),
                Some(Value::String(&StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "no matches here",
                ))),
            ),
            (
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                        StaticScalarExpression::String(StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            "key1",
                        )),
                    )]),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "search"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "replace"),
                )),
                None,
                None,
            ),
            (
                ScalarExpression::Static(StaticScalarExpression::Boolean(
                    BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "search"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "replace"),
                )),
                Some(ValueType::Null),
                Some(Value::Null),
            ),
        ]);
    }

    #[test]
    pub fn test_concat_text_scalar_expression_try_resolve() {
        let run_test = |values: ScalarExpression,
                        expected_type: Option<ValueType>,
                        expected_value: Option<&'static str>| {
            let mut e = CombineScalarExpression::new(QueryLocation::new_fake(), values);

            let pipeline: PipelineExpression = Default::default();

            let actual_type = e
                .try_resolve_string_value_type(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(expected_type, actual_type);

            let actual_value = e
                .try_resolve_string_static(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(
                expected_value.map(|v| v.into()),
                actual_value.as_ref().map(|v| v.to_value().to_string())
            );
        };

        run_test(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            ))),
            Some(ValueType::String),
            Some(""),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                ],
            ))),
            Some(ValueType::String),
            Some("helloworld"),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        18,
                    )),
                ],
            ))),
            Some(ValueType::String),
            Some("helloworld18"),
        );

        run_test(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(QueryLocation::new_fake(), vec![]),
            )),
            Some(ValueType::String),
            Some(""),
        );

        run_test(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                        )),
                    ],
                ),
            )),
            Some(ValueType::String),
            Some("hello18"),
        );

        run_test(
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        )),
                        ScalarExpression::Temporal(TemporalScalarExpression::Now(
                            NowScalarExpression::new(QueryLocation::new_fake()),
                        )),
                    ],
                ),
            )),
            Some(ValueType::String),
            None,
        );

        run_test(
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "some_var"),
                ValueAccessor::new(),
            )),
            None,
            None,
        );
    }

    #[test]
    pub fn test_join_text_scalar_expression_try_resolve() {
        let run_test = |separator: ScalarExpression,
                        values: ScalarExpression,
                        expected_type: Option<ValueType>,
                        expected_value: Option<&'static str>| {
            let mut e = JoinTextScalarExpression::new(QueryLocation::new_fake(), separator, values);

            let pipeline: PipelineExpression = Default::default();

            let actual_type = e
                .try_resolve_value_type(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(expected_type, actual_type);

            let actual_value = e
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(
                expected_value.map(|v| v.into()),
                actual_value.as_ref().map(|v| v.to_value().to_string())
            );
        };

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            ))),
            Some(ValueType::String),
            Some(""),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                ],
            ))),
            Some(ValueType::String),
            Some("hello|world"),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                ", ",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Array(ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello",
                    )),
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "world",
                    )),
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        18,
                    )),
                ],
            ))),
            Some(ValueType::String),
            Some("hello, world, 18"),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(QueryLocation::new_fake(), vec![]),
            )),
            Some(ValueType::String),
            Some(""),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                        )),
                    ],
                ),
            )),
            Some(ValueType::String),
            Some("hello|18"),
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Collection(CollectionScalarExpression::List(
                ListScalarExpression::new(
                    QueryLocation::new_fake(),
                    vec![
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                        )),
                        ScalarExpression::Temporal(TemporalScalarExpression::Now(
                            NowScalarExpression::new(QueryLocation::new_fake()),
                        )),
                    ],
                ),
            )),
            Some(ValueType::String),
            None,
        );

        run_test(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "|",
            ))),
            ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                StringScalarExpression::new(QueryLocation::new_fake(), "some_var"),
                ValueAccessor::new(),
            )),
            None,
            None,
        );
    }
}
