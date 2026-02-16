// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::date_utils::parse_date_time;
use data_engine_expressions::{DateTimeScalarExpression, StaticScalarExpression, StringValue};
use data_engine_parser_abstractions::{
    ParserError, parse_standard_string_literal, to_query_location,
};
use pest::iterators::Pair;

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
                _ => {
                    todo!("missed the buss")
                }
            }
        }
        _ => {
            todo!("AH HELL MA")
        }
    };

    Ok(StaticScalarExpression::DateTime(
        DateTimeScalarExpression::new(rule_query_location, datetime_value),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
    use data_engine_expressions::DateTimeValue;
    use pest::Parser;

    use crate::parser::pest::OplPestParser;

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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_fixed(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        micro: u32,
        offset: i32,
    ) -> DateTime<FixedOffset> {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_micro_opt(hour, min, sec, micro)
            .unwrap()
            .and_local_timezone(FixedOffset::east_opt(offset).unwrap())
            .unwrap()
    }

    fn valid_test_cases() -> Vec<(&'static str, DateTime<FixedOffset>)> {
        vec![
            // mid-endian (US) date format
            ("02/04/2026", create_utc(2026, 02, 04, 0, 0, 0, 0)),
            // mid-endian (US) date format with time
            ("02/04/2026 5:30 AM", create_utc(2026, 02, 04, 5, 30, 0, 0)),
            // ISO 8601
            ("2026-02-04", create_utc(2026, 02, 04, 0, 0, 0, 0)),
            // ISO 8601 with time
            ("2026-02-04T05:30:00", create_utc(2026, 02, 04, 5, 30, 0, 0)),
        ]
    }

    #[test]
    fn test_parse_from_string_literal() {
        for (input, expected) in valid_test_cases() {
            let expr = format!("datetime('{input}')");
            let mut result = OplPestParser::parse(Rule::datetime_expression, &expr).unwrap();
            match parse_datetime_expression(result.next().unwrap()).unwrap() {
                StaticScalarExpression::DateTime(d) => assert_eq!(d.get_value(), expected),
                s => panic!("Unexpected scalar expr. Expected DateTime found {s:?}"),
            }
        }
    }

    #[test]
    fn test_parse_from_invalid_string_literal() {}

    #[test]
    fn test_parse_from_datetime_literal() {}

    #[test]
    fn test_parse_from_invalid_datetime_literal() {}
}
