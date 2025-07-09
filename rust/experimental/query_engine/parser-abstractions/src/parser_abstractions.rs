use data_engine_expressions::*;
use pest::RuleType;
use pest::iterators::Pair;

use crate::ParserError;

/// Use a Pest rule to create a `QueryLocation`
/// This function extracts the start and end positions, as well as line and column numbers, from a Pest rule pair.
/// Mostly helps with debugging and error reporting.
pub fn to_query_location<R: RuleType>(rule: &Pair<R>) -> QueryLocation {
    let s = rule.as_span();
    let (line_number, column_number) = rule.line_col();
    QueryLocation::new(s.start(), s.end(), line_number, column_number)
        .expect("QueryLocation could not be constructed")
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
        _ => {
            panic!("Unexpected rule in bool_literal_rule: {bool_literal_rule} (rule: {rule_name})")
        }
    };

    StaticScalarExpression::Boolean(BooleanScalarExpression::new(query_location, value))
}

/// Parses an integer literal from a Pest rule pair and returns a `StaticScalarExpression` wrapping an `IntegerScalarExpression`.
/// Internally represents integers as i64.
pub fn parse_standard_integer_literal<R: RuleType>(
    integer_literal_rule: Pair<R>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&integer_literal_rule);

    let raw_value = integer_literal_rule.as_str();
    let parsed_value = raw_value.parse::<i64>();
    if parsed_value.is_err() {
        return Err(ParserError::SyntaxError(
            to_query_location(&integer_literal_rule),
            format!("'{raw_value}' could not be parsed as a literal of type 'signed 64 bit integer'"),
        ));
    }

    Ok(StaticScalarExpression::Integer(
        IntegerScalarExpression::new(query_location, parsed_value.unwrap()),
    ))
}

/// Parses a float literal from a Pest rule pair and returns a `StaticScalarExpression` wrapping a `DoubleScalarExpression`.
/// Internally represents floats as f64.
pub fn parse_standard_double_literal<R: RuleType>(
    double_literal_rule: Pair<R>,
    type_override: Option<&str>,
) -> Result<StaticScalarExpression, ParserError> {
    let error_type = type_override.unwrap_or("double");
    let query_location = to_query_location(&double_literal_rule);

    let raw_value = double_literal_rule.as_str();
    let parsed_value = raw_value.parse::<f64>();
    if parsed_value.is_err() {
        return Err(ParserError::SyntaxError(
            to_query_location(&double_literal_rule),
            // Some languages may use different terminology for 'doubles' ('float' being an example).
            // Allow callers to specify the expected type name for error messages.
            format!("'{raw_value}' could not be parsed as a literal of type '{error_type}'"),
        ));
    }

    Ok(StaticScalarExpression::Double(DoubleScalarExpression::new(
        query_location,
        parsed_value.unwrap(),
    )))
}

/// Parses a string literal from a Pest rule pair and returns a `StaticScalarExpression` wrapping a `StringScalarExpression`.
/// Handles basic escape sequences: \", \\, \n, \r, \t
pub fn parse_standard_string_literal<R: RuleType>(
    string_literal_rule: Pair<R>,
) -> StaticScalarExpression {
    let query_location = to_query_location(&string_literal_rule);

    let raw_string = string_literal_rule.as_str();
    let mut chars = raw_string.chars();
    let mut s = String::with_capacity(raw_string.len());
    let mut position = 1;
    let mut last_char = '\0';

    let mut c = chars.next();
    loop {
        if c.is_none() {
            break;
        }

        let mut current_char = c.unwrap();
        let mut skip_push = false;

        if position == 1 || current_char == '\\' {
            skip_push = true;
        } else if last_char == '\\' {
            match current_char {
                '"' => current_char = '"',
                '\\' => current_char = '\\',
                'n' => current_char = '\n',
                'r' => current_char = '\r',
                't' => current_char = '\t',
                _ => panic!("Unexpected escape character"),
            }
        }

        last_char = current_char;
        position += 1;

        c = chars.next();
        if c.is_none() {
            break;
        }

        if !skip_push {
            s.push(current_char);
        }
    }

    StaticScalarExpression::String(StringScalarExpression::new(query_location, s.as_str()))
}
