// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoding helpers for Linux user_events samples.
//!
//! The receiver handles two wire/layout layers:
//!
//! ```text
//! Linux tracepoint sample
//! +----------------------+----------------------------------+
//! | common_* fields      | producer-defined data            |
//! | tracefs metadata     | tracefs fields or EventHeader    |
//! +----------------------+----------------------------------+
//! 0                      user_data_offset
//! ```
//!
//! In `tracefs` mode, the producer-defined data is decoded with the field
//! offsets, sizes, locations, and type names from the tracefs `format` file.
//! This is the standard Linux tracepoint model used by perf/ftrace. After a
//! successful tracefs decode, downstream processors receive the decoded OTAP log
//! attributes, not the original raw tracepoint sample bytes. Unknown static
//! fields may be preserved as per-field base64 string attributes.
//!
//! Tracefs field values are currently decoded from live same-host samples, so
//! numeric conversion uses the host byte order. Cross-endian perf data should
//! move this path to `tracepoint_decode` so field extraction can use the
//! sample's source byte order.
//! TODO: Revisit tracefs decoding with `tracepoint_decode` and add explicit
//! `data_loc`, `rel_loc`, and cross-endian coverage before supporting offline
//! or cross-host perf data.
//!
//! References:
//! - Linux `user_events` registration and write ABI:
//!   <https://docs.kernel.org/trace/user_events.html>
//! - tracefs event `format` files, `common_*` fields, offsets, and sizes:
//!   <https://docs.kernel.org/trace/events.html#event-formats>
//!
//! In EventHeader mode, the producer-defined data is decoded as an
//! EventHeader payload:
//!
//! ```text
//! EventHeader payload
//! +-------------+----------------------+------------------+
//! | EventHeader | extension block(s)   | event data       |
//! | 8 bytes     | metadata, activity   | field values     |
//! +-------------+----------------------+------------------+
//!
//! Metadata extension
//! +----------------------+-------------------------------+
//! | event name, NUL      | field metadata blocks         |
//! +----------------------+-------------------------------+
//!
//! Field metadata block
//! +----------------------+----------+----------+----------+
//! | field name, NUL      | encoding | format?  | tag?     |
//! +----------------------+----------+----------+----------+
//! ```
//!
//! EventHeader is decoded structurally and vendor-neutrally here. Structs are
//! flattened into dot-separated attribute names, e.g. `PartB.body` or
//! `Request.path`, but this module does not attach semantic meaning to any
//! particular struct or field name. Schema-specific interpretation, such as
//! Microsoft Common Schema `PartA`/`PartB`/`PartC` promotion, belongs in a
//! processor. If EventHeader decoding fails, only the user payload region is
//! preserved as `linux.user_events.payload_base64`.

use std::borrow::Cow;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
#[cfg(feature = "user_events-eventheader")]
use tracepoint_decode::{
    EventHeaderEnumeratorContext, EventHeaderEnumeratorState, EventHeaderItemInfo,
};

use super::FormatConfig;
use super::session::{RawUserEventsRecord, TracefsField, TracefsFieldLocation};

// FieldEncoding raw values (from eventheader_types, avoids a new dep)
#[cfg(feature = "user_events-eventheader")]
const ENC_VALUE8: u8 = 2;
#[cfg(feature = "user_events-eventheader")]
const ENC_VALUE16: u8 = 3;
#[cfg(feature = "user_events-eventheader")]
const ENC_VALUE32: u8 = 4;
#[cfg(feature = "user_events-eventheader")]
const ENC_VALUE64: u8 = 5;
#[cfg(feature = "user_events-eventheader")]
const ENC_STRING_LENGTH16_CHAR8: u8 = 10;

// FieldFormat raw values
#[cfg(feature = "user_events-eventheader")]
const FMT_SIGNED_INT: u8 = 2;
#[cfg(feature = "user_events-eventheader")]
const FMT_BOOLEAN: u8 = 7;
#[cfg(feature = "user_events-eventheader")]
const FMT_FLOAT: u8 = 8;

/// Typed attribute value carried on a decoded user_events record.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum DecodedAttrValue {
    Str(String),
    Int(i64),
    Bool(bool),
    Double(f64),
}

impl PartialEq<str> for DecodedAttrValue {
    fn eq(&self, other: &str) -> bool {
        matches!(self, Self::Str(s) if s == other)
    }
}

impl PartialEq<&str> for DecodedAttrValue {
    fn eq(&self, other: &&str) -> bool {
        matches!(self, Self::Str(s) if s == *other)
    }
}

/// A decoded user_events record ready for Arrow encoding.
///
/// This intentionally keeps the decoder independent from the Arrow builders for
/// now, but it also means decoded strings and attributes are staged in owned
/// intermediate values before encoding.
/// TODO: Rework the receiver so decoding can write directly into
/// `ArrowRecordsBuilder`/`LogAttrs` builders and avoid these temporary
/// allocations on the hot path.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct DecodedUserEventsRecord {
    /// Event timestamp in Unix epoch nanoseconds.
    pub time_unix_nano: i64,
    /// Optional log body string.
    pub body: Option<String>,
    /// Optional promoted event name for the typed OTLP log field.
    pub event_name: Option<String>,
    /// Optional severity number.
    pub severity_number: Option<i32>,
    /// Optional severity text.
    pub severity_text: Option<Cow<'static, str>>,
    /// Optional OTLP log flags.
    pub flags: Option<u32>,
    /// Optional W3C trace id (16 bytes).
    pub trace_id: Option<[u8; 16]>,
    /// Optional W3C span id (8 bytes).
    pub span_id: Option<[u8; 8]>,
    /// Additional structured attributes, preserving the source typed value.
    pub attributes: Vec<(Cow<'static, str>, DecodedAttrValue)>,
}

impl DecodedUserEventsRecord {
    pub(super) fn from_raw(
        tracepoint: &str,
        value: RawUserEventsRecord,
        format: &FormatConfig,
    ) -> Self {
        match format {
            FormatConfig::Tracefs => Self::from_tracefs(tracepoint, value),
            #[cfg(feature = "user_events-eventheader")]
            FormatConfig::EventHeader => Self::from_eventheader(tracepoint, value),
        }
    }

    fn base_record(
        tracepoint: &str,
        value: RawUserEventsRecord,
        mut attributes: Vec<(Cow<'static, str>, DecodedAttrValue)>,
    ) -> Self {
        if let Some(process_id) = value.process_id {
            attributes.push((
                Cow::Borrowed("linux.user_events.process.pid"),
                DecodedAttrValue::Int(i64::from(process_id)),
            ));
        }
        if let Some(thread_id) = value.thread_id {
            attributes.push((
                Cow::Borrowed("linux.user_events.thread.id"),
                DecodedAttrValue::Int(i64::from(thread_id)),
            ));
        }

        Self {
            time_unix_nano: i64::try_from(value.timestamp_unix_nano).unwrap_or(i64::MAX),
            body: None,
            event_name: Some(tracepoint.to_owned()),
            severity_number: None,
            severity_text: None,
            flags: None,
            trace_id: None,
            span_id: None,
            attributes,
        }
    }

    fn from_tracefs(tracepoint: &str, value: RawUserEventsRecord) -> Self {
        let mut attributes = Vec::with_capacity(value.fields.len());
        for field in value.fields.iter() {
            // Receiver-internal transport fields are intentionally not emitted
            // as log attributes: the Ingestion backend treats OTLP attributes
            // as backend columns, so surfacing per-record diagnostics there
            // would pollute the application schema.
            if field.name.starts_with("common_") {
                continue;
            }
            if let Some(decoded) = decode_tracefs_field(field, &value.event_data) {
                attributes.push((Cow::Owned(field.name.clone()), decoded));
            }
        }
        Self::base_record(tracepoint, value, attributes)
    }

    #[cfg(feature = "user_events-eventheader")]
    fn from_eventheader(tracepoint: &str, value: RawUserEventsRecord) -> Self {
        let payload = value
            .event_data
            .get(value.user_data_offset..)
            .unwrap_or(value.event_data.as_slice());
        let attributes = decode_eventheader_attrs(tracepoint, payload);
        Self::base_record(tracepoint, value, attributes)
    }
}

fn decode_tracefs_field(field: &TracefsField, event_data: &[u8]) -> Option<DecodedAttrValue> {
    let bytes = tracefs_field_bytes(field, event_data)?;
    let type_name = field.type_name.trim();
    if matches!(field.location, TracefsFieldLocation::StaticString)
        || (type_name == "char" && field.size != 1)
    {
        return std::str::from_utf8(bytes)
            .ok()
            .map(|s| DecodedAttrValue::Str(s.trim_end_matches('\0').to_owned()));
    }

    match (type_name, bytes.len()) {
        ("bool" | "_Bool", 1) => Some(DecodedAttrValue::Bool(bytes[0] != 0)),
        ("char" | "signed char" | "s8" | "__s8" | "int8_t", 1) => {
            Some(DecodedAttrValue::Int(i64::from(bytes[0] as i8)))
        }
        ("unsigned char" | "u8" | "__u8" | "uint8_t", 1) => {
            Some(DecodedAttrValue::Int(i64::from(bytes[0])))
        }
        ("short" | "signed short" | "s16" | "__s16" | "int16_t", 2) => Some(DecodedAttrValue::Int(
            i64::from(i16::from_ne_bytes(bytes.try_into().ok()?)),
        )),
        ("unsigned short" | "u16" | "__u16" | "uint16_t", 2) => Some(DecodedAttrValue::Int(
            i64::from(u16::from_ne_bytes(bytes.try_into().ok()?)),
        )),
        ("int" | "signed int" | "s32" | "__s32" | "int32_t", 4) => Some(DecodedAttrValue::Int(
            i64::from(i32::from_ne_bytes(bytes.try_into().ok()?)),
        )),
        ("unsigned int" | "u32" | "__u32" | "uint32_t", 4) => Some(DecodedAttrValue::Int(
            i64::from(u32::from_ne_bytes(bytes.try_into().ok()?)),
        )),
        (
            "long" | "signed long" | "long long" | "signed long long" | "s64" | "__s64" | "int64_t",
            8,
        ) => Some(DecodedAttrValue::Int(i64::from_ne_bytes(
            bytes.try_into().ok()?,
        ))),
        ("unsigned long" | "unsigned long long" | "u64" | "__u64" | "uint64_t", 8) => {
            Some(DecodedAttrValue::Int(
                i64::try_from(u64::from_ne_bytes(bytes.try_into().ok()?)).unwrap_or(i64::MAX),
            ))
        }
        ("float", 4) => Some(DecodedAttrValue::Double(f64::from(f32::from_ne_bytes(
            bytes.try_into().ok()?,
        )))),
        ("double", 8) => Some(DecodedAttrValue::Double(f64::from_ne_bytes(
            bytes.try_into().ok()?,
        ))),
        _ => Some(DecodedAttrValue::Str(BASE64_STANDARD.encode(bytes))),
    }
}

fn tracefs_field_bytes<'a>(field: &TracefsField, event_data: &'a [u8]) -> Option<&'a [u8]> {
    match field.location {
        TracefsFieldLocation::Static => {
            let end = field.offset.checked_add(field.size)?;
            event_data.get(field.offset..end)
        }
        TracefsFieldLocation::StaticString => {
            let end = field.offset.checked_add(field.size)?;
            event_data.get(field.offset..end).map(until_nul)
        }
        TracefsFieldLocation::StaticUtf16String => {
            let end = field.offset.checked_add(field.size)?;
            event_data.get(field.offset..end)
        }
        TracefsFieldLocation::StaticLenPrefixArray => {
            let len_end = field.offset.checked_add(2)?;
            let len_bytes = event_data.get(field.offset..len_end)?;
            let len = usize::from(u16::from_ne_bytes(len_bytes.try_into().ok()?));
            let bytes_len = len.checked_mul(field.size)?;
            let end = len_end.checked_add(bytes_len)?;
            event_data.get(len_end..end)
        }
        TracefsFieldLocation::DynAbsolute | TracefsFieldLocation::DynRelative => {
            if field.size != 4 {
                return None;
            }
            let loc_end = field.offset.checked_add(field.size)?;
            let loc_bytes = event_data.get(field.offset..loc_end)?;
            let loc = u32::from_ne_bytes(loc_bytes.try_into().ok()?);
            let len = (loc >> 16) as usize;
            let mut start = (loc & 0xFFFF) as usize;
            if field.location == TracefsFieldLocation::DynRelative {
                start = start.checked_add(loc_end)?;
            }
            let end = start.checked_add(len)?;
            event_data.get(start..end)
        }
    }
}

fn until_nul(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

#[cfg(feature = "user_events-eventheader")]
fn decode_eventheader_attrs(
    tracepoint: &str,
    payload: &[u8],
) -> Vec<(Cow<'static, str>, DecodedAttrValue)> {
    let mut attrs = Vec::new();
    let mut context = EventHeaderEnumeratorContext::new();
    let Ok(mut en) = context.enumerate_with_name_and_data(
        tracepoint,
        payload,
        EventHeaderEnumeratorContext::MOVE_NEXT_LIMIT_DEFAULT,
    ) else {
        attrs.push((
            Cow::Borrowed("linux.user_events.payload_base64"),
            DecodedAttrValue::Str(BASE64_STANDARD.encode(payload)),
        ));
        return attrs;
    };

    let mut prefix: Vec<String> = Vec::new();
    while en.move_next() {
        match en.state() {
            EventHeaderEnumeratorState::StructBegin => {
                if let Ok(name) = std::str::from_utf8(en.item_info().name_bytes()) {
                    prefix.push(name.to_owned());
                }
            }
            EventHeaderEnumeratorState::StructEnd => {
                let _ = prefix.pop();
            }
            EventHeaderEnumeratorState::Value => {
                let item = en.item_info();
                let Ok(name) = std::str::from_utf8(item.name_bytes()) else {
                    continue;
                };
                if let Some(value) = item_as_any_scalar_value(&item) {
                    let key = if prefix.is_empty() {
                        name.to_owned()
                    } else {
                        format!("{}.{}", prefix.join("."), name)
                    };
                    attrs.push((Cow::Owned(key), value));
                }
            }
            EventHeaderEnumeratorState::Error | EventHeaderEnumeratorState::AfterLastItem => break,
            _ => {}
        }
    }
    attrs
}

#[cfg(feature = "user_events-eventheader")]
fn item_as_any_scalar_value(item: &EventHeaderItemInfo<'_>) -> Option<DecodedAttrValue> {
    let enc = item.metadata().encoding().without_flags().as_int();
    let fmt = item.metadata().format().as_int();
    match enc {
        e if e == ENC_STRING_LENGTH16_CHAR8 => std::str::from_utf8(item.value().bytes())
            .ok()
            .map(|s| DecodedAttrValue::Str(s.to_owned())),
        e if e == ENC_VALUE8 => {
            if fmt == FMT_BOOLEAN {
                Some(DecodedAttrValue::Bool(item.value().to_u8(0) != 0))
            } else if fmt == FMT_SIGNED_INT {
                Some(DecodedAttrValue::Int(
                    i64::from(item.value().to_u8(0) as i8),
                ))
            } else {
                Some(DecodedAttrValue::Int(i64::from(item.value().to_u8(0))))
            }
        }
        e if e == ENC_VALUE16 => {
            if fmt == FMT_SIGNED_INT {
                Some(DecodedAttrValue::Int(i64::from(item.value().to_i16(0))))
            } else {
                Some(DecodedAttrValue::Int(i64::from(item.value().to_u16(0))))
            }
        }
        e if e == ENC_VALUE32 => {
            if fmt == FMT_FLOAT {
                Some(DecodedAttrValue::Double(f64::from(f32::from_bits(
                    item.value().to_u32(0),
                ))))
            } else if fmt == FMT_BOOLEAN {
                Some(DecodedAttrValue::Bool(item.value().to_u32(0) != 0))
            } else if fmt == FMT_SIGNED_INT {
                Some(DecodedAttrValue::Int(i64::from(item.value().to_i32(0))))
            } else {
                Some(DecodedAttrValue::Int(i64::from(item.value().to_u32(0))))
            }
        }
        e if e == ENC_VALUE64 => {
            if fmt == FMT_FLOAT {
                Some(DecodedAttrValue::Double(f64::from_bits(
                    item.value().to_u64(0),
                )))
            } else if fmt == FMT_SIGNED_INT {
                Some(DecodedAttrValue::Int(item.value().to_i64(0)))
            } else {
                Some(DecodedAttrValue::Int(
                    i64::try_from(item.value().to_u64(0)).unwrap_or(i64::MAX),
                ))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[cfg(feature = "user_events-eventheader")]
    const ENC_STRUCT: u8 = 1;
    #[cfg(feature = "user_events-eventheader")]
    const FMT_DEFAULT: u8 = 0;

    fn raw_record(fields: Vec<TracefsField>, event_data: Vec<u8>) -> RawUserEventsRecord {
        raw_record_with_sample_metadata(fields, event_data, None, None)
    }

    fn raw_record_with_sample_metadata(
        fields: Vec<TracefsField>,
        event_data: Vec<u8>,
        process_id: Option<u32>,
        thread_id: Option<u32>,
    ) -> RawUserEventsRecord {
        RawUserEventsRecord {
            subscription_index: 0,
            timestamp_unix_nano: 42,
            process_id,
            thread_id,
            event_data,
            user_data_offset: 8,
            fields: Arc::from(fields),
        }
    }

    fn field(
        name: &str,
        type_name: &str,
        location: TracefsFieldLocation,
        offset: usize,
        size: usize,
    ) -> TracefsField {
        TracefsField {
            name: name.to_owned(),
            type_name: type_name.to_owned(),
            location,
            offset,
            size,
        }
    }

    fn decode_tracefs(fields: Vec<TracefsField>, event_data: Vec<u8>) -> DecodedUserEventsRecord {
        DecodedUserEventsRecord::from_raw(
            "user_events:my_event",
            raw_record(fields, event_data),
            &FormatConfig::Tracefs,
        )
    }

    fn attr<'a>(decoded: &'a DecodedUserEventsRecord, key: &str) -> Option<&'a DecodedAttrValue> {
        decoded
            .attributes
            .iter()
            .find_map(|(attr_key, value)| (attr_key == key).then_some(value))
    }

    #[cfg(feature = "user_events-eventheader")]
    fn add_value_field_meta(meta: &mut Vec<u8>, name: &str, encoding: u8, format: u8) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0);
        meta.push(encoding | 0x80);
        meta.push(format);
    }

    #[cfg(feature = "user_events-eventheader")]
    fn add_string_field_meta(meta: &mut Vec<u8>, name: &str) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0);
        meta.push(ENC_STRING_LENGTH16_CHAR8);
    }

    #[cfg(feature = "user_events-eventheader")]
    fn add_struct_meta(meta: &mut Vec<u8>, name: &str, field_count: u8) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0);
        meta.push(0x80 | ENC_STRUCT);
        meta.push(field_count);
    }

    #[cfg(feature = "user_events-eventheader")]
    fn add_string_data(data: &mut Vec<u8>, value: &str) {
        data.extend_from_slice(&(value.len() as u16).to_le_bytes());
        data.extend_from_slice(value.as_bytes());
    }

    #[cfg(feature = "user_events-eventheader")]
    fn build_eventheader_payload(
        event_name: &str,
        level: u8,
        meta_fields: &[u8],
        data: &[u8],
    ) -> Vec<u8> {
        let mut payload = vec![0x07, 0, 0, 0, 0, 0, 0, level];
        let mut meta = Vec::new();
        meta.extend_from_slice(event_name.as_bytes());
        meta.push(0);
        meta.extend_from_slice(meta_fields);

        payload.extend_from_slice(&(meta.len() as u16).to_le_bytes());
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(&meta);
        payload.extend_from_slice(data);
        payload
    }

    #[test]
    fn tracefs_decodes_standard_fields_as_attributes() {
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&200u32.to_ne_bytes());
        event_data.extend_from_slice(b"/checkout\0");
        let fields = vec![
            field("common_pid", "int", TracefsFieldLocation::Static, 4, 4),
            field("status", "u32", TracefsFieldLocation::Static, 8, 4),
            field(
                "endpoint",
                "char",
                TracefsFieldLocation::StaticString,
                12,
                10,
            ),
        ];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(decoded.event_name.as_deref(), Some("user_events:my_event"));
        assert!(decoded.body.is_none());
        assert_eq!(decoded.attributes.len(), 2);
        assert_eq!(decoded.attributes[0].0, "status");
        assert_eq!(decoded.attributes[0].1, DecodedAttrValue::Int(200));
        assert_eq!(decoded.attributes[1].0, "endpoint");
        assert_eq!(decoded.attributes[1].1, "/checkout");
    }

    #[test]
    fn tracefs_includes_perf_process_and_thread_metadata() {
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&200u32.to_ne_bytes());
        let fields = vec![field("status", "u32", TracefsFieldLocation::Static, 8, 4)];
        let raw = raw_record_with_sample_metadata(fields, event_data, Some(123), Some(456));

        let decoded =
            DecodedUserEventsRecord::from_raw("user_events:my_event", raw, &FormatConfig::Tracefs);

        assert_eq!(attr(&decoded, "status"), Some(&DecodedAttrValue::Int(200)));
        assert_eq!(
            attr(&decoded, "linux.user_events.process.pid"),
            Some(&DecodedAttrValue::Int(123))
        );
        assert_eq!(
            attr(&decoded, "linux.user_events.thread.id"),
            Some(&DecodedAttrValue::Int(456))
        );
    }

    #[test]
    fn tracefs_decodes_scalar_field_types() {
        let mut event_data = vec![0; 8];
        event_data.push(1);
        event_data.extend_from_slice(&(-42i32).to_ne_bytes());
        event_data.extend_from_slice(&123u64.to_ne_bytes());
        event_data.extend_from_slice(&2.5f64.to_ne_bytes());
        let fields = vec![
            field("ok", "bool", TracefsFieldLocation::Static, 8, 1),
            field("delta", "s32", TracefsFieldLocation::Static, 9, 4),
            field("count", "u64", TracefsFieldLocation::Static, 13, 8),
            field("duration_ms", "double", TracefsFieldLocation::Static, 21, 8),
        ];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(attr(&decoded, "ok"), Some(&DecodedAttrValue::Bool(true)));
        assert_eq!(attr(&decoded, "delta"), Some(&DecodedAttrValue::Int(-42)));
        assert_eq!(attr(&decoded, "count"), Some(&DecodedAttrValue::Int(123)));
        assert_eq!(
            attr(&decoded, "duration_ms"),
            Some(&DecodedAttrValue::Double(2.5))
        );
    }

    #[test]
    fn tracefs_decodes_dynamic_relative_string() {
        let mut event_data = vec![0; 8];
        let value = b"/dynamic\0";
        let location = (value.len() as u32) << 16;
        event_data.extend_from_slice(&location.to_ne_bytes());
        event_data.extend_from_slice(value);
        let fields = vec![field(
            "endpoint",
            "char",
            TracefsFieldLocation::DynRelative,
            8,
            4,
        )];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(
            attr(&decoded, "endpoint"),
            Some(&DecodedAttrValue::Str("/dynamic".to_owned()))
        );
    }

    #[test]
    fn tracefs_bounds_static_string_by_field_size() {
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(b"abcd");
        event_data.extend_from_slice(b"efgh\0");
        let fields = vec![field(
            "endpoint",
            "char",
            TracefsFieldLocation::StaticString,
            8,
            4,
        )];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(
            attr(&decoded, "endpoint"),
            Some(&DecodedAttrValue::Str("abcd".to_owned()))
        );
    }

    #[test]
    fn tracefs_rejects_dynamic_location_with_unexpected_size() {
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&0u16.to_ne_bytes());
        let fields = vec![field(
            "endpoint",
            "char",
            TracefsFieldLocation::DynRelative,
            8,
            2,
        )];

        let decoded = decode_tracefs(fields, event_data);

        assert!(decoded.attributes.is_empty());
    }

    #[test]
    fn tracefs_preserves_dynamic_non_string_bytes_without_nul_truncation() {
        let mut event_data = vec![0; 8];
        let value = [b'a', 0, b'b', 0];
        let location = (value.len() as u32) << 16;
        event_data.extend_from_slice(&location.to_ne_bytes());
        event_data.extend_from_slice(&value);
        let fields = vec![field(
            "opaque",
            "struct opaque",
            TracefsFieldLocation::DynRelative,
            8,
            4,
        )];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(
            attr(&decoded, "opaque"),
            Some(&DecodedAttrValue::Str("YQBiAA==".to_owned()))
        );
    }

    #[test]
    fn tracefs_preserves_unknown_static_bytes_as_base64() {
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        let fields = vec![field(
            "opaque",
            "struct opaque",
            TracefsFieldLocation::Static,
            8,
            4,
        )];

        let decoded = decode_tracefs(fields, event_data);

        assert_eq!(
            attr(&decoded, "opaque"),
            Some(&DecodedAttrValue::Str("3q2+7w==".to_owned()))
        );
    }

    #[test]
    fn tracefs_skips_fields_outside_event_data() {
        let fields = vec![field(
            "missing",
            "u32",
            TracefsFieldLocation::Static,
            128,
            4,
        )];

        let decoded = decode_tracefs(fields, vec![0; 8]);

        assert!(decoded.attributes.is_empty());
    }

    #[cfg(feature = "user_events-eventheader")]
    #[test]
    fn eventheader_decodes_microsoft_common_schema_shape_as_flattened_attributes() {
        let mut meta = Vec::new();
        let mut data = Vec::new();

        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        data.extend_from_slice(&0x400u32.to_le_bytes());

        add_struct_meta(&mut meta, "PartA", 6);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "2024-06-15T12:00:00Z");
        add_string_field_meta(&mut meta, "ext_dt_traceId");
        add_string_data(&mut data, "0102030405060708090a0b0c0d0e0f10");
        add_string_field_meta(&mut meta, "ext_dt_spanId");
        add_string_data(&mut data, "a1b2c3d4e5f60718");
        add_value_field_meta(&mut meta, "ext_dt_traceFlags", ENC_VALUE8, FMT_DEFAULT);
        data.push(1);
        add_string_field_meta(&mut meta, "ext_cloud_role");
        add_string_data(&mut data, "checkout");
        add_string_field_meta(&mut meta, "ext_cloud_roleInstance");
        add_string_data(&mut data, "instance-1");

        add_struct_meta(&mut meta, "PartB", 6);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "hello");
        add_string_field_meta(&mut meta, "name");
        add_string_data(&mut data, "CheckoutFailure");
        add_value_field_meta(&mut meta, "severityNumber", ENC_VALUE32, FMT_SIGNED_INT);
        data.extend_from_slice(&17i32.to_le_bytes());
        add_string_field_meta(&mut meta, "severityText");
        add_string_data(&mut data, "ERROR");
        add_value_field_meta(&mut meta, "eventId", ENC_VALUE64, FMT_SIGNED_INT);
        data.extend_from_slice(&42i64.to_le_bytes());

        add_struct_meta(&mut meta, "PartC", 1);
        add_value_field_meta(&mut meta, "status", ENC_VALUE32, FMT_SIGNED_INT);
        data.extend_from_slice(&500i32.to_le_bytes());

        let payload = build_eventheader_payload("Log", 4, &meta, &data);
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&payload);
        let decoded = DecodedUserEventsRecord::from_raw(
            "user_events:myprovider_L4K1",
            raw_record(Vec::new(), event_data),
            &FormatConfig::EventHeader,
        );

        assert_eq!(
            attr(&decoded, "__csver__"),
            Some(&DecodedAttrValue::Int(0x400))
        );
        assert_eq!(
            attr(&decoded, "PartA.ext_dt_traceId"),
            Some(&DecodedAttrValue::Str(
                "0102030405060708090a0b0c0d0e0f10".to_owned()
            ))
        );
        assert_eq!(
            attr(&decoded, "PartA.time"),
            Some(&DecodedAttrValue::Str("2024-06-15T12:00:00Z".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartA.ext_dt_spanId"),
            Some(&DecodedAttrValue::Str("a1b2c3d4e5f60718".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartA.ext_dt_traceFlags"),
            Some(&DecodedAttrValue::Int(1))
        );
        assert_eq!(
            attr(&decoded, "PartA.ext_cloud_role"),
            Some(&DecodedAttrValue::Str("checkout".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartA.ext_cloud_roleInstance"),
            Some(&DecodedAttrValue::Str("instance-1".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartB._typeName"),
            Some(&DecodedAttrValue::Str("Log".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartB.body"),
            Some(&DecodedAttrValue::Str("hello".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartB.name"),
            Some(&DecodedAttrValue::Str("CheckoutFailure".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartB.severityNumber"),
            Some(&DecodedAttrValue::Int(17))
        );
        assert_eq!(
            attr(&decoded, "PartB.severityText"),
            Some(&DecodedAttrValue::Str("ERROR".to_owned()))
        );
        assert_eq!(
            attr(&decoded, "PartB.eventId"),
            Some(&DecodedAttrValue::Int(42))
        );
        assert_eq!(
            attr(&decoded, "PartC.status"),
            Some(&DecodedAttrValue::Int(500))
        );
    }

    #[cfg(feature = "user_events-eventheader")]
    #[test]
    fn eventheader_decodes_float_and_boolean_scalar_paths() {
        let mut meta = Vec::new();
        let mut data = Vec::new();

        // ENC_VALUE32 + FMT_FLOAT -> f32 reinterpreted from raw bits
        add_value_field_meta(&mut meta, "f32_val", ENC_VALUE32, FMT_FLOAT);
        let f32_value: f32 = -3.5e10;
        data.extend_from_slice(&f32_value.to_bits().to_le_bytes());

        // ENC_VALUE64 + FMT_FLOAT -> f64 reinterpreted from raw bits
        add_value_field_meta(&mut meta, "f64_val", ENC_VALUE64, FMT_FLOAT);
        let f64_value: f64 = std::f64::consts::PI;
        data.extend_from_slice(&f64_value.to_bits().to_le_bytes());

        // Negative float to make sure we are not accidentally going through
        // a signed-integer path (which would produce a wildly different value).
        add_value_field_meta(&mut meta, "f64_neg", ENC_VALUE64, FMT_FLOAT);
        let f64_neg: f64 = -1.5;
        data.extend_from_slice(&f64_neg.to_bits().to_le_bytes());

        // ENC_VALUE8 + FMT_BOOLEAN
        add_value_field_meta(&mut meta, "bool8_true", ENC_VALUE8, FMT_BOOLEAN);
        data.push(1);
        add_value_field_meta(&mut meta, "bool8_false", ENC_VALUE8, FMT_BOOLEAN);
        data.push(0);

        // ENC_VALUE32 + FMT_BOOLEAN
        add_value_field_meta(&mut meta, "bool32_true", ENC_VALUE32, FMT_BOOLEAN);
        data.extend_from_slice(&1u32.to_le_bytes());
        add_value_field_meta(&mut meta, "bool32_false", ENC_VALUE32, FMT_BOOLEAN);
        data.extend_from_slice(&0u32.to_le_bytes());
        // A non-zero, non-one value should still decode to true.
        add_value_field_meta(&mut meta, "bool32_other", ENC_VALUE32, FMT_BOOLEAN);
        data.extend_from_slice(&0xdead_beefu32.to_le_bytes());

        let payload = build_eventheader_payload("ScalarPaths", 4, &meta, &data);
        let mut event_data = vec![0; 8];
        event_data.extend_from_slice(&payload);
        let decoded = DecodedUserEventsRecord::from_raw(
            "user_events:scalars_L4K1",
            raw_record(Vec::new(), event_data),
            &FormatConfig::EventHeader,
        );

        assert_eq!(
            attr(&decoded, "f32_val"),
            Some(&DecodedAttrValue::Double(f64::from(f32_value)))
        );
        assert_eq!(
            attr(&decoded, "f64_val"),
            Some(&DecodedAttrValue::Double(f64_value))
        );
        assert_eq!(
            attr(&decoded, "f64_neg"),
            Some(&DecodedAttrValue::Double(f64_neg))
        );
        assert_eq!(
            attr(&decoded, "bool8_true"),
            Some(&DecodedAttrValue::Bool(true))
        );
        assert_eq!(
            attr(&decoded, "bool8_false"),
            Some(&DecodedAttrValue::Bool(false))
        );
        assert_eq!(
            attr(&decoded, "bool32_true"),
            Some(&DecodedAttrValue::Bool(true))
        );
        assert_eq!(
            attr(&decoded, "bool32_false"),
            Some(&DecodedAttrValue::Bool(false))
        );
        assert_eq!(
            attr(&decoded, "bool32_other"),
            Some(&DecodedAttrValue::Bool(true))
        );
    }

    #[cfg(feature = "user_events-eventheader")]
    #[test]
    fn eventheader_invalid_payload_is_preserved_as_attribute() {
        let decoded = DecodedUserEventsRecord::from_raw(
            "user_events:my_event",
            raw_record(Vec::new(), vec![0, 1, 2]),
            &FormatConfig::EventHeader,
        );

        assert!(decoded.body.is_none());
        assert_eq!(decoded.attributes.len(), 1);
        assert_eq!(decoded.attributes[0].0, "linux.user_events.payload_base64");
    }
}
