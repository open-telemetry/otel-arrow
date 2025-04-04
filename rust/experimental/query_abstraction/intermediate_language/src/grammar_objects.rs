use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct Query {
    pub source: String,
    pub statements: Vec<Statement>,
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Query {{")?;
        writeln!(f, "  source: {:?},", self.source)?;
        writeln!(f, "  statements: [")?;
        for expression in &self.statements {
            writeln!(f, "    {:?},", expression)?;
        }
        writeln!(f, "  ]")?;
        writeln!(f, "}}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Reduce data volume by filtering out data via a [`Predicate`].
    /// * Predicate: The condition that determines which data to keep.
    Filter(Predicate),
    /// Add or alter data by default or based on an optional [`Predicate`].
    /// * Identifier: The name of the field to be added or altered.
    /// * Expression: The expression to be evaluated and assigned to the field.
    /// * Predicate: An optional condition that determines when the field should be added or altered.
    Extend(Identifier, Expression, Option<Predicate>),
    // Apply an aggregation function like sum, count, etc by grouping data.
    // Aggregate(...)
    // Transform data by applying more complex functions like replace, truncate, etc.
    // Transform(...)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Predicate(Predicate),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    BinaryLogicalExpression(BinaryLogicalExpression),
    ComparisonExpression(ComparisonExpression),
    NegatedExpression(NegatedExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NegatedExpression {
    expression: Box<Predicate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryLogicalExpression {
    left: Box<Expression>,
    boolean_operator: BooleanOperator,
    right: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonExpression {
    left: Box<Expression>,
    comparison_operator: ComparisonOperator,
    right: Box<Expression>,
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
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}
