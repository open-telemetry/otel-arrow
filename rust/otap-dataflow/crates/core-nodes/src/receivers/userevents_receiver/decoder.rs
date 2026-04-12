// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Decoding helpers for Linux userevents samples.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::Value;
use tracepoint_decode::{EventHeaderEnumeratorContext, PerfConvertOptions};

use super::FormatConfig;
use super::session::RawUsereventsRecord;

const ATTR_MODE: &str = "linux.userevents.decode.mode";
const ATTR_PROVIDER: &str = "event.provider";
const ATTR_EVENT_NAME: &str = "event.name";
const ATTR_CS_VERSION: &str = "cs.__csver__";
const ATTR_CS_TYPE_NAME: &str = "cs.part_b._typeName";
const ATTR_CS_PART_B_NAME: &str = "cs.part_b.name";
const ATTR_CS_PART_B_BODY: &str = "cs.part_b.body";
const ATTR_LEVEL: &str = "eventheader.level";
const ATTR_KEYWORD: &str = "eventheader.keyword";
const ATTR_FLATTEN_PREFIX: &str = "linux.userevents.flatten_prefix";
const ATTR_CUSTOM_BODY_FIELD: &str = "linux.userevents.custom.body_field";
const ATTR_CUSTOM_EVENT_NAME_FIELD: &str = "linux.userevents.custom.event_name_field";
const ATTR_CUSTOM_SEVERITY_NUMBER_FIELD: &str = "linux.userevents.custom.severity_number_field";
const ATTR_CUSTOM_SEVERITY_TEXT_FIELD: &str = "linux.userevents.custom.severity_text_field";
const ATTR_CUSTOM_ATTRIBUTES_FROM: &str = "linux.userevents.custom.attributes_from";
const BODY_ENCODING_BASE64: &str = "base64";

/// A decoded userevents record ready for Arrow encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DecodedUsereventsRecord {
    /// The concrete tracefs tracepoint name, e.g. `user_events:MyProvider_L5K1f`.
    pub tracepoint: String,
    /// Event timestamp in Unix epoch nanoseconds.
    pub time_unix_nano: i64,
    /// CPU that produced the sample.
    pub cpu: u32,
    /// Process identifier carried by the perf sample.
    pub pid: i32,
    /// Thread identifier carried by the perf sample.
    pub tid: i32,
    /// Perf sample identifier used to bind events to subscriptions.
    pub sample_id: u64,
    /// Encoded log body.
    pub body: String,
    /// Optional promoted event name.
    pub event_name: Option<String>,
    /// Size of the decoded payload in bytes.
    pub payload_size: usize,
    /// Optional severity number.
    pub severity_number: Option<i32>,
    /// Optional severity text.
    pub severity_text: Option<&'static str>,
    /// Additional structured attributes.
    pub attributes: Vec<(String, String)>,
}

impl DecodedUsereventsRecord {
    pub(super) fn from_raw(value: RawUsereventsRecord, format: &FormatConfig) -> Self {
        let identity = TracepointIdentity::parse(&value.tracepoint);
        let fallback_body = BASE64_STANDARD.encode(&value.payload);
        let mut attributes = vec![(ATTR_MODE.to_owned(), format.name().to_owned())];
        let mut severity_number = None;
        let mut severity_text = None;
        let mut event_name = None;
        let mut body = fallback_body;

        if let Some(provider) = identity.provider.as_ref() {
            attributes.push((ATTR_PROVIDER.to_owned(), provider.clone()));
        }
        if let Some(level) = identity.level {
            attributes.push((ATTR_LEVEL.to_owned(), level.to_string()));
        }
        if let Some(keyword) = identity.keyword.as_ref() {
            attributes.push((ATTR_KEYWORD.to_owned(), keyword.clone()));
        }

        match format {
            FormatConfig::Raw => {}
            FormatConfig::CommonSchemaOtelLogs => {
                event_name = Some("Log".to_owned());
                attributes.push((ATTR_EVENT_NAME.to_owned(), "Log".to_owned()));
                attributes.push((ATTR_CS_VERSION.to_owned(), "1024".to_owned()));
                attributes.push((ATTR_CS_TYPE_NAME.to_owned(), "Log".to_owned()));
                let mapped = identity
                    .level
                    .and_then(common_schema_severity_from_eventheader_level);
                severity_number = mapped.map(|(number, _)| number);
                severity_text = mapped.map(|(_, text)| text);
                if let Some(decoded) = decode_eventheader_json(&value.tracepoint, &value.payload) {
                    if let Some(part_b) = decoded.get("PartB") {
                        if let Some(name) = get_json_string(part_b.get("name")) {
                            event_name = Some(name.clone());
                            attributes.push((ATTR_EVENT_NAME.to_owned(), name.clone()));
                            attributes.push((ATTR_CS_PART_B_NAME.to_owned(), name));
                        }
                        if let Some(text_body) = get_json_string(part_b.get("body")) {
                            attributes.push((ATTR_CS_PART_B_BODY.to_owned(), text_body.clone()));
                            body = text_body;
                        }
                        if let Some(number) = get_json_i32(part_b.get("severityNumber")) {
                            severity_number = Some(number);
                        }
                        if let Some(text) = get_json_string(part_b.get("severityText")) {
                            severity_text = Some(leak_string(text));
                        }
                    }

                    flatten_json("cs", &decoded, &mut attributes);
                } else if let Some((name, text_body)) =
                    try_extract_common_schema_body_fields(&value.payload)
                {
                    event_name = Some(name.clone());
                    attributes.push((ATTR_EVENT_NAME.to_owned(), name.clone()));
                    attributes.push((ATTR_CS_PART_B_NAME.to_owned(), name));
                    attributes.push((ATTR_CS_PART_B_BODY.to_owned(), text_body.clone()));
                    body = text_body;
                }
            }
            FormatConfig::EventheaderFlat { flatten_prefix } => {
                attributes.push((ATTR_FLATTEN_PREFIX.to_owned(), flatten_prefix.clone()));
                let mapped = identity
                    .level
                    .and_then(common_schema_severity_from_eventheader_level);
                severity_number = mapped.map(|(number, _)| number);
                severity_text = mapped.map(|(_, text)| text);
                if let Some(decoded) = decode_eventheader_json(&value.tracepoint, &value.payload) {
                    flatten_json(flatten_prefix, &decoded, &mut attributes);
                    if let Some(name) = get_json_string(decoded.get("name")) {
                        event_name = Some(name.clone());
                        attributes.push((ATTR_EVENT_NAME.to_owned(), name));
                    }
                }
            }
            FormatConfig::CustomEventheader {
                body_field,
                severity_number_field,
                severity_text_field,
                event_name_field,
                attributes_from,
            } => {
                if let Some(body_field) = body_field {
                    attributes.push((ATTR_CUSTOM_BODY_FIELD.to_owned(), body_field.clone()));
                }
                if let Some(event_name_field) = event_name_field {
                    attributes.push((
                        ATTR_CUSTOM_EVENT_NAME_FIELD.to_owned(),
                        event_name_field.clone(),
                    ));
                }
                if let Some(severity_number_field) = severity_number_field {
                    attributes.push((
                        ATTR_CUSTOM_SEVERITY_NUMBER_FIELD.to_owned(),
                        severity_number_field.clone(),
                    ));
                }
                if let Some(severity_text_field) = severity_text_field {
                    attributes.push((
                        ATTR_CUSTOM_SEVERITY_TEXT_FIELD.to_owned(),
                        severity_text_field.clone(),
                    ));
                }
                if !attributes_from.is_empty() {
                    attributes.push((
                        ATTR_CUSTOM_ATTRIBUTES_FROM.to_owned(),
                        attributes_from.join(","),
                    ));
                }
                let mapped = identity
                    .level
                    .and_then(common_schema_severity_from_eventheader_level);
                severity_number = mapped.map(|(number, _)| number);
                severity_text = mapped.map(|(_, text)| text);
                if let Some(decoded) = decode_eventheader_json(&value.tracepoint, &value.payload) {
                    if let Some(event_name_field) = event_name_field {
                        if let Some(name) = lookup_path_as_string(&decoded, event_name_field) {
                            event_name = Some(name.clone());
                            attributes.push((ATTR_EVENT_NAME.to_owned(), name));
                        }
                    }
                    if let Some(body_field) = body_field {
                        if let Some(decoded_body) = lookup_path_as_string(&decoded, body_field) {
                            body = decoded_body;
                        }
                    }
                    if let Some(field) = severity_number_field {
                        if let Some(number) = lookup_path_as_i32(&decoded, field) {
                            severity_number = Some(number);
                        }
                    }
                    if let Some(field) = severity_text_field {
                        if let Some(text) = lookup_path_as_string(&decoded, field) {
                            severity_text = Some(leak_string(text));
                        }
                    }
                    for path in attributes_from {
                        if let Some(prefix) = path.strip_suffix(".*") {
                            if let Some(value) = lookup_path(&decoded, prefix) {
                                flatten_json(prefix, value, &mut attributes);
                            }
                        } else if let Some(value) = lookup_path_as_string(&decoded, path) {
                            attributes.push((path.clone(), value));
                        }
                    }
                }
            }
        }

        Self {
            tracepoint: value.tracepoint,
            time_unix_nano: value.timestamp_unix_nano as i64,
            cpu: value.cpu,
            pid: value.pid,
            tid: value.tid,
            sample_id: value.sample_id,
            body,
            event_name,
            payload_size: value.payload_size,
            severity_number,
            severity_text,
            attributes,
        }
    }
}

impl FormatConfig {
    fn name(&self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::CommonSchemaOtelLogs => "common_schema_otel_logs",
            Self::EventheaderFlat { .. } => "eventheader_flat",
            Self::CustomEventheader { .. } => "custom_eventheader",
        }
    }
}

#[derive(Debug, Default)]
struct TracepointIdentity {
    provider: Option<String>,
    level: Option<u8>,
    keyword: Option<String>,
}

impl TracepointIdentity {
    fn parse(tracepoint: &str) -> Self {
        let Some((_, name)) = tracepoint.split_once(':') else {
            return Self::default();
        };
        let Some((provider, suffix)) = name.rsplit_once("_L") else {
            return Self {
                provider: Some(name.to_owned()),
                level: None,
                keyword: None,
            };
        };
        let Some((level, keyword)) = suffix.split_once('K') else {
            return Self {
                provider: Some(name.to_owned()),
                level: None,
                keyword: None,
            };
        };
        Self {
            provider: Some(provider.to_owned()),
            level: level.parse::<u8>().ok(),
            keyword: Some(keyword.to_owned()),
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

fn decode_eventheader_json(tracepoint: &str, payload: &[u8]) -> Option<Value> {
    let mut context = EventHeaderEnumeratorContext::new();
    let mut enumerator = context
        .enumerate_with_name_and_data(
            tracepoint,
            payload,
            EventHeaderEnumeratorContext::MOVE_NEXT_LIMIT_DEFAULT,
        )
        .ok()?;
    let mut json = String::from("{");
    if !enumerator.write_json_item_and_move_next_sibling(
        &mut json,
        false,
        PerfConvertOptions::Default,
    ).ok()? {
        return None;
    }
    json.push('}');
    serde_json::from_str(&json).ok()
}

fn try_extract_common_schema_body_fields(payload: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(payload).ok()?;
    let name = extract_key_value(text, "name")?;
    let body = extract_key_value(text, "message").or_else(|| extract_key_value(text, "body"))?;
    Some((name, body))
}

fn extract_key_value(input: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}=");
    let start = input.find(&pattern)? + pattern.len();
    let rest = &input[start..];
    let end = rest.find(['\n', '\r', ';']).unwrap_or(rest.len());
    let value = rest[..end].trim_matches('"').trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn flatten_json(prefix: &str, value: &Value, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let next = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{}", normalize_key(key))
                };
                flatten_json(&next, value, out);
            }
        }
        Value::Array(array) => {
            if !array.is_empty() {
                out.push((prefix.to_owned(), Value::Array(array.clone()).to_string()));
            }
        }
        Value::String(text) => {
            out.push((prefix.to_owned(), text.clone()));
        }
        Value::Number(number) => {
            out.push((prefix.to_owned(), number.to_string()));
        }
        Value::Bool(boolean) => {
            out.push((prefix.to_owned(), boolean.to_string()));
        }
        Value::Null => {}
    }
}

fn normalize_key(key: &str) -> String {
    let mut result = String::with_capacity(key.len());
    for (index, ch) in key.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn get_json_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        _ => None,
    }
}

fn get_json_i32(value: Option<&Value>) -> Option<i32> {
    value?.as_i64().and_then(|number| i32::try_from(number).ok())
}

fn leak_string(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

fn lookup_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn lookup_path_as_string(value: &Value, path: &str) -> Option<String> {
    get_json_string(lookup_path(value, path))
}

fn lookup_path_as_i32(value: &Value, path: &str) -> Option<i32> {
    get_json_i32(lookup_path(value, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_record_is_base64_encoded() {
        let decoded = DecodedUsereventsRecord::from_raw(
            RawUsereventsRecord {
                tracepoint: "user_events:Example".to_owned(),
                timestamp_unix_nano: 42,
                cpu: 3,
                pid: 100,
                tid: 101,
                sample_id: 9,
                payload: vec![0x41, 0x42, 0x43],
                payload_size: 3,
            },
            &FormatConfig::Raw,
        );

        assert_eq!(decoded.tracepoint, "user_events:Example");
        assert_eq!(decoded.body, "QUJD");
        assert_eq!(decoded.payload_size, 3);
        assert_eq!(decoded.sample_id, 9);
        assert!(decoded.severity_number.is_none());
        assert!(decoded.event_name.is_none());
    }

    #[test]
    fn common_schema_mode_maps_severity_from_tracepoint_level() {
        let decoded = DecodedUsereventsRecord::from_raw(
            RawUsereventsRecord {
                tracepoint: "user_events:myprovider_L2K1".to_owned(),
                timestamp_unix_nano: 42,
                cpu: 1,
                pid: 1,
                tid: 1,
                sample_id: 1,
                payload: vec![0],
                payload_size: 1,
            },
            &FormatConfig::CommonSchemaOtelLogs,
        );

        assert_eq!(decoded.severity_number, Some(17));
        assert_eq!(decoded.severity_text, Some("ERROR"));
        assert!(decoded
            .attributes
            .iter()
            .any(|(key, value)| key == ATTR_PROVIDER && value == "myprovider"));
        assert!(decoded
            .attributes
            .iter()
            .any(|(key, value)| key == ATTR_EVENT_NAME && value == "Log"));
    }

    #[test]
    fn common_schema_mode_extracts_name_and_body_from_utf8_payload() {
        let decoded = DecodedUsereventsRecord::from_raw(
            RawUsereventsRecord {
                tracepoint: "user_events:myprovider_L2K1".to_owned(),
                timestamp_unix_nano: 42,
                cpu: 1,
                pid: 1,
                tid: 1,
                sample_id: 1,
                payload: br#"name=my-event-name;message=This is a test message"#.to_vec(),
                payload_size: 48,
            },
            &FormatConfig::CommonSchemaOtelLogs,
        );

        assert_eq!(decoded.event_name.as_deref(), Some("my-event-name"));
        assert_eq!(decoded.body, "This is a test message");
        assert!(decoded
            .attributes
            .iter()
            .any(|(key, value)| key == ATTR_CS_PART_B_BODY && value == "This is a test message"));
    }

    #[test]
    fn flatten_json_turns_nested_object_into_dot_attributes() {
        let mut out = Vec::new();
        flatten_json(
            "cs",
            &serde_json::json!({
                "PartB": {
                    "name": "evt",
                    "severityNumber": 17
                }
            }),
            &mut out,
        );
        assert!(out
            .iter()
            .any(|(key, value)| key == "cs.part_b.name" && value == "evt"));
        assert!(out
            .iter()
            .any(|(key, value)| key == "cs.part_b.severity_number" && value == "17"));
    }
}
