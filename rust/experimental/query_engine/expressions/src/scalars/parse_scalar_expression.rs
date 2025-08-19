// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseScalarExpression {
    /// Parses an inner string scalar value as JSON and returns a Map, Array,
    /// Integer, Double, or Null value, or returns an error for invalid input.
    Json(ParseJsonScalarExpression),

    /// Parses an inner string scalar value into a Regex value or returns an
    /// error for invalid input.
    Regex(ParseRegexScalarExpression),
}

impl ParseScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            ParseScalarExpression::Json(p) => p.try_resolve_value_type(pipeline),
            ParseScalarExpression::Regex(p) => p.try_resolve_value_type(pipeline),
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
            ParseScalarExpression::Json(p) => p.try_resolve_static(pipeline),
            ParseScalarExpression::Regex(p) => p.try_resolve_static(pipeline),
        }
    }
}

impl Expression for ParseScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ParseScalarExpression::Json(p) => p.get_query_location(),
            ParseScalarExpression::Regex(p) => p.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ParseScalarExpression::Json(_) => "ParseScalar(Json)",
            ParseScalarExpression::Regex(_) => "ParseScalar(Regex)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseJsonScalarExpression {
    query_location: QueryLocation,
    inner_expression: Box<ScalarExpression>,
}

impl ParseJsonScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: ScalarExpression,
    ) -> ParseJsonScalarExpression {
        Self {
            query_location,
            inner_expression: inner_expression.into(),
        }
    }

    pub fn get_inner_expression(&self) -> &ScalarExpression {
        &self.inner_expression
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        Ok(self
            .try_resolve_static(pipeline)?
            .map(|v| v.get_value_type()))
    }

    pub(crate) fn try_resolve_static(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        match self.inner_expression.try_resolve_static(pipeline)? {
            Some(v) => Ok(Some(ResolvedStaticScalarExpression::Value(
                if let Value::String(s) = v.to_value() {
                    StaticScalarExpression::from_json(self.query_location.clone(), s.get_value())?
                } else {
                    return Err(ExpressionError::ParseError(
                        self.get_query_location().clone(),
                        format!(
                            "Input of '{:?}' type could not be pased as JSON",
                            v.get_value_type()
                        ),
                    ));
                },
            ))),
            None => Ok(None),
        }
    }
}

impl Expression for ParseJsonScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ParseJsonScalarExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseRegexScalarExpression {
    query_location: QueryLocation,
    pattern: Box<ScalarExpression>,
    options: Option<Box<ScalarExpression>>,
}

impl ParseRegexScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        pattern: ScalarExpression,
        options: Option<ScalarExpression>,
    ) -> ParseRegexScalarExpression {
        Self {
            query_location,
            pattern: pattern.into(),
            options: options.map(|v| v.into()),
        }
    }

    pub fn get_pattern(&self) -> &ScalarExpression {
        &self.pattern
    }

    pub fn get_options(&self) -> Option<&ScalarExpression> {
        self.options.as_deref()
    }

    pub(crate) fn try_resolve_value_type(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ValueType>, ExpressionError> {
        Ok(self
            .try_resolve_static(pipeline)?
            .map(|v| v.get_value_type()))
    }

    pub(crate) fn try_resolve_static(
        &self,
        pipeline: &PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'_>>, ExpressionError> {
        let pattern = self.pattern.try_resolve_static(pipeline)?;

        let options = if let Some(o) = &self.options {
            match o.try_resolve_static(pipeline)? {
                Some(v) => Some(v),
                None => return Ok(None),
            }
        } else {
            None
        };

        let regex = match (pattern, options) {
            (Some(pattern), None) => {
                Value::parse_regex(&self.query_location, &pattern.to_value(), None)?
            }
            (Some(pattern), Some(options)) => Value::parse_regex(
                &self.query_location,
                &pattern.to_value(),
                Some(&options.to_value()),
            )?,
            _ => {
                return Ok(None);
            }
        };

        Ok(Some(ResolvedStaticScalarExpression::Value(
            StaticScalarExpression::Regex(RegexScalarExpression::new(
                self.query_location.clone(),
                regex,
            )),
        )))
    }
}

impl Expression for ParseRegexScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ParseRegexScalarExpression"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_json_scalar_expression_try_resolve() {
        fn run_test_success(input: &str, expected_value: Value) {
            let pipeline = Default::default();

            let expression = ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_type = expression.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(Some(expected_value.get_value_type()), actual_type);

            let actual_value = expression.try_resolve_static(&pipeline).unwrap();
            assert_eq!(
                Some(expected_value),
                actual_value.as_ref().map(|v| v.to_value())
            );
        }

        fn run_test_failure(input: &str) {
            let pipeline = Default::default();

            let expression = ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_type_error = expression.try_resolve_value_type(&pipeline).unwrap_err();
            assert!(matches!(
                actual_type_error,
                ExpressionError::ParseError(_, _)
            ));

            let actual_value_error = expression.try_resolve_static(&pipeline).unwrap_err();
            assert!(matches!(
                actual_value_error,
                ExpressionError::ParseError(_, _)
            ));
        }

        run_test_success(
            "18",
            Value::Integer(&IntegerScalarExpression::new(QueryLocation::new_fake(), 18)),
        );
        run_test_failure("hello world");
    }

    #[test]
    pub fn test_parse_regex_scalar_expression_try_resolve() {
        fn run_test_success(
            pattern: ScalarExpression,
            options: Option<ScalarExpression>,
            expected: Option<ValueType>,
        ) {
            let pipeline = Default::default();

            let expression =
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options);

            let actual_type = expression.try_resolve_value_type(&pipeline).unwrap();
            assert_eq!(expected, actual_type);

            let actual_value = expression
                .try_resolve_static(&pipeline)
                .unwrap()
                .map(|v| v.get_value_type());
            assert_eq!(expected, actual_value);
        }

        fn run_test_failure(pattern: ScalarExpression, options: Option<ScalarExpression>) {
            let pipeline = Default::default();

            let expression =
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options);

            expression.try_resolve_value_type(&pipeline).unwrap_err();

            expression.try_resolve_static(&pipeline).unwrap_err();
        }

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "[^a]ello world",
            ))),
            None,
            Some(ValueType::Regex),
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
            Some(ScalarExpression::Static(StaticScalarExpression::String(
                StringScalarExpression::new(QueryLocation::new_fake(), "i"),
            ))),
            Some(ValueType::Regex),
        );

        run_test_success(
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            )),
            None,
            None,
        );

        run_test_success(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "[^a]ello world",
            ))),
            Some(ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key1",
                    )),
                )]),
            ))),
            None,
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "(",
            ))),
            None,
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::Null(NullScalarExpression::new(
                QueryLocation::new_fake(),
            ))),
            None,
        );

        run_test_failure(
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                ".*",
            ))),
            Some(ScalarExpression::Static(StaticScalarExpression::Null(
                NullScalarExpression::new(QueryLocation::new_fake()),
            ))),
        );
    }
}
