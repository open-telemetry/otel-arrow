// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

pub(super) fn positive_timestamp(timestamp: i64) -> Result<i64, Error> {
    if timestamp > 0 {
        Ok(timestamp)
    } else {
        Err(Error::Timestamp("timestamp must be positive"))
    }
}

pub(super) fn cumulative_start_time(
    start_time_unix_nano: &mut i64,
    previous_timestamp_unix_nano: &mut i64,
    timestamp_unix_nano: i64,
) -> i64 {
    if timestamp_unix_nano < *previous_timestamp_unix_nano {
        *start_time_unix_nano = timestamp_unix_nano;
    }
    *previous_timestamp_unix_nano = timestamp_unix_nano;
    *start_time_unix_nano
}
