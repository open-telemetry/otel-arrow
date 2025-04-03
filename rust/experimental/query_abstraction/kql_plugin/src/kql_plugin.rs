use crate::kql_parser::{KqlParser, Rule};
use intermediate_language::{
    grammar_objects::*,
    query_processor::{QueryError, QueryProcessor, QueryResult},
};
use pest::{iterators::Pair, Parser};

/// `KqlPlugin`` implements the `QueryProcessor` trait for KQL (Kusto Query Language)
pub struct KqlPlugin;

impl QueryProcessor for KqlPlugin {
    fn process_query(input: &str) -> Result<Query, QueryError> {
        let parsed_query = KqlParser::parse(Rule::query, input)
            .map_err(|e| QueryError::ParseError(e.to_string()))?
            .next()
            .ok_or_else(|| QueryError::ParseError("Empty query".to_string()))?;

        let mut source = String::new();
        let mut statements = Vec::new();

        for query_piece in parsed_query.into_inner() {
            match query_piece.as_rule() {
                Rule::identifier => {
                    source = query_piece.as_str().to_string();
                }
                Rule::statement => {
                    statements.extend(Self::process_statement(query_piece)?);
                }
                Rule::EOI => (),
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'query'",
                        query_piece.as_rule()
                    )))
                }
            }
        }

        Ok(Query { source, statements })
    }
}

impl KqlPlugin {
    /// Process a statement, returning a vector of Statement objects
    fn process_statement(statement: Pair<Rule>) -> QueryResult<Vec<Statement>> {
        let mut statements = Vec::new();

        for statement_piece in statement.into_inner() {
            match statement_piece.as_rule() {
                Rule::pipe_token => continue,
                Rule::filter_statement => {
                    Self::process_filter_statement(statement_piece, &mut statements)?;
                }
                Rule::extend_statement => {
                    Self::process_extend_statement(statement_piece, &mut statements)?;
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'statement'",
                        statement_piece.as_rule()
                    )))
                }
            }
        }

        if statements.is_empty() {
            return Err(QueryError::ProcessingError(
                "Expected a valid statement but none was found".to_string(),
            ));
        }

        Ok(statements)
    }

    /// Process a filter statement and add it to the statements vector
    fn process_filter_statement(
        filter_statement: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<()> {
        for filter_piece in filter_statement.into_inner() {
            match filter_piece.as_rule() {
                Rule::where_token => continue,
                Rule::predicate => {
                    statements.push(Statement::Filter(Self::process_predicate(filter_piece)?));
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'filter_statement'",
                    filter_piece.as_rule()
                )))
                }
            }
        }
        Ok(())
    }

    /// Process an extend statement and add it to the statements vector
    fn process_extend_statement(
        extend_statement: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<()> {
        for extend_piece in extend_statement.into_inner() {
            match extend_piece.as_rule() {
                Rule::extend_token | Rule::comma_token => continue,
                Rule::assignment_expression => {
                    let (identifier, value) = Self::process_assignment(extend_piece)?;
                    statements.push(Statement::Extend(identifier, value, None));
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'extend_statement'",
                    extend_piece.as_rule()
                )))
                }
            }
        }
        Ok(())
    }

    /// Process a predicate, returning a Predicate object
    fn process_predicate(predicate: Pair<Rule>) -> QueryResult<Predicate> {
        if let Some(predicate_piece) = predicate.into_inner().next() {
            match predicate_piece.as_rule() {
                Rule::binary_logical_expression => {
                    return Ok(Predicate::BinaryLogicalExpression(
                        Self::process_binary_logical_expression(predicate_piece)?,
                    ));
                }
                Rule::comparison_expression => {
                    return Ok(Predicate::ComparisonExpression(
                        Self::process_comparison_expression(predicate_piece)?,
                    ));
                }
                Rule::negated_expression => {
                    return Ok(Predicate::NegatedExpression(Box::new(
                        Self::process_negated_expression(predicate_piece)?,
                    )));
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'predicate'",
                        predicate_piece.as_rule()
                    )))
                }
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid predicate but none was found".to_string(),
        ))
    }

    /// Process an assignment expression, returning an identifier and value
    fn process_assignment(assignment: Pair<Rule>) -> QueryResult<(Identifier, Expression)> {
        let mut identifier = Identifier {
            name: String::new(),
        };
        let mut value = None;

        for assignment_piece in assignment.into_inner() {
            match assignment_piece.as_rule() {
                Rule::identifier => identifier = Self::process_identifier(assignment_piece),
                Rule::assignment_token => continue,
                Rule::expression => {
                    value = Some(Self::process_expression(assignment_piece)?);
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'assignment'",
                        assignment_piece.as_rule()
                    )))
                }
            }
        }

        let value = value.ok_or_else(|| {
            QueryError::ProcessingError("Missing value in assignment expression".to_string())
        })?;

        Ok((identifier, value))
    }

    fn process_binary_logical_expression(
        binary_logical_expression: Pair<Rule>,
    ) -> QueryResult<BinaryLogicalExpression> {
        if let Some(expression_piece) = binary_logical_expression.into_inner().next() {
            match expression_piece.as_rule() {
                Rule::and_expression | Rule::or_expression => {
                    let mut left = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                    let mut right = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                    let mut boolean_operator = BooleanOperator::And; // Default value
                    let logical_expression = expression_piece.into_inner();
                    for logical_piece in logical_expression {
                        match logical_piece.as_rule() {
                            Rule::expression_base => {
                                left = Self::process_expression(logical_piece)?;
                            }
                            Rule::expression => {
                                right = Self::process_expression(logical_piece)?;
                            }
                            Rule::and_token => boolean_operator = BooleanOperator::And,
                            Rule::or_token => boolean_operator = BooleanOperator::Or,
                            _ => return Err(QueryError::ProcessingError(format!(
                                "Unexpected rule: '{:?}' found processing parent object: 'logical_expression'",
                                logical_piece.as_rule()
                            ))),
                        }
                    }
                    return Ok(BinaryLogicalExpression {
                        left: Box::new(left),
                        boolean_operator,
                        right: Box::new(right),
                    });
                }
                _ => return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'binary_logical_expression'",
                    expression_piece.as_rule()
                ))),
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid binary logical expression but none was found".to_string(),
        ))
    }

    fn process_comparison_expression(
        comparison_expression: Pair<Rule>,
    ) -> QueryResult<ComparisonExpression> {
        if let Some(expression_piece) = comparison_expression.into_inner().next() {
            match expression_piece.as_rule() {
                Rule::equals_expression
                | Rule::not_equals_expression
                | Rule::greater_than_expression
                | Rule::less_than_expression
                | Rule::greater_than_or_equal_to_expression
                | Rule::less_than_or_equal_to_expression => {
                    let mut left = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                    let mut right = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                    let mut comparison_operator = ComparisonOperator::Equal; // Default value
                    let comparison_expression = expression_piece.into_inner();
                    for comparison_piece in comparison_expression {
                        match comparison_piece.as_rule() {
                            Rule::expression_base => {
                                left = Self::process_expression(comparison_piece)?;
                            }
                            Rule::expression => {
                                right = Self::process_expression(comparison_piece)?;
                            }
                            Rule::equals_token => comparison_operator = ComparisonOperator::Equal,
                            Rule::not_equals_token => {
                                comparison_operator = ComparisonOperator::NotEqual
                            }
                            Rule::greater_than_token => {
                                comparison_operator = ComparisonOperator::GreaterThan
                            }
                            Rule::less_than_token => comparison_operator = ComparisonOperator::LessThan,
                            Rule::greater_than_or_equal_to_token => {
                                comparison_operator = ComparisonOperator::GreaterThanOrEqual
                            }
                            Rule::less_than_or_equal_to_token => {
                                comparison_operator = ComparisonOperator::LessThanOrEqual
                            }
                            _ => return Err(QueryError::ProcessingError(format!(
                                "Unexpected rule: '{:?}' found processing parent object: 'comparison_expression'",
                                comparison_piece.as_rule()
                            ))),
                        }
                    }
                    return Ok(ComparisonExpression {
                        left: Box::new(left),
                        comparison_operator,
                        right: Box::new(right),
                    });
                }
                _ => return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'comparison_expression'",
                    expression_piece.as_rule()
                ))),
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid comparison expression but none was found".to_string(),
        ))
    }

    fn process_negated_expression(negated_expression: Pair<Rule>) -> QueryResult<Expression> {
        for negated_piece in negated_expression.into_inner() {
            match negated_piece.as_rule() {
                Rule::not_token => continue,
                Rule::enclosed_expression => {
                    return Self::process_enclosed_expression(negated_piece)
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'negated_expression'",
                    negated_piece.as_rule()
                )))
                }
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid negated expression but none was found".to_string(),
        ))
    }

    fn process_enclosed_expression(enclosed_expression: Pair<Rule>) -> QueryResult<Expression> {
        for enclosed_piece in enclosed_expression.into_inner() {
            match enclosed_piece.as_rule() {
                Rule::open_paren_token | Rule::close_paren_token => continue,
                Rule::expression => {
                    return Self::process_expression(enclosed_piece);
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'enclosed_expression'",
                    enclosed_piece.as_rule()
                )))
                }
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid enclosed expression but none was found".to_string(),
        ))
    }

    fn process_expression(expression: Pair<Rule>) -> QueryResult<Expression> {
        if let Some(expression_piece) = expression.into_inner().next() {
            match expression_piece.as_rule() {
                Rule::predicate => {
                    return Ok(Expression::Predicate(Self::process_predicate(
                        expression_piece,
                    )?));
                }
                Rule::enclosed_expression => {
                    return Ok(Expression::EnclosedExpression(Box::new(
                        Self::process_enclosed_expression(expression_piece)?,
                    )));
                }
                Rule::literal => {
                    return Ok(Expression::Literal(Self::process_literal(expression_piece)));
                }
                Rule::identifier => {
                    return Ok(Expression::Identifier(Self::process_identifier(
                        expression_piece,
                    )));
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'expression'",
                        expression_piece.as_rule()
                    )))
                }
            }
        }
        Err(QueryError::ProcessingError(
            "Expected a valid expression but none was found".to_string(),
        ))
    }

    fn process_identifier(identifier: Pair<Rule>) -> Identifier {
        Identifier {
            name: identifier.as_str().to_string(),
        }
    }

    fn process_literal(literal: Pair<Rule>) -> Literal {
        if let Ok(value) = literal.as_str().parse::<i32>() {
            Literal::Int(value)
        } else if literal.as_str() == "true" {
            Literal::Bool(true)
        } else if literal.as_str() == "false" {
            Literal::Bool(false)
        } else {
            Literal::String(literal.as_str().trim_matches('"').to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_literal() {
        let literal = KqlParser::parse(Rule::literal, "42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_literal(literal);
        assert_eq!(result, Literal::Int(42));

        let literal = KqlParser::parse(Rule::literal, "true")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_literal(literal);
        assert_eq!(result, Literal::Bool(true));

        let literal = KqlParser::parse(Rule::literal, "\"hello\"")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_literal(literal);
        assert_eq!(result, Literal::String("hello".to_string()));
    }

    #[test]
    fn test_process_identifier() {
        let identifier = KqlParser::parse(Rule::identifier, "my_variable")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_identifier(identifier);
        assert_eq!(result.name, "my_variable");
    }

    #[test]
    fn test_process_expression_predicate() {
        let expression = KqlParser::parse(Rule::expression, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_expression(expression).unwrap();
        assert_eq!(
            result,
            Expression::Predicate(Predicate::ComparisonExpression(ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }))
        );
    }

    #[test]
    fn test_process_expression_enclosed_expression() {
        let expression = KqlParser::parse(Rule::expression, "(x == 42)")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_expression(expression).unwrap();
        assert_eq!(
            result,
            Expression::EnclosedExpression(Box::new(Expression::Predicate(
                Predicate::ComparisonExpression(ComparisonExpression {
                    left: Box::new(Expression::Identifier(Identifier {
                        name: "x".to_string(),
                    })),
                    comparison_operator: ComparisonOperator::Equal,
                    right: Box::new(Expression::Literal(Literal::Int(42))),
                })
            )))
        );
    }

    #[test]
    fn test_process_expression_literal() {
        let expression = KqlParser::parse(Rule::expression, "42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_expression(expression).unwrap();
        assert_eq!(result, Expression::Literal(Literal::Int(42)));
    }

    #[test]
    fn test_process_expression_identifier() {
        let expression = KqlParser::parse(Rule::expression, "my_variable")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_expression(expression).unwrap();
        assert_eq!(
            result,
            Expression::Identifier(Identifier {
                name: "my_variable".to_string(),
            })
        );
    }

    #[test]
    fn test_process_enclosed_expression() {
        let expression = KqlParser::parse(Rule::enclosed_expression, "(x == 42)")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_enclosed_expression(expression).unwrap();
        assert_eq!(
            result,
            Expression::Predicate(Predicate::ComparisonExpression(ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }))
        );
    }

    #[test]
    fn test_process_negated_expression() {
        let expression = KqlParser::parse(Rule::negated_expression, "not(x == 42)")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_negated_expression(expression).unwrap();
        assert_eq!(
            result,
            Expression::Predicate(Predicate::ComparisonExpression(ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }))
        );
    }

    #[test]
    fn test_process_comparison_expression_equals() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_comparison_expression_not_equals() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x != 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::NotEqual,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_comparison_expression_greater_than() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x > 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::GreaterThan,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_comparison_expression_less_than() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x < 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::LessThan,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_comparison_expression_greater_than_or_equal_to() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x >= 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::GreaterThanOrEqual,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_comparison_expression_less_than_or_equal_to() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x <= 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_comparison_expression(expression).unwrap();
        assert_eq!(
            result,
            ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::LessThanOrEqual,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }
        );
    }

    #[test]
    fn test_process_binary_logical_expression_and() {
        let expression = KqlParser::parse(Rule::binary_logical_expression, "x and y")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_binary_logical_expression(expression).unwrap();
        assert_eq!(
            result,
            BinaryLogicalExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                boolean_operator: BooleanOperator::And,
                right: Box::new(Expression::Identifier(Identifier {
                    name: "y".to_string(),
                })),
            }
        );
    }

    #[test]
    fn test_process_binary_logical_expression_or() {
        let expression = KqlParser::parse(Rule::binary_logical_expression, "x or y")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_binary_logical_expression(expression).unwrap();
        assert_eq!(
            result,
            BinaryLogicalExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                boolean_operator: BooleanOperator::Or,
                right: Box::new(Expression::Identifier(Identifier {
                    name: "y".to_string(),
                })),
            }
        );
    }

    #[test]
    fn test_process_binary_logical_expression_nested() {
        let expression = KqlParser::parse(
            Rule::binary_logical_expression,
            "((x == 42) and (y == 24)) or z == 10",
        )
        .unwrap()
        .next()
        .unwrap();
        let result = KqlPlugin::process_binary_logical_expression(expression).unwrap();
        assert_eq!(
            result,
            BinaryLogicalExpression {
                left: Box::new(Expression::EnclosedExpression(Box::new(
                    Expression::Predicate(Predicate::BinaryLogicalExpression(
                        BinaryLogicalExpression {
                            left: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "x".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::Equal,
                                        right: Box::new(Expression::Literal(Literal::Int(42))),
                                    }
                                ))
                            ))),
                            boolean_operator: BooleanOperator::And,
                            right: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "y".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::Equal,
                                        right: Box::new(Expression::Literal(Literal::Int(24))),
                                    }
                                ))
                            ))),
                        }
                    ))
                ))),
                boolean_operator: BooleanOperator::Or,
                right: Box::new(Expression::Predicate(Predicate::ComparisonExpression(
                    ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "z".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(10))),
                    }
                ))),
            }
        );
    }

    #[test]
    fn test_process_assignment_simple() {
        let assignment = KqlParser::parse(Rule::assignment_expression, "x = 42")
            .unwrap()
            .next()
            .unwrap();
        let (identifier, value) = KqlPlugin::process_assignment(assignment).unwrap();
        assert_eq!(identifier.name, "x");
        assert_eq!(value, Expression::Literal(Literal::Int(42)));
    }

    #[test]
    fn test_process_predicate_binary_logical_expression() {
        let predicate = KqlParser::parse(Rule::predicate, "x and y")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_predicate(predicate).unwrap();
        assert_eq!(
            result,
            Predicate::BinaryLogicalExpression(BinaryLogicalExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                boolean_operator: BooleanOperator::And,
                right: Box::new(Expression::Identifier(Identifier {
                    name: "y".to_string(),
                })),
            })
        );
    }

    #[test]
    fn test_process_predicate_comparison_expression() {
        let predicate = KqlParser::parse(Rule::predicate, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_predicate(predicate).unwrap();
        assert_eq!(
            result,
            Predicate::ComparisonExpression(ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            })
        );
    }

    #[test]
    fn test_process_predicate_negated_expression() {
        let predicate = KqlParser::parse(Rule::predicate, "not(x == 42)")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_predicate(predicate).unwrap();
        assert_eq!(
            result,
            Predicate::NegatedExpression(Box::new(Expression::Predicate(
                Predicate::ComparisonExpression(ComparisonExpression {
                    left: Box::new(Expression::Identifier(Identifier {
                        name: "x".to_string(),
                    })),
                    comparison_operator: ComparisonOperator::Equal,
                    right: Box::new(Expression::Literal(Literal::Int(42))),
                })
            )))
        );
    }

    #[test]
    fn test_process_predicate_nested_binary_logical_expression() {
        let predicate = KqlParser::parse(Rule::predicate, "(x == 42) and (y == 24)")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_predicate(predicate).unwrap();
        assert_eq!(
            result,
            Predicate::BinaryLogicalExpression(BinaryLogicalExpression {
                left: Box::new(Expression::EnclosedExpression(Box::new(
                    Expression::Predicate(Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "x".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(42))),
                    }))
                ))),
                boolean_operator: BooleanOperator::And,
                right: Box::new(Expression::EnclosedExpression(Box::new(
                    Expression::Predicate(Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "y".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(24))),
                    }))
                ))),
            })
        );
    }

    #[test]
    fn test_process_statement_filter_statement() {
        let statement = KqlParser::parse(Rule::statement, "| where x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_statement(statement).unwrap();
        assert_eq!(
            result,
            vec![Statement::Filter(Predicate::ComparisonExpression(
                ComparisonExpression {
                    left: Box::new(Expression::Identifier(Identifier {
                        name: "x".to_string(),
                    })),
                    comparison_operator: ComparisonOperator::Equal,
                    right: Box::new(Expression::Literal(Literal::Int(42))),
                }
            ))]
        );
    }

    #[test]
    fn test_process_statement_extend_statement() {
        let statement = KqlParser::parse(Rule::statement, "| extend x = 42")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_statement(statement).unwrap();
        assert_eq!(
            result,
            vec![Statement::Extend(
                Identifier {
                    name: "x".to_string(),
                },
                Expression::Literal(Literal::Int(42)),
                None
            )]
        );
    }

    #[test]
    fn test_process_statement_extend_statement_multiple() {
        let statement = KqlParser::parse(Rule::statement, "| extend x = 42, y = 24")
            .unwrap()
            .next()
            .unwrap();
        let result = KqlPlugin::process_statement(statement).unwrap();
        assert_eq!(
            result,
            vec![
                Statement::Extend(
                    Identifier {
                        name: "x".to_string(),
                    },
                    Expression::Literal(Literal::Int(42)),
                    None
                ),
                Statement::Extend(
                    Identifier {
                        name: "y".to_string(),
                    },
                    Expression::Literal(Literal::Int(24)),
                    None
                )
            ]
        );
    }

    #[test]
    fn test_process_query() {
        let input = "my_table | where my_variable == 5";
        let result = KqlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "my_table".to_string(),
                statements: vec![Statement::Filter(Predicate::ComparisonExpression(
                    ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "my_variable".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(5))),
                    }
                ))]
            }
        );
    }

    #[test]
    fn test_process_query_multi_line() {
        let input = "my_table 
        | where my_variable == 5";
        let result = KqlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "my_table".to_string(),
                statements: vec![Statement::Filter(Predicate::ComparisonExpression(
                    ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "my_variable".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(5))),
                    }
                ))]
            }
        );
    }

    #[test]
    fn test_process_query_complex() {
        let input = r#"my_table 
        | where my_variable == 5
        | where (x < 42) and (y > 24)
        | where not(z >= 10)
        | extend new_column = true, another_new_column = (another_variable == 100)
        "#;
        let result = KqlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "my_table".to_string(),
                statements: vec![
                    Statement::Filter(Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "my_variable".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(5))),
                    })),
                    Statement::Filter(Predicate::BinaryLogicalExpression(
                        BinaryLogicalExpression {
                            left: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "x".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::LessThan,
                                        right: Box::new(Expression::Literal(Literal::Int(42))),
                                    }
                                ))
                            ))),
                            boolean_operator: BooleanOperator::And,
                            right: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "y".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::GreaterThan,
                                        right: Box::new(Expression::Literal(Literal::Int(24))),
                                    }
                                ))
                            ))),
                        }
                    )),
                    Statement::Filter(Predicate::NegatedExpression(Box::new(
                        Expression::Predicate(Predicate::ComparisonExpression(
                            ComparisonExpression {
                                left: Box::new(Expression::Identifier(Identifier {
                                    name: "z".to_string(),
                                })),
                                comparison_operator: ComparisonOperator::GreaterThanOrEqual,
                                right: Box::new(Expression::Literal(Literal::Int(10))),
                            }
                        ))
                    ))),
                    Statement::Extend(
                        Identifier {
                            name: "new_column".to_string(),
                        },
                        Expression::Literal(Literal::Bool(true)),
                        None
                    ),
                    Statement::Extend(
                        Identifier {
                            name: "another_new_column".to_string(),
                        },
                        Expression::EnclosedExpression(Box::new(Expression::Predicate(
                            Predicate::ComparisonExpression(ComparisonExpression {
                                left: Box::new(Expression::Identifier(Identifier {
                                    name: "another_variable".to_string(),
                                })),
                                comparison_operator: ComparisonOperator::Equal,
                                right: Box::new(Expression::Literal(Literal::Int(100))),
                            })
                        ))),
                        None
                    )
                ]
            }
        );
    }
}
