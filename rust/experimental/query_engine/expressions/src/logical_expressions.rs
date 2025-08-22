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

    /// Returns the result of a sequence of logical expressions chained using
    /// logical `AND(&&)` and/or `OR(||)` operations.
    Chain(ChainLogicalExpression),

    /// Returns true if the haystack contains the needle.
    Contains(ContainsLogicalExpression),
}

impl LogicalExpression {
    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            LogicalExpression::Scalar(s) => s.try_resolve_static(pipeline),
            LogicalExpression::EqualTo(e) => e.try_resolve_static(pipeline),
            LogicalExpression::GreaterThan(g) => g.try_resolve_static(pipeline),
            LogicalExpression::GreaterThanOrEqualTo(g) => g.try_resolve_static(pipeline),
            LogicalExpression::Not(n) => n.try_resolve_static(pipeline),
            LogicalExpression::Chain(c) => c.try_resolve_static(pipeline),
            LogicalExpression::Contains(c) => c.try_resolve_static(pipeline),
        }
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
            LogicalExpression::Chain(c) => c.get_query_location(),
            LogicalExpression::Contains(c) => c.get_query_location(),
        }
    }

    fn get_name(&self) -> &'static str {
        match self {
            LogicalExpression::Scalar(_) => "LogicalExpression(Scalar)",
            LogicalExpression::EqualTo(_) => "LogicalExpression(EqualTo)",
            LogicalExpression::GreaterThan(_) => "LogicalExpression(GreaterThan)",
            LogicalExpression::GreaterThanOrEqualTo(_) => "LogicalExpression(GreaterThanOrEqualTo)",
            LogicalExpression::Not(_) => "LogicalExpression(Not)",
            LogicalExpression::Chain(_) => "LogicalExpression(Chain)",
            LogicalExpression::Contains(_) => "LogicalExpression(Contains)",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainLogicalExpression {
    query_location: QueryLocation,
    first_expression: Box<LogicalExpression>,
    chain_expressions: Vec<ChainedLogicalExpression>,
}

impl ChainLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        first_expression: LogicalExpression,
    ) -> ChainLogicalExpression {
        Self {
            query_location,
            first_expression: first_expression.into(),
            chain_expressions: Vec::new(),
        }
    }

    pub fn push_or(&mut self, expression: LogicalExpression) {
        self.chain_expressions
            .push(ChainedLogicalExpression::Or(expression));
    }

    pub fn push_and(&mut self, expression: LogicalExpression) {
        self.chain_expressions
            .push(ChainedLogicalExpression::And(expression));
    }

    pub fn get_expressions(&self) -> (&LogicalExpression, &[ChainedLogicalExpression]) {
        (&self.first_expression, &self.chain_expressions)
    }

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        if let Some(b) = try_resolve_logical_static(&self.first_expression, pipeline)? {
            let mut result = b;

            for c in &self.chain_expressions {
                match c {
                    ChainedLogicalExpression::Or(or) => {
                        if result {
                            // Short-circuiting chain because left-hand side of OR is true
                            break;
                        }

                        match try_resolve_logical_static(or, pipeline)? {
                            Some(b) => result = b,
                            None => return Ok(None),
                        }
                    }
                    ChainedLogicalExpression::And(and) => {
                        if !result {
                            // Short-circuiting chain because left-hand side of AND is false
                            break;
                        }

                        match try_resolve_logical_static(and, pipeline)? {
                            Some(b) => result = b,
                            None => return Ok(None),
                        }
                    }
                }
            }

            Ok(Some(ResolvedStaticScalarExpression::Value(
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    self.query_location.clone(),
                    result,
                )),
            )))
        } else {
            Ok(None)
        }
    }
}

impl Expression for ChainLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }

    fn get_name(&self) -> &'static str {
        "ChainLogicalExpression"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChainedLogicalExpression {
    Or(LogicalExpression),
    And(LogicalExpression),
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

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let left = self.get_left().try_resolve_static(pipeline)?;
        let right = self.get_right().try_resolve_static(pipeline)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let b = Value::are_values_equal(
                    &self.query_location,
                    &l.to_value(),
                    &r.to_value(),
                    self.case_insensitive,
                )?;

                Ok(Some(ResolvedStaticScalarExpression::Value(
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

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let left = self.get_left().try_resolve_static(pipeline)?;
        let right = self.get_right().try_resolve_static(pipeline)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let r = Value::compare_values(&self.query_location, &l.to_value(), &r.to_value())?;

                Ok(Some(ResolvedStaticScalarExpression::Value(
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

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let left = self.get_left().try_resolve_static(pipeline)?;
        let right = self.get_right().try_resolve_static(pipeline)?;

        match (left, right) {
            (Some(l), Some(r)) => {
                let r = Value::compare_values(&self.query_location, &l.to_value(), &r.to_value())?;

                Ok(Some(ResolvedStaticScalarExpression::Value(
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

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        if let Some(v) = try_resolve_logical_static(self.get_inner_expression(), pipeline)? {
            Ok(Some(ResolvedStaticScalarExpression::Value(
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

    pub(crate) fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        let haystack = self.get_haystack().try_resolve_static(pipeline)?;
        let needle = self.get_needle().try_resolve_static(pipeline)?;

        match (haystack, needle) {
            (Some(h), Some(n)) => {
                let r = Value::contains(
                    self.get_query_location(),
                    &h.to_value(),
                    &n.to_value(),
                    self.case_insensitive,
                )?;

                Ok(Some(ResolvedStaticScalarExpression::Value(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        self.query_location.clone(),
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
}

fn try_resolve_logical_static(
    logical_expression: &LogicalExpression,
    pipeline: &PipelineExpression,
) -> Result<Option<bool>, ExpressionError> {
    if let Some(s) = logical_expression.try_resolve_static(pipeline)? {
        let value = s.to_value();

        if let Some(b) = value.convert_to_bool() {
            Ok(Some(b))
        } else {
            Err(ExpressionError::TypeMismatch(
                logical_expression.get_query_location().clone(),
                format!(
                    "Value of '{:?}' type returned by logical expression could not be converted to bool",
                    value.get_value_type()
                ),
            ))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_to_try_resolve_static() {
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
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
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
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
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
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
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
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
    fn test_chain_try_resolve_static() {
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
        };

        run_test(
            LogicalExpression::Chain(ChainLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                        QueryLocation::new_fake(),
                        true,
                    )),
                )),
            )),
            Some(true),
        );

        run_test(
            LogicalExpression::Chain(ChainLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::Scalar(ScalarExpression::Temporal(
                    TemporalScalarExpression::Now(NowScalarExpression::new(
                        QueryLocation::new_fake(),
                    )),
                )),
            )),
            None,
        );

        let mut c1 = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        c1.push_or(LogicalExpression::Scalar(ScalarExpression::Temporal(
            TemporalScalarExpression::Now(NowScalarExpression::new(QueryLocation::new_fake())),
        )));

        // true || now() will evaluate to true because now() gets short-circuited
        run_test(LogicalExpression::Chain(c1), Some(true));

        let mut c2 = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        c2.push_and(LogicalExpression::Scalar(ScalarExpression::Temporal(
            TemporalScalarExpression::Now(NowScalarExpression::new(QueryLocation::new_fake())),
        )));

        // flase && now() will evaluate to false because now() gets short-circuited
        run_test(LogicalExpression::Chain(c2), Some(false));

        let mut c3 = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        c3.push_or(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(LogicalExpression::Chain(c3), Some(true));

        let mut c4 = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        c4.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(LogicalExpression::Chain(c4), Some(true));
    }

    #[test]
    fn test_contains_try_resolve_static() {
        let run_test = |input: LogicalExpression, expected: Option<bool>| {
            let pipeline = Default::default();

            let r = input.try_resolve_static(&pipeline).unwrap();

            assert_eq!(
                expected
                    .map(
                        |v| StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            QueryLocation::new_fake(),
                            v
                        ))
                    )
                    .as_ref()
                    .map(|v| v.to_value()),
                r.as_ref().map(|v| v.to_value())
            )
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
}
