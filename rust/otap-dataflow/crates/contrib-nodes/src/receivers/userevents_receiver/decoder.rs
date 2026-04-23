// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoding helpers for Linux userevents samples.

use std::borrow::Cow;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use tracepoint_decode::{
    EventHeaderEnumeratorContext, EventHeaderEnumeratorError, EventHeaderEnumeratorState,
    EventHeaderItemInfo,
};

use super::FormatConfig;
use super::session::RawUsereventsRecord;

// PartA-mapped OTel attribute keys
const ATTR_TRACE_ID: &str = "trace.id";
const ATTR_SPAN_ID: &str = "span.id";
const ATTR_EVENT_ID: &str = "eventId";
const ATTR_SERVICE_NAME: &str = "service.name";
const ATTR_SERVICE_INSTANCE_ID: &str = "service.instance.id";

// FieldEncoding raw values (from eventheader_types, avoids a new dep)
#[cfg(test)]
const ENC_STRUCT: u8 = 1;
const ENC_VALUE8: u8 = 2;
const ENC_VALUE16: u8 = 3;
const ENC_VALUE32: u8 = 4;
const ENC_VALUE64: u8 = 5;
const ENC_STRING_LENGTH16_CHAR8: u8 = 10;

// FieldFormat raw values
const FMT_DEFAULT: u8 = 0;
#[cfg(test)]
const FMT_UNSIGNED_INT: u8 = 1;
const FMT_SIGNED_INT: u8 = 2;
const FMT_BOOLEAN: u8 = 7;
const FMT_FLOAT: u8 = 8;
const FMT_STRING_UTF: u8 = 11;

/// Typed attribute value carried on a decoded userevents record.
///
/// The OTAP attribute Arrow builder supports `str`/`int`/`bool`/`double` value
/// types. Preserving the original typed encoding for PartC attributes and for
/// `eventId` lets the ingestion path emit Bond fields with the correct
/// `BT_*` type (e.g. `BT_INT64` instead of `BT_STRING` for numeric PartC
/// fields), which keeps numeric query predicates usable downstream.
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

// PartA reserved field names (used to skip conflicts in PartC)
const PART_A_RESERVED: &[&str] = &[
    "env_ver",
    "env_name",
    "env_time",
    "env_dt_traceId",
    "env_dt_spanId",
    "env_dt_traceFlags",
];
// PartB log field names (used to skip conflicts in PartC)
const PART_B_LOG_FIELDS: &[&str] = &["severityText", "severityNumber", "name", "eventId", "body"];

/// A decoded userevents record ready for Arrow encoding.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct DecodedUsereventsRecord {
    /// Event timestamp in Unix epoch nanoseconds.
    pub time_unix_nano: i64,
    /// Log body string.
    pub body: String,
    /// Optional promoted event name for the typed OTLP log field.
    pub event_name: Option<String>,
    /// Optional severity number.
    pub severity_number: Option<i32>,
    /// Optional severity text.
    pub severity_text: Option<Cow<'static, str>>,
    /// Optional OTLP log flags, used here for trace flags when present.
    pub flags: Option<u32>,
    /// Optional W3C trace id (16 bytes).
    pub trace_id: Option<[u8; 16]>,
    /// Optional W3C span id (8 bytes).
    pub span_id: Option<[u8; 8]>,
    /// Additional structured attributes, preserving the source typed value.
    pub attributes: Vec<(Cow<'static, str>, DecodedAttrValue)>,
}

impl DecodedUsereventsRecord {
    pub(super) fn from_raw(
        tracepoint: &str,
        value: RawUsereventsRecord,
        format: &FormatConfig,
        fallback_severity: Option<(i32, &'static str)>,
    ) -> (Self, bool) {
        // Receiver-internal diagnostics and transport identity (tracepoint,
        // provider, level, keyword, CPU/sample metadata) are deliberately not
        // pushed here. The Ingestion backend treats OTLP log attributes as
        // backend columns, so this path emits only typed OTLP fields and
        // application payload attrs.
        let _ = format;
        let mut attributes: Vec<(Cow<'static, str>, DecodedAttrValue)> = Vec::with_capacity(16);

        let mut time_override_nano: Option<i64> = None;
        let mut flags = None;
        let mut trace_id_typed: Option<[u8; 16]> = None;
        let mut span_id_typed: Option<[u8; 8]> = None;
        let body: String;
        let event_name: Option<String>;
        let mut severity_number: Option<i32>;
        let mut severity_text: Option<Cow<'static, str>>;
        let cs_failed: bool;

        if let Some(mut cs) = decode_cs_otel_logs(tracepoint, &value.payload) {
            // G1: The OTLP typed `event_name` column maps to the Bond `name`
            // field written by the Geneva uploader. The Rust user-events
            // exporter carries the user-visible event name in `PartB.name`
            // (e.g. "my-event-name"), while `EH.Name` is fixed to "Log" and
            // `PartA.name` is typically absent. Prefer PartB.name, then the
            // CS-level override (PartA.name or EH.Name), so the Ingestion
            // backend batches and writes per-event-name records instead of
            // collapsing them all to a single "Log" stream.
            event_name = Some(cs.part_b_name.take().unwrap_or(cs.event_name));
            body = cs.body.unwrap_or_default();
            severity_number = cs.severity_number;
            severity_text = cs.severity_text.map(Cow::Owned);
            // PartB.name is surfaced via the typed OTLP `event_name` column
            // above; no separate `cs.part_b.name` attribute is emitted so the
            // Ingestion backend does not see a redundant
            // `cs_part_b_name` backend column.
            if let Some(eid) = cs.event_id {
                // G3: eventId is emitted as a typed Int so the Ingestion
                // backend writes a Bond BT_INT64 column (matching mdsd's
                // behavior) instead of BT_STRING.
                attributes.push((Cow::Borrowed(ATTR_EVENT_ID), DecodedAttrValue::Int(eid)));
            }
            if let Some(time_str) = &cs.time {
                // TODO(perf): `parse_from_rfc3339` is on the Common Schema decode hot
                // path for every record with PartA.time. Keep it for correctness and
                // simplicity unless profiling shows it is a material throughput cost; if
                // it is, replace it with a fixed-format parser for the exporter's known
                // timestamp layout.
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
                    time_override_nano = dt.timestamp_nanos_opt();
                }
            }
            flags = cs.trace_flags;
            trace_id_typed = cs.trace_id;
            if let Some(tid_str) = cs.trace_id_text {
                attributes.push((Cow::Borrowed(ATTR_TRACE_ID), DecodedAttrValue::Str(tid_str)));
            }
            span_id_typed = cs.span_id;
            if let Some(sid_str) = cs.span_id_text {
                attributes.push((Cow::Borrowed(ATTR_SPAN_ID), DecodedAttrValue::Str(sid_str)));
            }
            if let Some(service_name) = cs.service_name {
                attributes.push((
                    Cow::Borrowed(ATTR_SERVICE_NAME),
                    DecodedAttrValue::Str(service_name),
                ));
            }
            if let Some(service_instance_id) = cs.service_instance_id {
                attributes.push((
                    Cow::Borrowed(ATTR_SERVICE_INSTANCE_ID),
                    DecodedAttrValue::Str(service_instance_id),
                ));
            }
            // G2: PartC attrs keep their source typed value (int/double/bool/str).
            for (k, v) in cs.part_c_attrs {
                attributes.push((Cow::Owned(k), v));
            }
            // Fall back to EventHeader level if PartB had no severity
            if severity_number.is_none() {
                let fallback_number = fallback_severity.map(|(number, _)| number);
                let fallback_text = fallback_severity.map(|(_, text)| Cow::Borrowed(text));
                severity_number = fallback_number;
                severity_text = severity_text.or(fallback_text);
            }
            cs_failed = false;
        } else {
            // Fallback: emit base64 body and map severity from EventHeader level
            body = BASE64_STANDARD.encode(&value.payload);
            event_name = Some("Log".to_owned());
            severity_number = fallback_severity.map(|(number, _)| number);
            severity_text = fallback_severity.map(|(_, text)| Cow::Borrowed(text));
            cs_failed = true;
        }

        (
            Self {
                time_unix_nano: time_override_nano.unwrap_or_else(|| {
                    i64::try_from(value.timestamp_unix_nano).unwrap_or(i64::MAX)
                }),
                body,
                event_name,
                severity_number,
                severity_text,
                flags,
                trace_id: trace_id_typed,
                span_id: span_id_typed,
                attributes,
            },
            cs_failed,
        )
    }
}

#[derive(Debug, Default)]
struct TracepointIdentity {
    level: Option<u8>,
}

impl TracepointIdentity {
    fn parse(tracepoint: &str) -> Self {
        let Some((_, name)) = tracepoint.split_once(':') else {
            return Self::default();
        };
        let Some((_, suffix)) = name.rsplit_once("_L") else {
            return Self::default();
        };
        let Some((level, _)) = suffix.split_once('K') else {
            return Self::default();
        };
        Self {
            level: level.parse::<u8>().ok(),
        }
    }
}

fn common_schema_severity_from_eventheader_level(level: u8) -> Option<(i32, &'static str)> {
    match level {
        1 | 2 => Some((17, "ERROR")),
        3 => Some((13, "WARN")),
        4 => Some((9, "INFO")),
        5 => Some((5, "DEBUG")),
        _ => None,
    }
}

pub(super) fn severity_fallback_from_tracepoint(tracepoint: &str) -> Option<(i32, &'static str)> {
    TracepointIdentity::parse(tracepoint)
        .level
        .and_then(common_schema_severity_from_eventheader_level)
}

/// Decoded Common Schema OTel Logs record extracted from EventHeader binary payload.
///
/// TODO(perf): This intermediate owned decode struct is intentionally kept for now because it
/// preserves a clean, pure `decode_cs_otel_logs()` contract and is the direct assertion surface
/// for the decoder unit tests. The remaining overhead from this layer is modest:
/// - one transient `part_c_attrs` Vec buffer per successful CS decode.
///
/// If profiling shows this layer is materially hot, revisit whether PartC attributes can be
/// written directly into the final attribute sink while preserving rollback on decode failure.
#[derive(Debug, Default)]
struct CsDecodedLog {
    event_name: String,
    time: Option<String>,
    trace_flags: Option<u32>,
    trace_id: Option<[u8; 16]>,
    trace_id_text: Option<String>,
    span_id: Option<[u8; 8]>,
    span_id_text: Option<String>,
    body: Option<String>,
    severity_number: Option<i32>,
    severity_text: Option<String>,
    part_b_name: Option<String>,
    event_id: Option<i64>,
    service_name: Option<String>,
    service_instance_id: Option<String>,
    part_c_attrs: Vec<(String, DecodedAttrValue)>,
}

/// Decode a Common Schema OTel Logs EventHeader binary payload.
/// Returns `None` if the payload cannot be parsed, `__csver__` != 0x400,
/// or PartB is missing / `_typeName` != "Log".
fn decode_cs_otel_logs(tracepoint: &str, payload: &[u8]) -> Option<CsDecodedLog> {
    let mut context = EventHeaderEnumeratorContext::new();
    let mut en = context
        .enumerate_with_name_and_data(
            tracepoint,
            payload,
            EventHeaderEnumeratorContext::MOVE_NEXT_LIMIT_DEFAULT,
        )
        .ok()?;

    let event_name = std::str::from_utf8(en.event_info().name_bytes())
        .unwrap_or("Log")
        .to_owned();

    // First field must be __csver__
    if !en.move_next() {
        return None;
    }
    if en.state() != EventHeaderEnumeratorState::Value {
        return None;
    }
    let csver_item = en.item_info();
    if csver_item.name_bytes() != b"__csver__" {
        return None;
    }
    let csver = item_as_u32(&csver_item)?;
    if csver != 0x400 {
        return None;
    }

    let mut cs = CsDecodedLog {
        event_name,
        ..Default::default()
    };
    let mut found_part_b = false;
    let mut part_a_done = false;
    let mut part_b_done = false;
    let mut part_c_done = false;

    // Walk top-level struct fields (PartA, PartC, PartB)
    while en.move_next() {
        match en.state() {
            EventHeaderEnumeratorState::StructBegin => {
                let item = en.item_info();
                let name = item.name_bytes();
                if name == b"PartA" {
                    if part_a_done {
                        return None;
                    }
                    if !walk_part_a(&mut en, &mut cs) {
                        return None;
                    }
                    part_a_done = true;
                } else if name == b"PartB" {
                    if part_b_done {
                        return None;
                    }
                    match walk_part_b(&mut en, &mut cs) {
                        None => return None,
                        Some(ok) => {
                            if ok {
                                found_part_b = true;
                            }
                            part_b_done = true;
                        }
                    }
                } else if name == b"PartC" {
                    if part_c_done {
                        return None;
                    }
                    if !walk_part_c(&mut en, &mut cs) {
                        return None;
                    }
                    part_c_done = true;
                } else {
                    // Unknown top-level struct → invalid schema
                    return None;
                }
            }
            EventHeaderEnumeratorState::Error => return None,
            EventHeaderEnumeratorState::AfterLastItem => break,
            _ => {}
        }
    }

    if en.last_error() != EventHeaderEnumeratorError::Success {
        return None;
    }

    if !found_part_b {
        return None;
    }
    Some(cs)
}

/// Walk PartA fields consuming until StructEnd.
/// Returns `true` on clean StructEnd, `false` on error / unexpected state.
fn walk_part_a(
    en: &mut tracepoint_decode::EventHeaderEnumerator<'_, '_, '_>,
    cs: &mut CsDecodedLog,
) -> bool {
    loop {
        if !en.move_next() {
            return false;
        }
        match en.state() {
            EventHeaderEnumeratorState::StructEnd => return true,
            EventHeaderEnumeratorState::Value => {
                let item = en.item_info();
                let name = item.name_bytes();
                if name == b"time" {
                    cs.time = item_as_string(&item);
                } else if name == b"name" {
                    // PartA name override (env_name in canonical schema)
                    if let Some(s) = item_as_string(&item) {
                        if !s.is_empty() {
                            cs.event_name = s;
                        }
                    }
                } else if name == b"ext_dt_traceId" {
                    let value = item.value().bytes();
                    if let Some(bytes) = parse_hex_bytes_ascii::<16>(value) {
                        cs.trace_id = Some(bytes);
                    } else {
                        cs.trace_id_text = item_as_string(&item);
                    }
                } else if name == b"ext_dt_spanId" {
                    let value = item.value().bytes();
                    if let Some(bytes) = parse_hex_bytes_ascii::<8>(value) {
                        cs.span_id = Some(bytes);
                    } else {
                        cs.span_id_text = item_as_string(&item);
                    }
                } else if name == b"ext_dt_traceFlags" {
                    cs.trace_flags = item_as_u32(&item);
                } else if name == b"ext_cloud_role" {
                    cs.service_name = item_as_string(&item).filter(|s| !s.is_empty());
                } else if name == b"ext_cloud_roleInstance" {
                    cs.service_instance_id = item_as_string(&item).filter(|s| !s.is_empty());
                }
            }
            EventHeaderEnumeratorState::Error => return false,
            EventHeaderEnumeratorState::AfterLastItem => return false,
            _ => return false,
        }
    }
}

/// Walk PartB fields consuming until StructEnd.
/// Returns `Some(type_name_ok)` on clean StructEnd, `None` on error /
/// unexpected state / unknown field.
fn walk_part_b(
    en: &mut tracepoint_decode::EventHeaderEnumerator<'_, '_, '_>,
    cs: &mut CsDecodedLog,
) -> Option<bool> {
    let mut type_name_ok = false;
    loop {
        if !en.move_next() {
            return None;
        }
        match en.state() {
            EventHeaderEnumeratorState::StructEnd => return Some(type_name_ok),
            EventHeaderEnumeratorState::Value => {
                let item = en.item_info();
                let name = item.name_bytes();
                if name == b"_typeName" {
                    if item_as_string(&item).as_deref() == Some("Log") {
                        type_name_ok = true;
                    }
                } else if name == b"body" {
                    cs.body = item_as_any_scalar_string(&item);
                } else if name == b"severityNumber" {
                    cs.severity_number = item_as_i32(&item);
                } else if name == b"severityText" {
                    cs.severity_text = item_as_string(&item);
                } else if name == b"name" {
                    cs.part_b_name = item_as_string(&item);
                } else if name == b"eventId" {
                    cs.event_id = item_as_i64(&item);
                } else {
                    // Unknown PartB field → invalid schema
                    return None;
                }
            }
            EventHeaderEnumeratorState::Error => return None,
            EventHeaderEnumeratorState::AfterLastItem => return None,
            _ => return None,
        }
    }
}

/// Walk PartC fields consuming until StructEnd.
/// Returns `true` on clean StructEnd, `false` on error / nested struct /
/// unexpected state.
fn walk_part_c(
    en: &mut tracepoint_decode::EventHeaderEnumerator<'_, '_, '_>,
    cs: &mut CsDecodedLog,
) -> bool {
    loop {
        if !en.move_next() {
            return false;
        }
        match en.state() {
            EventHeaderEnumeratorState::StructEnd => return true,
            EventHeaderEnumeratorState::Value => {
                let item = en.item_info();
                let name_bytes = item.name_bytes();
                let name = match std::str::from_utf8(name_bytes) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if PART_A_RESERVED.contains(&name) || PART_B_LOG_FIELDS.contains(&name) {
                    continue;
                }
                if let Some(val) = item_as_any_scalar_value(&item) {
                    cs.part_c_attrs.push((name.to_owned(), val));
                }
            }
            // Nested struct in PartC → reject (invalid schema)
            EventHeaderEnumeratorState::StructBegin => return false,
            EventHeaderEnumeratorState::Error => return false,
            EventHeaderEnumeratorState::AfterLastItem => return false,
            _ => return false,
        }
    }
}

/// Extract a UTF-8 string value from a StringLength16Char8-encoded field.
fn item_as_string(item: &EventHeaderItemInfo<'_>) -> Option<String> {
    let enc = item.metadata().encoding().without_flags().as_int();
    let fmt = item.metadata().format().as_int();
    if enc == ENC_STRING_LENGTH16_CHAR8 && (fmt == FMT_DEFAULT || fmt == FMT_STRING_UTF) {
        std::str::from_utf8(item.value().bytes())
            .ok()
            .map(str::to_owned)
    } else {
        None
    }
}

/// Extract a u32 value from Value32 or Value16 encoding (for __csver__ validation).
fn item_as_u32(item: &EventHeaderItemInfo<'_>) -> Option<u32> {
    match item.metadata().encoding().without_flags().as_int() {
        enc if enc == ENC_VALUE8 => Some(u32::from(item.value().to_u8(0))),
        enc if enc == ENC_VALUE32 => Some(item.value().to_u32(0)),
        enc if enc == ENC_VALUE16 => Some(u32::from(item.value().to_u16(0))),
        _ => None,
    }
}

/// Extract a signed integer from Value16 or Value32 encoding.
fn item_as_i32(item: &EventHeaderItemInfo<'_>) -> Option<i32> {
    match item.metadata().encoding().without_flags().as_int() {
        enc if enc == ENC_VALUE8 => Some(i32::from(item.value().to_i8(0))),
        enc if enc == ENC_VALUE16 => Some(i32::from(item.value().to_i16(0))),
        enc if enc == ENC_VALUE32 => Some(item.value().to_i32(0)),
        _ => None,
    }
}

/// Extract a signed 64-bit integer from Value64 encoding.
fn item_as_i64(item: &EventHeaderItemInfo<'_>) -> Option<i64> {
    match item.metadata().encoding().without_flags().as_int() {
        enc if enc == ENC_VALUE64 => Some(item.value().to_i64(0)),
        enc if enc == ENC_VALUE32 => Some(i64::from(item.value().to_i32(0))),
        enc if enc == ENC_VALUE16 => Some(i64::from(item.value().to_i16(0))),
        _ => None,
    }
}

/// Convert any scalar EventHeader field to a string representation.
/// Handles: strings, signed/unsigned integers, booleans, and floats.
fn item_as_any_scalar_string(item: &EventHeaderItemInfo<'_>) -> Option<String> {
    let enc = item.metadata().encoding().without_flags().as_int();
    let fmt = item.metadata().format().as_int();
    match enc {
        e if e == ENC_STRING_LENGTH16_CHAR8 => std::str::from_utf8(item.value().bytes())
            .ok()
            .map(str::to_owned),
        e if e == ENC_VALUE8 => {
            if fmt == FMT_BOOLEAN {
                Some((item.value().to_u8(0) != 0).to_string())
            } else if fmt == FMT_SIGNED_INT {
                Some((item.value().to_u8(0) as i8).to_string())
            } else {
                Some(item.value().to_u8(0).to_string())
            }
        }
        e if e == ENC_VALUE16 => {
            if fmt == FMT_SIGNED_INT {
                Some(item.value().to_i16(0).to_string())
            } else {
                Some(item.value().to_u16(0).to_string())
            }
        }
        e if e == ENC_VALUE32 => {
            if fmt == FMT_FLOAT {
                Some(f64::from(f32::from_bits(item.value().to_u32(0))).to_string())
            } else if fmt == FMT_BOOLEAN {
                Some((item.value().to_u32(0) != 0).to_string())
            } else if fmt == FMT_SIGNED_INT {
                Some(item.value().to_i32(0).to_string())
            } else {
                Some(item.value().to_u32(0).to_string())
            }
        }
        e if e == ENC_VALUE64 => {
            if fmt == FMT_FLOAT {
                Some(f64::from_bits(item.value().to_u64(0)).to_string())
            } else if fmt == FMT_SIGNED_INT {
                Some(item.value().to_i64(0).to_string())
            } else {
                Some(item.value().to_u64(0).to_string())
            }
        }
        _ => None,
    }
}

/// Convert any scalar EventHeader field to a typed `DecodedAttrValue`.
///
/// Used for PartC attributes so the receiver preserves the source field's
/// type through OTLP (str/int/bool/double) rather than stringifying every
/// value. The ingestion path then emits the corresponding Bond `BT_*` type.
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
                // Unsigned 64: preserve the integer column and clamp values
                // that do not fit OTLP/Bond's signed integer representation.
                Some(DecodedAttrValue::Int(
                    i64::try_from(item.value().to_u64(0)).unwrap_or(i64::MAX),
                ))
            }
        }
        _ => None,
    }
}

/// Parse ASCII hex bytes of exactly `N*2` characters into a `[u8; N]`.
/// Returns `None` if the length or any character is invalid.
fn parse_hex_bytes_ascii<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    if bytes.len() != N * 2 {
        return None;
    }
    let mut result = [0u8; N];
    for (i, pair) in bytes.chunks(2).enumerate() {
        let hi = hex_nibble(pair[0])?;
        let lo = hex_nibble(pair[1])?;
        result[i] = (hi << 4) | lo;
    }
    Some(result)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_record(
        tracepoint: &str,
        payload: Vec<u8>,
        timestamp_unix_nano: u64,
    ) -> (DecodedUsereventsRecord, bool) {
        DecodedUsereventsRecord::from_raw(
            tracepoint,
            RawUsereventsRecord {
                subscription_index: 0,
                timestamp_unix_nano,
                payload,
            },
            &FormatConfig::CommonSchemaOtelLogs,
            severity_fallback_from_tracepoint(tracepoint),
        )
    }

    #[test]
    fn common_schema_mode_maps_severity_from_tracepoint_level() {
        // Payload that is not a valid EventHeader → fallback path
        let (decoded, cs_failed) = decode_record("user_events:myprovider_L2K1", vec![0], 42);

        assert_eq!(decoded.severity_number, Some(17));
        assert_eq!(decoded.severity_text.as_deref(), Some("ERROR"));
        assert!(cs_failed);
        assert!(decoded.attributes.is_empty());
    }

    #[test]
    fn common_schema_mode_invalid_payload_falls_back_to_base64() {
        // Plain-text payload is not a valid EventHeader; decoder falls back
        let (decoded, cs_failed) = decode_record(
            "user_events:myprovider_L2K1",
            br#"name=my-event-name;message=This is a test message"#.to_vec(),
            42,
        );

        // Fallback: event_name is "Log", body is base64. No transport
        // identity/debug attributes are emitted downstream.
        assert_eq!(decoded.event_name.as_deref(), Some("Log"));
        assert!(cs_failed);
        assert!(decoded.attributes.is_empty());
    }

    // ── EventHeader binary test helpers ──────────────────────────────────────

    /// Build the fixed 8-byte EventHeader prefix.
    /// flags=0x07 (Pointer64|LittleEndian|Extension), version=0, id=0, tag=0,
    /// opcode=0 (Info), level as supplied.
    fn build_eventheader_header(level: u8) -> Vec<u8> {
        vec![0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, level]
    }

    /// Append a value field to the meta section.
    /// `encoding`: raw FieldEncoding value.  If `encoding & 0x80 != 0`, a
    /// format byte must be supplied and the chain flag is already set.
    fn add_value_field_meta(meta: &mut Vec<u8>, name: &str, encoding: u8, format: u8) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0x00); // NUL terminator
        meta.push(encoding | 0x80); // ChainFlag always set so we can write format
        meta.push(format);
    }

    /// Append a string (StringLength16Char8 = 10, Default format = 0) field.
    fn add_string_field_meta(meta: &mut Vec<u8>, name: &str) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0x00);
        // encoding=10, no chain flag → no format byte
        meta.push(ENC_STRING_LENGTH16_CHAR8);
    }

    /// Append a struct marker to the meta section.
    /// encoding=0x81 (0x80|Struct=1), format=field_count.
    fn add_struct_meta(meta: &mut Vec<u8>, name: &str, field_count: u8) {
        meta.extend_from_slice(name.as_bytes());
        meta.push(0x00);
        meta.push(0x80 | ENC_STRUCT); // ChainFlag | Struct
        meta.push(field_count);
    }

    /// Append a string value to the data section.
    fn add_string_data(data: &mut Vec<u8>, value: &str) {
        let len = value.len() as u16;
        data.extend_from_slice(&len.to_le_bytes());
        data.extend_from_slice(value.as_bytes());
    }

    /// Append an i16 value to the data section.
    fn add_i16_data(data: &mut Vec<u8>, value: i16) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    /// Append a u8 value to the data section.
    fn add_u8_data(data: &mut Vec<u8>, value: u8) {
        data.push(value);
    }

    /// Append a u16 value to the data section.
    fn add_u16_data(data: &mut Vec<u8>, value: u16) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    /// Append a u32 value to the data section.
    fn add_u32_data(data: &mut Vec<u8>, value: u32) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    /// Append an i64 value to the data section.
    fn add_i64_data(data: &mut Vec<u8>, value: i64) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    /// Append an f64 value to the data section.
    fn add_f64_data(data: &mut Vec<u8>, value: f64) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    /// Combine header + extension header + meta + data into a full payload.
    fn build_full_payload(level: u8, event_name: &str, meta_fields: &[u8], data: &[u8]) -> Vec<u8> {
        let mut payload = build_eventheader_header(level);

        // Build meta section: event_name + NUL + field meta
        let mut meta = Vec::new();
        meta.extend_from_slice(event_name.as_bytes());
        meta.push(0x00);
        meta.extend_from_slice(meta_fields);

        // Extension header: size (u16 LE), kind = 0x0001 (Metadata, no chain)
        let ext_size = meta.len() as u16;
        payload.extend_from_slice(&ext_size.to_le_bytes());
        payload.extend_from_slice(&0x0001u16.to_le_bytes());
        payload.extend_from_slice(&meta);
        payload.extend_from_slice(data);
        payload
    }

    /// Build a minimal valid CS OTel Logs payload with PartB only.
    ///
    /// Fields: __csver__(u32), PartB{_typeName, body, severityNumber(i16), severityText}
    fn build_cs_log_payload_part_b_only(
        event_name: &str,
        body: &str,
        severity_number: i16,
        severity_text: &str,
        level: u8,
    ) -> (String, Vec<u8>) {
        let mut meta = Vec::new();
        let mut data = Vec::new();

        // __csver__ as Value32 UnsignedInt (0x80|4, format=1)
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);

        // PartB struct with 4 fields: _typeName, body, severityNumber, severityText
        add_struct_meta(&mut meta, "PartB", 4);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, body);
        add_value_field_meta(&mut meta, "severityNumber", ENC_VALUE16, FMT_SIGNED_INT);
        add_i16_data(&mut data, severity_number);
        add_string_field_meta(&mut meta, "severityText");
        add_string_data(&mut data, severity_text);

        let tracepoint = format!("user_events:myprovider_L{level}K1");
        let payload = build_full_payload(level, event_name, &meta, &data);
        (tracepoint, payload)
    }

    /// Build a CS log payload with PartA and PartB.
    fn build_cs_log_payload_with_part_a(
        event_name: &str,
        time: &str,
        trace_id: &str,
        span_id: &str,
        body: &str,
        severity_number: i16,
        level: u8,
    ) -> (String, Vec<u8>) {
        let mut meta = Vec::new();
        let mut data = Vec::new();

        // __csver__
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);

        // PartA with 3 fields: time, ext_dt_traceId, ext_dt_spanId
        add_struct_meta(&mut meta, "PartA", 3);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, time);
        add_string_field_meta(&mut meta, "ext_dt_traceId");
        add_string_data(&mut data, trace_id);
        add_string_field_meta(&mut meta, "ext_dt_spanId");
        add_string_data(&mut data, span_id);

        // PartB with 3 fields: _typeName, body, severityNumber
        add_struct_meta(&mut meta, "PartB", 3);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, body);
        add_value_field_meta(&mut meta, "severityNumber", ENC_VALUE16, FMT_SIGNED_INT);
        add_i16_data(&mut data, severity_number);

        let tracepoint = format!("user_events:myprovider_L{level}K1");
        let payload = build_full_payload(level, event_name, &meta, &data);
        (tracepoint, payload)
    }

    /// Build a CS log payload with PartA, PartC and PartB.
    fn build_cs_log_payload_with_part_c(
        event_name: &str,
        body: &str,
        part_c_attrs: &[(&str, &str)],
        level: u8,
    ) -> (String, Vec<u8>) {
        let mut meta = Vec::new();
        let mut data = Vec::new();

        // __csver__
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);

        // PartC
        add_struct_meta(&mut meta, "PartC", part_c_attrs.len() as u8);
        for (k, v) in part_c_attrs {
            add_string_field_meta(&mut meta, k);
            add_string_data(&mut data, v);
        }

        // PartB with 2 fields: _typeName, body
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, body);

        let tracepoint = format!("user_events:myprovider_L{level}K1");
        let payload = build_full_payload(level, event_name, &meta, &data);
        (tracepoint, payload)
    }

    // ── CS decode unit tests ──────────────────────────────────────────────────

    #[test]
    fn cs_decode_extracts_event_name_from_eventheader() {
        let (tracepoint, payload) =
            build_cs_log_payload_part_b_only("MyLogEvent", "hello", 9, "INFO", 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.event_name, "MyLogEvent");
    }

    #[test]
    fn cs_decode_extracts_severity_from_part_b() {
        let (tracepoint, payload) =
            build_cs_log_payload_part_b_only("Log", "hello", 17, "ERROR", 2);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.severity_number, Some(17));
        assert_eq!(cs.severity_text.as_deref(), Some("ERROR"));
    }

    #[test]
    fn cs_decode_extracts_body_from_part_b() {
        let (tracepoint, payload) =
            build_cs_log_payload_part_b_only("Log", "This is the log body", 9, "INFO", 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.body.as_deref(), Some("This is the log body"));
    }

    #[test]
    fn cs_decode_part_c_attrs_are_flat() {
        let attrs = [("my_key", "my_value"), ("another", "42")];
        let (tracepoint, payload) = build_cs_log_payload_with_part_c("Log", "body", &attrs, 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert!(
            cs.part_c_attrs
                .iter()
                .any(|(k, v)| k == "my_key" && v == "my_value")
        );
        assert!(
            cs.part_c_attrs
                .iter()
                .any(|(k, v)| k == "another" && v == "42")
        );
        // Keys must NOT have a "cs." prefix
        assert!(!cs.part_c_attrs.iter().any(|(k, _)| k.starts_with("cs.")));
    }

    #[test]
    fn cs_decode_part_a_trace_context() {
        let (tracepoint, payload) = build_cs_log_payload_with_part_a(
            "Log",
            "2024-01-01T00:00:00Z",
            "0102030405060708090a0b0c0d0e0f10",
            "a1b2c3d4e5f60718",
            "body",
            9,
            4,
        );
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(
            cs.trace_id,
            Some([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
                0x0f, 0x10,
            ])
        );
        assert_eq!(
            cs.span_id,
            Some([0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18])
        );
    }

    #[test]
    fn cs_decode_part_a_time_override() {
        let (tracepoint, payload) =
            build_cs_log_payload_with_part_a("Log", "2024-06-15T12:00:00Z", "", "", "body", 9, 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.time.as_deref(), Some("2024-06-15T12:00:00Z"));
    }

    #[test]
    fn cs_decode_invalid_csver_returns_none() {
        // Build payload with __csver__ = 0x300 (wrong version)
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x300);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_missing_part_b_returns_none() {
        // Build payload with __csver__ but no PartB
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_from_raw_signals_no_failure_on_success() {
        let (tracepoint, payload) =
            build_cs_log_payload_part_b_only("MyEvent", "log body", 9, "INFO", 4);
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1_000_000_000);
        assert!(!cs_failed);
        assert_eq!(decoded.body, "log body");
        assert_eq!(decoded.event_name.as_deref(), Some("MyEvent"));
        assert_eq!(decoded.severity_number, Some(9));
    }

    #[test]
    fn cs_decode_from_raw_signals_failure_on_fallback() {
        let (decoded, cs_failed) = decode_record(
            "user_events:myprovider_L4K1",
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            42,
        );
        assert!(cs_failed);
        assert_eq!(decoded.event_name.as_deref(), Some("Log"));
        assert!(decoded.attributes.is_empty());
    }

    // ── Finding 1: enumerator error check ──────────────────────────────────────

    #[test]
    fn cs_decode_truncated_payload_returns_none() {
        let (tracepoint, mut payload) =
            build_cs_log_payload_part_b_only("Log", "body", 9, "INFO", 4);
        // Truncate data section
        let truncated_len = payload.len().saturating_sub(4);
        payload.truncate(truncated_len);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    // ── Finding 2: schema validation ────────────────────────────────────────────

    #[test]
    fn cs_decode_unknown_top_level_struct_returns_none() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartD", 1);
        add_string_field_meta(&mut meta, "something");
        add_string_data(&mut data, "value");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_duplicate_part_b_returns_none() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "first");
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "second");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_unknown_part_b_field_returns_none() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 3);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "hello");
        add_string_field_meta(&mut meta, "foo");
        add_string_data(&mut data, "bar");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_nested_struct_in_part_c_returns_none() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartC", 2);
        add_string_field_meta(&mut meta, "attr1");
        add_string_data(&mut data, "value1");
        add_struct_meta(&mut meta, "nested", 1);
        add_string_field_meta(&mut meta, "inner");
        add_string_data(&mut data, "innerval");
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    // ── Finding 3: typed trace/span ────────────────────────────────────────────

    #[test]
    fn cs_decode_valid_trace_span_hex_populates_typed_fields() {
        let trace_hex = "0102030405060708090a0b0c0d0e0f10";
        let span_hex = "a1b2c3d4e5f60718";
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartA", 3);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "2024-01-01T00:00:00Z");
        add_string_field_meta(&mut meta, "ext_dt_traceId");
        add_string_data(&mut data, trace_hex);
        add_string_field_meta(&mut meta, "ext_dt_spanId");
        add_string_data(&mut data, span_hex);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body text");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "MyEvent", &meta, &data);
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1_000_000);
        assert!(!cs_failed);
        let expected_trace: [u8; 16] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10,
        ];
        let expected_span: [u8; 8] = [0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0xf6, 0x07, 0x18];
        assert_eq!(decoded.trace_id, Some(expected_trace));
        assert_eq!(decoded.span_id, Some(expected_span));
        assert!(!decoded.attributes.iter().any(|(k, _)| k == ATTR_TRACE_ID));
        assert!(!decoded.attributes.iter().any(|(k, _)| k == ATTR_SPAN_ID));
    }

    #[test]
    fn cs_decode_malformed_trace_span_hex_falls_back_to_string_attr() {
        let trace_hex = "aaaa1111bbbb2222";
        let span_hex = "cccc3333";
        let (tracepoint, payload) = build_cs_log_payload_with_part_a(
            "Log",
            "2024-01-01T00:00:00Z",
            trace_hex,
            span_hex,
            "body",
            9,
            4,
        );
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1_000_000);
        assert!(!cs_failed);
        assert!(decoded.trace_id.is_none());
        assert!(decoded.span_id.is_none());
        assert!(
            decoded
                .attributes
                .iter()
                .any(|(k, v)| k == ATTR_TRACE_ID && v == trace_hex)
        );
        assert!(
            decoded
                .attributes
                .iter()
                .any(|(k, v)| k == ATTR_SPAN_ID && v == span_hex)
        );
    }

    #[test]
    fn cs_decode_part_a_cloud_role_fields_become_service_attrs() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);

        add_struct_meta(&mut meta, "PartA", 3);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "2024-01-01T00:00:00Z");
        add_string_field_meta(&mut meta, "ext_cloud_role");
        add_string_data(&mut data, "checkout-service");
        add_string_field_meta(&mut meta, "ext_cloud_roleInstance");
        add_string_data(&mut data, "instance-42");

        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "content");

        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "HeaderName", &meta, &data);
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1_000_000);

        assert!(!cs_failed);
        assert!(
            decoded
                .attributes
                .iter()
                .any(|(k, v)| k == ATTR_SERVICE_NAME && v == "checkout-service")
        );
        assert!(
            decoded
                .attributes
                .iter()
                .any(|(k, v)| k == ATTR_SERVICE_INSTANCE_ID && v == "instance-42")
        );
    }

    // ── Finding 4: PartA.name override ─────────────────────────────────────────

    #[test]
    fn cs_decode_part_a_name_overrides_event_name() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartA", 2);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "2024-01-01T00:00:00Z");
        add_string_field_meta(&mut meta, "name");
        add_string_data(&mut data, "OverriddenName");
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "content");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "HeaderName", &meta, &data);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.event_name, "OverriddenName");
    }

    // ── Finding 5: CS decode failure metric signal ─────────────────────────────

    #[test]
    fn cs_decode_failure_signals_metric_increment() {
        let (decoded, cs_failed) = decode_record(
            "user_events:myprovider_L4K1",
            vec![0xDE, 0xAD, 0xBE, 0xEF],
            1_000_000,
        );
        assert!(cs_failed, "CS decode failure should be signaled");
        assert_eq!(decoded.event_name.as_deref(), Some("Log"));
    }

    // ── Finding 9: severity fallback from EventHeader level ────────────────────

    #[test]
    fn cs_decode_missing_severity_number_falls_back_to_eventheader_level() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "no sev");
        let tracepoint = "user_events:myprovider_L2K1".to_owned();
        let payload = build_full_payload(2, "Log", &meta, &data);
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1_000_000);
        assert!(!cs_failed);
        assert_eq!(
            decoded.severity_number,
            Some(17),
            "Should fall back to ERROR (17) from level 2"
        );
        assert_eq!(decoded.severity_text.as_deref(), Some("ERROR"));
    }

    // ── Finding 10: malformed RFC3339 time uses perf timestamp ─────────────────

    #[test]
    fn cs_decode_malformed_rfc3339_time_uses_perf_timestamp() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_DEFAULT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartA", 1);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "not-a-valid-timestamp");
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body text");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        let perf_ts: u64 = 9_999_999_999;
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, perf_ts);
        assert!(!cs_failed);
        assert_eq!(
            decoded.time_unix_nano, perf_ts as i64,
            "Malformed RFC3339 time should fall back to perf sample timestamp"
        );
    }

    #[test]
    fn cs_decode_part_a_trace_flags_populates_log_flags() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartA", 2);
        add_string_field_meta(&mut meta, "time");
        add_string_data(&mut data, "2024-01-01T00:00:00Z");
        add_value_field_meta(&mut meta, "ext_dt_traceFlags", ENC_VALUE8, FMT_UNSIGNED_INT);
        add_u8_data(&mut data, 0x01);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body");
        let payload = build_full_payload(4, "Log", &meta, &data);
        let (decoded, cs_failed) = decode_record("user_events:myprovider_L4K1", payload, 1_000_000);
        assert!(!cs_failed);
        assert_eq!(decoded.flags, Some(1));
    }

    #[test]
    fn cs_decode_rejects_when_csver_not_first() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_string_field_meta(&mut meta, "before");
        add_string_data(&mut data, "x");
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_accepts_value16_csver() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE16, FMT_UNSIGNED_INT);
        add_u16_data(&mut data, 0x0400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_some());
    }

    #[test]
    fn cs_decode_accepts_exporter_csver_unsigned_format() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_some());
    }

    #[test]
    fn cs_decode_rejects_non_log_typename() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Span");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_rejects_part_b_without_typename() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body");
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        assert!(decode_cs_otel_logs(&tracepoint, &payload).is_none());
    }

    #[test]
    fn cs_decode_part_c_drops_part_b_name_collisions() {
        let attrs = [
            ("body", "collision"),
            ("severityText", "collision"),
            ("ok", "value"),
        ];
        let (tracepoint, payload) = build_cs_log_payload_with_part_c("Log", "body", &attrs, 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert!(
            cs.part_c_attrs
                .iter()
                .any(|(k, v)| k == "ok" && v == "value")
        );
        assert!(!cs.part_c_attrs.iter().any(|(k, _)| k == "body"));
        assert!(!cs.part_c_attrs.iter().any(|(k, _)| k == "severityText"));
    }

    #[test]
    fn cs_decode_part_c_drops_part_a_name_collisions() {
        let attrs = [
            ("env_time", "collision"),
            ("env_dt_traceId", "collision"),
            ("ok", "value"),
        ];
        let (tracepoint, payload) = build_cs_log_payload_with_part_c("Log", "body", &attrs, 4);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert!(
            cs.part_c_attrs
                .iter()
                .any(|(k, v)| k == "ok" && v == "value")
        );
        assert!(!cs.part_c_attrs.iter().any(|(k, _)| k == "env_time"));
        assert!(!cs.part_c_attrs.iter().any(|(k, _)| k == "env_dt_traceId"));
    }

    #[test]
    fn cs_decode_stringifies_integer_body() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_value_field_meta(&mut meta, "body", ENC_VALUE64, FMT_SIGNED_INT);
        add_i64_data(&mut data, -42);
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.body.as_deref(), Some("-42"));
    }

    #[test]
    fn cs_decode_stringifies_boolean_body() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_value_field_meta(&mut meta, "body", ENC_VALUE8, FMT_BOOLEAN);
        add_u8_data(&mut data, 1);
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.body.as_deref(), Some("true"));
    }

    #[test]
    fn cs_decode_stringifies_double_body() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_value_field_meta(&mut meta, "body", ENC_VALUE64, FMT_FLOAT);
        add_f64_data(&mut data, 3.5);
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert_eq!(cs.body.as_deref(), Some("3.5"));
    }

    #[test]
    fn cs_decode_extracts_event_id_attribute() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 3);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "body");
        add_string_data(&mut data, "body");
        add_value_field_meta(&mut meta, "eventId", ENC_VALUE32, FMT_SIGNED_INT);
        add_u32_data(&mut data, 20);
        let payload = build_full_payload(4, "Log", &meta, &data);
        let (decoded, cs_failed) = decode_record("user_events:myprovider_L4K1", payload, 1_000_000);
        assert!(!cs_failed);
        assert!(
            decoded
                .attributes
                .iter()
                .any(|(k, v)| k == ATTR_EVENT_ID && matches!(v, DecodedAttrValue::Int(20)))
        );
    }

    #[test]
    fn cs_decode_part_b_name_becomes_event_name() {
        // G1: OTLP `event_name` should reflect PartB.name (the user-configured
        // event name), not EH.Name, so the Ingestion backend routes/writes it
        // as the Bond `name` field for this record type.
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "name");
        add_string_data(&mut data, "my-event-name");
        let payload = build_full_payload(4, "Log", &meta, &data);
        let cs_direct = decode_cs_otel_logs("user_events:myprovider_L4K1", &payload);
        assert!(cs_direct.is_some(), "direct CS decode should succeed");
        assert_eq!(
            cs_direct.unwrap().part_b_name.as_deref(),
            Some("my-event-name")
        );
        let (decoded, cs_failed) = decode_record("user_events:myprovider_L4K1", payload, 1_000_000);
        assert!(!cs_failed);
        assert_eq!(decoded.event_name.as_deref(), Some("my-event-name"));
        // PartB.name is surfaced only via the typed `event_name` column; no
        // `cs.part_b.name` or EH `event.name` inspection attribute is emitted.
        assert!(decoded.attributes.is_empty());
    }

    #[test]
    fn cs_decode_part_c_preserves_typed_values() {
        // G2: PartC int/bool/double fields should land on the record as the
        // corresponding typed DecodedAttrValue variants, so the ingestion path
        // emits BT_INT64/BT_BOOL/BT_DOUBLE (not BT_STRING).
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 1);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_struct_meta(&mut meta, "PartC", 4);
        add_value_field_meta(&mut meta, "int_field", ENC_VALUE32, FMT_SIGNED_INT);
        add_u32_data(&mut data, (-7_i32) as u32);
        add_value_field_meta(&mut meta, "bool_field", ENC_VALUE8, FMT_BOOLEAN);
        data.push(1);
        add_value_field_meta(&mut meta, "double_field", ENC_VALUE64, FMT_FLOAT);
        data.extend_from_slice(&(2.5_f64).to_le_bytes());
        add_string_field_meta(&mut meta, "str_field");
        add_string_data(&mut data, "hello");
        let payload = build_full_payload(4, "Log", &meta, &data);
        let (decoded, cs_failed) = decode_record("user_events:myprovider_L4K1", payload, 1_000_000);
        assert!(!cs_failed);
        let find = |key: &str| {
            decoded
                .attributes
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(find("int_field"), Some(DecodedAttrValue::Int(-7)));
        assert_eq!(find("bool_field"), Some(DecodedAttrValue::Bool(true)));
        assert_eq!(find("double_field"), Some(DecodedAttrValue::Double(2.5)));
        assert_eq!(
            find("str_field"),
            Some(DecodedAttrValue::Str("hello".to_owned()))
        );
    }

    #[test]
    fn cs_decode_rfc3339_time_with_offset_converts_to_utc_nanos() {
        let (tracepoint, payload) = build_cs_log_payload_with_part_a(
            "Log",
            "2024-06-15T12:00:00+05:30",
            "",
            "",
            "body",
            9,
            4,
        );
        let (decoded, cs_failed) = decode_record(&tracepoint, payload, 1);
        assert!(!cs_failed);
        let expected = chrono::DateTime::parse_from_rfc3339("2024-06-15T12:00:00+05:30")
            .expect("valid time")
            .timestamp_nanos_opt()
            .expect("representable nanos");
        assert_eq!(decoded.time_unix_nano, expected);
    }

    #[test]
    fn cs_decode_invalid_utf8_string_field_is_dropped() {
        let mut meta = Vec::new();
        let mut data = Vec::new();
        add_value_field_meta(&mut meta, "__csver__", ENC_VALUE32, FMT_UNSIGNED_INT);
        add_u32_data(&mut data, 0x400);
        add_struct_meta(&mut meta, "PartB", 2);
        add_string_field_meta(&mut meta, "_typeName");
        add_string_data(&mut data, "Log");
        add_string_field_meta(&mut meta, "severityText");
        data.extend_from_slice(&(2u16).to_le_bytes());
        data.extend_from_slice(&[0xC3, 0x28]);
        let tracepoint = "user_events:myprovider_L4K1".to_owned();
        let payload = build_full_payload(4, "Log", &meta, &data);
        let cs = decode_cs_otel_logs(&tracepoint, &payload).expect("decode should succeed");
        assert!(cs.severity_text.is_none());
    }
}
