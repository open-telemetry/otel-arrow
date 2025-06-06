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

fn expand_year(mut year: u32) -> u32 {
    if year < 50 {
        year += 2000;
    } else if year < 100 {
        year += 1900;
    }
    year
}

pub(crate) fn parse_date(input: &str) -> Result<(u32, u32, u32, Range<usize>), ()> {
    let iso = ISO_DATE_REGEX.captures(input);
    if iso.is_some() {
        let captures = iso.unwrap();

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
    if rfc.is_some() {
        let captures = rfc.unwrap();

        let r = captures.get(0).unwrap().range();

        let day = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let month = Month::from_str(captures.get(2).unwrap().as_str());
        let year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        return Ok((month.unwrap().number_from_month(), day, expand_year(year), r));
    }

    let local = LOCAL_DATE_REGEX.captures(input);
    if local.is_some() {
        let captures = local.unwrap();

        let r = captures.get(0).unwrap().range();

        let month = Month::from_str(captures.get(1).unwrap().as_str());
        let day = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
        let year = captures.get(3).unwrap().as_str().parse::<u32>().unwrap();

        if month.is_err() {
            return Err(());
        }

        return Ok((month.unwrap().number_from_month(), day, expand_year(year), r));
    }

    let now = Utc::now();
    Ok((now.month(), now.day(), now.year() as u32, 0..0))
}

pub(crate) fn parse_time(input: &str) -> Result<(u32, u32, u32, u32, Range<usize>), ()> {
    let local = LOCAL_TIME_REGEX.captures(input);
    if local.is_some() {
        let captures = local.unwrap();

        let r = captures.get(0).unwrap().range();

        let mut hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();

        let mut minute = 0;
        let cminute = captures.get(2);
        if cminute.is_some() {
            minute = cminute.unwrap().as_str().parse::<u32>().unwrap();
        }

        if captures.get(3).unwrap().as_str().to_lowercase() == "pm" {
            hour += 12;
        }

        return Ok((hour, minute, 0, 0, r));
    }

    let iso = ISO_TIME_REGEX.captures(input);
    if iso.is_some() {
        let captures = iso.unwrap();

        let r = captures.get(0).unwrap().range();

        let hour = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let minute = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();

        let mut seconds = 0;
        let cseconds = captures.get(3);
        if cseconds.is_some() {
            seconds = cseconds.unwrap().as_str().parse::<u32>().unwrap();
        }

        let mut micro = 0;
        let cmicro = captures.get(4);
        if cmicro.is_some() {
            micro = cmicro.unwrap().as_str().parse::<u32>().unwrap();
        }

        return Ok((hour, minute, seconds, micro, r));
    }

    Ok((0, 0, 0, 0, 0..0))
}

pub(crate) fn parse_offset(input: &str) -> i32 {
    let mut offset: i32 = 0;
    let c = ISO_TIME_OFFSET_REGEX.captures(input);
    if c.is_some() {
        let captures = c.unwrap();

        let mut multipler: i32 = 1;
        if captures.get(1).unwrap().as_str() == "-" {
            multipler = -1;
        }

        let hours = captures.get(2).unwrap().as_str().parse::<i32>().unwrap();

        let mut minutes: i32 = 0;
        let cminutes = captures.get(3);
        if cminutes.is_some() {
            minutes = cminutes.unwrap().as_str().parse::<i32>().unwrap();
        }

        offset = (multipler * hours * 60 * 60) + (minutes * 60);
    }

    offset
}
