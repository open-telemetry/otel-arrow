// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog and CEF output from native logs in the existing syslog batch envelope.

use bytes::Bytes;
use chrono_tz::Tz;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::codecs::CodecConfiguration;
use otel_arrow_dfe_pdata::otlp::{BoundedBuf, ProtoBuffer};
use otel_arrow_dfe_pdata::views::otap::OtapLogsView;
use otel_arrow_dfe_pdata::views::otap::common::OtapAnyValueView;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_pdata_views::views::{
    common::{AnyValueView, AttributeView, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
    resource::ResourceView,
};

use crate::{
    CodecError, CodecOperation, EncodeOutput, EncodePolicy, PdataEncoder, PreparedCodecFactory,
    ResolvedCodec,
};
use std::{borrow::Cow, sync::Arc};

use super::codec::{BATCH_HEADER_LEN, FRAME_HEADER_LEN, MAGIC, RUN_HEADER_LEN};
use super::{MAX_BATCH_BYTES, MAX_BATCH_ITEMS, MAX_MESSAGE_SIZE};

mod cef;
mod fields;
mod rfc3164;
mod rfc5424;

use cef::CefFields;
use fields::{Header, append_bytes, severity};

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

    fn with_cef(self) -> Self {
        match self {
            Self::Rfc3164 => Self::CefRfc3164,
            Self::Rfc5424 => Self::CefRfc5424,
            other => other,
        }
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
        timezone: prepare_local_timezone(output_profile)?,
    }))
}

struct SyslogFactory {
    profile: Profile,
    explicit_profile: bool,
    timezone: Option<Tz>,
}

impl PreparedCodecFactory for SyslogFactory {
    fn create_encoder(
        &self,
        _: ResolvedCodec,
        policy: EncodePolicy,
    ) -> Result<Box<dyn PdataEncoder>, CodecError> {
        Ok(Box::new(SyslogEncoder::for_profile(
            policy,
            self.profile,
            self.timezone,
        )))
    }

    fn allows_encoded_forwarding(&self) -> bool {
        !self.explicit_profile
    }
}

fn parse_local_timezone(name: &str) -> Result<Tz, CodecError> {
    name.strip_prefix(':')
        .unwrap_or(name)
        .parse()
        .map_err(|error| {
            CodecError::configuration(format!(
                "RFC 3164 encoding requires a supported IANA timezone: {error}"
            ))
        })
}

fn prepare_local_timezone(profile: Profile) -> Result<Option<Tz>, CodecError> {
    match profile {
        Profile::Fixed(Format::Rfc3164 | Format::CefRfc3164) | Profile::All => {
            let name = match std::env::var("TZ") {
                Ok(name) => name,
                Err(std::env::VarError::NotPresent) => {
                    iana_time_zone::get_timezone().map_err(|error| {
                        CodecError::configuration(format!(
                            "cannot detect system IANA timezone for RFC 3164 encoding: {error}"
                        ))
                    })?
                }
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(CodecError::configuration(
                        "RFC 3164 encoding requires TZ to be a UTF-8 IANA timezone name",
                    ));
                }
            };
            parse_local_timezone(&name).map(Some)
        }
        _ => Ok(None),
    }
}

fn error(message: impl Into<Cow<'static, str>>) -> CodecError {
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
    timezone: Option<Tz>,
}

impl SyslogEncoder {
    pub(super) fn new(policy: EncodePolicy) -> Self {
        Self::for_profile(policy, Profile::Fixed(Format::default()), None)
    }

    fn for_profile(policy: EncodePolicy, profile: Profile, timezone: Option<Tz>) -> Self {
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
            timezone,
        }
    }

    #[cfg(test)]
    fn for_format(policy: EncodePolicy, format: Format) -> Self {
        let profile = Profile::Fixed(format);
        Self::for_profile(policy, profile, prepare_local_timezone(profile).unwrap())
    }
}

fn wire_time(
    format: Format,
    time: Option<u64>,
    timezone: Option<&Tz>,
    output: &mut String,
) -> Result<i32, CodecError> {
    output.clear();
    // The view exposes signed Arrow nanoseconds as u64 bits.
    let time = time.map(|value| value as i64);
    match format {
        Format::Cef => Ok(1970),
        Format::Rfc5424 | Format::CefRfc5424 => rfc5424::timestamp(time, output),
        Format::Rfc3164 | Format::CefRfc3164 => rfc3164::timestamp(time, timezone, output),
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
    let structured_data = rfc5424::structured_data(
        header.structured_data,
        matches!(format, Format::Rfc5424 | Format::CefRfc5424),
    )?;
    match format {
        Format::Cef => {}
        Format::Rfc5424 | Format::CefRfc5424 => {
            rfc5424::append_header(message, header, timestamp, structured_data)?
        }
        Format::Rfc3164 | Format::CefRfc3164 => rfc3164::append_header(message, header, timestamp)?,
    }
    let header_end = message.len();
    if format.is_cef() {
        if !message.is_empty() {
            append_bytes(message, b" ")?;
        }
        cef.ok_or_else(|| error("CEF encoding requires CEF fields"))?
            .append(message, body)?;
    } else if let Some(body) = body {
        if format == Format::Rfc5424 {
            rfc5424::append_message(message, structured_data, body)?;
        } else {
            append_bytes(message, b" ")?;
            append_bytes(message, body)?;
        }
    }
    if matches!(format, Format::Rfc5424 | Format::CefRfc5424) {
        let expected_body = if format.is_cef() {
            Some(&message[header_end + 1..])
        } else {
            body
        };
        rfc5424::validate_message(
            &message[header_end - structured_data.len()..],
            structured_data,
            expected_body,
        )?;
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
            return Err(error(format!(
                "syslog batches must contain 1..={MAX_BATCH_ITEMS} logs"
            )));
        }
        records
            .decode_transport_optimized_ids()
            .map_err(error_source)?;
        let view = OtapLogsView::try_from(&*records).map_err(error_source)?;
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
            let resource = resource_logs.resource();
            let resource_attributes = resource
                .as_ref()
                .map(|resource| resource.attributes().collect::<Vec<_>>())
                .unwrap_or_default();
            let mut resource_header = Header::default();
            let mut resource_initialized = false;
            for scope in resource_logs.scopes() {
                for log in scope.log_records() {
                    let mut format = match self.profile {
                        Profile::Fixed(format) => format,
                        Profile::All => ALL_FORMATS[next_format],
                    };
                    let attributes = log.attributes().collect::<Vec<_>>();
                    let parsed_cef = attributes.iter().any(|attribute| {
                        attribute.key() == b"input.format"
                            && attribute.value().is_some_and(|value| {
                                matches!(
                                    value.as_string(),
                                    Some(b"cef" | b"cef_rfc3164" | b"cef_rfc5424")
                                )
                            })
                    });
                    if parsed_cef {
                        format = format.with_cef();
                    }
                    let mut header = Header::default();
                    if format != Format::Cef {
                        if !resource_initialized {
                            for attribute in &resource_attributes {
                                let value = attribute.value().ok_or_else(|| {
                                    error("syslog resource attribute has no value")
                                })?;
                                resource_header.apply_resource(attribute.key(), value);
                            }
                            resource_initialized = true;
                        }
                        header = resource_header;
                    }
                    let mut cef = CefFields::new(parsed_cef);
                    // Field references must not outlive their attribute views.
                    for attribute in &attributes {
                        let key = attribute.key();
                        let value = attribute
                            .value()
                            .ok_or_else(|| error("syslog log attribute has no value"))?;
                        if format != Format::Cef {
                            header.apply_log(key, value)?;
                        } else if key == b"syslog.structured_data" {
                            header.structured_data = Some(value);
                        }
                        if format.is_cef() {
                            cef.apply(key, value)?;
                        }
                    }
                    if format != Format::Cef {
                        header.severity = Some(match header.severity {
                            Some(level) => level,
                            None => severity(log.severity_number())?,
                        });
                    }
                    let year = wire_time(
                        format,
                        log.time_unix_nano(),
                        self.timezone.as_ref(),
                        &mut self.timestamp,
                    )?;
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
                                _ => Err(error(format!(
                                    "syslog message text must be a string of at most {MAX_MESSAGE_SIZE} bytes"
                                ))),
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
                    // Recover the signed Arrow value exposed by the view.
                    let observed_time = log.observed_time_unix_nano().unwrap_or(0) as i64;
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
mod tests;
