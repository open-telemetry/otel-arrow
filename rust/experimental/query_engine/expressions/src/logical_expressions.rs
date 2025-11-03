// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalExpression {
    /// Resolve the boolean value for the logical expression using the inner
    /// scalar expression.
    ///
    /// Note: To be valid the inner expression should be a
    /// [`StaticScalarExpression::Boolean`] value or a resolved
    /// ([`ScalarExpression::Attached`], [`ScalarExpression::Source`], or
    /// [`ScalarExpression::Variable`]) value which is a boolean.
    Scalar(ScalarExpression),

    /// Returns true if two [`ScalarExpression`] are equal.
    EqualTo(EqualToLogicalExpression),

    /// Returns true if a [`ScalarExpression`] is greater than another
    /// [`ScalarExpression`].
    GreaterThan(GreaterThanLogicalExpression),

    /// Returns true if a [`ScalarExpression`] is greater than or equal to
    /// another [`ScalarExpression`].
    GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression),

    /// Returns true if the inner logical expression returns false.
    Not(NotLogicalExpression),

    /// Returns true if both of the inner logical expressions return true.
    And(AndLogicalExpression),

    /// Returns true if either of the inner logical expressions return true.
    Or(OrLogicalExpression),

    /// Returns true if the haystack contains the needle.
    Contains(ContainsLogicalExpression),

    /// Returns true if the haystack matches the pattern.
    Matches(MatchesLogicalExpression),
}

impl LogicalExpression {
    pub fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> Result<Option<bool>, ExpressionError> {
        if let Some(s) = match self {
            LogicalExpression::Scalar(s) => s.try_resolve_static(scope),
            LogicalExpression::EqualTo(e) => e.try_resolve_static(scope),
            LogicalExpression::GreaterThan(g) => g.try_resolve_static(scope),
            LogicalExpression::GreaterThanOrEqualTo(g) => g.try_resolve_static(scope),
            LogicalExpression::Not(n) => n.try_resolve_static(scope),
            LogicalExpression::And(a) => a.try_resolve_static(scope),
            LogicalExpression::Or(o) => o.try_resolve_static(scope),
            LogicalExpression::Contains(c) => c.try_resolve_static(scope),
            LogicalExpression::Matches(m) => m.try_resolve_static(scope),
        }? {
            match s {
                ResolvedStaticScalarExpression::FoldEligibleReference(r)
                | ResolvedStaticScalarExpression::Reference(r) => {
                    if let Value::Boolean(b) = r.to_value() {
                        // Note: We don't fold a static which is already a valid bool.
                        return Ok(Some(b.get_value()));
                    }
                }
                _ => {}
            }

            let value = s.to_value();

            if let Some(b) = value.convert_to_bool() {
                *self = LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.get_query_location().clone(),
                        b,
                    )),
                ));

                Ok(Some(b))
            } else {
                let t = value.get_value_type();
                Err(ExpressionError::TypeMismatch(
                    self.get_query_location().clone(),
                    format!(
                        "Value of '{:?}' type returned by logical expression could not be converted to bool",
                        t
                    ),
                ))
            }
        } else {
            Ok(None)
        }
    }

    fn fmt_binary_with_indent(
        f: &mut std::fmt::Formatter<'_>,
        indent: &str,
        name: &str,
        left: &ScalarExpression,
        right: &ScalarExpression,
    ) -> std::fmt::Result {
        writeln!(f, "{name}")?;
        write!(f, "{indent}├── Left(Scalar): ")?;
        left.fmt_with_indent(f, format!("{indent}│                 ").as_str())?;
        write!(f, "{indent}└── Right(Scalar): ")?;
        right.fmt_with_indent(f, format!("{indent}                   ").as_str())?;
        Ok(())
    }
}

impl Expression for LogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        match self {
            LogicalExpression::Scalar(s) => s.get_query_location(),
            LogicalExpression::EqualTo(e) => e.get_query_location(),
            LogicalExpression::GreaterThan(g) => g.get_query_location(),
            LogicalExpression::GreaterThanOrEqualTo(g) => g.get_query_location(),
            LogicalExpression::Not(n) => n.get_query_location(),
            LogicalExpression::And(a) => a.get_query_location(),
            LogicalExpression::Or(o) => o.get_query_location(),
            LogicalExpression::Contains(c) => c.get_query_location(),
            LogicalExpression::Matches(m) => m.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            LogicalExpression::Scalar(_) => "LogicalExpression(Scalar)",
            LogicalExpression::EqualTo(_) => "LogicalExpression(EqualTo)",
            LogicalExpression::GreaterThan(_) => "LogicalExpression(GreaterThan)",
            LogicalExpression::GreaterThanOrEqualTo(_) => "LogicalExpression(GreaterThanOrEqualTo)",
            LogicalExpression::Not(_) => "LogicalExpression(Not)",
            LogicalExpression::And(_) => "LogicalExpression(And)",
            LogicalExpression::Or(_) => "LogicalExpression(Or)",
            LogicalExpression::Contains(_) => "LogicalExpression(Contains)",
            LogicalExpression::Matches(_) => "LogicalExpression(Matches)",
        }
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            LogicalExpression::Scalar(s) => s.fmt_with_indent(f, indent),
            LogicalExpression::EqualTo(e) => {
                Self::fmt_binary_with_indent(f, indent, "EqualTo", &e.left, &e.right)
            }
            LogicalExpression::GreaterThan(g) => {
                Self::fmt_binary_with_indent(f, indent, "GreaterThan", &g.left, &g.right)
            }
            LogicalExpression::GreaterThanOrEqualTo(g) => {
                Self::fmt_binary_with_indent(f, indent, "GreaterThanOrEqualTo", &g.left, &g.right)
            }
            LogicalExpression::Not(n) => n.fmt_with_indent(f, indent),
            LogicalExpression::And(a) => a.fmt_with_indent(f, indent),
            LogicalExpression::Or(o) => o.fmt_with_indent(f, indent),
            LogicalExpression::Contains(c) => c.fmt_with_indent(f, indent),
            LogicalExpression::Matches(m) => m.fmt_with_indent(f, indent),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AndLogicalExpression {
    query_location: QueryLocation,
    left: Box<LogicalExpression>,
    right: Box<LogicalExpression>,
}

impl AndLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: LogicalExpression,
        right: LogicalExpression,
    ) -> AndLogicalExpression {
        Self {
            query_location,
            left: left.into(),
            right: right.into(),
        }
    }

    pub fn get_left(&self) -> &LogicalExpression {
        &self.left
    }

    pub fn get_right(&self) -> &LogicalExpression {
        &self.right
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let left = self.left.try_resolve_static(scope)?;
        let right = self.right.try_resolve_static(scope)?;

        Ok(match left {
            None => None,
            Some(false) => Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    self.query_location.clone(),
                    false,
                )),
            )),
            Some(true) => match right {
                None => None,
                Some(v) => Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
                        v,
                    )),
                )),
            },
        })
    }
}

impl Expression for AndLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "AndLogicalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "And")?;
        write!(f, "{indent}├── Left(Logical): ")?;
        self.left
            .fmt_with_indent(f, format!("{indent}│                  ").as_str())?;
        write!(f, "{indent}└── Right(Logical): ")?;
        self.right
            .fmt_with_indent(f, format!("{indent}                    ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrLogicalExpression {
    query_location: QueryLocation,
    left: Box<LogicalExpression>,
    right: Box<LogicalExpression>,
}

impl OrLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: LogicalExpression,
        right: LogicalExpression,
    ) -> OrLogicalExpression {
        Self {
            query_location,
            left: left.into(),
            right: right.into(),
        }
    }

    pub fn get_left(&self) -> &LogicalExpression {
        &self.left
    }

    pub fn get_right(&self) -> &LogicalExpression {
        &self.right
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let left = self.left.try_resolve_static(scope)?;
        let right = self.right.try_resolve_static(scope)?;

        Ok(match left {
            None => None,
            Some(true) => Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    self.query_location.clone(),
                    true,
                )),
            )),
            Some(false) => match right {
                None => None,
                Some(v) => Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
                        v,
                    )),
                )),
            },
        })
    }
}

impl Expression for OrLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "OrLogicalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Or")?;
        write!(f, "{indent}├── Left(Logical): ")?;
        self.left
            .fmt_with_indent(f, format!("{indent}│                  ").as_str())?;
        write!(f, "{indent}└── Right(Logical): ")?;
        self.right
            .fmt_with_indent(f, format!("{indent}                    ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EqualToLogicalExpression {
    query_location: QueryLocation,
    left: ScalarExpression,
    right: ScalarExpression,
    case_insensitive: bool,
}

impl EqualToLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: ScalarExpression,
        right: ScalarExpression,
        case_insensitive: bool,
    ) -> EqualToLogicalExpression {
        Self {
            query_location,
            left,
            right,
            case_insensitive,
        }
    }

    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    pub fn get_left(&self) -> &ScalarExpression {
        &self.left
    }

    pub fn get_right(&self) -> &ScalarExpression {
        &self.right
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let left = self.left.try_resolve_static(scope)?;
        let right = self.right.try_resolve_static(scope)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let b = Value::are_values_equal(
                    &self.query_location,
                    &l.to_value(),
                    &r.to_value(),
                    self.case_insensitive,
                )?;

                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
                        b,
                    )),
                )))
            }
            _ => Ok(None),
        }
    }
}

impl Expression for EqualToLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "EqualToLogicalExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreaterThanLogicalExpression {
    query_location: QueryLocation,
    left: ScalarExpression,
    right: ScalarExpression,
}

impl GreaterThanLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: ScalarExpression,
        right: ScalarExpression,
    ) -> GreaterThanLogicalExpression {
        Self {
            query_location,
            left,
            right,
        }
    }

    pub fn get_left(&self) -> &ScalarExpression {
        &self.left
    }

    pub fn get_right(&self) -> &ScalarExpression {
        &self.right
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let left = self.left.try_resolve_static(scope)?;
        let right = self.right.try_resolve_static(scope)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let r = Value::compare_values(&self.query_location, &l.to_value(), &r.to_value())?;

                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
                        r > 0,
                    )),
                )))
            }
            _ => Ok(None),
        }
    }
}

impl Expression for GreaterThanLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "GreaterThanLogicalExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GreaterThanOrEqualToLogicalExpression {
    query_location: QueryLocation,
    left: ScalarExpression,
    right: ScalarExpression,
}

impl GreaterThanOrEqualToLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: ScalarExpression,
        right: ScalarExpression,
    ) -> GreaterThanOrEqualToLogicalExpression {
        Self {
            query_location,
            left,
            right,
        }
    }

    pub fn get_left(&self) -> &ScalarExpression {
        &self.left
    }

    pub fn get_right(&self) -> &ScalarExpression {
        &self.right
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let left = self.left.try_resolve_static(scope)?;
        let right = self.right.try_resolve_static(scope)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let r = Value::compare_values(&self.query_location, &l.to_value(), &r.to_value())?;

                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
                        r >= 0,
                    )),
                )))
            }
            _ => Ok(None),
        }
    }
}

impl Expression for GreaterThanOrEqualToLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "GreaterThanOrEqualToLogicalExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotLogicalExpression {
    query_location: QueryLocation,
    inner_expression: Box<LogicalExpression>,
}

impl NotLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        inner_expression: LogicalExpression,
    ) -> NotLogicalExpression {
        Self {
            query_location,
            inner_expression: inner_expression.into(),
        }
    }

    pub fn get_inner_expression(&self) -> &LogicalExpression {
        &self.inner_expression
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        if let Some(v) = self.inner_expression.try_resolve_static(scope)? {
            Ok(Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    self.query_location.clone(),
                    !v,
                )),
            )))
        } else {
            Ok(None)
        }
    }
}

impl Expression for NotLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "NotLogicalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        write!(f, "Not(Logical): ")?;
        self.inner_expression
            .fmt_with_indent(f, format!("{indent}              ").as_str())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContainsLogicalExpression {
    query_location: QueryLocation,
    haystack: ScalarExpression,
    needle: ScalarExpression,
    case_insensitive: bool,
}

impl ContainsLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        haystack: ScalarExpression,
        needle: ScalarExpression,
        case_insensitive: bool,
    ) -> ContainsLogicalExpression {
        Self {
            query_location,
            haystack,
            needle,
            case_insensitive,
        }
    }

    pub fn get_case_insensitive(&self) -> bool {
        self.case_insensitive
    }

    pub fn get_haystack(&self) -> &ScalarExpression {
        &self.haystack
    }

    pub fn get_needle(&self) -> &ScalarExpression {
        &self.needle
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let query_location = &self.query_location;

        let haystack = self.haystack.try_resolve_static(scope)?;
        let needle = self.needle.try_resolve_static(scope)?;

        match (haystack, needle) {
            (Some(h), Some(n)) => {
                let r = Value::contains(
                    query_location,
                    &h.to_value(),
                    &n.to_value(),
                    self.case_insensitive,
                )?;

                Ok(Some(ResolvedStaticScalarExpression::Computed(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        query_location.clone(),
                        r,
                    )),
                )))
            }
            _ => Ok(None),
        }
    }
}

impl Expression for ContainsLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ContainsLogicalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Contains")?;
        write!(f, "{indent}├── Haystack(Scalar): ")?;
        self.haystack
            .fmt_with_indent(f, format!("{indent}│                     ").as_str())?;
        write!(f, "{indent}├── Needle(Scalar): ")?;
        self.needle
            .fmt_with_indent(f, format!("{indent}│                   ").as_str())?;
        writeln!(f, "{indent}└── CaseInsensitive: {}", self.case_insensitive)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchesLogicalExpression {
    query_location: QueryLocation,
    haystack: ScalarExpression,
    pattern: ScalarExpression,
}

impl MatchesLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        haystack: ScalarExpression,
        pattern: ScalarExpression,
    ) -> MatchesLogicalExpression {
        Self {
            query_location,
            haystack,
            pattern,
        }
    }

    pub fn get_haystack(&self) -> &ScalarExpression {
        &self.haystack
    }

    pub fn get_pattern(&self) -> &ScalarExpression {
        &self.pattern
    }

    pub(crate) fn try_resolve_static(
        &mut self,
        scope: &PipelineResolutionScope,
    ) -> ScalarStaticResolutionResult<'_> {
        let query_location = &self.query_location;

        let haystack = self.haystack.try_resolve_static(scope)?;
        let pattern =
            ResolvedStaticScalarExpression::try_resolve_static_regex(&mut self.pattern, scope)?;

        match (haystack, pattern) {
            (Some(h), Some(p)) => Ok(Some(ResolvedStaticScalarExpression::Computed(
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    query_location.clone(),
                    Value::matches(query_location, &h.to_value(), &Value::Regex(p))?,
                )),
            ))),
            _ => Ok(None),
        }
    }
}

impl Expression for MatchesLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "MatchesLogicalExpression"
    }

    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "Matches")?;
        write!(f, "{indent}├── Haystack(Scalar): ")?;
        self.haystack
            .fmt_with_indent(f, format!("{indent}│                     ").as_str())?;
        write!(f, "{indent}└── Pattern(Scalar): ")?;
        self.pattern
            .fmt_with_indent(f, format!("{indent}                     ").as_str())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;

    #[test]
    fn test_equal_to_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                true,
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -18),
                )),
                true,
            )),
            Some(false),
        );

        run_test(
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Temporal(TemporalScalarExpression::Now(
                    NowScalarExpression::new(QueryLocation::new_fake()),
                )),
                true,
            )),
            None,
        );
    }

    #[test]
    fn test_greater_than_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 19),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            Some(false),
        );

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 19),
                )),
            )),
            Some(false),
        );

        run_test(
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Temporal(TemporalScalarExpression::Now(
                    NowScalarExpression::new(QueryLocation::new_fake()),
                )),
            )),
            None,
        );
    }

    #[test]
    fn test_greater_than_or_equal_to_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 19),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 19),
                )),
            )),
            Some(false),
        );

        run_test(
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 18),
                )),
                ScalarExpression::Temporal(TemporalScalarExpression::Now(
                    NowScalarExpression::new(QueryLocation::new_fake()),
                )),
            )),
            None,
        );
    }

    #[test]
    fn test_not_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        run_test(
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Temporal(
                    TemporalScalarExpression::Now(NowScalarExpression::new(
                        QueryLocation::new_fake(),
                    )),
                )),
            )),
            None,
        );
    }

    #[test]
    fn test_and_or_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        // true || now() will evaluate to true because now() gets short-circuited
        run_test(
            LogicalExpression::Or(OrLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                LogicalExpression::Scalar(ScalarExpression::Temporal(
                    TemporalScalarExpression::Now(NowScalarExpression::new(
                        QueryLocation::new_fake(),
                    )),
                )),
            )),
            Some(true),
        );

        // flase && now() will evaluate to false because now() gets short-circuited
        run_test(
            LogicalExpression::And(AndLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                LogicalExpression::Scalar(ScalarExpression::Temporal(
                    TemporalScalarExpression::Now(NowScalarExpression::new(
                        QueryLocation::new_fake(),
                    )),
                )),
            )),
            Some(false),
        );

        // false || true evaluates to true
        run_test(
            LogicalExpression::Or(OrLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        false,
                    )),
                )),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
            Some(true),
        );

        // true && true evaluates to true
        run_test(
            LogicalExpression::And(AndLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
            Some(true),
        );
    }

    #[test]
    fn test_contains_try_resolve_static() {
        let run_test = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
                )),
                false,
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::Contains(ContainsLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Temporal(TemporalScalarExpression::Now(
                    NowScalarExpression::new(QueryLocation::new_fake()),
                )),
                false,
            )),
            None,
        );
    }

    #[test]
    fn test_matches_try_resolve_static() {
        let run_test_success = |mut input: LogicalExpression, expected: Option<bool>| {
            let pipeline: PipelineExpression = Default::default();

            let r = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap();

            assert_eq!(expected, r)
        };

        let run_test_failure = |mut input: LogicalExpression| {
            let pipeline: PipelineExpression = Default::default();

            let e = input
                .try_resolve_static(&pipeline.get_resolution_scope())
                .unwrap_err();

            assert!(matches!(e, ExpressionError::ParseError(_, _)));
        };

        run_test_success(
            LogicalExpression::Matches(MatchesLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::Regex(
                    RegexScalarExpression::new(
                        QueryLocation::new_fake(),
                        Regex::new("^hello world$").unwrap(),
                    ),
                )),
            )),
            Some(true),
        );

        run_test_success(
            LogicalExpression::Matches(MatchesLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "^hello.*$"),
                )),
            )),
            Some(true),
        );

        run_test_success(
            LogicalExpression::Matches(MatchesLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
                ScalarExpression::Temporal(TemporalScalarExpression::Now(
                    NowScalarExpression::new(QueryLocation::new_fake()),
                )),
            )),
            None,
        );

        run_test_failure(LogicalExpression::Matches(MatchesLogicalExpression::new(
            QueryLocation::new_fake(),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "\\",
            ))),
        )));
    }
}
