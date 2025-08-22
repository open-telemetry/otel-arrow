// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_negate_expression(
    negate_expression_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&negate_expression_rule);
    let mut inner = negate_expression_rule.into_inner();

    // Grammar guarantees exactly one scalar_expression
    let scalar_expr_rule = inner.next().unwrap();

    let scalar = parse_scalar_expression(scalar_expr_rule, state)?;

    Ok(ScalarExpression::Math(MathScalarExpression::Negate(
        UnaryMathematicalScalarExpression::new(query_location, scalar),
    )))
}

pub(crate) fn parse_bin_expression(
    bin_expression_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&bin_expression_rule);

    let mut inner = bin_expression_rule.into_inner();

    let left_scalar = parse_scalar_expression(inner.next().unwrap(), state)?;
    let right_scalar = parse_scalar_expression(inner.next().unwrap(), state)?;

    Ok(ScalarExpression::Math(MathScalarExpression::Bin(
        BinaryMathematicalScalarExpression::new(query_location, left_scalar, right_scalar),
    )))
}

pub(crate) fn parse_arithmetic_expression(
    arithmetic_expr_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&arithmetic_expr_rule);
    let mut inner_rules = arithmetic_expr_rule.into_inner();

    let first = inner_rules.next().unwrap();

    let mut current_expr = parse_arithmetic_factor(first, state)?;

    while let Some(op_rule) = inner_rules.next() {
        let right = parse_arithmetic_factor(inner_rules.next().unwrap(), state)?;

        current_expr = match op_rule.as_rule() {
            Rule::plus_token => ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    query_location.clone(),
                    current_expr,
                    right,
                ),
            )),
            Rule::minus_token => ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathematicalScalarExpression::new(
                    query_location.clone(),
                    current_expr,
                    right,
                ),
            )),
            _ => panic!("Unexpected operator in arithmetic_expression: {op_rule}"),
        };
    }

    Ok(current_expr)
}

fn parse_arithmetic_factor(
    factor_rule: Pair<Rule>,
    state: &dyn ParserScope,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&factor_rule);
    let mut inner_rules = factor_rule.into_inner();
    let mut current_expr = parse_scalar_expression(inner_rules.next().unwrap(), state)?;

    // Process multiplication/division/modulo operations
    while let Some(op_rule) = inner_rules.next() {
        let right = parse_scalar_expression(inner_rules.next().unwrap(), state)?;

        current_expr = match op_rule.as_rule() {
            Rule::multiply_token => ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathematicalScalarExpression::new(
                    query_location.clone(),
                    current_expr,
                    right,
                ),
            )),
            Rule::divide_token => ScalarExpression::Math(MathScalarExpression::Divide(
                BinaryMathematicalScalarExpression::new(
                    query_location.clone(),
                    current_expr,
                    right,
                ),
            )),
            Rule::modulo_token => ScalarExpression::Math(MathScalarExpression::Modulus(
                BinaryMathematicalScalarExpression::new(
                    query_location.clone(),
                    current_expr,
                    right,
                ),
            )),
            _ => panic!("Unexpected operator in arithmetic_factor: {op_rule}"),
        };
    }

    Ok(current_expr)
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use pest::Parser;

    use crate::KqlPestParser;

    use super::*;

    #[test]
    fn test_parse_negate_expression() {
        let test_cases = vec![
            (
                "-toint('1')",
                ScalarExpression::Convert(ConvertScalarExpression::Integer(
                    ConversionScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "1"),
                        )),
                    ),
                )),
            ),
            (
                "-todouble('1.0')",
                ScalarExpression::Convert(ConvertScalarExpression::Double(
                    ConversionScalarExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "1.0"),
                        )),
                    ),
                )),
            ),
        ];

        for (input, value) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::scalar_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_scalar_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Math(MathScalarExpression::Negate(n)) => {
                    assert_eq!(&value, n.get_value_expression());
                }
                _ => panic!("Expected MathScalarExpression::Negate"),
            }
        }
    }

    #[test]
    fn test_parse_bin_expression() {
        let test_cases = vec![
            (
                "bin(1009, 10)",
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1009),
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                )),
            ),
            (
                "bin(1009, 1 h)",
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1009),
                )),
                ScalarExpression::Static(StaticScalarExpression::TimeSpan(
                    TimeSpanScalarExpression::new(QueryLocation::new_fake(), TimeDelta::hours(1)),
                )),
            ),
        ];

        for (input, left, right) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::scalar_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_scalar_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Math(MathScalarExpression::Bin(b)) => {
                    assert_eq!(&left, b.get_left_expression());
                    assert_eq!(&right, b.get_right_expression());
                }
                _ => panic!("Expected MathScalarExpression::Bin"),
            }
        }
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        // Test simple addition
        run_test_success(
            "(5 + 3)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                ),
            )),
        );

        // Test simple subtraction
        run_test_success(
            "(10 - 4)",
            ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test simple multiplication
        run_test_success(
            "(6 * 7)",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 6),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 7),
                    )),
                ),
            )),
        );

        // Test simple division
        run_test_success(
            "(20 / 4)",
            ScalarExpression::Math(MathScalarExpression::Divide(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test simple modulo
        run_test_success(
            "(10 % 3)",
            ScalarExpression::Math(MathScalarExpression::Modulus(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                    )),
                ),
            )),
        );

        // Test operator precedence: multiplication before addition
        run_test_success(
            "(2 + 3 * 4)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Multiply(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                            )),
                        ),
                    )),
                ),
            )),
        );

        // Test parentheses override precedence
        run_test_success(
            "((2 + 3) * 4)",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                    )),
                ),
            )),
        );

        // Test complex expression with multiple operators
        run_test_success(
            "(10 + 20 / 4 - 3 * 2)",
            ScalarExpression::Math(MathScalarExpression::Subtract(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 10),
                            )),
                            ScalarExpression::Math(MathScalarExpression::Divide(
                                BinaryMathematicalScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    ScalarExpression::Static(StaticScalarExpression::Integer(
                                        IntegerScalarExpression::new(QueryLocation::new_fake(), 20),
                                    )),
                                    ScalarExpression::Static(StaticScalarExpression::Integer(
                                        IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                                    )),
                                ),
                            )),
                        ),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Multiply(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                        ),
                    )),
                ),
            )),
        );

        // Test arithmetic with doubles
        run_test_success(
            "(4.44 + 2.86)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 4.44),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Double(
                        DoubleScalarExpression::new(QueryLocation::new_fake(), 2.86),
                    )),
                ),
            )),
        );

        // Test arithmetic with variables
        run_test_success(
            "(x + y)",
            ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "x",
                            )),
                        )]),
                    )),
                    ScalarExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                            StaticScalarExpression::String(StringScalarExpression::new(
                                QueryLocation::new_fake(),
                                "y",
                            )),
                        )]),
                    )),
                ),
            )),
        );

        // Test nested parentheses
        run_test_success(
            "((2 + 3) * (4 + 5))",
            ScalarExpression::Math(MathScalarExpression::Multiply(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
                            )),
                        ),
                    )),
                    ScalarExpression::Math(MathScalarExpression::Add(
                        BinaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 4),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Integer(
                                IntegerScalarExpression::new(QueryLocation::new_fake(), 5),
                            )),
                        ),
                    )),
                ),
            )),
        );
    }

    #[test]
    fn test_pest_parse_arithmetic_expression() {
        // Add arithmetic expressions to the pest parser tests
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::scalar_expression,
            &[
                // Basic arithmetic
                "(1 + 2)",
                "(10 - 5)",
                "(3 * 4)",
                "(20 / 4)",
                "(10 % 3)",
                // With doubles
                "(3.14 + 2.86)",
                "(10.5 * 2.0)",
                // With variables
                "(x + y)",
                "(a * b + c)",
                // Operator precedence
                "(2 + 3 * 4)",
                "(10 - 6 / 2)",
                "(5 + 10 % 3)",
                // Parentheses
                "((2 + 3) * 4)",
                "(5 * (10 - 8))",
                "(((1 + 2) * 3) + 4)",
                // Unary operators
                "(-(5 + 3))",
                // Complex expressions
                "(a + b * c - d / e % f)",
                "((a + b) * (c - d) / (e + f))",
                "(1 + 2 * 3 - 4 / 2 + 5 % 3)",
            ],
            &[
                // Invalid expressions
                "(+ +)", "(5 +)", "(* 3)", "(10 /)", "(% 5)",
            ],
        );
    }

    #[test]
    fn test_arithmetic_constant_evaluation() {
        let evaluate_constant = |input: &str| -> Option<i64> {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            // Create a dummy pipeline for static resolution
            let pipeline = PipelineExpression::default();

            if let Ok(Some(resolved)) = expression.try_resolve_static(&pipeline) {
                match resolved.to_value() {
                    Value::Integer(i) => Some(i.get_value()),
                    _ => None,
                }
            } else {
                None
            }
        };
        // Test basic arithmetic
        assert_eq!(evaluate_constant("(2 + 3)"), Some(5));
        assert_eq!(evaluate_constant("(10 - 4)"), Some(6));
        assert_eq!(evaluate_constant("(6 * 7)"), Some(42));
        assert_eq!(evaluate_constant("(10 % 3)"), Some(1));
        assert_eq!(evaluate_constant("(10 / 2)"), Some(5));

        // Test operator precedence
        assert_eq!(evaluate_constant("(2 + 3 * 4)"), Some(14)); // not 20
        assert_eq!(evaluate_constant("((2 + 3) * 4)"), Some(20));
        assert_eq!(evaluate_constant("(10 - 6 * 2)"), Some(-2));
        assert_eq!(evaluate_constant("((2 + 3) * (4 + 5))"), Some(45));
        assert_eq!(evaluate_constant("(1 + 2 * 3 - 4 / 2 + 5 % 3)"), Some(7));
        assert_eq!(evaluate_constant("(1 + 2 * 3 - 5 / 2 + 5 % 3)"), Some(7));
        assert_eq!(evaluate_constant("(10 / 3)"), Some(3));

        // Test unary operators
        assert_eq!(evaluate_constant("(-5 + 3)"), Some(-2));
        assert_eq!(evaluate_constant("(-(5 + 3))"), Some(-8));
    }

    #[test]
    fn test_arithmetic_double_evaluation() {
        let evaluate_constant_double = |input: &str| -> Option<f64> {
            println!("Testing: {input}");

            let state = ParserState::new(input);
            let mut result = KqlPestParser::parse(Rule::scalar_expression, input).unwrap();
            let expression = parse_scalar_expression(result.next().unwrap(), &state).unwrap();

            let pipeline = PipelineExpression::default();

            if let Ok(Some(resolved)) = expression.try_resolve_static(&pipeline) {
                match resolved.to_value() {
                    Value::Double(d) => Some(d.get_value()),
                    _ => None,
                }
            } else {
                None
            }
        };

        // Division always returns double
        assert_eq!(evaluate_constant_double("(10.0 / 3)"), Some(10.0 / 3.0));
        assert_eq!(
            evaluate_constant_double("(1 + 2 * 3 - 5.0 / 2 + 5 % 3)"),
            Some(6.5)
        );

        // Double arithmetic
        assert_eq!(evaluate_constant_double("(3.14 + 2.86)"), Some(6.0));
        assert_eq!(evaluate_constant_double("(10.5 * 2.0)"), Some(21.0));
    }
}
