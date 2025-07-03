use crate::{
    Expression, ExpressionError, PipelineExpression, QueryLocation, ResolvedStaticScalarExpression,
    ScalarExpression,
};

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
}

impl LogicalExpression {
    pub fn try_resolve_static<'a, 'b, 'c>(
        &'a self,
        pipeline: &'b PipelineExpression,
    ) -> Result<Option<ResolvedStaticScalarExpression<'c>>, ExpressionError>
    where
        'a: 'c,
        'b: 'c,
    {
        match self {
            LogicalExpression::Scalar(s) => s.try_resolve_static(pipeline),
            // todo: Implement static resolution of logicals:
            LogicalExpression::EqualTo(_) => Ok(None),
            LogicalExpression::GreaterThan(_) => Ok(None),
            LogicalExpression::GreaterThanOrEqualTo(_) => Ok(None),
            LogicalExpression::Not(_) => Ok(None),
            LogicalExpression::Chain(_) => Ok(None),
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

    pub fn get_expressions(&self) -> (&LogicalExpression, &Vec<ChainedLogicalExpression>) {
        (&self.first_expression, &self.chain_expressions)
    }
}

impl Expression for ChainLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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
}

impl EqualToLogicalExpression {
    pub fn new(
        query_location: QueryLocation,
        left: ScalarExpression,
        right: ScalarExpression,
    ) -> EqualToLogicalExpression {
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
}

impl Expression for EqualToLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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
}

impl Expression for GreaterThanLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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
}

impl Expression for GreaterThanOrEqualToLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
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
}

impl Expression for NotLogicalExpression {
    fn get_query_location(&self) -> &QueryLocation {
        &self.query_location
    }
}
