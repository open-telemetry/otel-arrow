// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{iter::Peekable, str::CharIndices};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseScalarExpression {
    /// Parses an inner string scalar value as JSON and returns a Map, Array,
    /// Integer, Double, or Null value, or returns an error for invalid input.
    Json(ParseJsonScalarExpression),

    /// Parses an inner string scalar value as a JSONPath and returns an array
    /// representing the selection path or returns an error for invalid input.
    JsonPath(ParseJsonPathScalarExpression),

    /// Parses an inner string scalar value into a Regex value or returns an
    /// error for invalid input.
    Regex(ParseRegexScalarExpression),
}

impl ParseScalarExpression {
    pub(crate) fn try_resolve_value_type(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        match self {
            ParseScalarExpression::Json(p) => p.try_resolve_value_type(scope),
            ParseScalarExpression::JsonPath(_) => Ok(Some(ValueType::Array)),
            ParseScalarExpression::Regex(_) => Ok(Some(ValueType::Regex)),
        }
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        match self {
            ParseScalarExpression::Json(p) => p.try_resolve_static(scope),
            ParseScalarExpression::JsonPath(p) => p.try_resolve_static(scope),
            ParseScalarExpression::Regex(p) => p.try_resolve_static(scope),
        }
    }
}

impl Expression for ParseScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            ParseScalarExpression::Json(p) => p.get_query_location(),
            ParseScalarExpression::JsonPath(p) => p.get_query_location(),
            ParseScalarExpression::Regex(p) => p.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            ParseScalarExpression::Json(_) => "ParseScalar(Json)",
            ParseScalarExpression::JsonPath(_) => "ParseScalar(JsonPath)",
            ParseScalarExpression::Regex(_) => "ParseScalar(Regex)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            ParseScalarExpression::Json(p) => p.fmt_with_indent(f, indent),
            ParseScalarExpression::JsonPath(p) => p.fmt_with_indent(f, indent),
            ParseScalarExpression::Regex(p) => p.fmt_with_indent(f, indent),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedSelector {
    Key(String),
    Index(i64),
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
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ExpressionError> {
        Ok(self.try_resolve_static(scope)?.map(|v| v.get_value_type()))
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let query_location = self.inner_expression.get_query_location().clone();

        match self.inner_expression.try_resolve_static(scope)? {
            Some(v) => Ok(Some(ResolvedStaticScalarExpression::Computed(
                if let Value::String(s) = v.to_value() {
                    StaticScalarExpression::from_json(query_location, s.get_value())?
                } else {
                    let t = v.get_value_type();
                    return Err(ExpressionError::ParseError(
                        query_location,
                        format!("Input of '{:?}' type could not be pased as JSON", t),
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

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        let header = "ParseJson(Scalar): ";
        write!(f, "{header}")?;
        self.inner_expression
            .fmt_with_indent(f, format!("{indent}{}", " ".repeat(header.len())).as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseJsonPathScalarExpression {
    query_location: QueryLocation,
    inner_expression: Box<ScalarExpression>,
}

impl ParseJsonPathScalarExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: ScalarExpression,
    ) -> ParseJsonPathScalarExpression {
        Self {
            query_location,
            inner_expression: inner_expression.into(),
        }
    }

    pub fn get_inner_expression(&self) -> &ScalarExpression {
        &self.inner_expression
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let query_location = self.inner_expression.get_query_location().clone();

        match self.inner_expression.try_resolve_static(scope)? {
            Some(s) => match s.as_ref().to_value() {
                Value::String(s) => {
                    let mut values = Self::parse_json_path(&query_location, s.get_value())?;

                    Ok(Some(ResolvedStaticScalarExpression::Computed(
                        StaticScalarExpression::Array(ArrayScalarExpression::new(
                            self.query_location.clone(),
                            values
                                .drain(..)
                                .map(|v| match v {
                                    ParsedSelector::Index(i) => StaticScalarExpression::Integer(
                                        IntegerScalarExpression::new(QueryLocation::new_fake(), i),
                                    ),
                                    ParsedSelector::Key(key) => {
                                        StaticScalarExpression::String(StringScalarExpression::new(
                                            QueryLocation::new_fake(),
                                            key.as_str(),
                                        ))
                                    }
                                })
                                .collect(),
                        )),
                    )))
                }
                v => {
                    let t = v.get_value_type();
                    Err(ExpressionError::ParseError(
                        query_location.clone(),
                        format!("Input of '{:?}' type could not be pased as JSONPath", t),
                    ))
                }
            },
            None => Ok(None),
        }
    }

    pub fn parse_json_path(
        json_path_location: &QueryLocation,
        json_path: &str,
    ) -> Result<Vec<ParsedSelector>, ExpressionError> {
        // Note: Currently only a narrow subset of JSONPath is supported, based
        // on KQL requirements. See:
        // https://learn.microsoft.com/kusto/query/jsonpath

        fn parse_json_path_content(
            json_path_location: &QueryLocation,
            path_value: &mut Peekable<CharIndices>,
            in_brace: bool,
            first_char: Option<char>,
            quote_char: Option<char>,
        ) -> Result<String, ExpressionError> {
            let mut content = String::new();
            if let Some(c) = first_char {
                content.push(c);
            }

            let mut previous_char_was_escape = false;
            while let Some(c) = path_value.peek() {
                if in_brace {
                    if quote_char.is_some() {
                        if !previous_char_was_escape && c.1 == '\\' {
                            previous_char_was_escape = true;
                            path_value.next();
                            continue;
                        }

                        let c = if !previous_char_was_escape {
                            if Some(c.1) == quote_char {
                                let position = c.0;

                                path_value.next();
                                match path_value.next() {
                                    Some((_, ']')) => break,
                                    Some(c) => {
                                        return Err(ExpressionError::ParseError(
                                            json_path_location.clone(),
                                            format!(
                                                "JsonPath invalid character at position '{}'",
                                                c.0
                                            ),
                                        ));
                                    }
                                    None => {
                                        return Err(ExpressionError::ParseError(
                                            json_path_location.clone(),
                                            format!(
                                                "JsonPath unexpectedly ended at position '{}'",
                                                position
                                            ),
                                        ));
                                    }
                                }
                            }

                            c.1
                        } else {
                            previous_char_was_escape = false;
                            match c.1 {
                                '\'' => '\'',
                                '"' => '"',
                                '\\' => '\\',
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                _ => {
                                    return Err(ExpressionError::ParseError(
                                        json_path_location.clone(),
                                        format!(
                                            "JsonPath invalid escape sequence at position '{}'",
                                            c.0
                                        ),
                                    ));
                                }
                            }
                        };

                        content.push(c);
                        path_value.next();
                        continue;
                    }

                    if c.1 == ']' {
                        path_value.next();
                        break;
                    }

                    content.push(c.1);
                    path_value.next();
                } else {
                    if c.1 == '.' || c.1 == '[' {
                        break;
                    }
                    content.push(c.1);
                    path_value.next();
                }
            }

            Ok(content)
        }

        let mut selectors = Vec::new();

        let mut path_value = json_path.char_indices().peekable();

        if path_value.next() != Some((0, '$')) {
            return Err(ExpressionError::ParseError(
                json_path_location.clone(),
                "JsonPath must start with '$'".into(),
            ));
        }

        while let Some(c) = path_value.next() {
            let content = if c.1 == '.' {
                (
                    false,
                    parse_json_path_content(
                        json_path_location,
                        &mut path_value,
                        false,
                        None,
                        None,
                    )?,
                )
            } else if c.1 == '[' {
                match path_value.next() {
                    None => {
                        return Err(ExpressionError::ParseError(
                            json_path_location.clone(),
                            format!("JsonPath unexpectedly ended at position '{}'", c.0),
                        ));
                    }
                    Some((_, '\'')) => (
                        false,
                        parse_json_path_content(
                            json_path_location,
                            &mut path_value,
                            true,
                            None,
                            Some('\''),
                        )?,
                    ),
                    Some((_, '"')) => (
                        false,
                        parse_json_path_content(
                            json_path_location,
                            &mut path_value,
                            true,
                            None,
                            Some('"'),
                        )?,
                    ),
                    Some((_, v)) => (
                        true,
                        parse_json_path_content(
                            json_path_location,
                            &mut path_value,
                            true,
                            Some(v),
                            None,
                        )?,
                    ),
                }
            } else {
                return Err(ExpressionError::ParseError(
                    json_path_location.clone(),
                    format!("JsonPath unexpectedly ended at position '{}'", c.0),
                ));
            };

            if content.1.is_empty() {
                return Err(ExpressionError::ParseError(
                    json_path_location.clone(),
                    format!("JsonPath empty key specified at position '{}'", c.0),
                ));
            }

            let value = content.1;
            if content.0 {
                match value.parse::<i64>() {
                    Ok(i) => {
                        selectors.push(ParsedSelector::Index(i));
                    }
                    _ => {
                        return Err(ExpressionError::ParseError(
                            json_path_location.clone(),
                            format!(
                                "JsonPath index specified at position '{}' could not be parsed",
                                c.0
                            ),
                        ));
                    }
                }
            } else {
                selectors.push(ParsedSelector::Key(value));
            }
        }

        Ok(selectors)
    }
}

impl Expression for ParseJsonPathScalarExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ParseJsonPathScalarExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        let header = "ParseJsonPath(Scalar): ";
        write!(f, "{header}")?;
        self.inner_expression
            .fmt_with_indent(f, format!("{indent}{}", " ".repeat(header.len())).as_str())
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

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let pattern = self.pattern.try_resolve_static(scope)?;

        let options = if let Some(o) = &mut self.options {
            match o.try_resolve_static(scope)? {
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

        Ok(Some(ResolvedStaticScalarExpression::Computed(
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

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "ParseRegex")?;
        write!(f, "{indent}├── Pattern(Scalar): ")?;
        self.pattern
            .fmt_with_indent(f, format!("{indent}│                    ").as_str())?;
        if let Some(o) = &self.options {
            write!(f, "{indent}└── Options(Scalar): ")?;
            o.fmt_with_indent(f, format!("{indent}                     ").as_str())?;
        } else {
            writeln!(f, "{indent}└── Options: None")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_json_scalar_expression_try_resolve() {
        fn run_test_success(input: &str, expected_value: Value) {
            let pipeline: PipelineExpression = Default::default();

            let mut expression = ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_type = expression
                .try_resolve_value_type(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(Some(expected_value.get_value_type()), actual_type);

            let actual_value = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(
                Some(expected_value),
                actual_value.as_ref().map(|v| v.to_value())
            );
        }

        fn run_test_failure(input: &str) {
            let pipeline: PipelineExpression = Default::default();

            let mut expression = ParseJsonScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_type_error = expression
                .try_resolve_value_type(&pipeline.get_resolution_scope())
                .unwrap_err();
            assert!(matches!(
                actual_type_error,
                ExpressionError::ParseError(_, _)
            ));

            let actual_value_error = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap_err();
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
            let pipeline: PipelineExpression = Default::default();

            let mut expression =
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options);

            let actual_value = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap()
                .map(|v| v.get_value_type());
            assert_eq!(expected, actual_value);
        }

        fn run_test_failure(pattern: ScalarExpression, options: Option<ScalarExpression>) {
            let pipeline: PipelineExpression = Default::default();

            let mut expression =
                ParseRegexScalarExpression::new(QueryLocation::new_fake(), pattern, options);

            expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap_err();
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

    #[test]
    pub fn test_parse_json_path_try_resolve() {
        fn run_test_success(input: &str, expected_value: Value) {
            let pipeline: PipelineExpression = Default::default();

            let mut expression = ParseJsonPathScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_value = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();
            assert_eq!(
                Some(expected_value),
                actual_value.as_ref().map(|v| v.to_value())
            );
        }

        fn run_test_failure(input: &str) {
            let pipeline: PipelineExpression = Default::default();

            let mut expression = ParseJsonPathScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), input),
                )),
            );

            let actual_value_error = expression
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap_err();
            assert!(matches!(
                actual_value_error,
                ExpressionError::ParseError(_, _)
            ));
        }

        run_test_success(
            "$",
            Value::Array(&ArrayScalarExpression::new(
                QueryLocation::new_fake(),
                vec![],
            )),
        );

        run_test_failure("");
    }

    #[test]
    fn test_parse_json_path() {
        let run_test_success = |input: &str, expected: Vec<ParsedSelector>| {
            println!("Testing: {input}");

            let actual =
                ParseJsonPathScalarExpression::parse_json_path(&QueryLocation::new_fake(), input)
                    .unwrap();

            assert_eq!(expected, actual);
        };

        let run_test_failure = |input: &str, expected: &str| {
            println!("Testing: {input}");

            let actual =
                ParseJsonPathScalarExpression::parse_json_path(&QueryLocation::new_fake(), input)
                    .unwrap_err();

            if let ExpressionError::ParseError(_, msg) = actual {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected ParseError");
            }
        };

        run_test_failure("", "JsonPath must start with '$'");

        run_test_failure("key1", "JsonPath must start with '$'");

        run_test_success("$", vec![]);

        run_test_success("$.key1", vec![ParsedSelector::Key("key1".into())]);

        run_test_failure("$.", "JsonPath empty key specified at position '1'");

        run_test_failure("$.key1.", "JsonPath empty key specified at position '6'");

        run_test_success(
            "$.key1.key2",
            vec![
                ParsedSelector::Key("key1".into()),
                ParsedSelector::Key("key2".into()),
            ],
        );

        run_test_success("$['key1']", vec![ParsedSelector::Key("key1".into())]);

        run_test_success("$['key\\'1']", vec![ParsedSelector::Key("key'1".into())]);

        run_test_success(
            "$[\"key\\\"1\"]",
            vec![ParsedSelector::Key("key\"1".into())],
        );

        run_test_success(
            "$['key ] \" value']",
            vec![ParsedSelector::Key("key ] \" value".into())],
        );

        run_test_success(
            "$.key1[\"key2\"]",
            vec![
                ParsedSelector::Key("key1".into()),
                ParsedSelector::Key("key2".into()),
            ],
        );

        run_test_success("$[0]", vec![ParsedSelector::Index(0)]);

        run_test_success("$[-1]", vec![ParsedSelector::Index(-1)]);

        run_test_success(
            "$.key1[0]",
            vec![ParsedSelector::Key("key1".into()), ParsedSelector::Index(0)],
        );

        run_test_failure("$[", "JsonPath unexpectedly ended at position '1'");

        run_test_failure("$['key1'", "JsonPath unexpectedly ended at position '7'");

        run_test_failure("$[\"key1\"", "JsonPath unexpectedly ended at position '7'");

        run_test_failure(
            "$['key1'.key2",
            "JsonPath invalid character at position '8'",
        );

        run_test_failure(
            "$[0&]",
            "JsonPath index specified at position '1' could not be parsed",
        );
    }
}
