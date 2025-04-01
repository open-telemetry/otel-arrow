use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct Query {
    pub source: String,
    pub expressions: Vec<LogicalExpression>,
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Query {{")?;
        writeln!(f, "  source: {:?},", self.source)?;
        writeln!(f, "  logical_expressions: [")?;
        for expression in &self.expressions {
            writeln!(f, "    {:?},", expression)?;
        }
        writeln!(f, "  ]")?;
        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalExpression {
    /// Reduce data volume by filtering out data via a [`ConditionalExpression`].
    /// * ConditionalExpression: The condition that determines which data to keep.
    Filter(ConditionalExpression),
    /// Add or alter data by default or based on an optional [`ConditionalExpression`].
    /// * Identifier: The name of the field to be added or altered.
    /// * Expression: The expression to be evaluated and assigned to the field.
    /// * ConditionalExpression: An optional condition that determines when the field should be added or altered.
    Extend(Identifier, Expression, Option<ConditionalExpression>),
    // Apply an aggregation function like sum, count, etc by grouping data.
    // Aggregate(...)
    // Transform data by applying more complex functions like replace, truncate, etc.
    // Transform(...)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    ConditionalExpression(ConditionalExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpression {
    pub left: Box<Expression>,
    pub operator: ConditionalOperator,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Int(i32),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalOperator {
    And,
    Or,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}
