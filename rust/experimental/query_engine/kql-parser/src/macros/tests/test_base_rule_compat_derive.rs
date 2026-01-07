use data_engine_kql_parser_macros::BaseRuleCompatible;

#[derive(pest_derive::Parser, BaseRuleCompatible)]
#[grammar = "../../base.pest"]
#[grammar_inline = "macro_test_rule = {
    \"test_rule\"
}"]
struct TestParser;

#[cfg(test)]
mod test {
    use super::*;
    use data_engine_kql_parser::{ScalarExprRules, base_parser::TryAsBaseRule};
    use pest::Parser;

    #[test]
    fn test_rule_convert() {
        // check that we generated the TryFrom impl for base rules
        let rule = TestParser::parse(Rule::logical_expression, "1 == 1")
            .expect("should parse test_rule")
            .next()
            .expect("one rule");
        assert!(rule.try_as_base_rule().is_ok());

        // check that custom rule can be converted to base rule
        let rule = TestParser::parse(Rule::macro_test_rule, "test_rule")
            .expect("should parse test_rule")
            .next()
            .expect("one rule");

        assert!(rule.try_as_base_rule().is_err());
    }

    #[test]
    fn test_derives_pratt_parser() {
        // check that we can parse a scalar expression using the pratt parser
        _ = Rule::pratt_parser();
    }
}
