use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "kql.pest"]
#[allow(dead_code)] // Remove this line when the parser is used
pub struct KqlParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_true_token() {
        assert!(KqlParser::parse(Rule::true_token, "true").is_ok());
        assert!(KqlParser::parse(Rule::true_token, "false").is_err());
        assert!(KqlParser::parse(Rule::true_token, "tru").is_err());
    }

    #[test]
    fn test_false_token() {
        assert!(KqlParser::parse(Rule::false_token, "false").is_ok());
        assert!(KqlParser::parse(Rule::false_token, "true").is_err());
        assert!(KqlParser::parse(Rule::false_token, "fals").is_err());
    }

    #[test]
    fn test_integer_token() {
        assert!(KqlParser::parse(Rule::integer_token, "123").is_ok());
        assert!(KqlParser::parse(Rule::integer_token, ".53").is_err());
        assert!(KqlParser::parse(Rule::integer_token, "abc").is_err());
    }

    #[test]
    fn test_string_token() {
        assert!(KqlParser::parse(Rule::string_token, r#""hello""#).is_ok());
        assert!(KqlParser::parse(Rule::string_token, r#""hello"#).is_err());
        assert!(KqlParser::parse(Rule::string_token, r#"hello""#).is_err());
        assert!(KqlParser::parse(Rule::string_token, r#""""#).is_ok());
    }

    #[test]
    fn test_identifier() {
        assert!(KqlParser::parse(Rule::identifier, "abc").is_ok());
        assert!(KqlParser::parse(Rule::identifier, "abc_123").is_ok());
        assert!(KqlParser::parse(Rule::identifier, "_abc").is_ok());
        assert!(KqlParser::parse(Rule::identifier, "123abc").is_ok());
    }

    #[test]
    fn test_comparison_expression() {
        assert!(KqlParser::parse(Rule::comparison_expression, "x == 42").is_ok());
        assert!(KqlParser::parse(Rule::comparison_expression, r#"x != "hello""#).is_ok());
        assert!(KqlParser::parse(Rule::comparison_expression, "x = 42").is_err());
        assert!(KqlParser::parse(Rule::comparison_expression, "== 42").is_err());

        assert!(KqlParser::parse(Rule::comparison_expression, "x > 5").is_ok());
        assert!(KqlParser::parse(Rule::comparison_expression, "x < 5").is_ok());
        assert!(KqlParser::parse(Rule::comparison_expression, "y <= 10").is_ok());
        assert!(KqlParser::parse(Rule::comparison_expression, "y >= 10").is_ok());
    }

    #[test]
    fn test_binary_logical_expression() {
        assert!(KqlParser::parse(Rule::binary_logical_expression, "a and b").is_ok());
        assert!(KqlParser::parse(Rule::binary_logical_expression, "a and").is_err());
        assert!(KqlParser::parse(Rule::binary_logical_expression, "a or b").is_ok());
        assert!(KqlParser::parse(Rule::binary_logical_expression, "or b").is_err());

        assert!(
            KqlParser::parse(Rule::binary_logical_expression, "(a and b) or (c and d)").is_ok()
        );
        assert!(KqlParser::parse(Rule::binary_logical_expression, "a or (b and c)").is_ok());
        assert!(KqlParser::parse(Rule::binary_logical_expression, "a and b and c").is_ok());
    }

    #[test]
    fn test_negated_expression() {
        assert!(KqlParser::parse(Rule::negated_expression, "not(a)").is_ok());
        assert!(KqlParser::parse(Rule::negated_expression, "not(a and b)").is_ok());
        assert!(KqlParser::parse(Rule::negated_expression, "not a").is_err());
        assert!(KqlParser::parse(Rule::negated_expression, "!a").is_err());
    }

    #[test]
    fn test_expression() {
        assert!(KqlParser::parse(Rule::expression, "(a and b)").is_ok());
        assert!(KqlParser::parse(Rule::expression, "a and b").is_ok());
        assert!(KqlParser::parse(Rule::expression, "x == 42").is_ok());
        assert!(KqlParser::parse(Rule::expression, "my_variable").is_ok());
        assert!(KqlParser::parse(Rule::expression, "42").is_ok());
        assert!(KqlParser::parse(Rule::expression, "(y > 24) and z == 10").is_ok());
    }

    #[test]
    fn test_filter_statement() {
        assert!(KqlParser::parse(Rule::filter_statement, "where x == 42").is_ok());
        assert!(KqlParser::parse(Rule::filter_statement, r#"where x != "hello""#).is_ok());
        assert!(KqlParser::parse(Rule::filter_statement, "where x == 42 and y > 24").is_ok());
        assert!(KqlParser::parse(Rule::filter_statement, "where x == 42 or not(y > 24)").is_ok());
        assert!(KqlParser::parse(Rule::filter_statement, "where x = 42").is_err());
        assert!(KqlParser::parse(Rule::filter_statement, "x == 42").is_err());
    }

    #[test]
    fn test_extend_statement() {
        assert!(KqlParser::parse(Rule::extend_statement, "extend x = 42").is_ok());
        assert!(KqlParser::parse(Rule::extend_statement, "extend x=42").is_ok());
        assert!(KqlParser::parse(Rule::extend_statement, r#"extend y = "hello""#).is_ok());
        assert!(KqlParser::parse(Rule::extend_statement, "extend x = 42, y = 24").is_ok());
        assert!(KqlParser::parse(Rule::extend_statement, "extend x = y > 24").is_ok());
        assert!(
            KqlParser::parse(Rule::extend_statement, "extend x = (y > 24) and z == 10").is_ok()
        );
        assert!(KqlParser::parse(Rule::extend_statement, "extend x == 42").is_err());
        assert!(KqlParser::parse(Rule::extend_statement, "x = 42").is_err());
    }

    #[test]
    fn test_pipe_token() {
        assert!(KqlParser::parse(Rule::pipe_token, "|").is_ok());
        assert!(KqlParser::parse(Rule::pipe_token, "||").is_err());
        assert!(KqlParser::parse(Rule::pipe_token, " | ").is_err());
    }

    #[test]
    fn test_statement() {
        assert!(KqlParser::parse(Rule::statement, "| where x == 42").is_ok());
        assert!(KqlParser::parse(Rule::statement, r#"| extend y = "hello""#).is_ok());
        assert!(KqlParser::parse(Rule::statement, "| x == 42").is_err());
        assert!(KqlParser::parse(Rule::statement, "| extend x == 42").is_err());
        assert!(KqlParser::parse(Rule::statement, "| where x = 42").is_err());
        assert!(KqlParser::parse(Rule::statement, "extend x = 42").is_err());
        assert!(KqlParser::parse(Rule::statement, "|| extend x = 42").is_err());
    }

    #[test]
    fn test_query() {
        assert!(KqlParser::parse(Rule::query, "my_table").is_ok());
        assert!(KqlParser::parse(Rule::query, "my_table | where x == 42").is_ok());
        assert!(KqlParser::parse(
            Rule::query,
            r#"my_table | extend y = "hello" | where y != "world""#
        )
        .is_ok());
        assert!(KqlParser::parse(Rule::query, "my_table | where x = 42").is_err());
        assert!(KqlParser::parse(Rule::query, "my_table | extend x == 42").is_err());
        assert!(KqlParser::parse(Rule::query, "my_table | extend x = 42 | where x == 42").is_ok());
        assert!(KqlParser::parse(Rule::query, "my_table | extend x = 42 | x == 42").is_err());
        assert!(KqlParser::parse(Rule::query, "my_table where x == 42").is_err());
        let multiline_query = r#"my_table
            | where x == 42
            | extend y = "hello"
            | where y != "world""#;
        assert!(KqlParser::parse(Rule::query, multiline_query).is_ok());
    }
}
