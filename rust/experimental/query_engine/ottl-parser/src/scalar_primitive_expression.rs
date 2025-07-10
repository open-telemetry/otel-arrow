#[cfg(test)]
mod pest_tests {
    use crate::{OttlPestParser, Rule};
    use data_engine_parser_abstractions::*;

    #[test]
    fn test_pest_string_literal() {
        pest_test_helpers::test_pest_rule::<OttlPestParser, Rule>(
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

    #[test]
    fn test_pest_true_literal() {
        pest_test_helpers::test_pest_rule::<OttlPestParser, Rule>(
            Rule::true_literal,
            &["true"],
            &["True", "TRUE", "tru", "tRuE", "false"],
        );
    }

    #[test]
    fn test_pest_false_literal() {
        pest_test_helpers::test_pest_rule::<OttlPestParser, Rule>(
            Rule::false_literal,
            &["false"],
            &["False", "FALSE", "fals", "fAlSe", "true"],
        );
    }

    #[test]
    fn test_pest_integer_literal() {
        pest_test_helpers::test_pest_rule::<OttlPestParser, Rule>(
            Rule::integer_literal,
            &["123", "-123", "+123", "0"],
            &[".53", "abc", "12.3"],
        );
    }

    #[test]
    fn test_pest_float_literal() {
        pest_test_helpers::test_pest_rule::<OttlPestParser, Rule>(
            Rule::float_literal,
            &["1.0", "-1.0", "+1.0", ".5", "-.5", "+.5", "123.456"],
            &["1", "abc", "1.", "."],
        );
    }
}

#[cfg(test)]
mod parse_tests {
    use crate::{OttlPestParser, Rule};
    use data_engine_parser_abstractions::*;

    #[test]
    fn test_parse_string_literal() {
        parse_test_helpers::test_parse_string_literal::<OttlPestParser, Rule>(
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

    #[test]
    fn test_parse_bool_literal() {
        parse_test_helpers::test_parse_bool_literal::<OttlPestParser, Rule>(
            Rule::true_literal,
            Rule::false_literal,
            &[("true", true), ("false", false)],
        );
    }

    #[test]
    fn test_parse_integer_literal() {
        parse_test_helpers::test_parse_integer_literal::<OttlPestParser, Rule>(
            Rule::integer_literal,
            &[("123", 123), ("-123", -123), ("+123", 123), ("0", 0)],
        );
    }

    #[test]
    fn test_parse_float_literal() {
        parse_test_helpers::test_parse_double_literal::<OttlPestParser, Rule>(
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
}
