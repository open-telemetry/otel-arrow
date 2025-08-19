// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

#[derive(Clone, PartialEq)]
pub struct Query {
    pub source: String,
    pub statements: Vec<Statement>,
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Query")?;
        writeln!(f, "├── Source: {:?}", self.source)?;

        if self.statements.is_empty() {
            writeln!(f, "└── Statements: []")
        } else {
            writeln!(f, "└── Statements:")?;
            let last_idx = self.statements.len() - 1;
            for (i, stmt) in self.statements.iter().enumerate() {
                if i == last_idx {
                    write!(f, "    └── ")?;
                    stmt.fmt_with_indent(f, "        ")?;
                } else {
                    write!(f, "    ├── ")?;
                    stmt.fmt_with_indent(f, "    │   ")?;
                    writeln!(f)?;
                }
            }
            Ok(())
        }
    }
}

#[derive(Clone, PartialEq)]
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

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Filter(predicate) => {
                writeln!(f, "Filter")?;
                write!(f, "└── ")?;
                predicate.fmt_with_indent(f, "    ")
            }
            Statement::Extend(ident, expr, pred_opt) => {
                writeln!(f, "Extend")?;
                writeln!(f, "├── Field: {ident:?}")?;
                match pred_opt {
                    Some(pred) => {
                        writeln!(f, "├── Expression:")?;
                        write!(f, "│   └── ")?;
                        expr.fmt_with_indent(f, "│       ")?;
                        writeln!(f, "└── Condition:")?;
                        write!(f, "    └── ")?;
                        pred.fmt_with_indent(f, "        ")
                    }
                    None => {
                        writeln!(f, "└── Expression:")?;
                        write!(f, "    └── ")?;
                        expr.fmt_with_indent(f, "        ")
                    }
                }
            }
        }
    }
}

impl Statement {
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            Statement::Filter(predicate) => {
                writeln!(f, "Filter")?;
                write!(f, "{indent}└── ")?;
                predicate.fmt_with_indent(f, &format!("{indent}    "))
            }
            Statement::Extend(ident, expr, pred_opt) => {
                writeln!(f, "Extend")?;
                writeln!(f, "{indent}├── Field: {ident:?}")?;
                match pred_opt {
                    Some(pred) => {
                        writeln!(f, "{indent}├── Expression:")?;
                        write!(f, "{indent}│   └── ")?;
                        expr.fmt_with_indent(f, &format!("{indent}│       "))?;
                        writeln!(f)?;
                        writeln!(f, "{indent}└── Condition:")?;
                        write!(f, "{indent}    └── ")?;
                        pred.fmt_with_indent(f, &format!("{indent}        "))
                    }
                    None => {
                        writeln!(f, "{indent}└── Expression:")?;
                        write!(f, "{indent}    └── ")?;
                        expr.fmt_with_indent(f, &format!("{indent}        "))
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Predicate(Predicate),
    EnclosedExpression(Box<Expression>),
}

impl Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Identifier(ident) => write!(f, "{ident:?}"),
            Expression::Literal(lit) => write!(f, "{lit:?}"),
            Expression::Predicate(pred) => {
                writeln!(f, "Predicate")?;
                write!(f, "└── ")?;
                pred.fmt_with_indent(f, "    ")
            }
            Expression::EnclosedExpression(expr) => {
                writeln!(f, "EnclosedExpression")?;
                write!(f, "└── ")?;
                expr.fmt_with_indent(f, "    ")
            }
        }
    }
}

impl Expression {
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            Expression::Identifier(ident) => write!(f, "{ident:?}"),
            Expression::Literal(lit) => write!(f, "{lit:?}"),
            Expression::Predicate(pred) => {
                writeln!(f, "Predicate")?;
                write!(f, "{indent}└── ")?;
                pred.fmt_with_indent(f, &format!("{indent}    "))
            }
            Expression::EnclosedExpression(expr) => {
                writeln!(f, "EnclosedExpression")?;
                write!(f, "{indent}└── ")?;
                expr.fmt_with_indent(f, &format!("{indent}    "))
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Predicate {
    BinaryLogicalExpression(BinaryLogicalExpression),
    ComparisonExpression(ComparisonExpression),
    NegatedExpression(Box<Expression>),
}

impl Debug for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::BinaryLogicalExpression(ble) => {
                write!(f, "{ble:?}")
            }
            Predicate::ComparisonExpression(ce) => {
                write!(f, "{ce:?}")
            }
            Predicate::NegatedExpression(expr) => {
                writeln!(f, "NegatedExpression")?;
                write!(f, "└── ")?;
                expr.fmt_with_indent(f, "    ")
            }
        }
    }
}

impl Predicate {
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        match self {
            Predicate::BinaryLogicalExpression(ble) => ble.fmt_with_indent(f, indent),
            Predicate::ComparisonExpression(ce) => ce.fmt_with_indent(f, indent),
            Predicate::NegatedExpression(expr) => {
                writeln!(f, "NegatedExpression")?;
                write!(f, "{indent}└── ")?;
                expr.fmt_with_indent(f, &format!("{indent}    "))
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct BinaryLogicalExpression {
    pub left: Box<Expression>,
    pub boolean_operator: BooleanOperator,
    pub right: Box<Expression>,
}

impl Debug for BinaryLogicalExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BinaryLogicalExpression ({:?})", self.boolean_operator)?;
        write!(f, "├── ")?;
        self.left.fmt_with_indent(f, "│   ")?;
        writeln!(f)?;
        write!(f, "└── ")?;
        self.right.fmt_with_indent(f, "    ")
    }
}

impl BinaryLogicalExpression {
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "BinaryLogicalExpression ({:?})", self.boolean_operator)?;
        write!(f, "{indent}├── ")?;
        self.left.fmt_with_indent(f, &format!("{indent}│   "))?;
        writeln!(f)?;
        write!(f, "{indent}└── ")?;
        self.right.fmt_with_indent(f, &format!("{indent}    "))
    }
}

#[derive(Clone, PartialEq)]
pub struct ComparisonExpression {
    pub left: Box<Expression>,
    pub comparison_operator: ComparisonOperator,
    pub right: Box<Expression>,
}

impl Debug for ComparisonExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ComparisonExpression ({:?})", self.comparison_operator)?;
        write!(f, "├── ")?;
        self.left.fmt_with_indent(f, "│   ")?;
        writeln!(f)?;
        write!(f, "└── ")?;
        self.right.fmt_with_indent(f, "    ")
    }
}

impl ComparisonExpression {
    fn fmt_with_indent(&self, f: &mut std::fmt::Formatter<'_>, indent: &str) -> std::fmt::Result {
        writeln!(f, "ComparisonExpression ({:?})", self.comparison_operator)?;
        write!(f, "{indent}├── ")?;
        self.left.fmt_with_indent(f, &format!("{indent}│   "))?;
        writeln!(f)?;
        write!(f, "{indent}└── ")?;
        self.right.fmt_with_indent(f, &format!("{indent}    "))
    }
}

#[derive(Clone, PartialEq)]
pub struct Identifier {
    pub name: String,
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identifier({})", self.name)
    }
}

#[derive(Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Int(i32),
    String(String),
}

impl Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Bool(val) => write!(f, "Bool({val})"),
            Literal::Int(val) => write!(f, "Int({val})"),
            Literal::String(val) => write!(f, "String(\"{val}\")"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum BooleanOperator {
    And,
    Or,
}

impl Debug for BooleanOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanOperator::And => write!(f, "AND"),
            BooleanOperator::Or => write!(f, "OR"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Debug for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Equal => write!(f, "=="),
            ComparisonOperator::NotEqual => write!(f, "!="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::GreaterThanOrEqual => write!(f, ">="),
            ComparisonOperator::LessThanOrEqual => write!(f, "<="),
        }
    }
}
