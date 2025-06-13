use crate::parser_abstractions::*;
use data_engine_expressions::*;
use pest::RuleType;

/// Test helper to validate pest rules across different parsers
pub mod pest_test_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_pest_rule<P, R>(parser_rule: R, ok_inputs: &[&str], err_inputs: &[&str])
    where
        P: Parser<R>,
        R: RuleType,
    {
        for input in ok_inputs {
            assert!(
                P::parse(parser_rule, input).is_ok(),
                "Expected Ok for input: {}",
                input
            );
        }
        for input in err_inputs {
            assert!(
                P::parse(parser_rule, input).is_err(),
                "Expected Err for input: {}",
                input
            );
        }
    }
}

/// Test helper to validate parsing of common expressions across different
/// parsers that use generic parse functions.
pub mod parse_test_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_parse_bool_literal<P, R>(
        true_parser_rule: R,
        false_parser_rule: R,
        inputs: &[(&str, bool)],
    ) where
        P: Parser<R>,
        R: RuleType,
    {
        let run_test = |input: &str, expected: bool, rule: R| {
            let mut result = P::parse(rule, input).unwrap();
            let pair = result.next().unwrap();
            let expr = parse_standard_bool_literal(pair);

            match expr {
                StaticScalarExpression::Boolean(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type returned from parse_bool_literal"),
            }
        };

        for (input, expected) in inputs {
            if *expected {
                run_test(input, *expected, true_parser_rule);
            } else {
                run_test(input, *expected, false_parser_rule);
            }
        }
    }
}
