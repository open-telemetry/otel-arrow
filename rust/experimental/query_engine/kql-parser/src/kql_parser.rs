use std::collections::HashSet;

use chrono::{FixedOffset, NaiveDate};
use data_engine_expressions::*;
use pest::iterators::Pair;

use pest_derive::Parser;

use crate::{Error, date_utils};

#[derive(Parser)]
#[grammar = "kql.pest"]
#[allow(dead_code)]
pub(crate) struct KqlParser;

#[allow(dead_code)]
pub(crate) struct KqlParserState {
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
    variable_names: HashSet<Box<str>>,
}

impl KqlParserState {
    #[allow(dead_code)]
    pub fn new() -> KqlParserState {
        Self {
            default_source_map_key: None,
            attached_data_names: HashSet::new(),
            variable_names: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_default_source_map_key_name(mut self, name: &str) -> KqlParserState {
        if !name.is_empty() {
            self.default_source_map_key = Some(name.into());
        }

        self
    }

    #[allow(dead_code)]
    pub fn with_attached_data_names(mut self, names: &[&str]) -> KqlParserState {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }

    #[allow(dead_code)]
    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }
}

/// The goal of this code is to unescape string literal values as they come in
/// when parsed from pest:
/// * `'some \\' string'` -> `some ' string`
/// * `\"some \\\" string\"` -> `some \" string`
#[allow(dead_code)]
pub(crate) fn parse_string_literal(string_literal_rule: Pair<Rule>) -> String {
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

    s
}

#[allow(dead_code)]
pub(crate) fn parse_double_literal(double_literal_rule: Pair<Rule>) -> Result<f64, Error> {
    let raw_value = double_literal_rule.as_str();
    let parsed_value = raw_value.parse::<f64>();
    if parsed_value.is_err() {
        return Err(Error::SyntaxError(
            to_query_location(&double_literal_rule),
            format!(
                "'{}' could not be parsed as a literal of type 'double'",
                raw_value
            ),
        ));
    }

    Ok(parsed_value.unwrap())
}

#[allow(dead_code)]
pub(crate) fn parse_integer_literal(integer_literal_rule: Pair<Rule>) -> Result<i64, Error> {
    let raw_value = integer_literal_rule.as_str();
    let parsed_value = raw_value.parse::<i64>();
    if parsed_value.is_err() {
        return Err(Error::SyntaxError(
            to_query_location(&integer_literal_rule),
            format!(
                "'{}' could not be parsed as a literal of type 'integer'",
                raw_value
            ),
        ));
    }

    Ok(parsed_value.unwrap())
}

#[allow(dead_code)]
pub(crate) fn parse_datetime_expression(
    datetime_expression_rule: Pair<Rule>,
) -> Result<ScalarExpression, Error> {
    let query_location = to_query_location(&datetime_expression_rule);

    let datetime_rule = datetime_expression_rule.into_inner().next().unwrap();

    match datetime_rule.as_rule() {
        Rule::datetime_literal => {
            let original_value = datetime_rule.as_str();
            let mut raw_value: String = original_value.into();

            let date = date_utils::parse_date(&raw_value);
            if date.is_err() {
                return Err(Error::SyntaxError(
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
                return Err(Error::SyntaxError(
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
                return Err(Error::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let ndt = nd.unwrap().and_hms_micro_opt(hour, min, sec, micro);

            if ndt.is_none() {
                return Err(Error::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let tz = FixedOffset::east_opt(offset);
            if tz.is_none() {
                return Err(Error::SyntaxError(
                    to_query_location(&datetime_rule),
                    format!(
                        "'{}' could not be parsed as a literal of type 'datetime'",
                        original_value
                    ),
                ));
            }

            let l = ndt.unwrap().and_local_timezone(tz.unwrap());

            match l {
                chrono::offset::LocalResult::Single(date_time) => Ok(ScalarExpression::DateTime(
                    DateTimeScalarExpression::new(query_location, date_time),
                )),
                _ => Err(Error::SyntaxError(
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
) -> Result<ScalarExpression, Error> {
    let query_location = to_query_location(&real_expression_rule);

    let real_rule = real_expression_rule.into_inner().next().unwrap();

    match real_rule.as_rule() {
        Rule::positive_infinity_token => Ok(ScalarExpression::Double(DoubleScalarExpression::new(
            query_location,
            f64::INFINITY,
        ))),
        Rule::negative_infinity_token => Ok(ScalarExpression::Double(DoubleScalarExpression::new(
            query_location,
            f64::NEG_INFINITY,
        ))),
        Rule::double_literal => {
            let d = parse_double_literal(real_rule)?;

            Ok(ScalarExpression::Double(DoubleScalarExpression::new(
                query_location,
                d,
            )))
        }
        Rule::integer_literal => {
            let i = parse_integer_literal(real_rule)?;

            Ok(ScalarExpression::Double(DoubleScalarExpression::new(
                query_location,
                i as f64,
            )))
        }
        _ => panic!("Unexpected rule in real_expression: {}", real_rule),
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
///   [`KqlParserState`].
///
///   `unknown` -> `Source(MapKey("attributes"), MapKey("unknown"))`
#[allow(dead_code)]
pub(crate) fn parse_accessor_expression(
    accessor_expression_rule: Pair<Rule>,
    state: &KqlParserState,
) -> Result<ScalarExpression, Error> {
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

                let i = parse_integer_literal(pair)?;
                if i < i32::MIN as i64 || i > i32::MAX as i64 {
                    return Err(Error::SyntaxError(
                        location,
                        format!(
                            "'{}' value for array index is too large to fit into a 32bit value",
                            i
                        ),
                    ));
                }
                value_accessor.push_selector(ValueSelector::ArrayIndex(
                    IntegerScalarExpression::new(location, i),
                ));
            }
            Rule::string_literal => {
                value_accessor.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
                    to_query_location(&pair),
                    parse_string_literal(pair).as_str(),
                )));
            }
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

        if state.default_source_map_key.is_some() {
            value_accessor.insert_selector(
                0,
                ValueSelector::MapKey(StringScalarExpression::new(
                    query_location.clone(),
                    state.default_source_map_key.as_ref().unwrap(),
                )),
            );
        }

        return Ok(ScalarExpression::Source(SourceScalarExpression::new(
            query_location,
            value_accessor,
        )));
    }
}

pub(crate) fn to_query_location(rule: &Pair<Rule>) -> QueryLocation {
    let s = rule.as_span();
    let (line_number, column_number) = rule.line_col();
    QueryLocation::new(s.start(), s.end(), line_number, column_number)
}

#[cfg(test)]
mod pest_tests {
    use std::mem::discriminant;

    use super::*;
    use pest::{Parser, iterators::Pairs};

    #[test]
    fn test_true_literal() {
        assert!(KqlParser::parse(Rule::true_literal, "true").is_ok());
        assert!(KqlParser::parse(Rule::true_literal, "True").is_ok());
        assert!(KqlParser::parse(Rule::true_literal, "TRUE").is_ok());
        assert!(KqlParser::parse(Rule::true_literal, "tRuE").is_err());
        assert!(KqlParser::parse(Rule::true_literal, "false").is_err());
        assert!(KqlParser::parse(Rule::true_literal, "tru").is_err());
    }

    #[test]
    fn test_false_literal() {
        assert!(KqlParser::parse(Rule::false_literal, "false").is_ok());
        assert!(KqlParser::parse(Rule::false_literal, "False").is_ok());
        assert!(KqlParser::parse(Rule::false_literal, "FALSE").is_ok());
        assert!(KqlParser::parse(Rule::false_literal, "fAlSe").is_err());
        assert!(KqlParser::parse(Rule::false_literal, "true").is_err());
        assert!(KqlParser::parse(Rule::false_literal, "fals").is_err());
    }

    #[test]
    fn test_datetime_literal() {
        assert!(KqlParser::parse(Rule::datetime_literal, "").is_err());
        assert!(KqlParser::parse(Rule::datetime_literal, "\"").is_err());
        assert!(KqlParser::parse(Rule::datetime_literal, "'").is_err());
        assert!(KqlParser::parse(Rule::datetime_literal, "\\").is_err());

        assert!(KqlParser::parse(Rule::datetime_literal, "12/31/2025").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "12/31/2025 10 AM").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "12-31-2025 10:00PM").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "12-31-2025 13:00:00").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "2025-12-13 13:00:00 +08:00").is_ok());

        assert!(KqlParser::parse(Rule::datetime_literal, "November 7, 2025").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "Nov 7 25").is_ok());

        // ISO 8601
        assert!(KqlParser::parse(Rule::datetime_literal, "2014-05-25T08:20:03.123456Z").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "2014-11-08 15:55:55.123456Z").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "2014-11-08 15:55:55").is_ok());
        // RFC 822
        assert!(KqlParser::parse(Rule::datetime_literal, "Sat, 8 Nov 14 15:05:02 GMT").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "8 Nov 14 15:05 GMT").is_ok());
        // RFC 850
        assert!(
            KqlParser::parse(Rule::datetime_literal, "Saturday, 08-Nov-14 15:05:02 GMT").is_ok()
        );
        assert!(KqlParser::parse(Rule::datetime_literal, "08-Nov-14 15:05:02 GMT").is_ok());
        // Sortable
        assert!(KqlParser::parse(Rule::datetime_literal, "2014-11-08 15:05:25 GMT").is_ok());
        assert!(KqlParser::parse(Rule::datetime_literal, "2014-11-08T15:05:25 GMT").is_ok());
    }

    #[test]
    fn test_double_literal() {
        assert!(KqlParser::parse(Rule::double_literal, "1.0").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "-1.0").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "1.0e1").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "-1.0e1").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "1e1").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "-1e1").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "1e+1").is_ok());
        assert!(KqlParser::parse(Rule::double_literal, "1e-1").is_ok());

        assert!(KqlParser::parse(Rule::double_literal, "1").is_err());
        assert!(KqlParser::parse(Rule::double_literal, ".1").is_err());
        assert!(KqlParser::parse(Rule::double_literal, "abc").is_err());
    }

    #[test]
    fn test_integer_literal() {
        assert!(KqlParser::parse(Rule::integer_literal, "123").is_ok());
        assert!(KqlParser::parse(Rule::integer_literal, "-123").is_ok());
        assert!(KqlParser::parse(Rule::integer_literal, ".53").is_err());
        assert!(KqlParser::parse(Rule::integer_literal, "abc").is_err());
    }

    #[test]
    fn test_string_literal() {
        assert!(KqlParser::parse(Rule::string_literal, "\"hello\"").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "\"he\\\"llo\"").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'hello'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'he\"llo'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, "'he\\'llo'").is_ok());
        assert!(KqlParser::parse(Rule::string_literal, r#""hello"#).is_err());
        assert!(KqlParser::parse(Rule::string_literal, r#"hello""#).is_err());
        assert!(KqlParser::parse(Rule::string_literal, r#""""#).is_ok());
    }

    #[test]
    fn test_identifier_literal() {
        assert!(KqlParser::parse(Rule::identifier_literal, "Abc").is_ok());
        assert!(KqlParser::parse(Rule::identifier_literal, "abc_123").is_ok());
        assert!(KqlParser::parse(Rule::identifier_literal, "_abc").is_ok());
    }

    #[test]
    fn test_real_expression() {
        assert!(KqlParser::parse(Rule::real_expression, "real(1.0)").is_ok());
        assert!(KqlParser::parse(Rule::real_expression, "real(1)").is_ok());
        assert!(KqlParser::parse(Rule::real_expression, "real(+inf)").is_ok());
        assert!(KqlParser::parse(Rule::real_expression, "real(-inf)").is_ok());

        assert!(KqlParser::parse(Rule::real_expression, "real(.1)").is_err());
        assert!(KqlParser::parse(Rule::real_expression, "real()").is_err());
        assert!(KqlParser::parse(Rule::real_expression, "real(abc)").is_err());
    }

    #[test]
    fn test_accessor_expression() {
        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "Abc").unwrap(),
            &[
                (Rule::accessor_expression, "Abc"),
                (Rule::identifier_literal, "Abc"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc_123").unwrap(),
            &[
                (Rule::accessor_expression, "abc_123"),
                (Rule::identifier_literal, "abc_123"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "_abc").unwrap(),
            &[
                (Rule::accessor_expression, "_abc"),
                (Rule::identifier_literal, "_abc"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "array[0]").unwrap(),
            &[
                (Rule::accessor_expression, "array[0]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "0"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "array[-1]").unwrap(),
            &[
                (Rule::accessor_expression, "array[-1]"),
                (Rule::identifier_literal, "array"),
                (Rule::integer_literal, "-1"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name"),
            ],
        );

        validate_rule(
            KqlParser::parse(Rule::accessor_expression, "abc.name1.name2").unwrap(),
            &[
                (Rule::accessor_expression, "abc.name1.name2"),
                (Rule::identifier_literal, "abc"),
                (Rule::identifier_literal, "name1"),
                (Rule::identifier_literal, "name2"),
            ],
        );

        validate_rule(
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

        assert!(KqlParser::parse(Rule::accessor_expression, "123").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "+name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "-name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, "~name").is_err());
        assert!(KqlParser::parse(Rule::accessor_expression, ".name").is_err());
    }

    fn validate_rule(parsed: Pairs<'_, Rule>, expected: &[(Rule, &str)]) {
        let flat = parsed.flatten();

        assert_eq!(flat.len(), expected.len());

        for (index, rule) in flat.enumerate() {
            let expected = expected.get(index).unwrap();

            assert!(discriminant(&rule.as_rule()) == discriminant(&expected.0));
            assert_eq!(rule.as_str(), expected.1);
        }
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use chrono::{DateTime, Datelike, NaiveDate, Utc};
    use pest::Parser;

    #[test]
    fn test_parse_double_literal() {
        let run_test = |input: &str, expected: f64| {
            let mut result = KqlParser::parse(Rule::double_literal, input).unwrap();

            let f = parse_double_literal(result.next().unwrap());

            assert!(f.is_ok());
            assert_eq!(expected, f.unwrap());
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

            let i = parse_integer_literal(result.next().unwrap());

            assert!(i.is_ok());
            assert_eq!(expected, i.unwrap());
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

        if let Error::SyntaxError(_, _) = i.unwrap_err() {
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let run_test = |input: &str, expected: &str| {
            let mut result = KqlParser::parse(Rule::string_literal, input).unwrap();

            let actual = parse_string_literal(result.next().unwrap());

            assert_eq!(expected, actual);
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

            let expression = parse_datetime_expression(result.next().unwrap()).unwrap();

            if let ScalarExpression::DateTime(d) = expression {
                assert_eq!(expected, d.get_value());
            } else {
                panic!("Expected DateTimeScalarExpression");
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

            let expression = parse_real_expression(result.next().unwrap()).unwrap();

            if let ScalarExpression::Double(d) = expression {
                assert_eq!(expected, d.get_value());
            } else {
                panic!("Expected DoubleScalarExpression");
            }
        };

        run_test("real(1.0)", 1.0);
        run_test("real(1.0e10)", 1.0e10);
        run_test("real(1)", 1.0);
        run_test("real(+inf)", f64::INFINITY);
        run_test("real(-inf)", f64::NEG_INFINITY);
    }

    #[test]
    fn test_parse_accessor_expression_from_source() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "source.subkey['array'][0]").unwrap();

        let expression =
            parse_accessor_expression(result.next().unwrap(), &KqlParserState::new()).unwrap();

        if let ScalarExpression::Source(path) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(7, 13, 1, 8),
                        "subkey"
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(14, 21, 1, 15),
                        "array"
                    )),
                    ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                        QueryLocation::new(23, 24, 1, 24),
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

        let mut state = KqlParserState::new();

        state.push_variable_name("var");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        let mut v = ValueAccessor::new();
        v.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new(13, 21, 1, 14),
            "neg_attr",
        )));

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 22, 1, 1),
                        "subkey"
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Variable(
                        VariableScalarExpression::new(
                            QueryLocation::new(7, 10, 1, 8),
                            "var",
                            ValueAccessor::new()
                        )
                    )),
                    ValueSelector::ScalarExpression(ScalarExpression::Negate(
                        NegateScalarExpression::new(
                            QueryLocation::new(12, 13, 1, 13),
                            ScalarExpression::Source(SourceScalarExpression::new(
                                QueryLocation::new(13, 21, 1, 14),
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
            &KqlParserState::new().with_default_source_map_key_name("attributes"),
        )
        .unwrap();

        let mut v = ValueAccessor::new();
        v.push_selector(ValueSelector::MapKey(StringScalarExpression::new(
            QueryLocation::new(13, 21, 1, 14),
            "neg_attr",
        )));

        if let ScalarExpression::Source(s) = expression {
            assert_eq!(
                &[
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 6, 1, 1),
                        "attributes"
                    )),
                    ValueSelector::MapKey(StringScalarExpression::new(
                        QueryLocation::new(0, 6, 1, 1),
                        "subkey"
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
    fn test_parse_accessor_expression_from_attached() {
        let mut result =
            KqlParser::parse(Rule::accessor_expression, "resource['~at\\'tr~']").unwrap();

        let expression = parse_accessor_expression(
            result.next().unwrap(),
            &KqlParserState::new().with_attached_data_names(&["resource"]),
        )
        .unwrap();

        if let ScalarExpression::Attached(a) = expression {
            assert_eq!("resource", a.get_name());
            assert_eq!(
                &[ValueSelector::MapKey(StringScalarExpression::new(
                    QueryLocation::new(9, 19, 1, 10),
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

        let mut state = KqlParserState::new();

        state.push_variable_name("a");

        let expression = parse_accessor_expression(result.next().unwrap(), &state).unwrap();

        if let ScalarExpression::Variable(v) = expression {
            assert_eq!("a", v.get_name());
            assert_eq!(
                &[ValueSelector::ArrayIndex(IntegerScalarExpression::new(
                    QueryLocation::new(2, 4, 1, 3),
                    -1
                ))]
                .to_vec(),
                v.get_selectors()
            );
        } else {
            panic!("Expected VariableScalarExpression");
        }
    }
}
