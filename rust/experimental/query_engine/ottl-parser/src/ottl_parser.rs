use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ottl.pest"]
#[allow(dead_code)]
pub(crate) struct OttlParser;

#[cfg(test)]
mod pest_tests {
    use super::*;
    use data_engine_parser_generics::generic_pest_helpers::*;

    #[test]
    fn test_true_literal() {
        test_generic_true_literal::<OttlParser, Rule>(Rule::true_literal);
    }

    #[test]
    fn test_false_literal() {
        test_generic_false_literal::<OttlParser, Rule>(Rule::false_literal);
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use data_engine_parser_generics::generic_parse_helpers::*;

    #[test]
    fn test_parse_bool_literal() {
        test_generic_parse_bool_literal::<OttlParser, Rule>(
            Rule::true_literal,
            Rule::false_literal,
        );
    }
}
