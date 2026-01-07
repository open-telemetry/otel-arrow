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

    #[test]
    fn test() {
        // TODO fill in this test suite
    }
}
