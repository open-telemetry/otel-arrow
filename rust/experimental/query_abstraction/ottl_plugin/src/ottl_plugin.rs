use crate::ottl_parser::{OttlParser, Rule};
use intermediate_language::{
    grammar_objects::*,
    query_processor::{QueryError, QueryProcessor, QueryResult},
};
use pest::{iterators::Pair, Parser};

/// `OttlPlugin` implements the `QueryProcessor` trait for OTTL (OpenTelemetry Transform Language)
pub struct OttlPlugin;

impl QueryProcessor for OttlPlugin {
    fn process_query(input: &str) -> Result<Query, QueryError> {
        let parsed_query = OttlParser::parse(Rule::query, input)
            .map_err(|e| QueryError::ParseError(e.to_string()))?
            .next()
            .ok_or_else(|| QueryError::ParseError("Empty query".to_string()))?;

        let mut source = String::new();
        let mut statements = Vec::new();

        for query_piece in parsed_query.into_inner() {
            match query_piece.as_rule() {
                Rule::filter_query => {
                    source = Self::process_filter_query(query_piece, &mut statements)?;
                }
                Rule::transform_query => {
                    source = Self::process_transform_query(query_piece, &mut statements)?;
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

impl OttlPlugin {
    /// Process a filter query, returning the source and populating the statements vector
    fn process_filter_query(
        filter_query: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<String> {
        let mut source = String::new();

        for filter_piece in filter_query.into_inner() {
            match filter_piece.as_rule() {
                Rule::filter_token => continue,
                Rule::logs_token => continue,
                Rule::log_record_token => {
                    source = "logs".to_string();
                }
                Rule::filter_statement => {
                    Self::process_filter_statement(filter_piece, statements)?;
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'filter_query'",
                        filter_piece.as_rule()
                    )))
                }
            }
        }

        Ok(source)
    }

    /// Process a transform query, returning the source and populating the statements vector
    fn process_transform_query(
        transform_query: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<String> {
        let mut source = String::new();

        for transform_piece in transform_query.into_inner() {
            match transform_piece.as_rule() {
                Rule::transform_token => continue,
                Rule::log_statements_token => {
                    source = "log_statements".to_string();
                }
                Rule::extend_statement => {
                    Self::process_extend_statement(transform_piece, statements)?;
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'transform_query'",
                        transform_piece.as_rule()
                    )))
                }
            }
        }

        Ok(source)
    }

    /// Process a filter statement and add it to the statements vector
    fn process_filter_statement(
        filter_statement: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<()> {
        for filter_piece in filter_statement.into_inner() {
            match filter_piece.as_rule() {
                Rule::dash_token | Rule::single_quote_token => continue,
                Rule::predicate => {
                    statements.push(Statement::Filter(Self::process_predicate(filter_piece)?));
                }
                _ => return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'filter_statement'",
                    filter_piece.as_rule()
                ))),
            }
        }
        Ok(())
    }

    /// Process an extend statement and add it to the statements vector
    fn process_extend_statement(
        extend_statement: Pair<Rule>,
        statements: &mut Vec<Statement>,
    ) -> QueryResult<()> {
        let mut identifier = Identifier {
            name: String::new(),
        };
        let mut value = Expression::Literal(Literal::Int(0)); // Default placeholder
        let mut predicate = None;

        for extend_piece in extend_statement.into_inner() {
            match extend_piece.as_rule() {
                Rule::dash_token => continue,
                Rule::set_with_paren => {
                    let (id, expr) = Self::process_set_with_paren(extend_piece)?;
                    identifier = id;
                    value = expr;
                }
                Rule::where_token => continue,
                Rule::predicate => {
                    predicate = Some(Self::process_predicate(extend_piece)?);
                }
                _ => return Err(QueryError::ProcessingError(format!(
                    "Unexpected rule: '{:?}' found processing parent object: 'extend_statement'",
                    extend_piece.as_rule()
                ))),
            }
        }

        statements.push(Statement::Extend(identifier, value, predicate));
        Ok(())
    }

    /// Process a set_with_paren and extract the identifier and expression
    fn process_set_with_paren(set_with_paren: Pair<Rule>) -> QueryResult<(Identifier, Expression)> {
        for set_piece in set_with_paren.into_inner() {
            match set_piece.as_rule() {
                Rule::set_token | Rule::open_paren_token | Rule::close_paren_token => continue,
                Rule::set_tuple => {
                    let (id, expr) = Self::process_set_tuple(set_piece)?;
                    return Ok((id, expr));
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'set_with_paren'",
                        set_piece.as_rule()
                    )))
                }
            }
        }

        Err(QueryError::ProcessingError(
            "Expected a valid set_tuple but none was found".to_string(),
        ))
    }

    /// Process a set_tuple and extract the identifier and expression
    fn process_set_tuple(set_tuple: Pair<Rule>) -> QueryResult<(Identifier, Expression)> {
        let mut identifier = Identifier {
            name: String::new(),
        };
        let mut value = None;

        for set_piece in set_tuple.into_inner() {
            match set_piece.as_rule() {
                Rule::identifier => identifier = Self::process_identifier(set_piece),
                Rule::comma_token => continue,
                Rule::expression => {
                    value = Some(Self::process_expression(set_piece)?);
                }
                _ => {
                    return Err(QueryError::ProcessingError(format!(
                        "Unexpected rule: '{:?}' found processing parent object: 'set_tuple'",
                        set_piece.as_rule()
                    )))
                }
            }
        }

        let value = value
            .ok_or_else(|| QueryError::ProcessingError("Missing value in set tuple".to_string()))?;

        Ok((identifier, value))
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
                            Rule::less_than_token => {
                                comparison_operator = ComparisonOperator::LessThan
                            }
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
        let literal = OttlParser::parse(Rule::literal, "42")
            .unwrap()
            .next()
            .unwrap();
        let result = OttlPlugin::process_literal(literal);
        assert_eq!(result, Literal::Int(42));

        let literal = OttlParser::parse(Rule::literal, "true")
            .unwrap()
            .next()
            .unwrap();
        let result = OttlPlugin::process_literal(literal);
        assert_eq!(result, Literal::Bool(true));

        let literal = OttlParser::parse(Rule::literal, "\"hello\"")
            .unwrap()
            .next()
            .unwrap();
        let result = OttlPlugin::process_literal(literal);
        assert_eq!(result, Literal::String("hello".to_string()));
    }

    #[test]
    fn test_process_identifier() {
        let identifier = OttlParser::parse(Rule::identifier, "my_variable")
            .unwrap()
            .next()
            .unwrap();
        let result = OttlPlugin::process_identifier(identifier);
        assert_eq!(result.name, "my_variable");
    }

    #[test]
    fn test_process_expression_predicate() {
        let expression = OttlParser::parse(Rule::expression, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = OttlPlugin::process_expression(expression).unwrap();
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
    fn test_process_set_tuple() {
        let set_tuple = OttlParser::parse(Rule::set_tuple, "x, 42")
            .unwrap()
            .next()
            .unwrap();
        let (identifier, value) = OttlPlugin::process_set_tuple(set_tuple).unwrap();
        assert_eq!(identifier.name, "x");
        assert_eq!(value, Expression::Literal(Literal::Int(42)));
    }

    #[test]
    fn test_process_set_with_paren() {
        let set_with_paren = OttlParser::parse(Rule::set_with_paren, "set(x, 42)")
            .unwrap()
            .next()
            .unwrap();
        let (identifier, value) = OttlPlugin::process_set_with_paren(set_with_paren).unwrap();
        assert_eq!(identifier.name, "x");
        assert_eq!(value, Expression::Literal(Literal::Int(42)));
    }

    #[test]
    fn test_process_filter_statement() {
        let filter_statement = OttlParser::parse(Rule::filter_statement, "- 'x == 42'")
            .unwrap()
            .next()
            .unwrap();
        let mut statements = Vec::new();
        OttlPlugin::process_filter_statement(filter_statement, &mut statements).unwrap();
        assert_eq!(
            statements,
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
    fn test_process_extend_statement() {
        let extend_statement = OttlParser::parse(Rule::extend_statement, "- set(x, 42)")
            .unwrap()
            .next()
            .unwrap();
        let mut statements = Vec::new();
        OttlPlugin::process_extend_statement(extend_statement, &mut statements).unwrap();
        assert_eq!(
            statements,
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
    fn test_process_extend_statement_with_where() {
        let extend_statement = OttlParser::parse(
            Rule::extend_statement,
            r#"- set(x, 42) where someColumn == "blue""#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut statements = Vec::new();
        OttlPlugin::process_extend_statement(extend_statement, &mut statements).unwrap();
        assert_eq!(
            statements,
            vec![Statement::Extend(
                Identifier {
                    name: "x".to_string(),
                },
                Expression::Literal(Literal::Int(42)),
                Some(Predicate::ComparisonExpression(ComparisonExpression {
                    left: Box::new(Expression::Identifier(Identifier {
                        name: "someColumn".to_string(),
                    })),
                    comparison_operator: ComparisonOperator::Equal,
                    right: Box::new(Expression::Literal(Literal::String("blue".to_string()))),
                }))
            )]
        );
    }

    #[test]
    fn test_process_filter_query() {
        let filter_query = OttlParser::parse(
            Rule::filter_query,
            r#"filter:
            logs:
            log_record:
            - 'x == 42'"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut statements = Vec::new();
        let source = OttlPlugin::process_filter_query(filter_query, &mut statements).unwrap();
        assert_eq!(source, "logs");
        assert_eq!(
            statements,
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
    fn test_process_transform_query() {
        let transform_query = OttlParser::parse(
            Rule::transform_query,
            r#"transform:
            log_statements:
            - set(x, 42)"#,
        )
        .unwrap()
        .next()
        .unwrap();
        let mut statements = Vec::new();
        let source = OttlPlugin::process_transform_query(transform_query, &mut statements).unwrap();
        assert_eq!(source, "log_statements");
        assert_eq!(
            statements,
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
    fn test_process_query_filter() {
        let input = r#"filter:
        logs:
        log_record:
        - 'x == 42'"#;
        let result = OttlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "logs".to_string(),
                statements: vec![Statement::Filter(Predicate::ComparisonExpression(
                    ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "x".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(42))),
                    }
                ))]
            }
        );
    }

    #[test]
    fn test_process_query_transform() {
        let input = r#"transform:
        log_statements:
        - set(x, 42)"#;
        let result = OttlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "log_statements".to_string(),
                statements: vec![Statement::Extend(
                    Identifier {
                        name: "x".to_string(),
                    },
                    Expression::Literal(Literal::Int(42)),
                    None
                )]
            }
        );
    }

    #[test]
    fn test_process_query_transform_complex() {
        let input = r#"transform:
        log_statements:
        - set(x, 42)
        - set(y, "hello") where z > 10
        - set(a, true) where (b == 5) and (c != "world")"#;
        let result = OttlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "log_statements".to_string(),
                statements: vec![
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
                        Expression::Literal(Literal::String("hello".to_string())),
                        Some(Predicate::ComparisonExpression(ComparisonExpression {
                            left: Box::new(Expression::Identifier(Identifier {
                                name: "z".to_string(),
                            })),
                            comparison_operator: ComparisonOperator::GreaterThan,
                            right: Box::new(Expression::Literal(Literal::Int(10))),
                        }))
                    ),
                    Statement::Extend(
                        Identifier {
                            name: "a".to_string(),
                        },
                        Expression::Literal(Literal::Bool(true)),
                        Some(Predicate::BinaryLogicalExpression(
                            BinaryLogicalExpression {
                                left: Box::new(Expression::EnclosedExpression(Box::new(
                                    Expression::Predicate(Predicate::ComparisonExpression(
                                        ComparisonExpression {
                                            left: Box::new(Expression::Identifier(Identifier {
                                                name: "b".to_string(),
                                            })),
                                            comparison_operator: ComparisonOperator::Equal,
                                            right: Box::new(Expression::Literal(Literal::Int(5))),
                                        }
                                    ))
                                ))),
                                boolean_operator: BooleanOperator::And,
                                right: Box::new(Expression::EnclosedExpression(Box::new(
                                    Expression::Predicate(Predicate::ComparisonExpression(
                                        ComparisonExpression {
                                            left: Box::new(Expression::Identifier(Identifier {
                                                name: "c".to_string(),
                                            })),
                                            comparison_operator: ComparisonOperator::NotEqual,
                                            right: Box::new(Expression::Literal(Literal::String(
                                                "world".to_string()
                                            ))),
                                        }
                                    ))
                                ))),
                            }
                        ))
                    )
                ]
            }
        );
    }

    #[test]
    fn test_process_query_filter_complex() {
        let input = r#"filter:
        logs:
        log_record:
        - 'x == 42'
        - 'y > 10'
        - 'not(z <= 5)'
        - '(a == true) or (b != "test")'"#;
        let result = OttlPlugin::process_query(input);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(
            query,
            Query {
                source: "logs".to_string(),
                statements: vec![
                    Statement::Filter(Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "x".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(42))),
                    })),
                    Statement::Filter(Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "y".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::GreaterThan,
                        right: Box::new(Expression::Literal(Literal::Int(10))),
                    })),
                    Statement::Filter(Predicate::NegatedExpression(Box::new(
                        Expression::Predicate(Predicate::ComparisonExpression(
                            ComparisonExpression {
                                left: Box::new(Expression::Identifier(Identifier {
                                    name: "z".to_string(),
                                })),
                                comparison_operator: ComparisonOperator::LessThanOrEqual,
                                right: Box::new(Expression::Literal(Literal::Int(5))),
                            }
                        ))
                    ))),
                    Statement::Filter(Predicate::BinaryLogicalExpression(
                        BinaryLogicalExpression {
                            left: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "a".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::Equal,
                                        right: Box::new(Expression::Literal(Literal::Bool(true))),
                                    }
                                ))
                            ))),
                            boolean_operator: BooleanOperator::Or,
                            right: Box::new(Expression::EnclosedExpression(Box::new(
                                Expression::Predicate(Predicate::ComparisonExpression(
                                    ComparisonExpression {
                                        left: Box::new(Expression::Identifier(Identifier {
                                            name: "b".to_string(),
                                        })),
                                        comparison_operator: ComparisonOperator::NotEqual,
                                        right: Box::new(Expression::Literal(Literal::String(
                                            "test".to_string()
                                        ))),
                                    }
                                ))
                            ))),
                        }
                    ))
                ]
            }
        );
    }

    #[test]
    fn test_process_query_expect_parsing_error() {
        let invalid_inputs = vec![
            "filter:",
            "filter: logs:",
            "filter: logs: log_record:",
            "transform: log_statements:",
            "transform: log_statements: - set(x)",
            "transform: log_statements: - set(x,)",
            "filter: logs: log_record: - 'x = 42'",
        ];

        for input in invalid_inputs {
            let result = OttlPlugin::process_query(input);
            assert!(result.is_err(), "Expected error for input: {}", input);
            assert!(matches!(result, Err(QueryError::ParseError(_))));
        }
    }
}
