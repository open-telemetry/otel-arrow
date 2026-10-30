// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encode decoded Windows events into an OTAP log batch.
//!
//! This maps the shared [`WindowsEvent`] model to OTAP Arrow logs. System
//! identifiers become typed `windows.eventlog.*` attributes, the numeric level
//! drives `severity_number`, and `EventData` entries become string or array
//! attributes. The caller supplies the instrumentation scope name and any
//! receiver-specific attributes (for example source identity) through
//! [`MappingContext`]; this module does not acknowledge deliveries, recognize
//! synthetic markers, or know receiver protocols.

use super::model::{Content, Element, Payload, System, WindowsEvent};
use otel_arrow_dfe_pdata::{
    encode::record::{
        attributes::StrKeysAttributesRecordBatchBuilder, logs::LogsRecordBatchBuilder,
    },
    otap::{Logs, OtapArrowRecords},
    proto::opentelemetry::{arrow::v1::ArrowPayloadType, logs::v1::SeverityNumber},
};
use std::collections::BTreeMap;

const ATTR_PREFIX: &str = "windows.eventlog";

/// A receiver-supplied attribute value attached to every mapped record.
pub enum AttrValue<'a> {
    /// A string attribute value.
    Str(&'a str),
    /// A signed 64-bit integer attribute value.
    Int(i64),
}

/// Receiver-specific inputs that are the same for every event in a batch.
pub struct MappingContext<'a> {
    /// Instrumentation scope name recorded on each log record.
    pub scope_name: &'a str,
    /// Extra attributes (for example source identity) added to each record.
    pub extra_attributes: &'a [(&'a str, AttrValue<'a>)],
}

/// Encode a batch of decoded events into an OTAP log record set.
///
/// Returns `Ok(None)` for an empty batch. The caller is responsible for filtering
/// synthetic marker events before calling. `observed_time_unix_nano` is shared by
/// every record.
///
/// # Errors
///
/// Returns a diagnostic string when the batch exceeds `u16::MAX` events or when
/// structured-body serialization or Arrow construction fails.
pub fn encode(
    events: &[WindowsEvent],
    ctx: &MappingContext<'_>,
    observed_time_unix_nano: i64,
) -> Result<Option<OtapArrowRecords>, String> {
    if events.is_empty() {
        return Ok(None);
    }
    if events.len() > usize::from(u16::MAX) {
        return Err("invalid event batch size".into());
    }
    let mut logs = LogsRecordBatchBuilder::new();
    let mut attributes = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    for (index, event) in events.iter().enumerate() {
        let record_id = u16::try_from(index).map_err(|error| error.to_string())?;
        logs.append_id(Some(record_id));
        logs.append_time_unix_nano(event.system.time_created_unix_nano);
        logs.append_observed_time_unix_nano(observed_time_unix_nano);
        logs.append_severity_number(Some(severity_number(event.system.level)));
        let severity_text = event
            .rendering
            .as_ref()
            .and_then(|rendering| rendering.level.as_deref());
        logs.append_severity_text(severity_text.map(str::as_bytes));
        let name = event_name(&event.system);
        logs.append_event_name(Some(name.as_bytes()));
        match event
            .rendering
            .as_ref()
            .and_then(|rendering| rendering.message.as_deref())
        {
            Some(message) => logs.body.append_str(message.as_bytes()),
            None => logs.body.append_map(&structured_body(event)?),
        }
        encode_attributes(&mut attributes, record_id, event, ctx)?;
    }
    let count = events.len();
    logs.append_flags_n(None, count);
    logs.append_trace_id_n(None, count)
        .map_err(|error| error.to_string())?;
    logs.append_span_id_n(None, count)
        .map_err(|error| error.to_string())?;
    logs.resource.append_id_n(0, count);
    logs.resource.append_schema_url_n(None, count);
    logs.resource.append_dropped_attributes_count_n(0, count);
    logs.scope.append_id_n(0, count);
    logs.scope
        .append_name_n(Some(ctx.scope_name.as_bytes()), count);
    logs.scope.append_version_n(None, count);
    logs.scope.append_dropped_attributes_count_n(0, count);
    logs.append_schema_url_n(None, count);
    logs.append_dropped_attributes_count_n(0, count);
    let mut records = OtapArrowRecords::Logs(Logs::default());
    records
        .set(
            ArrowPayloadType::Logs,
            logs.finish().map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    records
        .set(
            ArrowPayloadType::LogAttrs,
            attributes.finish().map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    Ok(Some(records))
}

/// Append one event's attributes: typed `windows.eventlog.*`, context, and EventData.
fn encode_attributes(
    attributes: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    record_id: u16,
    event: &WindowsEvent,
    ctx: &MappingContext<'_>,
) -> Result<(), String> {
    let system = &event.system;
    emit(
        attributes,
        record_id,
        &format!("{ATTR_PREFIX}.event_id"),
        AttrValue::Int(i64::from(system.event_id)),
    );
    emit(
        attributes,
        record_id,
        &format!("{ATTR_PREFIX}.level"),
        AttrValue::Int(i64::from(system.level)),
    );
    emit_opt_str(attributes, record_id, "channel", system.channel.as_deref());
    // EventRecordID is a u64 sequence; emit it as an integer when it fits a signed
    // 64-bit value, otherwise as a decimal string so the value is never dropped.
    if let Some(id) = system.record_id {
        let key = format!("{ATTR_PREFIX}.record_id");
        match i64::try_from(id) {
            Ok(value) => emit(attributes, record_id, &key, AttrValue::Int(value)),
            Err(_) => {
                let decimal = id.to_string();
                emit(attributes, record_id, &key, AttrValue::Str(&decimal));
            }
        }
    }
    if let Some(task) = system.task {
        emit(
            attributes,
            record_id,
            &format!("{ATTR_PREFIX}.task"),
            AttrValue::Int(i64::from(task)),
        );
    }
    if let Some(opcode) = system.opcode {
        emit(
            attributes,
            record_id,
            &format!("{ATTR_PREFIX}.opcode"),
            AttrValue::Int(i64::from(opcode)),
        );
    }
    // Keywords is a 64-bit mask whose high bit exceeds i64; keep its raw hex string.
    emit_opt_str(
        attributes,
        record_id,
        "keywords",
        system.keywords.as_deref(),
    );
    emit_opt_str(
        attributes,
        record_id,
        "computer",
        system.computer.as_deref(),
    );
    emit_opt_str(
        attributes,
        record_id,
        "provider.name",
        system.provider_name.as_deref(),
    );
    emit_opt_str(
        attributes,
        record_id,
        "provider.guid",
        system.provider_guid.as_deref(),
    );
    emit_opt_str(
        attributes,
        record_id,
        "user.sid",
        system.user_sid.as_deref(),
    );
    for (key, value) in ctx.extra_attributes {
        let value = match value {
            AttrValue::Str(text) => AttrValue::Str(text),
            AttrValue::Int(number) => AttrValue::Int(*number),
        };
        emit(attributes, record_id, key, value);
    }
    if let Some(Payload::EventData(event_data)) = &event.payload {
        let mut values_by_name: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for entry in &event_data.entries {
            values_by_name
                .entry(&entry.name)
                .or_default()
                .push(&entry.value);
        }
        for (name, values) in values_by_name {
            attributes.append_key(&format!("{ATTR_PREFIX}.event_data.{name}"));
            attributes.append_parent_id(&record_id);
            if let [single] = values.as_slice() {
                attributes.any_values_builder.append_str(single.as_bytes());
            } else {
                let encoded = serde_cbor::to_vec(&values).map_err(|error| error.to_string())?;
                attributes.any_values_builder.append_slice(&encoded);
            }
        }
    }
    Ok(())
}

/// Append one attribute with a string or integer value.
fn emit(
    attributes: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    record_id: u16,
    key: &str,
    value: AttrValue<'_>,
) {
    attributes.append_key(key);
    match value {
        AttrValue::Str(text) => attributes.any_values_builder.append_str(text.as_bytes()),
        AttrValue::Int(number) => attributes.any_values_builder.append_int(number),
    }
    attributes.append_parent_id(&record_id);
}

/// Append a prefixed string attribute only when a value is present.
fn emit_opt_str(
    attributes: &mut StrKeysAttributesRecordBatchBuilder<u16>,
    record_id: u16,
    suffix: &str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        emit(
            attributes,
            record_id,
            &format!("{ATTR_PREFIX}.{suffix}"),
            AttrValue::Str(value),
        );
    }
}

/// Map a raw Windows level to the OpenTelemetry severity number for its range.
///
/// Levels 1-5 are the predefined Windows severity levels (Critical, Error,
/// Warning, Informational, Verbose) from Winmeta.xml; level 0 is LogAlways and
/// 16-255 are provider-defined. See
/// <https://learn.microsoft.com/en-us/windows/win32/wes/defining-severity-levels>.
fn severity_number(level: u8) -> i32 {
    let severity = match level {
        1 => SeverityNumber::Fatal,       // Windows Critical
        2 => SeverityNumber::Error,       // Windows Error
        3 => SeverityNumber::Warn,        // Windows Warning
        4 => SeverityNumber::Info,        // Windows Informational
        5 => SeverityNumber::Debug,       // Windows Verbose
        _ => SeverityNumber::Unspecified, // Windows LogAlways or custom
    };
    severity as i32
}

/// Build the event name as `<provider>.<event-id>`, or the id alone without a provider.
fn event_name(system: &System) -> String {
    match &system.provider_name {
        Some(provider) => format!("{provider}.{}", system.event_id),
        None => system.event_id.to_string(),
    }
}

/// Serialize the event's payload section as a CBOR map for the structured body.
fn structured_body(event: &WindowsEvent) -> Result<Vec<u8>, String> {
    let mut body = serde_json::Map::new();
    match &event.payload {
        Some(Payload::EventData(event_data)) => {
            let entries: Vec<_> = event_data
                .entries
                .iter()
                .map(|entry| serde_json::json!({"name": entry.name, "value": entry.value}))
                .collect();
            let mut map = serde_json::Map::new();
            let _ = map.insert("entries".into(), entries.into());
            if !event_data.complex.is_empty() {
                let complex: Vec<_> = event_data.complex.iter().map(element_json).collect();
                let _ = map.insert("complex".into(), complex.into());
            }
            if let Some(binary) = &event_data.binary {
                let _ = map.insert("binary".into(), binary.clone().into());
            }
            let _ = body.insert("event_data".into(), map.into());
        }
        Some(Payload::UserData(element)) => {
            let _ = body.insert("user_data".into(), element_json(element));
        }
        Some(Payload::Other(element)) => {
            let _ = body.insert("other".into(), element_json(element));
        }
        None => {}
    }
    serde_cbor::to_vec(&body).map_err(|error| error.to_string())
}

/// Preserve element/attribute names, namespaces, and ordered mixed content as JSON.
fn element_json(element: &Element) -> serde_json::Value {
    let attributes: Vec<_> = element
        .attributes
        .iter()
        .map(|attribute| {
            serde_json::json!({
                "name": attribute.name,
                "namespace": attribute.namespace.as_deref().unwrap_or_default(),
                "value": attribute.value,
            })
        })
        .collect();
    let content: Vec<_> = element
        .content
        .iter()
        .map(|item| match item {
            Content::Element(child) => element_json(child),
            Content::Text(text) => serde_json::Value::String(text.clone()),
        })
        .collect();
    serde_json::json!({
        "name": element.name,
        "namespace": element.namespace.as_deref().unwrap_or_default(),
        "attributes": attributes,
        "content": content,
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::xml::Document;
    use super::super::parse::from_document;
    use super::*;
    use arrow::array::{Array, Int32Array, Int64Array, StringArray, UInt16Array};
    use arrow::compute::cast;
    use arrow::datatypes::DataType;

    fn parse(xml: &str) -> WindowsEvent {
        from_document(&Document::parse(xml).unwrap()).unwrap()
    }

    /// Scenario: severity derives only from the numeric Windows level.
    /// Guarantees: standard levels map to OpenTelemetry range bases and unknown levels are UNSPECIFIED.
    #[test]
    fn maps_severity_from_level() {
        assert_eq!(
            [1, 2, 3, 4, 5, 0, 9].map(severity_number),
            [
                SeverityNumber::Fatal as i32,
                SeverityNumber::Error as i32,
                SeverityNumber::Warn as i32,
                SeverityNumber::Info as i32,
                SeverityNumber::Debug as i32,
                SeverityNumber::Unspecified as i32,
                SeverityNumber::Unspecified as i32,
            ]
        );
    }

    /// Scenario: the event name is built from provider and event id.
    /// Guarantees: a present provider prefixes the id, and an absent provider yields the id alone.
    #[test]
    fn builds_event_name() {
        let with_provider = parse(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><Provider Name='Svc'/><EventID>42</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System></Event>",
        );
        assert_eq!(event_name(&with_provider.system), "Svc.42");
        let without_provider = parse(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'><System><EventID>42</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System></Event>",
        );
        assert_eq!(event_name(&without_provider.system), "42");
    }

    /// Scenario: an event without a rendered message falls back to a structured payload body.
    /// Guarantees: the CBOR body preserves EventData entries, ComplexData, and the Binary payload.
    #[test]
    fn serializes_structured_body() {
        let event = parse(concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/></System>",
            "<EventData><Data Name='a'>one</Data><ComplexData><Child>nested</Child></ComplexData><Binary>0102FF</Binary></EventData>",
            "</Event>",
        ));
        let body: serde_json::Value =
            serde_cbor::from_slice(&structured_body(&event).unwrap()).unwrap();
        assert_eq!(body["event_data"]["entries"][0]["name"], "a");
        assert_eq!(body["event_data"]["entries"][0]["value"], "one");
        assert_eq!(body["event_data"]["complex"][0]["name"], "ComplexData");
        assert_eq!(
            body["event_data"]["complex"][0]["content"][0]["name"],
            "Child"
        );
        assert_eq!(body["event_data"]["binary"], "0102FF");
    }

    /// Scenario: an EventRecordID above i64::MAX is mapped.
    /// Guarantees: the value is preserved as a decimal string attribute, never dropped.
    #[test]
    fn preserves_large_record_id() {
        let event = parse(concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System><EventID>1</EventID><TimeCreated SystemTime='2026-09-22T19:28:11Z'/>",
            "<EventRecordID>18446744073709551615</EventRecordID></System></Event>",
        ));
        let ctx = MappingContext {
            scope_name: "scope",
            extra_attributes: &[],
        };
        let records = encode(&[event], &ctx, 0).unwrap().unwrap();
        let attrs = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let keys = cast(attrs.column_by_name("key").unwrap(), &DataType::Utf8).unwrap();
        let keys = keys.as_any().downcast_ref::<StringArray>().unwrap();
        let strings = cast(attrs.column_by_name("str").unwrap(), &DataType::Utf8).unwrap();
        let strings = strings.as_any().downcast_ref::<StringArray>().unwrap();
        let mut record_id = None;
        for index in 0..attrs.num_rows() {
            if keys.value(index) == "windows.eventlog.record_id" && !strings.is_null(index) {
                record_id = Some(strings.value(index).to_owned());
            }
        }
        assert_eq!(record_id.as_deref(), Some("18446744073709551615"));
    }

    /// Scenario: an empty batch is encoded.
    /// Guarantees: no records are produced for an empty batch.
    #[test]
    fn empty_batch_returns_none() {
        let ctx = MappingContext {
            scope_name: "scope",
            extra_attributes: &[],
        };
        assert!(encode(&[], &ctx, 0).unwrap().is_none());
    }

    /// Scenario: a batch of events is encoded with typed system attributes and context attributes.
    /// Guarantees: severity, event name, typed `windows.eventlog.*` attributes, repeated EventData
    /// arrays, and caller-supplied attributes are present on each record.
    #[test]
    fn encodes_event_batch() {
        let event = parse(concat!(
            "<Event xmlns='http://schemas.microsoft.com/win/2004/08/events/event'>",
            "<System><Provider Name='Example'/><EventID>42</EventID><Level>2</Level>",
            "<TimeCreated SystemTime='2026-09-22T19:28:11Z'/><Channel>Application</Channel>",
            "<EventRecordID>99</EventRecordID></System>",
            "<EventData><Data Name='name'>value</Data><Data Name='dup'>a</Data><Data Name='dup'>b</Data></EventData>",
            "</Event>",
        ));
        let ctx = MappingContext {
            scope_name: "windows_event_forwarding",
            extra_attributes: &[("source.principal", AttrValue::Str("host"))],
        };
        let records = encode(&[event.clone(), event], &ctx, 123).unwrap().unwrap();

        let logs = records.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs.num_rows(), 2);
        let severity = cast(
            logs.column_by_name("severity_number").unwrap(),
            &DataType::Int32,
        )
        .unwrap();
        assert_eq!(
            severity
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap()
                .values()
                .as_ref(),
            &[17, 17]
        );
        let event_names =
            cast(logs.column_by_name("event_name").unwrap(), &DataType::Utf8).unwrap();
        assert_eq!(
            event_names
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .value(0),
            "Example.42"
        );

        let attrs = records.get(ArrowPayloadType::LogAttrs).unwrap();
        let keys = cast(attrs.column_by_name("key").unwrap(), &DataType::Utf8).unwrap();
        let keys = keys.as_any().downcast_ref::<StringArray>().unwrap();
        let strings = cast(attrs.column_by_name("str").unwrap(), &DataType::Utf8).unwrap();
        let strings = strings.as_any().downcast_ref::<StringArray>().unwrap();
        let ints = cast(attrs.column_by_name("int").unwrap(), &DataType::Int64).unwrap();
        let ints = ints.as_any().downcast_ref::<Int64Array>().unwrap();
        let parents = attrs
            .column_by_name("parent_id")
            .unwrap()
            .as_any()
            .downcast_ref::<UInt16Array>()
            .unwrap();

        let mut record0_int = std::collections::HashMap::new();
        let mut record0_str = std::collections::HashMap::new();
        let mut dup_is_array = false;
        for index in 0..attrs.num_rows() {
            if parents.value(index) != 0 {
                continue;
            }
            let key = keys.value(index);
            if !strings.is_null(index) {
                let _ = record0_str.insert(key.to_owned(), strings.value(index).to_owned());
            } else if !ints.is_null(index) {
                let _ = record0_int.insert(key.to_owned(), ints.value(index));
            } else if key == "windows.eventlog.event_data.dup" {
                dup_is_array = true;
            }
        }
        assert_eq!(record0_int.get("windows.eventlog.event_id"), Some(&42));
        assert_eq!(record0_int.get("windows.eventlog.level"), Some(&2));
        assert_eq!(record0_int.get("windows.eventlog.record_id"), Some(&99));
        assert_eq!(
            record0_str
                .get("windows.eventlog.channel")
                .map(String::as_str),
            Some("Application")
        );
        assert_eq!(
            record0_str
                .get("windows.eventlog.provider.name")
                .map(String::as_str),
            Some("Example")
        );
        assert_eq!(
            record0_str
                .get("windows.eventlog.event_data.name")
                .map(String::as_str),
            Some("value")
        );
        assert_eq!(
            record0_str.get("source.principal").map(String::as_str),
            Some("host")
        );
        assert!(
            dup_is_array,
            "repeated EventData name should be an array attribute"
        );
        assert!(
            !record0_str
                .keys()
                .chain(record0_int.keys())
                .any(|key| key.starts_with("winlog."))
        );
    }
}
