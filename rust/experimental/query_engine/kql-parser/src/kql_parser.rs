use std::collections::HashSet;

use chrono::{FixedOffset, NaiveDate};
use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::date_utils;

#[derive(Parser)]
#[grammar = "kql.pest"]
#[allow(dead_code)]
pub(crate) struct KqlParser;

/// KQL-specific string literal parser that handles both single and double quotes.
/// Uses the shared standard parser for double quotes but handles single quotes locally
/// since KQL supports `\'` escape sequence which is not part of the OTTL standard.
#[allow(dead_code)]
pub(crate) fn parse_kql_string_literal(string_literal_rule: Pair<Rule>) -> StaticScalarExpression {
    let raw_string = string_literal_rule.as_str();
    
    // If it's a double-quoted string, use the shared parser
    if raw_string.starts_with('"') {
        return parse_standard_string_literal(string_literal_rule);
    }
    
    // Handle single-quoted strings with KQL-specific escape sequences
    let query_location = to_query_location(&string_literal_rule);
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
                '\'' => current_char = '\'',
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







#[allow(dead_code)]
pub(crate) fn parse_datetime_expression(
    datetime_expression_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&datetime_expression_rule);

    let datetime_rule = datetime_expression_rule.into_inner().next().unwrap();

    match datetime_rule.as_rule() {
        Rule::datetime_literal => {
            let original_value = datetime_rule.as_str();
            let mut raw_value: String = original_value.into();

            let date = date_utils::parse_date(&raw_value);
            if date.is_err() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let (month, day, year, range) = date.unwrap();

            raw_value.replace_range(range, "");

            let time = date_utils::parse_time(&raw_value);
            if time.is_err() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let (hour, min, sec, micro, range) = time.unwrap();

            raw_value.replace_range(range, "");

            let offset = date_utils::parse_offset(&raw_value);

            let nd = NaiveDate::from_ymd_opt(year as i32, month, day);
            if nd.is_none() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let ndt = nd.unwrap().and_hms_micro_opt(hour, min, sec, micro);

            if ndt.is_none() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let tz = FixedOffset::east_opt(offset);
            if tz.is_none() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let l = ndt.unwrap().and_local_timezone(tz.unwrap());

            match l {
                chrono::offset::LocalResult::Single(date_time) => {
                    Ok(StaticScalarExpression::DateTime(
                        DateTimeScalarExpression::new(query_location, date_time),
                    ))
                }
                _ => Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                )),
            }
        }
        _ => panic!("Unexpected rule in datetime_expression: {}", datetime_rule),
    }
}

#[allow(dead_code)]
pub(crate) fn parse_real_expression(
    real_expression_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&real_expression_rule);

    let real_rule = real_expression_rule.into_inner().next().unwrap();

    match real_rule.as_rule() {
        Rule::positive_infinity_token => Ok(StaticScalarExpression::Double(
            DoubleScalarExpression::new(query_location, f64::INFINITY),
        )),
        Rule::negative_infinity_token => Ok(StaticScalarExpression::Double(
            DoubleScalarExpression::new(query_location, f64::NEG_INFINITY),
        )),
        Rule::double_literal => parse_standard_float_literal(real_rule),
        Rule::integer_literal => match parse_standard_integer_literal(real_rule)? {
            StaticScalarExpression::Integer(v) => Ok(StaticScalarExpression::Double(
                DoubleScalarExpression::new(query_location, v.get_value() as f64),
            )),
            _ => panic!("Unexpected type returned from parse_integer_literal"),
        },
        _ => panic!("Unexpected rule in real_expression: {}", real_rule),
    }
}

#[allow(dead_code)]
pub(crate) fn parse_scalar_expression(
    scalar_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let scalar_rule = scalar_expression_rule.into_inner().next().unwrap();

    match scalar_rule.as_rule() {
        Rule::real_expression => Ok(ScalarExpression::Static(parse_real_expression(
            scalar_rule,
        )?)),
        Rule::datetime_expression => Ok(ScalarExpression::Static(parse_datetime_expression(
            scalar_rule,
        )?)),
        Rule::logical_expression => Ok(ScalarExpression::Logical(
            parse_logical_expression(scalar_rule, state)?.into(),
        )),
        Rule::true_literal | Rule::false_literal => Ok(ScalarExpression::Static(
            parse_standard_bool_literal(scalar_rule),
        )),
        Rule::double_literal => Ok(ScalarExpression::Static(parse_standard_float_literal(scalar_rule)?)),
        Rule::integer_literal => Ok(ScalarExpression::Static(parse_standard_integer_literal(
            scalar_rule,
        )?)),
        Rule::string_literal => Ok(ScalarExpression::Static(parse_kql_string_literal(scalar_rule))),
        Rule::accessor_expression => parse_accessor_expression(scalar_rule, state),
        Rule::scalar_expression => parse_scalar_expression(scalar_rule, state),
        _ => panic!("Unexpected rule in scalar_expression: {}", scalar_rule),
    }
}

pub(crate) fn parse_comparison_expression(
    comparison_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&comparison_expression_rule);

    let mut comparison_rules = comparison_expression_rule.into_inner();

    let left_rule = comparison_rules.next().unwrap();

    let left = match left_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(left_rule, state)?,
        _ => panic!("Unexpected rule in comparison_expression: {}", left_rule),
    };

    let operation_rule = comparison_rules.next().unwrap();

    let right_rule = comparison_rules.next().unwrap();

    let right = match right_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(right_rule, state)?,
        _ => panic!("Unexpected rule in comparison_expression: {}", right_rule),
    };

    match operation_rule.as_rule() {
        Rule::equals_token => Ok(LogicalExpression::EqualTo(EqualToLogicalExpression::new(
            query_location,
            left,
            right,
        ))),
        Rule::not_equals_token => Ok(LogicalExpression::Not(NotLogicalExpression::new(
            query_location.clone(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(query_location, left, right)),
        ))),

        Rule::greater_than_token => Ok(LogicalExpression::GreaterThan(
            GreaterThanLogicalExpression::new(query_location, left, right),
        )),
        Rule::greater_than_or_equal_to_token => Ok(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(query_location, left, right),
        )),

        Rule::less_than_token => {
            Ok(LogicalExpression::Not(NotLogicalExpression::new(
                query_location.clone(),
                LogicalExpression::GreaterThanOrEqualTo(
                    GreaterThanOrEqualToLogicalExpression::new(query_location, left, right),
                ),
            )))
        }
        Rule::less_than_or_equal_to_token => Ok(LogicalExpression::Not(NotLogicalExpression::new(
            query_location.clone(),
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                query_location,
                left,
                right,
            )),
        ))),

        _ => panic!("Unexpected rule in operation_rule: {}", operation_rule),
    }
}

pub(crate) fn parse_logical_expression(
    logical_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<LogicalExpression, ParserError> {
    let query_location = to_query_location(&logical_expression_rule);

    let mut logical_rules = logical_expression_rule.into_inner();

    let parse_rule =
        |logical_expression_rule: Pair<Rule>| -> Result<LogicalExpression, ParserError> {
            match logical_expression_rule.as_rule() {
                Rule::comparison_expression => {
                    Ok(parse_comparison_expression(logical_expression_rule, state)?)
                }
                Rule::logical_expression => {
                    Ok(parse_logical_expression(logical_expression_rule, state)?)
                }
                Rule::true_literal | Rule::false_literal => Ok(LogicalExpression::Scalar(
                    ScalarExpression::Static(parse_standard_bool_literal(logical_expression_rule)),
                )),
                Rule::accessor_expression => Ok(LogicalExpression::Scalar(
                    parse_accessor_expression(logical_expression_rule, state)?,
                )),
                _ => panic!(
                    "Unexpected rule in logical_expression_rule: {}",
                    logical_expression_rule
                ),
            }
        };

    let first_expression = parse_rule(logical_rules.next().unwrap())?;

    let mut chain_rules = Vec::new();
    loop {
        let rule = logical_rules.next();
        if rule.is_none() {
            break;
        }

        let chain_rule = rule.unwrap();
        match chain_rule.as_rule() {
            Rule::and_token => chain_rules.push(ChainedLogicalExpression::And(
                parse_logical_expression(logical_rules.next().unwrap(), state)?,
            )),
            Rule::or_token => chain_rules.push(ChainedLogicalExpression::Or(
                parse_logical_expression(logical_rules.next().unwrap(), state)?,
            )),
            _ => panic!("Unexpected rule in chain_rule: {}", chain_rule),
        }
    }

    if !chain_rules.is_empty() {
        let mut chain = ChainLogicalExpression::new(query_location, first_expression);

        for expression in chain_rules {
            match expression {
                ChainedLogicalExpression::Or(logical_expression) => {
                    chain.push_or(logical_expression)
                }
                ChainedLogicalExpression::And(logical_expression) => {
                    chain.push_and(logical_expression)
                }
            };
        }

        return Ok(LogicalExpression::Chain(chain));
    }

    Ok(first_expression)
}

/// The goal of this code is to translate a KQL dynamic access
/// (https://learn.microsoft.com/kusto/query/scalar-data-types/dynamic)
/// expression into a valid query scalar expression.
///
/// * `source.attributes.array[0]` -> `Source(MapKey("attributes"),
///   MapKey("array"), ArrayIndex(0))`
/// * `scope.name` -> `Attached("scope", MapKey("name"))`
/// * `var_name` -> `Variable("var_name")`
/// * `source.attributes[-var_name]` -> `Source(MapKey("attributes"),
///   Scalar(Negate(Variable("var_name"))))`
///
/// Special cases:
///
/// * If the root identifier is not `source` or something contained in either
///   attached names nor variables names we assume the user wants some default
///   behavior. This is controlled by `default_source_map_key` on
///   [`ParserState`].
///
///   `unknown` -> `Source(MapKey("attributes"), MapKey("unknown"))`
#[allow(dead_code)]
pub(crate) fn parse_accessor_expression(
    accessor_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&accessor_expression_rule);

    let mut accessor_rules = accessor_expression_rule.into_inner();

    let root_accessor_identity = accessor_rules.next().unwrap().as_str();

    let mut value_accessor = ValueAccessor::new();

    let mut negate_location = None;

    loop {
        let accessor = accessor_rules.next();
        if accessor.is_none() {
            break;
        }

        let pair = accessor.unwrap();
        match pair.as_rule() {
            Rule::integer_literal => {
                let location = to_query_location(&pair);

                match parse_standard_integer_literal(pair)? {
                    StaticScalarExpression::Integer(v) => {
                        let i = v.get_value();

                        if i < i32::MIN as i64 || i > i32::MAX as i64 {
                            return Err(ParserError::SyntaxError(
                                location,
                                format!(
                                    "'{}' value for array index is too large to fit into a 32bit value",
                                    i
                                ),
                            ));
                        }

                        value_accessor.push_selector(ValueSelector::ArrayIndex(v));
                    }
                    _ => panic!("Unexpected type returned from parse_integer_literal"),
                }
            }
            Rule::string_literal => match parse_kql_string_literal(pair) {
                StaticScalarExpression::String(v) => {
                    value_accessor.push_selector(ValueSelector::MapKey(v))
                }
                _ => panic!("Unexpected type returned from parse_string_literal"),
            },
            Rule::identifier_literal => {
                value_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
                    to_query_location(&pair),
                    pair.as_str(),
                )));
            }
            Rule::accessor_expression => {
                let expression = parse_accessor_expression(pair, state)?;

                if negate_location.is_some() {
                    value_accessor.push_selector(ValueSelector::ScalarExpression(
                        ScalarExpression::Negate(NegateScalarExpression::new(
                            negate_location.unwrap(),
                            expression,
                        )),
                    ));
                    negate_location = None;
                } else {
                    value_accessor.push_selector(ValueSelector::ScalarExpression(expression));
                }
            }
            Rule::minus_token => {
                negate_location = Some(to_query_location(&pair));
            }
            _ => panic!("Unexpected rule in accessor_expression: {}", pair),
        }
    }

    if root_accessor_identity == "source" {
        Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )))
    } else if state.attached_data_names.contains(root_accessor_identity) {
        return Ok(ScalarExpression::Attached(AttachedScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
    } else if state.variable_names.contains(root_accessor_identity) {
        return Ok(ScalarExpression::Variable(VariableScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
    } else {
        value_accessor.insert_selector(
            0,
            ValueSelector::MapKey(StringScalarExpression::new(
                query_location.clone(),
                root_accessor_identity,
            )),
        );

        return Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )));
    }
}

#[allow(dead_code)]
pub(crate) fn parse_assignment_expression(
    assignment_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<TransformExpression, ParserError> {
    let query_location = to_query_location(&assignment_expression_rule);

    let mut assignment_rules = assignment_expression_rule.into_inner();

    let destination_rule = assignment_rules.next().unwrap();
    let destination_rule_location = to_query_location(&destination_rule);
    let destination_rule_str = destination_rule.as_str();

    let accessor = match destination_rule.as_rule() {
        Rule::accessor_expression => parse_accessor_expression(destination_rule, state)?,
        _ => panic!(
            "Unexpected rule in assignment_expression: {}",
            destination_rule
        ),
    };

    let destination = match accessor {
        ScalarExpression::Source(s) => MutableValueExpression::Source(s),
        ScalarExpression::Variable(v) => MutableValueExpression::Variable(v),
        _ => {
            return Err(ParserError::SyntaxError(
                destination_rule_location,
                format!(
                    "'{}' destination accessor must refer to source or a variable to be used in an assignment expression",
                    destination_rule_str
                ),
            ));
        }
    };

    let source_rule = assignment_rules.next().unwrap();

    let scalar = match source_rule.as_rule() {
        Rule::scalar_expression => parse_scalar_expression(source_rule, state)?,
        _ => panic!("Unexpected rule in assignment_expression: {}", source_rule),
    };

    Ok(TransformExpression::Set(SetTransformExpression::new(
        query_location,
        ImmutableValueExpression::Scalar(scalar),
        destination,
    )))
}

#[allow(dead_code)]
pub(crate) fn parse_extend_expression(
    extend_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let extend_rules = extend_expression_rule.into_inner();

    let mut set_expressions = Vec::new();

    for rule in extend_rules {
        match rule.as_rule() {
            Rule::assignment_expression => {
                let assignment_expression = parse_assignment_expression(rule, state)?;

                if let TransformExpression::Set(s) = &assignment_expression {
                    match s.get_destination() {
                        MutableValueExpression::Source(_) => {}
                        MutableValueExpression::Variable(v) => {
                            let location = v.get_query_location();

                            return Err(ParserError::SyntaxError(
                                location.clone(),
                                format!(
                                    "'{}' destination accessor must refer to source to be used in an extend expression",
                                    state.get_query_slice(location).trim()
                                ),
                            ));
                        }
                    }
                    set_expressions.push(assignment_expression);
                } else {
                    panic!(
                        "Unexpected transformation in extend_expression: {:?}",
                        assignment_expression
                    )
                }
            }
            _ => panic!("Unexpected rule in extend_expression: {}", rule),
        }
    }

    Ok(set_expressions)
}

#[allow(dead_code)]
pub(crate) fn parse_project_expression(
    project_expression_rule: Pair<Rule>,
    state: &ParserState,
) -> Result<Vec<TransformExpression>, ParserError> {
    let query_location = to_query_location(&project_expression_rule);

    let project_rules = project_expression_rule.into_inner();

    let mut keys_to_keep: HashSet<Box<str>> = HashSet::new();
    let mut expressions = Vec::new();

    for rule in project_rules {
        let rule_location = to_query_location(&rule);

        match rule.as_rule() {
            Rule::assignment_expression => {
                let assignment_expression = parse_assignment_expression(rule, state)?;

                if let TransformExpression::Set(s) = &assignment_expression {
                    match s.get_destination() {
                        MutableValueExpression::Source(s) => {
                            let location = s.get_query_location();
                            let selectors = s.get_selectors();

                            let mut map_key = None;
                            if selectors.len() == 1 {
                                if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
                                    map_key = Some(k.get_value());
                                }
                            } else if selectors.len() == 2 {
                                // Note: If state has default_source_map_key we allow it
                                // to be referenced. For example key2, source.key2,
                                // attributes['key2'], and source.attributes['key2'] may
                                // all refer to the same thing when
                                // default_source_map_key=attributes.
                                if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
                                    let root_key = k.get_value();

                                    if Some(root_key) == state.get_default_source_map_key() {
                                        if let ValueSelector::MapKey(k) = selectors.get(1).unwrap()
                                        {
                                            map_key = Some(k.get_value())
                                        }
                                    }
                                }
                            }

                            if map_key.is_none() {
                                return Err(ParserError::SyntaxError(
                                    location.clone(),
                                    format!(
                                        "The '{}' accessor expression should refer to a top-level map key on the source when used in a project expression",
                                        state.get_query_slice(location).trim()
                                    ),
                                ));
                            }

                            keys_to_keep.insert(map_key.unwrap().into());
                        }
                        MutableValueExpression::Variable(v) => {
                            let location = v.get_query_location();

                            return Err(ParserError::SyntaxError(
                                location.clone(),
                                format!(
                                    "'{}' destination accessor must refer to source to be used in a project expression",
                                    state.get_query_slice(location).trim()
                                ),
                            ));
                        }
                    }
                    expressions.push(assignment_expression);
                } else {
                    panic!(
                        "Unexpected transformation in project_expression: {:?}",
                        assignment_expression
                    )
                }
            }
            Rule::accessor_expression => {
                let accessor_expression = parse_accessor_expression(rule, state)?;

                if let ScalarExpression::Source(s) = accessor_expression {
                    let selectors = s.get_selectors();

                    let mut map_key = None;
                    if selectors.len() == 1 {
                        if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
                            map_key = Some(k.get_value());
                        }
                    } else if selectors.len() == 2 {
                        // Note: If state has default_source_map_key we allow it
                        // to be referenced. For example key2, source.key2,
                        // attributes['key2'], and source.attributes['key2'] may
                        // all refer to the same thing when
                        // default_source_map_key=attributes.
                        if let ValueSelector::MapKey(k) = selectors.first().unwrap() {
                            let root_key = k.get_value();

                            if Some(root_key) == state.get_default_source_map_key() {
                                if let ValueSelector::MapKey(k) = selectors.get(1).unwrap() {
                                    map_key = Some(k.get_value())
                                }
                            }
                        }
                    }

                    if map_key.is_none() {
                        return Err(ParserError::SyntaxError(
                            rule_location.clone(),
                            format!(
                                "The '{}' accessor expression should refer to a top-level map key on the source when used in a project expression",
                                state.get_query_slice(&rule_location).trim()
                            ),
                        ));
                    }

                    keys_to_keep.insert(map_key.unwrap().into());
                } else {
                    return Err(ParserError::SyntaxError(
                        rule_location.clone(),
                        format!(
                            "To be valid in a project expression '{}' should be an assignment expression or an accessor expression which refers to the source",
                            state.get_query_slice(&rule_location).trim()
                        ),
                    ));
                }
            }
            _ => panic!("Unexpected rule in project_expression: {}", rule),
        }
    }

    expressions.push(TransformExpression::Clear(ClearTransformExpression::new(
        query_location.clone(),
        MutableValueExpression::Source(SourceScalarExpression::new(
            query_location,
            ValueAccessor::new(),
        )),
        keys_to_keep,
    )));

    Ok(expressions)
}

#[cfg(test)]
mod pest_tests {
    use super::*;
    use data_engine_parser_abstractions::pest_test_helpers;
    use pest::Parser;

    #[test]
    fn test_true_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::true_literal,
            &["true", "True", "TRUE"],
            &["tru", "tRuE", "false"],
        );
    }

    #[test]
    fn test_false_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::false_literal,
            &["false", "False", "FALSE"],
            &["fals", "fAlSe", "true"],
        );
    }

    #[test]
    fn test_datetime_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::datetime_literal,
            &[
                "12/31/2025",
                "12/31/2025 10 AM",
                "12-31-2025 10:00PM",
                "12-31-2025 13:00:00",
                "2025-12-13 13:00:00 +08:00",
                "November 7, 2025",
                "Nov 7 25",
            ],
            &["", "\"", "'", "\\"],
        );

        // ISO 8601
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::datetime_literal,
            &[
                "2014-05-25T08:20:03.123456Z",
                "2014-11-08 15:55:55.123456Z",
                "2014-11-08 15:55:55",
            ],
            &[],
        );
        // RFC 822
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::datetime_literal,
            &["Sat, 8 Nov 14 15:05:02 GMT", "8 Nov 14 15:05 GMT"],
            &[],
        );
        // RFC 850
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::datetime_literal,
            &["Saturday, 08-Nov-14 15:05:02 GMT", "08-Nov-14 15:05:02 GMT"],
            &[],
        );
        // Sortable
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::datetime_literal,
            &["2014-11-08 15:05:25 GMT", "2014-11-08T15:05:25 GMT"],
            &[],
        );
    }

    #[test]
    fn test_double_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::double_literal,
            &[
                "1.0", "-1.0", "1.0e1", "-1.0e1", "1e1", "-1e1", "1e+1", "1e-1",
            ],
            &["1", ".1", "abc"],
        );
    }

    #[test]
    fn test_integer_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::integer_literal,
            &["123", "-123"],
            &[".53", "abc"],
        );
    }

    #[test]
    fn test_string_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::string_literal,
            &[
                "\"hello\"",
                "\"he\\\"llo\"",
                "'hello'",
                "'he\"llo'",
                "'he\\'llo'",
                r#""""#,
            ],
            &[r#""hello"#, r#"hello""#],
        );
    }

    #[test]
    fn test_identifier_literal() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::identifier_literal,
            &["Abc", "abc_123", "_abc"],
            &[],
        );
    }

    #[test]
    fn test_real_expression() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::real_expression,
            &["real(1.0)", "real(1)", "real(+inf)", "real(-inf)"],
            &["real(.1)", "real()", "real(abc)"],
        );
    }

    #[test]
    fn test_comparison_expression() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::comparison_expression,
            &[
                "1 == 1",
                "(1) != true",
                "(1==1) > false",
                "1 >= 1",
                "1 < 1",
                "1 <= 1",
            ],
            &[],
        );
    }

    #[test]
    fn test_scalar_expression() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::scalar_expression,
            &[
                "1",
                "1e1",
                "real(1)",
                "datetime(6/9/2025)",
                "true",
                "false",
                "(true == true)",
                "\"hello world\"",
                "variable",
            ],
            &["!"],
        );
    }

    #[test]
    fn test_logical_expression() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::logical_expression,
            &[
                "true",
                "false",
                "variable",
                "a == b",
                "a == b or 10 < 1",
                "(true)",
                "(true or variable['a'])",
                "(variable['a'] == 'hello' or variable.b == 'world') and datetime(6/1/2025) > datetime(1/1/2025)",
            ],
            &["1"],
        );
    }

    #[test]
    fn test_accessor_expression() {
        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "Abc").unwrap(),
            &[
                (Rule::accessor_expression, "Abc"),
                (Rule::identifier_literal, "Abc"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "abc_123").unwrap(),
            &[
                (Rule::accessor_expression, "abc_123"),
                (Rule::identifier_literal, "abc_123"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "_abc").unwrap(),
            &[
                (Rule::accessor_expression, "_abc"),
                (Rule::identifier_literal, "_abc"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "array[0]").unwrap(),
            &[
                (Rule::accessor_expression, "array[0]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "0"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "array[-1]").unwrap(),
            &[
                (Rule::accessor_expression, "array[-1]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "-1"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name1.name2").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name1.name2"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name1"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(
                Rule::accessor_expression,
                "abc['~name-!'].name1[0][-sub].name2",
            )
            .unwrap(),
            &[
                (
                    Rule::accessor_expression,
                    "abc['~name-!'].name1[0][-sub].name2",
                ),
                (Rule::identifier_literal, "abc"),
                (Rule::string_literal, "'~name-!'"),
                (Rule::identifier_literal, "name1"),
                (Rule::integer_literal, "0"),
                (Rule::minus_token, "-"),
                (Rule::accessor_expression, "sub"),
                (Rule::identifier_literal, "sub"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::accessor_expression,
            &[],
            &["123", "+name", "-name", "~name", ".name"],
        );
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use chrono::{DateTime, Datelike, NaiveDate, Utc};
    use data_engine_parser_abstractions::parse_test_helpers;
    use pest::Parser;

    #[test]
    fn test_parse_bool_literal() {
        parse_test_helpers::test_parse_bool_literal::<KqlParser, Rule>(
            Rule::true_literal,
            Rule::false_literal,
            &[
                ("true", true),
                ("True", true),
                ("TRUE", true),
                ("false", false),
                ("False", false),
                ("FALSE", false),
            ],
        );
    }

    #[test]
    fn test_parse_double_literal() {
        let run_test = |input: &str, expected: f64| {
            let mut result = KqlParser::parse(Rule::double_literal, input).unwrap();

            let f = parse_standard_float_literal(result.next().unwrap());

            assert!(f.is_ok());

            match f.unwrap() {
                StaticScalarExpression::Double(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_double_literal"),
            }
        };

        run_test("1.0", 1.0);
        run_test("-1.0", -1.0);
        run_test("1.0e10", 1.0e10);
        run_test("-1.0e10", -1.0e10);
        run_test("1e-10", 1e-10);
        run_test("-1e10", -1e10);
        run_test("-1e+10", -1e10);

        run_test("1e+1000", f64::INFINITY);
        run_test("-1e+1000", f64::NEG_INFINITY);
    }

    #[test]
    fn test_parse_integer_literal() {
        let run_test = |input: &str, expected: i64| {
            let mut result = KqlParser::parse(Rule::integer_literal, input).unwrap();

            let i = parse_standard_integer_literal(result.next().unwrap());

            assert!(i.is_ok());

            match i.unwrap() {
                StaticScalarExpression::Integer(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_integer_literal"),
            }
        };

        run_test("1", 1);
        run_test("-1", -1);
    }

    #[test]
    fn test_parse_invalid_integer_literal() {
        let input = format!("{}", i64::MAX as i128 + 1);
        let result = KqlParser::parse(Rule::integer_literal, input.as_str());

        assert!(result.is_ok());

        let mut pairs = result.unwrap();
        let i = parse_standard_integer_literal(pairs.next().unwrap());

        assert!(i.is_err());

        if let ParserError::SyntaxError(_, _) = i.unwrap_err() {
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let run_test = |input: &str, expected: &str| {
            let mut result = KqlParser::parse(Rule::string_literal, input).unwrap();

            let actual = parse_kql_string_literal(result.next().unwrap());

            match actual {
                StaticScalarExpression::String(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_string_literal"),
            }
        };

        run_test("\"Hello world\"", "Hello world");
        run_test("\"Hello \\\" world\"", "Hello \" world");
        run_test("'Hello world'", "Hello world");
        run_test("'Hello \" world'", "Hello \" world");
        run_test("'Hello \\' world'", "Hello ' world");
    }

    #[test]
    fn test_parse_datetime_expression() {
        let run_test = |input: &str, expected: DateTime<FixedOffset>| {
            let mut result = KqlParser::parse(Rule::datetime_expression, input).unwrap();

            let d = parse_datetime_expression(result.next().unwrap());

            assert!(d.is_ok());

            match d.unwrap() {
                StaticScalarExpression::DateTime(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_datetime_expression"),
            }
        };

        let create_utc = |year: i32,
                          month: u32,
                          day: u32,
                          hour: u32,
                          min: u32,
                          sec: u32,
                          micro: u32|
         -> DateTime<FixedOffset> {
            NaiveDate::from_ymd_opt(year, month, day)
                .unwrap()
                .and_hms_micro_opt(hour, min, sec, micro)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap()
                .into()
        };

        let create_fixed = |year: i32,
                            month: u32,
                            day: u32,
                            hour: u32,
                            min: u32,
                            sec: u32,
                            micro: u32,
                            offset: i32|
         -> DateTime<FixedOffset> {
            NaiveDate::from_ymd_opt(year, month, day)
                .unwrap()
                .and_hms_micro_opt(hour, min, sec, micro)
                .unwrap()
                .and_local_timezone(FixedOffset::east_opt(offset).unwrap())
                .unwrap()
        };

        let now = Utc::now();

        run_test("datetime(12/31/2025)", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test("datetime(12/31/50)", create_utc(1950, 12, 31, 0, 0, 0, 0));
        run_test("datetime(12/31/49)", create_utc(2049, 12, 31, 0, 0, 0, 0));
        run_test("datetime(2025/12/31)", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test(
            "datetime(2025/12/31 22:30:10.1)",
            create_utc(2025, 12, 31, 22, 30, 10, 1),
        );
        run_test(
            "datetime(12-31-2025 10AM)",
            create_utc(2025, 12, 31, 10, 0, 0, 0),
        );
        run_test(
            "datetime(2025-12-31 10:30 PM)",
            create_utc(2025, 12, 31, 22, 30, 0, 0),
        );
        run_test(
            "datetime(10PM)",
            create_utc(now.year(), now.month(), now.day(), 22, 0, 0, 0),
        );

        // ISO 8601
        run_test(
            "datetime(2014-05-25T08:20:03.123456Z)",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test(
            "datetime(2009-06-15T13:45:30.0000000-07:00)",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, -7 * 60 * 60),
        );
        run_test(
            "datetime(2009-06-15T13:45:30.0000000+07:30)",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, (7 * 60 * 60) + (30 * 60)),
        );
        run_test(
            "datetime(2014-05-25T08:20:03.123456)",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test(
            "datetime(2014-05-25T08:20)",
            create_utc(2014, 5, 25, 8, 20, 0, 0),
        );
        run_test(
            "datetime(2014-11-08 15:55:55.123456Z)",
            create_utc(2014, 11, 8, 15, 55, 55, 123456),
        );
        run_test(
            "datetime(2014-11-08 15:55:55)",
            create_utc(2014, 11, 8, 15, 55, 55, 0),
        );
        run_test(
            "datetime(2014-11-08 15:55)",
            create_utc(2014, 11, 8, 15, 55, 0, 0),
        );
        run_test("datetime(2014-11-08)", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 822
        run_test(
            "datetime(Sat, 8 Nov 14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(Sat, 8 Nov 14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(8 Nov 14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(8 Nov 14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(8 Nov 14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(8 Nov 14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test("datetime(8 Nov 14)", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 850
        run_test(
            "datetime(Saturday, 08-Nov-14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(Saturday, 08-Nov-14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(Saturday, 08-Nov-14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(Saturday, 08-Nov-14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(08-Nov-14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(08-Nov-14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test(
            "datetime(08-Nov-14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(08-Nov-14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );

        // Sortable
        run_test(
            "datetime(2014-11-08 15:05:25)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test(
            "datetime(2014-11-08 15:05:25 GMT)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test(
            "datetime(2014-11-08 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(2014-11-08 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(2014-11-08T15:05:25)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test(
            "datetime(2014-11-08T15:05:25 GMT)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test(
            "datetime(2014-11-08T15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test(
            "datetime(2014-11-08T15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
    }

    #[test]
    fn test_parse_real_expression() {
        let run_test = |input: &str, expected: f64| {
            let mut result = KqlParser::parse(Rule::real_expression, input).unwrap();

            let r = parse_real_expression(result.next().unwrap());

            assert!(r.is_ok());

            match r.unwrap() {
                StaticScalarExpression::Double(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_real_expression"),
            }
        };

        run_test("real(1.0)", 1.0);
        run_test("real(1.0e10)", 1.0e10);
        run_test("real(1)", 1.0);
        run_test("real(+inf)", f64::INFINITY);
        run_test("real(-inf)", f64::NEG_INFINITY);
    }

    #[test]
    fn test_parse_comparison_expression() {
        let run_test = |input: &str, expected: LogicalExpression| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::comparison_expression, input).unwrap();

            let expression = parse_comparison_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        run_test(
            "(true == true) != true",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Logical(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                        ))
                        .into(),
                    ),
                    ScalarExpression::Static(StaticScalarExpression::Boolean(
                        BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                    )),
                )),
            )),
        );

        let mut source_accessor = ValueAccessor::new();

        source_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "source_path",
        )));

        run_test(
            "1 > source_path",
            LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    source_accessor,
                )),
            )),
        );

        let mut resource_accessor = ValueAccessor::new();

        resource_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "key",
        )));

        run_test(
            "(1) >= resource.key",
            LogicalExpression::GreaterThanOrEqualTo(GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                )),
                ScalarExpression::Attached(AttachedScalarExpression::new(
                    QueryLocation::new_fake(),
                    "resource",
                    resource_accessor,
                )),
            )),
        );

        run_test(
            "0 < (1)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThanOrEqualTo(
                    GreaterThanOrEqualToLogicalExpression::new(
                        QueryLocation::new_fake(),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::Integer(
                            IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
                        )),
                    ),
                ),
            )),
        );

        run_test(
            "0 <= (true == true)",
            LogicalExpression::Not(NotLogicalExpression::new(
                QueryLocation::new_fake(),
                LogicalExpression::GreaterThan(GreaterThanLogicalExpression::new(
                    QueryLocation::new_fake(),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                    )),
                    ScalarExpression::Logical(
                        LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                            ScalarExpression::Static(StaticScalarExpression::Boolean(
                                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
                            )),
                        ))
                        .into(),
                    ),
                )),
            )),
        );
    }

    #[test]
    fn test_parse_logical_expression() {
        let run_test = |input: &str, expected: LogicalExpression| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::logical_expression, input).unwrap();

            let expression = parse_logical_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        run_test(
            "true",
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), true),
            ))),
        );

        run_test(
            "false",
            LogicalExpression::Scalar(ScalarExpression::Static(StaticScalarExpression::Boolean(
                BooleanScalarExpression::new(QueryLocation::new_fake(), false),
            ))),
        );

        let mut source_accessor = ValueAccessor::new();
        source_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "name",
        )));

        run_test(
            "source.name",
            LogicalExpression::Scalar(ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                source_accessor,
            ))),
        );

        let mut resource_accessor = ValueAccessor::new();
        resource_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "attributes",
        )));
        resource_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "service.name",
        )));

        run_test(
            "resource.attributes['service.name']",
            LogicalExpression::Scalar(ScalarExpression::Attached(AttachedScalarExpression::new(
                QueryLocation::new_fake(),
                "resource",
                resource_accessor,
            ))),
        );

        let variable_accessor = ValueAccessor::new();

        run_test(
            "variable",
            LogicalExpression::Scalar(ScalarExpression::Variable(VariableScalarExpression::new(
                QueryLocation::new_fake(),
                "variable",
                variable_accessor.clone(),
            ))),
        );

        run_test(
            "variable == 'hello world'",
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    variable_accessor.clone(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        let mut chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    variable_accessor.clone(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        let mut severity_text_accessor = ValueAccessor::new();

        severity_text_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "SeverityText",
        )));

        chain.push_or(LogicalExpression::Not(NotLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    severity_text_accessor,
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "Info"),
                )),
            )),
        )));

        run_test(
            "variable == 'hello world' or SeverityText != 'Info'",
            LogicalExpression::Chain(chain),
        );

        let mut nested_logical = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::EqualTo(EqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    variable_accessor.clone(),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )),
            )),
        );

        let mut severity_text_accessor = ValueAccessor::new();

        severity_text_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "SeverityNumber",
        )));

        nested_logical.push_or(LogicalExpression::GreaterThanOrEqualTo(
            GreaterThanOrEqualToLogicalExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    severity_text_accessor,
                )),
                ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), 0),
                )),
            ),
        ));

        let mut nested_chain = ChainLogicalExpression::new(
            QueryLocation::new_fake(),
            LogicalExpression::Chain(nested_logical),
        );

        nested_chain.push_and(LogicalExpression::Scalar(ScalarExpression::Static(
            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                QueryLocation::new_fake(),
                true,
            )),
        )));

        run_test(
            "(variable == ('hello world') or (SeverityNumber) >= 0) and (true)",
            LogicalExpression::Chain(nested_chain),
        );
    }

    #[test]
    fn test_parse_accessor_expression_from_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "source.subkey['array'][0]").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new("source.subkey['array'][0]"),
        )
        .unwrap();

        if let ScalarExpression::Source(path) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "subkey"
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "array"
                    )),
                    ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        0
                    ))
                ]
                .to_vec(),
                path.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "subkey[var][-neg_attr]").unwrap();

        let mut state = ParserState::new("subkey[var][-neg_attr]");

        state.push_variable_name("var");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        let mut v = ValueAccessor::new();
        v.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "neg_attr",
        )));

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "subkey"
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            "var",
                            ValueAccessor::new()
                        )
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Negate(
                        NegateScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                v
                            ))
                        )
                    ))
                ]
                .to_vec(),
                s.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source_and_default_map() {
        let mut result = KqlParser::parse(Rule::accessor_expression, "subkey").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new("subkey").with_default_source_map_key_name("attributes"),
        )
        .unwrap();

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[ValueSelector::MapKey(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "subkey"
                ))]
                .to_vec(),
                s.get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_from_attached() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "resource['~at\\'tr~']").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new("resource['~at\\'tr~']").with_attached_data_names(&["resource"]),
        )
        .unwrap();

        if let ScalarExpression::Attached(a) = expression {
            assert_eq!("resource", a.get_name());
            assert_eq!(
                &[ValueSelector::MapKey(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "~at'tr~"
                ))]
                .to_vec(),
                a.get_selectors()
            );
        } else {
            panic!("Expected AttachedScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_from_variable() {
        let mut result = KqlParser::parse(Rule::accessor_expression, "a[-1]").unwrap();

        let mut state = ParserState::new("a[-1]");

        state.push_variable_name("a");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        if let ScalarExpression::Variable(v) = expression {
            assert_eq!("a", v.get_name());
            assert_eq!(
                &[ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    -1
                ))]
                .to_vec(),
                v.get_selectors()
            );
        } else {
            panic!("Expected VariableScalarExpression");
        }
    }

    #[test]
    fn test_parse_assignment_expression() {
        let run_test_success = |input: &str, expected: TransformExpression| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::assignment_expression, input).unwrap();

            let expression = parse_assignment_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::assignment_expression, input).unwrap();

            let error = parse_assignment_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        let mut new_attribute_accessor = ValueAccessor::new();

        new_attribute_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "new_attribute",
        )));

        run_test_success(
            "new_attribute = 1",
            TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    new_attribute_accessor,
                )),
            )),
        );

        run_test_success(
            "variable = 'hello world'",
            TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                )),
                MutableValueExpression::Variable(VariableScalarExpression::new(
                    QueryLocation::new_fake(),
                    "variable",
                    ValueAccessor::new(),
                )),
            )),
        );

        run_test_failure(
            "resource.attributes['new_attribute'] = 1",
            "'resource.attributes['new_attribute']' destination accessor must refer to source or a variable to be used in an assignment expression",
        );
    }

    #[test]
    fn test_parse_extend_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::extend_expression, input).unwrap();

            let expression = parse_extend_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new(input).with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::extend_expression, input).unwrap();

            let error = parse_extend_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        let mut new_attribute1_accessor = ValueAccessor::new();

        new_attribute1_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "new_attribute1",
        )));

        run_test_success(
            "extend new_attribute1 = 1",
            vec![TransformExpression::Set(SetTransformExpression::new(
                QueryLocation::new_fake(),
                ImmutableValueExpression::Scalar(ScalarExpression::Static(
                    StaticScalarExpression::Integer(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                )),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    new_attribute1_accessor.clone(),
                )),
            ))],
        );

        let mut new_attribute2_accessor = ValueAccessor::new();

        new_attribute2_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "new_attribute2",
        )));

        run_test_success(
            "extend new_attribute1 = 1, new_attribute2 = 2",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            1,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        new_attribute1_accessor,
                    )),
                )),
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Integer(IntegerScalarExpression::new(
                            QueryLocation::new_fake(),
                            2,
                        )),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        new_attribute2_accessor,
                    )),
                )),
            ],
        );

        run_test_failure(
            "extend variable.key = 1",
            "'variable.key' destination accessor must refer to source to be used in an extend expression",
        );
    }

    #[test]
    fn test_parse_project_expression() {
        let run_test_success = |input: &str, expected: Vec<TransformExpression>| {
            let mut state = ParserState::new(input)
                .with_default_source_map_key_name("attributes")
                .with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_expression, input).unwrap();

            let expression = parse_project_expression(result.next().unwrap(), &state).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected: &str| {
            let mut state = ParserState::new(input)
                .with_default_source_map_key_name("attributes")
                .with_attached_data_names(&["resource"]);

            state.push_variable_name("variable");

            let mut result = KqlParser::parse(Rule::project_expression, input).unwrap();

            let error = parse_project_expression(result.next().unwrap(), &state).unwrap_err();

            if let ParserError::SyntaxError(_, msg) = error {
                assert_eq!(expected, msg);
            } else {
                panic!("Expected SyntaxError");
            }
        };

        run_test_success(
            "project key1",
            vec![TransformExpression::Clear(ClearTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                HashSet::from(["key1".into()]),
            ))],
        );

        run_test_success(
            "project key1, key2",
            vec![TransformExpression::Clear(ClearTransformExpression::new(
                QueryLocation::new_fake(),
                MutableValueExpression::Source(SourceScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueAccessor::new(),
                )),
                HashSet::from(["key1".into(), "key2".into()]),
            ))],
        );

        let mut key1_accessor = ValueAccessor::new();

        key1_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "key1",
        )));

        run_test_success(
            "project key1 = variable",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            "variable",
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        key1_accessor.clone(),
                    )),
                )),
                TransformExpression::Clear(ClearTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from(["key1".into()]),
                )),
            ],
        );

        let mut key2_accessor = ValueAccessor::new();

        key2_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "attributes",
        )));

        key2_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new_fake(),
            "key2",
        )));

        run_test_success(
            "project key1 = variable, source.attributes['key2'] = resource['key1'], source.attributes['key3']",
            vec![
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new_fake(),
                            "variable",
                            ValueAccessor::new(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        key1_accessor.clone(),
                    )),
                )),
                TransformExpression::Set(SetTransformExpression::new(
                    QueryLocation::new_fake(),
                    ImmutableValueExpression::Scalar(ScalarExpression::Attached(
                        AttachedScalarExpression::new(
                            QueryLocation::new_fake(),
                            "resource",
                            key1_accessor.clone(),
                        ),
                    )),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        key2_accessor,
                    )),
                )),
                TransformExpression::Clear(ClearTransformExpression::new(
                    QueryLocation::new_fake(),
                    MutableValueExpression::Source(SourceScalarExpression::new(
                        QueryLocation::new_fake(),
                        ValueAccessor::new(),
                    )),
                    HashSet::from(["key1".into(), "key2".into(), "key3".into()]),
                )),
            ],
        );

        run_test_failure(
            "project body.complex",
            "The 'body.complex' accessor expression should refer to a top-level map key on the source when used in a project expression",
        );

        run_test_failure(
            "project a[0]",
            "The 'a[0]' accessor expression should refer to a top-level map key on the source when used in a project expression",
        );

        run_test_failure(
            "project variable = 1",
            "'variable' destination accessor must refer to source to be used in a project expression",
        );

        run_test_failure(
            "project resource.attributes['key']",
            "To be valid in a project expression 'resource.attributes['key']' should be an assignment expression or an accessor expression which refers to the source",
        );

        run_test_failure(
            "project source",
            "The 'source' accessor expression should refer to a top-level map key on the source when used in a project expression",
        );

        run_test_failure(
            "project body['name'] = 'hello world'",
            "The 'body['name']' accessor expression should refer to a top-level map key on the source when used in a project expression",
        );

        run_test_failure(
            "project source.body['some_attr']",
            "The 'source.body['some_attr']' accessor expression should refer to a top-level map key on the source when used in a project expression",
        );
    }
}
