use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ottl.pest"]
#[allow(dead_code)]
pub(crate) struct OttlParser;

#[cfg(test)]
mod pest_tests {
    use super::*;
    use data_engine_parser_abstractions::pest_test_helpers;

    #[test]
    fn test_true_literal() {
        pest_test_helpers::test_pest_rule::<OttlParser, Rule>(
            Rule::true_literal,
            &["true"],
            &["True", "TRUE", "tru", "tRuE", "false"],
        );
    }

    #[test]
    fn test_false_literal() {
        pest_test_helpers::test_pest_rule::<OttlParser, Rule>(
            Rule::false_literal,
            &["false"],
            &["False", "FALSE", "fals", "fAlSe", "true"],
        );
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use data_engine_parser_abstractions::parse_test_helpers;

    #[test]
    fn test_parse_bool_literal() {
        parse_test_helpers::test_parse_bool_literal::<OttlParser, Rule>(
            Rule::true_literal,
            Rule::false_literal,
            &[("true", true), ("false", false)],
        );
    }
}
