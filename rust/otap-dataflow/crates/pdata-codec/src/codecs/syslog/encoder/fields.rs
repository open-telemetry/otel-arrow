// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared borrowed header fields, scalar values, and bounded message writes.

use std::io::Write;

use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;
use otel_arrow_dfe_pdata_views::views::common::AnyValueView;

use super::{CodecError, MAX_MESSAGE_SIZE, error, error_source};

fn decimal_len(value: i64) -> usize {
    let magnitude = value.unsigned_abs();
    let digits = if magnitude == 0 {
        1
    } else {
        magnitude.ilog10() as usize + 1
    };
    digits + usize::from(value < 0)
}

fn decimal_bytes(value: i64, storage: &mut [u8; 20]) -> std::io::Result<&[u8]> {
    let length = {
        let mut decimal = std::io::Cursor::new(storage.as_mut_slice());
        write!(decimal, "{value}")?;
        decimal.position() as usize
    };
    Ok(&storage[..length])
}

#[derive(Clone, Copy)]
struct HeaderField<'a> {
    value: OtapAnyValueView<'a>,
    name: &'static str,
}

impl<'a> HeaderField<'a> {
    fn new(value: OtapAnyValueView<'a>, name: &'static str) -> Self {
        Self { value, name }
    }

    fn text(self, limit: usize) -> Result<&'a str, CodecError> {
        header_text(self.value, self.name, limit)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Header<'a> {
    hostname: HeaderField<'a>,
    app: HeaderField<'a>,
    process: Option<HeaderField<'a>>,
    process_str: Option<OtapAnyValueView<'a>>,
    pub(super) message_id: &'a str,
    pub(super) structured_data: Option<OtapAnyValueView<'a>>,
    pub(super) version: u8,
    pub(super) facility: u8,
    pub(super) severity: Option<u8>,
    pub(super) message: Option<OtapAnyValueView<'a>>,
    pub(super) content: Option<OtapAnyValueView<'a>>,
    pub(super) tag: Option<OtapAnyValueView<'a>>,
}

impl Default for Header<'_> {
    fn default() -> Self {
        Self {
            hostname: HeaderField::new(OtapAnyValueView::Str(b"-"), "syslog.host_name"),
            app: HeaderField::new(OtapAnyValueView::Str(b"-"), "syslog.app_name"),
            process: None,
            process_str: None,
            message_id: "-",
            structured_data: None,
            version: 1,
            facility: 1,
            severity: None,
            message: None,
            content: None,
            tag: None,
        }
    }
}

pub(super) fn header_text<'a>(
    value: OtapAnyValueView<'a>,
    name: &str,
    limit: usize,
) -> Result<&'a str, CodecError> {
    let OtapAnyValueView::Str(text) = value else {
        return Err(error(format!("{name} must be a string")));
    };
    if text.is_empty() || text.len() > limit || !text.iter().all(|byte| (33..=126).contains(byte)) {
        return Err(error(format!(
            "{name} must contain 1..={limit} printable non-space ASCII bytes"
        )));
    }
    // The byte validation above guarantees UTF-8.
    std::str::from_utf8(text).map_err(|_| error(format!("{name} must be ASCII")))
}

pub(super) fn number<'a>(
    value: &impl AnyValueView<'a>,
    name: &str,
    max: i64,
) -> Result<i64, CodecError> {
    value
        .as_int64()
        .filter(|value| (0..=max).contains(value))
        .ok_or_else(|| error(format!("{name} must be an integer in 0..={max}")))
}

impl<'a> Header<'a> {
    pub(super) fn apply_resource(&mut self, key: &[u8], value: OtapAnyValueView<'a>) {
        match key {
            b"host.name" => self.hostname = HeaderField::new(value, "host.name"),
            b"service.name" => self.app = HeaderField::new(value, "service.name"),
            b"process.pid" => {
                self.process = Some(HeaderField::new(value, "process.pid"));
            }
            _ => {}
        }
    }

    pub(super) fn apply_log(
        &mut self,
        key: &[u8],
        value: OtapAnyValueView<'a>,
    ) -> Result<(), CodecError> {
        match key {
            b"syslog.host_name" => {
                self.hostname = HeaderField::new(value, "syslog.host_name");
            }
            b"syslog.app_name" => self.app = HeaderField::new(value, "syslog.app_name"),
            b"syslog.process_id" => {
                self.process = Some(HeaderField::new(value, "syslog.process_id"));
            }
            b"syslog.process_id_str" => {
                self.process_str = Some(value);
            }
            b"syslog.msg_id" => self.message_id = header_text(value, "syslog.msg_id", 32)?,
            b"syslog.version" => self.version = number(&value, "syslog.version", 255)? as u8,
            b"syslog.facility" => {
                self.facility = number(&value, "syslog.facility", 23)? as u8;
            }
            b"syslog.severity" => {
                self.severity = Some(number(&value, "syslog.severity", 7)? as u8);
            }
            b"syslog.message" => self.message = Some(value),
            b"syslog.content" => self.content = Some(value),
            b"syslog.tag" => self.tag = Some(value),
            b"syslog.structured_data" => self.structured_data = Some(value),
            _ => {}
        }
        Ok(())
    }

    pub(super) fn fields(&self) -> Result<(&'a str, &'a str, FieldValue<'a>), CodecError> {
        let process = if let Some(value) = self.process_str {
            FieldValue::Text(header_text(value, "syslog.process_id_str", 128)?.as_bytes())
        } else if let Some(field) = self.process {
            FieldValue::Integer(number(&field.value, field.name, i64::MAX)?)
        } else {
            FieldValue::Text(b"-")
        };
        Ok((self.hostname.text(255)?, self.app.text(48)?, process))
    }
}

pub(super) fn severity(number: Option<i32>) -> Result<u8, CodecError> {
    match number.unwrap_or(0) {
        0 | 9..=12 => Ok(6),
        1..=8 => Ok(7),
        13..=16 => Ok(4),
        17..=20 => Ok(3),
        21..=24 => Ok(2),
        _ => Err(error("severity_number must be in 0..=24")),
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum FieldValue<'a> {
    Text(&'a [u8]),
    Integer(i64),
}

impl FieldValue<'_> {
    pub(super) fn len(self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Integer(value) => decimal_len(value),
        }
    }

    pub(super) fn append_raw(self, message: &mut Vec<u8>) -> Result<(), CodecError> {
        match self {
            Self::Text(text) => append_bytes(message, text),
            Self::Integer(value) => {
                let mut decimal = [0_u8; 20];
                append_bytes(
                    message,
                    decimal_bytes(value, &mut decimal).map_err(error_source)?,
                )
            }
        }
    }
}

pub(super) fn check_message_capacity(message: &[u8], additional: usize) -> Result<(), CodecError> {
    if additional > MAX_MESSAGE_SIZE.saturating_sub(message.len()) {
        return Err(error(format!(
            "encoded syslog/CEF message exceeds {MAX_MESSAGE_SIZE} bytes"
        )));
    }
    Ok(())
}

pub(super) fn append_bytes(message: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CodecError> {
    check_message_capacity(message, bytes.len())?;
    message.extend_from_slice(bytes);
    Ok(())
}
