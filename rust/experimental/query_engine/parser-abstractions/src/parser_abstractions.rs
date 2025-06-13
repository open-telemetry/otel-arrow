use data_engine_expressions::*;
use pest::RuleType;
use pest::iterators::Pair;

/// Use a Pest rule to create a `QueryLocation`
/// This function extracts the start and end positions, as well as line and column numbers, from a Pest rule pair.
/// Mostly helps with debugging and error reporting.
pub fn to_query_location<R: RuleType>(rule: &Pair<R>) -> QueryLocation {
    let s = rule.as_span();
    let (line_number, column_number) = rule.line_col();
    QueryLocation::new(s.start(), s.end(), line_number, column_number)
}

/// Parses a boolean literal from a Pest rule pair and returns a `StaticScalarExpression` wrapping a `BooleanScalarExpression`.
/// The rule name is required to be either `true_literal` or `false_literal`.
pub fn parse_standard_bool_literal<R: RuleType>(
    bool_literal_rule: Pair<R>,
) -> StaticScalarExpression {
    let query_location = to_query_location(&bool_literal_rule);
    let rule_name = format!("{:?}", bool_literal_rule.as_rule());

    let value = match rule_name.as_str() {
        "true_literal" => true,
        "false_literal" => false,
        _ => panic!(
            "Unexpected rule in bool_literal_rule: {} (rule: {})",
            bool_literal_rule, rule_name
        ),
    };

    StaticScalarExpression::Boolean(BooleanScalarExpression::new(query_location, value))
}
