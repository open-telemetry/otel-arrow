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
}
