use data_engine_expressions::*;
use pest::RuleType;
use crate::parser_generics::*;

// Common test helpers to validate common pest rules across different parsers
pub mod pest_test_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_true_literal_generic<P, R>(parser_true_rule: R)
    where
        P: Parser<R>,
        R: RuleType + std::fmt::Debug + Clone + PartialEq,
    {
        assert!(P::parse(parser_true_rule.clone(), "true").is_ok());
        assert!(P::parse(parser_true_rule.clone(), "True").is_ok());
        assert!(P::parse(parser_true_rule.clone(), "TRUE").is_ok());
        assert!(P::parse(parser_true_rule.clone(), "tRuE").is_err());
        assert!(P::parse(parser_true_rule.clone(), "false").is_err());
        assert!(P::parse(parser_true_rule, "tru").is_err());
    }

    pub fn test_false_literal_generic<P, R>(parser_false_rule: R)
    where
        P: Parser<R>,
        R: RuleType + std::fmt::Debug + Clone + PartialEq,
    {
        assert!(P::parse(parser_false_rule.clone(), "false").is_ok());
        assert!(P::parse(parser_false_rule.clone(), "False").is_ok());
        assert!(P::parse(parser_false_rule.clone(), "FALSE").is_ok());
        assert!(P::parse(parser_false_rule.clone(), "fAlSe").is_err());
        assert!(P::parse(parser_false_rule.clone(), "true").is_err());
        assert!(P::parse(parser_false_rule, "fals").is_err());
    }
}

// Common test helpers to validate parsing of common expressions across different parsers
pub mod parse_tests_helpers {
    use super::*;
    use pest::Parser;

    pub fn test_parse_bool_literal_generic<P, R>(
        parser_true_rule: R,
        parser_false_rule: R,
    ) where
        P: Parser<R>,
        R: RuleType + std::fmt::Debug + Clone + PartialEq,
    {
        let run_test = |input: &str, expected: bool, rule: R| {
            let mut result = P::parse(rule, input).unwrap();
            let pair = result.next().unwrap();
            let expr = parse_bool_literal(pair);

            match expr {
                StaticScalarExpression::Boolean(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type returned from parse_bool_literal"),
            }
        };

        run_test("true", true, parser_true_rule.clone());
        run_test("True", true, parser_true_rule.clone());
        run_test("TRUE", true, parser_true_rule);
        run_test("false", false, parser_false_rule.clone());
        run_test("False", false, parser_false_rule.clone());
        run_test("FALSE", false, parser_false_rule);
    }

}