// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TextScalarExpression {
    /// Returns a string with all occurrences of a lookup value replaced with a
    /// replacement value or null for invalid input.
    Replace(ReplaceTextScalarExpression),

    /// Returns true if regex expression matches the input.
    Match(MatchTextScalarExpression),

    /// Returns the first string value for a named capture in a regex for the input string
    Extract(ExtractTextScalarExpression),
}

impl TextScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            TextScalarExpression::Replace(r) => r.try_resolve_value_type(pipeline),
            TextScalarExpression::Match(m) => m.try_resolve_value_type(pipeline),
            TextScalarExpression::Extract(e) => e.try_resolve_value_type(pipeline),
        }
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            TextScalarExpression::Replace(r) => r.try_resolve_static(pipeline),
            TextScalarExpression::Match(m) => m.try_resolve_static(pipeline),
            TextScalarExpression::Extract(e) => e.try_resolve_static(pipeline),
        }
    }
}

impl Expression for TextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            TextScalarExpression::Replace(r) => r.get_query_location(),
            TextScalarExpression::Match(m) => m.get_query_location(),
            TextScalarExpression::Extract(e) => e.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            TextScalarExpression::Replace(_) => "TextScalar(Replace)",
            TextScalarExpression::Match(_) => "TextScalar(Match)",
            TextScalarExpression::Extract(_) => "TextScalar(Extract)",
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
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        if let Some(v) = self
            .get_haystack_expression()
            .try_resolve_value_type(pipeline)?
        {
            Ok(Some(match v {
                ValueType::String => ValueType::String,
                _ => ValueType::Null,
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let haystack_static = self
            .get_haystack_expression()
            .try_resolve_static(pipeline)?;
        let needle_static = self.get_needle_expression().try_resolve_static(pipeline)?;
        let replacement_static = self
            .get_replacement_expression()
            .try_resolve_static(pipeline)?;

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
                Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        self.query_location.clone(),
                        &result,
                    )),
                )))
            } else {
                Ok(Some(ResolvedStaticScalarExpression::Value(
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
pub struct MatchTextScalarExpression {
    query_location: QueryLocation,
    input_expression: Box<ScalarExpression>,
    pattern: Box<ScalarExpression>,
}

impl MatchTextScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        input_expression: ScalarExpression,
        pattern: ScalarExpression,
    ) -> MatchTextScalarExpression {
        Self {
            query_location,
            input_expression: input_expression.into(),
            pattern: pattern.into(),
        }
    }

    pub fn get_input_expression(&self) -> &ScalarExpression {
        &self.input_expression
    }

    pub fn get_pattern(&self) -> &ScalarExpression {
        &self.pattern
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        _pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        // Match always returns a boolean value (true if pattern matches, false otherwise)
        Ok(Some(ValueType::Boolean))
    }

    pub(crate) fn try_resolve_static(
        &self,
        _pipeline: &PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        // Match cannot be resolved statically since it depends on runtime input
        Ok(None)
    }
}

impl Expression for MatchTextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MatchTextScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractTextScalarExpression {
    query_location: QueryLocation,
    input_expression: Box<ScalarExpression>,
    pattern_expression: Box<ScalarExpression>,
    capture_name_or_index: Box<ScalarExpression>,
}

impl ExtractTextScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        input_expression: ScalarExpression,
        pattern_expression: ScalarExpression,
        capture_name_or_index: ScalarExpression,
    ) -> ExtractTextScalarExpression {
        Self {
            query_location,
            input_expression: input_expression.into(),
            pattern_expression: pattern_expression.into(),
            capture_name_or_index: capture_name_or_index.into(),
        }
    }

    pub fn get_input_expression(&self) -> &ScalarExpression {
        &self.input_expression
    }

    pub fn get_pattern_expression(&self) -> &ScalarExpression {
        &self.pattern_expression
    }

    pub fn get_capture_name_or_index(&self) -> &ScalarExpression {
        &self.capture_name_or_index
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        _pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        // Extract always returns a string value (the extracted text)
        Ok(Some(ValueType::String))
    }

    pub(crate) fn try_resolve_static(
        &self,
        _pipeline: &PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        // Extract cannot be resolved statically since it depends on runtime input
        Ok(None)
    }
}

impl Expression for ExtractTextScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ExtractTextScalarExpression"
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
                let e = ReplaceTextScalarExpression::new(
                    QueryLocation::new_fake(),
                    text,
                    lookup,
                    replacement,
                    false, // case_insensitive
                );

                let pipeline = Default::default();

                let actual_type = e.try_resolve_value_type(&pipeline).unwrap();
                assert_eq!(expected_type, actual_type);

                let actual_value = e.try_resolve_static(&pipeline).unwrap();
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
    pub fn test_match_text_scalar_expression_try_resolve() {
        fn run_test_success(
            input_expression: ScalarExpression,
            pattern: ScalarExpression,
            expected_type: Option<ValueType>,
        ) {
            let pipeline = Default::default();

            let expression = MatchTextScalarExpression::new(
                QueryLocation::new_fake(),
                input_expression,
                pattern,
            );

            let actual_type = expression.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(expected_type, actual_type);

            let actual_value = expression
                .try_resolve_static(&pipeline)
                .unwrap()
                .map(|v| v.get_value_type());
            // Match should always return None for static resolution
            assert_eq!(None, actual_value);
        }

        // Test with static input and pattern - should always return Boolean type but no static value
        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: NotifySliceRelease",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: .*",
            ))),
            Some(ValueType::Boolean),
        );

        // Test with dynamic input - should still return Boolean type
        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "message",
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: .*",
            ))),
            Some(ValueType::Boolean),
        );
    }

    #[test]
    pub fn test_extract_text_scalar_expression_try_resolve() {
        fn run_test_success(
            input_expression: ScalarExpression,
            pattern: ScalarExpression,
            capture_group: ScalarExpression,
            expected_type: Option<ValueType>,
        ) {
            let pipeline = Default::default();

            let expression = ExtractTextScalarExpression::new(
                QueryLocation::new_fake(),
                input_expression,
                pattern,
                capture_group,
            );

            let actual_type = expression.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(expected_type, actual_type);

            let actual_value = expression
                .try_resolve_static(&pipeline)
                .unwrap()
                .map(|v| v.get_value_type());
            // Extract should always return None for static resolution
            assert_eq!(None, actual_value);
        }

        // Test with static input and pattern - should always return String type but no static value
        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: NotifySliceRelease",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: (?P<resourceName>.*)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "resourceName",
            ))),
            Some(ValueType::String),
        );

        // Test with integer index instead of string
        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: NotifySliceRelease",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: (.*)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
            Some(ValueType::String),
        );

        // Test with dynamic input - should still return String type
        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "message",
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: (?P<resourceName>.*)",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "resourceName",
            ))),
            Some(ValueType::String),
        );

        // Test with dynamic pattern - should still return String type
        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "Event: NotifySliceRelease",
            ))),
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "pattern",
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "resourceName",
            ))),
            Some(ValueType::String),
        );

        // Test with both dynamic - should still return String type
        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "input",
                    )),
                )]),
            )),
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "pattern",
                    )),
                )]),
            )),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "name",
            ))),
            Some(ValueType::String),
        );
    }
}
