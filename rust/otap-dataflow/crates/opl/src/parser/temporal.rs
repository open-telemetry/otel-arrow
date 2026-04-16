// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use data_engine_expressions::QueryLocation;
use data_engine_parser_abstractions::ParserError;

/// Number of milliseconds in a nanosecond
const MINUTES_PER_SECOND: i32 = 60;
const SECONDS_PER_HOUR: i32 = 3600;
const MILLIS_PER_NANO_SECOND: u32 = 1_000_000;

pub(crate) fn parse_date_time(
    str_value: &str,
    query_location: &QueryLocation,
) -> Result<DateTime<FixedOffset>, ParserError> {
    let Ok(iso8601::DateTime {
        date: parsed_date,
        time: parsed_time,
    }) = iso8601::datetime(str_value)
    else {
        return Err(ParserError::SyntaxError(
            query_location.clone(),
            format!("Invalid datetime literal {str_value:?}"),
        ));
    };

    let naive_date = match parsed_date {
        iso8601::Date::YMD { year, month, day } => NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| {
                ParserError::SyntaxError(
                    query_location.clone(),
                    format!("Invalid date {parsed_date}"),
                )
            })?,
        other => {
            let format_name = match other {
                iso8601::Date::Ordinal { .. } => "Ordinal",
                iso8601::Date::Week { .. } => "Week",
                // safety: this case handled by outer match statement
                _ => unreachable!("unreachable"),
            };
            // Currently Week or Ordinal date format not yet supported
            return Err(ParserError::SyntaxNotSupported(
                query_location.clone(),
                format!("Date format {format_name} not yet supported"),
            ));
        }
    };
    // Convert time
    let naive_time = NaiveTime::from_hms_nano_opt(
        parsed_time.hour,
        parsed_time.minute,
        parsed_time.second,
        parsed_time.millisecond * MILLIS_PER_NANO_SECOND,
    )
    .ok_or_else(|| {
        ParserError::SyntaxError(
            query_location.clone(),
            format!("Invalid time {parsed_time}"),
        )
    })?;

    let naive_dt = naive_date.and_time(naive_time);

    // Convert timezone
    let dt = match parsed_time.tz_offset_hours {
        0 if parsed_time.tz_offset_minutes == 0 => {
            // UTC
            Utc.from_utc_datetime(&naive_dt).fixed_offset()
        }
        _ => {
            let offset_secs = parsed_time.tz_offset_hours * SECONDS_PER_HOUR
                + parsed_time.tz_offset_minutes * MINUTES_PER_SECOND;
            let offset = FixedOffset::east_opt(offset_secs).ok_or_else(|| {
                ParserError::SyntaxError(
                    query_location.clone(),
                    format!("Invalid offset {offset_secs:?}"),
                )
            })?;
            offset
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| {
                    ParserError::SyntaxError(
                        query_location.clone(),
                        format!("Invalid offset {naive_dt:?}"),
                    )
                })?
        }
    };
    Ok(dt)
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
    use data_engine_expressions::{DateTimeValue, ScalarExpression, StaticScalarExpression};
    use pest::Parser;

    use crate::parser::{Rule, expression::parse_tagged_literal, pest::OplPestParser};

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
            // ISO 8601 with time
            ("2026-02-04T05:30:00", create_utc(2026, 2, 4, 5, 30, 0, 0)),
            // Explicit timezone offsets - for good measure test one  case during daylight savings
            // and one case not during daylight savings ...
            (
                "2026-02-04T05:30:00-05:00",
                create_with_tz(2026, 2, 4, 5, 30, 0, 0, Canada::Eastern),
            ),
            (
                "2026-07-31T11:15-04:00",
                create_with_tz(2026, 7, 31, 11, 15, 0, 0, Canada::Eastern),
            ),
        ]
    }

    fn run_test_success(expr: &str, expected: DateTime<FixedOffset>) {
        let mut result = OplPestParser::parse(Rule::tagged_literal, expr).unwrap();
        match parse_tagged_literal(result.next().unwrap(), QueryLocation::new_fake()).unwrap() {
            ScalarExpression::Static(StaticScalarExpression::DateTime(d)) => assert_eq!(
                d.get_value(),
                expected,
                "failed to correctly parse expr {expr:?}"
            ),
            s => panic!("Unexpected scalar expr. Expected DateTime found {s:?}"),
        }
    }

    fn run_test_failure(expr: &str) -> ParserError {
        let mut result = OplPestParser::parse(Rule::tagged_literal, expr).unwrap();
        parse_tagged_literal(result.next().unwrap(), QueryLocation::new_fake()).unwrap_err()
    }

    #[test]
    fn test_parse_from_datetime_literal() {
        for (input, expected) in valid_test_cases() {
            let expr = format!("date_time\"{input}\"");
            run_test_success(&expr, expected);
        }
    }

    #[test]
    fn test_parse_from_invalid_datetime_date_only() {
        let err = run_test_failure("date_time\"2026-02-04\""); // not a valid datetime format
        assert_eq!("Invalid datetime literal \"2026-02-04\"", err.to_string())
    }

    #[test]
    fn test_parse_from_invalid_datetime_literal() {
        let err = run_test_failure("date_time\"halloween\""); // not a valid datetime format
        assert_eq!("Invalid datetime literal \"halloween\"", err.to_string())
    }

    #[test]
    fn test_parse_from_invalid_date_literal() {
        // Feb 30th an invalid day
        let err = run_test_failure("date_time\"2026-02-30T05:30:00\"");
        assert_eq!("Invalid date 2026-02-30", err.to_string())
    }

    #[test]
    fn test_parse_from_invalid_time_literal() {
        // 24:00 o'clock, an invalid time
        let err = run_test_failure("date_time\"2026-02-04T24:00:00\"");
        assert_eq!("Invalid time 24:00:00.0+00:00", err.to_string())
    }

    #[test]
    fn test_invalid_timezone() {
        // timezone 24 hours and 1 minute behind UTC = invalid
        let err = run_test_failure("date_time\"2026-02-04T05:30:00-24:01\"");
        assert_eq!("Invalid offset -86460", err.to_string())
    }

    /// Test some "valid" iso 8601 date formats that we don't support yet
    #[test]
    fn test_parse_from_ordinal_and_week_datetime_literal() {
        // Ordinal date format = 74th day of 2024 = March 14, 2024
        let err = run_test_failure("date_time\"2024074T00:00:00\"");
        assert_eq!("Date format Ordinal not yet supported", err.to_string());

        // Week date format = 2024, week 11, Friday = March 15, 2024
        let err = run_test_failure("date_time\"2024-W11-5T00:00:00\"");
        assert_eq!("Date format Week not yet supported", err.to_string());
    }
}
