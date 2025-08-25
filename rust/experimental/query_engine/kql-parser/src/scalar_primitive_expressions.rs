// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use chrono::TimeDelta;
use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, scalar_expression::parse_scalar_expression};

/// The goal of this code is to unescape string literal values as they come in
/// when parsed from pest:
/// * `'some \\' string'` -> `some ' string`
/// * `\"some \\\" string\"` -> `some \" string`
pub(crate) fn parse_string_literal(string_literal_rule: Pair<Rule>) -> StaticScalarExpression {
    let query_location = to_query_location(&string_literal_rule);

    let raw_string = string_literal_rule.as_str();
    let mut chars = raw_string.chars();
    let mut s = String::with_capacity(raw_string.len());
    let mut position = 1;
    let mut last_char = '\0';

    let mut c = chars.next();
    loop {
        debug_assert!(c.is_some());

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

pub(crate) fn parse_datetime_expression(
    datetime_expression_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&datetime_expression_rule);

    let datetime_rule = datetime_expression_rule.into_inner().next().unwrap();

    let original_value: String = match datetime_rule.as_rule() {
        Rule::string_literal => match parse_string_literal(datetime_rule) {
            StaticScalarExpression::String(v) => v.get_value().into(),
            _ => panic!("Unexpected type returned from parse_string_literal"),
        },
        Rule::datetime_literal => datetime_rule.as_str().into(),
        _ => panic!("Unexpected rule in datetime_expression: {datetime_rule}"),
    };

    let s = StringScalarExpression::new(query_location.clone(), &original_value);

    match Value::String(&s).convert_to_datetime() {
        Some(d) => Ok(StaticScalarExpression::DateTime(
            DateTimeScalarExpression::new(query_location, d),
        )),
        _ => Err(ParserError::SyntaxError(
            query_location,
            format!(
                "'{}' could not be parsed as a literal of type 'datetime'",
                original_value.trim()
            ),
        )),
    }
}

pub(crate) fn parse_timespan_expression(
    time_expression_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&time_expression_rule);

    let mut inner_rules = time_expression_rule.into_inner();

    let first_rule = inner_rules.next().unwrap();

    let string_value = match first_rule.as_rule() {
        Rule::string_literal => match parse_string_literal(first_rule) {
            StaticScalarExpression::String(v) => v,
            _ => panic!("Unexpected type returned from parse_string_literal"),
        },
        Rule::time_literal => {
            StringScalarExpression::new(query_location.clone(), first_rule.as_str())
        }
        Rule::integer_literal => {
            let i = first_rule.as_str().parse::<i64>().map_err(|_| {
                ParserError::SyntaxError(
                    to_query_location(&first_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'integer'",
                        first_rule.as_str()
                    ),
                )
            })?;

            let units = inner_rules.next();

            let nano_seconds = to_nano_seconds(i, 0.0, get_multiplier(units));

            return Ok(StaticScalarExpression::TimeSpan(
                TimeSpanScalarExpression::new(query_location, TimeDelta::nanoseconds(nano_seconds)),
            ));
        }
        Rule::double_literal => {
            let d = first_rule.as_str().parse::<f64>().map_err(|_| {
                ParserError::SyntaxError(
                    to_query_location(&first_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'double'",
                        first_rule.as_str()
                    ),
                )
            })?;

            let whole = d.trunc() as i64;
            let fraction = d.fract().abs();

            let units = inner_rules.next();

            let nano_seconds = to_nano_seconds(whole, fraction, get_multiplier(units));

            return Ok(StaticScalarExpression::TimeSpan(
                TimeSpanScalarExpression::new(query_location, TimeDelta::nanoseconds(nano_seconds)),
            ));
        }
        _ => panic!("Unexpected rule in time_expression: {first_rule}"),
    };

    return match Value::String(&string_value).convert_to_timespan() {
        Some(t) => Ok(StaticScalarExpression::TimeSpan(
            TimeSpanScalarExpression::new(query_location, t),
        )),
        _ => Err(ParserError::SyntaxError(
            query_location,
            format!(
                "'{}' could not be parsed as a literal of type 'timespan'",
                string_value.get_value()
            ),
        )),
    };

    fn get_multiplier(rule: Option<Pair<Rule>>) -> i64 {
        match rule.map(|r| r.as_str()) {
            Some("millisecond") | Some("milliseconds") | Some("ms") => {
                TimeDelta::milliseconds(1).num_nanoseconds().unwrap()
            }
            Some("hour") | Some("hours") | Some("h") => {
                TimeDelta::hours(1).num_nanoseconds().unwrap()
            }
            Some("minute") | Some("minutes") | Some("m") => {
                TimeDelta::minutes(1).num_nanoseconds().unwrap()
            }
            Some("second") | Some("seconds") | Some("s") => {
                TimeDelta::seconds(1).num_nanoseconds().unwrap()
            }
            Some("microsecond") | Some("microseconds") => {
                TimeDelta::microseconds(1).num_nanoseconds().unwrap()
            }
            Some("tick") | Some("ticks") => 100,
            _ => TimeDelta::days(1).num_nanoseconds().unwrap(),
        }
    }

    fn to_nano_seconds(mut whole: i64, fraction: f64, multiplier: i64) -> i64 {
        let negate = if whole < 0 {
            whole = whole.abs();
            true
        } else {
            false
        };

        let mut nanos = whole * multiplier;
        if fraction > 0.0 {
            nanos += (multiplier as f64 * fraction) as i64;
        }

        if negate { -nanos } else { nanos }
    }
}

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
        Rule::double_literal => parse_standard_double_literal(real_rule, None),
        Rule::integer_literal => match parse_standard_integer_literal(real_rule)? {
            StaticScalarExpression::Integer(v) => Ok(StaticScalarExpression::Double(
                DoubleScalarExpression::new(query_location, v.get_value() as f64),
            )),
            _ => panic!("Unexpected type returned from parse_standard_integer_literal"),
        },
        _ => panic!("Unexpected rule in real_expression: {real_rule}"),
    }
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
///   behavior. This is controlled by `source_keys` and `default_source_map_key`
///   on [`ParserState`].
///
///   `unknown` -> `Source(MapKey("attributes"), MapKey("unknown"))`
pub(crate) fn parse_accessor_expression(
    accessor_expression_rule: Pair<Rule>,
    state: &dyn ParserScope,
    allow_root_scalar: bool,
) -> Result<ScalarExpression, ParserError> {
    let query_location = to_query_location(&accessor_expression_rule);

    let mut accessor_rules = accessor_expression_rule.into_inner();

    let root_accessor_identity_rule = accessor_rules.next().unwrap();

    let mut root_full_identifier_literal_complete = false;
    let mut root_full_identifier_literal_depth = 0;
    let mut root_full_identifier_literal = String::new();

    let root_accessor_identity = match root_accessor_identity_rule.as_rule() {
        Rule::string_literal => match parse_string_literal(root_accessor_identity_rule) {
            StaticScalarExpression::String(v) => {
                root_full_identifier_literal_complete = true;
                v
            }
            _ => panic!("Unexpected type returned from parse_string_literal"),
        },
        Rule::identifier_literal => {
            root_full_identifier_literal.push_str(root_accessor_identity_rule.as_str());

            StringScalarExpression::new(
                to_query_location(&root_accessor_identity_rule),
                root_accessor_identity_rule.as_str(),
            )
        }
        _ => panic!("Unexpected rule in accessor_expression: {root_accessor_identity_rule}"),
    };

    let mut value_accessor = ValueAccessor::new();

    let mut negate_location = None;

    loop {
        let accessor = accessor_rules.next();
        if accessor.is_none() {
            break;
        }

        let pair = accessor.unwrap();
        let pair_value = pair.as_str();
        let add_to_root_literal = match pair.as_rule() {
            Rule::integer_literal => {
                match parse_standard_integer_literal(pair)? {
                    StaticScalarExpression::Integer(v) => {
                        let i = v.get_value();

                        if i < i32::MIN as i64 || i > i32::MAX as i64 {
                            return Err(ParserError::SyntaxError(
                                v.get_query_location().clone(),
                                format!(
                                    "'{i}' value for array index is too large to fit into a 32bit value"
                                ),
                            ));
                        }

                        value_accessor.push_selector(ScalarExpression::Static(
                            StaticScalarExpression::Integer(v),
                        ));
                    }
                    _ => panic!("Unexpected type returned from parse_standard_integer_literal"),
                }

                false
            }
            Rule::string_literal => {
                match parse_string_literal(pair) {
                    StaticScalarExpression::String(v) => value_accessor
                        .push_selector(ScalarExpression::Static(StaticScalarExpression::String(v))),
                    _ => panic!("Unexpected type returned from parse_string_literal"),
                }

                false
            }
            Rule::identifier_literal => {
                value_accessor.push_selector(ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        to_query_location(&pair),
                        pair.as_str(),
                    )),
                ));

                true
            }
            Rule::scalar_expression => {
                let scalar = parse_scalar_expression(pair, state)?;

                let value_type_result = scalar.try_resolve_value_type(state.get_pipeline());
                if let Err(e) = value_type_result {
                    return Err(ParserError::from(&e));
                }

                let value_type = value_type_result.unwrap();

                if negate_location.is_some() {
                    if let Some(t) = value_type
                        && t != ValueType::Integer
                    {
                        return Err(ParserError::QueryLanguageDiagnostic {
                            location: scalar.get_query_location().clone(),
                            diagnostic_id: "KS141",
                            message: "The expression must have the type int".into(),
                        });
                    }

                    value_accessor.push_selector(ScalarExpression::Math(
                        MathScalarExpression::Negate(UnaryMathematicalScalarExpression::new(
                            negate_location.unwrap(),
                            scalar,
                        )),
                    ));
                    negate_location = None;
                } else {
                    if let Some(t) = value_type
                        && t != ValueType::Integer
                        && t != ValueType::String
                    {
                        return Err(ParserError::QueryLanguageDiagnostic {
                            location: scalar.get_query_location().clone(),
                            diagnostic_id: "KS141",
                            message: "The expression must have one of the types: int or string"
                                .into(),
                        });
                    }

                    value_accessor.push_selector(scalar);
                }

                false
            }
            Rule::minus_token => {
                negate_location = Some(to_query_location(&pair));
                false
            }
            _ => panic!("Unexpected rule in accessor_expression: {pair}"),
        };

        if !root_full_identifier_literal_complete && add_to_root_literal {
            root_full_identifier_literal_depth += 1;
            root_full_identifier_literal.push('.');
            root_full_identifier_literal.push_str(pair_value);
        } else {
            root_full_identifier_literal_complete = true;
        }
    }

    if root_accessor_identity.get_value() == "source" {
        let value_type = get_value_type(state, &value_accessor);

        Ok(ScalarExpression::Source(
            SourceScalarExpression::new_with_value_type(query_location, value_accessor, value_type),
        ))
    } else if allow_root_scalar
        && state.is_attached_data_defined(root_accessor_identity.get_value())
    {
        Ok(ScalarExpression::Attached(AttachedScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )))
    } else if allow_root_scalar && state.is_variable_defined(root_accessor_identity.get_value()) {
        Ok(ScalarExpression::Variable(VariableScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )))
    } else {
        // Note: If the accessor_expression is being used as a scalar it is
        // perfectly valid to return a static constant value. However accessors
        // can also be used in extend & project* expressions where the behavior
        // needs to change such that a static is invalid at the root. Example:
        // * iff(const_bool, const_val1, const_val2): This is a nested scalar
        //   expression. const_bool, const_val1, and const_val2 should be
        //   resolved to their values.
        // * extend const_str = 1: This expression needs to execute as
        //   source['const_str'] = 1 so the const_str is not evaluated.
        if allow_root_scalar {
            if let Some((constant_id, value_type)) =
                state.get_constant(root_accessor_identity.get_value())
            {
                if value_accessor.has_selectors() {
                    // Note: It is not currently supported to access into a constant.
                    // This is because statics are currently simple things like string,
                    // bool, double, float, datetime. If it becomes possible to have a
                    // static map or array this should be supported.
                    panic!("Accessor into a constant value encountered")
                }

                return Ok(ScalarExpression::Constant(
                    ConstantScalarExpression::Reference(ReferenceConstantScalarExpression::new(
                        query_location,
                        value_type,
                        constant_id,
                    )),
                ));
            }
        }

        if let Some(schema) = state.get_source_schema() {
            match schema.get_schema_for_key(root_accessor_identity.get_value()) {
                Some(key) => {
                    if value_accessor.has_selectors() {
                        // Note: If we are selecting a well-defined key on the
                        // source we can do some validation. If there are child
                        // selectors they are only valid for maps and arrays.
                        // This logic could be improved further to inspect the
                        // next selector and see if it is a string (parent must
                        // be a map) or an int (parent must be an array).
                        if !matches!(
                            key.get_value_type(),
                            Some(ValueType::Map) | Some(ValueType::Array) | None
                        ) {
                            return Err(ParserError::SyntaxError(
                                root_accessor_identity.get_query_location().clone(),
                                format!(
                                    "Cannot access into key '{}' which is defined as a '{:?}' type",
                                    root_accessor_identity.get_value(),
                                    key.get_value_type()
                                ),
                            ));
                        }
                    }

                    value_accessor.insert_selector(
                        0,
                        ScalarExpression::Static(StaticScalarExpression::String(
                            root_accessor_identity,
                        )),
                    );
                }
                None => {
                    if let Some(default_map_key) = schema.get_default_map_key() {
                        // Note: If the root is not found and we have a default
                        // map we insert it into the expression. Let's say
                        // default_source_map_key=attributes. And we have keys
                        // source_keys[timestamp, body, attributes] defined. If
                        // we see a selector "key1" we rewrite that as
                        // "attributes.key1" so the search for "key1" looks into
                        // the default map ("attributes" in this case).

                        let root_location = root_accessor_identity.get_query_location().clone();

                        if root_full_identifier_literal_depth > 0 {
                            // Note: If we have a selector like "some.thing" it
                            // will parse as [some][thing] by default. We know
                            // we didn't find "some" as a defined key on source.
                            // What this code does is for the lookup to the
                            // default map (say attributes) we do
                            // attributes["some.thing"] instead of
                            // attributes["some"]["thing"].

                            while root_full_identifier_literal_depth > 0 {
                                value_accessor.remove_selector(0);
                                root_full_identifier_literal_depth -= 1;
                            }

                            value_accessor.insert_selector(
                                0,
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    StringScalarExpression::new(
                                        root_location.clone(),
                                        root_full_identifier_literal.as_str(),
                                    ),
                                )),
                            );
                        } else {
                            value_accessor.insert_selector(
                                0,
                                ScalarExpression::Static(StaticScalarExpression::String(
                                    root_accessor_identity,
                                )),
                            );
                        }

                        value_accessor.insert_selector(
                            0,
                            ScalarExpression::Static(StaticScalarExpression::String(
                                StringScalarExpression::new(root_location, default_map_key),
                            )),
                        );
                    } else {
                        return Err(ParserError::QueryLanguageDiagnostic {
                            location: root_accessor_identity.get_query_location().clone(),
                            diagnostic_id: "KS109",
                            message: format!(
                                "The name '{}' does not refer to any known column, table, variable or function",
                                root_accessor_identity.get_value()
                            ),
                        });
                    }
                }
            }
        } else {
            value_accessor.insert_selector(
                0,
                ScalarExpression::Static(StaticScalarExpression::String(root_accessor_identity)),
            );
        }

        let value_type = get_value_type(state, &value_accessor);

        Ok(ScalarExpression::Source(
            SourceScalarExpression::new_with_value_type(query_location, value_accessor, value_type),
        ))
    }
}

fn get_value_type(state: &dyn ParserScope, value_accessor: &ValueAccessor) -> Option<ValueType> {
    let selectors = value_accessor.get_selectors();
    let mut value_type = None;
    if selectors.is_empty() {
        value_type = Some(ValueType::Map);
    } else if selectors.len() == 1
        && let Some(schema) = state.get_source_schema()
        && let ScalarExpression::Static(StaticScalarExpression::String(key)) =
            selectors.first().unwrap()
        && let Some(key_schema) = schema.get_schema_for_key(key.get_value())
    {
        value_type = key_schema.get_value_type();
    }

    value_type
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Datelike, FixedOffset, Utc};
    use pest::Parser;

    use crate::{KqlPestParser, date_utils::*};

    use super::*;

    #[test]
    fn test_pest_parse_string_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
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
    fn test_parse_string_literal() {
        let run_test = |input: &str, expected: &str| {
            let mut result = KqlPestParser::parse(Rule::string_literal, input).unwrap();

            let actual = parse_string_literal(result.next().unwrap());

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
    fn test_pest_parse_bool_literal_rule() {
        parse_test_helpers::test_parse_bool_literal::<KqlPestParser, Rule>(
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
    fn test_pest_parse_double_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::double_literal,
            &[
                "1.0", "-1.0", "1.0e1", "-1.0e1", "1e1", "-1e1", "1e+1", "1e-1",
            ],
            &["1", ".1", "abc"],
        );
    }

    #[test]
    fn test_parse_double_literal() {
        let run_test = |input: &str, expected: f64| {
            let mut result = KqlPestParser::parse(Rule::double_literal, input).unwrap();

            let f = parse_standard_double_literal(result.next().unwrap(), None);

            assert!(f.is_ok());

            match f.unwrap() {
                StaticScalarExpression::Double(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_standard_float_literal"),
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
    fn test_pest_parse_integer_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::integer_literal,
            &["123", "-123"],
            &[".53", "abc"],
        );
    }

    #[test]
    fn test_parse_integer_literal() {
        let run_test = |input: &str, expected: i64| {
            let mut result = KqlPestParser::parse(Rule::integer_literal, input).unwrap();

            let i = parse_standard_integer_literal(result.next().unwrap());

            assert!(i.is_ok());

            match i.unwrap() {
                StaticScalarExpression::Integer(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_standard_integer_literal"),
            }
        };

        run_test("1", 1);
        run_test("-1", -1);
    }

    #[test]
    fn test_parse_invalid_integer_literal() {
        let input = format!("{}", i64::MAX as i128 + 1);
        let result = KqlPestParser::parse(Rule::integer_literal, input.as_str());

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
    fn test_pest_parse_datetime_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
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
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::datetime_literal,
            &[
                "2014-05-25T08:20:03.123456Z",
                "2014-11-08 15:55:55.123456Z",
                "2014-11-08 15:55:55",
            ],
            &[],
        );
        // RFC 822
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::datetime_literal,
            &["Sat, 8 Nov 14 15:05:02 GMT", "8 Nov 14 15:05 GMT"],
            &[],
        );
        // RFC 850
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::datetime_literal,
            &["Saturday, 08-Nov-14 15:05:02 GMT", "08-Nov-14 15:05:02 GMT"],
            &[],
        );
        // Sortable
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::datetime_literal,
            &["2014-11-08 15:05:25 GMT", "2014-11-08T15:05:25 GMT"],
            &[],
        );
    }

    #[test]
    fn test_parse_datetime_expression() {
        let run_test_success = |input: &str, expected: DateTime<FixedOffset>| {
            let mut result = KqlPestParser::parse(Rule::datetime_expression, input).unwrap();

            let d = parse_datetime_expression(result.next().unwrap());

            assert!(d.is_ok());

            match d.unwrap() {
                StaticScalarExpression::DateTime(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_datetime_expression"),
            }
        };

        let run_test_failure = |input: &str| {
            let mut result = KqlPestParser::parse(Rule::datetime_expression, input).unwrap();

            let d = parse_datetime_expression(result.next().unwrap());

            assert!(d.is_err());
        };

        run_test_failure("datetime('hello world')");
        run_test_failure("datetime('hello world 8/5/2025')");

        let now = Utc::now();

        run_test_success(
            "datetime('12/31/2025')",
            create_utc(2025, 12, 31, 0, 0, 0, 0),
        );
        run_test_success(
            "datetime('   12/31/2025   ')",
            create_utc(2025, 12, 31, 0, 0, 0, 0),
        );
        run_test_success("datetime(12/31/2025)", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test_success("datetime(12/31/50)", create_utc(1950, 12, 31, 0, 0, 0, 0));
        run_test_success("datetime(12/31/49)", create_utc(2049, 12, 31, 0, 0, 0, 0));
        run_test_success("datetime(2025/12/31)", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test_success(
            "datetime(2025/12/31 22:30:10.1)",
            create_utc(2025, 12, 31, 22, 30, 10, 1),
        );
        run_test_success(
            "datetime(12-31-2025 10AM)",
            create_utc(2025, 12, 31, 10, 0, 0, 0),
        );
        run_test_success(
            "datetime(2025-12-31 10:30 PM)",
            create_utc(2025, 12, 31, 22, 30, 0, 0),
        );
        run_test_success(
            "datetime(10PM)",
            create_utc(now.year(), now.month(), now.day(), 22, 0, 0, 0),
        );

        // ISO 8601
        run_test_success(
            "datetime(2014-05-25T08:20:03.123456Z)",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test_success(
            "datetime(2009-06-15T13:45:30.0000000-07:00)",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, -7 * 60 * 60),
        );
        run_test_success(
            "datetime(2009-06-15T13:45:30.0000000+07:30)",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, (7 * 60 * 60) + (30 * 60)),
        );
        run_test_success(
            "datetime(2014-05-25T08:20:03.123456)",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test_success(
            "datetime(2014-05-25T08:20)",
            create_utc(2014, 5, 25, 8, 20, 0, 0),
        );
        run_test_success(
            "datetime(2014-11-08 15:55:55.123456Z)",
            create_utc(2014, 11, 8, 15, 55, 55, 123456),
        );
        run_test_success(
            "datetime(2014-11-08 15:55:55)",
            create_utc(2014, 11, 8, 15, 55, 55, 0),
        );
        run_test_success(
            "datetime(2014-11-08 15:55)",
            create_utc(2014, 11, 8, 15, 55, 0, 0),
        );
        run_test_success("datetime(2014-11-08)", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 822
        run_test_success(
            "datetime(Sat, 8 Nov 14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(Sat, 8 Nov 14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(8 Nov 14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(8 Nov 14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(8 Nov 14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(8 Nov 14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success("datetime(8 Nov 14)", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 850
        run_test_success(
            "datetime(Saturday, 08-Nov-14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(Saturday, 08-Nov-14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(Saturday, 08-Nov-14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(Saturday, 08-Nov-14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(08-Nov-14 15:05:02 GMT)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(08-Nov-14 15:05:02)",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "datetime(08-Nov-14 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(08-Nov-14 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );

        // Sortable
        run_test_success(
            "datetime(2014-11-08 15:05:25)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success(
            "datetime(2014-11-08 15:05:25 GMT)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success(
            "datetime(2014-11-08 15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(2014-11-08 15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(2014-11-08T15:05:25)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success(
            "datetime(2014-11-08T15:05:25 GMT)",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success(
            "datetime(2014-11-08T15:05)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "datetime(2014-11-08T15:05 GMT)",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
    }

    #[test]
    fn test_parse_timespan_expression() {
        let run_test_success = |input: &str, expected: TimeDelta| {
            let mut result = KqlPestParser::parse(Rule::time_expression, input).unwrap();

            let d = parse_timespan_expression(result.next().unwrap());

            assert!(d.is_ok());

            match d.unwrap() {
                StaticScalarExpression::TimeSpan(v) => assert_eq!(expected, v.get_value()),
                _ => panic!("Unexpected type retured from parse_timespan_expression"),
            }
        };

        let run_test_failure = |input: &str| {
            let mut result = KqlPestParser::parse(Rule::time_expression, input).unwrap();

            let d = parse_timespan_expression(result.next().unwrap());

            assert!(d.is_err());
        };

        run_test_failure("timespan('hello world')");
        run_test_failure("timespan('hello 09:01:00 world')");

        run_test_success("timespan(1)", TimeDelta::days(1));
        run_test_success("timespan(-1)", TimeDelta::days(-1));
        run_test_success("timespan(1.5)", TimeDelta::days(1) + TimeDelta::hours(12));
        run_test_success(
            "timespan(-1.5)",
            -(TimeDelta::days(1) + TimeDelta::hours(12)),
        );
        run_test_success("timespan(1 day)", TimeDelta::days(1));
        run_test_success("timespan(-1 day)", TimeDelta::days(-1));
        run_test_success("timespan(1 days)", TimeDelta::days(1));
        run_test_success("timespan(-1 days)", TimeDelta::days(-1));
        run_test_success("timespan(1.0:0:0)", TimeDelta::days(1));
        run_test_success("timespan(-1.0:0:0)", TimeDelta::days(-1));
        run_test_success("timespan( '1.0:0:0' )", TimeDelta::days(1));
        run_test_success("timespan( '-1.0:0:0' )", TimeDelta::days(-1));
        run_test_success("1d", TimeDelta::days(1));
        run_test_success("1day", TimeDelta::days(1));
        run_test_success("1days", TimeDelta::days(1));
        run_test_success("-1.5 d", -(TimeDelta::days(1) + TimeDelta::hours(12)));
        run_test_success("-1.5 day", -(TimeDelta::days(1) + TimeDelta::hours(12)));
        run_test_success("-1.5 days", -(TimeDelta::days(1) + TimeDelta::hours(12)));

        // Test fractional calculations
        run_test_success("-1.5 h", -(TimeDelta::hours(1) + TimeDelta::minutes(30)));
        run_test_success("-1.5 m", -(TimeDelta::minutes(1) + TimeDelta::seconds(30)));
        run_test_success(
            "-1.5 s",
            -(TimeDelta::seconds(1) + TimeDelta::milliseconds(500)),
        );
        run_test_success(
            "-1.5 ms",
            -(TimeDelta::milliseconds(1) + TimeDelta::microseconds(500)),
        );
        run_test_success(
            "-1.5 microseconds",
            -(TimeDelta::microseconds(1) + TimeDelta::nanoseconds(500)),
        );
        run_test_success("-1.5 ticks", -(TimeDelta::nanoseconds(150)));

        // Test unit parsing
        run_test_success("1 h", TimeDelta::hours(1));
        run_test_success("1 hour", TimeDelta::hours(1));
        run_test_success("1 hours", TimeDelta::hours(1));

        run_test_success("1 microsecond", TimeDelta::microseconds(1));
        run_test_success("1 microseconds", TimeDelta::microseconds(1));

        run_test_success("1 ms", TimeDelta::milliseconds(1));
        run_test_success("1 millisecond", TimeDelta::milliseconds(1));
        run_test_success("1 milliseconds", TimeDelta::milliseconds(1));

        run_test_success("1 m", TimeDelta::minutes(1));
        run_test_success("1 minute", TimeDelta::minutes(1));
        run_test_success("1 minutes", TimeDelta::minutes(1));

        run_test_success("1 s", TimeDelta::seconds(1));
        run_test_success("1 second", TimeDelta::seconds(1));
        run_test_success("1 seconds", TimeDelta::seconds(1));

        run_test_success("1 tick", TimeDelta::nanoseconds(100));
        run_test_success("1 ticks", TimeDelta::nanoseconds(100));

        // Test full timespan string parsing. Note: There are a lot more tests
        // in date_utils (inside expressions) which really owns the parsing
        // logic. This is more of a sanity check.
        run_test_success(
            "timespan(-1.12:6:3.001)",
            -(TimeDelta::days(1)
                + TimeDelta::hours(12)
                + TimeDelta::minutes(6)
                + TimeDelta::seconds(3)
                + TimeDelta::milliseconds(1)),
        );
    }

    #[test]
    fn test_pest_parse_real_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::real_expression,
            &["real(1.0)", "real(1)", "real(+inf)", "real(-inf)"],
            &["real(.1)", "real()", "real(abc)"],
        );
    }

    #[test]
    fn test_parse_real_expression() {
        let run_test = |input: &str, expected: f64| {
            let mut result = KqlPestParser::parse(Rule::real_expression, input).unwrap();

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
    fn test_pest_parse_identifier_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::identifier_literal,
            &["Abc", "abc_123", "_abc"],
            &[],
        );
    }

    #[test]
    fn test_pest_parse_identifier_or_pattern_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::identifier_or_pattern_literal,
            &["*", "abc*", "a*b*c", "['something.*']"],
            &[],
        );
    }

    #[test]
    fn test_pest_parse_accessor_expression_rule() {
        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "Abc").unwrap(),
            &[
                (Rule::accessor_expression, "Abc"),
                (Rule::identifier_literal, "Abc"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "['hello world']").unwrap(),
            &[
                (Rule::accessor_expression, "['hello world']"),
                (Rule::string_literal, "'hello world'"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "abc_123").unwrap(),
            &[
                (Rule::accessor_expression, "abc_123"),
                (Rule::identifier_literal, "abc_123"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "_abc").unwrap(),
            &[
                (Rule::accessor_expression, "_abc"),
                (Rule::identifier_literal, "_abc"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "array[0]").unwrap(),
            &[
                (Rule::accessor_expression, "array[0]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "0"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "array[-1]").unwrap(),
            &[
                (Rule::accessor_expression, "array[-1]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "-1"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "abc.name").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "abc['name']").unwrap(),
            &[
                (Rule::accessor_expression, "abc['name']"),
                (Rule::identifier_literal, "abc"),
                (Rule::string_literal, "'name'"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "abc[-'name']").unwrap(),
            &[
                (Rule::accessor_expression, "abc[-'name']"),
                (Rule::identifier_literal, "abc"),
                (Rule::minus_token, "-"),
                (Rule::scalar_expression, "'name'"),
                (Rule::string_literal, "'name'"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(Rule::accessor_expression, "abc.name1.name2").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name1.name2"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name1"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlPestParser::parse(
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
                (Rule::scalar_expression, "sub"),
                (Rule::accessor_expression, "sub"),
                (Rule::identifier_literal, "sub"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        pest_test_helpers::test_pest_rule::<KqlPestParser, Rule>(
            Rule::accessor_expression,
            &[],
            &["123", "+name", "-name", "~name", ".name"],
        );
    }

    #[test]
    fn test_parse_accessor_expression_from_source() {
        let mut result =
            KqlPestParser::parse(Rule::accessor_expression, "source.subkey['array'][0]").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new("source.subkey['array'][0]"),
            true,
        )
        .unwrap();

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "subkey")
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "array")
                    )),
                    ScalarExpression::Static(StaticScalarExpression::Integer(
                        IntegerScalarExpression::new(QueryLocation::new_fake(), 0)
                    ))
                ]
                .to_vec(),
                s.get_value_accessor().get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source() {
        let mut result = KqlPestParser::parse(
            Rule::accessor_expression,
            "['sub.key thing'][var][-neg_attr]",
        )
        .unwrap();

        let mut state = ParserState::new("['sub.key thing'][var][-neg_attr]");

        state.push_variable_name("var");

        let expression = parse_accessor_expression(result.next().unwrap(), &state, true).unwrap();

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "sub.key thing")
                    )),
                    ScalarExpression::Variable(VariableScalarExpression::new(
                        QueryLocation::new_fake(),
                        StringScalarExpression::new(QueryLocation::new_fake(), "var"),
                        ValueAccessor::new()
                    )),
                    ScalarExpression::Math(MathScalarExpression::Negate(
                        UnaryMathematicalScalarExpression::new(
                            QueryLocation::new_fake(),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                                    StaticScalarExpression::String(StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "neg_attr"
                                    ))
                                )]),
                            ))
                        )
                    ))
                ]
                .to_vec(),
                s.get_value_accessor().get_selectors()
            );
        } else {
            panic!("Expected SourceScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source_and_default_map() {
        let run_test = |query: &str, expected: &[ScalarExpression]| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, query).unwrap();

            let expression = parse_accessor_expression(
                result.next().unwrap(),
                &ParserState::new_with_options(
                    query,
                    ParserOptions::new().with_source_map_schema(
                        ParserMapSchema::new().set_default_map_key("attributes"),
                    ),
                ),
                true,
            )
            .unwrap();

            if let ScalarExpression::Source(s) = expression {
                assert_eq!(expected, s.get_value_accessor().get_selectors());
            } else {
                panic!("Expected SourceScalarExpression");
            }
        };

        run_test(
            "subkey",
            &vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "subkey"),
                )),
            ],
        );

        run_test(
            "sub.key",
            &vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "sub.key"),
                )),
            ],
        );

        run_test(
            "sub.a.b['key1']",
            &vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "attributes"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "sub.a.b"),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "key1"),
                )),
            ],
        );
    }

    #[test]
    fn test_parse_accessor_expression_implicit_source_and_souce_keys() {
        let run_test_success = |query: &str, expected: SourceScalarExpression| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, query).unwrap();

            let expression = parse_accessor_expression(
                result.next().unwrap(),
                &ParserState::new_with_options(
                    query,
                    ParserOptions::new().with_source_map_schema(
                        ParserMapSchema::new()
                            .with_key_definition("int_key", ParserMapKeySchema::Integer)
                            .with_key_definition("key_without_type", ParserMapKeySchema::Any),
                    ),
                ),
                true,
            )
            .unwrap();

            if let ScalarExpression::Source(s) = expression {
                assert_eq!(expected, s);
            } else {
                panic!("Expected SourceScalarExpression");
            }
        };

        let run_test_failure = |query: &str, expected_id: &str, expected_msg: &str| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, query).unwrap();

            let error = parse_accessor_expression(
                result.next().unwrap(),
                &ParserState::new_with_options(
                    query,
                    ParserOptions::new().with_source_map_schema(
                        ParserMapSchema::new()
                            .with_key_definition("int_key", ParserMapKeySchema::Integer)
                            .with_key_definition("key_without_type", ParserMapKeySchema::Any),
                    ),
                ),
                true,
            )
            .unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "int_key",
            SourceScalarExpression::new_with_value_type(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "int_key",
                    )),
                )]),
                Some(ValueType::Integer),
            ),
        );

        run_test_success(
            "key_without_type",
            SourceScalarExpression::new_with_value_type(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "key_without_type",
                    )),
                )]),
                None,
            ),
        );

        run_test_failure(
            "unknown_key",
            "KS109",
            "The name 'unknown_key' does not refer to any known column, table, variable or function",
        );
    }

    #[test]
    fn test_parse_accessor_expression_from_attached() {
        let mut result =
            KqlPestParser::parse(Rule::accessor_expression, "resource['~at\\'tr~']").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new_with_options(
                "resource['~at\\'tr~']",
                ParserOptions::new().with_attached_data_names(&["resource"]),
            ),
            true,
        )
        .unwrap();

        if let ScalarExpression::Attached(a) = expression {
            assert_eq!("resource", a.get_name().get_value());
            assert_eq!(
                &[ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "~at'tr~")
                ))]
                .to_vec(),
                a.get_value_accessor().get_selectors()
            );
        } else {
            panic!("Expected AttachedScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_from_variable() {
        let mut result = KqlPestParser::parse(Rule::accessor_expression, "a[-1]").unwrap();

        let mut state = ParserState::new("a[-1]");

        state.push_variable_name("a");

        let expression = parse_accessor_expression(result.next().unwrap(), &state, true).unwrap();

        if let ScalarExpression::Variable(v) = expression {
            assert_eq!("a", v.get_name().get_value());
            assert_eq!(
                &[ScalarExpression::Static(StaticScalarExpression::Integer(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -1)
                ))]
                .to_vec(),
                v.get_value_accessor().get_selectors()
            );
        } else {
            panic!("Expected VariableScalarExpression");
        }
    }

    #[test]
    fn test_parse_accessor_expression_with_scalars_and_allow_root_scalars() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, input).unwrap();

            let mut state = ParserState::new(input);

            state.push_constant(
                "const_int",
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    1,
                )),
            );
            state.push_constant(
                "const_bool_true",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    true,
                )),
            );
            state.push_constant(
                "const_bool_false",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    false,
                )),
            );
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let expression =
                parse_accessor_expression(result.next().unwrap(), &state, true).unwrap();

            assert_eq!(expected, expression);
        };

        let run_test_failure = |input: &str, expected_id: &str, expected_msg: &str| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, input).unwrap();

            let mut state = ParserState::new(input);

            state.push_constant(
                "const_int",
                StaticScalarExpression::Integer(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    1,
                )),
            );
            state.push_constant(
                "const_bool_true",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    true,
                )),
            );
            state.push_constant(
                "const_bool_false",
                StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                    QueryLocation::new_fake(),
                    false,
                )),
            );
            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let error =
                parse_accessor_expression(result.next().unwrap(), &state, true).unwrap_err();

            if let ParserError::QueryLanguageDiagnostic {
                location: _,
                diagnostic_id: id,
                message: msg,
            } = error
            {
                assert_eq!(expected_id, id);
                assert_eq!(expected_msg, msg);
            } else {
                panic!("Expected QueryLanguageDiagnostic");
            }
        };

        run_test_success(
            "const_str",
            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::String,
                    3,
                ),
            )),
        );

        run_test_success(
            "const_int",
            ScalarExpression::Constant(ConstantScalarExpression::Reference(
                ReferenceConstantScalarExpression::new(
                    QueryLocation::new_fake(),
                    ValueType::Integer,
                    0,
                ),
            )),
        );

        // Note: The inner scalars referencing constants are folded into static
        // lookups in this test.
        run_test_success(
            "source[const_str][const_int]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Constant(ConstantScalarExpression::Reference(
                        ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::String,
                            3,
                        ),
                    )),
                    ScalarExpression::Constant(ConstantScalarExpression::Reference(
                        ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::Integer,
                            0,
                        ),
                    )),
                ]),
            )),
        );

        // Note: The inner accessors are folded into constants in this test.
        run_test_success(
            "source[iif(const_bool_false, 'a', const_str)]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Conditional(
                    ConditionalScalarExpression::new(
                        QueryLocation::new_fake(),
                        LogicalExpression::Scalar(ScalarExpression::Constant(
                            ConstantScalarExpression::Reference(
                                ReferenceConstantScalarExpression::new(
                                    QueryLocation::new_fake(),
                                    ValueType::Boolean,
                                    2,
                                ),
                            ),
                        )),
                        ScalarExpression::Static(StaticScalarExpression::String(
                            StringScalarExpression::new(QueryLocation::new_fake(), "a"),
                        )),
                        ScalarExpression::Constant(ConstantScalarExpression::Reference(
                            ReferenceConstantScalarExpression::new(
                                QueryLocation::new_fake(),
                                ValueType::String,
                                3,
                            ),
                        )),
                    ),
                )]),
            )),
        );

        run_test_failure(
            "source.sub[real(1)]",
            "KS141",
            "The expression must have one of the types: int or string",
        );

        run_test_failure(
            "sub[-('attr')]",
            "KS141",
            "The expression must have the type int",
        );

        run_test_failure(
            "source[const_bool_true]",
            "KS141",
            "The expression must have one of the types: int or string",
        );

        run_test_failure(
            "source[-const_str]",
            "KS141",
            "The expression must have the type int",
        );
    }

    #[test]
    fn test_parse_accessor_expression_with_scalars_and_disallow_root_scalars() {
        let run_test_success = |input: &str, expected: ScalarExpression| {
            let mut result = KqlPestParser::parse(Rule::accessor_expression, input).unwrap();

            let mut state = ParserState::new(input);

            state.push_constant(
                "const_str",
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "hello world",
                )),
            );

            let expression =
                parse_accessor_expression(result.next().unwrap(), &state, false).unwrap();

            assert_eq!(expected, expression);
        };

        run_test_success(
            "const_str",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                    StaticScalarExpression::String(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "const_str",
                    )),
                )]),
            )),
        );

        run_test_success(
            "const_str[const_str]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), "const_str"),
                    )),
                    ScalarExpression::Constant(ConstantScalarExpression::Reference(
                        ReferenceConstantScalarExpression::new(
                            QueryLocation::new_fake(),
                            ValueType::String,
                            0,
                        ),
                    )),
                ]),
            )),
        );
    }
}
