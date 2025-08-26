// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TextScalarExpression {
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
            TextScalarExpression::Replace(r) => r.try_resolve_value_type(scope),
        }
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        match self {
            TextScalarExpression::Replace(r) => r.try_resolve_static(scope),
        }
    }
}

impl Expression for TextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TextScalarExpression::Replace(u) => u.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::type_complexity)]
    pub fn test_replace_string_scalar_expression_try_resolve() {
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
}
