// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, str::FromStr, sync::LazyLock};

use chrono::{DateTime, FixedOffset, NaiveDate};
use chrono::{Datelike, Month, Utc};
use regex::Regex;

static ISO_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(\\d+)[-\\/](\\d+)[-\\/](\\d+)").unwrap());
static RFC_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(?:\\w*, )?(\\d+)[- ]([A-Za-z]+)[- ](\\d+)").unwrap());
static LOCAL_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("([A-Za-z])+[- ](\\d+),?[ \\-](\\d+)").unwrap());
static ISO_TIME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("(?:^|[^-+:\\d])(\\d+):(\\d+)(?::(\\d+)(?:\\.(\\d+))?)?(?:Z|(?: GMT))?").unwrap()
});
static LOCAL_TIME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(\\d+)(?::(\\d+))?\\s*([AaPp][Mm])").unwrap());
static ISO_TIME_OFFSET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("([-+])(\\d+)(?::(\\d+))?").unwrap());

fn expand_year(mut year: u32) -> u32 {
    if year < 50 {
        year += 2000;
    } else if year < 100 {
        year += 1900;
    }
    year
}

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

pub(crate) fn parse_date_time(input: &str) -> Result<DateTime<FixedOffset>, ()> {
    let mut raw_value: String = input.into();

    let date = parse_date(&raw_value);
    if date.is_err() {
        return Err(());
    }

    let (month, day, year, range) = date.unwrap();

    raw_value.replace_range(range, "");

    let time = parse_time(&raw_value);
    if time.is_err() {
        return Err(());
    }

    let (hour, min, sec, micro, range) = time.unwrap();

    raw_value.replace_range(range, "");

    let offset = match parse_offset(&raw_value) {
        Some((o, r)) => {
            raw_value.replace_range(r, "");
            o
        }
        None => 0,
    };

    if !raw_value.trim().is_empty() {
        return Err(());
    }

    let nd = NaiveDate::from_ymd_opt(year as i32, month, day);
    if nd.is_none() {
        return Err(());
    }

    let ndt = nd.unwrap().and_hms_micro_opt(hour, min, sec, micro);

    if ndt.is_none() {
        return Err(());
    }

    let tz = FixedOffset::east_opt(offset);
    if tz.is_none() {
        return Err(());
    }

    let l = ndt.unwrap().and_local_timezone(tz.unwrap());

    match l {
        chrono::offset::LocalResult::Single(date_time) => Ok(date_time),
        _ => Err(()),
    }
}

fn parse_date(input: &str) -> Result<(u32, u32, u32, Range<usize>), ()> {
    let iso = ISO_DATE_REGEX.captures(input);
    if let Some(captures) = iso {
        let r = captures.get(0).unwrap().range();

        let a = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let b = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let c = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
        if a > 99 {
            return Ok((b, c, a, r));
        } else {
            return Ok((a, b, expand_year(c), r));
        }
    }

    let rfc = RFC_DATE_REGEX.captures(input);
    if let Some(captures) = rfc {
        let r = captures.get(0).unwrap().range();

        let day = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let month = Month::from_str(captures.get(2).unwrap().as_str());
        let year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        return Ok((
            month.unwrap().number_from_month(),
            day,
            expand_year(year),
            r,
        ));
    }

    let local = LOCAL_DATE_REGEX.captures(input);
    if let Some(captures) = local {
        let r = captures.get(0).unwrap().range();

        let month = Month::from_str(captures.get(1).unwrap().as_str());
        let day = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        return Ok((
            month.unwrap().number_from_month(),
            day,
            expand_year(year),
            r,
        ));
    }

    let now = Utc::now();
    Ok((now.month(), now.day(), now.year() as u32, 0..0))
}

fn parse_time(input: &str) -> Result<(u32, u32, u32, u32, Range<usize>), ()> {
    let local = LOCAL_TIME_REGEX.captures(input);
    if let Some(captures) = local {
        let r = captures.get(0).unwrap().range();

        let mut hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();

        let mut minute = 0;
        let cminute = captures.get(2);
        if let Some(capture) = cminute {
            minute = capture.as_str().parse::<u32>().unwrap();
        }

        if captures.get(3).unwrap().as_str().to_lowercase() == "pm" {
            hour += 12;
        }

        return Ok((hour, minute, 0, 0, r));
    }

    let iso = ISO_TIME_REGEX.captures(input);
    if let Some(captures) = iso {
        let r = captures.get(0).unwrap().range();

        let hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let minute = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();

        let mut seconds = 0;
        let cseconds = captures.get(3);
        if let Some(cseconds) = cseconds {
            seconds = cseconds.as_str().parse::<u32>().unwrap();
        }

        let mut micro = 0;
        let cmicro = captures.get(4);
        if let Some(cmicro) = cmicro {
            micro = cmicro.as_str().parse::<u32>().unwrap();
        }

        return Ok((hour, minute, seconds, micro, r));
    }

    Ok((0, 0, 0, 0, 0..0))
}

fn parse_offset(input: &str) -> Option<(i32, Range<usize>)> {
    let c = ISO_TIME_OFFSET_REGEX.captures(input);
    if let Some(captures) = c {
        let r = captures.get(0).unwrap().range();

        let mut multipler: i32 = 1;
        if captures.get(1).unwrap().as_str() == "-" {
            multipler = -1;
        }

        let hours = captures.get(2).unwrap().as_str().parse::<i32>().unwrap();

        let mut minutes: i32 = 0;
        let cminutes = captures.get(3);
        if let Some(cminutes) = cminutes {
            minutes = cminutes.as_str().parse::<i32>().unwrap();
        }

        let offset = (multipler * hours * 60 * 60) + (minutes * 60);

        return Some((offset, r));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_time() {
        let run_test_success = |input: &str, expected: DateTime<FixedOffset>| {
            let actual = parse_date_time(input).unwrap();

            assert_eq!(expected, actual)
        };

        let run_test_failure = |input: &str| {
            parse_date_time(input).unwrap_err();
        };

        run_test_failure("hello world");
        run_test_failure("it was a good date (8/5/2025) to be alive");

        let now = Utc::now();

        run_test_success("12/31/2025", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test_success("12/31/50", create_utc(1950, 12, 31, 0, 0, 0, 0));
        run_test_success("12/31/49", create_utc(2049, 12, 31, 0, 0, 0, 0));
        run_test_success("2025/12/31", create_utc(2025, 12, 31, 0, 0, 0, 0));
        run_test_success(
            "2025/12/31 22:30:10.1",
            create_utc(2025, 12, 31, 22, 30, 10, 1),
        );
        run_test_success("12-31-2025 10AM", create_utc(2025, 12, 31, 10, 0, 0, 0));
        run_test_success(
            "2025-12-31 10:30 PM",
            create_utc(2025, 12, 31, 22, 30, 0, 0),
        );
        run_test_success(
            "10PM",
            create_utc(now.year(), now.month(), now.day(), 22, 0, 0, 0),
        );

        // ISO 8601
        run_test_success(
            "2014-05-25T08:20:03.123456Z",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test_success(
            "2009-06-15T13:45:30.0000000-07:00",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, -7 * 60 * 60),
        );
        run_test_success(
            "2009-06-15T13:45:30.0000000+07:30",
            create_fixed(2009, 6, 15, 13, 45, 30, 0, (7 * 60 * 60) + (30 * 60)),
        );
        run_test_success(
            "2014-05-25T08:20:03.123456",
            create_utc(2014, 5, 25, 8, 20, 3, 123456),
        );
        run_test_success("2014-05-25T08:20", create_utc(2014, 5, 25, 8, 20, 0, 0));
        run_test_success(
            "2014-11-08 15:55:55.123456Z",
            create_utc(2014, 11, 8, 15, 55, 55, 123456),
        );
        run_test_success(
            "2014-11-08 15:55:55",
            create_utc(2014, 11, 8, 15, 55, 55, 0),
        );
        run_test_success("2014-11-08 15:55", create_utc(2014, 11, 8, 15, 55, 0, 0));
        run_test_success("2014-11-08", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 822
        run_test_success(
            "Sat, 8 Nov 14 15:05:02 GMT",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "Sat, 8 Nov 14 15:05:02",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "8 Nov 14 15:05:02 GMT",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success("8 Nov 14 15:05:02", create_utc(2014, 11, 8, 15, 5, 2, 0));
        run_test_success("8 Nov 14 15:05 GMT", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("8 Nov 14 15:05", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("8 Nov 14", create_utc(2014, 11, 8, 0, 0, 0, 0));

        // RFC 850
        run_test_success(
            "Saturday, 08-Nov-14 15:05:02 GMT",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "Saturday, 08-Nov-14 15:05:02",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success(
            "Saturday, 08-Nov-14 15:05 GMT",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "Saturday, 08-Nov-14 15:05",
            create_utc(2014, 11, 8, 15, 5, 0, 0),
        );
        run_test_success(
            "08-Nov-14 15:05:02 GMT",
            create_utc(2014, 11, 8, 15, 5, 2, 0),
        );
        run_test_success("08-Nov-14 15:05:02", create_utc(2014, 11, 8, 15, 5, 2, 0));
        run_test_success("08-Nov-14 15:05 GMT", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("08-Nov-14 15:05", create_utc(2014, 11, 8, 15, 5, 0, 0));

        // Sortable
        run_test_success("2014-11-08 15:05:25", create_utc(2014, 11, 8, 15, 5, 25, 0));
        run_test_success(
            "2014-11-08 15:05:25 GMT",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success("2014-11-08 15:05", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("2014-11-08 15:05 GMT", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("2014-11-08T15:05:25", create_utc(2014, 11, 8, 15, 5, 25, 0));
        run_test_success(
            "2014-11-08T15:05:25 GMT",
            create_utc(2014, 11, 8, 15, 5, 25, 0),
        );
        run_test_success("2014-11-08T15:05", create_utc(2014, 11, 8, 15, 5, 0, 0));
        run_test_success("2014-11-08T15:05 GMT", create_utc(2014, 11, 8, 15, 5, 0, 0));
    }
}
