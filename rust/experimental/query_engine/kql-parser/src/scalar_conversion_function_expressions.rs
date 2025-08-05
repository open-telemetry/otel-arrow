use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

pub(crate) fn parse_tostring_expression(
    tostring_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&tostring_rule);
    let mut inner = tostring_rule.into_inner();

    // Grammar guarantees exactly one scalar_expression
    let scalar_expr_rule = inner.next().unwrap();

    // Parse and wrap in conversion expression
    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::String(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_toint_expression(
    toint_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&toint_rule);
    let mut inner = toint_rule.into_inner();

    // Grammar guarantees exactly one scalar_expression
    let scalar_expr_rule = inner.next().unwrap();

    // Parse and wrap in conversion expression
    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Integer(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_tobool_expression(
    tobool_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&tobool_rule);
    let mut inner = tobool_rule.into_inner();

    let scalar_expr_rule = inner.next().unwrap();

    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Boolean(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_tofloat_expression(
    tofloat_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&tofloat_rule);
    let mut inner = tofloat_rule.into_inner();

    let scalar_expr_rule = inner.next().unwrap();

    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Double(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_tolong_expression(
    tolong_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&tolong_rule);
    let mut inner = tolong_rule.into_inner();

    let scalar_expr_rule = inner.next().unwrap();

    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Integer(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_toreal_expression(
    toreal_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&toreal_rule);
    let mut inner = toreal_rule.into_inner();

    let scalar_expr_rule = inner.next().unwrap();

    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Double(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

pub(crate) fn parse_todouble_expression(
    todouble_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&todouble_rule);
    let mut inner = todouble_rule.into_inner();

    let scalar_expr_rule = inner.next().unwrap();

    let inner_expr = parse_scalar_expression(scalar_expr_rule, state)?;
    Ok(ScalarExpression::Convert(ConvertScalarExpression::Double(
        ConversionScalarExpression::new(query_location, inner_expr),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KqlPestParser;
    use pest::Parser;

    #[test]
    fn test_pest_parse_tostring_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::tostring_expression,
            &[
                "tostring(123)",
                "tostring(-456)",
                "tostring(0)",
                "tostring(1.5)",
                "tostring(-3.14)",
                "tostring(true)",
                "tostring(false)",
                "tostring(null)",
                "tostring(\"hello\")",
                "tostring('world')",
                "tostring(\"\")",
                "tostring(x)",
                "tostring(myVariable)",
                "tostring(_var123)",
                "tostring((42))",
                "tostring(((x)))",
                "tostring(iff(true, 1, 0))",
                "tostring( 42 )",
                "tostring(  42  )",
            ],
            &[
                "tostring()",
                "tostring(1, 2)",
                "tostring(1, 2, 3)",
                "tostring 42",
                "tostring42",
                "tostring(,)",
                "tostring(42,)",
                "tostring(,42)",
                "toString(42)",
                "TOSTRING(42)",
                "ToString(42)",
                "toSTRING(42)",
                "tostring(",
                "tostring(42",
                "tostring)",
                "tostring[]",
                "tostring{}",
                "tostring<42>",
            ],
        );
    }

    #[test]
    fn test_tostring_query_location() {
        let input = "tostring(42)";
        let state = ParserState::new(input);
        let mut parsed = KqlPestParser::parse(Rule::tostring_expression, input).unwrap();

        let result = parse_tostring_expression(parsed.next().unwrap(), &state).unwrap();

        // Verify the result has proper location tracking
        match result {
            ScalarExpression::Convert(ConvertScalarExpression::String(conv_expr)) => {
                let location = conv_expr.get_query_location();
                let start_and_end = location.get_start_and_end_positions();
                assert_eq!(start_and_end.0, 0);
                assert_eq!(start_and_end.1, input.len());
            }
            _ => panic!("Expected ConvertScalarExpression::String"),
        }
    }

    #[test]
    fn test_tostring_creates_correct_conversion_expression() {
        // This test verifies that the parser creates the correct AST structure
        // that will later be evaluated by ConvertScalarExpression::String

        let test_cases = vec![
            ("tostring(42)", "integer literal"),
            ("tostring(true)", "boolean literal"),
            ("tostring(4.44)", "float literal"),
            ("tostring(\"hello\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::tostring_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_tostring_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            // Verify we created the correct AST node that will trigger
            // the String conversion logic in convert_scalar_expression.rs
            match result {
                ScalarExpression::Convert(ConvertScalarExpression::String(_conv_expr)) => {
                    // This confirms the parser created the right structure
                    // that will use the conversion logic you showed

                    // When this AST is evaluated, it will:
                    // 1. Evaluate the inner expression
                    // 2. Call convert_to_string() on the result
                    // 3. Return a StringScalarExpression with the converted value

                    println!("Parser correctly created String conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::String for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_toint_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::toint_expression,
            &[
                "toint(123)",
                "toint(-456)",
                "toint(0)",
                "toint(1.5)",
                "toint(-3.14)",
                "toint(true)",
                "toint(false)",
                "toint(null)",
                "toint(\"42\")",
                "toint('123')",
                "toint(x)",
                "toint(myVariable)",
                "toint(_var123)",
                "toint((42))",
                "toint(iff(true, 1, 0))",
                "toint( 42 )",
            ],
            &[
                "toint()",
                "toint(1, 2)",
                "toint 42",
                "toint42",
                "toInt(42)",
                "TOINT(42)",
                "ToInt(42)",
                "toint(",
                "toint(42",
                "toint)",
            ],
        );
    }

    #[test]
    fn test_toint_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("toint(42)", "integer literal"),
            ("toint(true)", "boolean literal"),
            ("toint(4.44)", "float literal"),
            ("toint(\"42\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::toint_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_toint_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Integer(_conv_expr)) => {
                    println!("Parser correctly created Integer conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Integer for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_tobool_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::tobool_expression,
            &[
                "tobool(123)",
                "tobool(0)",
                "tobool(1.5)",
                "tobool(0.0)",
                "tobool(true)",
                "tobool(false)",
                "tobool(null)",
                "tobool(\"true\")",
                "tobool('false')",
                "tobool(x)",
                "tobool(myVariable)",
                "tobool(_var123)",
                "tobool((42))",
                "tobool(iff(true, 1, 0))",
                "tobool( 42 )",
            ],
            &[
                "tobool()",
                "tobool(1, 2)",
                "tobool 42",
                "tobool42",
                "toBool(42)",
                "TOBOOL(42)",
                "ToBool(42)",
                "tobool(",
                "tobool(42",
                "tobool)",
            ],
        );
    }

    #[test]
    fn test_tobool_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("tobool(42)", "integer literal"),
            ("tobool(true)", "boolean literal"),
            ("tobool(4.44)", "float literal"),
            ("tobool(\"true\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::tobool_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_tobool_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Boolean(_conv_expr)) => {
                    println!("Parser correctly created Boolean conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Boolean for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_tofloat_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::tofloat_expression,
            &[
                "tofloat(123)",
                "tofloat(-456)",
                "tofloat(0)",
                "tofloat(1.5)",
                "tofloat(-3.14)",
                "tofloat(true)",
                "tofloat(false)",
                "tofloat(null)",
                "tofloat(\"42.5\")",
                "tofloat('123.456')",
                "tofloat(x)",
                "tofloat(myVariable)",
                "tofloat(_var123)",
                "tofloat((42))",
                "tofloat(iff(true, 1, 0))",
                "tofloat( 42 )",
            ],
            &[
                "tofloat()",
                "tofloat(1, 2)",
                "tofloat 42",
                "tofloat42",
                "toFloat(42)",
                "TOFLOAT(42)",
                "ToFloat(42)",
                "tofloat(",
                "tofloat(42",
                "tofloat)",
            ],
        );
    }

    #[test]
    fn test_tofloat_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("tofloat(42)", "integer literal"),
            ("tofloat(true)", "boolean literal"),
            ("tofloat(4.44)", "float literal"),
            ("tofloat(\"42.5\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::tofloat_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_tofloat_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Double(_conv_expr)) => {
                    println!("Parser correctly created Double conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Double for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_tolong_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::tolong_expression,
            &[
                "tolong(123)",
                "tolong(-456)",
                "tolong(0)",
                "tolong(1.5)",
                "tolong(-3.14)",
                "tolong(true)",
                "tolong(false)",
                "tolong(null)",
                "tolong(\"42\")",
                "tolong('123')",
                "tolong(x)",
                "tolong(myVariable)",
                "tolong(_var123)",
                "tolong((42))",
                "tolong(iff(true, 1, 0))",
                "tolong( 42 )",
            ],
            &[
                "tolong()",
                "tolong(1, 2)",
                "tolong 42",
                "tolong42",
                "toLong(42)",
                "TOLONG(42)",
                "ToLong(42)",
                "tolong(",
                "tolong(42",
                "tolong)",
            ],
        );
    }

    #[test]
    fn test_tolong_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("tolong(42)", "integer literal"),
            ("tolong(true)", "boolean literal"),
            ("tolong(4.44)", "float literal"),
            ("tolong(\"42\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::tolong_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_tolong_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Integer(_conv_expr)) => {
                    println!("Parser correctly created Integer conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Integer for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_toreal_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::toreal_expression,
            &[
                "toreal(123)",
                "toreal(-456)",
                "toreal(0)",
                "toreal(1.5)",
                "toreal(-3.14)",
                "toreal(true)",
                "toreal(false)",
                "toreal(null)",
                "toreal(\"42.5\")",
                "toreal('123.456')",
                "toreal(x)",
                "toreal(myVariable)",
                "toreal(_var123)",
                "toreal((42))",
                "toreal(iff(true, 1, 0))",
                "toreal( 42 )",
            ],
            &[
                "toreal()",
                "toreal(1, 2)",
                "toreal 42",
                "toreal42",
                "toReal(42)",
                "TOREAL(42)",
                "ToReal(42)",
                "toreal(",
                "toreal(42",
                "toreal)",
            ],
        );
    }

    #[test]
    fn test_toreal_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("toreal(42)", "integer literal"),
            ("toreal(true)", "boolean literal"),
            ("toreal(4.44)", "float literal"),
            ("toreal(\"42.5\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::toreal_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_toreal_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Double(_conv_expr)) => {
                    println!("Parser correctly created Double conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Double for: {description}"),
            }
        }
    }

    #[test]
    fn test_pest_parse_todouble_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::todouble_expression,
            &[
                "todouble(123)",
                "todouble(-456)",
                "todouble(0)",
                "todouble(1.5)",
                "todouble(-3.14)",
                "todouble(true)",
                "todouble(false)",
                "todouble(null)",
                "todouble(\"42.5\")",
                "todouble('123.456')",
                "todouble(x)",
                "todouble(myVariable)",
                "todouble(_var123)",
                "todouble((42))",
                "todouble(iff(true, 1, 0))",
                "todouble( 42 )",
            ],
            &[
                "todouble()",
                "todouble(1, 2)",
                "todouble 42",
                "todouble42",
                "toDouble(42)",
                "TODOUBLE(42)",
                "ToDouble(42)",
                "todouble(",
                "todouble(42",
                "todouble)",
            ],
        );
    }

    #[test]
    fn test_todouble_creates_correct_conversion_expression() {
        let test_cases = vec![
            ("todouble(42)", "integer literal"),
            ("todouble(true)", "boolean literal"),
            ("todouble(4.44)", "float literal"),
            ("todouble(\"42.5\")", "string literal"),
        ];

        for (input, description) in test_cases {
            let state = ParserState::new(input);
            let mut parsed = KqlPestParser::parse(Rule::todouble_expression, input)
                .unwrap_or_else(|_| panic!("Failed to parse: {input}"));

            let result = parse_todouble_expression(parsed.next().unwrap(), &state)
                .unwrap_or_else(|_| panic!("Failed to parse expression: {input}"));

            match result {
                ScalarExpression::Convert(ConvertScalarExpression::Double(_conv_expr)) => {
                    println!("Parser correctly created Double conversion AST for {description}");
                }
                _ => panic!("Expected ConvertScalarExpression::Double for: {description}"),
            }
        }
    }
}
