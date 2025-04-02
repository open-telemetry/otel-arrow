use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "ottl.pest"]
#[allow(dead_code)] // Remove this line when the parser is used
pub struct OttlParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_true_token() {
        assert!(OttlParser::parse(Rule::true_token, "true").is_ok());
        assert!(OttlParser::parse(Rule::true_token, "false").is_err());
        assert!(OttlParser::parse(Rule::true_token, "tru").is_err());
    }

    #[test]
    fn test_false_token() {
        assert!(OttlParser::parse(Rule::false_token, "false").is_ok());
        assert!(OttlParser::parse(Rule::false_token, "true").is_err());
        assert!(OttlParser::parse(Rule::false_token, "fals").is_err());
    }

    #[test]
    fn test_integer_token() {
        assert!(OttlParser::parse(Rule::integer_token, "123").is_ok());
        assert!(OttlParser::parse(Rule::integer_token, ".53").is_err());
        assert!(OttlParser::parse(Rule::integer_token, "abc").is_err());
    }

    #[test]
    fn test_string_token() {
        assert!(OttlParser::parse(Rule::string_token, r#""hello""#).is_ok());
        assert!(OttlParser::parse(Rule::string_token, r#""hello"#).is_err());
        assert!(OttlParser::parse(Rule::string_token, r#"hello""#).is_err());
        assert!(OttlParser::parse(Rule::string_token, r#""""#).is_ok());
    }

    #[test]
    fn test_identifier() {
        assert!(OttlParser::parse(Rule::identifier, "abc").is_ok());
        assert!(OttlParser::parse(Rule::identifier, "abc_123").is_ok());
        assert!(OttlParser::parse(Rule::identifier, "_abc").is_ok());
        assert!(OttlParser::parse(Rule::identifier, "123abc").is_ok());
    }

    #[test]
    fn test_comparison_expression() {
        assert!(OttlParser::parse(Rule::comparison_expression, "x == 42").is_ok());
        assert!(OttlParser::parse(Rule::comparison_expression, r#"x != "hello""#).is_ok());
        assert!(OttlParser::parse(Rule::comparison_expression, "x = 42").is_err());
        assert!(OttlParser::parse(Rule::comparison_expression, "== 42").is_err());

        assert!(OttlParser::parse(Rule::comparison_expression, "x > 5").is_ok());
        assert!(OttlParser::parse(Rule::comparison_expression, "x < 5").is_ok());
        assert!(OttlParser::parse(Rule::comparison_expression, "y <= 10").is_ok());
        assert!(OttlParser::parse(Rule::comparison_expression, "y >= 10").is_ok());
    }

    #[test]
    fn test_boolean_expression() {
        assert!(OttlParser::parse(Rule::boolean_expression, "a and b").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "a and").is_err());
        assert!(OttlParser::parse(Rule::boolean_expression, "a or b").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "or b").is_err());
        assert!(OttlParser::parse(Rule::boolean_expression, "not(a)").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "not a").is_err());
        assert!(OttlParser::parse(Rule::boolean_expression, "!a").is_err());

        assert!(OttlParser::parse(Rule::boolean_expression, "not(a and b)").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "(a and b) or (c and d)").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "a or (b and c)").is_ok());
        assert!(OttlParser::parse(Rule::boolean_expression, "a and b and c").is_ok());
    }

    #[test]
    fn test_expression() {
        assert!(OttlParser::parse(Rule::expression, "(a and b)").is_ok());
        assert!(OttlParser::parse(Rule::expression, "a and b").is_ok());
        assert!(OttlParser::parse(Rule::expression, "x == 42").is_ok());
        assert!(OttlParser::parse(Rule::expression, "my_variable").is_ok());
        assert!(OttlParser::parse(Rule::expression, "42").is_ok());
        assert!(OttlParser::parse(Rule::expression, "(y > 24) and z == 10").is_ok());
    }

    #[test]
    fn test_filter_expression() {
        assert!(OttlParser::parse(Rule::filter_expression, "- 'x == 42'").is_ok());
        assert!(OttlParser::parse(Rule::filter_expression, r#"- 'x != "hello"'"#).is_ok());
        assert!(OttlParser::parse(Rule::filter_expression, "- 'x == 42 and y > 24'").is_ok());
        assert!(OttlParser::parse(Rule::filter_expression, "- 'x == 42 or not(y > 24)'").is_ok());
        assert!(OttlParser::parse(Rule::filter_expression, "- 'x = 42").is_err());
        assert!(OttlParser::parse(Rule::filter_expression, "- x = 42").is_err());
        assert!(OttlParser::parse(Rule::filter_expression, "'x == 42'").is_err());
    }

    #[test]
    fn test_extend_expression() {
        assert!(OttlParser::parse(Rule::extend_expression, "- set(x, 42)").is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, "- set(x,42)").is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, r#"- set(y, "hello")"#).is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, "- set (x, y > 24)").is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, "- set (x, (y > 24))").is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, "- set(x, 42) where y > 24").is_ok());
        assert!(OttlParser::parse(Rule::extend_expression, "- set(x, 42) where (y > 24)").is_ok());
        assert!(
            OttlParser::parse(Rule::extend_expression, "- set(x, y > 24) where z == 10").is_ok()
        );
        assert!(
            OttlParser::parse(Rule::extend_expression, "- set(x, y > 24 and (z == 10))").is_ok()
        );
        assert!(
            OttlParser::parse(Rule::extend_expression, "- set(x, (y > 24) and z == 10)").is_ok()
        );
        assert!(OttlParser::parse(Rule::extend_expression, "set(x, 42").is_err());
        assert!(OttlParser::parse(Rule::extend_expression, "- set(x = 42").is_err());
        assert!(OttlParser::parse(Rule::extend_expression, "- (x, 42").is_err());
    }

    #[test]
    fn test_dash_token() {
        assert!(OttlParser::parse(Rule::dash_token, "-").is_ok());
        assert!(OttlParser::parse(Rule::dash_token, "--").is_err());
        assert!(OttlParser::parse(Rule::dash_token, " - ").is_err());
    }

    #[test]
    fn test_filter_query() {
        assert!(
            OttlParser::parse(Rule::filter_query, "filter: logs: log_record: - 'x == 42'").is_ok()
        );
        assert!(
            OttlParser::parse(Rule::filter_query, "filter: logs: log_record: - 'x = 42'").is_err()
        );
        assert!(OttlParser::parse(
            Rule::filter_query,
            "filter: logs: log_record: - 'x == 42' - 'y == true'"
        )
        .is_ok());
        let multiline_query = r#"filter:
        logs:
        log_record:
        - 'x == 5'
        - 'y == true'"#;
        assert!(OttlParser::parse(Rule::filter_query, multiline_query).is_ok());
    }

    #[test]
    fn test_transform_query() {
        assert!(OttlParser::parse(
            Rule::transform_query,
            "transform: log_statements: - set(x, 42)"
        )
        .is_ok());
        assert!(OttlParser::parse(
            Rule::transform_query,
            "transform: log_statements: - set(x = 42)"
        )
        .is_err());
        assert!(OttlParser::parse(
            Rule::transform_query,
            r#"transform: log_statements: - set(x, 42) where someColumn == "blue""#
        )
        .is_ok());
        let multiline_query = r#"transform:
        log_statements:
        - set(x, 42)
        - set(y, 42) where someColumn == "blue""#;
        assert!(OttlParser::parse(Rule::transform_query, multiline_query).is_ok());
    }
}
