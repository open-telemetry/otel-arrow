// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::date_utils::parse_date_time;
use data_engine_expressions::{DateTimeScalarExpression, StaticScalarExpression, StringValue};
use data_engine_parser_abstractions::{
    ParserError, parse_standard_string_literal, to_query_location,
};
use pest::iterators::Pair;

#[cfg(test)]
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

use crate::parser::invalid_child_rule_error;
use crate::parser::{Rule, expression::no_inner_rule_error};

pub(crate) fn parse_datetime_expression(
    datetime_expression_rule: Pair<'_, Rule>,
) -> Result<StaticScalarExpression, ParserError> {
    let query_location = to_query_location(&datetime_expression_rule);
    let mut inner_rules = datetime_expression_rule.into_inner();
    let rule = inner_rules
        .next()
        .ok_or_else(|| no_inner_rule_error(query_location))?;
    let rule_query_location = to_query_location(&rule);

    let datetime_value = match rule.as_rule() {
        Rule::datetime_literal => {
            let datetime_value = rule.as_str();
            parse_date_time(datetime_value).map_err(|_e| {
                ParserError::SyntaxError(
                    rule_query_location.clone(),
                    format!("Invalid datetime literal {datetime_value}"),
                )
            })?
        }
        Rule::string_literal => {
            let static_str_scalar_expr = parse_standard_string_literal(rule);
            match static_str_scalar_expr {
                StaticScalarExpression::String(str) => {
                    let str_value = str.get_value();
                    parse_date_time(str_value).map_err(|_e| {
                        ParserError::SyntaxError(
                            rule_query_location.clone(),
                            format!("Invalid datetime string literal {str_value:?}"),
                        )
                    })?
                }
                invalid_expr => {
                    return Err(ParserError::SyntaxError(
                        rule_query_location,
                        format!("Expected static string literal, found {:?}", invalid_expr),
                    ));
                }
            }
        }
        other_rule => {
            return Err(invalid_child_rule_error(
                rule_query_location.clone(),
                Rule::datetime_expression,
                other_rule,
            ));
        }
    };

    Ok(StaticScalarExpression::DateTime(
        DateTimeScalarExpression::new(rule_query_location, datetime_value),
    ))
}

/// test helper for creating [`DateTime`]s
#[cfg(test)]
pub(crate) fn create_utc(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    micro: u32,
) -> DateTime<FixedOffset> {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_micro_opt(hour, min, sec, micro)
        .unwrap()
        .and_local_timezone(Utc)
        .unwrap()
        .into()
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::TimeZone;
    use chrono_tz::{Canada, Tz};
    use data_engine_expressions::DateTimeValue;
    use pest::Parser;

    use crate::parser::pest::OplPestParser;

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_with_tz(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        micro: u32,
        tz: Tz,
    ) -> DateTime<FixedOffset> {
        let native_dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_micro_opt(hour, min, sec, micro)
            .unwrap();
        tz.from_local_datetime(&native_dt).unwrap().fixed_offset()
    }

    /// Generate a few valid datetime expressions, as well as the expected `DateTime` we expect
    /// them to be parsed as.
    ///
    /// Note: a more comprehensive suite of formats already exists for the [`parse_date_time`]
    /// utility to which [`parse_datetime_expression`] delegates
    fn valid_test_cases() -> Vec<(&'static str, DateTime<FixedOffset>)> {
        vec![
            // mid-endian (US) date format
            ("02/04/2026", create_utc(2026, 2, 4, 0, 0, 0, 0)),
            // mid-endian (US) date format with time
            ("02/04/2026 5:30 AM", create_utc(2026, 2, 4, 5, 30, 0, 0)),
            // ISO 8601
            ("2026-02-04", create_utc(2026, 2, 4, 0, 0, 0, 0)),
            // ISO 8601 with time
            ("2026-02-04T05:30:00", create_utc(2026, 2, 4, 5, 30, 0, 0)),
            // RFC 822
            ("4 Feb 26 15:05", create_utc(2026, 2, 4, 15, 5, 0, 0)),
            // RFC 822 with time and day of week
            (
                "Wed, 4 Feb 26 15:05:02 GMT",
                create_utc(2026, 2, 4, 15, 5, 2, 0),
            ),
            // Explicit timezone offsets
            (
                "2026-02-04T05:30:00-05:00",
                create_with_tz(2026, 2, 4, 5, 30, 0, 0, Canada::Eastern),
            ),
            (
                "2026-07-31T11:15-04:00",
                create_with_tz(2026, 7, 31, 11, 15, 0, 0, Canada::Eastern),
            ),
            // TODO reenable this test case when TZ parsing bug fixed:
            // https://github.com/open-telemetry/otel-arrow/issues/2047
            // (
            //     "Wed, 4 Feb 26 15:05:02 EST",
            //     create_with_tz(2026, 02, 04, 15, 5, 2, 0, Canada::Eastern),
            // ),
        ]
    }

    fn run_test_success(expr: &str, expected: DateTime<FixedOffset>) {
        let mut result = OplPestParser::parse(Rule::datetime_expression, expr).unwrap();
        match parse_datetime_expression(result.next().unwrap()).unwrap() {
            StaticScalarExpression::DateTime(d) => assert_eq!(
                d.get_value(),
                expected,
                "failed to correctly parse expr {expr:?}"
            ),
            s => panic!("Unexpected scalar expr. Expected DateTime found {s:?}"),
        }
    }

    fn run_test_failure(expr: &str) -> ParserError {
        let mut result = OplPestParser::parse(Rule::datetime_expression, expr).unwrap();
        parse_datetime_expression(result.next().unwrap()).unwrap_err()
    }

    #[test]
    fn test_parse_from_datetime_literal() {
        for (input, expected) in valid_test_cases() {
            let expr = format!("datetime({input})");
            run_test_success(&expr, expected);
        }
    }

    #[test]
    fn test_parse_from_invalid_datetime_literal() {
        let err = run_test_failure("datetime(halloween)"); // not a valid date format
        assert_eq!("Invalid datetime literal halloween", err.to_string())
    }

    #[test]
    fn test_parse_from_string_literal() {
        for (input, expected) in valid_test_cases() {
            let expr = format!("datetime('{input}')");
            run_test_success(&expr, expected);
        }
    }

    #[test]
    fn test_parse_from_invalid_string_literal() {
        let err = run_test_failure("datetime('monday')"); // not a valid date format
        assert_eq!(
            "Invalid datetime string literal \"monday\"",
            err.to_string()
        )
    }
}
