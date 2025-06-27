use chrono::{FixedOffset, NaiveDate};
use data_engine_expressions::*;
use data_engine_parser_abstractions::*;
use pest::iterators::Pair;

use crate::{Rule, date_utils, scalar_expression::parse_scalar_expression};

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

pub(crate) fn parse_double_literal(
    double_literal_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&double_literal_rule);

    let raw_value = double_literal_rule.as_str();
    let parsed_value = raw_value.parse::<f64>();
    if parsed_value.is_err() {
        return Err(ParserError::SyntaxError(
            to_query_location(&double_literal_rule),
            format!(
                "'{}' could not be parsed as a literal of type 'double'",
                raw_value.trim()
            ),
        ));
    }

    Ok(StaticScalarExpression::Double(DoubleScalarExpression::new(
        query_location,
        parsed_value.unwrap(),
    )))
}

pub(crate) fn parse_integer_literal(
    integer_literal_rule: Pair<Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&integer_literal_rule);

    let raw_value = integer_literal_rule.as_str();
    let parsed_value = raw_value.parse::<i64>();
    if parsed_value.is_err() {
        return Err(ParserError::SyntaxError(
            to_query_location(&integer_literal_rule),
            format!(
                "'{}' could not be parsed as a literal of type 'integer'",
                raw_value.trim()
            ),
        ));
    }

    Ok(StaticScalarExpression::Integer(
        IntegerScalarExpression::new(query_location, parsed_value.unwrap()),
    ))
}

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
                        original_value.trim()
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
                        original_value.trim()
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
                        original_value.trim()
                    ),
                ));
            }

            let ndt = nd.unwrap().and_hms_micro_opt(hour, min, sec, micro);

            if ndt.is_none() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value.trim()
                    ),
                ));
            }

            let tz = FixedOffset::east_opt(offset);
            if tz.is_none() {
                return Err(ParserError::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value.trim()
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
                        original_value.trim()
                    ),
                )),
            }
        }
        _ => panic!("Unexpected rule in datetime_expression: {datetime_rule}"),
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
        Rule::double_literal => parse_double_literal(real_rule),
        Rule::integer_literal => match parse_integer_literal(real_rule)? {
            StaticScalarExpression::Integer(v) => Ok(StaticScalarExpression::Double(
                DoubleScalarExpression::new(query_location, v.get_value() as f64),
            )),
            _ => panic!("Unexpected type returned from parse_integer_literal"),
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
///   behavior. This is controlled by `default_source_map_key` on
///   [`ParserState`].
///
///   `unknown` -> `Source(MapKey("attributes"), MapKey("unknown"))`
pub(crate) fn parse_accessor_expression(
    accessor_expression_rule: Pair<Rule>,
    state: &ParserState,
    allow_root_scalar: bool,
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
            Rule::integer_literal => match parse_integer_literal(pair)? {
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

                    value_accessor.push_selector(ValueSelector::ArrayIndex(v));
                }
                _ => panic!("Unexpected type returned from parse_integer_literal"),
            },
            Rule::string_literal => match parse_string_literal(pair) {
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
            Rule::scalar_expression => {
                let scalar = parse_scalar_expression(pair, state)?;

                // Note: If the returned scalar is a statically known string or
                // integer we fold it into a MapKey or ArrayIndex. Otherwise we
                // return an error that the value is invalid for an accessor
                // expression.
                if let ScalarExpression::Static(s) = &scalar {
                    if let StaticScalarExpression::String(str) = s {
                        if negate_location.is_some() {
                            return Err(ParserError::QueryLanguageDiagnostic {
                                location: scalar.get_query_location().clone(),
                                diagnostic_id: "KS141",
                                message: "The expression must have the type int".into(),
                            });
                        }

                        value_accessor.push_selector(ValueSelector::MapKey(str.clone()));
                    } else if let StaticScalarExpression::Integer(i) = s {
                        if negate_location.is_some() {
                            value_accessor.push_selector(ValueSelector::ArrayIndex(
                                IntegerScalarExpression::new(
                                    negate_location.unwrap(),
                                    -i.get_value(),
                                ),
                            ));
                            negate_location = None;
                        } else {
                            value_accessor.push_selector(ValueSelector::ArrayIndex(i.clone()));
                        }
                    } else {
                        return Err(ParserError::QueryLanguageDiagnostic {
                            location: scalar.get_query_location().clone(),
                            diagnostic_id: "KS141",
                            message: "The expression must have one of the types: int or string"
                                .into(),
                        });
                    }
                } else {
                    let value = scalar.to_value();

                    if negate_location.is_some() {
                        if value.is_some() && !matches!(value, Some(Value::Integer(_))) {
                            return Err(ParserError::QueryLanguageDiagnostic {
                                location: scalar.get_query_location().clone(),
                                diagnostic_id: "KS141",
                                message: "The expression must have the type int".into(),
                            });
                        }

                        value_accessor.push_selector(ValueSelector::ScalarExpression(
                            ScalarExpression::Negate(NegateScalarExpression::new(
                                negate_location.unwrap(),
                                scalar,
                            )),
                        ));
                        negate_location = None;
                    } else {
                        if value.is_some()
                            && !matches!(value, Some(Value::Integer(_)))
                            && !matches!(value, Some(Value::String(_)))
                        {
                            return Err(ParserError::QueryLanguageDiagnostic {
                                location: scalar.get_query_location().clone(),
                                diagnostic_id: "KS141",
                                message: "The expression must have one of the types: int or string"
                                    .into(),
                            });
                        }

                        value_accessor.push_selector(ValueSelector::ScalarExpression(scalar));
                    }
                }
            }
            Rule::minus_token => {
                negate_location = Some(to_query_location(&pair));
            }
            _ => panic!("Unexpected rule in accessor_expression: {pair}"),
        }
    }

    if root_accessor_identity == "source" {
        Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )))
    } else if state.is_attached_data_defined(root_accessor_identity) {
        return Ok(ScalarExpression::Attached(AttachedScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
    } else if state.is_variable_defined(root_accessor_identity) {
        return Ok(ScalarExpression::Variable(VariableScalarExpression::new(
            query_location,
            root_accessor_identity,
            value_accessor,
        )));
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
            if let Some(constant) = state.try_get_constant(root_accessor_identity) {
                if value_accessor.has_selectors() {
                    // Note: It is not currently supported to access into a constant.
                    // This is because statics are currently simple things like string,
                    // bool, double, float, datetime. If it becomes possible to have a
                    // static map or array this should be supported.
                    panic!("Accessor into a constant value encountered")
                }

                return Ok(ScalarExpression::Static(constant.clone()));
            }
        }

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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Datelike, Utc};
    use pest::Parser;

    use crate::{
        KqlParser,
        date_utils::{create_fixed, create_utc},
    };

    use super::*;

    #[test]
    fn test_pest_parse_string_literal_rule() {
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
    fn test_parse_string_literal() {
        let run_test = |input: &str, expected: &str| {
            let mut result = KqlParser::parse(Rule::string_literal, input).unwrap();

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
    fn test_pest_parse_double_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
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
            let mut result = KqlParser::parse(Rule::double_literal, input).unwrap();

            let f = parse_double_literal(result.next().unwrap());

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
    fn test_pest_parse_integer_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::integer_literal,
            &["123", "-123"],
            &[".53", "abc"],
        );
    }

    #[test]
    fn test_parse_integer_literal() {
        let run_test = |input: &str, expected: i64| {
            let mut result = KqlParser::parse(Rule::integer_literal, input).unwrap();

            let i = parse_integer_literal(result.next().unwrap());

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
        let i = parse_integer_literal(pairs.next().unwrap());

        assert!(i.is_err());

        if let ParserError::SyntaxError(_, _) = i.unwrap_err() {
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_pest_parse_datetime_literal_rule() {
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
    fn test_pest_parse_real_expression_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::real_expression,
            &["real(1.0)", "real(1)", "real(+inf)", "real(-inf)"],
            &["real(.1)", "real()", "real(abc)"],
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
    fn test_parse_identifier_literal_rule() {
        pest_test_helpers::test_pest_rule::<KqlParser, Rule>(
            Rule::identifier_literal,
            &["Abc", "abc_123", "_abc"],
            &[],
        );
    }

    #[test]
    fn test_pest_parse_accessor_expression_rule() {
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
            KqlParser::parse(Rule::accessor_expression, "abc['name']").unwrap(),
            &[
                (Rule::accessor_expression, "abc['name']"),
                (Rule::identifier_literal, "abc"),
                (Rule::string_literal, "'name'"),
            ],
        );

        pest_test_helpers::test_compound_pest_rule(
            KqlParser::parse(Rule::accessor_expression, "abc[-'name']").unwrap(),
            &[
                (Rule::accessor_expression, "abc[-'name']"),
                (Rule::identifier_literal, "abc"),
                (Rule::minus_token, "-"),
                (Rule::scalar_expression, "'name'"),
                (Rule::string_literal, "'name'"),
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
                (Rule::scalar_expression, "sub"),
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

    #[test]
    fn test_parse_accessor_expression_from_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "source.subkey['array'][0]").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new("source.subkey['array'][0]"),
            true,
        )
        .unwrap();

        if let ScalarExpression::Source(s) = expression {
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
                s.get_value_accessor().get_selectors()
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

        let expression = parse_accessor_expression(result.next().unwrap(), &state, true).unwrap();

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
                                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                                    StringScalarExpression::new(
                                        QueryLocation::new_fake(),
                                        "neg_attr"
                                    )
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
        let mut result = KqlParser::parse(Rule::accessor_expression, "subkey").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &ParserState::new_with_options(
                "subkey",
                ParserOptions::new().with_default_source_map_key_name("attributes"),
            ),
            true,
        )
        .unwrap();

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[ValueSelector::MapKey(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    "subkey"
                ))]
                .to_vec(),
                s.get_value_accessor().get_selectors()
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
            &ParserState::new_with_options(
                "resource['~at\\'tr~']",
                ParserOptions::new().with_attached_data_names(&["resource"]),
            ),
            true,
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
                a.get_value_accessor().get_selectors()
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

        let expression = parse_accessor_expression(result.next().unwrap(), &state, true).unwrap();

        if let ScalarExpression::Variable(v) = expression {
            assert_eq!("a", v.get_name());
            assert_eq!(
                &[ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                    QueryLocation::new_fake(),
                    -1
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
            let mut result = KqlParser::parse(Rule::accessor_expression, input).unwrap();

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
            let mut result = KqlParser::parse(Rule::accessor_expression, input).unwrap();

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
            ScalarExpression::Static(StaticScalarExpression::String(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "hello world",
            ))),
        );

        run_test_success(
            "const_int",
            ScalarExpression::Static(StaticScalarExpression::Integer(
                IntegerScalarExpression::new(QueryLocation::new_fake(), 1),
            )),
        );

        // Note: The inner scalars referencing constants are folded into static
        // lookups in this test.
        run_test_success(
            "source[const_str][const_int]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                    ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                        QueryLocation::new_fake(),
                        1,
                    )),
                ]),
            )),
        );

        // Note: The inner scalar is folded into a constant in this test.
        run_test_success(
            "source[iif(const_bool_false, 'a', const_str)]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                    StringScalarExpression::new(QueryLocation::new_fake(), "hello world"),
                )]),
            )),
        );

        // Note: The inner scalar is folded into a constant in this test.
        run_test_success(
            "source[-iif(const_bool_true, const_int, 0)]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![ValueSelector::ArrayIndex(
                    IntegerScalarExpression::new(QueryLocation::new_fake(), -1),
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
            let mut result = KqlParser::parse(Rule::accessor_expression, input).unwrap();

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
                ValueAccessor::new_with_selectors(vec![ValueSelector::MapKey(
                    StringScalarExpression::new(QueryLocation::new_fake(), "const_str"),
                )]),
            )),
        );

        run_test_success(
            "const_str[const_str]",
            ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "const_str",
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new_fake(),
                        "hello world",
                    )),
                ]),
            )),
        );
    }
}
