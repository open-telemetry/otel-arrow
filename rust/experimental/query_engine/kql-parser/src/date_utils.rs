use std::{ops::Range, str::FromStr, sync::LazyLock};

use chrono::{Datelike, Month, Utc};
use regex::Regex;

static ISO_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(\\d+)[-\\/](\\d+)[-\\/](\\d+)").unwrap());
static RFC_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(\\d+)[- ]([A-Za-z]+)[- ](\\d+)").unwrap());
static LOCAL_DATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("([A-Za-z])+[- ](\\d+),?[ \\-](\\d+)").unwrap());
static ISO_TIME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(?:^|[^-+:\\d])(\\d+):(\\d+)(?::(\\d+)(?:\\.(\\d+))?)?").unwrap());
static LOCAL_TIME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("(\\d+)(?::(\\d+))?\\s*([AaPp][Mm])").unwrap());
static ISO_TIME_OFFSET_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("([-+])(\\d+)(?::(\\d+))?").unwrap());

pub(crate) fn parse_date(input: &str) -> Result<(u32, u32, u32, Range<usize>), ()> {
    let iso = ISO_DATE_REGEX.captures(input);
    if !iso.is_none() {
        let captures = iso.unwrap();

        let r = captures.get(0).unwrap().range();

        let a = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let b = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let mut c = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();
        if a > 99 {
            return Ok((b, c, a, r));
        } else {
            if c < 50 {
                c = c + 2000;
            } else if c < 100 {
                c = c + 1900;
            }
            return Ok((a, b, c, r));
        }
    }

    let rfc = RFC_DATE_REGEX.captures(input);
    if !rfc.is_none() {
        let captures = rfc.unwrap();

        let r = captures.get(0).unwrap().range();

        let day = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let month = Month::from_str(captures.get(2).unwrap().as_str());
        let mut year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        if year < 50 {
            year = year + 2000;
        } else if year < 100 {
            year = year + 1900;
        }
        return Ok((month.unwrap().number_from_month(), day, year, r));
    }

    let local = LOCAL_DATE_REGEX.captures(input);
    if !local.is_none() {
        let captures = local.unwrap();

        let r = captures.get(0).unwrap().range();

        let month = Month::from_str(captures.get(1).unwrap().as_str());
        let day = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let mut year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        if year < 50 {
            year = year + 2000;
        } else if year < 100 {
            year = year + 1900;
        }
        return Ok((month.unwrap().number_from_month(), day, year, r));
    }

    let now = Utc::now();
    Ok((now.month(), now.day(), now.year() as u32, 0..0))
}

pub(crate) fn parse_time(input: &str) -> Result<(u32, u32, u32, u32, Range<usize>), ()> {
    let local = LOCAL_TIME_REGEX.captures(input);
    if !local.is_none() {
        let captures = local.unwrap();

        let r = captures.get(0).unwrap().range();

        let mut hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();

        let mut minute = 0;
        let cminute = captures.get(2);
        if !cminute.is_none() {
            minute = cminute.unwrap().as_str().parse::<u32>().unwrap();
        }

        if captures.get(3).unwrap().as_str().to_lowercase() == "pm" {
            hour = hour + 12;
        }

        return Ok((hour, minute, 0, 0, r));
    }

    let iso = ISO_TIME_REGEX.captures(input);
    if !iso.is_none() {
        let captures = iso.unwrap();

        let r = captures.get(0).unwrap().range();

        let hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let minute = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();

        let mut seconds = 0;
        let cseconds = captures.get(3);
        if !cseconds.is_none() {
            seconds = cseconds.unwrap().as_str().parse::<u32>().unwrap();
        }

        let mut micro = 0;
        let cmicro = captures.get(4);
        if !cmicro.is_none() {
            micro = cmicro.unwrap().as_str().parse::<u32>().unwrap();
        }

        return Ok((hour, minute, seconds, micro, r));
    }

    Ok((0, 0, 0, 0, 0..0))
}

pub(crate) fn parse_offset(input: &str) -> i32 {
    let mut offset: i32 = 0;
    let c = ISO_TIME_OFFSET_REGEX.captures(&input);
    if !c.is_none() {
        let captures = c.unwrap();

        let mut multipler: i32 = 1;
        if captures.get(1).unwrap().as_str() == "-" {
            multipler = -1;
        }

        let hours = captures.get(2).unwrap().as_str().parse::<i32>().unwrap();

        let mut minutes: i32 = 0;
        let cminutes = captures.get(3);
        if !cminutes.is_none() {
            minutes = cminutes.unwrap().as_str().parse::<i32>().unwrap();
        }

        offset = (multipler * hours * 60 * 60) + (minutes * 60);
    }

    offset
}
