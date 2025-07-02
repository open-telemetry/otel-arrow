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


    #[test]
    fn test_integer_literal() {
        pest_test_helpers::test_pest_rule::<OttlParser, Rule>(
            Rule::integer_literal,
            &["123", "-123", "+123", "0"],
            &[".53", "abc", "12.3"],
        );
    }

    #[test]
    fn test_float_literal() {
        pest_test_helpers::test_pest_rule::<OttlParser, Rule>(
            Rule::float_literal,
            &["1.0", "-1.0", "+1.0", ".5", "-.5", "+.5", "123.456"],
            &["1", "abc", "1.", "."],
        );
    }

    #[test]
    fn test_string_literal() {
        pest_test_helpers::test_pest_rule::<OttlParser, Rule>(
            Rule::string_literal,
            &[
                "\"hello\"",
                "\"\"",
                "\"hello world\"",
                "\"he\\\"llo\"",
                "\"he\\nllo\"",
            ],
            &["hello", "'hello'", "\"hello", "hello\""],
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

    #[test]
    fn test_parse_integer_literal() {
        parse_test_helpers::test_parse_integer_literal::<OttlParser, Rule>(
            Rule::integer_literal,
            &[("123", 123), ("-123", -123), ("+123", 123), ("0", 0)],
        );
    }

    #[test]
    fn test_parse_float_literal() {
        parse_test_helpers::test_parse_float_literal::<OttlParser, Rule>(
            Rule::float_literal,
            &[
                ("1.0", 1.0),
                ("-1.0", -1.0),
                ("+1.0", 1.0),
                (".5", 0.5),
                ("-.5", -0.5),
                ("+.5", 0.5),
                ("123.456", 123.456),
            ],
        );
    }

    #[test]
    fn test_parse_string_literal() {
        parse_test_helpers::test_parse_string_literal::<OttlParser, Rule>(
            Rule::string_literal,
            &[
                ("\"hello\"", "hello"),
                ("\"\"", ""),
                ("\"hello world\"", "hello world"),
                ("\"he\\\"llo\"", "he\"llo"),
                ("\"he\\nllo\"", "he\nllo"),
            ],
        );
    }
}
