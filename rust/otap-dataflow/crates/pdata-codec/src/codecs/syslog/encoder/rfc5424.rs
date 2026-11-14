// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! RFC5424 timestamp and header serialization.

use std::io::Write;

use chrono::{DateTime, Utc};
use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;

use super::fields::{Header, append_bytes};
use super::{CodecError, MAX_MESSAGE_SIZE, error, error_source};

const UTF8_BOM: &[u8] = b"\xef\xbb\xbf";

pub(super) fn timestamp(time: Option<i64>, output: &mut String) -> Result<i32, CodecError> {
    if let Some(value) = time.filter(|time| *time != 0) {
        let time = DateTime::<Utc>::from_timestamp_nanos(value);
        time.format("%Y-%m-%dT%H:%M:%S%.6fZ")
            .write_to(output)
            .map_err(error_source)?;
    } else {
        output.push('-');
    }
    Ok(1970)
}

pub(super) fn structured_data(
    value: Option<OtapAnyValueView<'_>>,
    supported: bool,
) -> Result<&[u8], CodecError> {
    match value {
        None | Some(OtapAnyValueView::Str(b"-")) => Ok(b"-".as_slice()),
        Some(OtapAnyValueView::Str(data)) if data.len() <= MAX_MESSAGE_SIZE => {
            if !supported {
                return Err(error(
                    "selected profile cannot represent RFC 5424 structured data",
                ));
            }
            let (parsed, remainder) = super::super::parser::rfc5424::parse_structured_data(data);
            if parsed != Some(data) || remainder.is_some() {
                return Err(error(
                    "syslog.structured_data must contain only structured data",
                ));
            }
            Ok(data)
        }
        Some(_) => Err(error(format!(
            "syslog.structured_data must be a string of at most {MAX_MESSAGE_SIZE} bytes"
        ))),
    }
}

pub(super) fn append_header(
    message: &mut Vec<u8>,
    header: &Header<'_>,
    timestamp: &str,
    structured_data: &[u8],
) -> Result<(), CodecError> {
    let priority = u16::from(header.facility) * 8 + u16::from(header.severity.unwrap_or(6));
    let (hostname, app, process) = header.fields()?;
    write!(
        message,
        "<{priority}>{} {timestamp} {hostname} {app} ",
        header.version
    )
    .map_err(error_source)?;
    process.append_raw(message)?;
    write!(message, " {} ", header.message_id).map_err(error_source)?;
    append_bytes(message, structured_data)?;
    Ok(())
}

pub(super) fn append_message(
    message: &mut Vec<u8>,
    structured_data: &[u8],
    body: &[u8],
) -> Result<(), CodecError> {
    append_bytes(message, b" ")?;
    // The receiver skips spaces when looking for another SD element and strips
    // one leading BOM from the message. Protect both boundaries without changing
    // a literal BOM already present in the semantic message.
    if body.is_empty()
        || body.starts_with(UTF8_BOM)
        || (structured_data != b"-" && body.iter().find(|byte| **byte != b' ') == Some(&b'['))
    {
        append_bytes(message, UTF8_BOM)?;
    }
    append_bytes(message, body)
}

pub(super) fn validate_message(
    data_and_message: &[u8],
    structured_data: &[u8],
    body: Option<&[u8]>,
) -> Result<(), CodecError> {
    let (parsed_data, parsed_message) =
        super::super::parser::rfc5424::parse_structured_data(data_and_message);
    let parsed_message =
        parsed_message.map(|message| message.strip_prefix(UTF8_BOM).unwrap_or(message));
    if parsed_data != (structured_data != b"-").then_some(structured_data) || parsed_message != body
    {
        return Err(error(
            "RFC 5424 structured data and message cannot be encoded without changing their meaning",
        ));
    }
    Ok(())
}
