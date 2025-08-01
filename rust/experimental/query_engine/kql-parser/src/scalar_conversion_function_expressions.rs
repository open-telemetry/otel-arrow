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

    // Create the error once since it's the same for both cases
    let arg_count_error = || ParserError::QueryLanguageDiagnostic {
        location: query_location.clone(),
        diagnostic_id: "KS119",
        message: "The function 'tostring' expects 1 argument.".to_string(),
    };

    // Get exactly one argument
    let scalar_expr_rule = inner.next().ok_or_else(arg_count_error)?;

    // Ensure no extra arguments
    if inner.next().is_some() {
        return Err(arg_count_error());
    }

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
    fn test_tostring_with_integer() {
        let input = "tostring(123)";
        let mut pairs = KqlPestParser::parse(Rule::tostring_expression, input)
            .expect("Failed to parse tostring expression");

        let tostring_pair = pairs.next().unwrap();
        let state = ParserState::new(input);

        let result = parse_tostring_expression(tostring_pair, &state);
        assert!(
            result.is_ok(),
            "Failed to parse tostring(123): {:?}",
            result
        );

        if let Ok(ScalarExpression::Convert(ConvertScalarExpression::String(conv))) = &result {
            
            // Verify the inner expression is a static integer
            if let ScalarExpression::Static(static_expr) = conv.get_inner_expression() {
                
                // Check if it's an integer
                match static_expr {
                    StaticScalarExpression::Integer(int_expr) => {
                        assert_eq!(int_expr.get_value(), 123);
                    }
                    _ => panic!("Expected integer static expression inside tostring()"),
                }
            } else {
                panic!("Expected static expression inside tostring()");
            }
        } else {
            panic!("Expected Convert::String expression");
        }
    }

    #[test]
    fn test_tostring_with_string() {
        let input = r#"tostring("hello")"#;
        let mut pairs = KqlPestParser::parse(Rule::tostring_expression, input)
            .expect("Failed to parse tostring expression");

        let tostring_pair = pairs.next().unwrap();
        let state = ParserState::new(input);

        let result = parse_tostring_expression(tostring_pair, &state);
        assert!(
            result.is_ok(),
            "Failed to parse tostring(\"hello\"): {:?}",
            result
        );
    }

    #[test]
    fn test_tostring_with_boolean() {
        let input = "tostring(true)";
        let mut pairs = KqlPestParser::parse(Rule::tostring_expression, input)
            .expect("Failed to parse tostring expression");

        let tostring_pair = pairs.next().unwrap();
        let state = ParserState::new(input);

        let result = parse_tostring_expression(tostring_pair, &state);
        assert!(
            result.is_ok(),
            "Failed to parse tostring(true): {:?}",
            result
        );
    }

    #[test]
    fn test_tostring_with_null() {
        let input = "tostring(null)";
        let mut pairs = KqlPestParser::parse(Rule::tostring_expression, input)
            .expect("Failed to parse tostring expression");

        let tostring_pair = pairs.next().unwrap();
        let state = ParserState::new(input);

        let result = parse_tostring_expression(tostring_pair, &state);
        assert!(
            result.is_ok(),
            "Failed to parse tostring(null): {:?}",
            result
        );
    }

    #[test]
    fn test_tostring_no_arguments() {
        let input = "tostring()";
        let parse_result = KqlPestParser::parse(Rule::tostring_expression, input);

        // This might fail at the pest level if the grammar requires an argument
        if let Ok(mut pairs) = parse_result {
            let tostring_pair = pairs.next().unwrap();
            let state = ParserState::new(input);

            let result = parse_tostring_expression(tostring_pair, &state);
            assert!(result.is_err(), "Expected error for tostring()");

            if let Err(ParserError::SyntaxError(_, msg)) = result {
                assert!(msg.contains("requires exactly one argument"));
            } else {
                panic!("Expected SyntaxError");
            }
        }
    }

    #[test]
    fn test_tostring_multiple_arguments() {
        let input = "tostring(123, 456)";
        let parse_result = KqlPestParser::parse(Rule::tostring_expression, input);

        // This might fail at the pest level if the grammar enforces single argument
        if let Ok(mut pairs) = parse_result {
            let tostring_pair = pairs.next().unwrap();
            let state = ParserState::new(input);

            let result = parse_tostring_expression(tostring_pair, &state);
            assert!(result.is_err(), "Expected error for tostring(123, 456)");

            if let Err(ParserError::SyntaxError(_, msg)) = result {
                assert!(msg.contains("accepts only one argument"));
            } else {
                panic!("Expected SyntaxError");
            }
        }
    }

    #[test]
    fn test_tostring_nested() {
        let input = "tostring(tostring(123))";
        let mut pairs = KqlPestParser::parse(Rule::tostring_expression, input)
            .expect("Failed to parse nested tostring expression");

        let tostring_pair = pairs.next().unwrap();
        let state = ParserState::new(input);

        let result = parse_tostring_expression(tostring_pair, &state);
        assert!(
            result.is_ok(),
            "Failed to parse nested tostring: {:?}",
            result
        );
    }
}
