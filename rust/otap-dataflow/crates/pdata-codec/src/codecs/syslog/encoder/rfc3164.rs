// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! RFC3164 timestamp and header serialization.

use std::io::Write;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::Tz;

use super::fields::{FieldValue, Header, append_bytes, header_text};
use super::{CodecError, MAX_MESSAGE_SIZE, error, error_source};

pub(super) fn timestamp(
    time: Option<i64>,
    timezone: Option<&Tz>,
    output: &mut String,
) -> Result<i32, CodecError> {
    let time = time
        .filter(|time| *time != 0)
        .ok_or_else(|| error("RFC 3164 encoding requires an event timestamp"))?;
    let timezone = timezone.ok_or_else(|| error("RFC 3164 timezone was not prepared"))?;
    let local = DateTime::<Utc>::from_timestamp_nanos(time).with_timezone(timezone);
    if timezone
        .from_local_datetime(&local.naive_local())
        .single()
        .is_none()
    {
        return Err(error(
            "RFC 3164 cannot represent an ambiguous local timestamp",
        ));
    }
    local
        .format("%b %e %T")
        .write_to(output)
        .map_err(error_source)?;
    Ok(local.year())
}

pub(super) fn append_header(
    message: &mut Vec<u8>,
    header: &Header<'_>,
    timestamp: &str,
) -> Result<(), CodecError> {
    let priority = u16::from(header.facility) * 8 + u16::from(header.severity.unwrap_or(6));
    let (hostname, app, process) = header.fields()?;
    if hostname.contains(':')
        || app.len() > 32
        || app.contains([':', '[', ']'])
        || matches!(process, FieldValue::Text(text) if text != b"-" && !text.iter().all(u8::is_ascii_digit))
    {
        return Err(error(
            "RFC 3164 requires an unambiguous hostname and app tag with a numeric process ID",
        ));
    }
    let original_tag = header
        .tag
        .map(|value| header_text(value, "syslog.tag", MAX_MESSAGE_SIZE))
        .transpose()?;
    let tag = original_tag.filter(|tag| {
        let (original_app, original_process) =
            super::super::parser::rfc3164::parse_tag_components(Some(tag.as_bytes()));
        original_app == Some(app.as_bytes())
            && match (original_process, process) {
                (None, FieldValue::Text(b"-")) => true,
                (Some(original), FieldValue::Text(text)) => original == text,
                (Some(original), FieldValue::Integer(value)) => {
                    std::str::from_utf8(original)
                        .ok()
                        .and_then(|text| text.parse::<i64>().ok())
                        == Some(value)
                }
                _ => false,
            }
    });
    write!(message, "<{priority}>{timestamp} {hostname} ").map_err(error_source)?;
    if let Some(tag) = tag {
        if tag.contains(':') {
            return Err(error("syslog.tag must not contain a colon"));
        }
        append_bytes(message, tag.as_bytes())?;
    } else {
        append_bytes(message, app.as_bytes())?;
        if process != FieldValue::Text(b"-") {
            append_bytes(message, b"[")?;
            process.append_raw(message)?;
            append_bytes(message, b"]")?;
        }
    }
    append_bytes(message, b":")?;
    Ok(())
}
