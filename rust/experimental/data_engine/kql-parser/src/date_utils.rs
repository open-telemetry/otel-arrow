// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

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
