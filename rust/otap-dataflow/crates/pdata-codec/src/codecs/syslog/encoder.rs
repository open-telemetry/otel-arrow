// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog and CEF output from native logs in the existing syslog batch envelope.

use bytes::Bytes;
use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::codecs::CodecConfiguration;
use otel_arrow_dfe_pdata::otlp::{BoundedBuf, ProtoBuffer};
use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_pdata_views::views::{
    common::{AnyValueView, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
};

use crate::encoder::{decimal_bytes, decimal_len};
use crate::native::{AttributeIndex, NativeLogs};
use crate::{
    CodecError, CodecOperation, EncodeOutput, EncodePolicy, PdataEncoder, PreparedCodecFactory,
    ResolvedCodec,
};
use std::io::Write;
use std::sync::Arc;

use super::codec::{BATCH_HEADER_LEN, FRAME_HEADER_LEN, MAGIC, RUN_HEADER_LEN};
use super::{MAX_BATCH_BYTES, MAX_BATCH_ITEMS, MAX_MESSAGE_SIZE};

const MAX_RETAINED_BUFFER_CAPACITY: usize = 256 * 1024;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Format {
    Rfc3164,
    #[default]
    Rfc5424,
    Cef,
    CefRfc3164,
    CefRfc5424,
}

impl Format {
    fn parse(name: &str) -> Result<Self, CodecError> {
        match name {
            "rfc3164" => Ok(Self::Rfc3164),
            "rfc5424" => Ok(Self::Rfc5424),
            "cef" => Ok(Self::Cef),
            "cef_rfc3164" => Ok(Self::CefRfc3164),
            "cef_rfc5424" => Ok(Self::CefRfc5424),
            _ => Err(CodecError::configuration("unsupported syslog/CEF profile")),
        }
    }

    fn is_cef(self) -> bool {
        matches!(self, Self::Cef | Self::CefRfc3164 | Self::CefRfc5424)
    }
}

const ALL_FORMATS: [Format; 5] = [
    Format::Rfc3164,
    Format::Rfc5424,
    Format::Cef,
    Format::CefRfc3164,
    Format::CefRfc5424,
];

#[derive(Clone, Copy)]
enum Profile {
    Fixed(Format),
    All,
}

pub(super) fn prepare(
    config: &CodecConfiguration,
) -> Result<Arc<dyn PreparedCodecFactory>, CodecError> {
    if config.options.keys().any(|key| key != "profile") {
        return Err(CodecError::configuration("unknown syslog/CEF codec option"));
    }
    let profile = config.options.get("profile");
    let output_profile = match profile {
        Some(profile) => match profile
            .as_str()
            .ok_or_else(|| CodecError::configuration("syslog/CEF profile must be a string"))?
        {
            "all" => Profile::All,
            name => Profile::Fixed(Format::parse(name)?),
        },
        None => Profile::Fixed(Format::default()),
    };
    Ok(Arc::new(SyslogFactory {
        profile: output_profile,
        explicit_profile: profile.is_some(),
    }))
}

struct SyslogFactory {
    profile: Profile,
    explicit_profile: bool,
}

impl PreparedCodecFactory for SyslogFactory {
    fn create_encoder(
        &self,
        _: ResolvedCodec,
        policy: EncodePolicy,
    ) -> Result<Box<dyn PdataEncoder>, CodecError> {
        Ok(Box::new(SyslogEncoder::for_profile(policy, self.profile)))
    }

    fn allows_encoded_forwarding(&self) -> bool {
        !self.explicit_profile
    }
}

fn error(message: impl Into<String>) -> CodecError {
    CodecError::operation(
        &super::codec::SYSLOG_CEF_ENCODING,
        CodecOperation::Encode,
        super::codec::framing_error(message),
    )
}

pub(super) struct SyslogEncoder {
    max_bytes: usize,
    profile: Profile,
    next_format: usize,
    buffer: ProtoBuffer,
    message: Vec<u8>,
    timestamp: String,
    log_rows: Vec<(u16, u16, usize)>,
    log_index: AttributeIndex,
    resource_index: AttributeIndex,
}

impl SyslogEncoder {
    pub(super) fn new(policy: EncodePolicy) -> Self {
        Self::for_profile(policy, Profile::Fixed(Format::default()))
    }

    fn for_profile(policy: EncodePolicy, profile: Profile) -> Self {
        let max_bytes = policy
            .max_encoded_size
            .map_or(MAX_BATCH_BYTES, |limit| limit.get().min(MAX_BATCH_BYTES));
        Self {
            profile,
            next_format: 0,
            max_bytes,
            buffer: ProtoBuffer::with_capacity_and_limit(0, max_bytes),
            message: Vec::new(),
            timestamp: String::new(),
            log_rows: Vec::new(),
            log_index: AttributeIndex::default(),
            resource_index: AttributeIndex::default(),
        }
    }

    #[cfg(test)]
    fn for_format(policy: EncodePolicy, format: Format) -> Self {
        Self::for_profile(policy, Profile::Fixed(format))
    }
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
struct Header<'a> {
    hostname: HeaderField<'a>,
    app: HeaderField<'a>,
    process: Option<HeaderField<'a>>,
    process_str: Option<OtapAnyValueView<'a>>,
    message_id: &'a str,
    facility: u8,
    severity: Option<u8>,
    message: Option<OtapAnyValueView<'a>>,
    content: Option<OtapAnyValueView<'a>>,
}

impl Default for Header<'_> {
    fn default() -> Self {
        Self {
            hostname: HeaderField::new(OtapAnyValueView::Str(b"-"), "syslog.host_name"),
            app: HeaderField::new(OtapAnyValueView::Str(b"-"), "syslog.app_name"),
            process: None,
            process_str: None,
            message_id: "-",
            facility: 1,
            severity: None,
            message: None,
            content: None,
        }
    }
}

fn header_text<'a>(
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

fn number<'a>(value: &impl AnyValueView<'a>, name: &str, max: i64) -> Result<i64, CodecError> {
    value
        .as_int64()
        .filter(|value| (0..=max).contains(value))
        .ok_or_else(|| error(format!("{name} must be an integer in 0..={max}")))
}

impl<'a> Header<'a> {
    fn apply_resource(&mut self, key: &[u8], value: OtapAnyValueView<'a>) {
        match key {
            b"host.name" => self.hostname = HeaderField::new(value, "host.name"),
            b"service.name" => self.app = HeaderField::new(value, "service.name"),
            b"process.pid" => {
                self.process = Some(HeaderField::new(value, "process.pid"));
            }
            _ => {}
        }
    }

    fn apply_log(&mut self, key: &[u8], value: OtapAnyValueView<'a>) -> Result<(), CodecError> {
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
            b"syslog.facility" => {
                self.facility = number(&value, "syslog.facility", 23)? as u8;
            }
            b"syslog.severity" => {
                self.severity = Some(number(&value, "syslog.severity", 7)? as u8);
            }
            b"syslog.message" => self.message = Some(value),
            b"syslog.content" => self.content = Some(value),
            b"syslog.structured_data" if value.as_string() != Some(b"-") => {
                return Err(error(
                    "RFC 5424 structured data encoding is not supported by this prototype",
                ));
            }
            _ => {}
        }
        Ok(())
    }

    fn fields(&self) -> Result<(&'a str, &'a str, FieldValue<'a>), CodecError> {
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

fn severity(number: Option<i32>) -> Result<u8, CodecError> {
    match number.unwrap_or(0) {
        0 | 9..=12 => Ok(6),
        1..=8 => Ok(7),
        13..=16 => Ok(4),
        17..=20 => Ok(3),
        21..=24 => Ok(2),
        _ => Err(error("severity_number must be in 0..=24")),
    }
}

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

#[derive(Clone, Copy, PartialEq)]
enum FieldValue<'a> {
    Text(&'a [u8]),
    Integer(i64),
}

impl FieldValue<'_> {
    fn len(self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Integer(value) => decimal_len(value),
        }
    }

    fn append(self, message: &mut Vec<u8>, header: bool) -> Result<(), CodecError> {
        match self {
            Self::Text(text) => append_escaped(message, text, header),
            Self::Integer(_) => self.append_raw(message),
        }
    }

    fn append_raw(self, message: &mut Vec<u8>) -> Result<(), CodecError> {
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

#[derive(Default)]
struct CefFields<'a> {
    version: u8,
    headers: [Option<FieldValue<'a>>; CEF_HEADERS.len()],
    extensions: [Option<FieldValue<'a>>; CEF_EXTENSIONS.len()],
    message: Option<OtapAnyValueView<'a>>,
    custom_extensions: std::collections::HashMap<&'a [u8], FieldValue<'a>>,
    custom_size: usize,
}

fn cef_value(value: OtapAnyValueView<'_>) -> Result<FieldValue<'_>, CodecError> {
    if let OtapAnyValueView::Str(text) = value {
        if text.len() > MAX_MESSAGE_SIZE {
            return Err(error("CEF field exceeds 16 KiB"));
        }
        Ok(FieldValue::Text(text))
    } else if let OtapAnyValueView::Int(number) = value {
        Ok(FieldValue::Integer(number))
    } else {
        Err(error("mapped CEF fields must be strings or integers"))
    }
}

impl<'a> CefFields<'a> {
    fn apply(&mut self, key: &'a [u8], value: OtapAnyValueView<'a>) -> Result<(), CodecError> {
        if key == b"cef.version" {
            self.version = number(&value, "cef.version", 1)? as u8;
        } else if let Some(index) = CEF_HEADERS.iter().position(|name| *name == key) {
            let value = cef_value(value)?;
            if matches!(value, FieldValue::Text(text) if text.is_empty() || text.iter().any(|byte| matches!(byte, b'\r' | b'\n')))
            {
                return Err(error(
                    "CEF headers must be nonempty and contain no CR or LF",
                ));
            }
            if index == CEF_HEADERS.len() - 1 {
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
        } else if key == b"msg" {
            self.message = Some(value);
        } else if let Some(index) = CEF_EXTENSIONS.iter().position(|name| *name == key) {
            self.extensions[index] = Some(cef_value(value)?);
        } else if let Some(key) = key.strip_prefix(b"cef.extension.") {
            if !key.first().is_some_and(u8::is_ascii_alphabetic)
                || !key
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                || CEF_EXTENSIONS.contains(&key)
            {
                return Err(error(
                    "custom CEF extension key must be an identifier and must not shadow a standard field",
                ));
            }
            let value = cef_value(value)?;
            let previous_size = self
                .custom_extensions
                .get(key)
                .map_or(0, |value| key.len() + value.len());
            let size = self.custom_size - previous_size + key.len() + value.len();
            if size > MAX_MESSAGE_SIZE {
                return Err(error("custom CEF extensions exceed 16 KiB"));
            }
            self.custom_size = size;
            _ = self.custom_extensions.insert(key, value);
        }
        Ok(())
    }

    fn append(self, message: &mut Vec<u8>, body: Option<&[u8]>) -> Result<(), CodecError> {
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
        for (key, value) in CEF_EXTENSIONS.into_iter().zip(self.extensions) {
            // Empty generated bodies must not hide parsed message attributes.
            let value = if key == b"msg" {
                if let Some(body) = body.filter(|body| !body.is_empty()) {
                    Some(FieldValue::Text(body))
                } else {
                    self.message
                        .map(cef_value)
                        .transpose()?
                        .or(body.map(FieldValue::Text))
                }
            } else {
                value
            };
            if let Some(value) = value {
                if !first {
                    append_bytes(message, b" ")?;
                }
                first = false;
                append_bytes(message, key)?;
                append_bytes(message, b"=")?;
                value.append(message, false)?;
            }
        }
        for (key, value) in self.custom_extensions {
            if !first {
                append_bytes(message, b" ")?;
            }
            first = false;
            append_bytes(message, key)?;
            append_bytes(message, b"=")?;
            value.append(message, false)?;
        }
        Ok(())
    }
}

fn append_bytes(message: &mut Vec<u8>, bytes: &[u8]) -> Result<(), CodecError> {
    if bytes.len() > MAX_MESSAGE_SIZE.saturating_sub(message.len()) {
        return Err(error("encoded syslog/CEF message exceeds 16 KiB"));
    }
    message.extend_from_slice(bytes);
    Ok(())
}

fn append_escaped(message: &mut Vec<u8>, value: &[u8], header: bool) -> Result<(), CodecError> {
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

fn wire_time(format: Format, time: Option<u64>, output: &mut String) -> Result<i32, CodecError> {
    output.clear();
    match format {
        Format::Cef => Ok(1970),
        Format::Rfc5424 | Format::CefRfc5424 => {
            if let Some(value) = time.filter(|time| *time != 0) {
                let time = DateTime::<Utc>::from_timestamp(
                    (value / 1_000_000_000) as i64,
                    (value % 1_000_000_000) as u32,
                )
                .ok_or_else(|| error("log timestamp cannot be represented as RFC 5424"))?;
                time.format("%Y-%m-%dT%H:%M:%S%.6fZ")
                    .write_to(output)
                    .map_err(error_source)?;
            } else {
                output.push('-');
            }
            Ok(1970)
        }
        Format::Rfc3164 | Format::CefRfc3164 => {
            let time = time
                .filter(|time| *time != 0)
                .ok_or_else(|| error("RFC 3164 encoding requires an event timestamp"))?;
            let time = DateTime::from_timestamp(
                (time / 1_000_000_000) as i64,
                (time % 1_000_000_000) as u32,
            )
            .ok_or_else(|| error("log timestamp cannot be represented as RFC 3164"))?
            .with_timezone(&Local);
            if Local
                .from_local_datetime(&time.naive_local())
                .single()
                .is_none()
            {
                return Err(error(
                    "RFC 3164 cannot represent an ambiguous local timestamp",
                ));
            }
            time.format("%b %e %T")
                .write_to(output)
                .map_err(error_source)?;
            Ok(time.year())
        }
    }
}

fn serialize_message(
    message: &mut Vec<u8>,
    format: Format,
    header: &Header<'_>,
    timestamp: &str,
    body: Option<&[u8]>,
    cef: Option<CefFields<'_>>,
) -> Result<(), CodecError> {
    message.clear();
    let priority = u16::from(header.facility) * 8 + u16::from(header.severity.unwrap_or(6));
    match format {
        Format::Cef => {}
        Format::Rfc5424 | Format::CefRfc5424 => {
            let (hostname, app, process) = header.fields()?;
            write!(message, "<{priority}>1 {timestamp} {hostname} {app} ",)
                .map_err(error_source)?;
            process.append_raw(message)?;
            write!(message, " {} -", header.message_id).map_err(error_source)?;
        }
        Format::Rfc3164 | Format::CefRfc3164 => {
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
            write!(message, "<{priority}>{timestamp} {hostname} {app}").map_err(error_source)?;
            if process != FieldValue::Text(b"-") {
                append_bytes(message, b"[")?;
                process.append_raw(message)?;
                append_bytes(message, b"]")?;
            }
            append_bytes(message, b":")?;
        }
    }
    if format.is_cef() {
        if !message.is_empty() {
            append_bytes(message, b" ")?;
        }
        cef.ok_or_else(|| error("CEF encoding requires CEF fields"))?
            .append(message, body)?;
    } else if let Some(body) = body {
        append_bytes(message, b" ")?;
        append_bytes(message, body)?;
    }
    Ok(())
}

fn append_batch(buffer: &mut ProtoBuffer, bytes: &[u8]) -> Result<(), CodecError> {
    buffer
        .try_extend_slice(bytes)
        .map_err(|_| CodecError::EncodedSizeLimitExceeded {
            encoding: super::codec::SYSLOG_CEF_ENCODING,
            actual: buffer.len() + bytes.len(),
            limit: buffer.limit(),
        })
}

impl SyslogEncoder {
    fn encode_into(&mut self, records: &mut OtapArrowRecords) -> Result<(), CodecError> {
        self.buffer.clear();
        if records.signal_type() != SignalType::Logs {
            return Err(error("syslog encoding supports only logs"));
        }
        let count = records.num_items();
        if count == 0 || count > MAX_BATCH_ITEMS {
            return Err(error("syslog batches must contain 1..=8192 logs"));
        }
        let input = NativeLogs::new(records).map_err(error_source)?;
        let log_attrs = input.log_attributes().map_err(error_source)?;
        let resource_attrs = input.resource_attributes().map_err(error_source)?;
        let _scope_attrs = input.scope_attributes().map_err(error_source)?;
        self.log_index.rebuild(log_attrs.as_ref());
        self.resource_index.rebuild(resource_attrs.as_ref());
        let view = input.view().map_err(error_source)?;
        input.order_rows(&mut self.log_rows);
        let mut log_rows = self.log_rows.iter();
        append_batch(&mut self.buffer, MAGIC)?;
        append_batch(&mut self.buffer, &(count as u32).to_be_bytes())?;
        let mut encoded_size = BATCH_HEADER_LEN;
        let mut previous_context = None;
        let mut item_count = 0;
        let mut run_count = 0_u32;
        let mut run_count_offset = 0;
        // Commit the rotation only after a whole batch has encoded successfully.
        let mut next_format = self.next_format;
        for resource_logs in view.resources() {
            let mut resource_header = Header::default();
            let mut resource_initialized = false;
            for scope in resource_logs.scopes() {
                for log in scope.log_records() {
                    let &(resource_id, _, row) = log_rows
                        .next()
                        .ok_or_else(|| error("syslog log row does not match hierarchy"))?;
                    let format = match self.profile {
                        Profile::Fixed(format) => format,
                        Profile::All => ALL_FORMATS[next_format],
                    };
                    let mut header = Header::default();
                    if format != Format::Cef {
                        if !resource_initialized {
                            if let Some(resource) = &resource_attrs {
                                for (key, value) in
                                    resource.attributes(self.resource_index.rows_for(resource_id))
                                {
                                    resource_header.apply_resource(key, value);
                                }
                            }
                            resource_initialized = true;
                        }
                        header = resource_header;
                    }
                    let mut cef = CefFields::default();
                    if let Some(attrs) = &log_attrs
                        && let Some(id) = input.log_id(row)
                    {
                        for (key, value) in attrs.attributes(self.log_index.rows_for(id)) {
                            if format != Format::Cef {
                                header.apply_log(key, value)?;
                            }
                            if format.is_cef() {
                                cef.apply(key, value)?;
                            }
                        }
                    }
                    header.severity = Some(match header.severity {
                        Some(level) => level,
                        None => severity(log.severity_number())?,
                    });
                    let year = wire_time(format, log.time_unix_nano(), &mut self.timestamp)?;
                    let body = log.body();
                    let body = match body.as_ref() {
                        Some(body) if body.value_type() == ValueType::Empty => None,
                        Some(body) => Some(body.as_string().ok_or_else(|| {
                            error("syslog/CEF encoding currently requires a string log body")
                        })?),
                        None => None,
                    };
                    let body = if format.is_cef() {
                        body
                    } else if let Some(body) = body.filter(|body| !body.is_empty()) {
                        Some(body)
                    } else {
                        let message = if format == Format::Rfc3164 {
                            header.content.or(header.message)
                        } else {
                            header.message.or(header.content)
                        };
                        message
                            .map(|value| match value {
                                OtapAnyValueView::Str(message)
                                    if message.len() <= MAX_MESSAGE_SIZE =>
                                {
                                    Ok(message)
                                }
                                _ => Err(error(
                                    "syslog message text must be a string of at most 16 KiB",
                                )),
                            })
                            .transpose()?
                            .or(body)
                    };
                    serialize_message(
                        &mut self.message,
                        format,
                        &header,
                        &self.timestamp,
                        body,
                        format.is_cef().then_some(cef),
                    )?;
                    let observed_time = i64::try_from(log.observed_time_unix_nano().unwrap_or(0))
                        .map_err(|_| {
                        error("observed timestamp exceeds syslog batch timestamp range")
                    })?;
                    let context = (observed_time, year);
                    let new_run = previous_context != Some(context);
                    encoded_size += FRAME_HEADER_LEN + self.message.len();
                    if new_run {
                        encoded_size += RUN_HEADER_LEN;
                    }
                    if encoded_size > self.max_bytes {
                        return Err(CodecError::EncodedSizeLimitExceeded {
                            encoding: super::codec::SYSLOG_CEF_ENCODING,
                            actual: encoded_size,
                            limit: self.max_bytes,
                        });
                    }
                    if new_run {
                        if previous_context.is_some() {
                            self.buffer.as_mut_slice()[run_count_offset..run_count_offset + 4]
                                .copy_from_slice(&run_count.to_be_bytes());
                        }
                        append_batch(&mut self.buffer, &observed_time.to_be_bytes())?;
                        run_count_offset = self.buffer.len();
                        append_batch(&mut self.buffer, &0_u32.to_be_bytes())?;
                        append_batch(&mut self.buffer, &year.to_be_bytes())?;
                        run_count = 0;
                    }
                    run_count += 1;
                    item_count += 1;
                    append_batch(&mut self.buffer, &(self.message.len() as u32).to_be_bytes())?;
                    append_batch(&mut self.buffer, &self.message)?;
                    previous_context = Some(context);
                    if matches!(self.profile, Profile::All) {
                        next_format = (next_format + 1) % ALL_FORMATS.len();
                    }
                }
            }
        }
        if item_count != count {
            return Err(error("syslog log count does not match batch item count"));
        }
        self.buffer.as_mut_slice()[run_count_offset..run_count_offset + 4]
            .copy_from_slice(&run_count.to_be_bytes());
        self.next_format = next_format;
        Ok(())
    }
}

impl PdataEncoder for SyslogEncoder {
    fn encode(&mut self, mut records: OtapArrowRecords) -> Result<Bytes, CodecError> {
        Ok(self.prepare_encode(&mut records)?.into_bytes())
    }

    fn prepare_encode<'a>(
        &'a mut self,
        records: &mut OtapArrowRecords,
    ) -> Result<EncodeOutput<'a>, CodecError> {
        let result = self.encode_into(records);
        self.log_index.reset(MAX_RETAINED_BUFFER_CAPACITY);
        self.resource_index.reset(MAX_RETAINED_BUFFER_CAPACITY);
        self.log_rows.clear();
        if let Err(error) = result {
            self.buffer.clear();
            self.buffer.retain_capacity(MAX_RETAINED_BUFFER_CAPACITY);
            self.message.clear();
            return Err(error);
        }
        Ok(EncodeOutput::buffer(
            &mut self.buffer,
            MAX_RETAINED_BUFFER_CAPACITY,
        ))
    }
}

fn error_source(source: impl std::error::Error + Send + Sync + 'static) -> CodecError {
    CodecError::operation(
        &super::codec::SYSLOG_CEF_ENCODING,
        CodecOperation::Encode,
        source,
    )
}

#[cfg(test)]
mod tests {
    use super::super::SyslogBatchBuilder;
    use super::*;
    use crate::EncodedPdataBatch;
    use crate::native::AttributeColumns;
    use crate::{CodecService, CodecServiceBuilder, EncodingPlan, OtapPayload, PdataEncoding};
    use arrow::array::{RecordBatch, StructArray, UInt16Array, UInt32Array};
    use otel_arrow_dfe_config::codecs::CodecsConfig;
    use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
    use otel_arrow_dfe_pdata::encode::encode_logs_otap_batch;
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_dfe_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
        resource::v1::Resource,
    };
    use otel_arrow_dfe_pdata::schema::consts;
    use otel_arrow_dfe_pdata::views::otap::OtapLogsView;
    use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};
    use otel_arrow_dfe_pdata_views::views::common::AttributeView;

    fn records(logs: Vec<LogRecord>, attributes: Vec<KeyValue>) -> OtapArrowRecords {
        encode_logs_otap_batch(&LogsData {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes,
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: logs,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        })
        .unwrap()
    }

    fn log(body: &str) -> LogRecord {
        LogRecord {
            body: Some(AnyValue::new_string(body)),
            ..Default::default()
        }
    }

    fn text(key: &str, value: &str) -> KeyValue {
        KeyValue::new(key, AnyValue::new_string(value))
    }

    fn integer(key: &str, value: i64) -> KeyValue {
        KeyValue::new(key, AnyValue::new_int(value))
    }

    fn messages(bytes: &[u8]) -> Vec<&[u8]> {
        assert_eq!(&bytes[..4], b"SLG1");
        let count = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let mut cursor = BATCH_HEADER_LEN;
        let mut messages = Vec::new();
        while messages.len() < count {
            let run_count = u32::from_be_bytes(bytes[cursor + 8..cursor + 12].try_into().unwrap());
            assert_ne!(run_count, 0);
            cursor += RUN_HEADER_LEN;
            for _ in 0..run_count {
                let len =
                    u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
                cursor += FRAME_HEADER_LEN;
                messages.push(&bytes[cursor..cursor + len]);
                cursor += len;
            }
        }
        assert_eq!(messages.len(), count);
        assert_eq!(cursor, bytes.len());
        messages
    }

    fn configured_service(options: &str) -> Result<CodecService, CodecError> {
        let config: CodecsConfig = serde_yaml::from_str(&format!(
            "bindings:\n  syslog-batch:\n    options: {options}"
        ))
        .unwrap();
        CodecServiceBuilder::from_global_registry()
            .unwrap()
            .with_configuration(CodecsConfig::resolve([&config]))
            .build()
    }

    fn cef_attributes() -> Vec<KeyValue> {
        vec![
            text("cef.device_vendor", "vendor"),
            text("cef.device_product", "product"),
            text("cef.device_version", "1.0"),
            text("cef.device_event_class_id", "100"),
            text("cef.name", "event"),
            integer("cef.severity", 6),
            text("src", "192.0.2.1"),
        ]
    }

    fn decode(bytes: Bytes) -> Vec<LogRecord> {
        use prost::Message;
        let service = CodecService::new().unwrap();
        let codec = super::super::resolve_syslog().unwrap();
        let records = OtapPayload::from(codec.admit(SignalType::Logs, bytes).unwrap())
            .try_into_otap(&service)
            .unwrap();
        let otlp: OtlpProtoBytes = records.try_into_with_default().unwrap();
        let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp else {
            panic!("expected logs");
        };
        LogsData::decode(bytes)
            .unwrap()
            .resource_logs
            .into_iter()
            .flat_map(|resource| resource.scope_logs)
            .flat_map(|scope| scope.log_records)
            .collect()
    }

    /// Scenario: CEF fields contain plain spans, adjacent escapes, UTF-8 bytes, or output exactly at the message-size limit.
    /// Guarantees: Span copying preserves header/extension escaping byte-for-byte and rejects expansion beyond 16 KiB.
    #[test]
    fn syslog_encoder_escaping_spans_preserves_bytes_and_limits() {
        for (header, input, expected) in [
            (true, b"".as_slice(), b"".as_slice()),
            (false, b"plain text".as_slice(), b"plain text".as_slice()),
            (true, br"|a\=b|".as_slice(), br"\|a\\=b\|".as_slice()),
            (
                false,
                b"\\a=b\n\rc|".as_slice(),
                br"\\a\=b\n\rc|".as_slice(),
            ),
            (false, b"\xc3\xa9".as_slice(), b"\xc3\xa9".as_slice()),
        ] {
            let mut message = b"prefix:".to_vec();
            append_escaped(&mut message, input, header).unwrap();
            assert_eq!(&message[..7], b"prefix:");
            assert_eq!(&message[7..], expected);
        }
        for header in [false, true] {
            let mut message = vec![b'x'; MAX_MESSAGE_SIZE - 2];
            append_escaped(&mut message, b"\\", header).unwrap();
            assert_eq!(message.len(), MAX_MESSAGE_SIZE);
            assert_eq!(&message[MAX_MESSAGE_SIZE - 2..], b"\\\\");
            let mut message = vec![b'x'; MAX_MESSAGE_SIZE - 1];
            assert!(append_escaped(&mut message, b"\\", header).is_err());
            assert!(message.len() <= MAX_MESSAGE_SIZE);
            let mut message = Vec::new();
            let plain = vec![b'x'; MAX_MESSAGE_SIZE];
            append_escaped(&mut message, &plain, header).unwrap();
            assert_eq!(message, plain);
            assert!(append_escaped(&mut message, b"x", header).is_err());
        }
    }

    /// Scenario: Physical logs and attributes are interleaved across resources and scopes.
    /// Guarantees: The codec associates headers and messages with row IDs, preserving hierarchy order and each parent's duplicate precedence.
    #[test]
    fn syslog_encoder_index_matches_reordered_hierarchy() {
        let group = |host, messages: &[&[&str]]| ResourceLogs {
            resource: Some(Resource {
                attributes: vec![text("host.name", host)],
                ..Default::default()
            }),
            scope_logs: messages
                .iter()
                .map(|messages| ScopeLogs {
                    log_records: messages
                        .iter()
                        .map(|message| {
                            let mut log = log("");
                            log.attributes = vec![
                                text("syslog.message", "overridden"),
                                text("syslog.message", message),
                            ];
                            log
                        })
                        .collect(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let mut input = encode_logs_otap_batch(&LogsData {
            resource_logs: vec![
                group("first", &[&["one", "two"], &["three"]]),
                group("second", &[&["four"]]),
                group("third", &[&["five"]]),
            ],
        })
        .unwrap();
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        let expected = encoder.encode(input.clone()).unwrap();
        for payload in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
        ] {
            let batch = input.get(payload).unwrap();
            let order = if payload == ArrowPayloadType::Logs {
                vec![3, 2, 0, 4, 1]
            } else {
                let parents = MaybeDictArrayAccessor::<UInt16Array>::try_new(
                    batch.column_by_name(consts::PARENT_ID).unwrap(),
                )
                .unwrap();
                let mut ranks = std::collections::BTreeMap::new();
                let mut order: Vec<_> = (0..batch.num_rows())
                    .map(|row| {
                        let rank = ranks.entry(parents.value_at(row)).or_insert(0);
                        let result = (*rank, row as u32);
                        *rank += 1;
                        result
                    })
                    .collect();
                order.sort_unstable();
                order.into_iter().map(|(_, row)| row).collect()
            };
            let order = UInt32Array::from(order);
            let columns = batch
                .columns()
                .iter()
                .map(|column| arrow::compute::take(column.as_ref(), &order, None).unwrap())
                .collect();
            let reordered = RecordBatch::try_new(batch.schema(), columns).unwrap();
            input.set(payload, reordered).unwrap();
        }
        assert_eq!(encoder.encode(input).unwrap(), expected);
    }

    /// Scenario: Logs override cached resource defaults across scopes and subsequent resource groups.
    /// Guarantees: Per-log overrides never change later defaults, and each resource supplies only its own header fields.
    #[test]
    fn syslog_encoder_resource_defaults_are_isolated() {
        let group = |attributes, logs: Vec<Vec<LogRecord>>| ResourceLogs {
            resource: Some(Resource {
                attributes,
                ..Default::default()
            }),
            scope_logs: logs
                .into_iter()
                .map(|log_records| ScopeLogs {
                    log_records,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };
        let mut overridden = log("override");
        overridden.attributes = vec![
            text("syslog.host_name", "override-host"),
            text("syslog.app_name", "override-app"),
            text("syslog.process_id_str", r"opaque\=pid"),
        ];
        let input = encode_logs_otap_batch(&LogsData {
            resource_logs: vec![
                group(
                    vec![
                        text("host.name", "first"),
                        text("service.name", "first-app"),
                        integer("process.pid", 11),
                    ],
                    vec![vec![overridden, log("same-scope")], vec![log("next-scope")]],
                ),
                group(
                    vec![
                        text("host.name", "second"),
                        text("service.name", "second-app"),
                        integer("process.pid", 22),
                    ],
                    vec![vec![log("next-resource")]],
                ),
                group(vec![], vec![vec![log("no-defaults")]]),
            ],
        })
        .unwrap();
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        let bytes = encoder.encode(input).unwrap();
        assert_eq!(
            messages(&bytes),
            [
                br"<14>1 - override-host override-app opaque\=pid - - override".as_slice(),
                b"<14>1 - first first-app 11 - - same-scope",
                b"<14>1 - first first-app 11 - - next-scope",
                b"<14>1 - second second-app 22 - - next-resource",
                b"<14>1 - - - - - - no-defaults",
            ]
        );
    }

    /// Scenario: Invalid resource headers are overridden by valid per-log fields in each syslog profile.
    /// Guarantees: Unused resource values cannot reject logs, while invalid fallbacks still fail when selected.
    #[test]
    fn syslog_encoder_validates_only_resolved_defaults() {
        for format in [
            Format::Rfc3164,
            Format::Rfc5424,
            Format::CefRfc3164,
            Format::CefRfc5424,
        ] {
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            for (override_key, fallback) in [
                ("syslog.host_name", integer("host.name", 1)),
                ("syslog.host_name", text("host.name", &"x".repeat(256))),
                ("syslog.app_name", text("service.name", &"x".repeat(49))),
                ("syslog.process_id", integer("process.pid", -1)),
            ] {
                let mut input = log("body");
                input.time_unix_nano = 1_704_164_645_000_000_000;
                input.attributes = cef_attributes();
                input.attributes.extend([
                    text("syslog.host_name", "host"),
                    text("syslog.app_name", "app"),
                    integer("syslog.process_id", 42),
                ]);
                let bytes = encoder
                    .encode(records(vec![input.clone()], vec![fallback.clone()]))
                    .unwrap();
                let decoded = decode(bytes);
                for expected in [
                    text("syslog.host_name", "host"),
                    text("syslog.app_name", "app"),
                    integer("syslog.process_id", 42),
                ] {
                    assert!(decoded[0].attributes.contains(&expected));
                }
                input
                    .attributes
                    .retain(|attribute| attribute.key != override_key);
                assert!(
                    encoder
                        .encode(records(vec![input], vec![fallback]))
                        .is_err()
                );
            }
        }
    }

    /// Scenario: RFC 5424 PROCIDs have leading zeros, a minus sign, or opaque text alongside numeric attributes in either order.
    /// Guarantees: The selected string survives parser re-encoding unchanged, unused numeric values are ignored, and invalid selected strings fail.
    #[test]
    fn syslog_encoder_preserves_opaque_procids() {
        for format in [Format::Rfc5424, Format::CefRfc5424] {
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            for (process, number) in [("00123", 123), ("-42", -42), ("opaque", -1)] {
                for reverse in [false, true] {
                    let mut input = log("body");
                    input.attributes = cef_attributes();
                    input.attributes.extend([
                        text("syslog.process_id_str", process),
                        integer("syslog.process_id", number),
                    ]);
                    if reverse {
                        input.attributes.reverse();
                    }
                    let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
                    let prefix = format!("<14>1 - - - {process} - - ");
                    assert!(messages(&bytes)[0].starts_with(prefix.as_bytes()));
                    let decoded = decode(bytes.clone());
                    assert!(
                        decoded[0]
                            .attributes
                            .contains(&text("syslog.process_id_str", process))
                    );
                    let rewritten = encoder.encode(records(decoded, vec![])).unwrap();
                    assert_eq!(messages(&rewritten), messages(&bytes));
                }
            }
            for invalid in [
                text("syslog.process_id_str", "bad value"),
                text("syslog.process_id_str", &"x".repeat(129)),
                integer("syslog.process_id_str", 42),
            ] {
                let mut input = log("body");
                input.attributes = cef_attributes();
                input
                    .attributes
                    .extend([integer("syslog.process_id", 42), invalid]);
                assert!(encoder.encode(records(vec![input], vec![])).is_err());
            }
        }
    }

    /// Scenario: Borrowed CEF fields contain signed integer extremes and custom keys are replaced in unsorted order.
    /// Guarantees: Text fields borrow native storage, decimal formatting is exact, and custom output preserves bounded last-value-wins replacement regardless of order.
    #[test]
    fn syslog_encoder_borrows_fields_and_formats_integers() {
        let mut input = log("");
        input.attributes = cef_attributes();
        input.attributes.extend([
            integer("cef.extension.z", i64::MAX),
            text("cef.extension.a", "borrowed"),
            integer("cef.extension.m", i64::MIN),
            integer("spt", 0),
        ]);
        let batch = records(vec![input], vec![]);
        let array = StructArray::from(batch.get(ArrowPayloadType::LogAttrs).unwrap().clone());
        let columns = AttributeColumns::new(&array).unwrap();
        let mut index = AttributeIndex::default();
        index.rebuild(Some(&columns));
        let mut fields = CefFields::default();
        for (key, value) in columns.attributes(index.rows_for(0)) {
            fields.apply(key, value).unwrap();
            if key == b"cef.device_vendor" || key == b"cef.extension.a" {
                let OtapAnyValueView::Str(source) = value else {
                    panic!("expected native string");
                };
                let borrowed = if key == b"cef.device_vendor" {
                    fields.headers[0].unwrap()
                } else {
                    let (stored_key, value) = fields
                        .custom_extensions
                        .get_key_value(b"a".as_slice())
                        .unwrap();
                    assert_eq!(stored_key.as_ptr(), key[b"cef.extension.".len()..].as_ptr());
                    *value
                };
                let FieldValue::Text(borrowed) = borrowed else {
                    panic!("expected borrowed CEF text");
                };
                assert_eq!(borrowed.as_ptr(), source.as_ptr());
                assert_eq!(borrowed.len(), source.len());
            }
        }
        fields
            .apply(b"cef.extension.a", OtapAnyValueView::Int(0))
            .unwrap();
        let mut bytes = Vec::new();
        fields.append(&mut bytes, None).unwrap();
        let extensions: Vec<_> = bytes
            .strip_prefix(b"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 spt=0 ")
            .unwrap()
            .split(|byte| *byte == b' ')
            .collect();
        assert_eq!(extensions.len(), 3);
        for expected in [
            b"a=0".as_slice(),
            b"m=-9223372036854775808",
            b"z=9223372036854775807",
        ] {
            assert!(extensions.contains(&expected));
        }
        let mut fields = CefFields::default();
        fields
            .apply(b"cef.extension.a", OtapAnyValueView::Int(i64::MIN))
            .unwrap();
        let text = vec![b'x'; MAX_MESSAGE_SIZE - 1];
        fields
            .apply(b"cef.extension.a", OtapAnyValueView::Str(&text))
            .unwrap();
        assert!(
            fields
                .apply(b"cef.extension.b", OtapAnyValueView::Int(0))
                .is_err()
        );
    }

    /// Scenario: Bare CEF records contain invalid resource and syslog header fields that wrapped CEF would use.
    /// Guarantees: Lazy resource validation and the shared attribute pass preserve bare CEF's independence from syslog metadata.
    #[test]
    fn syslog_encoder_bare_cef_ignores_syslog_headers() {
        let mut input = log("body");
        input.attributes = cef_attributes();
        input.attributes.push(integer("syslog.host_name", 42));
        let batch = records(vec![input], vec![integer("host.name", 42)]);
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
        let bytes = encoder.encode(batch.clone()).unwrap();
        assert_eq!(
            messages(&bytes),
            [b"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=body".as_slice()]
        );
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::CefRfc5424);
        assert!(encoder.encode(batch).is_err());
    }

    /// Scenario: Every fixed profile prepares repeated batches and detaches output before another encode.
    /// Guarantees: Borrowed output reuses message and batch allocations, framing matches the shared builder, and owned bytes remain independent.
    #[test]
    fn syslog_encoder_reuses_buffers_and_preserves_owned_output() {
        for format in ALL_FORMATS {
            let mut input = log("body");
            input.time_unix_nano = 1_704_164_645_000_000_000;
            input.observed_time_unix_nano = input.time_unix_nano + 7;
            input.attributes = cef_attributes();
            let mut later = input.clone();
            later.time_unix_nano = 1_735_787_045_000_000_000;
            let mut batch = records(vec![input.clone(), later, input], vec![]);
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            let output = encoder.prepare_encode(&mut batch).unwrap();
            let pointer = output.as_ref().as_ptr();
            let expected = Bytes::copy_from_slice(output.as_ref());
            let years = match format {
                Format::Rfc3164 | Format::CefRfc3164 => [2024, 2025, 2024],
                _ => [1970; 3],
            };
            let mut builder = SyslogBatchBuilder::new();
            for (message, year) in messages(&expected).into_iter().zip(years) {
                builder.append_with_year(message, year).unwrap();
            }
            assert_eq!(
                builder
                    .finish(1_704_164_645_000_000_007)
                    .unwrap()
                    .into_parts()
                    .0,
                expected
            );
            drop(output);
            let message_pointer = encoder.message.as_ptr();
            let output = encoder.prepare_encode(&mut batch).unwrap();
            assert_eq!(output.as_ref().as_ptr(), pointer);
            assert_eq!(output.as_ref(), expected.as_ref());
            let owned = output.into_bytes();
            assert_eq!(encoder.message.as_ptr(), message_pointer);
            let mut different = log("different");
            different.time_unix_nano = 1_704_164_645_000_000_000;
            different.attributes = cef_attributes();
            let output = encoder.encode(records(vec![different], vec![])).unwrap();
            assert_ne!(output, owned);
            assert_eq!(owned, expected);
        }
    }

    /// Scenario: Large batches succeed or fail after filling reusable storage, followed by a small valid batch.
    /// Guarantees: Success and failure bound retained capacity, failed output is cleared, and subsequent output contains no stale frames.
    #[test]
    fn syslog_encoder_bounds_retained_buffers_and_recovers() {
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        let large = records(vec![log(&"x".repeat(8192)); 64], vec![]);
        let output = encoder.prepare_encode(&mut large.clone()).unwrap();
        assert!(output.as_ref().len() > MAX_RETAINED_BUFFER_CAPACITY);
        drop(output);
        assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
        let owned = encoder.encode(large.clone()).unwrap();
        assert!(owned.len() > MAX_RETAINED_BUFFER_CAPACITY);
        assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
        let mut invalid = log("invalid");
        invalid.body = Some(AnyValue::new_int(1));
        let mut inputs = vec![log(&"x".repeat(8192)); 64];
        inputs.push(invalid);
        assert!(
            encoder
                .prepare_encode(&mut records(inputs, vec![]))
                .is_err()
        );
        assert!(encoder.buffer.is_empty());
        assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
        assert!(encoder.message.is_empty());
        let output = encoder.encode(records(vec![log("ok")], vec![])).unwrap();
        assert_eq!(messages(&output), vec![b"<14>1 - - - - - - ok".as_slice()]);
        assert_eq!(messages(&owned).len(), 64);
    }

    /// Scenario: Pipeline codec bindings select each profile for the same native log fields.
    /// Guarantees: Production factories emit each expected wire format, escape CEF values, and decode the chosen format and message.
    #[test]
    fn syslog_encoder_profiles_use_codec_bindings() {
        let time = 1_704_164_645_123_456_789;
        for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
            let service = configured_service(&format!("{{profile: {profile}}}")).unwrap();
            let mut input = log("event=1 \\ path\nnext\rline");
            input.time_unix_nano = time;
            input.observed_time_unix_nano = time + 1;
            input.attributes = cef_attributes();
            input.attributes.extend([
                text("syslog.app_name", "app"),
                text("syslog.msg_id", "ID47"),
                integer("syslog.process_id", 42),
                integer("syslog.facility", 16),
                integer("syslog.severity", 6),
                // Input metadata must not override the output profile.
                text("input.format", "unrelated"),
            ]);
            let mut payload =
                OtapPayload::from(records(vec![input], vec![text("host.name", "host")]));
            let plan = EncodingPlan::resolve(
                service.registry(),
                &PdataEncoding::new("syslog-batch"),
                EncodePolicy::default(),
            )
            .unwrap();
            let bytes = payload.encode_bytes(&service, &plan).unwrap();
            let body = if profile.starts_with("cef") {
                r"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=event\=1 \\ path\nnext\rline"
            } else {
                "event=1 \\ path\nnext\rline"
            };
            let expected = match profile {
                "cef" => body.to_owned(),
                "rfc3164" | "cef_rfc3164" => {
                    let local = DateTime::from_timestamp_nanos(time as i64).with_timezone(&Local);
                    format!("<134>{} host app[42]: {body}", local.format("%b %e %T"))
                }
                _ => format!("<134>1 2024-01-02T03:04:05.123456Z host app 42 ID47 - {body}"),
            };
            assert_eq!(messages(&bytes), vec![expected.as_bytes()], "{profile}");
            let decoded = decode(bytes);
            assert_eq!(decoded.len(), 1);
            assert_eq!(decoded[0].observed_time_unix_nano, time + 1);
            assert!(
                decoded[0]
                    .attributes
                    .contains(&text("input.format", profile))
            );
            if profile.starts_with("cef") {
                assert!(
                    decoded[0]
                        .attributes
                        .contains(&text("msg", "event=1 \\ path\nnext\rline"))
                );
            }
            match profile {
                "cef" => assert_eq!(decoded[0].time_unix_nano, 0),
                "rfc3164" | "cef_rfc3164" => {
                    assert_eq!(
                        decoded[0].time_unix_nano,
                        time / 1_000_000_000 * 1_000_000_000
                    );
                }
                _ => assert_eq!(decoded[0].time_unix_nano, time / 1000 * 1000),
            }
        }
    }

    /// Scenario: Parser-style message attributes coexist with absent, empty, or transformed bodies in every output format.
    /// Guarantees: Empty bodies preserve attributes, format-specific fields win regardless of order, and nonempty bodies remain authoritative through re-encoding.
    #[test]
    fn syslog_encoder_uses_parser_message_attributes() {
        let time = 1_704_164_645_000_000_000;
        for format in ALL_FORMATS {
            for (body, replacement) in [
                (None, None),
                (Some(AnyValue::default()), None),
                (Some(AnyValue::new_string("")), None),
                (
                    Some(AnyValue::new_string("transformed message")),
                    Some("transformed message"),
                ),
            ] {
                for reverse in [false, true] {
                    let mut attributes = cef_attributes();
                    attributes.extend([
                        text("syslog.content", "RFC 3164 message"),
                        text("syslog.message", "RFC 5424 message"),
                        text("msg", "CEF message"),
                    ]);
                    if reverse {
                        attributes.reverse();
                    }
                    let input = LogRecord {
                        body: body.clone(),
                        time_unix_nano: time,
                        attributes,
                        ..Default::default()
                    };
                    let (key, expected) = match format {
                        Format::Rfc3164 => ("syslog.content", "RFC 3164 message"),
                        Format::Rfc5424 => ("syslog.message", "RFC 5424 message"),
                        _ => ("msg", "CEF message"),
                    };
                    let expected = replacement.unwrap_or(expected);
                    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
                    let decoded = decode(encoder.encode(records(vec![input], vec![])).unwrap());
                    assert!(decoded[0].body.is_none());
                    assert!(decoded[0].attributes.contains(&text(key, expected)));
                    let decoded = decode(encoder.encode(records(decoded, vec![])).unwrap());
                    assert!(decoded[0].body.is_none());
                    assert!(decoded[0].attributes.contains(&text(key, expected)));
                }
            }
        }
    }

    /// Scenario: Message fallbacks have an invalid type or size while the body is present, empty, absent, or invalid.
    /// Guarantees: Only nonempty string bodies override invalid fallbacks in every profile; selected invalid values still fail.
    #[test]
    fn syslog_encoder_validates_only_selected_messages() {
        let oversized = "x".repeat(MAX_MESSAGE_SIZE + 1);
        for format in ALL_FORMATS {
            let keys: &[&str] = if format.is_cef() {
                &["msg"]
            } else {
                &["syslog.message", "syslog.content"]
            };
            for key in keys {
                for invalid in [AnyValue::new_bool(true), AnyValue::new_string(&oversized)] {
                    for body in [
                        None,
                        Some(AnyValue::default()),
                        Some(AnyValue::new_string("")),
                        Some(AnyValue::new_string("ok")),
                        Some(AnyValue::new_int(42)),
                    ] {
                        let mut attributes = cef_attributes();
                        attributes.push(KeyValue::new(*key, invalid.clone()));
                        let input = LogRecord {
                            body: body.clone(),
                            time_unix_nano: 1_704_164_645_000_000_000,
                            attributes,
                            ..Default::default()
                        };
                        let mut encoder =
                            SyslogEncoder::for_format(EncodePolicy::default(), format);
                        let result = encoder.encode(records(vec![input], vec![]));
                        if body == Some(AnyValue::new_string("ok")) {
                            let decoded = decode(result.unwrap());
                            let output_key = match format {
                                Format::Rfc3164 => "syslog.content",
                                Format::Rfc5424 => "syslog.message",
                                _ => "msg",
                            };
                            assert!(decoded[0].attributes.contains(&text(output_key, "ok")));
                        } else {
                            assert!(result.is_err(), "{key} with body {body:?}");
                        }
                    }
                }
            }
        }
    }

    /// Scenario: A valid format-specific syslog message coexists with an invalid lower-priority fallback.
    /// Guarantees: Fallback precedence is order-independent, and only the selected attribute is validated.
    #[test]
    fn syslog_encoder_validates_only_preferred_message_attribute() {
        for (format, preferred, fallback) in [
            (Format::Rfc3164, "syslog.content", "syslog.message"),
            (Format::Rfc5424, "syslog.message", "syslog.content"),
        ] {
            for reverse in [false, true] {
                for body in [None, Some(AnyValue::new_string(""))] {
                    for invalid_preferred in [false, true] {
                        let mut attributes = vec![
                            text(
                                if invalid_preferred {
                                    fallback
                                } else {
                                    preferred
                                },
                                "ok",
                            ),
                            KeyValue::new(
                                if invalid_preferred {
                                    preferred
                                } else {
                                    fallback
                                },
                                AnyValue::new_bool(true),
                            ),
                        ];
                        if reverse {
                            attributes.reverse();
                        }
                        let input = LogRecord {
                            body: body.clone(),
                            time_unix_nano: 1_704_164_645_000_000_000,
                            attributes,
                            ..Default::default()
                        };
                        let mut encoder =
                            SyslogEncoder::for_format(EncodePolicy::default(), format);
                        let result = encoder.encode(records(vec![input], vec![]));
                        if invalid_preferred {
                            assert!(result.is_err());
                        } else {
                            let decoded = decode(result.unwrap());
                            assert!(decoded[0].attributes.contains(&text(preferred, "ok")));
                        }
                    }
                }
            }
        }
    }

    /// Scenario: An explicitly selected profile receives already encoded syslog in another format.
    /// Guarantees: Explicit output settings rewrite compatible envelopes instead of forwarding the old wire format.
    #[test]
    fn syslog_encoder_profile_rewrites_matching_encoding() {
        let service = configured_service("{profile: rfc5424}").unwrap();
        let codec = super::super::resolve_syslog().unwrap();
        let mut builder = SyslogBatchBuilder::new();
        builder
            .append_with_year(b"<134>Jun 15 12:00:00 host app[42]: original", 2024)
            .unwrap();
        let mut payload: OtapPayload = codec.admit_built(builder.finish(123).unwrap()).unwrap();
        let plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let rewritten = payload.encode_bytes(&service, &plan).unwrap();
        assert!(messages(&rewritten)[0].starts_with(b"<134>1 "));
        let decoded = decode(rewritten);
        assert!(
            decoded[0]
                .attributes
                .contains(&text("syslog.message", "original"))
        );
    }

    /// Scenario: RFC 3164 logs cross years and output budgets include each receipt-year run.
    /// Guarantees: Encoded years survive deferred decoding, second precision is explicit, and the complete framed-byte limit is enforced.
    #[test]
    fn syslog_encoder_rfc3164_preserves_year_runs_and_limits() {
        let times: Vec<_> = [2024, 2025, 2024]
            .map(|year| {
                Utc.with_ymd_and_hms(year, 6, 15, 12, 0, 0)
                    .unwrap()
                    .timestamp_nanos_opt()
                    .unwrap() as u64
            })
            .into();
        let inputs: Vec<_> = times
            .iter()
            .map(|time| LogRecord {
                time_unix_nano: *time,
                ..log("message")
            })
            .collect();
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc3164);
        let bytes = encoder.encode(records(inputs.clone(), vec![])).unwrap();
        assert_eq!(messages(&bytes).len(), 3);
        let decoded = decode(bytes.clone());
        assert_eq!(
            decoded
                .iter()
                .map(|log| log.time_unix_nano)
                .collect::<Vec<_>>(),
            times
        );
        encoder.max_bytes = bytes.len() - 1;
        assert!(matches!(
            encoder.encode(records(inputs.clone(), vec![])),
            Err(CodecError::EncodedSizeLimitExceeded { .. })
        ));
        encoder.max_bytes += 1;
        assert_eq!(encoder.encode(records(inputs, vec![])).unwrap(), bytes);
    }

    /// Scenario: Observed timestamps repeat, change, decrease, or are absent within the same year, including explicit profile rewrites.
    /// Guarantees: Every profile preserves per-record observed time, coalesces only adjacent equal contexts, and budgets all run headers exactly.
    #[test]
    fn syslog_encoder_preserves_observed_runs() {
        let observed_times = [100, 100, 200, 0, 100];
        for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
            let inputs: Vec<_> = observed_times
                .iter()
                .map(|observed| LogRecord {
                    time_unix_nano: 1_704_164_645_000_000_000,
                    observed_time_unix_nano: *observed,
                    attributes: cef_attributes(),
                    ..log("body")
                })
                .collect();
            let mut encoder =
                SyslogEncoder::for_format(EncodePolicy::default(), Format::parse(profile).unwrap());
            let bytes = encoder.encode(records(inputs.clone(), vec![])).unwrap();
            let decoded = decode(bytes.clone());
            assert_eq!(
                decoded
                    .iter()
                    .map(|log| log.observed_time_unix_nano)
                    .collect::<Vec<_>>(),
                observed_times,
                "{profile}"
            );
            let expected_size = BATCH_HEADER_LEN
                + 4 * RUN_HEADER_LEN
                + messages(&bytes)
                    .iter()
                    .map(|message| FRAME_HEADER_LEN + message.len())
                    .sum::<usize>();
            assert_eq!(bytes.len(), expected_size);
            encoder.max_bytes = expected_size - 1;
            assert!(matches!(
                encoder.encode(records(inputs.clone(), vec![])),
                Err(CodecError::EncodedSizeLimitExceeded { actual, limit, .. })
                    if actual == expected_size && limit == expected_size - 1
            ));
            encoder.max_bytes = expected_size;
            assert_eq!(encoder.encode(records(inputs, vec![])).unwrap(), bytes);

            let service = configured_service(&format!("{{profile: {profile}}}")).unwrap();
            let plan = EncodingPlan::resolve(
                service.registry(),
                &PdataEncoding::new("syslog-batch"),
                EncodePolicy::default(),
            )
            .unwrap();
            let mut payload =
                OtapPayload::from(plan.codec().admit(SignalType::Logs, bytes).unwrap());
            let rewritten = payload.encode_bytes(&service, &plan).unwrap();
            assert_eq!(
                decode(rewritten)
                    .iter()
                    .map(|log| log.observed_time_unix_nano)
                    .collect::<Vec<_>>(),
                observed_times,
                "{profile}"
            );
        }
    }

    /// Scenario: Profile options or mapped CEF/RFC 3164 fields are invalid, missing, or oversized after escaping.
    /// Guarantees: Invalid requests fail explicitly, CEF headers escape delimiters, and errors do not poison encoder reuse.
    #[test]
    fn syslog_encoder_profiles_reject_invalid_fields_and_options() {
        for options in ["{profile: PRIVATE}", "{profile: 1}", "{unknown: true}"] {
            let error = configured_service(options).err().unwrap().to_string();
            assert!(!error.contains("PRIVATE"));
        }
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
        assert!(
            encoder
                .encode(records(vec![log("message")], vec![]))
                .is_err()
        );
        for attribute in [
            integer("cef.version", 2),
            text("cef.severity", "11"),
            text("cef.name", "PRIVATE\ninjected"),
            text("cef.name", ""),
        ] {
            let mut input = log("PRIVATE");
            input.attributes = cef_attributes();
            input
                .attributes
                .retain(|existing| existing.key != attribute.key);
            input.attributes.push(attribute);
            let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
            assert!(!error.to_string().contains("PRIVATE"));
        }
        let mut input = log(&"\\".repeat(MAX_MESSAGE_SIZE / 2));
        input.attributes = cef_attributes();
        assert!(encoder.encode(records(vec![input], vec![])).is_err());
        let mut input = log("ok");
        input.attributes = cef_attributes();
        input.attributes.retain(|attr| attr.key != "cef.name");
        input.attributes.push(text("cef.name", "pipe|slash\\"));
        let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
        assert_eq!(
            messages(&bytes),
            vec![br"CEF:0|vendor|product|1.0|100|pipe\|slash\\|6|src=192.0.2.1 msg=ok".as_slice()]
        );
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc3164);
        assert!(
            encoder
                .encode(records(vec![log("missing timestamp")], vec![]))
                .is_err()
        );
        let mut input = log("message");
        input.time_unix_nano = 1_704_164_645_000_000_000;
        input.attributes = vec![text("syslog.app_name", "injected:tag")];
        assert!(encoder.encode(records(vec![input], vec![])).is_err());
    }

    /// Scenario: Vendor CEF extensions contain delimiters, invalid keys, or excessive escaped output.
    /// Guarantees: Explicit custom fields round-trip with escaping, unrelated attributes stay private, and invalid or oversized extensions fail without poisoning reuse.
    #[test]
    fn syslog_encoder_vendor_extensions_are_explicit_and_bounded() {
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
        for key in [
            "cef.extension.",
            "cef.extension.bad key",
            "cef.extension.bad=key",
            "cef.extension.msg",
        ] {
            let mut input = log("body");
            input.attributes = cef_attributes();
            input.attributes.push(text(key, "PRIVATE"));
            let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
            assert!(!error.to_string().contains("PRIVATE"));
        }
        for value in [
            "x".repeat(MAX_MESSAGE_SIZE),
            "\\".repeat(MAX_MESSAGE_SIZE / 2),
        ] {
            let mut input = log("body");
            input.attributes = cef_attributes();
            input
                .attributes
                .push(text("cef.extension.vendorField", &value));
            assert!(encoder.encode(records(vec![input], vec![])).is_err());
        }
        let mut input = log("body");
        input.attributes = cef_attributes();
        input.attributes.extend([
            text("cef.extension.cs1Label", "PolicyName"),
            text("cef.extension.cs1", "rule=web\\allow\nnext\rline"),
            integer("cef.extension.cn1", 42),
            text("cef.extension.vendorZone", "trusted"),
            text("private.attribute", "PRIVATE"),
        ]);
        let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
        let wire_messages = messages(&bytes);
        assert_eq!(wire_messages.len(), 1);
        let extensions: Vec<_> = wire_messages[0]
            .strip_prefix(b"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=body ")
            .unwrap()
            .split(|byte| *byte == b' ')
            .collect();
        assert_eq!(extensions.len(), 4);
        for expected in [
            b"cn1=42".as_slice(),
            br"cs1=rule\=web\\allow\nnext\rline",
            b"cs1Label=PolicyName",
            b"vendorZone=trusted",
        ] {
            assert!(extensions.contains(&expected));
        }
        let decoded = decode(bytes);
        assert!(
            decoded[0]
                .attributes
                .contains(&text("cs1", "rule=web\\allow\nnext\rline"))
        );
        assert!(decoded[0].attributes.contains(&text("cn1", "42")));
        assert!(
            decoded[0]
                .attributes
                .contains(&text("vendorZone", "trusted"))
        );
    }

    /// Scenario: The all-profile codec consumes uneven batches, encounters a mid-batch failure, and is instantiated in another service.
    /// Guarantees: Records rotate across all five formats across successful batches, failed batches leave the cursor unchanged, and instances rotate independently.
    #[test]
    fn syslog_encoder_all_profiles_round_robin_across_batches() {
        let input = || {
            let mut input = log("body");
            input.time_unix_nano = 1_704_164_645_000_000_000;
            input.attributes = cef_attributes();
            input.attributes.extend([
                text("syslog.host_name", "host"),
                text("syslog.app_name", "app"),
                integer("syslog.process_id", 42),
            ]);
            input
        };
        let service = configured_service("{profile: all}").unwrap();
        let plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let mut actual = Vec::new();
        for size in [2, 1, 4, 3] {
            if actual.len() == 2 {
                let mut invalid = input();
                invalid.attributes.clear();
                let mut payload = OtapPayload::from(records(vec![input(), invalid], vec![]));
                assert!(payload.encode_bytes(&service, &plan).is_err());
            }
            let mut payload =
                OtapPayload::from(records((0..size).map(|_| input()).collect(), vec![]));
            let bytes = payload.encode_bytes(&service, &plan).unwrap();
            for record in decode(bytes) {
                let attribute = record
                    .attributes
                    .iter()
                    .find(|attribute| attribute.key == "input.format")
                    .unwrap();
                actual.push(attribute.value.clone().unwrap());
            }
        }
        let expected = ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"];
        assert_eq!(
            actual,
            expected
                .into_iter()
                .cycle()
                .take(10)
                .map(AnyValue::new_string)
                .collect::<Vec<_>>()
        );
        let independent = configured_service("{profile: all}").unwrap();
        let independent_plan = EncodingPlan::resolve(
            independent.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let mut payload = OtapPayload::from(records(vec![input()], vec![]));
        let decoded = decode(
            payload
                .encode_bytes(&independent, &independent_plan)
                .unwrap(),
        );
        assert!(
            decoded[0]
                .attributes
                .contains(&text("input.format", "rfc3164"))
        );
    }

    /// Scenario: A native log has resource identity, explicit syslog overrides, and nanosecond time.
    /// Guarantees: The registered encoder emits the expected RFC 5424 bytes and a decodable envelope.
    #[test]
    fn syslog_encoder_maps_native_fields_and_decodes() {
        let time = 1_704_164_645_123_456_789;
        let mut input = log("request completed");
        input.time_unix_nano = time;
        input.observed_time_unix_nano = time + 1;
        input.severity_number = 17;
        input.attributes = vec![
            text("syslog.host_name", "edge"),
            text("syslog.msg_id", "event"),
            integer("syslog.facility", 20),
            integer("syslog.severity", 6),
        ];
        let mut records = records(
            vec![input],
            vec![
                text("host.name", "fallback"),
                text("service.name", "app"),
                integer("process.pid", 42),
            ],
        );
        records.encode_transport_optimized().unwrap();
        let codecs = CodecService::new().unwrap();
        let plan = EncodingPlan::resolve(
            codecs.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let bytes = codecs.encode_bytes(&mut records, &plan).unwrap();
        assert_eq!(
            messages(&bytes),
            vec![
                b"<166>1 2024-01-02T03:04:05.123456Z edge app 42 event - request completed"
                    .as_slice()
            ]
        );
        let mut payload: OtapPayload = plan.codec().admit(SignalType::Logs, bytes).unwrap().into();
        assert_eq!(payload.num_items(), 1);
        let mut decoded = payload.try_into_otap(&codecs).unwrap();
        decoded.decode_transport_optimized_ids().unwrap();
        let view = OtapLogsView::try_from(&decoded).unwrap();
        let resource = view.resources().next().unwrap();
        let scope = resource.scopes().next().unwrap();
        let log = scope.log_records().next().unwrap();
        assert_eq!(log.time_unix_nano(), Some(time / 1000 * 1000));
        assert_eq!(log.observed_time_unix_nano(), Some(time + 1));
        assert!(log.attributes().any(|attribute| {
            attribute.key() == b"syslog.message"
                && attribute.value().unwrap().as_string() == Some(b"request completed")
        }));
    }

    /// Scenario: Logs omit headers and bodies or use standard OTel severity levels.
    /// Guarantees: Missing values become NILVALUE, syslog message attributes remain usable, and levels map explicitly.
    #[test]
    fn syslog_encoder_handles_missing_fields_and_severity() {
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        let mut inputs = vec![LogRecord::default()];
        for level in [1, 5, 9, 13, 17, 21] {
            inputs.push(LogRecord {
                severity_number: level,
                body: Some(AnyValue::new_string("message")),
                ..Default::default()
            });
        }
        inputs.push(LogRecord {
            attributes: vec![text("syslog.message", "forwarded")],
            ..Default::default()
        });
        let bytes = encoder.encode(records(inputs, vec![])).unwrap();
        assert_eq!(
            messages(&bytes),
            vec![
                b"<14>1 - - - - - -".as_slice(),
                b"<15>1 - - - - - - message",
                b"<15>1 - - - - - - message",
                b"<14>1 - - - - - - message",
                b"<12>1 - - - - - - message",
                b"<11>1 - - - - - - message",
                b"<10>1 - - - - - - message",
                b"<14>1 - - - - - - forwarded",
            ]
        );
    }

    /// Scenario: Inputs exceed message, item, batch-byte, or output-policy limits.
    /// Guarantees: No oversized batch succeeds, exact message boundaries work, and failures do not poison reuse.
    #[test]
    fn syslog_encoder_enforces_bounds_and_recovers() {
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        let prefix = b"<14>1 - - - - - - ";
        let body = "x".repeat(MAX_MESSAGE_SIZE - prefix.len());
        let bytes = encoder.encode(records(vec![log(&body)], vec![])).unwrap();
        assert_eq!(messages(&bytes)[0].len(), MAX_MESSAGE_SIZE);
        assert!(
            encoder
                .encode(records(vec![log(&(body.clone() + "x"))], vec![]))
                .is_err()
        );
        assert!(encoder.encode(records(vec![], vec![])).is_err());
        assert!(
            encoder
                .encode(records(vec![log("x"); MAX_BATCH_ITEMS + 1], vec![]))
                .is_err()
        );
        assert!(
            encoder
                .encode(records(vec![log(&body); 128], vec![]))
                .is_err()
        );
        let mut limited = SyslogEncoder::new(EncodePolicy {
            max_encoded_size: std::num::NonZeroUsize::new(32),
        });
        assert!(matches!(
            limited.encode(records(vec![log("x")], vec![])),
            Err(CodecError::EncodedSizeLimitExceeded { limit: 32, .. })
        ));
        let bytes = encoder.encode(records(vec![log("ok")], vec![])).unwrap();
        assert_eq!(messages(&bytes), vec![b"<14>1 - - - - - - ok".as_slice()]);
    }

    /// Scenario: Mapped headers contain invalid types, ranges, separators, or unsupported structured data.
    /// Guarantees: Invalid inputs fail explicitly without truncation, content injection, or telemetry in diagnostics.
    #[test]
    fn syslog_encoder_rejects_unrepresentable_fields() {
        let mut encoder = SyslogEncoder::new(EncodePolicy::default());
        for attribute in [
            text("syslog.host_name", "PRIVATE\ninjected"),
            text("syslog.app_name", &"x".repeat(49)),
            integer("syslog.facility", 24),
            integer("syslog.severity", -1),
            text("syslog.severity", "PRIVATE"),
            text("syslog.structured_data", "[PRIVATE]"),
        ] {
            let mut input = log("PRIVATE");
            input.attributes.push(attribute);
            let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
            assert!(!error.to_string().contains("PRIVATE"));
        }
        let mut input = log("unused");
        input.body = Some(AnyValue::new_int(42));
        assert!(encoder.encode(records(vec![input], vec![])).is_err());
        let mut input = log("message");
        input.severity_number = 25;
        assert!(encoder.encode(records(vec![input], vec![])).is_err());
    }
}
