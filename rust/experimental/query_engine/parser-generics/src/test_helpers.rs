use data_engine_expressions::*;
use pest::RuleType;
use crate::parser_generics::*;

/// Test helpers to validate common pest rules across different parsers that
/// adhere to common conventions for certain pest rules.
pub mod generic_pest_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_generic_true_literal<P, R>(parser_true_rule: R)
    where
        P: Parser<R>,
        R: RuleType,
    {
        assert!(P::parse(parser_true_rule, "true").is_ok());
        assert!(P::parse(parser_true_rule, "True").is_ok());
        assert!(P::parse(parser_true_rule, "TRUE").is_ok());
        assert!(P::parse(parser_true_rule, "tRuE").is_err());
        assert!(P::parse(parser_true_rule, "false").is_err());
        assert!(P::parse(parser_true_rule, "tru").is_err());
    }

    pub fn test_generic_false_literal<P, R>(parser_false_rule: R)
    where
        P: Parser<R>,
        R: RuleType,
    {
        assert!(P::parse(parser_false_rule, "false").is_ok());
        assert!(P::parse(parser_false_rule, "False").is_ok());
        assert!(P::parse(parser_false_rule, "FALSE").is_ok());
        assert!(P::parse(parser_false_rule, "fAlSe").is_err());
        assert!(P::parse(parser_false_rule, "true").is_err());
        assert!(P::parse(parser_false_rule, "fals").is_err());
    }
}

/// Test helpers to validate parsing of common expressions across different
/// parsers that use generic parse functions.
pub mod generic_parse_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_generic_parse_bool_literal<P, R>(
        parser_true_rule: R,
        parser_false_rule: R,
    ) where
        P: Parser<R>,
        R: RuleType,
    {
        let run_test = |input: &str, expected: bool, rule: R| {
            let mut result = P::parse(rule, input).unwrap();
            let pair = result.next().unwrap();
            let expr = generic_parse_bool_literal(pair);

            match expr {
                StaticScalarExpression::Boolean(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type returned from parse_bool_literal"),
            }
        };

        run_test("true", true, parser_true_rule);
        run_test("True", true, parser_true_rule);
        run_test("TRUE", true, parser_true_rule);
        run_test("false", false, parser_false_rule);
        run_test("False", false, parser_false_rule);
        run_test("FALSE", false, parser_false_rule);
    }
}