// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! CEF header and occurrence-preserving extension serialization.

use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;

use super::super::arrow_records_encoder::KnownAttributeKey;
use super::fields::{FieldValue, append_bytes, number};
use super::{CodecError, MAX_MESSAGE_SIZE, error};

const CEF_HEADERS: [&[u8]; 6] = [
    b"cef.device_vendor",
    b"cef.device_product",
    b"cef.device_version",
    b"cef.device_event_class_id",
    b"cef.name",
    b"cef.severity",
];
const CEF_EXTENSIONS: [&[u8]; 9] = [
    b"src",
    b"dst",
    b"spt",
    b"dpt",
    b"proto",
    b"suser",
    b"deviceExternalId",
    b"act",
    b"msg",
];

#[derive(Default)]
pub(super) struct CefFields<'a> {
    parsed_cef: bool,
    version: u8,
    pub(super) headers: [Option<FieldValue<'a>>; CEF_HEADERS.len()],
    pub(super) extensions: Vec<(&'a [u8], OtapAnyValueView<'a>)>,
    pub(super) extension_size: usize,
}

pub(super) fn cef_value(value: OtapAnyValueView<'_>) -> Result<FieldValue<'_>, CodecError> {
    if let OtapAnyValueView::Str(text) = value {
        if text.len() > MAX_MESSAGE_SIZE {
            return Err(error(format!("CEF field exceeds {MAX_MESSAGE_SIZE} bytes")));
        }
        Ok(FieldValue::Text(text))
    } else if let OtapAnyValueView::Int(number) = value {
        Ok(FieldValue::Integer(number))
    } else {
        Err(error("mapped CEF fields must be strings or integers"))
    }
}

impl<'a> CefFields<'a> {
    pub(super) fn new(parsed_cef: bool) -> Self {
        Self {
            parsed_cef,
            ..Default::default()
        }
    }

    pub(super) fn apply(
        &mut self,
        key: &'a [u8],
        value: OtapAnyValueView<'a>,
    ) -> Result<(), CodecError> {
        if key == b"cef.version" {
            self.version = number(&value, "cef.version", 1)? as u8;
        } else if let Some(index) = CEF_HEADERS.iter().position(|name| *name == key) {
            let value = cef_value(value)?;
            if matches!(value, FieldValue::Text(text) if (!self.parsed_cef && text.is_empty()) || text.iter().any(|byte| matches!(byte, b'\r' | b'\n')))
            {
                return Err(error(
                    "CEF headers must be nonempty and contain no CR or LF",
                ));
            }
            if index == CEF_HEADERS.len() - 1 && !self.parsed_cef {
                let valid_severity = match value {
                    FieldValue::Integer(value) => (0..=10).contains(&value),
                    FieldValue::Text(text) => {
                        matches!(
                            text,
                            b"Unknown" | b"Low" | b"Medium" | b"High" | b"Very-High"
                        ) || std::str::from_utf8(text)
                            .ok()
                            .and_then(|text| text.parse::<u8>().ok())
                            .is_some_and(|severity| severity <= 10)
                    }
                };
                if !valid_severity {
                    return Err(error("cef.severity must be 0..=10 or a CEF severity name"));
                }
            }
            self.headers[index] = Some(value);
        } else if CEF_EXTENSIONS.contains(&key)
            || (!KnownAttributeKey::is_known_name(key)
                && (self.parsed_cef || !key.contains(&b'.') || key.starts_with(b"cef.")))
        {
            if key.is_empty()
                || !key
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            {
                return Err(error(
                    "CEF extension key must contain only ASCII letters, digits, underscores or hyphens",
                ));
            }
            // A nonempty body overrides msg, including its type validation.
            let value_size = if key == b"msg" {
                0
            } else {
                cef_value(value)?.len()
            };
            let size = self.extension_size + key.len() + value_size + 2;
            if size > MAX_MESSAGE_SIZE {
                return Err(error(format!(
                    "CEF extensions exceed {MAX_MESSAGE_SIZE} bytes"
                )));
            }
            self.extension_size = size;
            self.extensions.push((key, value));
        }
        Ok(())
    }

    pub(super) fn append(
        mut self,
        message: &mut Vec<u8>,
        body: Option<&'a [u8]>,
    ) -> Result<(), CodecError> {
        // Empty generated bodies must not hide parsed message attributes.
        if let Some(body) = body.filter(|body| !body.is_empty()) {
            self.extensions.retain(|(key, _)| *key != b"msg");
            self.extensions.push((b"msg", OtapAnyValueView::Str(body)));
        } else if let Some(body) = body
            && !self.extensions.iter().any(|(key, _)| *key == b"msg")
        {
            self.extensions.push((b"msg", OtapAnyValueView::Str(body)));
        }
        let mut trailing_spaces = None;
        for (index, (_, value)) in self.extensions.iter().enumerate() {
            let value = cef_value(*value)?;
            // Only ASCII spaces separate receiver extensions; other whitespace
            // (including escaped CR/LF) can remain before another key.
            if matches!(value, FieldValue::Text(text) if text.ends_with(b" "))
                && trailing_spaces.replace(index).is_some()
            {
                return Err(error(
                    "CEF cannot preserve multiple extension values ending in spaces",
                ));
            }
        }
        if let Some(index) = trailing_spaces {
            let key = self.extensions[index].0;
            if self.extensions[index + 1..]
                .iter()
                .any(|(later_key, _)| *later_key == key)
            {
                return Err(error(
                    "CEF cannot preserve duplicate order for an extension value ending in spaces",
                ));
            }
        }
        append_bytes(
            message,
            if self.version == 0 {
                b"CEF:0|"
            } else {
                b"CEF:1|"
            },
        )?;
        for header in self.headers {
            let header =
                header.ok_or_else(|| error("CEF encoding requires all six CEF headers"))?;
            header.append(message, true)?;
            append_bytes(message, b"|")?;
        }
        let mut first = true;
        let ordered = self
            .extensions
            .iter()
            .enumerate()
            .filter(|(index, _)| Some(*index) != trailing_spaces)
            .map(|(_, extension)| extension)
            .chain(trailing_spaces.map(|index| &self.extensions[index]));
        for (key, value) in ordered {
            if !first {
                append_bytes(message, b" ")?;
            }
            first = false;
            append_bytes(message, key)?;
            append_bytes(message, b"=")?;
            cef_value(*value)?.append(message, false)?;
        }
        Ok(())
    }
}

impl FieldValue<'_> {
    pub(super) fn append(self, message: &mut Vec<u8>, header: bool) -> Result<(), CodecError> {
        match self {
            Self::Text(text) => append_escaped(message, text, header),
            Self::Integer(_) => self.append_raw(message),
        }
    }
}

pub(super) fn append_escaped(
    message: &mut Vec<u8>,
    value: &[u8],
    header: bool,
) -> Result<(), CodecError> {
    super::fields::check_message_capacity(message, value.len())?;
    let mut start = 0;
    for (index, byte) in value.iter().enumerate() {
        let escaped: &[u8] = match byte {
            b'\\' => b"\\\\",
            b'|' if header => b"\\|",
            b'=' if !header => b"\\=",
            b'\n' if !header => b"\\n",
            b'\r' if !header => b"\\r",
            _ => continue,
        };
        if start != index {
            append_bytes(message, &value[start..index])?;
        }
        append_bytes(message, escaped)?;
        start = index + 1;
    }
    append_bytes(message, &value[start..])
}
