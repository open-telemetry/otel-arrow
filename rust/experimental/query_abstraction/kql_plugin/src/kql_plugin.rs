use crate::kql_parser::{KqlParser, Rule};
use intermediate_language::{grammar_objects::*, query_processor::QueryProcessor};
use pest::{iterators::Pair, Parser};

pub struct KqlPlugin;

impl QueryProcessor for KqlPlugin {
    fn process_query(input: &str) -> Query {
        let mut source = "".to_string();
        let mut statements: Vec<Statement> = Vec::new();
        let query = KqlParser::parse(Rule::query, input)
            .expect("Failed to parse query")
            .next()
            .unwrap();

        for query_piece in query.clone().into_inner() {
            match query_piece.as_rule() {
                Rule::identifier => {
                    source = query_piece.as_str().to_string();
                }
                Rule::statement => {
                    statements.extend(process_statement(query_piece));
                }
                Rule::EOI => (),
                _ => handle_unexpected_rule("query", query_piece),
            }
        }

        Query { source, statements }
    }
}

fn process_statement(statement: Pair<Rule>) -> Vec<Statement> {
    let mut statements: Vec<Statement> = Vec::new();
    for statement_piece in statement.into_inner() {
        match statement_piece.as_rule() {
            Rule::pipe_token => continue,
            Rule::filter_statement => {
                let filter_statement = statement_piece.into_inner();
                for filter_piece in filter_statement {
                    match filter_piece.as_rule() {
                        Rule::where_token => continue,
                        Rule::predicate => {
                            statements.push(Statement::Filter(process_predicate(filter_piece)));
                        }
                        _ => handle_unexpected_rule("filter_statement", filter_piece),
                    }
                }
            }
            Rule::extend_statement => {
                let extend_statement = statement_piece.into_inner();
                for extend_piece in extend_statement {
                    match extend_piece.as_rule() {
                        Rule::extend_token | Rule::comma_token => continue,
                        Rule::assignment_expression => {
                            let (identifier, value) = process_assignment(extend_piece);
                            statements.push(Statement::Extend(identifier, value, None));
                        }
                        _ => handle_unexpected_rule("extend_statement", extend_piece),
                    }
                }
            }
            _ => handle_unexpected_rule("statement", statement_piece),
        }
    }
    if statements.is_empty() {
        panic!("Expected a valid statement but none was found");
    }
    statements
}

fn process_predicate(predicate: Pair<Rule>) -> Predicate {
    for predicate_piece in predicate.into_inner() {
        match predicate_piece.as_rule() {
            Rule::binary_logical_expression => {
                return Predicate::BinaryLogicalExpression(process_binary_logical_expression(
                    predicate_piece,
                ));
            }
            Rule::comparison_expression => {
                return Predicate::ComparisonExpression(process_comparison_expression(
                    predicate_piece,
                ));
            }
            Rule::negated_expression => {
                return Predicate::NegatedExpression(Box::new(process_negated_expression(
                    predicate_piece,
                )));
            }
            _ => handle_unexpected_rule("predicate", predicate_piece),
        }
    }
    unreachable!("Expected a valid predicate but none was found");
}

fn process_assignment(assignment: Pair<Rule>) -> (Identifier, Expression) {
    let mut identifier = Identifier {
        name: "".to_string(),
    }; // Placeholder for actual value
    let mut value = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
    for assignment_piece in assignment.into_inner() {
        match assignment_piece.as_rule() {
            Rule::identifier => identifier = process_identifier(assignment_piece),
            Rule::assignment_token => continue,
            Rule::expression => {
                value = process_expression(assignment_piece);
            }
            _ => handle_unexpected_rule("assignment", assignment_piece),
        }
    }
    (identifier, value)
}

fn process_binary_logical_expression(
    binary_logical_expression: Pair<Rule>,
) -> BinaryLogicalExpression {
    for expression_piece in binary_logical_expression.into_inner() {
        match expression_piece.as_rule() {
            Rule::and_expression | Rule::or_expression => {
                let mut left = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                let mut right = Expression::Literal(Literal::Int(0)); // Placeholder for actual value
                let mut boolean_operator = BooleanOperator::And; // Default value
                let logical_expression = expression_piece.into_inner();
                for logical_piece in logical_expression {
                    match logical_piece.as_rule() {
                        Rule::expression_base => {
                            left = process_expression(logical_piece);
                        }
                        Rule::expression => {
                            right = process_expression(logical_piece);
                        }
                        Rule::and_token => boolean_operator = BooleanOperator::And,
                        Rule::or_token => boolean_operator = BooleanOperator::Or,
                        _ => handle_unexpected_rule("logical_expression", logical_piece),
                    }
                }
                return BinaryLogicalExpression {
                    left: Box::new(left),
                    boolean_operator,
                    right: Box::new(right),
                };
            }
            _ => handle_unexpected_rule("binary_logical_expression", expression_piece),
        }
    }
    unreachable!("Expected a valid binary logical expression but none was found");
}

fn process_comparison_expression(comparison_expression: Pair<Rule>) -> ComparisonExpression {
    for expression_piece in comparison_expression.into_inner() {
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
                            left = process_expression(comparison_piece);
                        }
                        Rule::expression => {
                            right = process_expression(comparison_piece);
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
                        _ => handle_unexpected_rule("comparison_expression", comparison_piece),
                    }
                }
                return ComparisonExpression {
                    left: Box::new(left),
                    comparison_operator,
                    right: Box::new(right),
                };
            }
            _ => handle_unexpected_rule("comparison_expression", expression_piece),
        }
    }
    unreachable!("Expected a valid comparison expression but none was found");
}

fn process_negated_expression(negated_expression: Pair<Rule>) -> Expression {
    for negated_piece in negated_expression.into_inner() {
        match negated_piece.as_rule() {
            Rule::not_token => continue,
            Rule::enclosed_expression => return process_enclosed_expression(negated_piece),
            _ => handle_unexpected_rule("negated_expression", negated_piece),
        }
    }
    unreachable!("Expected a valid negated expression but none was found");
}

fn process_enclosed_expression(enclosed_expression: Pair<Rule>) -> Expression {
    for enclosed_piece in enclosed_expression.into_inner() {
        match enclosed_piece.as_rule() {
            Rule::open_paren_token | Rule::close_paren_token => continue,
            Rule::expression => {
                return process_expression(enclosed_piece);
            }
            _ => handle_unexpected_rule("enclosed_expression", enclosed_piece),
        }
    }
    unreachable!("Expected a valid enclosed expression but none was found");
}

fn process_expression(expression: Pair<Rule>) -> Expression {
    for expression_piece in expression.into_inner() {
        match expression_piece.as_rule() {
            Rule::predicate => {
                return Expression::Predicate(process_predicate(expression_piece));
            }
            Rule::enclosed_expression => {
                return Expression::EnclosedExpression(Box::new(process_enclosed_expression(
                    expression_piece,
                )));
            }
            Rule::literal => {
                return Expression::Literal(process_literal(expression_piece));
            }
            Rule::identifier => {
                return Expression::Identifier(process_identifier(expression_piece));
            }
            _ => handle_unexpected_rule("expression", expression_piece),
        }
    }
    unreachable!("Expected a valid expression but none was found");
}

fn process_identifier(identifier: Pair<Rule>) -> Identifier {
    Identifier {
        name: identifier.as_str().to_string(),
    }
}

fn process_literal(literal: Pair<Rule>) -> Literal {
    if let Ok(value) = literal.as_str().parse::<i32>() {
        return Literal::Int(value);
    } else if literal.as_str() == "true" {
        return Literal::Bool(true);
    } else if literal.as_str() == "false" {
        return Literal::Bool(false);
    } else {
        return Literal::String(literal.as_str().trim_matches('"').to_string());
    }
}

fn handle_unexpected_rule(parent_object: &str, found_rule: Pair<Rule>) {
    panic!(
        "Unexpected rule: '{:?}' found processing parent object: '{:?}'",
        found_rule.as_rule(),
        parent_object
    );
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
        let result = process_literal(literal);
        assert_eq!(result, Literal::Int(42));

        let literal = KqlParser::parse(Rule::literal, "true")
            .unwrap()
            .next()
            .unwrap();
        let result = process_literal(literal);
        assert_eq!(result, Literal::Bool(true));

        let literal = KqlParser::parse(Rule::literal, "\"hello\"")
            .unwrap()
            .next()
            .unwrap();
        let result = process_literal(literal);
        assert_eq!(result, Literal::String("hello".to_string()));
    }

    #[test]
    fn test_process_identifier() {
        let identifier = KqlParser::parse(Rule::identifier, "my_variable")
            .unwrap()
            .next()
            .unwrap();
        let result = process_identifier(identifier);
        assert_eq!(result.name, "my_variable");
    }

    #[test]
    fn test_process_expression_predicate() {
        let expression = KqlParser::parse(Rule::expression, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = process_expression(expression);
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
        let result = process_expression(expression);
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
        let result = process_expression(expression);
        assert_eq!(
            result,
            Expression::Literal(Literal::Int(42))
        );
    }

    
    #[test]
    fn test_process_expression_identifier() {
        let expression = KqlParser::parse(Rule::expression, "my_variable")
            .unwrap()
            .next()
            .unwrap();
        let result = process_expression(expression);
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
        let result = process_enclosed_expression(expression);
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
        let result = process_negated_expression(expression);
        assert_eq!(
            result,
            Expression::Predicate(
                Predicate::ComparisonExpression(ComparisonExpression {
                    left: Box::new(Expression::Identifier(Identifier {
                        name: "x".to_string(),
                    })),
                    comparison_operator: ComparisonOperator::Equal,
                    right: Box::new(Expression::Literal(Literal::Int(42))),
                })
            )
        );
    }

    #[test]
    fn test_process_comparison_expression_equals() {
        let expression = KqlParser::parse(Rule::comparison_expression, "x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = process_comparison_expression(expression);
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
        let result = process_comparison_expression(expression);
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
        let result = process_comparison_expression(expression);
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
        let result = process_comparison_expression(expression);
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
        let result = process_comparison_expression(expression);
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
        let result = process_comparison_expression(expression);
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
        let result = process_binary_logical_expression(expression);
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
        let result = process_binary_logical_expression(expression);
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
        let result = process_binary_logical_expression(expression);
        assert_eq!(
            result,
            BinaryLogicalExpression {
                left: Box::new(Expression::EnclosedExpression(Box::new(
                    Expression::Predicate(Predicate::BinaryLogicalExpression(
                        BinaryLogicalExpression {
                            left: Box::new(Expression::EnclosedExpression(Box::new(Expression::Predicate(
                                Predicate::ComparisonExpression(ComparisonExpression {
                                    left: Box::new(Expression::Identifier(Identifier {
                                        name: "x".to_string(),
                                    })),
                                    comparison_operator: ComparisonOperator::Equal,
                                    right: Box::new(Expression::Literal(Literal::Int(42))),
                                })
                            )))),
                            boolean_operator: BooleanOperator::And,
                            right: Box::new(Expression::EnclosedExpression(Box::new(Expression::Predicate(
                                Predicate::ComparisonExpression(ComparisonExpression {
                                    left: Box::new(Expression::Identifier(Identifier {
                                        name: "y".to_string(),
                                    })),
                                    comparison_operator: ComparisonOperator::Equal,
                                    right: Box::new(Expression::Literal(Literal::Int(24))),
                                })
                            )))),
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
        let (identifier, value) = process_assignment(assignment);
        assert_eq!(identifier.name, "x");
        assert_eq!(value, Expression::Literal(Literal::Int(42)));
    }

    #[test]
    fn test_process_predicate_binary_logical_expression() {
        let predicate = KqlParser::parse(Rule::predicate, "x and y")
            .unwrap()
            .next()
            .unwrap();
        let result = process_predicate(predicate);
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
        let result = process_predicate(predicate);
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
        let result = process_predicate(predicate);
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
        let result = process_predicate(predicate);
        assert_eq!(
            result,
            Predicate::BinaryLogicalExpression(BinaryLogicalExpression {
                left: Box::new(Expression::EnclosedExpression(Box::new(Expression::Predicate(
                    Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "x".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(42))),
                    })
                )))),
                boolean_operator: BooleanOperator::And,
                right: Box::new(Expression::EnclosedExpression(Box::new(Expression::Predicate(
                    Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "y".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(24))),
                    })
                )))),
            })
        );
    }

    #[test]
    fn test_process_statement_filter_statement() {
        let statement = KqlParser::parse(Rule::statement, "| where x == 42")
            .unwrap()
            .next()
            .unwrap();
        let result = process_statement(statement);
        assert_eq!(
            result,
            vec![Statement::Filter(Predicate::ComparisonExpression(ComparisonExpression {
                left: Box::new(Expression::Identifier(Identifier {
                    name: "x".to_string(),
                })),
                comparison_operator: ComparisonOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Int(42))),
            }))]
        );
    }

    #[test]
    fn test_process_statement_extend_statement() {
        let statement = KqlParser::parse(Rule::statement, "| extend x = 42")
            .unwrap()
            .next()
            .unwrap();
        let result = process_statement(statement);
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
        let result = process_statement(statement);
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
        let query = KqlPlugin::process_query(input);
        assert_eq!(
            query,
            Query {
                source: "my_table".to_string(),
                statements: vec![Statement::Filter(
                    Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "my_variable".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(5))),
                    })
                )]
            }
        )
    }

    #[test]
    fn test_process_query_multi_line() {
        let input = "my_table 
        | where my_variable == 5";
        let query = KqlPlugin::process_query(input);
        assert_eq!(
            query,
            Query {
                source: "my_table".to_string(),
                statements: vec![Statement::Filter(
                    Predicate::ComparisonExpression(ComparisonExpression {
                        left: Box::new(Expression::Identifier(Identifier {
                            name: "my_variable".to_string(),
                        })),
                        comparison_operator: ComparisonOperator::Equal,
                        right: Box::new(Expression::Literal(Literal::Int(5))),
                    })
                )]
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
        let query = KqlPlugin::process_query(input);
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
                        Expression::EnclosedExpression(Box::new(
                            Expression::Predicate(Predicate::ComparisonExpression(
                                ComparisonExpression {
                                    left: Box::new(Expression::Identifier(Identifier {
                                        name: "another_variable".to_string(),
                                    })),
                                    comparison_operator: ComparisonOperator::Equal,
                                    right: Box::new(Expression::Literal(Literal::Int(100))),
                                }
                            ))
                        )),
                        None
                    )
                ]
            }
        );
    }
}
